//! Structural bracket pairing and depth-based foreground decoration.

use std::ops::Range;

use tree_sitter::{QueryCursor, StreamingIterator, Tree};

use crate::{StyledSpan, language_runtime::LanguageRuntime, theme::Theme};

#[derive(Debug, Eq, PartialEq)]
struct BracketPair {
    open: Range<usize>,
    close: Range<usize>,
}

pub(crate) fn styled_brackets(
    runtime: &LanguageRuntime,
    tree: &Tree,
    source: &str,
    spans: &[StyledSpan],
    theme: &Theme,
) -> Vec<StyledSpan> {
    let Some(query) = runtime.brackets_query.as_ref() else {
        return Vec::new();
    };
    let names = query.capture_names();
    let mut cursor = QueryCursor::new();
    let mut matches = cursor.matches(query, tree.root_node(), source.as_bytes());
    let mut pairs = Vec::new();
    while let Some(matched) = matches.next() {
        let mut open = None;
        let mut close = None;
        let mut valid = true;
        for capture in matched.captures() {
            let node = capture.node;
            // Recovery nodes must not manufacture a bracket pair. Complete inner
            // structures in an otherwise incomplete document remain usable.
            if node.is_missing() || node.parent().is_some_and(|parent| parent.is_error()) {
                valid = false;
                continue;
            }
            match names[capture.index as usize] {
                "open" => open = Some(node.byte_range()),
                "close" => close = Some(node.byte_range()),
                _ => {}
            }
        }
        if let (Some(open), Some(close)) = (open, close)
            && valid
            && !open.is_empty()
            && !close.is_empty()
            && open.end <= close.start
        {
            pairs.push(BracketPair { open, close });
        }
    }
    pairs.sort_by(|a, b| {
        a.open
            .start
            .cmp(&b.open.start)
            .then(b.close.end.cmp(&a.close.end))
    });
    pairs.dedup();

    let mut enclosing: Vec<usize> = Vec::new();
    let mut brackets = Vec::with_capacity(pairs.len() * 2);
    for pair in pairs {
        while enclosing.last().is_some_and(|end| *end <= pair.open.start) {
            enclosing.pop();
        }
        // Ambiguous/crossing ranges do not have a well-defined nesting depth.
        if enclosing.last().is_some_and(|end| pair.close.end > *end) {
            continue;
        }
        let depth = enclosing.len();
        enclosing.push(pair.close.end);
        brackets.push((pair.open, depth));
        brackets.push((pair.close, depth));
    }
    brackets.sort_by_key(|(range, _)| range.start);

    let mut span_index = 0;
    let mut overlays = Vec::with_capacity(brackets.len());
    for (range, depth) in brackets {
        // A multi-byte delimiter can cross style boundaries. Recolor each
        // existing segment to retain its background, effects and link semantics.
        while span_index < spans.len() && spans[span_index].range.end <= range.start {
            span_index += 1;
        }
        for span in &spans[span_index..] {
            if span.range.start >= range.end {
                break;
            }
            let start = range.start.max(span.range.start);
            let end = range.end.min(span.range.end);
            if start < end {
                overlays.push(StyledSpan {
                    range: start..end,
                    style: theme.bracket_style(span.style, depth),
                    hyperlink: span.hyperlink.clone(),
                });
            }
        }
    }
    overlays
}
