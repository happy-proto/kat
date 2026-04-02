use std::ops::Range;

use anyhow::{Context, Result};
use tree_sitter::{Node, Query, QueryCursor, StreamingIterator, Tree};

use crate::{
    document_kind::{DocumentKind, DocumentProfile},
    language_aliases::normalize_language_name,
    language_runtime::{LanguageRuntime, runtime},
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum InjectionDecode {
    None,
    JavaScriptLiteral,
    JavaScriptString,
    PythonString,
    RustString,
    GoString,
}

#[derive(Debug)]
pub(crate) struct InjectionCandidate {
    pub(crate) ranges: Vec<Range<usize>>,
    pub(crate) language_name: String,
    pub(crate) is_combined: bool,
    pub(crate) merge_parent_styles: bool,
    pub(crate) decode: InjectionDecode,
}

pub(crate) fn collect_injection_candidates(
    document_kind: DocumentKind,
    language_runtime: &LanguageRuntime,
    tree: &Tree,
    source: &str,
) -> Result<Vec<InjectionCandidate>> {
    let mut candidates = collect_query_injection_candidates(
        language_runtime,
        tree,
        source,
        document_kind.runtime_name(),
    )?;
    candidates.extend(collect_host_injection_candidates(
        document_kind,
        language_runtime,
        tree,
        source,
    )?);
    Ok(candidates)
}

fn collect_query_injection_candidates(
    language_runtime: &LanguageRuntime,
    tree: &Tree,
    source: &str,
    runtime_name: &str,
) -> Result<Vec<InjectionCandidate>> {
    if language_runtime.injections_query.trim().is_empty() {
        return Ok(Vec::new());
    }

    let query = Query::new(
        &language_runtime.language,
        language_runtime.injections_query,
    )
    .with_context(|| format!("failed to compile injections query for {runtime_name}"))?;
    let capture_names = query.capture_names();
    let mut cursor = QueryCursor::new();
    let mut candidates = Vec::new();

    let mut matches = cursor.matches(&query, tree.root_node(), source.as_bytes());
    while {
        matches.advance();
        matches.get().is_some()
    } {
        let query_match = matches
            .get()
            .expect("query match should exist immediately after advance");
        let mut injection_language = query
            .property_settings(query_match.pattern_index)
            .iter()
            .find(|property| property.key.as_ref() == "injection.language")
            .and_then(|property| property.value.as_deref())
            .and_then(normalize_language_name)
            .map(str::to_owned);

        let injection_combined = query
            .property_settings(query_match.pattern_index)
            .iter()
            .any(|property| property.key.as_ref() == "injection.combined");
        let merge_parent_styles = query
            .property_settings(query_match.pattern_index)
            .iter()
            .any(|property| property.key.as_ref() == "kat.merge-parent");
        let decode = query
            .property_settings(query_match.pattern_index)
            .iter()
            .find(|property| property.key.as_ref() == "kat.decode")
            .and_then(|property| property.value.as_deref())
            .map(InjectionDecode::from_query_value)
            .unwrap_or(InjectionDecode::None);
        let mut content_ranges = Vec::new();

        for capture in query_match.captures {
            let capture_name = capture_names[capture.index as usize];
            match capture_name {
                "injection.language" if injection_language.is_none() => {
                    let node_text = &source[capture.node.byte_range()];
                    injection_language = normalize_language_name(node_text).map(str::to_owned);
                }
                "injection.content" => content_ranges.push(capture.node.byte_range()),
                _ => {}
            }
        }

        let Some(injection_language) = injection_language else {
            continue;
        };

        if runtime(&injection_language).is_none() || content_ranges.is_empty() {
            continue;
        }

        if injection_combined {
            candidates.push(InjectionCandidate {
                ranges: normalize_ranges(content_ranges),
                language_name: injection_language,
                is_combined: true,
                merge_parent_styles,
                decode,
            });
            continue;
        }

        for range in content_ranges {
            if range.start < range.end {
                candidates.push(InjectionCandidate {
                    ranges: vec![range],
                    language_name: injection_language.clone(),
                    is_combined: false,
                    merge_parent_styles,
                    decode,
                });
            }
        }
    }

    Ok(candidates)
}

fn collect_host_injection_candidates(
    document_kind: DocumentKind,
    language_runtime: &LanguageRuntime,
    tree: &Tree,
    source: &str,
) -> Result<Vec<InjectionCandidate>> {
    match (document_kind.runtime_name(), document_kind.profile()) {
        ("dockerfile", _) => {
            collect_dockerfile_injection_candidates(language_runtime, tree, source)
        }
        (
            "yaml",
            DocumentProfile::GitHubActionsWorkflow | DocumentProfile::GitHubActionMetadata,
        ) => Ok(collect_github_actions_yaml_injection_candidates(
            tree, source,
        )),
        _ => Ok(Vec::new()),
    }
}

fn collect_dockerfile_injection_candidates(
    language_runtime: &LanguageRuntime,
    tree: &Tree,
    source: &str,
) -> Result<Vec<InjectionCandidate>> {
    let query = Query::new(
        &language_runtime.language,
        language_runtime.injections_query,
    )
    .context("failed to compile dockerfile injections query")?;
    let capture_names = query.capture_names();
    let mut cursor = QueryCursor::new();
    let mut current_shell = String::from("bash");
    let mut candidates = Vec::new();

    let mut matches = cursor.matches(&query, tree.root_node(), source.as_bytes());
    while {
        matches.advance();
        matches.get().is_some()
    } {
        let query_match = matches
            .get()
            .expect("query match should exist immediately after advance");
        let mut matched_shell = None;
        let mut content_ranges = Vec::new();

        for capture in query_match.captures {
            let capture_name = capture_names[capture.index as usize];
            match capture_name {
                "shell.language" if matched_shell.is_none() => {
                    let raw = &source[capture.node.byte_range()];
                    matched_shell = normalize_language_name(raw).map(str::to_owned);
                }
                "shell.content" => content_ranges.push(capture.node.byte_range()),
                _ => {}
            }
        }

        if let Some(shell) = matched_shell {
            if runtime(&shell).is_some() {
                current_shell = shell;
            }
        }

        if runtime(&current_shell).is_none() {
            continue;
        }

        for range in content_ranges {
            if range.start < range.end {
                candidates.push(InjectionCandidate {
                    ranges: vec![range],
                    language_name: current_shell.clone(),
                    is_combined: false,
                    merge_parent_styles: false,
                    decode: InjectionDecode::None,
                });
            }
        }
    }

    Ok(candidates)
}

fn collect_github_actions_yaml_injection_candidates(
    tree: &Tree,
    source: &str,
) -> Vec<InjectionCandidate> {
    let mut candidates = Vec::new();

    walk_tree(tree.root_node(), &mut |node| {
        if node.kind() != "block_mapping" {
            return;
        }

        let mut shell_language = None;
        let mut run_ranges = Vec::new();

        for pair in named_children(node) {
            if pair.kind() != "block_mapping_pair" {
                continue;
            }

            let Some(key) = pair.child_by_field_name("key") else {
                continue;
            };
            let Some(value) = pair.child_by_field_name("value") else {
                continue;
            };
            let Some(key_text) = yaml_scalar_text(key, source) else {
                continue;
            };

            match key_text {
                "shell" => {
                    shell_language = yaml_scalar_text(value, source)
                        .and_then(normalize_language_name)
                        .map(str::to_owned);
                }
                "run" => {
                    if let Some(range) = yaml_injection_content_range(value, source) {
                        run_ranges.push(range);
                    }
                }
                _ => {}
            }
        }

        if run_ranges.is_empty() {
            return;
        }

        let language_name = shell_language
            .filter(|shell| runtime(shell).is_some())
            .unwrap_or_else(|| String::from("bash"));

        if runtime(&language_name).is_none() {
            return;
        }

        for range in run_ranges {
            candidates.push(InjectionCandidate {
                ranges: vec![range],
                language_name: language_name.clone(),
                is_combined: true,
                merge_parent_styles: false,
                decode: InjectionDecode::None,
            });
        }
    });

    candidates
}

fn yaml_injection_content_range(node: Node, source: &str) -> Option<Range<usize>> {
    match node.kind() {
        "block_node" | "flow_node" => {
            named_children(node).find_map(|child| yaml_injection_content_range(child, source))
        }
        "plain_scalar" | "string_scalar" | "double_quote_scalar" | "single_quote_scalar" => {
            Some(node.byte_range())
        }
        "block_scalar" => {
            let range = node.byte_range();
            let text = &source[range.clone()];
            let Some(line_break) = text.find('\n') else {
                return Some(range);
            };
            let content_start = range.start + line_break + 1;
            Some(content_start..range.end)
        }
        _ => named_children(node).find_map(|child| yaml_injection_content_range(child, source)),
    }
}

fn yaml_scalar_text<'a>(node: Node, source: &'a str) -> Option<&'a str> {
    match node.kind() {
        "block_node" | "flow_node" => {
            named_children(node).find_map(|child| yaml_scalar_text(child, source))
        }
        "plain_scalar" | "string_scalar" | "double_quote_scalar" | "single_quote_scalar" => {
            Some(source[node.byte_range()].trim_matches(|ch| matches!(ch, '"' | '\'')))
        }
        _ => named_children(node).find_map(|child| yaml_scalar_text(child, source)),
    }
}

fn named_children(node: Node) -> impl Iterator<Item = Node> {
    (0..node.named_child_count()).filter_map(move |index| node.named_child(index as u32))
}

fn walk_tree(node: Node, visit: &mut impl FnMut(Node)) {
    visit(node);

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        walk_tree(child, visit);
    }
}

fn normalize_ranges(mut ranges: Vec<Range<usize>>) -> Vec<Range<usize>> {
    ranges.retain(|range| range.start < range.end);
    ranges.sort_by(|left, right| left.start.cmp(&right.start).then(left.end.cmp(&right.end)));
    ranges
}

impl InjectionDecode {
    fn from_query_value(value: &str) -> Self {
        match value {
            "javascript-literal" => Self::JavaScriptLiteral,
            "javascript-string" => Self::JavaScriptString,
            "python-string" => Self::PythonString,
            "rust-string" => Self::RustString,
            "go-string" => Self::GoString,
            _ => Self::None,
        }
    }
}
