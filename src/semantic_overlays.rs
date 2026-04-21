use std::ops::Range;

use anyhow::{Context, Result};
use tree_sitter::{Node, Parser};

use crate::{
    document_kind::{DocumentKind, DocumentProfile},
    language_aliases::normalize_language_name,
    language_runtime::{runtime, supports_runtime},
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SemanticCaptureSpan {
    pub range: Range<usize>,
    pub capture: &'static str,
}

pub(crate) fn github_actions_expression_spans(source: &str) -> Vec<SemanticCaptureSpan> {
    let mut spans = Vec::new();
    collect_github_actions_embedded_expression_spans(0..source.len(), source, &mut spans);
    spans.sort_by(|left, right| {
        left.range
            .start
            .cmp(&right.range.start)
            .then(left.range.end.cmp(&right.range.end))
            .then(left.capture.cmp(right.capture))
    });
    spans.dedup_by(|left, right| left.range == right.range && left.capture == right.capture);
    spans
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ShellLanguage {
    Bash,
    Fish,
    Zsh,
}

#[derive(Clone, Copy)]
struct CommandSpec {
    name: &'static str,
    command_capture: Option<&'static str>,
    first_bare_capture: Option<&'static str>,
    bare_capture: Option<&'static str>,
    bare_value_filter: ValueFilter,
    option_values: &'static [OptionValueSpec],
}

#[derive(Clone, Copy)]
struct OptionValueSpec {
    options: &'static [&'static str],
    capture: &'static str,
    arity: OptionValueArity,
    value_filter: ValueFilter,
}

#[derive(Clone, Copy)]
enum OptionValueArity {
    Next(usize),
    RestUntilOption,
}

#[derive(Clone, Copy)]
enum ValueFilter {
    Any,
    Identifierish,
}

const FISH_COMMAND_SPECS: &[CommandSpec] = &[
    CommandSpec {
        name: "emit",
        command_capture: Some("function.builtin"),
        first_bare_capture: Some("keyword.directive"),
        bare_capture: None,
        bare_value_filter: ValueFilter::Any,
        option_values: &[],
    },
    CommandSpec {
        name: "functions",
        command_capture: Some("function.builtin"),
        first_bare_capture: None,
        bare_capture: None,
        bare_value_filter: ValueFilter::Any,
        option_values: &[OptionValueSpec {
            options: &["-q", "--query", "-D", "--details", "-c", "--copy"],
            capture: "function",
            arity: OptionValueArity::Next(1),
            value_filter: ValueFilter::Identifierish,
        }],
    },
    CommandSpec {
        name: "read",
        command_capture: Some("function.builtin"),
        first_bare_capture: None,
        bare_capture: Some("variable.parameter"),
        bare_value_filter: ValueFilter::Identifierish,
        option_values: &[],
    },
    CommandSpec {
        name: "set",
        command_capture: Some("function.builtin"),
        first_bare_capture: None,
        bare_capture: Some("variable.parameter"),
        bare_value_filter: ValueFilter::Identifierish,
        option_values: &[],
    },
    CommandSpec {
        name: "status",
        command_capture: Some("function.builtin"),
        first_bare_capture: Some("keyword.directive"),
        bare_capture: None,
        bare_value_filter: ValueFilter::Any,
        option_values: &[],
    },
    CommandSpec {
        name: "string",
        command_capture: Some("function.builtin"),
        first_bare_capture: Some("keyword.directive"),
        bare_capture: None,
        bare_value_filter: ValueFilter::Any,
        option_values: &[],
    },
    CommandSpec {
        name: "type",
        command_capture: Some("function.builtin"),
        first_bare_capture: None,
        bare_capture: None,
        bare_value_filter: ValueFilter::Any,
        option_values: &[OptionValueSpec {
            options: &["-q", "--query", "-w", "--wraps", "-t", "--type"],
            capture: "function",
            arity: OptionValueArity::Next(1),
            value_filter: ValueFilter::Identifierish,
        }],
    },
];

const BASH_COMMAND_SPECS: &[CommandSpec] = &[
    CommandSpec {
        name: "printf",
        command_capture: Some("function.builtin"),
        first_bare_capture: None,
        bare_capture: None,
        bare_value_filter: ValueFilter::Any,
        option_values: &[],
    },
    CommandSpec {
        name: "read",
        command_capture: Some("function.builtin"),
        first_bare_capture: None,
        bare_capture: Some("variable.parameter"),
        bare_value_filter: ValueFilter::Identifierish,
        option_values: &[],
    },
    CommandSpec {
        name: "set",
        command_capture: Some("function.builtin"),
        first_bare_capture: None,
        bare_capture: Some("keyword.directive"),
        bare_value_filter: ValueFilter::Identifierish,
        option_values: &[],
    },
    CommandSpec {
        name: "shopt",
        command_capture: Some("function.builtin"),
        first_bare_capture: None,
        bare_capture: Some("keyword.directive"),
        bare_value_filter: ValueFilter::Identifierish,
        option_values: &[],
    },
    CommandSpec {
        name: "source",
        command_capture: Some("function.builtin"),
        first_bare_capture: None,
        bare_capture: None,
        bare_value_filter: ValueFilter::Any,
        option_values: &[],
    },
    CommandSpec {
        name: "type",
        command_capture: Some("function.builtin"),
        first_bare_capture: None,
        bare_capture: None,
        bare_value_filter: ValueFilter::Any,
        option_values: &[],
    },
];

const ZSH_COMMAND_SPECS: &[CommandSpec] = &[
    CommandSpec {
        name: "autoload",
        command_capture: Some("function.builtin"),
        first_bare_capture: None,
        bare_capture: Some("function"),
        bare_value_filter: ValueFilter::Identifierish,
        option_values: &[],
    },
    CommandSpec {
        name: "emulate",
        command_capture: Some("function.builtin"),
        first_bare_capture: Some("keyword.directive"),
        bare_capture: None,
        bare_value_filter: ValueFilter::Any,
        option_values: &[],
    },
    CommandSpec {
        name: "functions",
        command_capture: Some("function.builtin"),
        first_bare_capture: None,
        bare_capture: Some("function"),
        bare_value_filter: ValueFilter::Identifierish,
        option_values: &[],
    },
    CommandSpec {
        name: "print",
        command_capture: Some("function.builtin"),
        first_bare_capture: None,
        bare_capture: None,
        bare_value_filter: ValueFilter::Any,
        option_values: &[],
    },
    CommandSpec {
        name: "printf",
        command_capture: Some("function.builtin"),
        first_bare_capture: None,
        bare_capture: None,
        bare_value_filter: ValueFilter::Any,
        option_values: &[],
    },
    CommandSpec {
        name: "read",
        command_capture: Some("function.builtin"),
        first_bare_capture: None,
        bare_capture: Some("variable.parameter"),
        bare_value_filter: ValueFilter::Identifierish,
        option_values: &[],
    },
    CommandSpec {
        name: "setopt",
        command_capture: Some("function.builtin"),
        first_bare_capture: None,
        bare_capture: Some("keyword.directive"),
        bare_value_filter: ValueFilter::Identifierish,
        option_values: &[],
    },
    CommandSpec {
        name: "source",
        command_capture: Some("function.builtin"),
        first_bare_capture: None,
        bare_capture: None,
        bare_value_filter: ValueFilter::Any,
        option_values: &[],
    },
    CommandSpec {
        name: "unsetopt",
        command_capture: Some("function.builtin"),
        first_bare_capture: None,
        bare_capture: Some("keyword.directive"),
        bare_value_filter: ValueFilter::Identifierish,
        option_values: &[],
    },
];

pub(crate) fn debug_named_language_tree(language_name: &str, source: &str) -> Result<String> {
    let tree = parse_language_tree(language_name, source)?;
    Ok(tree.root_node().to_sexp())
}

pub(crate) fn debug_semantics(language_name: &str, source: &str) -> Result<String> {
    let spans = semantic_capture_spans_for(language_name, DocumentProfile::Plain, source)?;
    let mut rendered = String::new();

    for span in spans {
        let text = source.get(span.range.clone()).unwrap_or("");
        rendered.push_str(&format!(
            "{} {}..{} {}\n",
            span.capture,
            span.range.start,
            span.range.end,
            text.escape_debug()
        ));
    }

    Ok(rendered)
}

pub(crate) fn semantic_capture_spans(
    document_kind: DocumentKind,
    source: &str,
) -> Result<Vec<SemanticCaptureSpan>> {
    semantic_capture_spans_for(
        document_kind.runtime_name(),
        document_kind.profile(),
        source,
    )
}

fn semantic_capture_spans_for(
    language_name: &str,
    profile: DocumentProfile,
    source: &str,
) -> Result<Vec<SemanticCaptureSpan>> {
    let has_runtime_overlays = matches!(
        language_name,
        "bash"
            | "batch"
            | "fish"
            | "python"
            | "powershell"
            | "zsh"
            | "regex"
            | "regex_javascript"
            | "regex_python"
            | "regex_rust"
            | "regex_go"
            | "regex_posix"
            | "sql"
            | "sql_postgres"
            | "sql_mysql"
            | "sql_sqlite"
            | "jsdoc"
    );
    let has_profile_overlays = matches!(
        profile,
        DocumentProfile::GitHubActionsWorkflow | DocumentProfile::GitHubActionMetadata
    ) && language_name == "yaml"
        || profile == DocumentProfile::FishVariables && language_name == "fish";

    if !has_runtime_overlays && !has_profile_overlays {
        return Ok(Vec::new());
    }

    let tree = parse_language_tree(language_name, source)?;
    let mut spans = Vec::new();

    walk_tree(tree.root_node(), &mut |node| {
        if let Some(shell_language) = ShellLanguage::from_name(language_name) {
            match shell_language {
                ShellLanguage::Fish => collect_fish_node_spans(node, source, &mut spans),
                ShellLanguage::Bash => collect_bash_node_spans(node, source, &mut spans),
                ShellLanguage::Zsh => collect_zsh_node_spans(node, source, &mut spans),
            }
        }

        if language_name == "powershell" {
            collect_powershell_node_spans(node, source, &mut spans);
        }

        if language_name == "python" {
            collect_python_node_spans(node, source, &mut spans);
        }

        if language_name == "batch" {
            collect_batch_node_spans(node, source, &mut spans);
        }

        if is_regex_language(language_name) {
            collect_regex_node_spans(node, source, &mut spans);
        }

        if is_sql_language(language_name) {
            collect_sql_node_spans(language_name, node, source, &mut spans);
        }

        if language_name == "jsdoc" {
            collect_jsdoc_node_spans(node, source, &mut spans);
        }

        if language_name == "yaml"
            && matches!(
                profile,
                DocumentProfile::GitHubActionsWorkflow | DocumentProfile::GitHubActionMetadata
            )
        {
            collect_github_actions_node_spans(node, source, &mut spans);
        }
    });

    if language_name == "fish" && profile == DocumentProfile::FishVariables {
        collect_fish_variables_spans(source, &mut spans);
    }

    spans.sort_by(|left, right| {
        left.range
            .start
            .cmp(&right.range.start)
            .then(left.range.end.cmp(&right.range.end))
            .then(left.capture.cmp(right.capture))
    });
    spans.dedup_by(|left, right| left.range == right.range && left.capture == right.capture);

    Ok(spans)
}

fn collect_github_actions_node_spans(
    node: Node,
    source: &str,
    spans: &mut Vec<SemanticCaptureSpan>,
) {
    match node.kind() {
        "plain_scalar"
        | "string_scalar"
        | "double_quote_scalar"
        | "single_quote_scalar"
        | "block_scalar" => {
            collect_github_actions_embedded_expression_spans(node.byte_range(), source, spans)
        }
        "block_mapping_pair" => collect_github_actions_pair_spans(node, source, spans),
        _ => {}
    }
}

fn collect_github_actions_pair_spans(
    node: Node,
    source: &str,
    spans: &mut Vec<SemanticCaptureSpan>,
) {
    collect_github_actions_uses_spans(node, source, spans);
    collect_github_actions_bare_if_spans(node, source, spans);
    collect_github_actions_schema_spans(node, source, spans);
}

fn collect_github_actions_embedded_expression_spans(
    range: Range<usize>,
    source: &str,
    spans: &mut Vec<SemanticCaptureSpan>,
) {
    let text = &source[range.clone()];
    let mut offset = 0;

    while let Some(start) = text[offset..].find("${{") {
        let start = offset + start;
        let Some(end_rel) = text[start + 3..].find("}}") else {
            break;
        };
        let end = start + 3 + end_rel;
        let absolute_start = range.start + start;
        let absolute_end = range.start + end + 2;

        push_capture(
            spans,
            absolute_start..absolute_start + 3,
            "punctuation.special",
        );
        lex_github_actions_expression(
            &source[absolute_start + 3..range.start + end],
            absolute_start + 3,
            spans,
        );
        push_capture(
            spans,
            range.start + end..absolute_end,
            "punctuation.special",
        );
        offset = end + 2;
    }
}

fn collect_github_actions_bare_if_spans(
    node: Node,
    source: &str,
    spans: &mut Vec<SemanticCaptureSpan>,
) {
    let Some(key) = node.child_by_field_name("key") else {
        return;
    };
    let Some(value) = node.child_by_field_name("value") else {
        return;
    };
    let Some(key_text) = yaml_scalar_text(key, source) else {
        return;
    };
    if key_text != "if" {
        return;
    }

    let Some((text, range)) = yaml_scalar_with_range(value, source) else {
        return;
    };
    if text.contains("${{") {
        return;
    }

    lex_github_actions_expression(text, range.start, spans);
}

fn lex_github_actions_expression(
    source: &str,
    absolute_start: usize,
    spans: &mut Vec<SemanticCaptureSpan>,
) {
    let bytes = source.as_bytes();
    let mut index = 0;
    let mut previous_was_dot = false;

    while index < bytes.len() {
        let ch = bytes[index] as char;
        if ch.is_whitespace() {
            index += 1;
            continue;
        }

        if matches!(ch, '\'' | '"') {
            let start = index;
            index += 1;
            while index < bytes.len() {
                let inner = bytes[index] as char;
                index += 1;
                if inner == '\\' && index < bytes.len() {
                    index += 1;
                    continue;
                }
                if inner == ch {
                    break;
                }
            }
            push_capture(
                spans,
                absolute_start + start..absolute_start + index,
                "string",
            );
            previous_was_dot = false;
            continue;
        }

        if ch.is_ascii_digit() {
            let start = index;
            index += 1;
            while index < bytes.len() && matches!(bytes[index] as char, '0'..='9' | '.' | '_') {
                index += 1;
            }
            push_capture(
                spans,
                absolute_start + start..absolute_start + index,
                "number",
            );
            previous_was_dot = false;
            continue;
        }

        if is_identifier_start(ch) {
            let start = index;
            index += 1;
            while index < bytes.len() && is_identifier_continue(bytes[index] as char) {
                index += 1;
            }

            let token = &source[start..index];
            let capture = if matches!(token, "true" | "false") {
                "boolean"
            } else if token == "null" {
                "constant.builtin"
            } else {
                let mut lookahead = index;
                while lookahead < bytes.len() && (bytes[lookahead] as char).is_whitespace() {
                    lookahead += 1;
                }

                if lookahead < bytes.len() && bytes[lookahead] as char == '(' {
                    "function"
                } else if previous_was_dot {
                    "property"
                } else {
                    "variable"
                }
            };

            push_capture(
                spans,
                absolute_start + start..absolute_start + index,
                capture,
            );
            previous_was_dot = false;
            continue;
        }

        if let Some((capture, width, dot)) = github_actions_operator_capture(&bytes[index..]) {
            push_capture(
                spans,
                absolute_start + index..absolute_start + index + width,
                capture,
            );
            index += width;
            previous_was_dot = dot;
            continue;
        }

        index += 1;
        previous_was_dot = false;
    }
}

fn collect_github_actions_uses_spans(
    node: Node,
    source: &str,
    spans: &mut Vec<SemanticCaptureSpan>,
) {
    let Some(key) = node.child_by_field_name("key") else {
        return;
    };
    let Some(value) = node.child_by_field_name("value") else {
        return;
    };
    let Some(key_text) = yaml_scalar_text(key, source) else {
        return;
    };
    if key_text != "uses" {
        return;
    }

    let Some((text, range)) = yaml_scalar_with_range(value, source) else {
        return;
    };

    if let Some(local) = text.strip_prefix("./") {
        let local_start = range.start + (text.len() - local.len());
        push_capture(spans, range.start..local_start, "punctuation.special");
        push_capture(spans, local_start..range.end, "string");
        return;
    }

    if text.strip_prefix("docker://").is_some() {
        let reference_start = range.start + ("docker://".len());
        push_capture(spans, range.start..reference_start, "punctuation.special");
        push_capture(spans, reference_start..range.end, "string.special");
        return;
    }

    let Some((owner, rest)) = text.split_once('/') else {
        return;
    };
    let Some((repo_and_path, reference)) = rest.rsplit_once('@') else {
        return;
    };
    let (repo, action_path) = repo_and_path
        .split_once('/')
        .map_or((repo_and_path, None), |(repo, path)| (repo, Some(path)));

    let owner_start = range.start;
    let owner_end = owner_start + owner.len();
    let slash_end = owner_end + 1;
    let repo_end = slash_end + repo.len();
    let mut at_start = repo_end;

    push_capture(spans, owner_start..owner_end, "type");
    push_capture(spans, owner_end..slash_end, "punctuation.delimiter");
    push_capture(spans, slash_end..repo_end, "function");

    if let Some(action_path) = action_path {
        let path_slash_end = repo_end + 1;
        let path_end = path_slash_end + action_path.len();
        push_capture(spans, repo_end..path_slash_end, "punctuation.delimiter");
        push_capture(spans, path_slash_end..path_end, "string.special");
        at_start = path_end;
    }

    let at_end = at_start + 1;
    push_capture(spans, at_start..at_end, "keyword.operator");
    push_capture(spans, at_end..at_end + reference.len(), "string.special");
}

fn collect_github_actions_schema_spans(
    node: Node,
    source: &str,
    spans: &mut Vec<SemanticCaptureSpan>,
) {
    let Some(key) = node.child_by_field_name("key") else {
        return;
    };
    let Some(value) = node.child_by_field_name("value") else {
        return;
    };
    let Some(key_text) = yaml_scalar_text(key, source) else {
        return;
    };
    let ancestor_keys = yaml_enclosing_pair_keys(node, source);
    let ancestor_key = ancestor_keys.first().copied();

    match key_text {
        "shell" => {
            if let Some(range) = yaml_scalar_head_token_range(value, source) {
                let token_text = &source[range.clone()];
                if normalize_language_name(token_text).is_some_and(supports_runtime) {
                    push_capture(spans, range, "type.builtin");
                }
            }
        }
        "using" if ancestor_key == Some("runs") => {
            if let Some((text, range)) = yaml_scalar_with_range(value, source)
                && matches!(
                    text,
                    "composite" | "docker" | "node12" | "node16" | "node20" | "node24"
                )
            {
                push_capture(spans, range, "type.builtin");
            }
        }
        "runs-on" => {
            collect_github_actions_runner_value_spans(value, source, spans);
        }
        "runner" if ancestor_keys.starts_with(&["include", "matrix"]) => {
            if let Some((text, range)) = yaml_scalar_with_range(value, source)
                && is_github_actions_runner_label(text)
            {
                push_capture(spans, range, "type");
            }
        }
        "cache" if ancestor_key == Some("with") => {
            if let Some((text, range)) = yaml_scalar_with_range(value, source)
                && matches!(text, "pnpm" | "npm" | "yarn")
            {
                push_capture(spans, range, "type.builtin");
            }
        }
        "if-no-files-found" if ancestor_key == Some("with") => {
            if let Some((text, range)) = yaml_scalar_with_range(value, source)
                && matches!(text, "error" | "warn" | "ignore")
            {
                push_capture(spans, range, "type.builtin");
            }
        }
        _ if ancestor_key == Some("permissions") => {
            if let Some((text, range)) = yaml_scalar_with_range(value, source)
                && matches!(text, "read" | "write" | "none")
            {
                push_capture(spans, range, "type.builtin");
            }
        }
        _ => {}
    }
}

fn is_github_actions_runner_label(text: &str) -> bool {
    matches!(
        text,
        "ubuntu-latest"
            | "ubuntu-24.04"
            | "ubuntu-22.04"
            | "ubuntu-20.04"
            | "windows-latest"
            | "windows-2025"
            | "windows-2022"
            | "windows-2019"
            | "macos-latest"
            | "macos-15"
            | "macos-14"
            | "macos-13"
            | "linux"
            | "self-hosted"
    )
}

fn github_actions_operator_capture(bytes: &[u8]) -> Option<(&'static str, usize, bool)> {
    let first = *bytes.first()? as char;
    let second = bytes.get(1).copied().map(char::from);

    match (first, second) {
        ('&', Some('&'))
        | ('|', Some('|'))
        | ('=', Some('='))
        | ('!', Some('='))
        | ('<', Some('='))
        | ('>', Some('=')) => Some(("keyword.operator", 2, false)),
        ('!', _) | ('<', _) | ('>', _) => Some(("keyword.operator", 1, false)),
        ('(', _) | (')', _) | ('[', _) | (']', _) => Some(("punctuation.bracket", 1, false)),
        ('.', _) | (',', _) => Some(("punctuation.delimiter", 1, first == '.')),
        (':', _) => Some(("punctuation.special", 1, false)),
        _ => None,
    }
}

fn collect_github_actions_runner_value_spans(
    node: Node,
    source: &str,
    spans: &mut Vec<SemanticCaptureSpan>,
) {
    if let Some((text, range)) = yaml_scalar_with_range(node, source) {
        if is_github_actions_runner_label(text) {
            push_capture(spans, range, "type");
        }
        return;
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        collect_github_actions_runner_value_spans(child, source, spans);
    }
}

fn yaml_enclosing_pair_keys<'a>(node: Node<'a>, source: &'a str) -> Vec<&'a str> {
    let mut cursor = node.parent();
    let mut keys = Vec::new();

    while let Some(parent) = cursor {
        if parent.kind() == "block_mapping_pair" {
            let Some(key) = parent.child_by_field_name("key") else {
                cursor = parent.parent();
                continue;
            };
            if let Some(key_text) = yaml_scalar_text(key, source) {
                keys.push(key_text);
            }
        }
        cursor = parent.parent();
    }

    keys
}

fn is_identifier_start(ch: char) -> bool {
    ch == '_' || ch.is_ascii_alphabetic()
}

fn is_identifier_continue(ch: char) -> bool {
    ch == '_' || ch == '-' || ch.is_ascii_alphanumeric()
}

fn yaml_scalar_text<'a>(node: Node, source: &'a str) -> Option<&'a str> {
    yaml_scalar_with_range(node, source).map(|(text, _)| text)
}

fn yaml_scalar_with_range<'a>(node: Node, source: &'a str) -> Option<(&'a str, Range<usize>)> {
    match node.kind() {
        "block_node" | "flow_node" => {
            let mut cursor = node.walk();
            node.named_children(&mut cursor)
                .find_map(|child| yaml_scalar_with_range(child, source))
        }
        "plain_scalar"
        | "string_scalar"
        | "double_quote_scalar"
        | "single_quote_scalar"
        | "block_scalar" => Some((
            source[node.byte_range()].trim_matches(|ch| matches!(ch, '"' | '\'')),
            node.byte_range(),
        )),
        _ => {
            let mut cursor = node.walk();
            node.named_children(&mut cursor)
                .find_map(|child| yaml_scalar_with_range(child, source))
        }
    }
}

fn yaml_scalar_head_token_range(node: Node, source: &str) -> Option<Range<usize>> {
    let (_, range) = yaml_scalar_with_range(node, source)?;
    let raw = &source[range.clone()];
    let start_offset = raw
        .char_indices()
        .find(|(_, ch)| !ch.is_whitespace() && !matches!(ch, '"' | '\''))
        .map(|(index, _)| index)?;
    let token = &raw[start_offset..];
    let end_offset = token
        .char_indices()
        .find(|(_, ch)| ch.is_whitespace() || matches!(ch, '"' | '\''))
        .map(|(index, _)| index)
        .unwrap_or(token.len());
    Some(range.start + start_offset..range.start + start_offset + end_offset)
}

impl ShellLanguage {
    fn from_name(language_name: &str) -> Option<Self> {
        match language_name {
            "bash" => Some(Self::Bash),
            "fish" => Some(Self::Fish),
            "zsh" => Some(Self::Zsh),
            _ => None,
        }
    }
}

fn is_regex_language(language_name: &str) -> bool {
    matches!(
        language_name,
        "regex" | "regex_javascript" | "regex_python" | "regex_rust" | "regex_go" | "regex_posix"
    )
}

fn is_sql_language(language_name: &str) -> bool {
    matches!(
        language_name,
        "sql" | "sql_postgres" | "sql_mysql" | "sql_sqlite"
    )
}

fn parse_language_tree(language_name: &str, source: &str) -> Result<tree_sitter::Tree> {
    let language_runtime = runtime(language_name)
        .with_context(|| format!("missing language runtime for {language_name}"))?;
    let mut parser = Parser::new();
    parser
        .set_language(&language_runtime.language)
        .with_context(|| format!("failed to set parser language for {language_name}"))?;
    parser
        .parse(source, None)
        .with_context(|| format!("failed to parse source for {language_name}"))
}

fn walk_tree(node: Node<'_>, visit: &mut impl FnMut(Node<'_>)) {
    visit(node);

    let mut cursor = node.walk();
    if cursor.goto_first_child() {
        loop {
            walk_tree(cursor.node(), visit);
            if !cursor.goto_next_sibling() {
                break;
            }
        }
    }
}

fn collect_fish_node_spans(node: Node<'_>, source: &str, spans: &mut Vec<SemanticCaptureSpan>) {
    match node.kind() {
        "command" => collect_command_family_spans(node, source, FISH_COMMAND_SPECS, spans),
        "function_definition" => collect_fish_function_option_value_spans(node, source, spans),
        "list_element_access" => collect_bracket_pair_spans(node, spans),
        "range" => collect_operator_text_span(node, source, "..", "operator", spans),
        "variable_expansion" => collect_fish_special_variable_spans(node, source, spans),
        "case_clause" => collect_case_clause_pattern_spans(node, spans),
        _ => {}
    }
}

fn collect_fish_variables_spans(source: &str, spans: &mut Vec<SemanticCaptureSpan>) {
    let mut line_start = 0;
    for (index, ch) in source.char_indices() {
        if ch == '\n' {
            collect_fish_variables_line_spans(line_start, &source[line_start..index], spans);
            line_start = index + 1;
        }
    }

    if line_start < source.len() {
        collect_fish_variables_line_spans(line_start, &source[line_start..], spans);
    }
}

fn collect_fish_variables_line_spans(
    line_start: usize,
    line: &str,
    spans: &mut Vec<SemanticCaptureSpan>,
) {
    let directive_end = line
        .char_indices()
        .find_map(|(index, ch)| ch.is_ascii_whitespace().then_some(index));
    let Some(directive_end) = directive_end else {
        return;
    };
    let directive = &line[..directive_end];
    if directive.is_empty()
        || !directive
            .chars()
            .all(|ch| ch.is_ascii_uppercase() || ch == '_')
    {
        return;
    }

    let rest_start = directive_end
        + line[directive_end..]
            .chars()
            .take_while(|ch| ch.is_ascii_whitespace())
            .map(char::len_utf8)
            .sum::<usize>();
    if rest_start >= line.len() {
        return;
    }

    let rest = &line[rest_start..];
    let Some(separator_offset) = rest.find(':') else {
        return;
    };
    let name = &rest[..separator_offset];
    if name.is_empty() {
        return;
    }

    push_capture(
        spans,
        line_start..line_start + directive_end,
        "keyword.directive",
    );
    push_capture(
        spans,
        line_start + rest_start..line_start + rest_start + separator_offset,
        "string.special.key",
    );
    let separator_start = line_start + rest_start + separator_offset;
    push_capture(
        spans,
        separator_start..separator_start + 1,
        "punctuation.special",
    );

    let value_start = separator_start + 1;
    let value = &rest[separator_offset + 1..];
    if value.is_empty() {
        return;
    }

    if value.chars().all(|ch| ch.is_ascii_digit()) {
        push_capture(spans, value_start..value_start + value.len(), "number");
        return;
    }

    if is_fish_variables_path_name(name) {
        push_fish_variables_path_value_spans(value_start, value, spans);
        return;
    }

    push_capture(spans, value_start..value_start + value.len(), "string");
    push_fish_variables_escape_spans(value_start, value, spans);
}

fn push_fish_variables_path_value_spans(
    absolute_start: usize,
    value: &str,
    spans: &mut Vec<SemanticCaptureSpan>,
) {
    let mut segment_start = 0;
    let mut index = 0;

    while index < value.len() {
        let Some((escape_start, escape_end, code)) = find_next_fish_variables_escape(value, index)
        else {
            break;
        };

        if code.eq_ignore_ascii_case("1e") {
            if segment_start < escape_start {
                push_capture(
                    spans,
                    absolute_start + segment_start..absolute_start + escape_start,
                    "string.special.path",
                );
            }
            push_capture(
                spans,
                absolute_start + escape_start..absolute_start + escape_end,
                "punctuation.special",
            );
            segment_start = escape_end;
        } else {
            push_capture(
                spans,
                absolute_start + escape_start..absolute_start + escape_end,
                "string.escape",
            );
        }

        index = escape_end;
    }

    if segment_start < value.len() {
        push_capture(
            spans,
            absolute_start + segment_start..absolute_start + value.len(),
            "string.special.path",
        );
    }
}

fn push_fish_variables_escape_spans(
    absolute_start: usize,
    value: &str,
    spans: &mut Vec<SemanticCaptureSpan>,
) {
    let mut index = 0;
    while let Some((escape_start, escape_end, code)) = find_next_fish_variables_escape(value, index)
    {
        let capture = if code.eq_ignore_ascii_case("1e") {
            "punctuation.special"
        } else {
            "string.escape"
        };
        push_capture(
            spans,
            absolute_start + escape_start..absolute_start + escape_end,
            capture,
        );
        index = escape_end;
    }
}

fn find_next_fish_variables_escape(value: &str, from: usize) -> Option<(usize, usize, &str)> {
    let bytes = value.as_bytes();
    let mut index = from;

    while index + 3 < bytes.len() {
        if bytes[index] == b'\\'
            && bytes[index + 1] == b'x'
            && bytes[index + 2].is_ascii_hexdigit()
            && bytes[index + 3].is_ascii_hexdigit()
        {
            return Some((index, index + 4, &value[index + 2..index + 4]));
        }
        index += 1;
    }

    None
}

fn is_fish_variables_path_name(name: &str) -> bool {
    let upper = name.to_ascii_uppercase();
    matches!(name, "fish_user_paths")
        || upper == "PATH"
        || upper.ends_with("_PATH")
        || upper.ends_with("_PATHS")
        || upper.ends_with("PATH")
}

fn collect_bash_node_spans(node: Node<'_>, source: &str, spans: &mut Vec<SemanticCaptureSpan>) {
    match node.kind() {
        "command" => collect_command_family_spans(node, source, BASH_COMMAND_SPECS, spans),
        "declaration_command" => {
            collect_bash_like_declaration_command_spans(node, source, "bash", spans)
        }
        "unset_command" => collect_bash_like_unset_command_spans(node, source, spans),
        "subscript" => collect_bash_subscript_spans(node, spans),
        _ => {}
    }
}

fn collect_zsh_node_spans(node: Node<'_>, source: &str, spans: &mut Vec<SemanticCaptureSpan>) {
    match node.kind() {
        "command" => collect_command_family_spans(node, source, ZSH_COMMAND_SPECS, spans),
        "declaration_command" => {
            collect_bash_like_declaration_command_spans(node, source, "zsh", spans)
        }
        "unset_command" => collect_bash_like_unset_command_spans(node, source, spans),
        "subscript" => collect_zsh_subscript_spans(node, spans),
        "expansion_modifier" | "zsh_array_subscript_flags" | "expansion_flags" => {
            push_capture(spans, node.byte_range(), "keyword.directive")
        }
        _ => {}
    }
}

fn collect_powershell_node_spans(
    node: Node<'_>,
    source: &str,
    spans: &mut Vec<SemanticCaptureSpan>,
) {
    match node.kind() {
        "command" => {
            let Some(name_node) = node.child_by_field_name("command_name") else {
                return;
            };
            let Some(name_text) = node_text(name_node, source) else {
                return;
            };
            if is_powershell_builtin_command(name_text) {
                push_capture(spans, name_node.byte_range(), "function.builtin");
            }
        }
        "function_name" => push_capture(spans, node.byte_range(), "function.definition"),
        "variable" => {
            if let Some(name_text) = node_text(node, source)
                && is_powershell_special_variable(name_text)
            {
                push_capture(spans, node.byte_range(), "variable.special");
            }
        }
        _ => {}
    }
}

fn collect_batch_node_spans(node: Node<'_>, source: &str, spans: &mut Vec<SemanticCaptureSpan>) {
    match node.kind() {
        "cmd" => {
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if child.kind() == "command_name" {
                    if let Some(name_text) = node_text(child, source)
                        && is_batch_builtin_command(name_text)
                    {
                        push_capture(spans, child.byte_range(), "function.builtin");
                    }
                    break;
                }
            }
        }
        "label" => push_capture(spans, node.byte_range(), "function.special.definition"),
        "call_stmt" => {
            if let Some(range) = find_text_range_in_node(node, source, ":") {
                push_capture(spans, range, "function");
            }
        }
        "goto_stmt" => {
            if let Some(range) = find_text_range_in_node(node, source, ":eof") {
                push_capture(spans, range, "keyword.directive");
            }
        }
        _ => {}
    }
}

fn collect_regex_node_spans(node: Node<'_>, source: &str, spans: &mut Vec<SemanticCaptureSpan>) {
    match node.kind() {
        "character_class" => collect_bracket_pair_spans(node, spans),
        "class_range" => collect_regex_class_range_spans(node, source, spans),
        "count_quantifier" => collect_regex_count_quantifier_spans(node, source, spans),
        "inline_flags_group" => collect_regex_inline_flags_spans(node, source, spans),
        "character_class_escape" => collect_regex_unicode_property_spans(node, source, spans),
        _ => {}
    }
}

fn collect_sql_node_spans(
    language_name: &str,
    node: Node<'_>,
    source: &str,
    spans: &mut Vec<SemanticCaptureSpan>,
) {
    match node.kind() {
        "function_language" if language_name == "sql_postgres" => {
            for child in node.named_children(&mut node.walk()) {
                if child.kind() == "identifier" {
                    push_capture(spans, child.byte_range(), "type.builtin");
                }
            }
        }
        "field" if language_name == "sql_postgres" => {
            if let Some(opclass) = node.child_by_field_name("opclass") {
                push_capture(spans, opclass.byte_range(), "type.builtin");
            }
        }
        "table_option" if language_name == "sql_mysql" => {
            collect_mysql_table_option_spans(node, source, spans)
        }
        _ => {}
    }
}

fn collect_jsdoc_node_spans(node: Node<'_>, source: &str, spans: &mut Vec<SemanticCaptureSpan>) {
    match node.kind() {
        "inline_tag" => collect_jsdoc_inline_tag_spans(node, source, spans),
        "optional_identifier" => collect_jsdoc_optional_identifier_spans(node, spans),
        _ => {}
    }
}

fn collect_python_node_spans(node: Node<'_>, source: &str, spans: &mut Vec<SemanticCaptureSpan>) {
    match node.kind() {
        "assignment" => {
            if let Some(left) = node.child_by_field_name("left") {
                collect_python_binding_target_spans(left, source, spans);
            }
        }
        "attribute" => collect_python_attribute_object_spans(node, source, spans),
        _ => {}
    }
}

fn collect_python_binding_target_spans(
    node: Node<'_>,
    source: &str,
    spans: &mut Vec<SemanticCaptureSpan>,
) {
    match node.kind() {
        "identifier" if node_text(node, source).is_some_and(is_python_local_name) => {
            push_capture(spans, node.byte_range(), "variable.local");
        }
        "pattern_list" | "tuple" | "list" | "list_pattern" | "tuple_pattern" => {
            for child in node.children(&mut node.walk()) {
                if child.is_named() {
                    collect_python_binding_target_spans(child, source, spans);
                }
            }
        }
        _ => {}
    }
}

fn collect_python_attribute_object_spans(
    node: Node<'_>,
    source: &str,
    spans: &mut Vec<SemanticCaptureSpan>,
) {
    let Some(object) = node.child_by_field_name("object") else {
        return;
    };
    if object.kind() != "identifier" {
        return;
    }
    if node_text(object, source).is_some_and(is_python_local_name) {
        push_capture(spans, object.byte_range(), "variable.local");
    }
}

fn collect_command_family_spans(
    node: Node<'_>,
    source: &str,
    specs: &[CommandSpec],
    spans: &mut Vec<SemanticCaptureSpan>,
) {
    let Some(name_node) = node.child_by_field_name("name") else {
        return;
    };
    let Some(name_text) = node_text(name_node, source) else {
        return;
    };
    let Some(spec) = specs.iter().find(|spec| spec.name == name_text) else {
        return;
    };

    if let Some(capture) = spec.command_capture {
        push_capture(spans, name_node.byte_range(), capture);
    }

    let arguments = field_children(node, "argument");
    let mut seen_first_bare = false;
    let mut pending_capture: Option<(&'static str, usize, ValueFilter)> = None;

    for argument in arguments {
        let Some(argument_text) = node_text(argument, source) else {
            continue;
        };

        if let Some((capture, remaining, value_filter)) = pending_capture {
            if is_option_like(argument_text) {
                pending_capture = None;
            } else {
                if matches_value_filter(argument_text, value_filter) {
                    push_capture(spans, argument.byte_range(), capture);
                }
                pending_capture = match remaining {
                    0 | 1 => None,
                    _ => Some((capture, remaining - 1, value_filter)),
                };
                continue;
            }
        }

        if let Some((capture, arity, value_filter)) =
            option_value_capture(spec.option_values, argument_text)
        {
            pending_capture = Some((capture, arity.pending_count(), value_filter));
            continue;
        }

        if is_option_like(argument_text) {
            continue;
        }

        if !seen_first_bare {
            if let Some(capture) = spec.first_bare_capture {
                push_capture(spans, argument.byte_range(), capture);
            }
            seen_first_bare = true;
        }

        if let Some(capture) = spec.bare_capture
            && matches_value_filter(argument_text, spec.bare_value_filter)
        {
            push_capture(spans, argument.byte_range(), capture);
        }
    }
}

fn collect_fish_function_option_value_spans(
    node: Node<'_>,
    source: &str,
    spans: &mut Vec<SemanticCaptureSpan>,
) {
    let options = field_children(node, "option");
    let option_specs = [
        OptionValueSpec {
            options: &["--argument-names"],
            capture: "variable.parameter",
            arity: OptionValueArity::RestUntilOption,
            value_filter: ValueFilter::Identifierish,
        },
        OptionValueSpec {
            options: &["--on-variable"],
            capture: "variable.parameter",
            arity: OptionValueArity::Next(1),
            value_filter: ValueFilter::Identifierish,
        },
        OptionValueSpec {
            options: &["--on-event", "--on-job-exit", "--inherit-variable"],
            capture: "keyword.directive",
            arity: OptionValueArity::Next(1),
            value_filter: ValueFilter::Any,
        },
    ];
    let mut pending_capture: Option<(&'static str, usize, ValueFilter)> = None;

    for option in options {
        let Some(option_text) = node_text(option, source) else {
            continue;
        };

        if let Some((capture, remaining, value_filter)) = pending_capture {
            if is_option_like(option_text) {
                pending_capture = None;
            } else {
                if matches_value_filter(option_text, value_filter) {
                    push_capture(spans, option.byte_range(), capture);
                }
                pending_capture = match remaining {
                    0 | 1 => None,
                    _ => Some((capture, remaining - 1, value_filter)),
                };
                continue;
            }
        }

        if let Some((capture, arity, value_filter)) =
            option_value_capture(&option_specs, option_text)
        {
            pending_capture = Some((capture, arity.pending_count(), value_filter));
        }
    }
}

fn collect_fish_special_variable_spans(
    node: Node<'_>,
    source: &str,
    spans: &mut Vec<SemanticCaptureSpan>,
) {
    let Some(first_named_child) = node.named_child(0) else {
        return;
    };
    let Some(variable_name) = node_text(first_named_child, source) else {
        return;
    };
    if !is_fish_special_variable_name(variable_name) {
        return;
    }

    push_capture(
        spans,
        node.start_byte()..first_named_child.end_byte(),
        "variable.special",
    );
}

fn collect_bash_like_declaration_command_spans(
    node: Node<'_>,
    source: &str,
    shell: &str,
    spans: &mut Vec<SemanticCaptureSpan>,
) {
    if let Some(keyword_node) = node.child(0) {
        let capture = if matches!(shell, "bash" | "zsh") {
            "function.builtin"
        } else {
            "function"
        };
        push_capture(spans, keyword_node.byte_range(), capture);
    }

    for child in node.children(&mut node.walk()) {
        match child.kind() {
            "variable_assignment" => {
                if let Some(name) = child.child_by_field_name("name") {
                    push_capture(spans, name.byte_range(), "variable.parameter");
                }
            }
            "word" | "variable_name" | "simple_variable_name" => {
                if let Some(text) = node_text(child, source)
                    && !is_option_like(text)
                    && looks_like_identifierish(text)
                {
                    push_capture(spans, child.byte_range(), "variable.parameter");
                }
            }
            _ => {}
        }
    }
}

fn collect_bash_like_unset_command_spans(
    node: Node<'_>,
    source: &str,
    spans: &mut Vec<SemanticCaptureSpan>,
) {
    if let Some(keyword_node) = node.child(0) {
        push_capture(spans, keyword_node.byte_range(), "function.builtin");
    }

    for child in node.children(&mut node.walk()) {
        match child.kind() {
            "word" | "variable_name" | "simple_variable_name" => {
                if let Some(text) = node_text(child, source)
                    && !is_option_like(text)
                    && looks_like_identifierish(text)
                {
                    push_capture(spans, child.byte_range(), "variable.parameter");
                }
            }
            _ => {}
        }
    }
}

fn collect_bracket_pair_spans(node: Node<'_>, spans: &mut Vec<SemanticCaptureSpan>) {
    let range = node.byte_range();
    if range.end.saturating_sub(range.start) < 2 {
        return;
    }

    push_capture(spans, range.start..(range.start + 1), "punctuation.bracket");
    push_capture(spans, (range.end - 1)..range.end, "punctuation.bracket");
}

fn collect_operator_text_span(
    node: Node<'_>,
    source: &str,
    needle: &str,
    capture: &'static str,
    spans: &mut Vec<SemanticCaptureSpan>,
) {
    let Some(text) = node_text(node, source) else {
        return;
    };
    if let Some(offset) = text.find(needle) {
        push_capture(
            spans,
            (node.start_byte() + offset)..(node.start_byte() + offset + needle.len()),
            capture,
        );
    }
}

fn collect_regex_class_range_spans(
    node: Node<'_>,
    source: &str,
    spans: &mut Vec<SemanticCaptureSpan>,
) {
    if let Some(offset) = node_text(node, source).and_then(|text| text.find('-')) {
        let start = node.start_byte() + offset;
        push_capture(spans, start..(start + 1), "operator.regex");
    }
}

fn collect_regex_count_quantifier_spans(
    node: Node<'_>,
    source: &str,
    spans: &mut Vec<SemanticCaptureSpan>,
) {
    let Some(text) = node_text(node, source) else {
        return;
    };
    if text.len() < 2 {
        return;
    }

    push_capture(
        spans,
        node.start_byte()..(node.start_byte() + 1),
        "punctuation.bracket",
    );
    push_capture(
        spans,
        (node.end_byte() - 1)..node.end_byte(),
        "punctuation.bracket",
    );

    if let Some(offset) = text.find(',') {
        let start = node.start_byte() + offset;
        push_capture(spans, start..(start + 1), "punctuation.delimiter");
    }

    for child in node.named_children(&mut node.walk()) {
        if child.kind() == "decimal_digits" {
            push_capture(spans, child.byte_range(), "number.quantifier.regex");
        }
    }
}

fn collect_regex_inline_flags_spans(
    node: Node<'_>,
    source: &str,
    spans: &mut Vec<SemanticCaptureSpan>,
) {
    let Some(text) = node_text(node, source) else {
        return;
    };
    if !text.starts_with("(?") || !text.ends_with(')') {
        return;
    }

    push_capture(
        spans,
        node.start_byte()..(node.start_byte() + 2),
        "punctuation.bracket.regex",
    );
    push_capture(
        spans,
        (node.end_byte() - 1)..node.end_byte(),
        "punctuation.bracket.regex",
    );

    if let Some(offset) = text.find(':') {
        let start = node.start_byte() + offset;
        push_capture(spans, start..(start + 1), "punctuation.delimiter.regex");
    }
    for (offset, ch) in text.char_indices() {
        if ch == '-' {
            let start = node.start_byte() + offset;
            push_capture(spans, start..(start + 1), "operator.regex");
        }
    }
    for child in node.named_children(&mut node.walk()) {
        if child.kind() == "flags" {
            push_capture(spans, child.byte_range(), "keyword.operator.regex");
        }
    }
}

fn collect_regex_unicode_property_spans(
    node: Node<'_>,
    source: &str,
    spans: &mut Vec<SemanticCaptureSpan>,
) {
    let Some(expression) = node.named_child(0) else {
        return;
    };
    if expression.kind() != "unicode_property_value_expression" {
        return;
    }

    let Some(text) = node_text(node, source) else {
        return;
    };
    let prefix_len = if text.starts_with("\\p") || text.starts_with("\\P") {
        2
    } else {
        return;
    };
    let base = node.start_byte();

    push_capture(spans, base..(base + prefix_len), "operator.regex");

    if let Some(open_offset) = text.find('{') {
        let start = base + open_offset;
        push_capture(spans, start..(start + 1), "punctuation.bracket.regex");
    }
    if let Some(close_offset) = text.rfind('}') {
        let start = base + close_offset;
        push_capture(spans, start..(start + 1), "punctuation.bracket.regex");
    }
    if let Some(eq_offset) = text.find('=') {
        let start = base + eq_offset;
        push_capture(spans, start..(start + 1), "operator.regex");
    }
    for child in expression.named_children(&mut expression.walk()) {
        match child.kind() {
            "unicode_property_name" | "unicode_property_value" => {
                push_capture(spans, child.byte_range(), "type.builtin")
            }
            _ => {}
        }
    }
}

fn collect_mysql_table_option_spans(
    node: Node<'_>,
    source: &str,
    spans: &mut Vec<SemanticCaptureSpan>,
) {
    let Some(name) = node.child_by_field_name("name") else {
        return;
    };
    let Some(value) = node.child_by_field_name("value") else {
        return;
    };
    let Some(name_text) = node_text(name, source) else {
        return;
    };

    if matches!(
        name_text.to_ascii_lowercase().as_str(),
        "engine" | "charset" | "character set" | "collate"
    ) && value.kind() == "identifier"
    {
        push_capture(spans, value.byte_range(), "type.builtin");
    }
}

fn collect_jsdoc_inline_tag_spans(
    node: Node<'_>,
    source: &str,
    spans: &mut Vec<SemanticCaptureSpan>,
) {
    let mut cursor = node.walk();
    let mut children = node.named_children(&mut cursor);
    let _tag_name = children.next();
    let Some(description) = children.next() else {
        return;
    };
    let Some(text) = node_text(description, source) else {
        return;
    };
    let base = description.start_byte();
    let capture = if text.contains('/') {
        "text.uri"
    } else {
        "variable.jsdoc"
    };
    let mut chunk_start = None;

    for (offset, ch) in text.char_indices() {
        if matches!(ch, '.' | '#' | '~' | '/' | ':') {
            if let Some(start) = chunk_start.take() {
                push_capture(spans, (base + start)..(base + offset), capture);
            }
            let start = base + offset;
            push_capture(
                spans,
                start..(start + ch.len_utf8()),
                "punctuation.delimiter",
            );
        } else if !ch.is_whitespace() {
            if chunk_start.is_none() {
                chunk_start = Some(offset);
            }
        } else if let Some(start) = chunk_start.take() {
            push_capture(spans, (base + start)..(base + offset), capture);
        }
    }

    if let Some(start) = chunk_start {
        push_capture(spans, (base + start)..(base + text.len()), capture);
    }
}

fn collect_jsdoc_optional_identifier_spans(node: Node<'_>, spans: &mut Vec<SemanticCaptureSpan>) {
    let range = node.byte_range();
    if range.end.saturating_sub(range.start) < 2 {
        return;
    }

    push_capture(spans, range.start..(range.start + 1), "punctuation.bracket");
    push_capture(spans, (range.end - 1)..range.end, "punctuation.bracket");
}

fn option_value_capture(
    specs: &[OptionValueSpec],
    argument_text: &str,
) -> Option<(&'static str, OptionValueArity, ValueFilter)> {
    specs.iter().find_map(|spec| {
        spec.options
            .iter()
            .any(|option| option == &argument_text)
            .then_some((spec.capture, spec.arity, spec.value_filter))
    })
}

fn field_children<'tree>(node: Node<'tree>, field_name: &str) -> Vec<Node<'tree>> {
    let mut children = Vec::new();
    for index in 0..node.child_count() {
        let index = index as u32;
        if node.field_name_for_child(index) == Some(field_name)
            && let Some(child) = node.child(index)
        {
            children.push(child);
        }
    }
    children
}

fn node_text<'a>(node: Node<'_>, source: &'a str) -> Option<&'a str> {
    source.get(node.byte_range())
}

fn is_option_like(text: &str) -> bool {
    text.starts_with('-') && text.len() > 1
}

impl OptionValueArity {
    fn pending_count(self) -> usize {
        match self {
            Self::Next(count) => count,
            Self::RestUntilOption => usize::MAX,
        }
    }
}

fn matches_value_filter(text: &str, filter: ValueFilter) -> bool {
    match filter {
        ValueFilter::Any => true,
        ValueFilter::Identifierish => looks_like_identifierish(text),
    }
}

fn looks_like_identifierish(text: &str) -> bool {
    !text.is_empty()
        && text
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | ':' | '-'))
}

fn is_python_local_name(text: &str) -> bool {
    let mut chars = text.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    first == '_' || first.is_ascii_lowercase()
}

fn is_powershell_builtin_command(name: &str) -> bool {
    matches!(
        name,
        "Write-Host"
            | "Write-Output"
            | "Write-Error"
            | "Write-Warning"
            | "Get-Item"
            | "Get-ChildItem"
            | "Set-Item"
            | "Remove-Item"
            | "Test-Path"
            | "Join-Path"
            | "New-Item"
            | "Invoke-WebRequest"
            | "Invoke-RestMethod"
            | "Select-Object"
            | "Where-Object"
            | "ForEach-Object"
            | "Set-StrictMode"
    )
}

fn is_powershell_special_variable(name: &str) -> bool {
    name.starts_with("$env:")
        || matches!(
            name,
            "$PSVersionTable"
                | "$Error"
                | "$HOME"
                | "$PWD"
                | "$PID"
                | "$LASTEXITCODE"
                | "$?"
                | "$_"
        )
}

fn is_batch_builtin_command(name: &str) -> bool {
    matches!(
        name.to_ascii_lowercase().as_str(),
        "echo" | "set" | "call" | "goto" | "shift" | "start" | "copy" | "move" | "del" | "dir"
    )
}

fn find_text_range_in_node(node: Node<'_>, source: &str, needle: &str) -> Option<Range<usize>> {
    let range = node.byte_range();
    let text = source.get(range.clone())?;
    let offset = text.find(needle)?;
    Some(range.start + offset..range.start + offset + needle.len())
}

fn is_fish_special_variable_name(name: &str) -> bool {
    matches!(
        name,
        "argv"
            | "status"
            | "pipestatus"
            | "last_pid"
            | "CMD_DURATION"
            | "SHLVL"
            | "PWD"
            | "HOME"
            | "USER"
            | "hostname"
            | "version"
    ) || name.starts_with("fish_")
}

fn collect_case_clause_pattern_spans(node: Node<'_>, spans: &mut Vec<SemanticCaptureSpan>) {
    for child in node.children(&mut node.walk()) {
        match child.kind() {
            "word" | "concatenation" | "single_quote_string" | "double_quote_string" | "glob" => {
                push_capture(spans, child.byte_range(), "string.regex");
            }
            _ => break,
        }
    }
}

fn collect_bash_subscript_spans(node: Node<'_>, spans: &mut Vec<SemanticCaptureSpan>) {
    collect_bracket_pair_spans(node, spans);
    if let Some(index) = node.child_by_field_name("index") {
        push_number_like_tokens(index, spans);
    }
}

fn collect_zsh_subscript_spans(node: Node<'_>, spans: &mut Vec<SemanticCaptureSpan>) {
    collect_bracket_pair_spans(node, spans);
    if let Some(flags) = node.child_by_field_name("flags") {
        push_capture(spans, flags.byte_range(), "keyword.directive");
    }
    if let Some(index) = node.child_by_field_name("index") {
        push_number_like_tokens(index, spans);
    }
}

fn push_number_like_tokens(node: Node<'_>, spans: &mut Vec<SemanticCaptureSpan>) {
    match node.kind() {
        "number" | "integer" | "float" => push_capture(spans, node.byte_range(), "number"),
        _ => {
            for child in node.children(&mut node.walk()) {
                push_number_like_tokens(child, spans);
            }
        }
    }
}

fn push_capture(spans: &mut Vec<SemanticCaptureSpan>, range: Range<usize>, capture: &'static str) {
    if range.start < range.end {
        spans.push(SemanticCaptureSpan { range, capture });
    }
}
