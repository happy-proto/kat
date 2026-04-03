mod document_kind;
mod grammar_registry;
mod host_injections;
mod language_aliases;
mod language_runtime;
mod semantic_overlays;
mod sql_dialect;
mod terminal_background;
mod theme;

use std::{ops::Range, path::Path};

use anyhow::{Context, Result};
use tree_sitter::Parser;
use tree_sitter_highlight::{Highlight, HighlightEvent, Highlighter};

use crate::document_kind::{DocumentKind, yaml_document_kind};
use crate::grammar_registry::grammar;
use crate::host_injections::{InjectionCandidate, InjectionDecode, collect_injection_candidates};
use crate::language_aliases::{normalize_language_name, shebang_interpreter_name};
use crate::language_runtime::{global_highlight_name, runtime};
use crate::semantic_overlays::{
    debug_named_language_tree as debug_language_tree_impl, debug_semantics as debug_semantics_impl,
    github_actions_expression_spans, semantic_capture_spans,
};
use crate::sql_dialect::resolve_sql_runtime;
use crate::theme::{Theme, TokenStyle};

#[cfg(test)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SupportedLanguage {
    Bash,
    Batch,
    Css,
    Dockerfile,
    Fish,
    Go,
    GoMod,
    GoSum,
    GoWork,
    Graphql,
    Hcl,
    Html,
    Ignore,
    JavaScript,
    Just,
    Json,
    Markdown,
    Proto,
    Powershell,
    Python,
    Sql,
    Rust,
    Textproto,
    Toml,
    Yaml,
    Zsh,
}

pub fn render(source_path: Option<&Path>, source: &str) -> Result<String> {
    render_with_theme(source_path, source, &Theme::detect())
}

pub fn detected_language_name(source_path: Option<&Path>, source: &str) -> Option<&'static str> {
    detect_document_kind(source_path, source).map(DocumentKind::runtime_name)
}

#[cfg(test)]
fn detect_language(source_path: Option<&Path>, source: &str) -> Option<SupportedLanguage> {
    let document_kind = detect_document_kind(source_path, source)?;
    Some(match document_kind.runtime_name() {
        "json" => SupportedLanguage::Json,
        "bash" => SupportedLanguage::Bash,
        "batch" => SupportedLanguage::Batch,
        "dockerfile" => SupportedLanguage::Dockerfile,
        "fish" => SupportedLanguage::Fish,
        "zsh" => SupportedLanguage::Zsh,
        "toml" => SupportedLanguage::Toml,
        "yaml" => SupportedLanguage::Yaml,
        "hcl" => SupportedLanguage::Hcl,
        "rust" => SupportedLanguage::Rust,
        "python" => SupportedLanguage::Python,
        "go" => SupportedLanguage::Go,
        "gomod" => SupportedLanguage::GoMod,
        "gowork" => SupportedLanguage::GoWork,
        "gosum" => SupportedLanguage::GoSum,
        "graphql" => SupportedLanguage::Graphql,
        "sql" => SupportedLanguage::Sql,
        "textproto" => SupportedLanguage::Textproto,
        "html" => SupportedLanguage::Html,
        "ignore" => SupportedLanguage::Ignore,
        "css" => SupportedLanguage::Css,
        "javascript" => SupportedLanguage::JavaScript,
        "markdown" => SupportedLanguage::Markdown,
        "proto" => SupportedLanguage::Proto,
        "powershell" => SupportedLanguage::Powershell,
        "just" => SupportedLanguage::Just,
        other => panic!("unsupported test language mapping for {other}"),
    })
}

fn render_with_theme(source_path: Option<&Path>, source: &str, theme: &Theme) -> Result<String> {
    match detect_document_kind(source_path, source) {
        Some(document_kind) => highlight_document(document_kind, source_path, source, theme),
        None => Ok(source.to_owned()),
    }
}

pub fn highlight_json(source: &str) -> Result<String> {
    highlight_named_language("json", source, &Theme::detect())
}

pub fn highlight_ignore(source: &str) -> Result<String> {
    highlight_named_language("ignore", source, &Theme::detect())
}

pub fn highlight_bash(source: &str) -> Result<String> {
    highlight_named_language("bash", source, &Theme::detect())
}

pub fn highlight_batch(source: &str) -> Result<String> {
    highlight_named_language("batch", source, &Theme::detect())
}

pub fn highlight_dockerfile(source: &str) -> Result<String> {
    highlight_named_language("dockerfile", source, &Theme::detect())
}

pub fn highlight_toml(source: &str) -> Result<String> {
    highlight_named_language("toml", source, &Theme::detect())
}

pub fn highlight_fish(source: &str) -> Result<String> {
    highlight_named_language("fish", source, &Theme::detect())
}

pub fn highlight_zsh(source: &str) -> Result<String> {
    highlight_named_language("zsh", source, &Theme::detect())
}

pub fn highlight_powershell(source: &str) -> Result<String> {
    highlight_named_language("powershell", source, &Theme::detect())
}

pub fn highlight_yaml(source: &str) -> Result<String> {
    highlight_named_language("yaml", source, &Theme::detect())
}

pub fn highlight_hcl(source: &str) -> Result<String> {
    highlight_named_language("hcl", source, &Theme::detect())
}

pub fn highlight_rust(source: &str) -> Result<String> {
    highlight_named_language("rust", source, &Theme::detect())
}

pub fn highlight_just(source: &str) -> Result<String> {
    highlight_named_language("just", source, &Theme::detect())
}

pub fn highlight_python(source: &str) -> Result<String> {
    highlight_named_language("python", source, &Theme::detect())
}

pub fn highlight_go(source: &str) -> Result<String> {
    highlight_named_language("go", source, &Theme::detect())
}

pub fn highlight_gomod(source: &str) -> Result<String> {
    highlight_named_language("gomod", source, &Theme::detect())
}

pub fn highlight_gowork(source: &str) -> Result<String> {
    highlight_named_language("gowork", source, &Theme::detect())
}

pub fn highlight_gosum(source: &str) -> Result<String> {
    highlight_named_language("gosum", source, &Theme::detect())
}

pub fn highlight_sql(source: &str) -> Result<String> {
    highlight_named_language("sql", source, &Theme::detect())
}

pub fn highlight_html(source: &str) -> Result<String> {
    highlight_named_language("html", source, &Theme::detect())
}

pub fn highlight_css(source: &str) -> Result<String> {
    highlight_named_language("css", source, &Theme::detect())
}

pub fn highlight_graphql(source: &str) -> Result<String> {
    highlight_named_language("graphql", source, &Theme::detect())
}

pub fn highlight_proto(source: &str) -> Result<String> {
    highlight_named_language("proto", source, &Theme::detect())
}

pub fn highlight_textproto(source: &str) -> Result<String> {
    highlight_named_language("textproto", source, &Theme::detect())
}

pub fn highlight_javascript(source: &str) -> Result<String> {
    highlight_named_language("javascript", source, &Theme::detect())
}

pub fn highlight_markdown(source: &str) -> Result<String> {
    highlight_named_language("markdown", source, &Theme::detect())
}

pub fn debug_named_language_tree(language_name: &str, source: &str) -> Result<String> {
    debug_language_tree_impl(language_name, source)
}

pub fn debug_semantics(language_name: &str, source: &str) -> Result<String> {
    debug_semantics_impl(language_name, source)
}

pub fn debug_shell_semantics(language_name: &str, source: &str) -> Result<String> {
    debug_semantics(language_name, source)
}

fn highlight_document(
    document_kind: DocumentKind,
    source_path: Option<&Path>,
    source: &str,
    theme: &Theme,
) -> Result<String> {
    if document_kind.runtime_name() == "sql" {
        return highlight_named_language_with_path(document_kind, source_path, source, theme);
    }

    highlight_document_kind(document_kind, source_path, source, theme)
}

fn highlight_named_language(language_name: &str, source: &str, theme: &Theme) -> Result<String> {
    highlight_document_kind(plain_document_kind(language_name), None, source, theme)
}

fn highlight_document_kind(
    document_kind: DocumentKind,
    source_path: Option<&Path>,
    source: &str,
    theme: &Theme,
) -> Result<String> {
    highlight_named_language_with_path(document_kind, source_path, source, theme)
}

fn plain_document_kind(language_name: &str) -> DocumentKind {
    match language_name {
        "json" => DocumentKind::plain("json"),
        "ignore" => DocumentKind::plain("ignore"),
        "dockerfile" => DocumentKind::plain("dockerfile"),
        "bash" => DocumentKind::plain("bash"),
        "batch" => DocumentKind::plain("batch"),
        "fish" => DocumentKind::plain("fish"),
        "powershell" => DocumentKind::plain("powershell"),
        "zsh" => DocumentKind::plain("zsh"),
        "toml" => DocumentKind::plain("toml"),
        "yaml" => DocumentKind::plain("yaml"),
        "hcl" => DocumentKind::plain("hcl"),
        "rust" => DocumentKind::plain("rust"),
        "python" => DocumentKind::plain("python"),
        "go" => DocumentKind::plain("go"),
        "gomod" => DocumentKind::plain("gomod"),
        "gowork" => DocumentKind::plain("gowork"),
        "gosum" => DocumentKind::plain("gosum"),
        "proto" => DocumentKind::plain("proto"),
        "sql" => DocumentKind::plain("sql"),
        "sql_postgres" => DocumentKind::plain("sql_postgres"),
        "sql_mysql" => DocumentKind::plain("sql_mysql"),
        "sql_sqlite" => DocumentKind::plain("sql_sqlite"),
        "textproto" => DocumentKind::plain("textproto"),
        "html" => DocumentKind::plain("html"),
        "css" => DocumentKind::plain("css"),
        "javascript" => DocumentKind::plain("javascript"),
        "graphql" => DocumentKind::plain("graphql"),
        "regex" => DocumentKind::plain("regex"),
        "regex_javascript" => DocumentKind::plain("regex_javascript"),
        "regex_python" => DocumentKind::plain("regex_python"),
        "regex_rust" => DocumentKind::plain("regex_rust"),
        "regex_go" => DocumentKind::plain("regex_go"),
        "regex_posix" => DocumentKind::plain("regex_posix"),
        "jsdoc" => DocumentKind::plain("jsdoc"),
        "userscript_metadata" => DocumentKind::plain("userscript_metadata"),
        "markdown" => DocumentKind::plain("markdown"),
        "markdown_inline" => DocumentKind::plain("markdown_inline"),
        "just" => DocumentKind::plain("just"),
        other => panic!("unsupported runtime name {other}"),
    }
}

fn highlight_named_language_with_path(
    document_kind: DocumentKind,
    source_path: Option<&Path>,
    source: &str,
    theme: &Theme,
) -> Result<String> {
    let render_data = highlight_named_language_render_data(document_kind, source_path, source, theme)?;
    Ok(render_styled_spans(
        source,
        &render_data.spans,
        &render_data.line_pads,
        theme,
    ))
}

fn highlight_named_language_spans(
    document_kind: DocumentKind,
    source_path: Option<&Path>,
    source: &str,
    theme: &Theme,
) -> Result<Vec<StyledSpan>> {
    Ok(highlight_named_language_render_data(document_kind, source_path, source, theme)?.spans)
}

struct HighlightRenderData {
    spans: Vec<StyledSpan>,
    line_pads: Vec<LinePad>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct LinePad {
    line_end: usize,
    width: usize,
    style: TokenStyle,
}

fn highlight_named_language_render_data(
    document_kind: DocumentKind,
    source_path: Option<&Path>,
    source: &str,
    theme: &Theme,
) -> Result<HighlightRenderData> {
    let resolved_document_kind =
        resolve_highlight_document_kind(document_kind, source_path, source);
    let resolved_runtime_name = resolved_document_kind.runtime_name();
    let language_runtime = runtime(resolved_runtime_name)
        .with_context(|| format!("missing language runtime for {resolved_runtime_name}"))?;
    let mut highlighter = Highlighter::new();
    let events = highlighter
        .highlight(
            &language_runtime.flat_configuration,
            source.as_bytes(),
            None,
            |_| None,
        )
        .context("failed to highlight source")?;

    let mut spans = collect_styled_spans(source, events, theme)?;
    spans = overlay_semantic_captures(resolved_document_kind, source, spans, theme)?;
    let nested_regions =
        collect_top_level_injection_regions(resolved_document_kind, source, theme)?;

    for region in &nested_regions {
        spans = overlay_nested_region(spans, &region);
    }

    spans = overlay_nested_region_tint(spans, &nested_regions, theme);

    Ok(HighlightRenderData {
        line_pads: build_line_pads(source, &nested_regions, theme),
        spans,
    })
}

fn overlay_semantic_captures(
    document_kind: DocumentKind,
    source: &str,
    parent_spans: Vec<StyledSpan>,
    theme: &Theme,
) -> Result<Vec<StyledSpan>> {
    let invalid_style = theme.token_style_for("invalid.illegal.regex", "");
    let overlays = semantic_capture_spans(document_kind, source)?
        .into_iter()
        .filter_map(|span| {
            let text = &source[span.range.clone()];
            theme
                .token_style_for(span.capture, text)
                .map(|overlay_style| {
                    let style = match (
                        style_covering_span(&parent_spans, span.range.start, span.range.end),
                        invalid_style,
                    ) {
                        (Some(parent_style), Some(invalid_style))
                            if parent_style == invalid_style =>
                        {
                            Some(parent_style.merge(overlay_style))
                        }
                        _ => Some(overlay_style),
                    };
                    StyledSpan {
                        range: span.range,
                        style,
                    }
                })
        })
        .collect::<Vec<_>>();

    Ok(overlay_style_spans(parent_spans, overlays))
}

fn collect_styled_spans<'a>(
    source: &str,
    events: impl Iterator<Item = Result<HighlightEvent, tree_sitter_highlight::Error>> + 'a,
    theme: &Theme,
) -> Result<Vec<StyledSpan>> {
    let mut active_highlights: Vec<Highlight> = Vec::new();
    let mut spans = Vec::new();

    for event in events {
        match event.context("failed to process highlight event")? {
            HighlightEvent::Source { start, end } => {
                if start == end {
                    continue;
                }

                let text = &source[start..end];
                let style = theme
                    .merged_token_style_for(
                        active_highlights
                            .iter()
                            .map(|highlight| global_highlight_name(highlight.0)),
                        text,
                    )
                    .or_else(|| theme.default_style());
                push_span(&mut spans, start..end, style);
            }
            HighlightEvent::HighlightStart(highlight) => active_highlights.push(highlight),
            HighlightEvent::HighlightEnd => {
                active_highlights.pop();
            }
        }
    }

    if spans.is_empty() && !source.is_empty() {
        spans.push(StyledSpan {
            range: 0..source.len(),
            style: theme.default_style(),
        });
    }

    Ok(spans)
}

fn detect_document_kind(source_path: Option<&Path>, source: &str) -> Option<DocumentKind> {
    let just = grammar("just");
    if matches_path(just, source_path) {
        return Some(DocumentKind::plain("just"));
    }

    let toml = grammar("toml");
    if matches_path(toml, source_path) {
        return Some(DocumentKind::plain("toml"));
    }

    let yaml = grammar("yaml");
    if matches_path(yaml, source_path) {
        return Some(yaml_document_kind(source_path));
    }

    // TODO: Keep HCL as one generic runtime for now. If Terraform/tfvars or
    // Nomad later need different file detection or semantics, split that at the
    // detector/runtime-overlay layer instead of fragmenting the base grammar.
    let hcl = grammar("hcl");
    if matches_path(hcl, source_path) {
        return Some(DocumentKind::plain("hcl"));
    }

    let rust = grammar("rust");
    if matches_path(rust, source_path) {
        return Some(DocumentKind::plain("rust"));
    }

    let python = grammar("python");
    if matches_path(python, source_path) {
        return Some(DocumentKind::plain("python"));
    }

    let go = grammar("go");
    if matches_path(go, source_path) {
        return Some(DocumentKind::plain("go"));
    }

    let gomod = grammar("gomod");
    if matches_path(gomod, source_path) {
        return Some(DocumentKind::plain("gomod"));
    }

    let gowork = grammar("gowork");
    if matches_path(gowork, source_path) {
        return Some(DocumentKind::plain("gowork"));
    }

    let gosum = grammar("gosum");
    if matches_path(gosum, source_path) {
        return Some(DocumentKind::plain("gosum"));
    }

    let sql = grammar("sql");
    if matches_path(sql, source_path) {
        return Some(DocumentKind::plain("sql"));
    }

    let html = grammar("html");
    if matches_path(html, source_path) {
        return Some(DocumentKind::plain("html"));
    }

    let css = grammar("css");
    if matches_path(css, source_path) {
        return Some(DocumentKind::plain("css"));
    }

    let graphql = grammar("graphql");
    if matches_path(graphql, source_path) {
        return Some(DocumentKind::plain("graphql"));
    }

    let proto = grammar("proto");
    if matches_path(proto, source_path) {
        return Some(DocumentKind::plain("proto"));
    }

    let textproto = grammar("textproto");
    if matches_path(textproto, source_path) {
        return Some(DocumentKind::plain("textproto"));
    }

    let javascript = grammar("javascript");
    if matches_path(javascript, source_path) || matches_shebang(javascript, source) {
        return Some(DocumentKind::plain("javascript"));
    }

    let fish = grammar("fish");
    if matches_path(fish, source_path) || matches_shebang(fish, source) {
        return Some(DocumentKind::plain("fish"));
    }

    let zsh = grammar("zsh");
    if matches_path(zsh, source_path) || matches_shebang(zsh, source) {
        return Some(DocumentKind::plain("zsh"));
    }

    let powershell = grammar("powershell");
    if matches_path(powershell, source_path) || matches_shebang(powershell, source) {
        return Some(DocumentKind::plain("powershell"));
    }

    let batch = grammar("batch");
    if matches_path(batch, source_path) {
        return Some(DocumentKind::plain("batch"));
    }

    let markdown = grammar("markdown");
    if matches_path(markdown, source_path) {
        return Some(DocumentKind::plain("markdown"));
    }

    let json = grammar("json");
    if matches_path(json, source_path) || looks_like_json(source) {
        return Some(DocumentKind::plain("json"));
    }

    let ignore = grammar("ignore");
    if matches_path(ignore, source_path) {
        return Some(DocumentKind::plain("ignore"));
    }

    let dockerfile = grammar("dockerfile");
    if matches_path(dockerfile, source_path) {
        return Some(DocumentKind::plain("dockerfile"));
    }

    if source_path.is_none() && looks_like_graphql(source) {
        return Some(DocumentKind::plain("graphql"));
    }

    if source_path.is_none() && looks_like_sql(source) {
        return Some(DocumentKind::plain("sql"));
    }

    let bash = grammar("bash");
    if matches_path(bash, source_path) || matches_shebang(bash, source) {
        return Some(DocumentKind::plain("bash"));
    }

    None
}

fn matches_path(grammar: &grammar_registry::GrammarSpec, source_path: Option<&Path>) -> bool {
    source_path.is_some_and(|path| {
        path.file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| {
                grammar.filenames.iter().any(|filename| filename == name)
                    || grammar
                        .filename_prefixes
                        .iter()
                        .any(|prefix| name.starts_with(prefix))
            })
            || path
                .extension()
                .and_then(|extension| extension.to_str())
                .is_some_and(|extension| {
                    grammar
                        .extensions
                        .iter()
                        .any(|expected_extension| expected_extension == extension)
                })
    })
}

fn looks_like_json(source: &str) -> bool {
    let trimmed = source.trim_start();
    trimmed.starts_with('{')
}

fn looks_like_sql(source: &str) -> bool {
    let trimmed = source.trim_start();
    let upper = trimmed.to_ascii_uppercase();
    [
        "SELECT ",
        "WITH ",
        "INSERT ",
        "UPDATE ",
        "DELETE ",
        "CREATE ",
        "ALTER ",
        "DROP ",
        "TRUNCATE ",
        "MERGE ",
        "VACUUM",
        "PRAGMA ",
        "EXPLAIN ",
    ]
    .iter()
    .any(|prefix| upper.starts_with(prefix))
}

fn looks_like_graphql(source: &str) -> bool {
    let trimmed = source.trim_start();
    let lower = trimmed.to_ascii_lowercase();
    [
        "query ",
        "mutation ",
        "subscription ",
        "fragment ",
        "schema ",
        "type ",
        "interface ",
        "union ",
        "enum ",
        "input ",
        "directive ",
        "extend ",
    ]
    .iter()
    .any(|prefix| lower.starts_with(prefix))
}

fn matches_shebang(grammar: &grammar_registry::GrammarSpec, source: &str) -> bool {
    source.lines().next().is_some_and(|line| {
        if !line.starts_with("#!") {
            return false;
        }

        let Some(interpreter) = shebang_interpreter_name(line) else {
            return false;
        };
        let normalized = normalize_language_name(interpreter).unwrap_or(interpreter);

        normalized == grammar.name
            || grammar.shebang_substrings.iter().any(|alias| {
                let alias = alias.as_str();
                normalize_language_name(alias).unwrap_or(alias) == normalized
            })
    })
}

fn resolve_highlight_document_kind(
    document_kind: DocumentKind,
    source_path: Option<&Path>,
    source: &str,
) -> DocumentKind {
    match document_kind.runtime_name() {
        "sql" | "sql_postgres" | "sql_mysql" | "sql_sqlite" => DocumentKind::with_profile(
            resolve_sql_runtime(source_path, document_kind.runtime_name(), source),
            document_kind.profile(),
        ),
        _ => document_kind,
    }
}

#[derive(Debug)]
struct NestedRegion {
    source_ranges: Vec<Range<usize>>,
    block_ranges: Vec<Range<usize>>,
    overlays: Vec<StyledSpan>,
    merge_parent_styles: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct StyledSpan {
    range: Range<usize>,
    style: Option<TokenStyle>,
}

fn collect_top_level_injection_regions(
    document_kind: DocumentKind,
    source: &str,
    theme: &Theme,
) -> Result<Vec<NestedRegion>> {
    let runtime_name = document_kind.runtime_name();
    let language_runtime = runtime(runtime_name)
        .with_context(|| format!("missing language runtime for {runtime_name}"))?;
    if language_runtime.injections_query.trim().is_empty() {
        let mut parser = Parser::new();
        parser
            .set_language(&language_runtime.language)
            .with_context(|| format!("failed to set parser language for {runtime_name}"))?;
        let tree = parser
            .parse(source, None)
            .with_context(|| format!("failed to parse source for {runtime_name}"))?;
        return render_injection_candidates(
            source,
            theme,
            prune_to_top_level_injection_regions(collect_injection_candidates(
                document_kind,
                language_runtime,
                &tree,
                source,
            )?),
        );
    }

    let mut parser = Parser::new();
    parser
        .set_language(&language_runtime.language)
        .with_context(|| format!("failed to set parser language for {runtime_name}"))?;
    let tree = parser
        .parse(source, None)
        .with_context(|| format!("failed to parse source for {runtime_name}"))?;

    let candidates = collect_injection_candidates(document_kind, language_runtime, &tree, source)?;
    render_injection_candidates(
        source,
        theme,
        prune_to_top_level_injection_regions(merge_adjacent_combined_candidates(
            source, candidates,
        )),
    )
}

fn render_injection_candidates(
    source: &str,
    theme: &Theme,
    mut top_level: Vec<InjectionCandidate>,
) -> Result<Vec<NestedRegion>> {
    let mut rendered = Vec::with_capacity(top_level.len());

    for candidate in top_level.drain(..) {
        let normalized = candidate.language_name.as_str();
        let (virtual_source, source_map) = build_virtual_source(
            source,
            &candidate.ranges,
            candidate.is_combined,
            candidate.decode,
        );
        rendered.push(NestedRegion {
            block_ranges: build_block_ranges(source, &candidate.ranges),
            source_ranges: candidate.ranges,
            overlays: map_virtual_spans_to_source(
                &render_virtual_injection_spans(
                    normalized,
                    &virtual_source,
                    candidate.highlight_github_expressions,
                    theme,
                )?,
                &source_map,
            ),
            merge_parent_styles: candidate.merge_parent_styles,
        });
    }

    Ok(rendered)
}

fn render_virtual_injection_spans(
    language_name: &str,
    source: &str,
    highlight_github_expressions: bool,
    theme: &Theme,
) -> Result<Vec<StyledSpan>> {
    let mut spans =
        highlight_named_language_spans(plain_document_kind(language_name), None, source, theme)?;

    if highlight_github_expressions {
        spans = overlay_github_actions_expression_styles(source, spans, theme);
    }

    Ok(spans)
}

fn overlay_github_actions_expression_styles(
    source: &str,
    parent_spans: Vec<StyledSpan>,
    theme: &Theme,
) -> Vec<StyledSpan> {
    let overlays = github_actions_expression_spans(source)
        .into_iter()
        .filter_map(|span| {
            let text = &source[span.range.clone()];
            theme
                .token_style_for(span.capture, text)
                .map(|style| StyledSpan {
                    range: span.range,
                    style: Some(style),
                })
        })
        .collect();

    overlay_style_spans(parent_spans, overlays)
}

fn prune_to_top_level_injection_regions(
    mut candidates: Vec<InjectionCandidate>,
) -> Vec<InjectionCandidate> {
    candidates.sort_by(|left, right| {
        candidate_extent(&left.ranges)
            .start
            .cmp(&candidate_extent(&right.ranges).start)
            .then(
                candidate_extent(&right.ranges)
                    .end
                    .cmp(&candidate_extent(&left.ranges).end),
            )
    });

    let mut pruned: Vec<InjectionCandidate> = Vec::new();

    for candidate in candidates {
        if let Some(last) = pruned.last() {
            if candidate_extent(&candidate.ranges).start < candidate_extent(&last.ranges).end {
                continue;
            }
        }

        pruned.push(candidate);
    }

    pruned
}

fn merge_adjacent_combined_candidates(
    source: &str,
    mut candidates: Vec<InjectionCandidate>,
) -> Vec<InjectionCandidate> {
    candidates.sort_by(|left, right| {
        candidate_extent(&left.ranges)
            .start
            .cmp(&candidate_extent(&right.ranges).start)
            .then(
                candidate_extent(&left.ranges)
                    .end
                    .cmp(&candidate_extent(&right.ranges).end),
            )
    });

    let mut merged: Vec<InjectionCandidate> = Vec::new();

    for candidate in candidates {
        if let Some(last) = merged.last_mut() {
            if last.is_combined
                && candidate.is_combined
                && last.language_name == candidate.language_name
                && last.merge_parent_styles == candidate.merge_parent_styles
                && can_merge_combined_candidates(source, last, &candidate)
            {
                last.ranges.extend(candidate.ranges);
                last.ranges = normalize_ranges(std::mem::take(&mut last.ranges));
                continue;
            }
        }

        merged.push(candidate);
    }

    merged
}

fn can_merge_combined_candidates(
    source: &str,
    left: &InjectionCandidate,
    right: &InjectionCandidate,
) -> bool {
    let left_extent = candidate_extent(&left.ranges);
    let right_extent = candidate_extent(&right.ranges);
    if left_extent.end > right_extent.start {
        return false;
    }

    source[left_extent.end..right_extent.start]
        .chars()
        .all(|ch| ch.is_whitespace() || matches!(ch, '/' | '*' | '!'))
}

fn candidate_extent(ranges: &[Range<usize>]) -> Range<usize> {
    let start = ranges.first().map_or(0, |range| range.start);
    let end = ranges.last().map_or(start, |range| range.end);
    start..end
}

fn normalize_ranges(mut ranges: Vec<Range<usize>>) -> Vec<Range<usize>> {
    ranges.retain(|range| range.start < range.end);
    ranges.sort_by(|left, right| left.start.cmp(&right.start).then(left.end.cmp(&right.end)));
    ranges
}

fn push_span(spans: &mut Vec<StyledSpan>, range: Range<usize>, style: Option<TokenStyle>) {
    if range.start >= range.end {
        return;
    }

    if let Some(last) = spans.last_mut() {
        if last.range.end == range.start && last.style == style {
            last.range.end = range.end;
            return;
        }
    }

    spans.push(StyledSpan { range, style });
}

fn build_virtual_source(
    source: &str,
    ranges: &[Range<usize>],
    strip_shared_indent: bool,
    decode: InjectionDecode,
) -> (String, Vec<Range<usize>>) {
    let mut virtual_source = String::new();
    let mut source_map = Vec::new();
    let shared_indent = if strip_shared_indent {
        shared_leading_indent(source, ranges)
    } else {
        0
    };

    for (index, range) in ranges.iter().enumerate() {
        if index > 0 && !virtual_source.ends_with('\n') {
            virtual_source.push('\n');
            source_map.push(0..0);
        }

        let source_slice = &source[range.clone()];
        let trim_len = source_slice
            .chars()
            .take_while(|ch| matches!(ch, ' ' | '\t'))
            .map(char::len_utf8)
            .take(shared_indent)
            .sum::<usize>();
        let trimmed_range = (range.start + trim_len)..range.end;

        append_injection_content(
            source,
            trimmed_range,
            decode,
            &mut virtual_source,
            &mut source_map,
        );
    }

    (virtual_source, source_map)
}

fn build_block_ranges(source: &str, ranges: &[Range<usize>]) -> Vec<Range<usize>> {
    let left_indent = shared_leading_indent(source, ranges);
    let mut block_ranges = Vec::new();

    for range in ranges {
        let mut line_start = line_start_offset(source, range.start);
        let line_end_limit = range.end;

        while line_start < line_end_limit {
            let line_end = source[line_start..]
                .find('\n')
                .map(|offset| line_start + offset)
                .unwrap_or(source.len());
            let line = &source[line_start..line_end];

            if !line.trim().is_empty() {
                let indent_bytes = line
                    .chars()
                    .take_while(|ch| matches!(ch, ' ' | '\t'))
                    .map(char::len_utf8)
                    .take(left_indent)
                    .sum::<usize>();
                let block_start = (line_start + indent_bytes).min(line_end);
                if block_start < line_end {
                    block_ranges.push(block_start..line_end);
                }
            }

            if line_end >= line_end_limit || line_end == source.len() {
                break;
            }
            line_start = line_end + 1;
        }
    }

    normalize_ranges(block_ranges)
}

fn line_start_offset(source: &str, offset: usize) -> usize {
    source[..offset]
        .rfind('\n')
        .map(|index| index + 1)
        .unwrap_or(0)
}

fn append_injection_content(
    source: &str,
    range: Range<usize>,
    decode: InjectionDecode,
    virtual_source: &mut String,
    source_map: &mut Vec<Range<usize>>,
) {
    match decode {
        InjectionDecode::None => append_raw_range(source, range, virtual_source, source_map),
        InjectionDecode::JavaScriptLiteral => {
            decode_javascript_string_literal(source, range, virtual_source, source_map)
        }
        InjectionDecode::JavaScriptString => {
            decode_javascript_string_content(source, range, virtual_source, source_map)
        }
        InjectionDecode::PythonString => {
            decode_python_string_content(source, range, virtual_source, source_map)
        }
        InjectionDecode::RustString => {
            decode_rust_string_content(source, range, virtual_source, source_map)
        }
        InjectionDecode::GoString => {
            decode_go_string_content(source, range, virtual_source, source_map)
        }
    }
}

fn append_raw_range(
    source: &str,
    range: Range<usize>,
    virtual_source: &mut String,
    source_map: &mut Vec<Range<usize>>,
) {
    let slice = &source[range.clone()];
    virtual_source.push_str(slice);
    for offset in 0..slice.len() {
        source_map.push((range.start + offset)..(range.start + offset + 1));
    }
}

fn append_mapped_text(
    virtual_source: &mut String,
    source_map: &mut Vec<Range<usize>>,
    source_range: Range<usize>,
    text: &str,
) {
    virtual_source.push_str(text);
    for _ in 0..text.len() {
        source_map.push(source_range.clone());
    }
}

fn decode_javascript_string_content(
    source: &str,
    range: Range<usize>,
    virtual_source: &mut String,
    source_map: &mut Vec<Range<usize>>,
) {
    match surrounding_delimiter(source, &range) {
        Some('\'') | Some('"') => decode_backslash_escaped_range(
            source,
            range,
            DecodeFlavor::JavaScript,
            virtual_source,
            source_map,
        ),
        _ => append_raw_range(source, range, virtual_source, source_map),
    }
}

fn decode_python_string_content(
    source: &str,
    range: Range<usize>,
    virtual_source: &mut String,
    source_map: &mut Vec<Range<usize>>,
) {
    if python_string_is_raw(source, &range) {
        append_raw_range(source, range, virtual_source, source_map);
    } else {
        decode_backslash_escaped_range(
            source,
            range,
            DecodeFlavor::Python,
            virtual_source,
            source_map,
        );
    }
}

fn decode_rust_string_content(
    source: &str,
    range: Range<usize>,
    virtual_source: &mut String,
    source_map: &mut Vec<Range<usize>>,
) {
    if rust_string_is_raw(source, &range) {
        append_raw_range(source, range, virtual_source, source_map);
    } else {
        decode_backslash_escaped_range(
            source,
            range,
            DecodeFlavor::Rust,
            virtual_source,
            source_map,
        );
    }
}

fn decode_go_string_content(
    source: &str,
    range: Range<usize>,
    virtual_source: &mut String,
    source_map: &mut Vec<Range<usize>>,
) {
    match surrounding_delimiter(source, &range) {
        Some('`') => append_raw_range(source, range, virtual_source, source_map),
        Some('"') => decode_backslash_escaped_range(
            source,
            range,
            DecodeFlavor::Go,
            virtual_source,
            source_map,
        ),
        _ => append_raw_range(source, range, virtual_source, source_map),
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum DecodeFlavor {
    JavaScript,
    Python,
    Rust,
    Go,
}

fn decode_backslash_escaped_range(
    source: &str,
    range: Range<usize>,
    flavor: DecodeFlavor,
    virtual_source: &mut String,
    source_map: &mut Vec<Range<usize>>,
) {
    let slice = &source[range.clone()];
    let bytes = slice.as_bytes();
    let mut cursor = 0;
    let mut plain_start = 0;

    while cursor < bytes.len() {
        if bytes[cursor] != b'\\' {
            cursor += 1;
            continue;
        }

        if plain_start < cursor {
            append_raw_range(
                source,
                (range.start + plain_start)..(range.start + cursor),
                virtual_source,
                source_map,
            );
        }

        let escape_start = cursor;
        cursor += 1;
        if cursor >= bytes.len() {
            append_mapped_text(
                virtual_source,
                source_map,
                (range.start + escape_start)..(range.start + cursor),
                "\\",
            );
            break;
        }

        let next = bytes[cursor] as char;
        let decoded = match next {
            '\\' => Some(("\\".to_owned(), 1)),
            '\'' => Some(("\'".to_owned(), 1)),
            '"' => Some(("\"".to_owned(), 1)),
            'n' => Some(("\n".to_owned(), 1)),
            'r' => Some(("\r".to_owned(), 1)),
            't' => Some(("\t".to_owned(), 1)),
            'b' => Some(("\u{0008}".to_owned(), 1)),
            'f' => Some(("\u{000C}".to_owned(), 1)),
            'v' if matches!(flavor, DecodeFlavor::JavaScript | DecodeFlavor::Go) => {
                Some(("\u{000B}".to_owned(), 1))
            }
            '0' => Some(("\0".to_owned(), 1)),
            'x' => decode_fixed_width_hex(bytes, cursor + 1, 2),
            'u' => match flavor {
                DecodeFlavor::Rust | DecodeFlavor::JavaScript
                    if bytes.get(cursor + 1) == Some(&b'{') =>
                {
                    decode_braced_hex(bytes, cursor + 2)
                }
                DecodeFlavor::Go if bytes.get(cursor + 1) == Some(&b'{') => {
                    decode_braced_hex(bytes, cursor + 2)
                }
                _ => decode_fixed_width_hex(bytes, cursor + 1, 4),
            },
            'U' if flavor == DecodeFlavor::Python => decode_fixed_width_hex(bytes, cursor + 1, 8),
            '\n' if matches!(flavor, DecodeFlavor::JavaScript | DecodeFlavor::Python) => {
                Some((String::new(), 1))
            }
            _ => None,
        };

        if let Some((text, consumed_after_marker)) = decoded {
            let consumed = 1 + consumed_after_marker;
            let escape_range =
                (range.start + escape_start)..(range.start + escape_start + consumed);
            append_mapped_text(virtual_source, source_map, escape_range, &text);
            cursor = escape_start + consumed;
        } else {
            append_raw_range(
                source,
                (range.start + escape_start)..(range.start + escape_start + 2),
                virtual_source,
                source_map,
            );
            cursor = escape_start + 2;
        }

        plain_start = cursor;
    }

    if plain_start < bytes.len() {
        append_raw_range(
            source,
            (range.start + plain_start)..range.end,
            virtual_source,
            source_map,
        );
    }
}

fn decode_fixed_width_hex(bytes: &[u8], start: usize, width: usize) -> Option<(String, usize)> {
    let digits = bytes.get(start..start + width)?;
    if !digits.iter().all(u8::is_ascii_hexdigit) {
        return None;
    }

    let value = u32::from_str_radix(std::str::from_utf8(digits).ok()?, 16).ok()?;
    let ch = char::from_u32(value)?;
    Some((ch.to_string(), 1 + width))
}

fn decode_braced_hex(bytes: &[u8], start: usize) -> Option<(String, usize)> {
    let rest = bytes.get(start..)?;
    let close_offset = rest.iter().position(|byte| *byte == b'}')?;
    let digits = &rest[..close_offset];
    if digits.is_empty() || !digits.iter().all(u8::is_ascii_hexdigit) {
        return None;
    }

    let value = u32::from_str_radix(std::str::from_utf8(digits).ok()?, 16).ok()?;
    let ch = char::from_u32(value)?;
    Some((ch.to_string(), close_offset + 2))
}

fn surrounding_delimiter(source: &str, range: &Range<usize>) -> Option<char> {
    source[..range.start].chars().next_back()
}

fn python_string_is_raw(source: &str, range: &Range<usize>) -> bool {
    let prefix = literal_prefix(source, range, &['"', '\'']);
    prefix.chars().any(|ch| matches!(ch, 'r' | 'R'))
}

fn rust_string_is_raw(source: &str, range: &Range<usize>) -> bool {
    let prefix = literal_prefix(source, range, &['"']);
    prefix.chars().any(|ch| matches!(ch, 'r' | 'R'))
}

fn literal_prefix(source: &str, range: &Range<usize>, delimiters: &[char]) -> String {
    let mut prefix = String::new();
    let mut cursor = range.start;

    while cursor > 0 {
        let previous = source[..cursor].chars().next_back().unwrap_or_default();
        if previous.is_ascii_alphabetic() || previous == '#' || delimiters.contains(&previous) {
            prefix.insert(0, previous);
            cursor -= previous.len_utf8();
            continue;
        }
        break;
    }

    prefix
}

fn shared_leading_indent(source: &str, ranges: &[Range<usize>]) -> usize {
    ranges
        .iter()
        .filter_map(|range| {
            let text = &source[range.clone()];
            if text.trim().is_empty() {
                return None;
            }

            Some(
                text.chars()
                    .take_while(|ch| matches!(ch, ' ' | '\t'))
                    .count(),
            )
        })
        .min()
        .unwrap_or(0)
}

fn map_virtual_spans_to_source(
    virtual_spans: &[StyledSpan],
    source_map: &[Range<usize>],
) -> Vec<StyledSpan> {
    let mut mapped = Vec::new();

    for span in virtual_spans {
        let mut active_range: Option<Range<usize>> = None;

        for index in span.range.clone() {
            let mapped_range = source_map.get(index).cloned().unwrap_or(0..0);
            if mapped_range.start == mapped_range.end {
                if let Some(range) = active_range.take() {
                    push_span(&mut mapped, range, span.style);
                }
                continue;
            }

            match &mut active_range {
                Some(current) if *current == mapped_range || current.end == mapped_range.start => {
                    current.end = mapped_range.end;
                }
                Some(current) => {
                    let finished = current.clone();
                    *current = mapped_range;
                    push_span(&mut mapped, finished, span.style);
                }
                None => active_range = Some(mapped_range),
            }
        }

        if let Some(range) = active_range {
            push_span(&mut mapped, range, span.style);
        }
    }

    mapped
}

fn overlay_nested_region(parent_spans: Vec<StyledSpan>, region: &NestedRegion) -> Vec<StyledSpan> {
    let source_len = parent_spans.last().map_or(0, |span| span.range.end);
    let mut boundaries = vec![0, source_len];

    for span in &parent_spans {
        boundaries.push(span.range.start);
        boundaries.push(span.range.end);
    }

    for range in &region.source_ranges {
        boundaries.push(range.start);
        boundaries.push(range.end);
    }

    for span in &region.overlays {
        boundaries.push(span.range.start);
        boundaries.push(span.range.end);
    }

    boundaries.sort_unstable();
    boundaries.dedup();

    let mut result = Vec::new();

    for window in boundaries.windows(2) {
        let start = window[0];
        let end = window[1];
        if start == end {
            continue;
        }

        let parent_style = style_covering_span(&parent_spans, start, end);
        let resolved_style = if point_in_ranges(start, &region.source_ranges)
            && point_in_ranges(end.saturating_sub(1), &region.source_ranges)
        {
            let child_style = style_covering_span(&region.overlays, start, end);
            match (region.merge_parent_styles, parent_style, child_style) {
                (true, Some(parent), Some(child)) => Some(parent.merge(child)),
                (true, Some(parent), None) => Some(parent),
                (true, None, Some(child)) => Some(child),
                (true, None, None) => None,
                (false, _, Some(child)) => Some(child),
                (false, parent, None) => parent,
            }
        } else {
            parent_style
        };

        push_span(&mut result, start..end, resolved_style);
    }

    result
}

fn overlay_style_spans(
    parent_spans: Vec<StyledSpan>,
    overlay_spans: Vec<StyledSpan>,
) -> Vec<StyledSpan> {
    if overlay_spans.is_empty() {
        return parent_spans;
    }

    let source_len = parent_spans.last().map_or(0, |span| span.range.end);
    let mut boundaries = vec![0, source_len];

    for span in &parent_spans {
        boundaries.push(span.range.start);
        boundaries.push(span.range.end);
    }

    for span in &overlay_spans {
        boundaries.push(span.range.start);
        boundaries.push(span.range.end);
    }

    boundaries.sort_unstable();
    boundaries.dedup();

    let mut result = Vec::new();

    for window in boundaries.windows(2) {
        let start = window[0];
        let end = window[1];
        if start == end {
            continue;
        }

        let style = style_covering_span(&overlay_spans, start, end)
            .or_else(|| style_covering_span(&parent_spans, start, end));
        push_span(&mut result, start..end, style);
    }

    result
}

fn overlay_nested_region_tint(
    parent_spans: Vec<StyledSpan>,
    nested_regions: &[NestedRegion],
    theme: &Theme,
) -> Vec<StyledSpan> {
    let Some(tint_style) = theme.nested_region_tint() else {
        return parent_spans;
    };

    let mut overlays = Vec::new();
    for region in nested_regions {
        for range in &region.block_ranges {
            push_span(&mut overlays, range.clone(), Some(tint_style));
        }
    }
    merge_background_overlay_spans(parent_spans, overlays)
}

fn merge_background_overlay_spans(
    parent_spans: Vec<StyledSpan>,
    overlay_spans: Vec<StyledSpan>,
) -> Vec<StyledSpan> {
    if overlay_spans.is_empty() {
        return parent_spans;
    }

    let source_len = parent_spans.last().map_or(0, |span| span.range.end);
    let mut boundaries = vec![0, source_len];

    for span in &parent_spans {
        boundaries.push(span.range.start);
        boundaries.push(span.range.end);
    }

    for span in &overlay_spans {
        boundaries.push(span.range.start);
        boundaries.push(span.range.end);
    }

    boundaries.sort_unstable();
    boundaries.dedup();

    let mut result = Vec::new();

    for window in boundaries.windows(2) {
        let start = window[0];
        let end = window[1];
        if start == end {
            continue;
        }

        let style = match (
            style_covering_span(&parent_spans, start, end),
            style_covering_span(&overlay_spans, start, end),
        ) {
            (Some(parent), Some(overlay)) => Some(parent.merge(overlay)),
            (Some(parent), None) => Some(parent),
            (None, Some(overlay)) => Some(overlay),
            (None, None) => None,
        };
        push_span(&mut result, start..end, style);
    }

    result
}

fn point_in_ranges(point: usize, ranges: &[Range<usize>]) -> bool {
    ranges
        .iter()
        .any(|range| point >= range.start && point < range.end)
}

fn style_covering_span(spans: &[StyledSpan], start: usize, end: usize) -> Option<TokenStyle> {
    spans
        .iter()
        .find(|span| span.range.start <= start && span.range.end >= end)
        .and_then(|span| span.style)
}

fn build_line_pads(source: &str, nested_regions: &[NestedRegion], theme: &Theme) -> Vec<LinePad> {
    let Some(style) = theme.nested_region_tint() else {
        return Vec::new();
    };

    let mut pads = Vec::new();

    for region in nested_regions {
        let block_width = region
            .block_ranges
            .iter()
            .map(|range| range.end.saturating_sub(line_start_offset(source, range.start)))
            .max()
            .unwrap_or(0);

        for range in &region.block_ranges {
            let line_start = line_start_offset(source, range.start);
            let line_width = range.end.saturating_sub(line_start);
            let pad_width = block_width.saturating_sub(line_width);
            if pad_width > 0 {
                pads.push(LinePad {
                    line_end: range.end,
                    width: pad_width,
                    style,
                });
            }
        }
    }

    pads.sort_by_key(|pad| pad.line_end);
    pads
}

fn render_styled_spans(
    source: &str,
    spans: &[StyledSpan],
    line_pads: &[LinePad],
    theme: &Theme,
) -> String {
    let mut rendered = String::with_capacity(source.len() + source.len() / 8);
    let mut remaining_pads = line_pads.iter().peekable();

    for span in spans {
        let segment = &source[span.range.clone()];
        if let Some(style) = span.style {
            let style_prefix = style.to_style(theme.color_mode()).render().to_string();
            let mut segment_start = 0;

            while let Some(relative_newline) = segment[segment_start..].find('\n') {
                let newline_index = segment_start + relative_newline;
                let line_end = span.range.start + newline_index;
                rendered.push_str(&style_prefix);
                rendered.push_str(&segment[segment_start..newline_index]);
                rendered.push_str("\x1b[0m");
                emit_line_pad(&mut rendered, &mut remaining_pads, line_end, theme);
                rendered.push('\n');
                segment_start = newline_index + 1;
            }

            if segment_start < segment.len() {
                rendered.push_str(&style_prefix);
                rendered.push_str(&segment[segment_start..]);
                rendered.push_str("\x1b[0m");
                if span.range.end == source.len() {
                    emit_line_pad(&mut rendered, &mut remaining_pads, source.len(), theme);
                }
            }
        } else {
            let mut segment_start = 0;

            while let Some(relative_newline) = segment[segment_start..].find('\n') {
                let newline_index = segment_start + relative_newline;
                let line_end = span.range.start + newline_index;
                rendered.push_str(&segment[segment_start..newline_index]);
                emit_line_pad(&mut rendered, &mut remaining_pads, line_end, theme);
                rendered.push('\n');
                segment_start = newline_index + 1;
            }

            if segment_start < segment.len() {
                rendered.push_str(&segment[segment_start..]);
                if span.range.end == source.len() {
                    emit_line_pad(&mut rendered, &mut remaining_pads, source.len(), theme);
                }
            }
        }
    }

    rendered
}

fn emit_line_pad<'a>(
    rendered: &mut String,
    remaining_pads: &mut std::iter::Peekable<std::slice::Iter<'a, LinePad>>,
    line_end: usize,
    theme: &Theme,
) {
    while let Some(pad) = remaining_pads.peek() {
        if pad.line_end != line_end {
            break;
        }

        let pad = **pad;
        rendered.push_str(&pad.style.to_style(theme.color_mode()).render().to_string());
        rendered.push_str(&" ".repeat(pad.width));
        rendered.push_str("\x1b[0m");
        remaining_pads.next();
    }
}
#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::{Path, PathBuf},
    };

    use super::{
        SupportedLanguage, collect_top_level_injection_regions, debug_semantics,
        detect_document_kind, detect_language, highlight_named_language, render_with_theme,
    };
    use crate::{
        document_kind::{DocumentProfile, yaml_document_kind},
        sql_dialect::detect_sql_dialect,
        theme::{ColorMode, Theme},
    };
    use anstyle::RgbColor;

    struct FixtureCase {
        relative_path: &'static str,
        expect_highlight: bool,
        expected_fragments: &'static [&'static str],
    }

    const FIXTURE_CASES: &[FixtureCase] = &[
        FixtureCase {
            relative_path: "json/basic.json",
            expect_highlight: true,
            expected_fragments: &["\"name\"", "// comment", "true", "3"],
        },
        FixtureCase {
            relative_path: "json/rich.json",
            expect_highlight: true,
            expected_fragments: &["\"theme\"", "\"Dracula\"", "true", "null", "second"],
        },
        FixtureCase {
            relative_path: "bash/script.sh",
            expect_highlight: true,
            expected_fragments: &["if", "echo", "\"kat\""],
        },
        FixtureCase {
            relative_path: "bash/rich.sh",
            expect_highlight: true,
            expected_fragments: &["#!/usr/bin/env bash", "pipefail", "printf", "=~", "second"],
        },
        FixtureCase {
            relative_path: "bash/sql_heredoc.sh",
            expect_highlight: true,
            expected_fragments: &["psql", "RETURNING", "slug", "sqlite3", "AUTO_INCREMENT"],
        },
        FixtureCase {
            relative_path: "fish/rich.fish",
            expect_highlight: true,
            expected_fragments: &[
                "#!/usr/bin/env fish",
                "function",
                "set",
                "string",
                "render_theme",
            ],
        },
        FixtureCase {
            relative_path: "zsh/rich.zsh",
            expect_highlight: true,
            expected_fragments: &["#!/usr/bin/env zsh", "emulate", "autoload", "[[", "(#i)"],
        },
        FixtureCase {
            relative_path: "toml/Cargo.toml",
            expect_highlight: true,
            expected_fragments: &["package", "name", "\"kat\""],
        },
        FixtureCase {
            relative_path: "toml/Cargo.lock",
            expect_highlight: true,
            expected_fragments: &["version", "package", "\"kat\""],
        },
        FixtureCase {
            relative_path: "toml/rich.toml",
            expect_highlight: true,
            expected_fragments: &["display-name", "released_at", "meta", "showcase"],
        },
        FixtureCase {
            relative_path: "yaml/site-config.yaml",
            expect_highlight: true,
            expected_fragments: &["title", "Dracula", "true", "theme"],
        },
        FixtureCase {
            relative_path: "yaml/actions.yaml",
            expect_highlight: true,
            expected_fragments: &[
                "actions/github-script",
                "console",
                "log",
                "&defaults",
                "*defaults",
            ],
        },
        FixtureCase {
            relative_path: "proto/schema.proto",
            expect_highlight: true,
            expected_fragments: &[
                "syntax",
                "proto3",
                "message",
                "ThemePreview",
                "service",
                "GetTheme",
            ],
        },
        FixtureCase {
            relative_path: "textproto/theme.textproto",
            expect_highlight: true,
            expected_fragments: &["name", "\"Dracula\"", "enabled", "true", "tags"],
        },
        FixtureCase {
            relative_path: "hcl/nomad-job.hcl",
            expect_highlight: true,
            expected_fragments: &[
                "job",
                "datacenters",
                "driver",
                "ENABLE_METRICS",
                "destination",
                "endif",
            ],
        },
        FixtureCase {
            relative_path: "rust/main.rs",
            expect_highlight: true,
            expected_fragments: &["fn", "42", "println"],
        },
        FixtureCase {
            relative_path: "rust/rich.rs",
            expect_highlight: true,
            expected_fragments: &["Renderable", "render", "macro_rules!", "themed!", "true"],
        },
        FixtureCase {
            relative_path: "rust/injections.rs",
            expect_highlight: true,
            expected_fragments: &["SELECT", "regex!", "Regex", "new", "kind"],
        },
        FixtureCase {
            relative_path: "rust/doc_comments.rs",
            expect_highlight: true,
            expected_fragments: &["Preview", "markdown", "list item", "inline code"],
        },
        FixtureCase {
            relative_path: "rust/doc_comments_nested.rs",
            expect_highlight: true,
            expected_fragments: &["Guide", "nested", "Nested", "return", "42"],
        },
        FixtureCase {
            relative_path: "python/main.py",
            expect_highlight: true,
            expected_fragments: &["class", "def", "return", "42"],
        },
        FixtureCase {
            relative_path: "python/rich_features.py",
            expect_highlight: true,
            expected_fragments: &["dataclass", "ThemePreview", "render", "self", "\"kat\""],
        },
        FixtureCase {
            relative_path: "python/advanced.py",
            expect_highlight: true,
            expected_fragments: &[
                "classmethod",
                "build_default",
                "__init__",
                "property",
                "isinstance",
            ],
        },
        FixtureCase {
            relative_path: "python/injections.py",
            expect_highlight: true,
            expected_fragments: &["CREATE", "AUTOINCREMENT", "WITHOUT", "re", "compile"],
        },
        FixtureCase {
            relative_path: "go/rich.go",
            expect_highlight: true,
            expected_fragments: &[
                "package",
                "Renderer",
                "DefaultTheme",
                "regexp",
                "SELECT",
                "RETURNING",
                "AUTO_INCREMENT",
                "WITHOUT",
                "themes",
                "MustCompile",
                "console",
                "log",
            ],
        },
        FixtureCase {
            relative_path: "go/go.mod",
            expect_highlight: true,
            expected_fragments: &[
                "module",
                "github.com/dcjanus/kat",
                "toolchain",
                "go1.24.1",
                "replace",
                "../forks/lipgloss",
                "retract",
            ],
        },
        FixtureCase {
            relative_path: "go/go.work",
            expect_highlight: true,
            expected_fragments: &[
                "go",
                "1.24.0",
                "use",
                "../shared/theme-kit",
                "replace",
                "github.com/dcjanus/theme-kit",
            ],
        },
        FixtureCase {
            relative_path: "go/go.sum",
            expect_highlight: true,
            expected_fragments: &[
                "github.com/charmbracelet/lipgloss",
                "v1.0.0",
                "go.mod",
                "h1",
                "+incompatible",
            ],
        },
        FixtureCase {
            relative_path: "html/index.html",
            expect_highlight: true,
            expected_fragments: &["style", "--accent", "@param", "console", "log"],
        },
        FixtureCase {
            relative_path: "html/nested_regions.html",
            expect_highlight: true,
            expected_fragments: &["style", "--accent", "@param", "console", "greet"],
        },
        FixtureCase {
            relative_path: "html/attribute_injections.html",
            expect_highlight: true,
            expected_fragments: &["style", "--accent", "onclick", "console", "clicked"],
        },
        FixtureCase {
            relative_path: "html/rich.html",
            expect_highlight: true,
            expected_fragments: &["Tom", "&amp;", "theme-card", "accent", "Dracula"],
        },
        FixtureCase {
            relative_path: "css/theme.css",
            expect_highlight: true,
            expected_fragments: &["@media", "--accent", "panel"],
        },
        FixtureCase {
            relative_path: "css/rich.css",
            expect_highlight: true,
            expected_fragments: &["@supports", "app", "panel", "before", "1.5", "rem"],
        },
        FixtureCase {
            relative_path: "javascript/report.js",
            expect_highlight: true,
            expected_fragments: &["const", "render", "console", "log"],
        },
        FixtureCase {
            relative_path: "javascript/rich.js",
            expect_highlight: true,
            expected_fragments: &[
                "#theme",
                "constructor",
                "/dracula/",
                "gi",
                "html",
                "ThemePreview",
            ],
        },
        FixtureCase {
            relative_path: "javascript/component.jsx",
            expect_highlight: true,
            expected_fragments: &["Panel", "section", "className", "PreviewCard", "title"],
        },
        FixtureCase {
            relative_path: "javascript/tagged_templates.js",
            expect_highlight: true,
            expected_fragments: &["css", "--accent", "\"enabled\"", "html", "section"],
        },
        FixtureCase {
            relative_path: "javascript/graphql.js",
            expect_highlight: true,
            expected_fragments: &["fragment", "ThemeFields", "ThemeBySlug", "$", "themes"],
        },
        FixtureCase {
            relative_path: "javascript/injections.js",
            expect_highlight: true,
            expected_fragments: &["SELECT", "snapshotQuery", "RETURNING", "kind", "slug"],
        },
        FixtureCase {
            relative_path: "javascript/userscript.user.js",
            expect_highlight: true,
            expected_fragments: &[
                "==UserScript==",
                "name",
                "GM_addStyle",
                "https://example.com/lib.js",
                "document-start",
            ],
        },
        FixtureCase {
            relative_path: "markdown/sample.md",
            expect_highlight: true,
            expected_fragments: &[
                "Preview",
                "layout",
                "\"solarized\"",
                "emphasis",
                "strong",
                "strikethrough",
                "done item",
                "pending item",
                "Column",
                "site",
                "let value = 42",
                "```",
                "return",
                "\"mode\"",
                "\"showcase\"",
            ],
        },
        FixtureCase {
            relative_path: "markdown/nested_runtime.md",
            expect_highlight: true,
            expected_fragments: &[
                "Nested Runtime",
                "draft",
                "emphasis",
                "strong",
                "strike",
                "Preview",
                "render",
                "\"enabled\"",
                "inline html block",
            ],
        },
        FixtureCase {
            relative_path: "markdown/userscript_fence.md",
            expect_highlight: true,
            expected_fragments: &[
                "Userscript Fence",
                "==UserScript==",
                "grant",
                "GM_addStyle",
                "document-end",
            ],
        },
        FixtureCase {
            relative_path: "markdown/go_fence.md",
            expect_highlight: true,
            expected_fragments: &["Go Fence", "package", "Renderer", "Render", "NewRenderer"],
        },
        FixtureCase {
            relative_path: "markdown/graphql_fence.md",
            expect_highlight: true,
            expected_fragments: &["GraphQL Fence", "query", "ThemeBySlug", "$", "fragment"],
        },
        FixtureCase {
            relative_path: "markdown/sql_dialects.md",
            expect_highlight: true,
            expected_fragments: &["SQL Dialects", "RETURNING", "AUTO_INCREMENT", "WITHOUT"],
        },
        FixtureCase {
            relative_path: "just/Justfile",
            expect_highlight: true,
            expected_fragments: &["build", "cargo", "--target", "target"],
        },
        FixtureCase {
            relative_path: "just/injections.just",
            expect_highlight: true,
            expected_fragments: &["import", "json", "print", "source", ".env"],
        },
        FixtureCase {
            relative_path: "just/nested_recipes.just",
            expect_highlight: true,
            expected_fragments: &["python-demo", "Preview", "render", "source", "HOME"],
        },
        FixtureCase {
            relative_path: "just/heredoc_recipes.just",
            expect_highlight: true,
            expected_fragments: &["SELECT", "enabled", "ORDER"],
        },
        FixtureCase {
            relative_path: "sql/rich.sql",
            expect_highlight: true,
            expected_fragments: &[
                "WITH",
                "recent_themes",
                "preview_count",
                "$kat$dracula$kat$",
                "featured",
            ],
        },
        FixtureCase {
            relative_path: "sql/schema.postgres.sql",
            expect_highlight: true,
            expected_fragments: &["jsonb", "CONFLICT", "RETURNING", "$tag$dracula$tag$"],
        },
        FixtureCase {
            relative_path: "sql/schema.mysql.sql",
            expect_highlight: true,
            expected_fragments: &["AUTO_INCREMENT", "ENGINE", "REPLACE", "UNSIGNED"],
        },
        FixtureCase {
            relative_path: "sql/schema.sqlite.sql",
            expect_highlight: true,
            expected_fragments: &["AUTOINCREMENT", "STRICT", "WITHOUT", "REPLACE"],
        },
        FixtureCase {
            relative_path: "graphql/schema.graphql",
            expect_highlight: true,
            expected_fragments: &["schema", "Query", "Theme", "fragment", "ThemeBySlug", "$"],
        },
        FixtureCase {
            relative_path: "plain/notes.txt",
            expect_highlight: false,
            expected_fragments: &["plain text"],
        },
    ];

    #[test]
    fn fixture_suite_matches_expected_rendering_behavior() {
        let theme = Theme::for_mode(ColorMode::TrueColor);

        for case in FIXTURE_CASES {
            let path = fixture_path(case.relative_path);
            let source = read_file(&path);
            let rendered = render_with_theme(Some(path.as_path()), &source, &theme)
                .unwrap_or_else(|error| panic!("failed to render {}: {error}", path.display()));

            if case.expect_highlight {
                assert!(
                    rendered.contains("\x1b["),
                    "expected ANSI highlighting for {}",
                    path.display()
                );
            } else {
                assert_eq!(
                    rendered,
                    source,
                    "plain fixture should render unchanged: {}",
                    path.display()
                );
            }

            for fragment in case.expected_fragments {
                assert!(
                    rendered.contains(fragment),
                    "missing fragment {fragment:?} for {}",
                    path.display()
                );
            }
        }
    }

    #[test]
    fn showcase_suite_renders_without_errors() {
        let theme = Theme::for_mode(ColorMode::TrueColor);

        for path in collect_files("testdata/showcase") {
            let source = read_file(&path);
            let rendered =
                render_with_theme(Some(path.as_path()), &source, &theme).unwrap_or_else(|error| {
                    panic!(
                        "failed to render showcase fixture {}: {error}",
                        path.display()
                    )
                });

            if is_supported_highlight_path(&path) {
                assert!(
                    rendered.contains("\x1b["),
                    "expected ANSI highlighting for supported showcase {}",
                    path.display()
                );
            } else {
                assert!(
                    !rendered.is_empty(),
                    "unsupported showcase should still render output: {}",
                    path.display()
                );
            }
        }
    }

    #[test]
    fn just_injections_highlight_nested_languages() {
        let theme = Theme::for_mode(ColorMode::TrueColor);
        let path = fixture_path("just/injections.just");
        let source = read_file(&path);
        let rendered = render_with_theme(Some(path.as_path()), &source, &theme)
            .unwrap_or_else(|error| panic!("failed to render {}: {error}", path.display()));

        assert!(
            rendered.contains("\x1b[38;2;255;121;198mimport"),
            "expected injected Python keyword styling for import"
        );
        assert!(
            rendered.contains("\x1b[38;2;255;184;108mTrue"),
            "expected injected Python boolean styling for True"
        );
        assert!(
            rendered.contains("\x1b[38;2;139;233;253msource"),
            "expected injected Bash builtin styling for source"
        );
        assert!(
            rendered.contains("\x1b[3m\x1b[38;2;189;147;249mconsole"),
            "expected bun shebang to reuse injected JavaScript builtin styling"
        );
    }

    #[test]
    fn just_shell_aliases_reuse_supported_runtimes() {
        let theme = Theme::for_mode(ColorMode::TrueColor);
        let path = fixture_path("just/runtime_matrix.just");
        let source = read_file(&path);
        let rendered = render_with_theme(Some(path.as_path()), &source, &theme)
            .unwrap_or_else(|error| panic!("failed to render {}: {error}", path.display()));

        let zsh_source_count = rendered.matches("\x1b[38;2;139;233;253msource").count();
        assert!(
            zsh_source_count >= 2,
            "expected global zsh shell setting to route both recipe bodies and external commands into zsh runtime"
        );
        assert!(
            rendered.matches("\x1b[38;2;139;233;253mprintf").count() >= 2,
            "expected zsh global shell setting to keep zsh builtin styling across recipe bodies and external commands"
        );
        assert!(
            rendered.contains("\x1b[38;2;139;233;253msource"),
            "expected fish shebang recipe to route recipe body into fish runtime"
        );
    }

    #[test]
    fn fish_and_zsh_detect_by_extension_filename_and_shebang() {
        assert!(matches!(
            detect_language(Some(Path::new(".envrc")), "export THEME=Dracula\n"),
            Some(SupportedLanguage::Bash)
        ));
        assert!(matches!(
            detect_language(Some(Path::new(".bash_aliases")), "alias k='kat'\n"),
            Some(SupportedLanguage::Bash)
        ));
        assert!(matches!(
            detect_language(Some(Path::new("config.fish")), "set theme Dracula\n"),
            Some(SupportedLanguage::Fish)
        ));
        assert!(matches!(
            detect_language(Some(Path::new(".zshrc")), "emulate -L zsh\n"),
            Some(SupportedLanguage::Zsh)
        ));
        assert!(matches!(
            detect_language(Some(Path::new(".zsh_aliases")), "setopt EXTENDED_GLOB\n"),
            Some(SupportedLanguage::Zsh)
        ));
        assert!(matches!(
            detect_language(
                Some(Path::new("dracula.zsh-theme")),
                "autoload -Uz colors\n"
            ),
            Some(SupportedLanguage::Zsh)
        ));
        assert!(matches!(
            detect_language(None, "#!/usr/bin/env fish\nset theme Dracula\n"),
            Some(SupportedLanguage::Fish)
        ));
        assert!(matches!(
            detect_language(None, "#!/usr/bin/env zsh\nemulate -L zsh\n"),
            Some(SupportedLanguage::Zsh)
        ));
        assert!(matches!(
            detect_language(Some(Path::new(".gitignore")), "target/\n"),
            Some(SupportedLanguage::Ignore)
        ));
        assert!(matches!(
            detect_language(Some(Path::new(".dockerignore")), "node_modules/\n"),
            Some(SupportedLanguage::Ignore)
        ));
        assert!(matches!(
            detect_language(Some(Path::new("Dockerfile.dockerignore")), ".git\n"),
            Some(SupportedLanguage::Ignore)
        ));
        assert!(matches!(
            detect_language(Some(Path::new(".npmignore")), "dist/\n"),
            Some(SupportedLanguage::Ignore)
        ));
        assert!(matches!(
            detect_language(Some(Path::new("Dockerfile")), "FROM alpine:3.21\n"),
            Some(SupportedLanguage::Dockerfile)
        ));
        assert!(matches!(
            detect_language(
                Some(Path::new("Dockerfile.dev")),
                "RUN printf '%s\\n' \"$HOME\"\n"
            ),
            Some(SupportedLanguage::Dockerfile)
        ));
        assert!(matches!(
            detect_language(
                Some(Path::new("example.hcl")),
                "job \"api\" { datacenters = [\"dc1\"] }\n"
            ),
            Some(SupportedLanguage::Hcl)
        ));
        assert!(matches!(
            detect_language(
                Some(Path::new("example.nomad")),
                "job \"api\" { datacenters = [\"dc1\"] }\n"
            ),
            Some(SupportedLanguage::Hcl)
        ));
        assert!(matches!(
            detect_language(
                Some(Path::new("scripts/profile.ps1")),
                "Write-Host \"kat\"\n"
            ),
            Some(SupportedLanguage::Powershell)
        ));
        assert!(matches!(
            detect_language(None, "#!/usr/bin/env pwsh\nWrite-Host \"kat\"\n"),
            Some(SupportedLanguage::Powershell)
        ));
        assert!(matches!(
            detect_language(Some(Path::new("scripts/build.cmd")), "@echo off\r\n"),
            Some(SupportedLanguage::Batch)
        ));

        let workflow_kind = detect_document_kind(
            Some(Path::new(".github/workflows/build.yml")),
            "name: Build\njobs: {}\n",
        )
        .expect("expected GitHub workflow path to detect as YAML");
        assert_eq!(workflow_kind.runtime_name(), "yaml");
        assert_eq!(
            workflow_kind.profile(),
            DocumentProfile::GitHubActionsWorkflow
        );

        let action_kind = detect_document_kind(
            Some(Path::new("action.yml")),
            "name: Example\nruns:\n  using: composite\n",
        )
        .expect("expected action metadata path to detect as YAML");
        assert_eq!(action_kind.runtime_name(), "yaml");
        assert_eq!(action_kind.profile(), DocumentProfile::GitHubActionMetadata);

        assert!(matches!(
            detect_language(
                Some(Path::new("proto/theme.proto")),
                "syntax = \"proto3\";\nmessage Theme {}\n",
            ),
            Some(SupportedLanguage::Proto)
        ));
        assert!(matches!(
            detect_language(
                Some(Path::new("proto/theme.textproto")),
                "name: \"Dracula\"\nenabled: true\n",
            ),
            Some(SupportedLanguage::Textproto)
        ));
    }

    #[test]
    fn fish_and_zsh_highlight_shell_specific_constructs() {
        let theme = Theme::for_mode(ColorMode::TrueColor);

        let fish_path = fixture_path("fish/rich.fish");
        let fish_source = read_file(&fish_path);
        let fish_rendered = render_with_theme(Some(fish_path.as_path()), &fish_source, &theme)
            .unwrap_or_else(|error| panic!("failed to render {}: {error}", fish_path.display()));
        assert!(
            fish_rendered.contains("\x1b[3m\x1b[38;2;80;250;123m#!/usr/bin/env fish"),
            "expected fish shebang directive styling"
        );
        assert!(
            fish_rendered.contains("\x1b[38;2;255;121;198mfunction"),
            "expected fish function keyword styling"
        );
        assert!(
            fish_rendered.contains("\x1b[38;2;139;233;253mset"),
            "expected fish builtin command styling"
        );
        assert!(
            fish_rendered.contains("\x1b[38;2;139;233;253mcontains"),
            "expected fish contains builtin styling"
        );
        assert!(
            fish_rendered.contains("\x1b[38;2;189;147;249m--argument-names"),
            "expected fish option styling"
        );
        assert!(
            fish_rendered.contains("\x1b[38;2;139;233;253memit"),
            "expected fish emit builtin styling"
        );
        assert!(
            fish_rendered.contains("\x1b[38;2;139;233;253mfunctions"),
            "expected fish functions builtin styling"
        );
        assert!(
            fish_rendered.contains("\x1b[38;2;139;233;253mtype"),
            "expected fish type builtin styling"
        );
        assert!(
            fish_rendered.contains("\x1b[3m\x1b[38;2;80;250;123mcurrent-filename"),
            "expected fish status subcommand directive styling"
        );
        assert!(
            fish_rendered.contains("\x1b[38;2;255;85;85mDra*"),
            "expected fish case glob pattern styling"
        );
        assert!(
            fish_rendered.contains("\x1b[3m\x1b[38;2;255;184;108mnew_theme"),
            "expected fish function argument names styling"
        );
        assert!(
            fish_rendered.contains("\x1b[3m\x1b[38;2;255;184;108mTHEME_NAME"),
            "expected fish function handler target styling"
        );
        assert!(
            fish_rendered.contains("\x1b[3m\x1b[38;2;80;250;123mreplace"),
            "expected fish string subcommand directive styling"
        );
        assert!(
            fish_rendered.contains("\x1b[3m\x1b[38;2;189;147;249m$argv"),
            "expected fish argv special variable styling"
        );
        assert!(
            fish_rendered.contains("\x1b[3m\x1b[38;2;189;147;249m$status"),
            "expected fish status special variable styling"
        );
        assert!(
            fish_rendered.contains("\x1b[3m\x1b[38;2;189;147;249m$fish_pid"),
            "expected fish fish_pid special variable styling"
        );
        assert!(
            fish_rendered.contains("\x1b[3m\x1b[38;2;189;147;249m$last_pid"),
            "expected fish last_pid special variable styling"
        );
        assert!(
            fish_rendered.contains("\x1b[38;2;255;121;198m.."),
            "expected fish list range operator styling"
        );

        let zsh_path = fixture_path("zsh/rich.zsh");
        let zsh_source = read_file(&zsh_path);
        let zsh_rendered = render_with_theme(Some(zsh_path.as_path()), &zsh_source, &theme)
            .unwrap_or_else(|error| panic!("failed to render {}: {error}", zsh_path.display()));
        assert!(
            zsh_rendered.contains("\x1b[3m\x1b[38;2;80;250;123m#!/usr/bin/env zsh"),
            "expected zsh shebang directive styling"
        );
        assert!(
            zsh_rendered.contains("\x1b[38;2;139;233;253memulate"),
            "expected zsh builtin command styling"
        );
        assert!(
            zsh_rendered.contains("\x1b[38;2;248;248;242m[["),
            "expected zsh test command bracket styling"
        );
        assert!(
            zsh_rendered.contains("\x1b[3m\x1b[38;2;80;250;123mEXTENDED_GLOB"),
            "expected zsh setopt option-name styling"
        );
        assert!(
            zsh_rendered.contains("\x1b[38;2;139;233;253msource"),
            "expected zsh source builtin styling"
        );
        assert!(
            zsh_rendered.contains("\x1b[38;2;139;233;253mread"),
            "expected zsh read builtin styling"
        );
        assert!(
            zsh_rendered.contains("\x1b[3m\x1b[38;2;255;184;108mtheme_line"),
            "expected zsh read target variable styling"
        );
        assert!(
            zsh_rendered.contains("\x1b[38;2;248;248;242m["),
            "expected zsh subscript bracket styling"
        );
        assert!(
            zsh_rendered.contains("\x1b[3m\x1b[38;2;80;250;123m:l"),
            "expected zsh expansion modifier styling"
        );
        assert!(
            zsh_rendered.contains("\x1b[3m\x1b[38;2;80;250;123m(I)"),
            "expected zsh subscript flag styling"
        );
    }

    #[test]
    fn powershell_and_batch_highlight_shell_specific_constructs() {
        let theme = Theme::for_mode(ColorMode::TrueColor);

        let powershell_path = fixture_path("powershell/rich.ps1");
        let powershell_source = read_file(&powershell_path);
        let powershell_rendered =
            render_with_theme(Some(powershell_path.as_path()), &powershell_source, &theme)
                .unwrap_or_else(|error| {
                    panic!("failed to render {}: {error}", powershell_path.display())
                });
        assert!(
            powershell_rendered.contains("\x1b[38;2;255;121;198mparam"),
            "expected PowerShell param keyword styling"
        );
        assert!(
            powershell_rendered.contains("\x1b[38;2;139;233;253mstring"),
            "expected PowerShell type styling"
        );
        assert!(
            powershell_rendered.contains("\x1b[38;2;139;233;253mWrite-Host"),
            "expected PowerShell builtin command styling"
        );
        assert!(
            powershell_rendered.contains("\x1b[3m\x1b[38;2;189;147;249m$env:GITHUB_REF"),
            "expected PowerShell environment variables to receive special-variable styling"
        );

        let batch_path = fixture_path("batch/rich.cmd");
        let batch_source = read_file(&batch_path);
        let batch_rendered = render_with_theme(Some(batch_path.as_path()), &batch_source, &theme)
            .unwrap_or_else(|error| panic!("failed to render {}: {error}", batch_path.display()));
        assert!(
            batch_rendered.contains("\x1b[38;2;255;121;198m@echo off"),
            "expected batch echo-off keyword styling"
        );
        assert!(
            batch_rendered.contains("\x1b[38;2;255;121;198mset"),
            "expected batch set keyword styling"
        );
        assert!(
            batch_rendered.contains("\x1b[38;2;139;233;253mecho"),
            "expected batch builtin command styling"
        );
        assert!(
            batch_rendered.contains("\x1b[38;2;80;250;123m:build"),
            "expected batch labels to receive dedicated label styling"
        );
        assert!(
            batch_rendered.contains("\x1b[3m\x1b[38;2;80;250;123m:eof"),
            "expected batch :eof targets to receive directive styling"
        );
    }

    #[test]
    fn justfile_windows_shells_reuse_batch_and_powershell_runtimes() {
        let theme = Theme::for_mode(ColorMode::TrueColor);
        let path = fixture_path("just/windows_shells.just");
        let source = read_file(&path);
        let rendered = render_with_theme(Some(path.as_path()), &source, &theme)
            .unwrap_or_else(|error| panic!("failed to render {}: {error}", path.display()));

        assert!(
            rendered.contains("\x1b[38;2;255;121;198m@echo off"),
            "expected cmd-backed Justfile recipes to reuse batch highlighting"
        );
        assert!(
            rendered.contains("\x1b[38;2;139;233;253mWrite-Host"),
            "expected pwsh shebang recipes to reuse PowerShell builtin highlighting"
        );
    }

    #[test]
    fn html_injections_highlight_script_and_style_content() {
        let theme = Theme::for_mode(ColorMode::TrueColor);
        let path = fixture_path("html/index.html");
        let source = read_file(&path);
        let rendered = render_with_theme(Some(path.as_path()), &source, &theme)
            .unwrap_or_else(|error| panic!("failed to render {}: {error}", path.display()));

        assert!(
            rendered.contains("\x1b[38;2;255;121;198mstyle"),
            "expected html tag-name styling for style tag"
        );
        assert!(
            rendered.contains("\x1b[38;2;241;250;140m#8be9fd"),
            "expected injected css color styling for custom property value"
        );
        assert!(
            rendered.contains("\x1b[3m\x1b[38;2;189;147;249mconsole"),
            "expected injected javascript builtin styling for console"
        );
        assert!(
            rendered.contains("\x1b[38;2;255;121;198m@param"),
            "expected jsdoc tag styling for @param"
        );
    }

    #[test]
    fn bash_heredoc_interpreter_blocks_highlight_nested_languages() {
        let theme = Theme::for_mode(ColorMode::TrueColor);
        let path = fixture_path("bash/heredoc_injections.sh");
        let source = read_file(&path);
        let rendered = render_with_theme(Some(path.as_path()), &source, &theme)
            .unwrap_or_else(|error| panic!("failed to render {}: {error}", path.display()));

        assert!(
            rendered.contains("\x1b[38;2;139;233;253mPreview"),
            "expected python heredoc to reuse python class styling"
        );
        assert!(
            rendered.contains("\x1b[38;2;80;250;123mrender"),
            "expected python heredoc to reuse python method styling"
        );
        assert!(
            rendered.contains("\x1b[3m\x1b[38;2;189;147;249mconsole"),
            "expected node heredoc to reuse javascript builtin styling"
        );
        assert!(
            rendered.contains("\x1b[38;2;139;233;253msource"),
            "expected bash heredoc to reuse bash builtin styling"
        );
    }

    #[test]
    fn html_attribute_injections_highlight_css_and_javascript() {
        let theme = Theme::for_mode(ColorMode::TrueColor);
        let path = fixture_path("html/attribute_injections.html");
        let source = read_file(&path);
        let rendered = render_with_theme(Some(path.as_path()), &source, &theme)
            .unwrap_or_else(|error| panic!("failed to render {}: {error}", path.display()));

        assert!(
            rendered.contains("\x1b[38;2;80;250;123mvar"),
            "expected injected CSS function styling inside style attribute"
        );
        assert!(
            rendered.contains("\x1b[3m\x1b[38;2;189;147;249mconsole"),
            "expected injected JavaScript builtin styling inside onclick attribute"
        );
        assert!(
            rendered.contains("\x1b[38;2;255;121;198mreturn"),
            "expected injected JavaScript keyword styling inside onclick attribute"
        );
    }

    #[test]
    fn go_highlights_generics_methods_builtins_and_directives() {
        let theme = Theme::for_mode(ColorMode::TrueColor);
        let path = fixture_path("go/rich.go");
        let source = read_file(&path);
        let rendered = render_with_theme(Some(path.as_path()), &source, &theme)
            .unwrap_or_else(|error| panic!("failed to render {}: {error}", path.display()));

        assert!(
            rendered.contains("\x1b[3m\x1b[38;2;80;250;123m//go:build linux"),
            "expected go build directive styling"
        );
        assert!(
            rendered.contains("\x1b[3m\x1b[38;2;139;233;253mshowcase"),
            "expected package namespace styling"
        );
        assert!(
            rendered.contains("\x1b[38;2;139;233;253mRenderer"),
            "expected go type styling for Renderer"
        );
        assert!(
            rendered.contains("\x1b[38;2;80;250;123mNewRenderer"),
            "expected go function definition styling"
        );
        assert!(
            rendered.contains("\x1b[38;2;80;250;123mRender"),
            "expected go method definition styling"
        );
        assert!(
            rendered.contains("\x1b[38;2;139;233;253mmake"),
            "expected go builtin function styling for make"
        );
        assert!(
            rendered.contains("\x1b[3m\x1b[38;2;139;233;253mstring"),
            "expected go builtin type styling for string"
        );
    }

    #[test]
    fn go_injections_and_markdown_fences_reuse_same_runtime() {
        let theme = Theme::for_mode(ColorMode::TrueColor);

        let go_path = fixture_path("go/rich.go");
        let go_source = read_file(&go_path);
        let go_rendered = render_with_theme(Some(go_path.as_path()), &go_source, &theme)
            .unwrap_or_else(|error| panic!("failed to render {}: {error}", go_path.display()));

        assert!(
            go_rendered.contains("\x1b[38;2;255;184;108mtrue"),
            "expected injected JSON boolean styling in Go"
        );
        assert!(
            go_rendered.contains("\x1b[38;2;255;121;198mSELECT"),
            "expected injected SQL keyword styling in Go"
        );
        assert!(
            go_rendered.contains("\x1b[38;2;255;121;198mRETURNING"),
            "expected Go sql:postgres hint to route through postgres SQL runtime"
        );
        assert!(
            go_rendered.contains("\x1b[38;2;255;121;198mAUTO_INCREMENT"),
            "expected Go sql:mysql hint to route through mysql SQL runtime"
        );
        assert!(
            go_rendered.contains("\x1b[38;2;255;121;198mWITHOUT"),
            "expected Go sql:sqlite hint to route through sqlite SQL runtime"
        );
        assert!(
            go_rendered.contains("\x1b[3m\x1b[38;2;189;147;249mconsole"),
            "expected injected JavaScript builtin styling in Go"
        );
        assert!(
            go_rendered.contains("\x1b[38;2;255;121;198msection"),
            "expected injected HTML tag styling in Go"
        );
        assert!(
            go_rendered.contains("--accent"),
            "expected injected CSS custom property content in Go"
        );
        assert!(
            go_rendered.contains("\x1b[38;2;139;233;253mprintf"),
            "expected injected Bash builtin styling in Go"
        );
        assert!(
            go_rendered.contains("\x1b[38;2;139;233;253msection"),
            "expected injected regex named group styling in Go"
        );
        assert!(
            go_rendered.contains("\x1b[3m\x1b[38;2;139;233;253malpha"),
            "expected Go regexp.MustCompilePOSIX case to reuse posix regex styling"
        );
        assert!(
            go_rendered.contains("\x1b[38;2;139;233;253mescapedSection"),
            "expected Go escaped regex string case to reuse regex named group styling"
        );

        let markdown_path = fixture_path("markdown/go_fence.md");
        let markdown_source = read_file(&markdown_path);
        let markdown_rendered =
            render_with_theme(Some(markdown_path.as_path()), &markdown_source, &theme)
                .unwrap_or_else(|error| {
                    panic!("failed to render {}: {error}", markdown_path.display())
                });

        assert!(
            markdown_rendered.contains("\x1b[38;2;139;233;253mRenderer"),
            "expected fenced go block to reuse go type styling"
        );
        assert!(
            markdown_rendered.contains("\x1b[38;2;80;250;123mRender"),
            "expected fenced go block to reuse go method styling"
        );
        assert!(
            markdown_rendered.contains("\x1b[38;2;80;250;123mNewRenderer"),
            "expected fenced golang block alias to reuse go function styling"
        );

        let sql_markdown_path = fixture_path("markdown/sql_dialects.md");
        let sql_markdown_source = read_file(&sql_markdown_path);
        let sql_markdown_rendered = render_with_theme(
            Some(sql_markdown_path.as_path()),
            &sql_markdown_source,
            &theme,
        )
        .unwrap_or_else(|error| {
            panic!("failed to render {}: {error}", sql_markdown_path.display())
        });
        assert!(
            sql_markdown_rendered.contains("\x1b[38;2;255;121;198mRETURNING"),
            "expected fenced postgres block alias to route through postgres SQL runtime"
        );
        assert!(
            sql_markdown_rendered.contains("\x1b[38;2;255;121;198mAUTO_INCREMENT"),
            "expected fenced mysql block alias to route through mysql SQL runtime"
        );
        assert!(
            sql_markdown_rendered.contains("\x1b[38;2;255;121;198mWITHOUT"),
            "expected fenced sqlite block alias to route through sqlite SQL runtime"
        );
    }

    #[test]
    fn go_module_workspace_and_sum_files_use_dedicated_runtimes() {
        let theme = Theme::for_mode(ColorMode::TrueColor);

        let gomod_path = fixture_path("go/go.mod");
        let gomod_source = read_file(&gomod_path);
        let gomod_rendered = render_with_theme(Some(gomod_path.as_path()), &gomod_source, &theme)
            .unwrap_or_else(|error| panic!("failed to render {}: {error}", gomod_path.display()));
        assert!(
            gomod_rendered.contains("\x1b[38;2;255;121;198mmodule"),
            "expected go.mod directive keyword styling"
        );
        assert!(
            gomod_rendered.contains("\x1b[38;2;139;233;253mgithub.com/dcjanus/kat"),
            "expected go.mod module path styling"
        );
        assert!(
            gomod_rendered.contains("\x1b[38;2;241;250;140mgo1.24.1"),
            "expected go.mod toolchain value styling"
        );
        assert!(
            gomod_rendered.contains("\x1b[38;2;139;233;253m../forks/lipgloss"),
            "expected go.mod replacement file path styling"
        );

        let gowork_path = fixture_path("go/go.work");
        let gowork_source = read_file(&gowork_path);
        let gowork_rendered =
            render_with_theme(Some(gowork_path.as_path()), &gowork_source, &theme).unwrap_or_else(
                |error| panic!("failed to render {}: {error}", gowork_path.display()),
            );
        assert!(
            gowork_rendered.contains("\x1b[38;2;255;121;198muse"),
            "expected go.work use directive styling"
        );
        assert!(
            gowork_rendered.contains("\x1b[38;2;139;233;253m../shared/theme-kit"),
            "expected go.work workspace path styling"
        );

        let gosum_path = fixture_path("go/go.sum");
        let gosum_source = read_file(&gosum_path);
        let gosum_rendered = render_with_theme(Some(gosum_path.as_path()), &gosum_source, &theme)
            .unwrap_or_else(|error| panic!("failed to render {}: {error}", gosum_path.display()));
        assert!(
            gosum_rendered.contains("\x1b[38;2;139;233;253mgithub.com/charmbracelet/lipgloss"),
            "expected go.sum module path styling"
        );
        assert!(
            gosum_rendered.contains("\x1b[3m\x1b[38;2;80;250;123mh1"),
            "expected go.sum hash version styling"
        );
        assert!(
            gosum_rendered.contains("\x1b[38;2;255;121;198m+incompatible"),
            "expected go.sum compatibility suffix styling"
        );
    }

    #[test]
    fn nested_fixtures_lock_supported_injection_scenarios() {
        let theme = Theme::for_mode(ColorMode::TrueColor);

        let markdown_path = fixture_path("markdown/nested_runtime.md");
        let markdown_source = read_file(&markdown_path);
        let markdown_rendered =
            render_with_theme(Some(markdown_path.as_path()), &markdown_source, &theme)
                .unwrap_or_else(|error| {
                    panic!("failed to render {}: {error}", markdown_path.display())
                });
        assert!(
            markdown_rendered.contains("title"),
            "expected TOML frontmatter property styling in nested markdown fixture"
        );
        assert!(
            markdown_rendered.contains("\x1b[38;2;139;233;253mPreview"),
            "expected injected Python class styling in nested markdown fixture"
        );
        assert!(
            markdown_rendered.contains("\x1b[38;2;255;184;108mtrue"),
            "expected injected JSON boolean styling in nested markdown fixture"
        );

        let html_path = fixture_path("html/nested_regions.html");
        let html_source = read_file(&html_path);
        let html_rendered = render_with_theme(Some(html_path.as_path()), &html_source, &theme)
            .unwrap_or_else(|error| panic!("failed to render {}: {error}", html_path.display()));
        assert!(
            html_rendered.contains("ff79c6"),
            "expected injected CSS styling in nested HTML fixture"
        );
        assert!(
            html_rendered.contains("\x1b[3m\x1b[38;2;189;147;249mconsole"),
            "expected injected JavaScript builtin styling in nested HTML fixture"
        );
        assert!(
            html_rendered.contains("\x1b[38;2;255;121;198m@param"),
            "expected injected JSDoc styling in nested HTML fixture"
        );

        let html_attr_path = fixture_path("html/attribute_injections.html");
        let html_attr_source = read_file(&html_attr_path);
        let html_attr_rendered =
            render_with_theme(Some(html_attr_path.as_path()), &html_attr_source, &theme)
                .unwrap_or_else(|error| {
                    panic!("failed to render {}: {error}", html_attr_path.display())
                });
        assert!(
            html_attr_rendered.contains("\x1b[38;2;80;250;123mvar"),
            "expected injected CSS styling in HTML attribute fixture"
        );
        assert!(
            html_attr_rendered.contains("\x1b[3m\x1b[38;2;189;147;249mconsole"),
            "expected injected JavaScript styling in HTML attribute fixture"
        );

        let just_path = fixture_path("just/nested_recipes.just");
        let just_source = read_file(&just_path);
        let just_rendered = render_with_theme(Some(just_path.as_path()), &just_source, &theme)
            .unwrap_or_else(|error| panic!("failed to render {}: {error}", just_path.display()));
        assert!(
            just_rendered.contains("\x1b[38;2;139;233;253mPreview"),
            "expected injected Python class styling in nested Just fixture"
        );
        assert!(
            just_rendered.contains("\x1b[38;2;80;250;123mrender"),
            "expected injected Python method styling in nested Just fixture"
        );
        assert!(
            just_rendered.contains("\x1b[38;2;139;233;253msource"),
            "expected injected Bash builtin styling in nested Just fixture"
        );

        let just_heredoc_path = fixture_path("just/heredoc_recipes.just");
        let just_heredoc_source = read_file(&just_heredoc_path);
        let just_heredoc_rendered = render_with_theme(
            Some(just_heredoc_path.as_path()),
            &just_heredoc_source,
            &theme,
        )
        .unwrap_or_else(|error| {
            panic!("failed to render {}: {error}", just_heredoc_path.display())
        });
        assert!(
            just_heredoc_rendered.contains("\x1b[38;2;139;233;253mPreview"),
            "expected bash heredoc injections inside Justfile to reuse python class styling"
        );
        assert!(
            just_heredoc_rendered.contains("\x1b[3m\x1b[38;2;189;147;249mconsole"),
            "expected bash heredoc injections inside Justfile to reuse javascript builtin styling"
        );
        assert!(
            just_heredoc_rendered.contains("\x1b[38;2;139;233;253msource"),
            "expected bash heredoc injections inside Justfile to reuse bash builtin styling"
        );

        let js_path = fixture_path("javascript/tagged_templates.js");
        let js_source = read_file(&js_path);
        let js_rendered = render_with_theme(Some(js_path.as_path()), &js_source, &theme)
            .unwrap_or_else(|error| panic!("failed to render {}: {error}", js_path.display()));
        assert!(
            js_rendered.contains("--accent"),
            "expected injected CSS styling in JavaScript tagged template fixture"
        );
        assert!(
            js_rendered.contains("\x1b[38;2;255;184;108mtrue"),
            "expected injected JSON boolean styling in JavaScript tagged template fixture"
        );
        assert!(
            js_rendered.contains("\x1b[38;2;255;121;198msection"),
            "expected injected HTML tag styling in JavaScript tagged template fixture"
        );
    }

    #[test]
    fn markdown_injections_highlight_inline_and_fenced_content() {
        let theme = Theme::for_mode(ColorMode::TrueColor);
        let path = fixture_path("markdown/sample.md");
        let source = read_file(&path);
        let rendered = render_with_theme(Some(path.as_path()), &source, &theme)
            .unwrap_or_else(|error| panic!("failed to render {}: {error}", path.display()));

        assert!(
            rendered.contains("layout"),
            "expected markdown YAML frontmatter property styling"
        );
        assert!(
            rendered.contains("\x1b[38;2;241;250;140m\"solarized\""),
            "expected markdown YAML frontmatter string styling"
        );
        assert!(
            rendered.contains("\x1b[1m\x1b[38;2;189;147;249mPreview"),
            "expected markdown heading styling"
        );
        assert!(
            rendered.contains("\x1b[1m\x1b[38;2;189;147;249mSecondary Heading"),
            "expected markdown setext heading styling"
        );
        assert!(
            rendered.contains("\x1b[3m\x1b[38;2;241;250;140memphasis"),
            "expected markdown inline emphasis styling"
        );
        assert!(
            rendered.contains("\x1b[1m\x1b[38;2;255;184;108mstrong"),
            "expected markdown strong-emphasis styling"
        );
        assert!(
            rendered.contains("\x1b[9m\x1b[38;2;98;114;164mstrikethrough"),
            "expected markdown strikethrough styling"
        );
        assert!(
            rendered.contains("\x1b[38;2;139;233;253m- [x]"),
            "expected markdown task-list marker styling"
        );
        assert!(
            rendered.contains("\x1b[38;2;139;233;253m1."),
            "expected markdown ordered-list marker styling"
        );
        assert!(
            rendered.contains("\x1b[38;2;139;233;253mhttps://example.com"),
            "expected markdown link uri styling"
        );
        assert!(
            rendered.contains("\x1b[38;2;139;233;253m<https://example.com/autolink>"),
            "expected markdown autolink styling"
        );
        assert!(
            rendered.contains("\x1b[38;2;139;233;253m<hello@example.com>"),
            "expected markdown email autolink styling"
        );
        assert!(
            rendered.contains("\x1b[38;2;255;121;198m[docs-ref]"),
            "expected markdown reference link label styling"
        );
        assert!(
            rendered.contains("\x1b[38;2;241;250;140m\"Docs Title\""),
            "expected markdown link title styling"
        );
        assert!(
            rendered.contains("\x1b[38;2;248;248;242m|"),
            "expected markdown pipe table delimiter styling"
        );
        assert!(
            rendered.contains("\x1b[38;2;248;248;242m```rust"),
            "expected markdown fenced code delimiter line styling"
        );
        assert!(
            rendered.contains("\x1b[38;2;80;250;123mlet value = 42"),
            "expected markdown inline code-span styling"
        );
        assert!(
            rendered.contains("\x1b[38;2;255;184;108mplain fence without syntax"),
            "expected markdown plain fenced code block styling"
        );
        assert!(
            rendered.contains("\x1b[3m\x1b[38;2;241;250;140m>"),
            "expected markdown blockquote marker styling"
        );
        assert!(
            rendered.contains("\x1b[38;2;255;121;198mkbd"),
            "expected markdown inline HTML to route through HTML highlighting"
        );
        assert!(
            rendered.contains("\x1b[38;2;255;121;198mreturn"),
            "expected fenced rust block injection styling"
        );
    }

    #[test]
    fn markdown_frontmatter_supports_yaml_and_toml_metadata() {
        let theme = Theme::for_mode(ColorMode::TrueColor);

        let yaml_path = fixture_path("markdown/sample.md");
        let yaml_source = read_file(&yaml_path);
        let yaml_rendered = render_with_theme(Some(yaml_path.as_path()), &yaml_source, &theme)
            .unwrap_or_else(|error| panic!("failed to render {}: {error}", yaml_path.display()));

        assert!(
            yaml_rendered.contains("layout"),
            "expected YAML frontmatter property styling"
        );
        assert!(
            yaml_rendered.contains("\x1b[38;2;255;184;108mtrue"),
            "expected YAML frontmatter boolean styling"
        );

        let toml_path = Path::new("testdata/showcase/markdown/mixed-code-blocks.md").to_path_buf();
        let toml_source = read_file(&toml_path);
        let toml_rendered = render_with_theme(Some(toml_path.as_path()), &toml_source, &theme)
            .unwrap_or_else(|error| panic!("failed to render {}: {error}", toml_path.display()));

        assert!(
            toml_rendered.contains("title"),
            "expected TOML frontmatter property styling"
        );
        assert!(
            toml_rendered.contains("\x1b[38;2;255;184;108mtrue"),
            "expected TOML frontmatter boolean styling"
        );
    }

    #[test]
    fn cargo_lock_is_detected_as_toml() {
        let theme = Theme::for_mode(ColorMode::TrueColor);
        let path = fixture_path("toml/Cargo.lock");
        let source = read_file(&path);
        let rendered = render_with_theme(Some(path.as_path()), &source, &theme)
            .unwrap_or_else(|error| panic!("failed to render {}: {error}", path.display()));

        assert!(
            rendered.contains("\x1b["),
            "expected ANSI highlighting for {}",
            path.display()
        );
        assert!(
            rendered.contains("\"kat\""),
            "expected TOML string content to remain visible for {}",
            path.display()
        );
    }

    #[test]
    fn uv_lock_is_detected_as_toml() {
        let theme = Theme::for_mode(ColorMode::TrueColor);
        let path = fixture_path("toml/uv.lock");
        let source = read_file(&path);
        let rendered = render_with_theme(Some(path.as_path()), &source, &theme)
            .unwrap_or_else(|error| panic!("failed to render {}: {error}", path.display()));

        assert!(
            rendered.contains("\x1b["),
            "expected ANSI highlighting for {}",
            path.display()
        );
        assert!(
            rendered.contains("\"kat\""),
            "expected TOML string content to remain visible for {}",
            path.display()
        );
    }

    #[test]
    fn ignore_files_are_detected_and_highlight_ignore_syntax() {
        let theme = Theme::for_mode(ColorMode::TrueColor);
        let path = fixture_path("ignore/.gitignore");
        let source = read_file(&path);
        let rendered = render_with_theme(Some(path.as_path()), &source, &theme)
            .unwrap_or_else(|error| panic!("failed to render {}: {error}", path.display()));

        assert!(
            rendered.contains("\x1b[3m\x1b[38;2;98;114;164m# ignore build output"),
            "expected ignore-file comments to reuse comment styling"
        );
        assert!(
            rendered.contains("\x1b[38;2;255;121;198m!"),
            "expected ignore-file negation to use operator styling"
        );
    }

    #[test]
    fn dockerfile_highlights_directives_and_injects_shell_forms() {
        let theme = Theme::for_mode(ColorMode::TrueColor);
        let path = fixture_path("dockerfile/Dockerfile");
        let source = read_file(&path);
        let rendered = render_with_theme(Some(path.as_path()), &source, &theme)
            .unwrap_or_else(|error| panic!("failed to render {}: {error}", path.display()));

        assert!(
            rendered.contains("\x1b[38;2;255;121;198mFROM"),
            "expected Dockerfile instruction keyword styling"
        );
        assert!(
            rendered.contains("\x1b[38;2;139;233;253mprintf"),
            "expected shell-form RUN/CMD/ENTRYPOINT content to reuse bash builtin styling"
        );
        assert!(
            rendered.contains("\x1b[38;2;189;147;249m--version"),
            "expected shell-form command options to reuse bash parameter styling"
        );
    }

    #[test]
    fn dockerfile_healthcheck_and_heredoc_reuse_bash_runtime() {
        let theme = Theme::for_mode(ColorMode::TrueColor);
        let path = fixture_path("dockerfile/Dockerfile.advanced");
        let source = read_file(&path);
        let rendered = render_with_theme(Some(path.as_path()), &source, &theme)
            .unwrap_or_else(|error| panic!("failed to render {}: {error}", path.display()));

        assert!(
            rendered.contains("\x1b[38;2;255;121;198mHEALTHCHECK"),
            "expected HEALTHCHECK instruction keyword styling"
        );
        assert!(
            rendered.contains("\x1b[38;2;139;233;253mprintf"),
            "expected HEALTHCHECK CMD shell form to reuse bash builtin styling"
        );
        assert!(
            rendered.contains("\x1b[38;2;139;233;253msource"),
            "expected Dockerfile heredoc lines to reuse bash builtin styling"
        );
        assert!(
            rendered.contains("\x1b[38;2;255;121;198m$"),
            "expected Dockerfile heredoc bash expansions to be highlighted"
        );
    }

    #[test]
    fn dockerfile_shell_instruction_routes_shell_regions_to_runtime() {
        let theme = Theme::for_mode(ColorMode::TrueColor);
        let path = fixture_path("dockerfile/Dockerfile.shells");
        let source = read_file(&path);
        let rendered = render_with_theme(Some(path.as_path()), &source, &theme)
            .unwrap_or_else(|error| panic!("failed to render {}: {error}", path.display()));

        assert!(
            rendered.contains("\x1b[38;2;139;233;253msource"),
            "expected SHELL [\"zsh\", ...] to route shell-form RUN into zsh runtime"
        );
        assert!(
            rendered.contains("\x1b[38;2;139;233;253mset"),
            "expected SHELL [\"fish\", ...] to route shell-form RUN into fish runtime"
        );
    }

    #[test]
    fn dockerfile_highlights_keys_ports_and_paths_with_more_specific_semantics() {
        let theme = Theme::for_mode(ColorMode::TrueColor);
        let path = fixture_path("dockerfile/Dockerfile.semantics");
        let source = read_file(&path);
        let rendered = render_with_theme(Some(path.as_path()), &source, &theme)
            .unwrap_or_else(|error| panic!("failed to render {}: {error}", path.display()));

        assert!(
            rendered.contains("\x1b[38;2;139;233;253mAPP_HOME"),
            "expected ENV / ARG keys to use key-style highlighting"
        );
        assert!(
            rendered.contains("\x1b[38;2;139;233;253morg.opencontainers.image.source"),
            "expected LABEL keys to use key-style highlighting"
        );
        assert!(
            rendered.contains("\x1b[38;2;255;184;108m8080"),
            "expected EXPOSE ports to use number styling"
        );
        assert!(
            rendered.contains("\x1b[38;2;241;250;140m/app"),
            "expected Dockerfile paths to use string styling"
        );
    }

    #[test]
    fn dockerfile_highlights_param_names_separately_from_values() {
        let theme = Theme::for_mode(ColorMode::TrueColor);
        let path = fixture_path("dockerfile/Dockerfile.params");
        let source = read_file(&path);
        let rendered = render_with_theme(Some(path.as_path()), &source, &theme)
            .unwrap_or_else(|error| panic!("failed to render {}: {error}", path.display()));

        assert!(
            rendered.contains("\x1b[3m\x1b[38;2;80;250;123m--from"),
            "expected Dockerfile param names to keep attribute styling"
        );
        assert!(
            rendered.contains("\x1b[38;2;241;250;140mbuilder"),
            "expected Dockerfile param values to use string-style highlighting"
        );
        assert!(
            rendered.contains("\x1b[38;2;189;147;249mBUILDPLATFORM"),
            "expected generic param expansions to split out the variable name semantics"
        );
        assert!(
            rendered.contains("\x1b[38;2;241;250;140m5s"),
            "expected HEALTHCHECK param values to stop inheriting attribute styling"
        );
        assert!(
            (rendered.contains("\x1b[38;2;189;147;249mCACHE_DIR")
                || rendered.contains("\x1b[38;2;189;147;249mCACHE_STAGE"))
                && rendered.contains("\x1b[38;2;255;184;108m1000"),
            "expected mount param payload to keep expansion and numeric semantics"
        );
    }

    #[test]
    fn dockerfile_highlights_mount_param_pairs_with_key_value_semantics() {
        let theme = Theme::for_mode(ColorMode::TrueColor);
        let path = fixture_path("dockerfile/Dockerfile.params");
        let source = read_file(&path);
        let rendered = render_with_theme(Some(path.as_path()), &source, &theme)
            .unwrap_or_else(|error| panic!("failed to render {}: {error}", path.display()));

        assert!(
            rendered.contains("\x1b[38;2;139;233;253mtype"),
            "expected mount param keys to use key-style highlighting"
        );
        assert!(
            rendered.contains("\x1b[38;2;139;233;253mtarget"),
            "expected mount param keys to use key-style highlighting"
        );
        assert!(
            rendered.contains("\x1b[38;2;139;233;253mcache"),
            "expected mount param enum-like values to use type styling"
        );
        assert!(
            rendered.contains("CACHE_DIR") && rendered.contains("CACHE_STAGE"),
            "expected mount param expansion-based values to render"
        );
        assert!(
            rendered.contains("\x1b[38;2;189;147;249mreadonly"),
            "expected bare mount flags to use constant styling"
        );
        assert!(
            rendered.contains("\x1b[38;2;255;184;108m1000"),
            "expected mount param numeric values to use number styling"
        );
        assert!(
            rendered.contains("\x1b[38;2;139;233;253mlocked")
                && rendered.contains("\x1b[38;2;139;233;253mcache"),
            "expected mount param enum-like values to use type-style highlighting"
        );
        assert!(
            rendered.contains("\x1b[38;2;241;250;140mcache-") && rendered.contains("TARGETARCH"),
            "expected mount id values to keep string fragments while splitting embedded expansions"
        );
        assert!(
            rendered.contains("APP_ENV"),
            "expected env mount values to render as an independent identifier-like token"
        );
    }

    #[test]
    fn dockerfile_highlights_json_form_commands_and_paths_with_host_semantics() {
        let theme = Theme::for_mode(ColorMode::TrueColor);
        let path = fixture_path("dockerfile/Dockerfile.exec");
        let source = read_file(&path);
        let rendered = render_with_theme(Some(path.as_path()), &source, &theme)
            .unwrap_or_else(|error| panic!("failed to render {}: {error}", path.display()));

        assert!(
            rendered.contains("\x1b[38;2;139;233;253m\"python\""),
            "expected JSON-form CMD executable to use command-style highlighting"
        );
        assert!(
            rendered.contains("\x1b[38;2;139;233;253m\"sh\""),
            "expected JSON-form ENTRYPOINT executable to use command-style highlighting"
        );
        assert!(
            rendered.contains("\x1b[38;2;139;233;253m\"zsh\""),
            "expected SHELL executable to use command-style highlighting"
        );
        assert!(
            rendered.contains("\x1b[38;2;241;250;140m\"app\"")
                && rendered.contains("\x1b[38;2;241;250;140m\"echo hi\""),
            "expected non-option JSON-form argv entries to remain string styled"
        );
        assert!(
            rendered.contains("\x1b[38;2;241;250;140m\"/data\""),
            "expected JSON-form VOLUME paths to keep string styling"
        );
        assert!(
            rendered.contains("\x1b[3m\x1b[38;2;80;250;123m\"-m\"")
                && rendered.contains("\x1b[3m\x1b[38;2;80;250;123m\"-lc\"")
                && rendered.contains("\x1b[3m\x1b[38;2;80;250;123m\"--config\""),
            "expected JSON-form option argv entries to use attribute styling"
        );
        assert!(
            rendered.contains("\x1b[38;2;241;250;140m\"./bin/server\""),
            "expected path-like JSON-form executables to keep path/string styling"
        );
    }

    #[test]
    fn dockerfile_highlights_mount_type_families_with_key_aware_semantics() {
        let theme = Theme::for_mode(ColorMode::TrueColor);
        let path = fixture_path("dockerfile/Dockerfile.mount-kinds");
        let source = read_file(&path);
        let rendered = render_with_theme(Some(path.as_path()), &source, &theme)
            .unwrap_or_else(|error| panic!("failed to render {}: {error}", path.display()));

        assert!(
            rendered.contains("\x1b[38;2;139;233;253msecret")
                && rendered.contains("\x1b[38;2;139;233;253mssh")
                && rendered.contains("\x1b[38;2;139;233;253mbind"),
            "expected mount type values to keep family/type styling"
        );
        assert!(
            rendered.contains("\x1b[38;2;241;250;140mapi-key")
                && rendered.contains("\x1b[38;2;241;250;140mdefault"),
            "expected secret/ssh ids to use string-like styling"
        );
        assert!(
            rendered.contains("\x1b[38;2;189;147;249mrequired")
                && rendered.contains("\x1b[38;2;189;147;249mreadonly"),
            "expected bare flags in mount families to keep constant styling"
        );
        assert!(
            rendered.contains("API_KEY"),
            "expected env mount values to remain an independently rendered identifier-like token"
        );
        assert!(
            rendered.contains("\x1b[38;2;241;250;140m./hack")
                && rendered.contains("\x1b[38;2;241;250;140m/work"),
            "expected bind mount source and target paths to keep string/path styling"
        );
    }

    #[test]
    fn dockerfile_highlights_json_form_with_unified_command_option_path_env_and_expansion_semantics()
     {
        let theme = Theme::for_mode(ColorMode::TrueColor);
        let path = fixture_path("dockerfile/Dockerfile.exec-advanced");
        let source = read_file(&path);
        let rendered = render_with_theme(Some(path.as_path()), &source, &theme)
            .unwrap_or_else(|error| panic!("failed to render {}: {error}", path.display()));

        assert!(
            rendered.contains("\x1b[38;2;241;250;140m\"./bin/server\""),
            "expected path-like JSON-form executable to stay string styled"
        );
        assert!(
            rendered.contains("\x1b[3m\x1b[38;2;80;250;123m\"--config\"")
                && rendered.contains("\x1b[3m\x1b[38;2;80;250;123m\"--root\""),
            "expected JSON-form options to keep attribute styling across commands"
        );
        assert!(
            rendered.contains("\x1b[38;2;241;250;140m\"/etc/app.toml\"")
                && rendered.contains("\x1b[38;2;241;250;140m\"/workspace\""),
            "expected path-like JSON-form argv values to use string/path styling"
        );
        assert!(
            rendered.contains("APP_ENV=prod"),
            "expected env-style JSON-form argv to render as its own structured token"
        );
        assert!(
            rendered.contains("PORT"),
            "expected JSON-form expansion argv to keep expansion semantics visible"
        );
    }

    #[test]
    fn rustdoc_comments_highlight_markdown_content() {
        let theme = Theme::for_mode(ColorMode::TrueColor);
        let path = fixture_path("rust/doc_comments.rs");
        let source = read_file(&path);
        let rendered = render_with_theme(Some(path.as_path()), &source, &theme)
            .unwrap_or_else(|error| panic!("failed to render {}: {error}", path.display()));

        assert!(
            rendered.contains("\x1b[1m\x1b[38;2;189;147;249mPreview"),
            "expected rustdoc heading styling"
        );
        assert!(
            rendered.contains("\x1b[3m\x1b[38;2;241;250;140mmarkdown"),
            "expected rustdoc emphasis styling"
        );
        assert!(
            rendered.contains("\x1b[38;2;80;250;123minline code"),
            "expected rustdoc code-span styling"
        );
    }

    #[test]
    fn rustdoc_markdown_fenced_code_reuses_nested_language_runtimes() {
        let theme = Theme::for_mode(ColorMode::TrueColor);
        let path = fixture_path("rust/doc_comments_nested.rs");
        let source = read_file(&path);
        let rendered = render_with_theme(Some(path.as_path()), &source, &theme)
            .unwrap_or_else(|error| panic!("failed to render {}: {error}", path.display()));

        assert!(
            rendered.contains("\x1b[1m\x1b[38;2;189;147;249mGuide"),
            "expected rustdoc markdown heading styling"
        );
        assert!(
            rendered.contains("\x1b[38;2;255;121;198mfn"),
            "expected nested rust fenced block styling inside rustdoc"
        );
        assert!(
            rendered.contains("\x1b[38;2;139;233;253mNested"),
            "expected nested python class styling inside rustdoc"
        );
        assert!(
            rendered.contains("\x1b[38;2;80;250;123mrender"),
            "expected nested python method styling inside rustdoc"
        );
    }

    #[test]
    fn rustdoc_fenced_rust_does_not_inherit_markdown_literal_color_for_plain_identifiers() {
        let theme = Theme::for_mode(ColorMode::TrueColor);
        let path = Path::new("testdata/showcase/rust/macros.rs").to_path_buf();
        let source = read_file(&path);
        let rendered = render_with_theme(Some(path.as_path()), &source, &theme)
            .unwrap_or_else(|error| panic!("failed to render {}: {error}", path.display()));

        assert!(
            rendered.contains("\x1b[38;2;248;248;242m rendered "),
            "expected rustdoc fenced rust local binding to keep foreground styling"
        );
        assert!(
            !rendered.contains("\x1b[38;2;241;250;140m rendered = "),
            "plain rust identifier in rustdoc fence should not inherit markdown literal yellow"
        );
    }

    #[test]
    fn python_highlights_class_and_method_names() {
        let theme = Theme::for_mode(ColorMode::TrueColor);
        let path = fixture_path("python/main.py");
        let source = read_file(&path);
        let rendered = render_with_theme(Some(path.as_path()), &source, &theme)
            .unwrap_or_else(|error| panic!("failed to render {}: {error}", path.display()));

        assert!(
            rendered.contains("\x1b[38;2;139;233;253mGreeter"),
            "expected python class name styling"
        );
        assert!(
            rendered.contains("\x1b[38;2;80;250;123mhello"),
            "expected python method name styling"
        );
    }

    #[test]
    fn python_highlights_decorators_parameters_and_calls() {
        let theme = Theme::for_mode(ColorMode::TrueColor);
        let path = fixture_path("python/rich_features.py");
        let source = read_file(&path);
        let rendered = render_with_theme(Some(path.as_path()), &source, &theme)
            .unwrap_or_else(|error| panic!("failed to render {}: {error}", path.display()));

        assert!(
            rendered.contains("\x1b[38;2;255;121;198m@"),
            "expected decorator marker styling"
        );
        assert!(
            rendered.contains("\x1b[38;2;80;250;123mdataclass"),
            "expected decorator name styling"
        );
        assert!(
            rendered.contains("\x1b[3m\x1b[38;2;189;147;249mself"),
            "expected self parameter styling"
        );
        assert!(
            rendered.contains("\x1b[38;2;139;233;253mprint"),
            "expected builtin call styling"
        );
        assert!(
            rendered.contains("\x1b[38;2;139;233;253mThemePreview"),
            "expected class call styling"
        );
        assert!(
            rendered.contains("\x1b[38;2;80;250;123mrender"),
            "expected method call styling"
        );
    }

    #[test]
    fn python_highlights_advanced_semantics() {
        let theme = Theme::for_mode(ColorMode::TrueColor);
        let path = fixture_path("python/advanced.py");
        let source = read_file(&path);
        let rendered = render_with_theme(Some(path.as_path()), &source, &theme)
            .unwrap_or_else(|error| panic!("failed to render {}: {error}", path.display()));

        assert!(
            rendered.contains("\x1b[38;2;80;250;123mclassmethod"),
            "expected decorator builtin styling for classmethod"
        );
        assert!(
            rendered.contains("\x1b[38;2;80;250;123mbuild_default"),
            "expected function definition styling for build_default"
        );
        assert!(
            rendered.contains("\x1b[38;2;80;250;123m__init__"),
            "expected constructor method styling for __init__"
        );
        assert!(
            rendered.contains("\x1b[3m\x1b[38;2;139;233;253mstr"),
            "expected builtin type styling for str annotations"
        );
        assert!(
            rendered.contains("\x1b[38;2;139;233;253misinstance"),
            "expected builtin predicate styling for isinstance"
        );
    }

    #[test]
    fn sql_highlights_keywords_types_parameters_and_dollar_strings() {
        let theme = Theme::for_mode(ColorMode::TrueColor);
        let path = fixture_path("sql/rich.sql");
        let source = read_file(&path);
        let rendered = render_with_theme(Some(path.as_path()), &source, &theme)
            .unwrap_or_else(|error| panic!("failed to render {}: {error}", path.display()));

        assert!(
            rendered.contains("\x1b[3m\x1b[38;2;98;114;164m-- Dashboard query"),
            "expected SQL comment styling"
        );
        assert!(
            rendered.contains("\x1b[38;2;255;121;198mWITH"),
            "expected SQL keyword styling for WITH"
        );
        assert!(
            rendered.contains("\x1b[38;2;139;233;253mCOUNT"),
            "expected SQL function-call styling for COUNT"
        );
        assert!(
            rendered.contains("\x1b[3m\x1b[38;2;255;184;108m$1"),
            "expected SQL parameter styling"
        );
        assert!(
            rendered.contains("\x1b[38;2;241;250;140m$kat$dracula$kat$"),
            "expected SQL dollar-quoted string styling"
        );
        assert!(
            rendered.contains("\x1b[38;2;255;184;108m10"),
            "expected SQL numeric literal styling"
        );
    }

    #[test]
    fn sql_dialect_detector_uses_path_and_content_signals() {
        assert_eq!(
            detect_sql_dialect(Some(Path::new("db/schema.postgres.sql")), "SELECT 1;")
                .runtime_name(),
            "sql_postgres"
        );
        assert_eq!(
            detect_sql_dialect(Some(Path::new("db/schema.psql")), "SELECT 1;").runtime_name(),
            "sql_postgres"
        );
        assert_eq!(
            detect_sql_dialect(
                Some(Path::new("db/schema.sql")),
                "INSERT INTO themes VALUES ($1) ON CONFLICT (id) DO NOTHING RETURNING id;"
            )
            .runtime_name(),
            "sql_postgres"
        );
        assert_eq!(
            detect_sql_dialect(Some(Path::new("db/schema.mysql")), "SELECT 1;").runtime_name(),
            "sql_mysql"
        );
        assert_eq!(
            detect_sql_dialect(
                Some(Path::new("db/schema.sql")),
                "CREATE TABLE `themes` (`id` BIGINT UNSIGNED NOT NULL AUTO_INCREMENT) ENGINE=InnoDB;"
            )
            .runtime_name(),
            "sql_mysql"
        );
        assert_eq!(
            detect_sql_dialect(Some(Path::new("db/schema.sqlite3")), "SELECT 1;").runtime_name(),
            "sql_sqlite"
        );
        assert_eq!(
            detect_sql_dialect(
                Some(Path::new("db/schema.sql")),
                "CREATE TABLE themes (id INTEGER PRIMARY KEY AUTOINCREMENT) STRICT, WITHOUT ROWID;"
            )
            .runtime_name(),
            "sql_sqlite"
        );
        assert_eq!(
            detect_sql_dialect(
                Some(Path::new("db/schema.sql")),
                "CREATE FUNCTION refresh_theme_cache() RETURNS void LANGUAGE plpgsql IMMUTABLE PARALLEL SAFE AS $$ BEGIN RETURN; END; $$; CREATE INDEX theme_payload_gin ON theme_snapshots USING GIN (payload);"
            )
            .runtime_name(),
            "sql_postgres"
        );
        assert_eq!(
            detect_sql_dialect(
                Some(Path::new("db/schema.sql")),
                "CREATE UNLOGGED TABLE theme_cache (id bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY); CREATE EXTENSION IF NOT EXISTS pgcrypto;"
            )
            .runtime_name(),
            "sql_postgres"
        );
        assert_eq!(
            detect_sql_dialect(
                Some(Path::new("db/schema.sql")),
                "CREATE TABLE theme_snapshots (id INT UNSIGNED NOT NULL AUTO_INCREMENT, name VARCHAR(255) CHARACTER SET utf8mb4, PRIMARY KEY (id)) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4; INSERT INTO theme_snapshots (id, name) VALUES (1, 'Dracula') ON DUPLICATE KEY UPDATE name = VALUES(name);"
            )
            .runtime_name(),
            "sql_mysql"
        );
        assert_eq!(
            detect_sql_dialect(
                Some(Path::new("db/schema.sql")),
                "CREATE TABLE `theme_snapshots` (`id` BIGINT UNSIGNED ZEROFILL NOT NULL AUTO_INCREMENT) ENGINE=InnoDB; INSERT IGNORE INTO `theme_snapshots` (`id`) VALUES (1); SHOW CREATE TABLE `theme_snapshots`;"
            )
            .runtime_name(),
            "sql_mysql"
        );
        assert_eq!(
            detect_sql_dialect(
                Some(Path::new("db/schema.sql")),
                "PRAGMA foreign_keys = ON; VACUUM; INSERT OR IGNORE INTO theme_cache (slug) VALUES ('dracula');"
            )
            .runtime_name(),
            "sql_sqlite"
        );
        assert_eq!(
            detect_sql_dialect(
                Some(Path::new("db/schema.sql")),
                "BEGIN IMMEDIATE; ATTACH DATABASE 'theme-cache.db' AS cache; REINDEX; INSERT OR FAIL INTO theme_cache (slug) VALUES ('kat');"
            )
            .runtime_name(),
            "sql_sqlite"
        );
        assert_eq!(
            detect_sql_dialect(
                Some(Path::new("db/schema.mysql.sql")),
                "INSERT INTO theme_snapshots (payload) VALUES ($1::jsonb) RETURNING id;"
            )
            .runtime_name(),
            "sql_postgres"
        );
        assert_eq!(
            detect_sql_dialect(
                Some(Path::new("db/schema.sql")),
                "SELECT id, name FROM themes;"
            )
            .runtime_name(),
            "sql"
        );
    }

    #[test]
    fn sql_dialect_runtimes_highlight_postgres_mysql_and_sqlite_specific_tokens() {
        let theme = Theme::for_mode(ColorMode::TrueColor);

        let postgres_path = fixture_path("sql/schema.postgres.sql");
        let postgres_source = read_file(&postgres_path);
        let postgres_rendered =
            render_with_theme(Some(postgres_path.as_path()), &postgres_source, &theme)
                .unwrap_or_else(|error| {
                    panic!("failed to render {}: {error}", postgres_path.display())
                });
        assert!(
            postgres_rendered.contains("\x1b[38;2;139;233;253mjsonb"),
            "expected postgres runtime to highlight jsonb as builtin type"
        );
        assert!(
            postgres_rendered.contains("\x1b[38;2;255;121;198mUNLOGGED"),
            "expected postgres runtime to highlight UNLOGGED"
        );
        assert!(
            postgres_rendered.contains("\x1b[38;2;255;121;198mALWAYS"),
            "expected postgres runtime to highlight ALWAYS"
        );
        assert!(
            postgres_rendered.contains("\x1b[38;2;255;121;198mRETURNING"),
            "expected postgres runtime to highlight RETURNING"
        );
        assert!(
            postgres_rendered.contains("\x1b[38;2;241;250;140m$tag$dracula$tag$"),
            "expected postgres runtime to highlight dollar-quoted strings"
        );

        let mysql_path = fixture_path("sql/schema.mysql.sql");
        let mysql_source = read_file(&mysql_path);
        let mysql_rendered = render_with_theme(Some(mysql_path.as_path()), &mysql_source, &theme)
            .unwrap_or_else(|error| panic!("failed to render {}: {error}", mysql_path.display()));
        assert!(
            mysql_rendered.contains("\x1b[38;2;255;121;198mAUTO_INCREMENT"),
            "expected mysql runtime to highlight AUTO_INCREMENT"
        );
        assert!(
            mysql_rendered.contains("\x1b[38;2;255;121;198mENGINE"),
            "expected mysql runtime to highlight ENGINE"
        );
        assert!(
            mysql_rendered.contains("\x1b[38;2;255;121;198mZEROFILL"),
            "expected mysql runtime to highlight ZEROFILL"
        );
        assert!(
            mysql_rendered.contains("\x1b[38;2;255;121;198mREPLACE"),
            "expected mysql runtime to highlight REPLACE"
        );

        let sqlite_path = fixture_path("sql/schema.sqlite.sql");
        let sqlite_source = read_file(&sqlite_path);
        let sqlite_rendered =
            render_with_theme(Some(sqlite_path.as_path()), &sqlite_source, &theme).unwrap_or_else(
                |error| panic!("failed to render {}: {error}", sqlite_path.display()),
            );
        assert!(
            sqlite_rendered.contains("\x1b[38;2;255;121;198mWITHOUT"),
            "expected sqlite runtime to highlight WITHOUT"
        );
    }

    #[test]
    fn sql_dialect_runtimes_highlight_deeper_dialect_specific_constructs() {
        let theme = Theme::for_mode(ColorMode::TrueColor);

        let postgres_rendered = highlight_named_language(
            "sql_postgres",
            "CREATE FUNCTION refresh_theme_cache() RETURNS void LANGUAGE plpgsql IMMUTABLE PARALLEL SAFE AS $$ BEGIN RETURN; END; $$;\nCREATE INDEX theme_payload_gin ON theme_snapshots USING GIN (payload jsonb_path_ops);",
            &theme,
        )
        .expect("failed to highlight postgres SQL source");
        assert!(
            postgres_rendered.contains("\x1b[38;2;255;121;198mLANGUAGE"),
            "expected postgres runtime to highlight LANGUAGE"
        );
        assert!(
            postgres_rendered.contains("\x1b[38;2;255;121;198mIMMUTABLE"),
            "expected postgres runtime to highlight IMMUTABLE"
        );
        assert!(
            postgres_rendered.contains("\x1b[38;2;255;121;198mPARALLEL"),
            "expected postgres runtime to highlight PARALLEL"
        );
        assert!(
            postgres_rendered.contains("\x1b[3m\x1b[38;2;139;233;253mGIN"),
            "expected postgres runtime to highlight GIN as a builtin index method"
        );

        let mysql_rendered = highlight_named_language(
            "sql_mysql",
            "CREATE TABLE theme_snapshots (id INT UNSIGNED NOT NULL AUTO_INCREMENT, name VARCHAR(255) CHARACTER SET utf8mb4, PRIMARY KEY (id)) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;\nINSERT HIGH_PRIORITY IGNORE INTO theme_snapshots (id, name) VALUES (1, 'Dracula');\nREPLACE DELAYED LOW_PRIORITY INTO theme_snapshots (id, name) VALUES (2, 'Kat');",
            &theme,
        )
        .expect("failed to highlight mysql SQL source");
        assert!(
            mysql_rendered.contains("\x1b[38;2;255;121;198mSET"),
            "expected mysql runtime to highlight SET inside CHARACTER SET"
        );
        assert!(
            mysql_rendered.contains("\x1b[38;2;255;121;198mHIGH_PRIORITY"),
            "expected mysql runtime to highlight HIGH_PRIORITY"
        );
        assert!(
            mysql_rendered.contains("\x1b[38;2;255;121;198mIGNORE"),
            "expected mysql runtime to highlight IGNORE"
        );
        assert!(
            mysql_rendered.contains("\x1b[38;2;255;121;198mDELAYED"),
            "expected mysql runtime to highlight DELAYED"
        );
        assert!(
            mysql_rendered.contains("\x1b[38;2;255;121;198mUNSIGNED"),
            "expected mysql runtime to highlight UNSIGNED as a type qualifier"
        );

        let sqlite_rendered = highlight_named_language(
            "sql_sqlite",
            "PRAGMA foreign_keys = ON;\nVACUUM;\nINSERT OR IGNORE INTO theme_cache (slug) VALUES ('dracula');\nSELECT slug FROM theme_cache WHERE slug GLOB 'dr*';",
            &theme,
        )
        .expect("failed to highlight sqlite SQL source");
        assert!(
            sqlite_rendered.contains("\x1b[38;2;255;121;198mVACUUM"),
            "expected sqlite runtime to highlight VACUUM"
        );
        assert!(
            sqlite_rendered.contains("\x1b[38;2;255;121;198mIGNORE"),
            "expected sqlite runtime to highlight IGNORE"
        );
    }

    #[test]
    fn javascript_python_rust_and_regex_hosts_reuse_nested_sql_and_regex_runtimes() {
        let theme = Theme::for_mode(ColorMode::TrueColor);

        let js_path = fixture_path("javascript/injections.js");
        let js_source = read_file(&js_path);
        let js_rendered = render_with_theme(Some(js_path.as_path()), &js_source, &theme)
            .unwrap_or_else(|error| panic!("failed to render {}: {error}", js_path.display()));
        assert!(
            js_rendered.contains("\x1b[38;2;255;121;198mSELECT"),
            "expected injected SQL keyword styling in JavaScript"
        );
        assert!(
            js_rendered.contains("\x1b[38;2;255;121;198mRETURNING"),
            "expected JavaScript sql:postgres hint to route through postgres SQL runtime"
        );
        assert!(
            js_rendered.contains("\x1b[38;2;255;121;198mAUTO_INCREMENT"),
            "expected JavaScript sql:mysql comment-hosted string to route through mysql SQL runtime"
        );
        assert!(
            js_rendered.contains("\x1b[38;2;139;233;253mkind"),
            "expected injected regex named group styling in JavaScript"
        );
        assert!(
            js_rendered.contains("\x1b[38;2;255;121;198m\\b"),
            "expected injected regex escape styling in JavaScript"
        );
        assert!(
            js_rendered.contains("\x1b[38;2;139;233;253msection"),
            "expected JavaScript RegExp constructor pattern to reuse regex named group styling"
        );
        assert!(
            js_rendered.contains("\x1b[38;2;139;233;253mescapedSection"),
            "expected JavaScript escaped RegExp constructor pattern to reuse regex named group styling"
        );
        assert!(
            js_rendered.contains("\x1b[4m\x1b[38;2;255;85;85m(?P<legacy>"),
            "expected JavaScript template-string RegExp constructor to use javascript-specific regex runtime"
        );

        let python_path = fixture_path("python/injections.py");
        let python_source = read_file(&python_path);
        let python_rendered =
            render_with_theme(Some(python_path.as_path()), &python_source, &theme).unwrap_or_else(
                |error| panic!("failed to render {}: {error}", python_path.display()),
            );
        assert!(
            python_rendered.contains("\x1b[38;2;255;121;198mWITHOUT"),
            "expected Python sql:sqlite hint to route through sqlite SQL runtime"
        );
        assert!(
            python_rendered.contains("\x1b[38;2;139;233;253msection"),
            "expected injected regex named group styling in Python"
        );
        assert!(
            python_rendered.contains("\x1b[4m\x1b[38;2;255;85;85m\\p"),
            "expected Python regex injection to flag unicode property escapes as invalid"
        );
        assert!(
            python_rendered.contains("\x1b[38;2;139;233;253mword"),
            "expected Python regex injection to highlight named backreference group labels"
        );
        assert!(
            python_rendered.contains("\x1b[38;2;139;233;253mescaped_section"),
            "expected Python escaped regex string case to reuse regex named group styling"
        );

        let rust_path = fixture_path("rust/injections.rs");
        let rust_source = read_file(&rust_path);
        let rust_rendered = render_with_theme(Some(rust_path.as_path()), &rust_source, &theme)
            .unwrap_or_else(|error| panic!("failed to render {}: {error}", rust_path.display()));
        assert!(
            rust_rendered.contains("\x1b[38;2;255;121;198mSELECT"),
            "expected injected SQL keyword styling in Rust"
        );
        assert!(
            rust_rendered.contains("\x1b[38;2;139;233;253mkind"),
            "expected injected regex named group styling in Rust"
        );
        assert!(
            rust_rendered.contains("\x1b[38;2;255;121;198mi"),
            "expected injected regex inline flag character styling in Rust"
        );
        assert!(
            rust_rendered.contains("\x1b[3m\x1b[38;2;139;233;253mL"),
            "expected Rust regex injection to highlight unicode property names"
        );
        assert!(
            rust_rendered.contains("\x1b[4m\x1b[38;2;255;85;85m(?P=word)"),
            "expected Rust regex injection to flag named backreferences as invalid"
        );
        assert!(
            rust_rendered.contains("\x1b[38;2;139;233;253mescaped_section"),
            "expected Rust escaped regex string case to reuse regex named group styling"
        );
    }

    #[test]
    fn regex_highlights_lookarounds_posix_unicode_and_inline_flags() {
        let theme = Theme::for_mode(ColorMode::TrueColor);
        let rendered = highlight_named_language(
            "regex",
            r"(?im-s:(?<=\b)(?<word>[[:alpha:]]+)\p{Script=Latin}(?P=word)(?!\d)){2,4}?",
            &theme,
        )
        .expect("failed to highlight regex source");

        assert!(
            rendered.contains("\x1b[38;2;255;121;198mim-s"),
            "expected regex inline flags styling"
        );
        assert!(
            rendered.contains("\x1b[3m\x1b[38;2;139;233;253malpha"),
            "expected regex POSIX class name styling"
        );
        assert!(
            rendered.contains("\x1b[38;2;255;121;198m\\p"),
            "expected regex unicode property escape styling"
        );
        assert!(
            rendered.contains("\x1b[3m\x1b[38;2;139;233;253mScript"),
            "expected regex unicode property name styling"
        );
        assert!(
            rendered.contains("\x1b[3m\x1b[38;2;139;233;253mLatin"),
            "expected regex unicode property value styling"
        );
        assert!(
            rendered.contains("\x1b[38;2;255;121;198m="),
            "expected regex unicode property separator styling"
        );
        assert!(
            rendered.contains("\x1b[38;2;255;184;108m2"),
            "expected regex quantifier lower bound styling"
        );
    }

    #[test]
    fn regex_semantic_overlay_reports_quantifiers_unicode_and_class_ranges() {
        let output = debug_semantics("regex", r"(?im-s:\p{Script=Latin}[a-z0-9-]{2,4})")
            .expect("failed to render regex semantic overlay");

        assert!(
            output.contains("keyword.operator.regex") && output.contains("im"),
            "expected regex semantic overlay to include inline flag spans: {output}"
        );
        assert!(
            output.contains("operator.regex") && output.contains("\\p"),
            "expected regex semantic overlay to include unicode property escape spans: {output}"
        );
        assert!(
            output.contains("type.builtin") && output.contains("Script"),
            "expected regex semantic overlay to include unicode property name spans: {output}"
        );
        assert!(
            output.contains("type.builtin") && output.contains("Latin"),
            "expected regex semantic overlay to include unicode property value spans: {output}"
        );
        assert!(
            output.contains("number.quantifier.regex") && output.contains("2"),
            "expected regex semantic overlay to include quantifier number spans: {output}"
        );
        assert!(
            output.contains("punctuation.delimiter") && output.contains(","),
            "expected regex semantic overlay to include quantifier delimiter spans: {output}"
        );
        assert!(
            output.contains("operator.regex") && output.contains("-"),
            "expected regex semantic overlay to include class range operator spans: {output}"
        );
    }

    #[test]
    fn regex_host_runtimes_flag_unsupported_constructs() {
        let theme = Theme::for_mode(ColorMode::TrueColor);

        let python_rendered =
            highlight_named_language("regex_python", r"(?<name>\w+)\k<name>\p{L}", &theme)
                .expect("failed to highlight python regex source");
        assert!(
            python_rendered.contains("\x1b[4m\x1b[38;2;255;85;85m(?<name>"),
            "expected python regex runtime to flag JS-style named groups as invalid"
        );
        let rust_rendered =
            highlight_named_language("regex_rust", r"(?=foo)(?P<name>\w+)(?P=name)", &theme)
                .expect("failed to highlight rust regex source");
        assert!(
            rust_rendered.contains("\x1b[4m\x1b[38;2;255;85;85m(?=foo)"),
            "expected rust regex runtime to flag lookahead as invalid"
        );

        let go_rendered = highlight_named_language("regex_go", r"(?=foo)\1", &theme)
            .expect("failed to highlight go regex source");
        assert!(
            go_rendered.contains("\x1b[4m\x1b[38;2;255;85;85m(?=foo)"),
            "expected go regex runtime to flag lookahead as invalid"
        );

        let javascript_rendered =
            highlight_named_language("regex_javascript", r"(?i:theme)", &theme)
                .expect("failed to highlight javascript regex source");
        assert!(
            javascript_rendered.contains("\x1b[4m\x1b[38;2;255;85;85m(?i:theme)"),
            "expected javascript regex runtime to flag inline flag groups as invalid"
        );

        let posix_rendered = highlight_named_language("regex_posix", r"(?:theme)+?", &theme)
            .expect("failed to highlight posix regex source");
        assert!(
            posix_rendered.contains("\x1b[4m\x1b[38;2;255;85;85m(?:theme)"),
            "expected posix regex runtime to flag non-capturing groups as invalid"
        );
    }

    #[test]
    fn sql_like_api_calls_in_hosts_reuse_sql_runtime() {
        let theme = Theme::for_mode(ColorMode::TrueColor);

        let js_rendered = render_with_theme(
            Some(Path::new("inline.js")),
            "export async function load(db) { return db.query(\"SELECT id FROM themes WHERE slug = $1 RETURNING id\"); }",
            &theme,
        )
        .expect("failed to render inline javascript SQL source");
        assert!(
            js_rendered.contains("\x1b[38;2;255;121;198mSELECT"),
            "expected JavaScript db.query string to reuse SQL runtime"
        );
        assert!(
            js_rendered.contains("\x1b[3m\x1b[38;2;255;184;108m$1"),
            "expected JavaScript db.query string to preserve SQL parameter styling"
        );
        let js_sqlite_rendered = render_with_theme(
            Some(Path::new("inline.js")),
            "export function load(db) { db.get(\"SELECT id FROM themes WHERE slug = ?\"); db.run(\"INSERT INTO theme_cache (slug) VALUES (?)\"); }",
            &theme,
        )
        .expect("failed to render inline javascript sqlite-style SQL source");
        assert!(
            js_sqlite_rendered.contains("\x1b[38;2;255;121;198mSELECT"),
            "expected JavaScript db.get string to reuse SQL runtime"
        );
        assert!(
            js_sqlite_rendered.contains("\x1b[38;2;255;121;198mINSERT"),
            "expected JavaScript db.run string to reuse SQL runtime"
        );

        let python_rendered = render_with_theme(
            Some(Path::new("inline.py")),
            "def load(cursor):\n    return cursor.execute(\"SELECT id FROM themes WHERE slug = ?\")\n",
            &theme,
        )
        .expect("failed to render inline python SQL source");
        assert!(
            python_rendered.contains("\x1b[38;2;255;121;198mSELECT"),
            "expected Python cursor.execute string to reuse SQL runtime"
        );

        let go_rendered = render_with_theme(
            Some(Path::new("inline.go")),
            "package main\nfunc load(db DB) { db.Query(\"SELECT id FROM themes WHERE slug = $1\") }\n",
            &theme,
        )
        .expect("failed to render inline go SQL source");
        assert!(
            go_rendered.contains("\x1b[38;2;255;121;198mSELECT"),
            "expected Go db.Query string to reuse SQL runtime"
        );

        let rust_rendered = render_with_theme(
            Some(Path::new("inline.rs")),
            "fn load(client: Client) { let _ = client.query(\"SELECT id FROM themes WHERE slug = $1\", &[]); }\n",
            &theme,
        )
        .expect("failed to render inline rust SQL source");
        assert!(
            rust_rendered.contains("\x1b[38;2;255;121;198mSELECT"),
            "expected Rust client.query string to reuse SQL runtime"
        );
        let rust_prepare_rendered = render_with_theme(
            Some(Path::new("inline.rs")),
            "fn load(client: Client) { let _ = client.prepare_typed(\"SELECT id FROM themes WHERE slug = $1\", &[]); let _ = client.simple_query(\"SELECT id FROM themes\"); }\n",
            &theme,
        )
        .expect("failed to render inline rust extended SQL source");
        assert!(
            rust_prepare_rendered.contains("\x1b[38;2;255;121;198mSELECT"),
            "expected Rust prepare_typed/simple_query strings to reuse SQL runtime"
        );
    }

    #[test]
    fn detect_language_uses_extensionless_sql_and_graphql_heuristics() {
        assert!(matches!(
            detect_language(None, "SELECT id FROM themes WHERE enabled = true;"),
            Some(SupportedLanguage::Sql)
        ));
        assert!(matches!(
            detect_language(
                None,
                "query ThemeBySlug($slug: ID!) { theme(slug: $slug) { id } }"
            ),
            Some(SupportedLanguage::Graphql)
        ));
    }

    #[test]
    fn bash_and_just_sql_heredocs_reuse_sql_runtime() {
        let theme = Theme::for_mode(ColorMode::TrueColor);

        let bash_path = fixture_path("bash/sql_heredoc.sh");
        let bash_source = read_file(&bash_path);
        let bash_rendered = render_with_theme(Some(bash_path.as_path()), &bash_source, &theme)
            .unwrap_or_else(|error| panic!("failed to render {}: {error}", bash_path.display()));
        assert!(
            bash_rendered.contains("\x1b[38;2;255;121;198mRETURNING"),
            "expected psql heredoc to route through postgres SQL runtime"
        );
        assert!(
            bash_rendered.contains("\x1b[38;2;255;121;198mAUTO_INCREMENT"),
            "expected mysql heredoc to route through mysql SQL runtime"
        );
        assert!(
            bash_rendered.contains("\x1b[38;2;255;121;198mWITHOUT"),
            "expected sqlite heredoc to route through sqlite SQL runtime"
        );
        assert!(
            bash_rendered.contains("\x1b[38;2;80;250;123mpsql"),
            "expected Bash command styling to remain visible around SQL injection"
        );

        let just_path = fixture_path("just/heredoc_recipes.just");
        let just_source = read_file(&just_path);
        let just_rendered = render_with_theme(Some(just_path.as_path()), &just_source, &theme)
            .unwrap_or_else(|error| panic!("failed to render {}: {error}", just_path.display()));
        assert!(
            just_rendered.contains("\x1b[38;2;255;121;198mSELECT"),
            "expected Justfile Bash heredoc to reuse SQL runtime"
        );
    }

    #[test]
    fn javascript_highlights_classes_private_fields_regex_and_jsx() {
        let theme = Theme::for_mode(ColorMode::TrueColor);

        let rich_path = fixture_path("javascript/rich.js");
        let rich_source = read_file(&rich_path);
        let rich_rendered = render_with_theme(Some(rich_path.as_path()), &rich_source, &theme)
            .unwrap_or_else(|error| panic!("failed to render {}: {error}", rich_path.display()));

        assert!(
            rich_rendered.contains("\x1b[38;2;139;233;253mThemePreview"),
            "expected class definition styling for ThemePreview"
        );
        assert!(
            rich_rendered.contains("\x1b[38;2;248;248;242m#theme"),
            "expected private field property styling for #theme"
        );
        assert!(
            rich_rendered.contains("\x1b[38;2;80;250;123mrender"),
            "expected method definition styling for render"
        );
        assert!(
            rich_rendered.contains("\x1b[38;2;255;85;85m/dracula/"),
            "expected regex body styling for /dracula/"
        );
        assert!(
            rich_rendered.contains("\x1b[38;2;255;121;198mgi"),
            "expected regex flag styling for gi"
        );
        assert!(
            rich_rendered.contains("\x1b[3m\x1b[38;2;189;147;249mconsole"),
            "expected builtin console styling"
        );

        let jsx_path = fixture_path("javascript/component.jsx");
        let jsx_source = read_file(&jsx_path);
        let jsx_rendered = render_with_theme(Some(jsx_path.as_path()), &jsx_source, &theme)
            .unwrap_or_else(|error| panic!("failed to render {}: {error}", jsx_path.display()));

        assert!(
            jsx_rendered.contains("\x1b[38;2;255;121;198msection"),
            "expected intrinsic JSX tag styling for section"
        );
        assert!(
            jsx_rendered.contains("\x1b[38;2;139;233;253mPreviewCard"),
            "expected component JSX tag styling for PreviewCard"
        );
        assert!(
            jsx_rendered.contains("\x1b[3m\x1b[38;2;80;250;123mclassName"),
            "expected JSX attribute styling for className"
        );
    }

    #[test]
    fn css_highlights_selectors_namespace_custom_properties_and_units() {
        let theme = Theme::for_mode(ColorMode::TrueColor);
        let path = fixture_path("css/rich.css");
        let source = read_file(&path);
        let rendered = render_with_theme(Some(path.as_path()), &source, &theme)
            .unwrap_or_else(|error| panic!("failed to render {}: {error}", path.display()));

        assert!(
            rendered.contains("\x1b[38;2;255;121;198m@supports"),
            "expected at-rule styling for @supports"
        );
        assert!(
            rendered.contains("\x1b[38;2;255;121;198m#"),
            "expected id selector punctuation styling for #"
        );
        assert!(
            rendered.contains("\x1b[38;2;139;233;253mapp"),
            "expected id selector name styling for app"
        );
        assert!(
            rendered.contains("\x1b[38;2;255;121;198m."),
            "expected class selector punctuation styling for ."
        );
        assert!(
            rendered.contains("\x1b[38;2;139;233;253mpanel"),
            "expected class selector name styling for panel"
        );
        assert!(
            rendered.contains("\x1b[38;2;255;121;198m:"),
            "expected pseudo selector punctuation styling for :"
        );
        assert!(
            rendered.contains("\x1b[38;2;139;233;253mhover"),
            "expected pseudo selector name styling for hover"
        );
        assert!(
            rendered.contains("\x1b[38;2;189;147;249mrem"),
            "expected unit styling for rem"
        );
    }

    #[test]
    fn html_highlights_entities_custom_elements_and_nested_language_regions() {
        let theme = Theme::for_mode(ColorMode::TrueColor);
        let path = fixture_path("html/rich.html");
        let source = read_file(&path);
        let rendered = render_with_theme(Some(path.as_path()), &source, &theme)
            .unwrap_or_else(|error| panic!("failed to render {}: {error}", path.display()));

        assert!(
            rendered.contains("\x1b[38;2;255;121;198msection"),
            "expected html element styling for section"
        );
        assert!(
            rendered.contains("\x1b[38;2;241;250;140m&amp;"),
            "expected entity styling for &amp;"
        );
        assert!(
            rendered.contains("\x1b[38;2;255;121;198mtheme-card"),
            "expected custom element tag styling for theme-card"
        );
        assert!(
            rendered.contains("\x1b[3m\x1b[38;2;80;250;123maccent"),
            "expected html attribute styling for accent"
        );
    }

    #[test]
    fn jsdoc_highlights_tags_types_and_identifiers() {
        let theme = Theme::for_mode(ColorMode::TrueColor);
        let rendered = highlight_named_language(
            "jsdoc",
            "/**\n * @param {ThemePreview} preview Render the preview state.\n */",
            &theme,
        )
        .expect("failed to highlight jsdoc source");

        assert!(
            rendered.contains("\x1b[38;2;255;121;198m@param"),
            "expected jsdoc tag styling for @param"
        );
        assert!(
            rendered.contains("\x1b[38;2;139;233;253mThemePreview"),
            "expected jsdoc type styling for ThemePreview"
        );
    }

    #[test]
    fn jsdoc_highlights_optionals_defaults_inline_tags_and_code_fences() {
        let theme = Theme::for_mode(ColorMode::TrueColor);
        let rendered = highlight_named_language(
            "jsdoc",
            "/**\n * Render {@link ThemePreview} for docs.\n * @param {ThemePreview} [preview=ThemeStore.default] Preview value.\n * @param {number} [retryCount=3] Retries.\n * @see theme/preview\n * ```js\n * console.log(preview)\n * ```\n */",
            &theme,
        )
        .expect("failed to highlight jsdoc source");

        assert!(
            rendered.contains("\x1b[38;2;255;121;198m@param"),
            "expected jsdoc tag styling for @param"
        );
        assert!(
            rendered.contains("\x1b[38;2;248;248;242m["),
            "expected jsdoc optional parameter bracket styling"
        );
        assert!(
            rendered.contains("\x1b[38;2;255;121;198m="),
            "expected jsdoc default-value operator styling"
        );
        assert!(
            rendered.contains("\x1b[38;2;255;184;108m3"),
            "expected jsdoc numeric default styling"
        );
        assert!(
            rendered.contains("\x1b[38;2;255;121;198m."),
            "expected jsdoc member-expression delimiter styling"
        );
        assert!(
            rendered.contains("\x1b[38;2;241;250;140mjs"),
            "expected jsdoc code-block language styling"
        );
    }

    #[test]
    fn sql_semantic_overlay_reports_dialect_specific_identifiers() {
        let postgres_output = debug_semantics(
            "sql_postgres",
            "CREATE FUNCTION refresh_theme_cache() RETURNS void LANGUAGE plpgsql IMMUTABLE PARALLEL SAFE AS $$ BEGIN RETURN; END; $$;\nCREATE INDEX theme_payload_gin ON theme_snapshots USING GIN (payload jsonb_path_ops);",
        )
        .expect("failed to render postgres semantic overlay");
        assert!(
            postgres_output.contains("type.builtin") && postgres_output.contains("plpgsql"),
            "expected postgres semantic overlay to include function language spans: {postgres_output}"
        );
        assert!(
            postgres_output.contains("type.builtin") && postgres_output.contains("jsonb_path_ops"),
            "expected postgres semantic overlay to include opclass spans: {postgres_output}"
        );

        let mysql_output = debug_semantics(
            "sql_mysql",
            "CREATE TABLE theme_snapshots (name VARCHAR(255)) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;",
        )
        .expect("failed to render mysql semantic overlay");
        assert!(
            mysql_output.contains("type.builtin") && mysql_output.contains("InnoDB"),
            "expected mysql semantic overlay to include engine value spans: {mysql_output}"
        );
        assert!(
            mysql_output.contains("type.builtin") && mysql_output.contains("utf8mb4"),
            "expected mysql semantic overlay to include charset value spans: {mysql_output}"
        );
    }

    #[test]
    fn jsdoc_semantic_overlay_reports_inline_reference_targets() {
        let output = debug_semantics(
            "jsdoc",
            "/**\n * Render {@link ThemePreview#render} and {@link module:theme/preview}.\n */",
        )
        .expect("failed to render jsdoc semantic overlay");

        assert!(
            output.contains("variable.jsdoc") && output.contains("ThemePreview"),
            "expected jsdoc semantic overlay to include member reference identifiers: {output}"
        );
        assert!(
            output.contains("punctuation.delimiter") && output.contains("#"),
            "expected jsdoc semantic overlay to include member delimiter spans: {output}"
        );
        assert!(
            output.contains("text.uri") && output.contains("theme"),
            "expected jsdoc semantic overlay to include path-like inline reference spans: {output}"
        );
        assert!(
            output.contains("punctuation.delimiter") && output.contains("/"),
            "expected jsdoc semantic overlay to include path delimiter spans: {output}"
        );
    }

    #[test]
    fn graphql_runtime_is_reused_by_javascript_and_markdown_hosts() {
        let theme = Theme::for_mode(ColorMode::TrueColor);

        let graphql_path = fixture_path("graphql/schema.graphql");
        let graphql_source = read_file(&graphql_path);
        let graphql_rendered =
            render_with_theme(Some(graphql_path.as_path()), &graphql_source, &theme)
                .unwrap_or_else(|error| {
                    panic!("failed to render {}: {error}", graphql_path.display())
                });
        assert!(
            graphql_rendered.contains("\x1b[38;2;255;121;198mquery"),
            "expected top-level GraphQL keyword styling"
        );
        assert!(
            graphql_rendered.contains("\x1b[38;2;139;233;253mTheme"),
            "expected top-level GraphQL type styling"
        );
        assert!(
            graphql_rendered.contains("\x1b[38;2;255;121;198m$"),
            "expected top-level GraphQL variable sigil styling"
        );

        let js_path = fixture_path("javascript/graphql.js");
        let js_source = read_file(&js_path);
        let js_rendered = render_with_theme(Some(js_path.as_path()), &js_source, &theme)
            .unwrap_or_else(|error| panic!("failed to render {}: {error}", js_path.display()));
        assert!(
            js_rendered.contains("\x1b[38;2;255;121;198mfragment"),
            "expected JavaScript gql tagged template to reuse GraphQL keyword styling"
        );
        assert!(
            js_rendered.contains("\x1b[38;2;80;250;123mThemeBySlug"),
            "expected JavaScript gql tagged template to reuse GraphQL operation styling"
        );
        assert!(
            js_rendered.contains("\x1b[38;2;255;121;198m$"),
            "expected JavaScript comment-hosted GraphQL string to reuse GraphQL variable styling"
        );

        let markdown_path = fixture_path("markdown/graphql_fence.md");
        let markdown_source = read_file(&markdown_path);
        let markdown_rendered =
            render_with_theme(Some(markdown_path.as_path()), &markdown_source, &theme)
                .unwrap_or_else(|error| {
                    panic!("failed to render {}: {error}", markdown_path.display())
                });
        assert!(
            markdown_rendered.contains("\x1b[38;2;255;121;198mquery"),
            "expected markdown fenced graphql block to reuse GraphQL keyword styling"
        );
        assert!(
            markdown_rendered.contains("\x1b[38;2;80;250;123mThemeBySlug"),
            "expected markdown fenced graphql block to reuse GraphQL operation styling"
        );
        assert!(
            markdown_rendered.contains("\x1b[38;2;255;121;198mfragment"),
            "expected markdown fenced gql alias to reuse GraphQL keyword styling"
        );
    }

    #[test]
    fn injected_python_reuses_the_same_highlight_rules() {
        let theme = Theme::for_mode(ColorMode::TrueColor);

        let just_path = fixture_path("just/injections.just");
        let just_source = read_file(&just_path);
        let just_rendered = render_with_theme(Some(just_path.as_path()), &just_source, &theme)
            .unwrap_or_else(|error| panic!("failed to render {}: {error}", just_path.display()));

        assert!(
            just_rendered.contains("\x1b[38;2;139;233;253mPreview"),
            "expected injected python class styling in Justfile"
        );
        assert!(
            just_rendered.contains("\x1b[38;2;80;250;123mrender"),
            "expected injected python method styling in Justfile"
        );

        let markdown_path =
            Path::new("testdata/showcase/markdown/mixed-code-blocks.md").to_path_buf();
        let markdown_source = read_file(&markdown_path);
        let markdown_rendered =
            render_with_theme(Some(markdown_path.as_path()), &markdown_source, &theme)
                .unwrap_or_else(|error| {
                    panic!("failed to render {}: {error}", markdown_path.display())
                });

        assert!(
            markdown_rendered.contains("\x1b[38;2;139;233;253mPreview"),
            "expected injected python class styling in Markdown"
        );
        assert!(
            markdown_rendered.contains("\x1b[38;2;80;250;123mrender"),
            "expected injected python method styling in Markdown"
        );
    }

    #[test]
    fn userscript_metadata_block_highlights_keys_urls_patterns_and_special_values() {
        let theme = Theme::for_mode(ColorMode::TrueColor);
        let path = fixture_path("javascript/userscript.user.js");
        let source = read_file(&path);
        let rendered = render_with_theme(Some(path.as_path()), &source, &theme)
            .unwrap_or_else(|error| panic!("failed to render {}: {error}", path.display()));

        assert!(
            rendered.contains("\x1b[3m\x1b[38;2;80;250;123m==UserScript=="),
            "expected userscript start marker styling"
        );
        assert!(
            rendered.contains("\x1b[38;2;139;233;253mname"),
            "expected userscript metadata key styling"
        );
        assert!(
            rendered.contains("\x1b[3m\x1b[38;2;80;250;123m:zh-CN"),
            "expected userscript localized key suffix styling"
        );
        assert!(
            rendered.contains("\x1b[38;2;139;233;253mhttps://example.com/kat"),
            "expected userscript URL styling"
        );
        assert!(
            rendered.contains("\x1b[38;2;255;85;85m*://example.com/*"),
            "expected userscript match-pattern styling"
        );
        assert!(
            rendered.contains("\x1b[38;2;139;233;253mGM_addStyle"),
            "expected userscript grant API styling"
        );
        assert!(
            rendered.contains("\x1b[38;2;189;147;249mnone"),
            "expected userscript special grant value styling"
        );
        assert!(
            rendered.contains("icon"),
            "expected userscript resource alias styling"
        );
        assert!(
            rendered.contains("\x1b[38;2;189;147;249mdocument-start"),
            "expected userscript run-at enum styling"
        );
    }

    #[test]
    fn userscript_metadata_highlighting_reuses_javascript_runtime_in_markdown_fences() {
        let theme = Theme::for_mode(ColorMode::TrueColor);
        let path = fixture_path("markdown/userscript_fence.md");
        let source = read_file(&path);
        let rendered = render_with_theme(Some(path.as_path()), &source, &theme)
            .unwrap_or_else(|error| panic!("failed to render {}: {error}", path.display()));

        assert!(
            rendered.contains("\x1b[3m\x1b[38;2;80;250;123m==UserScript=="),
            "expected fenced JavaScript userscript marker styling in Markdown"
        );
        assert!(
            rendered.contains("\x1b[38;2;139;233;253mgrant"),
            "expected fenced JavaScript userscript key styling in Markdown"
        );
        assert!(
            rendered.contains("\x1b[38;2;139;233;253mGM_addStyle"),
            "expected fenced JavaScript userscript grant styling in Markdown"
        );
        assert!(
            rendered.contains("\x1b[38;2;189;147;249mdocument-end"),
            "expected fenced JavaScript userscript enum styling in Markdown"
        );
    }

    #[test]
    fn json_highlights_keys_booleans_escapes_and_punctuation() {
        let theme = Theme::for_mode(ColorMode::TrueColor);
        let path = fixture_path("json/rich.json");
        let source = read_file(&path);
        let rendered = render_with_theme(Some(path.as_path()), &source, &theme)
            .unwrap_or_else(|error| panic!("failed to render {}: {error}", path.display()));

        assert!(
            rendered.contains("\x1b[38;2;139;233;253m\"theme\""),
            "expected JSON object key styling"
        );
        assert!(
            rendered.contains("\x1b[38;2;255;184;108mtrue"),
            "expected JSON boolean styling"
        );
        assert!(
            rendered.contains("\x1b[38;2;189;147;249mnull"),
            "expected JSON null styling"
        );
        assert!(
            rendered.contains("\x1b[38;2;255;121;198m\\\\"),
            "expected JSON escape sequence styling"
        );
        assert!(
            rendered.contains("\x1b[38;2;248;248;242m{"),
            "expected JSON bracket punctuation styling"
        );
        assert!(
            rendered.contains("\x1b[38;2;255;121;198m:"),
            "expected JSON delimiter punctuation styling"
        );
    }

    #[test]
    fn bash_highlights_shebang_special_variables_regex_and_parameters() {
        let theme = Theme::for_mode(ColorMode::TrueColor);
        let path = fixture_path("bash/rich.sh");
        let source = read_file(&path);
        let rendered = render_with_theme(Some(path.as_path()), &source, &theme)
            .unwrap_or_else(|error| panic!("failed to render {}: {error}", path.display()));

        assert!(
            rendered.contains("\x1b[3m\x1b[38;2;80;250;123m#!/usr/bin/env bash"),
            "expected Bash shebang directive styling"
        );
        assert!(
            rendered.contains("\x1b[38;2;255;121;198m=~"),
            "expected Bash regex operator styling"
        );
        assert!(
            rendered.contains("\x1b[38;2;255;85;85m^kat.*"),
            "expected Bash regex literal styling"
        );
        assert!(
            rendered.contains("\x1b[3m\x1b[38;2;189;147;249m#"),
            "expected Bash special variable styling"
        );
        assert!(
            rendered.contains("\x1b[3m\x1b[38;2;80;250;123mpipefail"),
            "expected Bash set option styling"
        );
        assert!(
            rendered.contains("\x1b[38;2;241;250;140m$'line\\nsecond'"),
            "expected Bash ANSI-C string styling"
        );
        assert!(
            rendered.contains("\x1b[38;2;139;233;253mread"),
            "expected Bash read builtin styling"
        );
        assert!(
            rendered.contains("\x1b[38;2;139;233;253mdeclare"),
            "expected Bash declare builtin styling"
        );
        assert!(
            rendered.contains("\x1b[38;2;139;233;253munset"),
            "expected Bash unset builtin styling"
        );
        assert!(
            rendered.contains("\x1b[3m\x1b[38;2;255;184;108mtheme_line"),
            "expected Bash read target variable styling"
        );
        assert!(
            rendered.contains("\x1b[38;2;248;248;242m["),
            "expected Bash subscript bracket styling"
        );
        assert!(
            rendered.contains("\x1b[38;2;255;184;108m0"),
            "expected Bash subscript index number styling"
        );
    }

    #[test]
    fn toml_highlights_quoted_keys_datetimes_inline_tables_and_cargo_lock() {
        let theme = Theme::for_mode(ColorMode::TrueColor);

        let rich_path = fixture_path("toml/rich.toml");
        let rich_source = read_file(&rich_path);
        let rich_rendered = render_with_theme(Some(rich_path.as_path()), &rich_source, &theme)
            .unwrap_or_else(|error| panic!("failed to render {}: {error}", rich_path.display()));

        assert!(
            rich_rendered.contains("\x1b[38;2;139;233;253m\"display-name\""),
            "expected TOML quoted-key property styling"
        );
        assert!(
            rich_rendered.contains("\x1b[38;2;255;121;198m\\n"),
            "expected TOML escape sequence styling"
        );
        assert!(
            rich_rendered.contains("\x1b[38;2;255;184;108m2026-03-26T18:00:00Z"),
            "expected TOML datetime styling"
        );
        assert!(
            rich_rendered.contains("\x1b[38;2;248;248;242m{"),
            "expected TOML inline-table bracket styling"
        );
        assert!(
            rich_rendered.contains("\x1b[38;2;255;184;108mtrue"),
            "expected TOML boolean styling inside inline table"
        );

        let cargo_lock_path = fixture_path("toml/Cargo.lock");
        let cargo_lock_source = read_file(&cargo_lock_path);
        let cargo_lock_rendered =
            render_with_theme(Some(cargo_lock_path.as_path()), &cargo_lock_source, &theme)
                .unwrap_or_else(|error| {
                    panic!("failed to render {}: {error}", cargo_lock_path.display())
                });

        assert!(
            cargo_lock_rendered.contains("\x1b["),
            "expected Cargo.lock to be detected as TOML"
        );
        assert!(
            cargo_lock_rendered.contains("\x1b[38;2;139;233;253mversion"),
            "expected Cargo.lock keys to reuse TOML property styling"
        );

        let uv_lock_path = fixture_path("toml/uv.lock");
        let uv_lock_source = read_file(&uv_lock_path);
        let uv_lock_rendered =
            render_with_theme(Some(uv_lock_path.as_path()), &uv_lock_source, &theme)
                .unwrap_or_else(|error| {
                    panic!("failed to render {}: {error}", uv_lock_path.display())
                });

        assert!(
            uv_lock_rendered.contains("\x1b["),
            "expected uv.lock to be detected as TOML"
        );
        assert!(
            uv_lock_rendered.contains("\x1b[38;2;139;233;253mversion"),
            "expected uv.lock keys to reuse TOML property styling"
        );
    }

    #[test]
    fn yaml_highlights_anchors_tags_block_scalars_and_github_script_injection() {
        let theme = Theme::for_mode(ColorMode::TrueColor);
        let path = fixture_path("yaml/actions.yaml");
        let source = read_file(&path);
        let rendered = render_with_theme(Some(path.as_path()), &source, &theme)
            .unwrap_or_else(|error| panic!("failed to render {}: {error}", path.display()));

        assert!(
            rendered.contains("\x1b[3m\x1b[4m\x1b[38;2;80;250;123m&defaults"),
            "expected YAML anchor styling"
        );
        assert!(
            rendered.contains("\x1b[3m\x1b[4m\x1b[38;2;80;250;123m*defaults"),
            "expected YAML alias styling"
        );
        assert!(
            rendered.contains("\x1b[38;2;139;233;253m!color"),
            "expected YAML tag styling"
        );
        assert!(
            rendered.contains("\x1b[38;2;255;121;198m|"),
            "expected YAML block-scalar indicator styling"
        );
        assert!(
            rendered.contains("\x1b[3m\x1b[38;2;189;147;249mconsole"),
            "expected YAML github-script block to inject JavaScript"
        );
        assert!(
            rendered.contains("\x1b[38;2;255;121;198mconst"),
            "expected YAML github-script block to reuse JavaScript keyword styling"
        );
    }

    #[test]
    fn github_actions_yaml_profiles_inject_run_steps_and_highlight_expressions() {
        let theme = Theme::for_mode(ColorMode::TrueColor);
        let workflow_path = fixture_path("yaml/github-actions-workflow.yaml");
        let workflow_source = read_file(&workflow_path);
        let workflow_rendered = render_with_theme(
            Some(Path::new(".github/workflows/build.yml")),
            &workflow_source,
            &theme,
        )
        .unwrap_or_else(|error| panic!("failed to render {}: {error}", workflow_path.display()));
        let workflow_regions = collect_top_level_injection_regions(
            yaml_document_kind(Some(Path::new(".github/workflows/build.yml"))),
            &workflow_source,
            &theme,
        )
        .expect("expected GitHub Actions workflow injections to resolve");
        assert!(
            workflow_regions.iter().any(|region| {
                region.overlays.iter().any(|span| {
                    &workflow_source[span.range.clone()] == "printf"
                        && span.style == theme.token_style_for("function.builtin", "printf")
                })
            }),
            "expected GitHub Actions host resolver to produce Bash overlays for run blocks"
        );

        assert!(
            workflow_rendered.contains("\x1b[38;2;139;233;253mprintf"),
            "expected GitHub Actions run block to reuse Bash runtime styling"
        );
        assert!(
            workflow_rendered.contains("\x1b[38;2;139;233;253mprint"),
            "expected shell: python run block to reuse Python builtin styling"
        );
        assert!(
            workflow_rendered.contains("\x1b[38;2;248;248;242mgithub"),
            "expected GitHub Actions expressions to highlight context identifiers"
        );
        assert!(
            workflow_rendered.contains("\x1b[38;2;255;121;198m&&"),
            "expected GitHub Actions expressions to highlight logical operators"
        );
        assert!(
            workflow_rendered.contains("\x1b[38;2;139;233;253mactions"),
            "expected uses owner to receive dedicated GitHub Actions styling"
        );
        assert!(
            workflow_rendered.contains("\x1b[38;2;80;250;123mcheckout"),
            "expected uses repository to receive dedicated GitHub Actions styling"
        );

        let action_path = fixture_path("yaml/action.yaml");
        let action_source = read_file(&action_path);
        let action_rendered =
            render_with_theme(Some(Path::new("action.yml")), &action_source, &theme)
                .unwrap_or_else(|error| {
                    panic!("failed to render {}: {error}", action_path.display())
                });
        assert!(
            action_rendered.contains("\x1b[38;2;80;250;123mecho"),
            "expected composite action run step to reuse Bash runtime styling"
        );
        assert!(
            action_rendered.contains("\x1b[3m\x1b[38;2;139;233;253mcomposite"),
            "expected runs.using values to receive GitHub Actions schema-aware styling"
        );
    }

    #[test]
    fn github_actions_advanced_workflow_inherits_default_shell_and_highlights_nested_expressions() {
        let theme = Theme::for_mode(ColorMode::TrueColor);
        let workflow_path = fixture_path("yaml/github-actions-workflow-advanced.yaml");
        let workflow_source = read_file(&workflow_path);
        let workflow_rendered = render_with_theme(
            Some(Path::new(".github/workflows/build-matrix.yml")),
            &workflow_source,
            &theme,
        )
        .unwrap_or_else(|error| panic!("failed to render {}: {error}", workflow_path.display()));
        let workflow_regions = collect_top_level_injection_regions(
            yaml_document_kind(Some(Path::new(".github/workflows/build-matrix.yml"))),
            &workflow_source,
            &theme,
        )
        .expect("expected advanced GitHub Actions workflow injections to resolve");

        assert!(
            workflow_regions.iter().any(|region| {
                region.overlays.iter().any(|span| {
                    &workflow_source[span.range.clone()] == "set"
                        && span.style == theme.token_style_for("function.builtin", "set")
                })
            }),
            "expected defaults.run.shell to drive Bash highlighting for run blocks"
        );
        assert!(
            workflow_regions.iter().any(|region| {
                region.overlays.iter().any(|span| {
                    &workflow_source[span.range.clone()] == "print"
                        && span.style == theme.token_style_for("function.builtin", "print")
                })
            }),
            "expected shell templates like python {{0}} to resolve to the Python runtime"
        );
        assert!(
            workflow_rendered.contains("\x1b[38;2;255;121;198m&&"),
            "expected bare if expressions to reuse GitHub Actions operator styling"
        );
        assert!(
            workflow_rendered.contains("\x1b[38;2;255;121;198m||"),
            "expected bare if expressions to reuse GitHub Actions operator styling"
        );
        assert!(
            workflow_regions.iter().any(|region| {
                region.overlays.iter().any(|span| {
                    &workflow_source[span.range.clone()] == "${{"
                        && span.style == theme.token_style_for("punctuation.special", "${{")
                })
            }),
            "expected GitHub Actions expressions inside run blocks to keep workflow punctuation styling"
        );
        assert!(
            workflow_regions.iter().any(|region| {
                region.overlays.iter().any(|span| {
                    &workflow_source[span.range.clone()] == "target"
                        && span.style == theme.token_style_for("property", "target")
                })
            }),
            "expected nested run-block expressions to highlight GitHub Actions property segments"
        );
        assert!(
            workflow_rendered.contains("\x1b[38;2;255;121;198mdocker://"),
            "expected docker uses refs to highlight the docker:// prefix"
        );
        assert!(
            workflow_rendered.contains("\x1b[38;2;80;250;123msetup-node"),
            "expected uses refs with subpaths to keep repository highlighting"
        );
        assert!(
            workflow_rendered.contains("\x1b[38;2;241;250;140mbin"),
            "expected uses refs with subpaths to highlight the nested action path"
        );
        assert!(
            workflow_rendered.contains("\x1b[38;2;139;233;253mWrite-Host"),
            "expected pwsh run blocks to reuse PowerShell builtin highlighting"
        );
        assert!(
            workflow_rendered.contains("\x1b[38;2;255;121;198m@echo off"),
            "expected cmd run blocks to reuse batch highlighting"
        );
        assert!(
            workflow_rendered.contains("\x1b[3m\x1b[38;2;139;233;253mpwsh"),
            "expected shell values to receive schema-aware runtime styling"
        );
        assert!(
            workflow_rendered.contains("\x1b[3m\x1b[38;2;139;233;253mcmd"),
            "expected cmd shell values to receive schema-aware runtime styling"
        );
        assert!(
            workflow_rendered.contains("\x1b[3m\x1b[38;2;139;233;253mread"),
            "expected permissions values to receive schema-aware styling"
        );
        assert!(
            workflow_rendered.contains("\x1b[3m\x1b[38;2;139;233;253mpnpm"),
            "expected with.cache values to receive schema-aware styling"
        );
        assert!(
            workflow_rendered.contains("\x1b[3m\x1b[38;2;139;233;253merror"),
            "expected with.if-no-files-found values to receive schema-aware styling"
        );
        assert!(
            workflow_rendered.contains("\x1b[38;2;139;233;253mubuntu-latest"),
            "expected runs-on labels to receive GitHub Actions runner styling"
        );
        assert!(
            workflow_rendered.contains("\x1b[38;2;139;233;253mself-hosted"),
            "expected runs-on array labels to receive GitHub Actions runner styling"
        );
    }

    #[test]
    fn proto_highlights_keywords_types_and_literals() {
        let theme = Theme::for_mode(ColorMode::TrueColor);
        let rendered = render_with_theme(
            Some(Path::new("theme.proto")),
            r#"syntax = "proto3";

message ThemePreview {
  string name = 1;
  bool enabled = 2;
}
"#,
            &theme,
        )
        .expect("expected proto highlight to succeed");

        assert!(
            rendered.contains("\x1b["),
            "expected proto output to contain ANSI styling"
        );
        assert!(
            rendered.contains("syntax"),
            "expected proto keyword to be present in rendered output"
        );
        assert!(
            rendered.contains("\"proto3\""),
            "expected proto syntax literal to be present in rendered output"
        );
        assert!(
            rendered.contains("ThemePreview"),
            "expected proto message name to be present in rendered output"
        );
        assert!(
            rendered.contains("string"),
            "expected proto scalar type to be present in rendered output"
        );
        assert!(
            rendered.contains("1"),
            "expected proto field number to be present in rendered output"
        );
    }

    #[test]
    fn textproto_highlights_fields_strings_booleans_and_numbers() {
        let theme = Theme::for_mode(ColorMode::TrueColor);
        let rendered = render_with_theme(
            Some(Path::new("theme.textproto")),
            r#"name: "Dracula"
enabled: true
priority: 7
"#,
            &theme,
        )
        .expect("expected textproto highlight to succeed");

        assert!(
            rendered.contains("\x1b["),
            "expected textproto output to contain ANSI styling"
        );
        assert!(
            rendered.contains("name"),
            "expected textproto field name to be present in rendered output"
        );
        assert!(
            rendered.contains(":"),
            "expected textproto delimiter to be present in rendered output"
        );
        assert!(
            rendered.contains("\"Dracula\""),
            "expected textproto string to be present in rendered output"
        );
        assert!(
            rendered.contains("true"),
            "expected textproto boolean to be present in rendered output"
        );
        assert!(
            rendered.contains("7"),
            "expected textproto number to be present in rendered output"
        );
    }

    #[test]
    fn rust_highlights_traits_attributes_macros_and_special_variables() {
        let theme = Theme::for_mode(ColorMode::TrueColor);
        let path = fixture_path("rust/rich.rs");
        let source = read_file(&path);
        let rendered = render_with_theme(Some(path.as_path()), &source, &theme)
            .unwrap_or_else(|error| panic!("failed to render {}: {error}", path.display()));

        assert!(
            rendered.contains("\x1b[3m\x1b[38;2;80;250;123mderive"),
            "expected Rust attribute name styling"
        );
        assert!(
            rendered.contains("\x1b[38;2;139;233;253mRenderable"),
            "expected Rust trait/interface styling"
        );
        assert!(
            rendered.contains("\x1b[38;2;80;250;123mrender"),
            "expected Rust method call styling"
        );
        assert!(
            rendered.contains("\x1b[38;2;80;250;123mthemed"),
            "expected Rust macro styling"
        );
        assert!(
            rendered.contains("\x1b[3m\x1b[38;2;189;147;249mself"),
            "expected Rust self styling"
        );
        assert!(
            rendered.contains("\x1b[38;2;248;248;242m preview "),
            "expected Rust local variable bindings to keep foreground styling"
        );
    }

    #[test]
    fn just_recipe_body_uses_block_region_tint_without_tinting_header_line() {
        let source = "install:\n    pnpm install\n    cargo install --path .\n";
        let tint = RgbColor(1, 2, 3);
        let theme = Theme::for_mode_with_nested_region_tint(ColorMode::TrueColor, Some(tint));
        let rendered = render_with_theme(Some(Path::new("Justfile")), source, &theme)
            .expect("expected Justfile source to render");
        let lines: Vec<_> = rendered.lines().collect();

        assert!(
            !line_has_background(lines[0], tint),
            "recipe header line should not be part of the nested block"
        );
        assert!(
            line_has_background(lines[1], tint),
            "first recipe command line should receive nested block tint"
        );
        assert!(
            line_has_background(lines[2], tint),
            "second recipe command line should receive nested block tint"
        );

        let source_lines: Vec<_> = source.lines().collect();
        let block_width = source_lines[1..]
            .iter()
            .map(|line| line.len())
            .max()
            .expect("expected recipe body lines");
        let short_pad = " ".repeat(block_width - source_lines[1].len());

        assert!(
            trailing_background_pad_width(lines[1], tint) == short_pad.len(),
            "shorter recipe body line should be padded to the block width"
        );
        assert!(
            trailing_background_pad_width(lines[2], tint) == 0,
            "widest recipe body line should not receive trailing block padding"
        );
    }

    #[test]
    fn markdown_fenced_code_uses_block_region_tint_only_inside_fence_body() {
        let tint = RgbColor(1, 2, 3);
        let theme = Theme::for_mode_with_nested_region_tint(ColorMode::TrueColor, Some(tint));
        let path = fixture_path("markdown/go_fence.md");
        let source = read_file(&path);
        let rendered = render_with_theme(Some(path.as_path()), &source, &theme)
            .unwrap_or_else(|error| panic!("failed to render {}: {error}", path.display()));
        let lines: Vec<_> = rendered.lines().collect();
        assert!(
            line_has_background(find_line_containing(&lines, "package preview"), tint),
            "fenced code content should receive nested block tint"
        );

        let short_line = "package preview";
        let wide_line = "func (r *Renderer) Render() string {";
        let short_pad = " ".repeat(wide_line.len() - short_line.len());

        assert!(
            trailing_background_pad_width(find_line_containing(&lines, short_line), tint)
                == short_pad.len(),
            "shorter fenced code line should be padded to the block width"
        );
        assert!(
            trailing_background_pad_width(find_line_containing(&lines, wide_line), tint) == 0,
            "widest fenced code line should not receive trailing block padding"
        );
    }

    #[test]
    fn github_actions_run_block_uses_block_region_tint_without_tinting_run_header() {
        let source = "jobs:\n  build:\n    steps:\n      - run: |\n          echo hi\n          printf '%s\\n' \"$GITHUB_REF\"\n";
        let tint = RgbColor(1, 2, 3);
        let theme = Theme::for_mode_with_nested_region_tint(ColorMode::TrueColor, Some(tint));
        let rendered = render_with_theme(
            Some(Path::new(".github/workflows/demo.yml")),
            source,
            &theme,
        )
        .expect("expected workflow source to render");
        let lines: Vec<_> = rendered.lines().collect();
        assert!(
            !line_has_background(lines[3], tint),
            "run: | header line should stay outside the nested block"
        );
        assert!(
            line_has_background(lines[4], tint),
            "first run body line should receive nested block tint"
        );
        assert!(
            line_has_background(lines[5], tint),
            "second run body line should receive nested block tint"
        );

        let source_lines: Vec<_> = source.lines().collect();
        let body_lines = [source_lines[4], source_lines[5]];
        let block_width = body_lines.iter().map(|line| line.len()).max().unwrap_or(0);
        let short_pad = " ".repeat(block_width - body_lines[0].len());

        assert!(
            trailing_background_pad_width(lines[4], tint) == short_pad.len(),
            "shorter run body line should be padded to the block width"
        );
        assert!(
            trailing_background_pad_width(lines[5], tint) == 0,
            "widest run body line should not receive trailing block padding"
        );
    }

    fn line_has_background(line: &str, background: RgbColor) -> bool {
        line.contains(&background_escape(background))
    }

    fn find_line_containing<'a>(lines: &'a [&str], needle: &str) -> &'a str {
        lines.iter()
            .copied()
            .find(|line| strip_ansi(line).contains(needle))
            .unwrap_or_else(|| panic!("expected rendered output to contain line fragment {needle:?}"))
    }

    fn background_escape(background: RgbColor) -> String {
        format!("\x1b[48;2;{};{};{}m", background.0, background.1, background.2)
    }

    fn strip_ansi(text: &str) -> String {
        let bytes = text.as_bytes();
        let mut stripped = String::with_capacity(text.len());
        let mut index = 0;

        while index < bytes.len() {
            if bytes[index] == 0x1b && bytes.get(index + 1) == Some(&b'[') {
                index += 2;
                while index < bytes.len() {
                    let byte = bytes[index];
                    index += 1;
                    if (0x40..=0x7e).contains(&byte) {
                        break;
                    }
                }
                continue;
            }

            let ch = text[index..].chars().next().unwrap_or_default();
            stripped.push(ch);
            index += ch.len_utf8();
        }

        stripped
    }

    fn trailing_background_pad_width(line: &str, background: RgbColor) -> usize {
        let reset = "\x1b[0m";
        let Some(prefix) = line.strip_suffix(reset) else {
            return 0;
        };
        let background_escape = background_escape(background);
        let Some(background_start) = prefix.rfind(&background_escape) else {
            return 0;
        };
        let pad = &prefix[(background_start + background_escape.len())..];
        if pad.chars().all(|ch| ch == ' ') {
            pad.len()
        } else {
            0
        }
    }

    fn fixture_path(relative_path: &str) -> PathBuf {
        Path::new("testdata/fixtures").join(relative_path)
    }

    fn read_file(path: &Path) -> String {
        fs::read_to_string(path)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
    }

    fn collect_files(root: &str) -> Vec<PathBuf> {
        let mut files = Vec::new();
        collect_files_recursive(Path::new(root), &mut files);
        files.sort();
        files
    }

    fn collect_files_recursive(root: &Path, files: &mut Vec<PathBuf>) {
        for entry in fs::read_dir(root)
            .unwrap_or_else(|error| panic!("failed to read directory {}: {error}", root.display()))
        {
            let entry = entry.unwrap_or_else(|error| {
                panic!("failed to read entry in {}: {error}", root.display())
            });
            let path = entry.path();

            if path.is_dir() {
                collect_files_recursive(&path, files);
            } else {
                files.push(path);
            }
        }
    }

    fn is_supported_highlight_path(path: &Path) -> bool {
        detect_language(Some(path), "").is_some()
    }
}
fn decode_javascript_string_literal(
    source: &str,
    range: Range<usize>,
    virtual_source: &mut String,
    source_map: &mut Vec<Range<usize>>,
) {
    let slice = &source[range.clone()];
    if slice.len() >= 2
        && matches!(slice.as_bytes().first(), Some(b'\'') | Some(b'"'))
        && slice.as_bytes().first() == slice.as_bytes().last()
    {
        decode_javascript_string_content(
            source,
            (range.start + 1)..(range.end - 1),
            virtual_source,
            source_map,
        );
    } else {
        append_raw_range(source, range, virtual_source, source_map);
    }
}
