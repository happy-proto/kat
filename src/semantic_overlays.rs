use std::ops::Range;

use anyhow::{Context, Result};
use tree_sitter::{Node, Parser};

use crate::language_runtime::runtime;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SemanticCaptureSpan {
    pub range: Range<usize>,
    pub capture: &'static str,
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
    let spans = semantic_capture_spans(language_name, source)?;
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
    language_name: &str,
    source: &str,
) -> Result<Vec<SemanticCaptureSpan>> {
    if !matches!(
        language_name,
        "bash"
            | "fish"
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
    ) {
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

        if is_regex_language(language_name) {
            collect_regex_node_spans(node, source, &mut spans);
        }

        if is_sql_language(language_name) {
            collect_sql_node_spans(language_name, node, source, &mut spans);
        }

        if language_name == "jsdoc" {
            collect_jsdoc_node_spans(node, source, &mut spans);
        }
    });

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

        if let Some(capture) = spec.bare_capture {
            if matches_value_filter(argument_text, spec.bare_value_filter) {
                push_capture(spans, argument.byte_range(), capture);
            }
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

        if let Some((capture, arity, value_filter)) = option_value_capture(&option_specs, option_text)
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
                if let Some(text) = node_text(child, source) {
                    if !is_option_like(text) && looks_like_identifierish(text) {
                        push_capture(spans, child.byte_range(), "variable.parameter");
                    }
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
                if let Some(text) = node_text(child, source) {
                    if !is_option_like(text) && looks_like_identifierish(text) {
                        push_capture(spans, child.byte_range(), "variable.parameter");
                    }
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

    push_capture(spans, node.start_byte()..(node.start_byte() + 1), "punctuation.bracket");
    push_capture(spans, (node.end_byte() - 1)..node.end_byte(), "punctuation.bracket");

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
    let capture = if text.contains('/') { "text.uri" } else { "variable.jsdoc" };
    let mut chunk_start = None;

    for (offset, ch) in text.char_indices() {
        if matches!(ch, '.' | '#' | '~' | '/' | ':') {
            if let Some(start) = chunk_start.take() {
                push_capture(spans, (base + start)..(base + offset), capture);
            }
            let start = base + offset;
            push_capture(spans, start..(start + ch.len_utf8()), "punctuation.delimiter");
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
        if node.field_name_for_child(index) == Some(field_name) {
            if let Some(child) = node.child(index) {
                children.push(child);
            }
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

fn push_capture(
    spans: &mut Vec<SemanticCaptureSpan>,
    range: Range<usize>,
    capture: &'static str,
) {
    if range.start < range.end {
        spans.push(SemanticCaptureSpan { range, capture });
    }
}
