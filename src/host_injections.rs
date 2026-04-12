use std::ops::Range;

use anyhow::{Context, Result};
use tree_sitter::{Node, QueryCursor, StreamingIterator, Tree};

use crate::{
    document_kind::{DocumentKind, DocumentProfile},
    language_aliases::normalize_language_name,
    language_runtime::{LanguageRuntime, supports_runtime},
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum InjectionVisualKind {
    Transparent,
    Block,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum InjectionVisualAnchor {
    Content,
    LineStart,
}

#[derive(Debug)]
pub(crate) struct InjectionCandidate {
    pub(crate) ranges: Vec<Range<usize>>,
    pub(crate) language_name: String,
    pub(crate) is_combined: bool,
    pub(crate) merge_parent_styles: bool,
    pub(crate) decode: InjectionDecode,
    pub(crate) highlight_github_expressions: bool,
    pub(crate) visual_kind: Option<InjectionVisualKind>,
    pub(crate) visual_level_bump: Option<usize>,
    pub(crate) visual_anchor: Option<InjectionVisualAnchor>,
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
    _runtime_name: &str,
) -> Result<Vec<InjectionCandidate>> {
    let Some(query) = language_runtime.injections_query.as_ref() else {
        return Ok(Vec::new());
    };
    let capture_names = query.capture_names();
    let mut cursor = QueryCursor::new();
    let mut candidates = Vec::new();

    let mut matches = cursor.matches(query, tree.root_node(), source.as_bytes());
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
        let visual_kind = query
            .property_settings(query_match.pattern_index)
            .iter()
            .find(|property| property.key.as_ref() == "kat.visual")
            .and_then(|property| property.value.as_deref())
            .map(InjectionVisualKind::from_query_value);
        let visual_level_bump = query
            .property_settings(query_match.pattern_index)
            .iter()
            .find(|property| property.key.as_ref() == "kat.visual-level")
            .and_then(|property| property.value.as_deref())
            .map(parse_visual_level_bump)
            .transpose()?;
        let visual_anchor = query
            .property_settings(query_match.pattern_index)
            .iter()
            .find(|property| property.key.as_ref() == "kat.visual-anchor")
            .and_then(|property| property.value.as_deref())
            .map(InjectionVisualAnchor::from_query_value);

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

        if !supports_runtime(&injection_language) || content_ranges.is_empty() {
            continue;
        }

        if injection_combined {
            candidates.push(InjectionCandidate {
                ranges: normalize_ranges(content_ranges),
                language_name: injection_language,
                is_combined: true,
                merge_parent_styles,
                decode,
                highlight_github_expressions: false,
                visual_kind,
                visual_level_bump,
                visual_anchor,
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
                    highlight_github_expressions: false,
                    visual_kind,
                    visual_level_bump,
                    visual_anchor,
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
    let mut candidates = Vec::new();

    match (document_kind.runtime_name(), document_kind.profile()) {
        ("dockerfile", _) => candidates.extend(collect_dockerfile_injection_candidates(
            language_runtime,
            tree,
            source,
        )?),
        (
            "yaml",
            DocumentProfile::GitHubActionsWorkflow | DocumentProfile::GitHubActionMetadata,
        ) => candidates.extend(collect_github_actions_yaml_injection_candidates(
            tree, source,
        )),
        ("javascript" | "typescript" | "tsx", _) => {
            candidates.extend(collect_ecmascript_comment_injection_candidates(
                document_kind.runtime_name(),
                tree,
                source,
            ))
        }
        _ => {}
    }

    Ok(candidates)
}

fn collect_dockerfile_injection_candidates(
    language_runtime: &LanguageRuntime,
    tree: &Tree,
    source: &str,
) -> Result<Vec<InjectionCandidate>> {
    let query = language_runtime
        .injections_query
        .as_ref()
        .context("missing cached dockerfile injections query")?;
    let capture_names = query.capture_names();
    let mut cursor = QueryCursor::new();
    let mut current_shell = String::from("bash");
    let mut candidates = Vec::new();

    let mut matches = cursor.matches(query, tree.root_node(), source.as_bytes());
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

        if let Some(shell) = matched_shell
            && supports_runtime(&shell)
        {
            current_shell = shell;
        }

        if !supports_runtime(&current_shell) {
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
                    highlight_github_expressions: false,
                    visual_kind: Some(InjectionVisualKind::Block),
                    visual_level_bump: Some(1),
                    visual_anchor: Some(InjectionVisualAnchor::Content),
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

    walk_github_actions_yaml_node(tree.root_node(), source, None, &mut candidates);

    candidates
}

fn collect_ecmascript_comment_injection_candidates(
    runtime_name: &str,
    tree: &Tree,
    source: &str,
) -> Vec<InjectionCandidate> {
    let mut candidates = Vec::new();
    walk_ecmascript_comment_injection_nodes(
        runtime_name,
        tree.root_node(),
        source,
        &mut candidates,
    );
    candidates
}

fn walk_ecmascript_comment_injection_nodes(
    runtime_name: &str,
    node: Node,
    source: &str,
    candidates: &mut Vec<InjectionCandidate>,
) {
    if node.kind() == "comment" {
        collect_ecmascript_comment_injection_candidate(runtime_name, node, source, candidates);
    }

    for child in named_children(node) {
        walk_ecmascript_comment_injection_nodes(runtime_name, child, source, candidates);
    }
}

fn collect_ecmascript_comment_injection_candidate(
    runtime_name: &str,
    comment: Node,
    source: &str,
    candidates: &mut Vec<InjectionCandidate>,
) {
    let Some(language_name) =
        ecmascript_comment_injection_language(runtime_name, &source[comment.byte_range()])
    else {
        return;
    };
    let Some(next) = comment.next_named_sibling() else {
        return;
    };

    match runtime_name {
        "javascript" => {
            push_javascript_comment_injection_candidates(language_name, next, candidates)
        }
        "typescript" | "tsx" => {
            push_typescript_comment_injection_candidates(language_name, next, candidates)
        }
        _ => {}
    }
}

fn ecmascript_comment_injection_language(
    runtime_name: &str,
    comment_text: &str,
) -> Option<&'static str> {
    let hint = parse_block_comment_hint(comment_text)?;

    match runtime_name {
        "javascript" => match hint {
            "html" => Some("html"),
            "sql" => Some("sql"),
            "sql:postgres" | "sql:postgresql" | "sql:pgsql" => Some("sql_postgres"),
            "sql:mysql" | "sql:mariadb" => Some("sql_mysql"),
            "sql:sqlite" | "sql:sqlite3" => Some("sql_sqlite"),
            "gql" | "graphql" => Some("graphql"),
            "css" => Some("css"),
            _ => None,
        },
        "typescript" | "tsx" => match hint {
            "html" => Some("html"),
            "sql" => Some("sql"),
            "gql" | "graphql" => Some("graphql"),
            "css" => Some("css"),
            _ => None,
        },
        _ => None,
    }
}

fn parse_block_comment_hint(comment_text: &str) -> Option<&str> {
    let trimmed = comment_text.trim();
    let inner = trimmed.strip_prefix("/*")?.strip_suffix("*/")?;
    Some(inner.trim())
}

fn push_javascript_comment_injection_candidates(
    language_name: &'static str,
    node: Node,
    candidates: &mut Vec<InjectionCandidate>,
) {
    match node.kind() {
        "string" => candidates.push(build_host_injection_candidate(
            vec![node.byte_range()],
            language_name,
            InjectionDecode::JavaScriptLiteral,
        )),
        "template_string" => {
            for fragment in named_children(node) {
                if fragment.kind() != "string_fragment" {
                    continue;
                }

                let decode = if language_name == "css" {
                    InjectionDecode::None
                } else {
                    InjectionDecode::JavaScriptString
                };
                candidates.push(build_host_injection_candidate(
                    vec![fragment.byte_range()],
                    language_name,
                    decode,
                ));
            }
        }
        _ => {}
    }
}

fn push_typescript_comment_injection_candidates(
    language_name: &'static str,
    node: Node,
    candidates: &mut Vec<InjectionCandidate>,
) {
    if node.kind() != "template_string" {
        return;
    }

    for fragment in named_children(node) {
        if fragment.kind() != "string_fragment" {
            continue;
        }

        candidates.push(build_host_injection_candidate(
            vec![fragment.byte_range()],
            language_name,
            InjectionDecode::None,
        ));
    }
}

fn build_host_injection_candidate(
    ranges: Vec<Range<usize>>,
    language_name: &'static str,
    decode: InjectionDecode,
) -> InjectionCandidate {
    InjectionCandidate {
        ranges,
        language_name: language_name.to_owned(),
        is_combined: false,
        merge_parent_styles: false,
        decode,
        highlight_github_expressions: false,
        visual_kind: Some(InjectionVisualKind::Transparent),
        visual_level_bump: Some(0),
        visual_anchor: Some(InjectionVisualAnchor::Content),
    }
}

fn walk_github_actions_yaml_node(
    node: Node,
    source: &str,
    inherited_shell: Option<&str>,
    candidates: &mut Vec<InjectionCandidate>,
) {
    if node.kind() == "block_mapping" {
        collect_github_actions_yaml_mapping_candidates(node, source, inherited_shell, candidates);
        return;
    }

    for child in named_children(node) {
        walk_github_actions_yaml_node(child, source, inherited_shell, candidates);
    }
}

fn collect_github_actions_yaml_mapping_candidates(
    node: Node,
    source: &str,
    inherited_shell: Option<&str>,
    candidates: &mut Vec<InjectionCandidate>,
) {
    let mut explicit_shell = None;
    let mut default_shell = None;
    let mut run_ranges = Vec::new();
    let mut nested_values = Vec::new();

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
                explicit_shell = github_actions_shell_language(value, source);
            }
            "defaults" => {
                default_shell = github_actions_defaults_shell(value, source);
            }
            "run" => {
                if let Some(range) = yaml_injection_content_range(value, source) {
                    run_ranges.push(range);
                }
            }
            _ => {}
        }

        nested_values.push(value);
    }

    let inherited_shell = default_shell.as_deref().or(inherited_shell);

    if !run_ranges.is_empty() {
        let language_name = explicit_shell
            .as_deref()
            .or(inherited_shell)
            .filter(|shell| supports_runtime(shell))
            .unwrap_or("bash")
            .to_owned();

        if supports_runtime(&language_name) {
            for range in run_ranges {
                candidates.push(InjectionCandidate {
                    ranges: vec![range],
                    language_name: language_name.clone(),
                    is_combined: true,
                    merge_parent_styles: false,
                    decode: InjectionDecode::None,
                    highlight_github_expressions: true,
                    visual_kind: Some(InjectionVisualKind::Block),
                    visual_level_bump: Some(1),
                    visual_anchor: Some(InjectionVisualAnchor::Content),
                });
            }
        }
    }

    for value in nested_values {
        walk_github_actions_yaml_node(value, source, inherited_shell, candidates);
    }
}

fn github_actions_shell_language(node: Node, source: &str) -> Option<String> {
    yaml_scalar_text(node, source)
        .and_then(normalize_language_name)
        .map(str::to_owned)
}

fn github_actions_defaults_shell(node: Node, source: &str) -> Option<String> {
    let run = yaml_mapping_value_for_key(node, source, "run")?;
    let shell = yaml_mapping_value_for_key(run, source, "shell")?;
    github_actions_shell_language(shell, source)
}

fn yaml_mapping_value_for_key<'a>(
    node: Node<'a>,
    source: &str,
    key_name: &str,
) -> Option<Node<'a>> {
    match node.kind() {
        "block_node" | "flow_node" => named_children(node)
            .find_map(|child| yaml_mapping_value_for_key(child, source, key_name)),
        "block_mapping" | "flow_mapping" => named_children(node).find_map(|pair| {
            if pair.kind() != "block_mapping_pair" && pair.kind() != "flow_pair" {
                return None;
            }

            let key = pair.child_by_field_name("key")?;
            let value = pair.child_by_field_name("value")?;
            let key_text = yaml_scalar_text(key, source)?;
            (key_text == key_name).then_some(value)
        }),
        _ => None,
    }
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

impl InjectionVisualKind {
    fn from_query_value(value: &str) -> Self {
        match value {
            "block" => Self::Block,
            _ => Self::Transparent,
        }
    }
}

impl InjectionVisualAnchor {
    fn from_query_value(value: &str) -> Self {
        match value {
            "line-start" => Self::LineStart,
            _ => Self::Content,
        }
    }
}

pub(crate) fn default_visual_level_bump(visual_kind: InjectionVisualKind) -> usize {
    match visual_kind {
        InjectionVisualKind::Transparent => 0,
        InjectionVisualKind::Block => 1,
    }
}

fn parse_visual_level_bump(value: &str) -> Result<usize> {
    value
        .trim()
        .trim_start_matches('+')
        .parse::<usize>()
        .with_context(|| format!("invalid kat.visual-level value: {value}"))
}
