use std::{ops::Range, path::Path};

use tree_sitter::{Node, Parser, Tree};

use crate::{
    document_kind::{DocumentKind, DocumentProfile},
    host_injections::{
        InjectionCandidate, InjectionDecode, InjectionProjection,
        InjectionProjectionSegment as Segment, InjectionVisualKind,
    },
    language_aliases::normalize_language_name,
    semantic_overlays::SemanticCaptureSpan,
};

pub(crate) fn document_kind(path: Option<&Path>, source: &str) -> DocumentKind {
    if path
        .and_then(Path::file_name)
        .and_then(|name| name.to_str())
        .is_some_and(|name| matches!(name, "hk.pkl" | "hk.local.pkl"))
        || hk_schema_source(source)
    {
        DocumentKind::with_profile("pkl", DocumentProfile::HkConfig)
    } else {
        DocumentKind::plain("pkl")
    }
}

fn hk_schema_source(source: &str) -> bool {
    // Only inspect the module header. Import strings and comments are not schemas.
    let mut parser = Parser::new();
    if parser
        .set_language(&tree_sitter_pkl::LANGUAGE.into())
        .is_err()
    {
        return false;
    }
    parser
        .parse(source, None)
        .is_some_and(|tree| hk_schema(tree.root_node(), source))
}

fn hk_schema(root: Node<'_>, source: &str) -> bool {
    children(root)
        .filter(|node| node.kind() == "moduleHeader")
        .flat_map(children)
        .filter(|node| node.kind() == "extendsOrAmendsClause")
        .flat_map(children)
        .filter(|node| node.kind() == "stringConstant")
        .filter_map(|node| literal_text(node, source))
        .any(|uri| {
            uri == "hk.pkl"
                || uri.ends_with("/hk.pkl")
                || (uri.starts_with("package://github.com/jdx/hk/")
                    && uri.ends_with("#/Config.pkl"))
        })
}

pub(crate) fn injections(kind: DocumentKind, tree: &Tree, source: &str) -> Vec<InjectionCandidate> {
    let hk = kind.profile() == DocumentProfile::HkConfig || hk_schema(tree.root_node(), source);
    let mut result = Vec::new();
    visit(tree.root_node(), &mut |node| {
        if node.kind() == "docComment" {
            let mut segments = Vec::new();
            let mut ranges = Vec::new();
            let mut start = node.start_byte();
            for line in source[node.byte_range()].split_inclusive('\n') {
                if let Some(prefix) = line.find("///") {
                    let content = start + prefix + 3;
                    let content =
                        content + usize::from(line.as_bytes().get(prefix + 3) == Some(&b' '));
                    let range = content..start + line.len();
                    ranges.push(range.clone());
                    segments.push(Segment::Source(range));
                }
                start += line.len();
            }
            if !ranges.is_empty() {
                result.push(candidate("markdown", ranges, segments, false));
            }
        }
        if node.kind() == "unqualifiedAccessExpr"
            && children(node)
                .next()
                .is_some_and(|name| &source[name.byte_range()] == "Regex")
            && let Some(args) = children(node).find(|child| child.kind() == "argumentList")
            && let Some(value) = children(args).next().filter(|child| is_literal(*child))
            && let Some((range, segments)) = project_literal(value, source, false)
        {
            result.push(candidate("regex", vec![range], segments, false));
        }
        if hk
            && let Some(command) = command_literal(node, source)
            && let Some(language) = command.language
            && let Some((range, segments)) = project_literal(node, source, true)
        {
            result.push(candidate(
                language,
                vec![range],
                segments,
                node.kind() == "mlStringLiteralExpr",
            ));
        }
    });
    result
}

pub(crate) fn semantic_spans(
    profile: DocumentProfile,
    tree: &Tree,
    source: &str,
) -> Vec<SemanticCaptureSpan> {
    if profile != DocumentProfile::HkConfig && !hk_schema(tree.root_node(), source) {
        return Vec::new();
    }
    let mut spans = Vec::new();
    visit(tree.root_node(), &mut |node| {
        if command_literal(node, source).is_none() {
            return;
        }
        for part in children(node)
            .filter(|part| matches!(part.kind(), "slStringLiteralPart" | "mlStringLiteralPart"))
        {
            for range in template_ranges(part.byte_range(), source) {
                spans.push(SemanticCaptureSpan {
                    range: range.start..range.start + 2,
                    capture: "punctuation.special",
                });
                spans.push(SemanticCaptureSpan {
                    range: range.start + 2..range.end - 2,
                    capture: "variable.builtin",
                });
                spans.push(SemanticCaptureSpan {
                    range: range.end - 2..range.end,
                    capture: "punctuation.special",
                });
            }
        }
    });
    spans
}

fn candidate(
    language: &'static str,
    ranges: Vec<Range<usize>>,
    segments: Vec<Segment>,
    block: bool,
) -> InjectionCandidate {
    InjectionCandidate {
        document_kind: DocumentKind::plain(language),
        ranges,
        projection: InjectionProjection::TemplateSegments(segments),
        is_combined: false,
        strip_shared_indent: false,
        merge_parent_styles: false,
        decode: InjectionDecode::None,
        highlight_github_expressions: false,
        visual_kind: Some(if block {
            InjectionVisualKind::RectBlock
        } else {
            InjectionVisualKind::Transparent
        }),
        visual_level_bump: None,
        visual_anchor: None,
    }
}

struct Command {
    language: Option<&'static str>,
}

fn command_literal(node: Node<'_>, source: &str) -> Option<Command> {
    if !is_literal(node) {
        return None;
    }
    let property = node.parent()?;
    let key = property_name(property, source)?;
    let (step, platform) = if matches!(key, "check" | "fix" | "check_list_files" | "check_diff") {
        (property.parent()?, None)
    } else if matches!(key, "linux" | "macos" | "windows" | "other") {
        let script_body = property.parent()?;
        let script = script_body.parent()?;
        if script.kind() != "newExpr" || !type_mentions(script, source, "Script") {
            return None;
        }
        let command = script.parent()?;
        if !matches!(
            property_name(command, source)?,
            "check" | "fix" | "check_list_files" | "check_diff"
        ) {
            return None;
        }
        (command.parent()?, Some(key))
    } else {
        return None;
    };

    if !is_step_body(step, source) {
        return None;
    }
    let shell = children(step).find(|child| property_name(*child, source) == Some("shell"));
    let language = match shell {
        Some(shell) => children(shell)
            .find(|child| is_literal(*child))
            .and_then(|value| literal_text(value, source))
            .and_then(|shell| shell_language(&shell)),
        None if platform == Some("windows") => Some("batch"),
        None => Some("bash"),
    };
    Some(Command { language })
}

fn is_step_body(mut node: Node<'_>, source: &str) -> bool {
    while let Some(parent) = node.parent() {
        if property_name(parent, source) == Some("steps") {
            return true;
        }
        // A property nested inside a step (for example env.check) is not a
        // command sink. Only the step's own object body carries shell settings.
        if property_name(parent, source).is_some() {
            return false;
        }
        // AST types distinguish Step and typed step mappings from arbitrary objects.
        if parent.kind() == "newExpr" && type_mentions(parent, source, "Step") {
            return true;
        }
        node = parent;
    }
    false
}

fn type_mentions(node: Node<'_>, source: &str, name: &str) -> bool {
    let mut found = false;
    for child in children(node).filter(|child| child.kind() != "objectBody") {
        visit(child, &mut |part| {
            found |=
                part.kind() == "identifier" && source[part.byte_range()].trim_matches('`') == name;
        });
    }
    found
}

fn shell_language(shell: &str) -> Option<&'static str> {
    let name = normalize_language_name(shell)?;
    match name {
        "bash" | "ash" | "dash" | "ksh" | "mksh" => Some("bash"),
        "zsh" => Some("zsh"),
        "fish" => Some("fish"),
        "powershell" => Some("powershell"),
        "batch" => Some("batch"),
        "python" => Some("python"),
        _ => None,
    }
}

fn property_name<'a>(node: Node<'_>, source: &'a str) -> Option<&'a str> {
    if !matches!(node.kind(), "classProperty" | "objectProperty") {
        return None;
    }
    children(node)
        .find(|child| child.kind() == "identifier")
        .map(|name| source[name.byte_range()].trim_matches('`'))
}

fn is_literal(node: Node<'_>) -> bool {
    matches!(
        node.kind(),
        "slStringLiteralExpr" | "mlStringLiteralExpr" | "stringConstant"
    )
}

fn literal_text(node: Node<'_>, source: &str) -> Option<String> {
    let (_, segments) = project_literal(node, source, false)?;
    let mut text = String::new();
    for segment in segments {
        match segment {
            Segment::Source(range) => text.push_str(&source[range]),
            Segment::MappedText { text: value, .. } => text.push_str(&value),
            Segment::Synthetic(_) => return None,
        }
    }
    Some(text)
}

// Project AST string parts, retaining exact UTF-8 source mappings for decoded escapes.
// Dynamic expressions and hk templates become unmapped placeholders so their Pkl
// or hk styles survive instead of being overwritten by the injected language.
fn project_literal(node: Node<'_>, source: &str, hk: bool) -> Option<(Range<usize>, Vec<Segment>)> {
    let raw = &source[node.byte_range()];
    let hashes = raw.bytes().take_while(|byte| *byte == b'#').count();
    let quotes = if raw.get(hashes..)?.starts_with("\"\"\"") {
        3
    } else {
        1
    };
    let delimiter = hashes + quotes;
    if raw.len() < 2 * delimiter || node.has_error() {
        return None;
    }
    let mut content = node.start_byte() + delimiter..node.end_byte() - delimiter;
    let mut indent = "";
    if quotes == 3 {
        let first_newline = source[content.clone()].find('\n')?;
        content.start += first_newline + 1;
        let last_newline = source[content.clone()].rfind('\n');
        // Empty multiline literals have only the closing delimiter's indentation.
        let closing_line = last_newline.map_or(content.start, |offset| content.start + offset + 1);
        indent = &source[closing_line..content.end];
        if !indent.chars().all(|ch| matches!(ch, ' ' | '\t')) {
            return None;
        }
        content.end = last_newline.map_or(content.start, |offset| content.start + offset);
        if source.as_bytes().get(content.end.wrapping_sub(1)) == Some(&b'\r') {
            content.end -= 1;
        }
    }
    let mut segments = Vec::new();
    for part in children(node) {
        let range = part.start_byte().max(content.start)..part.end_byte().min(content.end);
        if range.is_empty() {
            continue;
        }
        match part.kind() {
            "slStringLiteralPart" | "mlStringLiteralPart" => {
                let ranges = if hk {
                    template_ranges(range.clone(), source)
                } else {
                    Vec::new()
                };
                let mut start = range.start;
                for template in ranges {
                    append_literal_part(
                        start..template.start,
                        content.start,
                        indent,
                        source,
                        &mut segments,
                    );
                    segments.push(Segment::Synthetic("kat_value".into()));
                    start = template.end;
                }
                append_literal_part(
                    start..range.end,
                    content.start,
                    indent,
                    source,
                    &mut segments,
                );
            }
            "stringInterpolation" => segments.push(Segment::Synthetic("kat_value".into())),
            "escapeSequence" => {
                let escape = source[range.clone()]
                    .trim_start_matches('\\')
                    .trim_start_matches('#');
                let text = match escape {
                    "n" => "\n".into(),
                    "r" => "\r".into(),
                    "t" => "\t".into(),
                    "\\" => "\\".into(),
                    "\"" => "\"".into(),
                    _ => {
                        let code = escape.strip_prefix("u{")?.strip_suffix('}')?;
                        char::from_u32(u32::from_str_radix(code, 16).ok()?)?.to_string()
                    }
                };
                segments.push(Segment::MappedText {
                    text,
                    source: range,
                });
            }
            _ => {}
        }
    }
    Some((content, segments))
}

fn append_literal_part(
    range: Range<usize>,
    content_start: usize,
    indent: &str,
    source: &str,
    segments: &mut Vec<Segment>,
) {
    let mut start = range.start;
    for line in source[range.clone()].split_inclusive('\n') {
        let mut end = start + line.len();
        let at_line_start =
            start == content_start || source.as_bytes().get(start.wrapping_sub(1)) == Some(&b'\n');
        let trimmed = if at_line_start && line.starts_with(indent) {
            start + indent.len()
        } else {
            start
        };
        let crlf = line.ends_with("\r\n");
        if crlf {
            end -= 2;
        }
        if trimmed < end {
            segments.push(Segment::Source(trimmed..end));
        }
        if crlf {
            segments.push(Segment::MappedText {
                text: "\n".into(),
                source: end..end + 2,
            });
        }
        start += line.len();
    }
}

fn template_ranges(range: Range<usize>, source: &str) -> Vec<Range<usize>> {
    let mut ranges = Vec::new();
    let mut start = range.start;
    while let Some(open) = source[start..range.end].find("{{") {
        let open = start + open;
        let Some(close) = source[open + 2..range.end].find("}}") else {
            break;
        };
        let end = open + 2 + close + 2;
        ranges.push(open..end);
        start = end;
    }
    ranges
}

fn children(node: Node<'_>) -> impl Iterator<Item = Node<'_>> {
    (0..node.named_child_count()).filter_map(move |index| node.named_child(index as u32))
}

fn visit(node: Node<'_>, visitor: &mut impl FnMut(Node<'_>)) {
    visitor(node);
    for child in children(node) {
        visit(child, visitor);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(source: &str) -> Tree {
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_pkl::LANGUAGE.into())
            .unwrap();
        parser.parse(source, None).unwrap()
    }

    #[test]
    fn pkl_fixtures_parse_without_recovery_nodes() {
        for source in [
            include_str!("../testdata/fixtures/pkl/config.pkl"),
            include_str!("../testdata/fixtures/pkl/hk.pkl"),
            include_str!("../testdata/fixtures/pkl/hk.local.pkl"),
        ] {
            let tree = parse(source);
            assert!(
                !tree.root_node().has_error(),
                "{}",
                tree.root_node().to_sexp()
            );
        }
    }

    #[test]
    fn projected_pkl_literals_match_decoded_values_and_source_ranges() {
        for (literal, expected) in [
            (r#""echo \"blue\"\nready""#, "echo \"blue\"\nready"),
            (r###"##"literal \n then \##n"##"###, "literal \\n then \n"),
            ("\"\"\"\r\n\tleft\r\n\tright\r\n\t\"\"\"", "left\nright"),
            ("\"\"\"\n  \"\"\"", ""),
            (r#""echo \(label) {{files}}""#, "echo kat_value kat_value"),
            (r#""\u{1f7e3}""#, "🟣"),
        ] {
            let source = format!("value = {literal}");
            let tree = parse(&source);
            assert!(!tree.root_node().has_error(), "{literal}");
            let property = children(tree.root_node()).last().unwrap();
            let node = children(property).find(|node| is_literal(*node)).unwrap();
            let (range, segments) = project_literal(node, &source, true).unwrap();
            let injection = candidate("bash", vec![range], segments, false);
            let (text, map) = crate::build_candidate_virtual_source(&source, &injection);
            assert_eq!(text, expected, "{literal}");
            assert_eq!(map.len(), text.len());
            assert!(map.iter().all(|range| range.end <= source.len()));
            if text == "🟣" {
                assert!(
                    map.iter()
                        .all(|range| &source[range.clone()] == r"\u{1f7e3}")
                );
            }
        }
    }
}
