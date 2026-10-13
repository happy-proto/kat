use crate::analysis::AnalysisDocument;
use crate::debug_progress::log as progress_log;
use crate::detection::detect_document_kind;
use crate::document::PreparedDocument;
use crate::document::RenderOptions;
use crate::document::detected_theme;
use crate::document_kind::DocumentKind;
use crate::document_kind::named_document_kind;
use crate::language_aliases::normalize_language_name;
use crate::language_runtime::supports_runtime;
use crate::layout::LayoutDocument;
use crate::render_data::StyledSpan;
use crate::render_data::VisualRegion;
use crate::render_ops::RenderPlan;
use crate::semantic_overlays::debug_named_language_tree as debug_language_tree_impl;
use crate::semantic_overlays::debug_semantics as debug_semantics_impl;
use crate::terminal;
use crate::terminal::TerminalCapabilities;
use crate::theme::ColorMode;
use crate::theme::Theme;
use crate::timing::RenderOutput;
use crate::timing::RenderTimings;
use crate::visual::VisualDocument;
use anyhow::Result;
use anyhow::anyhow;
use serde::Serialize;
use std::path::Path;
use std::time::Instant;

pub fn render(source_path: Option<&Path>, source: &str) -> Result<String> {
    Ok(render_with_timing(source_path, source)?.output)
}

pub fn render_named_language(language_name: &str, source: &str) -> Result<String> {
    Ok(render_named_language_with_timing(language_name, source)?.output)
}

pub fn render_with_timing(source_path: Option<&Path>, source: &str) -> Result<RenderOutput> {
    render_with_timing_and_terminal_width(source_path, source, None)
}

pub fn render_named_language_with_timing(
    language_name: &str,
    source: &str,
) -> Result<RenderOutput> {
    render_named_language_with_timing_and_terminal_width(language_name, source, None)
}

pub fn render_with_timing_and_terminal_width(
    source_path: Option<&Path>,
    source: &str,
    terminal_width: Option<usize>,
) -> Result<RenderOutput> {
    render_with_timing_and_terminal_options(
        source_path,
        source,
        terminal_width,
        /*hyperlinks_enabled*/ false,
    )
}

pub fn render_with_timing_and_terminal_options(
    source_path: Option<&Path>,
    source: &str,
    terminal_width: Option<usize>,
    hyperlinks_enabled: bool,
) -> Result<RenderOutput> {
    let rendered = PreparedDocument::detect(
        source_path,
        source,
        RenderOptions::terminal(hyperlinks_enabled),
    )?
    .render(terminal_width);
    Ok(RenderOutput {
        output: rendered.output,
        timings: rendered.timings,
    })
}

pub fn render_named_language_with_timing_and_terminal_width(
    language_name: &str,
    source: &str,
    terminal_width: Option<usize>,
) -> Result<RenderOutput> {
    render_named_language_with_timing_and_terminal_options(
        language_name,
        source,
        terminal_width,
        /*hyperlinks_enabled*/ false,
    )
}

pub fn render_named_language_with_timing_and_terminal_options(
    language_name: &str,
    source: &str,
    terminal_width: Option<usize>,
    hyperlinks_enabled: bool,
) -> Result<RenderOutput> {
    let rendered = PreparedDocument::named_language(
        language_name,
        source,
        RenderOptions::terminal(hyperlinks_enabled),
    )?
    .render(terminal_width);
    Ok(RenderOutput {
        output: rendered.output,
        timings: rendered.timings,
    })
}

pub fn strip_terminal_hyperlinks(text: &str) -> String {
    terminal::strip_osc8_hyperlinks(text)
}

pub fn detected_language_name(source_path: Option<&Path>, source: &str) -> Option<&'static str> {
    detect_document_kind(source_path, source).map(DocumentKind::runtime_name)
}

pub(crate) fn analyze_with_theme(
    source_path: Option<&Path>,
    source: &str,
    theme: &Theme,
) -> Result<AnalysisDocument> {
    AnalysisDocument::detect(source_path, source, *theme, None)
}

pub(crate) fn analyze_named_language_with_theme(
    language_name: &str,
    source: &str,
    theme: &Theme,
) -> Result<AnalysisDocument> {
    AnalysisDocument::named_language(language_name, source, *theme, None)
}

pub(crate) fn visual_with_theme(
    source_path: Option<&Path>,
    source: &str,
    theme: &Theme,
) -> Result<VisualDocument> {
    let analysis = analyze_with_theme(source_path, source, theme)?;
    Ok(VisualDocument::from_analysis(&analysis))
}

pub(crate) fn visual_named_language_with_theme(
    language_name: &str,
    source: &str,
    theme: &Theme,
) -> Result<VisualDocument> {
    let analysis = analyze_named_language_with_theme(language_name, source, theme)?;
    Ok(VisualDocument::from_analysis(&analysis))
}

pub(crate) fn layout_with_theme(
    source_path: Option<&Path>,
    source: &str,
    theme: &Theme,
    terminal_width: Option<usize>,
) -> Result<LayoutDocument> {
    let visual = visual_with_theme(source_path, source, theme)?;
    Ok(LayoutDocument::from_visual(
        source,
        visual.spans(),
        visual.regions(),
        *theme,
        terminal_width,
    ))
}

pub(crate) fn layout_named_language_with_theme(
    language_name: &str,
    source: &str,
    theme: &Theme,
    terminal_width: Option<usize>,
) -> Result<LayoutDocument> {
    let visual = visual_named_language_with_theme(language_name, source, theme)?;
    Ok(LayoutDocument::from_visual(
        source,
        visual.spans(),
        visual.regions(),
        *theme,
        terminal_width,
    ))
}

pub(crate) fn render_plan_with_theme(
    source_path: Option<&Path>,
    source: &str,
    theme: &Theme,
) -> Result<RenderPlan> {
    let layout = layout_with_theme(source_path, source, theme, None)?;
    Ok(RenderPlan::compile(&layout, *theme))
}

pub(crate) fn render_plan_named_language_with_theme(
    language_name: &str,
    source: &str,
    theme: &Theme,
) -> Result<RenderPlan> {
    let layout = layout_named_language_with_theme(language_name, source, theme, None)?;
    Ok(RenderPlan::compile(&layout, *theme))
}

pub fn debug_analysis_json(source_path: Option<&Path>, source: &str) -> Result<String> {
    let analysis = analyze_with_theme(source_path, source, &debug_theme())?;
    to_pretty_json(&analysis.snapshot())
}

pub fn debug_named_language_analysis_json(language_name: &str, source: &str) -> Result<String> {
    let analysis = analyze_named_language_with_theme(language_name, source, &debug_theme())?;
    to_pretty_json(&analysis.snapshot())
}

pub fn debug_visual_json(source_path: Option<&Path>, source: &str) -> Result<String> {
    let visual = visual_with_theme(source_path, source, &debug_theme())?;
    to_pretty_json(&visual.snapshot())
}

pub fn debug_named_language_visual_json(language_name: &str, source: &str) -> Result<String> {
    let visual = visual_named_language_with_theme(language_name, source, &debug_theme())?;
    to_pretty_json(&visual.snapshot())
}

pub fn debug_layout_json(source_path: Option<&Path>, source: &str) -> Result<String> {
    let layout = layout_with_theme(source_path, source, &debug_theme(), None)?;
    to_pretty_json(&layout.snapshot())
}

pub fn debug_named_language_layout_json(language_name: &str, source: &str) -> Result<String> {
    let layout = layout_named_language_with_theme(language_name, source, &debug_theme(), None)?;
    to_pretty_json(&layout.snapshot())
}

pub fn debug_render_ops_json(source_path: Option<&Path>, source: &str) -> Result<String> {
    let theme = debug_theme();
    let plan = render_plan_with_theme(source_path, source, &theme)?;
    to_pretty_json(&plan.snapshot(theme))
}

pub fn debug_named_language_render_ops_json(language_name: &str, source: &str) -> Result<String> {
    let theme = debug_theme();
    let plan = render_plan_named_language_with_theme(language_name, source, &theme)?;
    to_pretty_json(&plan.snapshot(theme))
}

pub fn debug_terminal_json(source_path: Option<&Path>, source: &str) -> Result<String> {
    let capabilities = TerminalCapabilities::for_debug_layers();
    let theme = capabilities.theme();
    let plan = render_plan_with_theme(source_path, source, &theme)?;
    let output = plan.encode(theme, /*hyperlinks_enabled*/ false);
    to_pretty_json(&terminal::TerminalRenderSnapshot {
        capabilities: capabilities.snapshot(),
        render_plan: plan.snapshot(theme),
        encoded_output: terminal::escape_control_sequences(&output),
        encoded_bytes: output.len(),
    })
}

pub fn debug_named_language_terminal_json(language_name: &str, source: &str) -> Result<String> {
    let capabilities = TerminalCapabilities::for_debug_layers();
    let theme = capabilities.theme();
    let plan = render_plan_named_language_with_theme(language_name, source, &theme)?;
    let output = plan.encode(theme, /*hyperlinks_enabled*/ false);
    to_pretty_json(&terminal::TerminalRenderSnapshot {
        capabilities: capabilities.snapshot(),
        render_plan: plan.snapshot(theme),
        encoded_output: terminal::escape_control_sequences(&output),
        encoded_bytes: output.len(),
    })
}

pub(crate) fn debug_theme() -> Theme {
    Theme::new(ColorMode::TrueColor, Some(anstyle::RgbColor(16, 24, 32)))
}

pub(crate) fn to_pretty_json<T: Serialize>(value: &T) -> Result<String> {
    Ok(serde_json::to_string_pretty(value)?)
}

macro_rules! highlight_languages {
    ($($function:ident => $name:literal),* $(,)?) => {
        $(pub fn $function(source: &str) -> Result<String> {
            highlight_named_language($name, source, &detected_theme())
        })*
    };
}
highlight_languages! {
    highlight_json => "json",
    highlight_query => "query",
    highlight_ignore => "ignore",
    highlight_git_link => "git_link",
    highlight_git_mailmap => "git_mailmap",
    highlight_git_log => "git_log",
    highlight_git_config => "git_config",
    highlight_bash => "bash",
    highlight_batch => "batch",
    highlight_dockerfile => "dockerfile",
    highlight_toml => "toml",
    highlight_fish => "fish",
    highlight_zsh => "zsh",
    highlight_powershell => "powershell",
    highlight_yaml => "yaml",
    highlight_hcl => "hcl",
    highlight_rust => "rust",
    highlight_just => "just",
    highlight_python => "python",
    highlight_c => "c",
    highlight_cpp => "cpp",
    highlight_java => "java",
    highlight_kotlin => "kotlin",
    highlight_csharp => "csharp",
    highlight_groovy => "groovy",
    highlight_diff => "diff",
    highlight_properties => "properties",
    highlight_php => "php",
    highlight_scala => "scala",
    highlight_swift => "swift",
    highlight_dart => "dart",
    highlight_elixir => "elixir",
    highlight_zig => "zig",
    highlight_ssh_config => "ssh_config",
    highlight_gitattributes => "gitattributes",
    highlight_git_commit => "git_commit",
    highlight_git_rebase => "git_rebase",
    highlight_requirements => "requirements",
    highlight_apache => "apache",
    highlight_scss => "scss",
    highlight_sass => "sass",
    highlight_jq => "jq",
    highlight_less => "less",
    highlight_dot => "dot",
    highlight_nginx => "nginx",
    highlight_typescript => "typescript",
    highlight_tsx => "tsx",
    highlight_go => "go",
    highlight_gomod => "gomod",
    highlight_gowork => "gowork",
    highlight_gosum => "gosum",
    highlight_sql => "sql",
    highlight_html => "html",
    highlight_vue => "vue",
    highlight_svelte => "svelte",
    highlight_css => "css",
    highlight_graphql => "graphql",
    highlight_proto => "proto",
    highlight_cel => "cel",
    highlight_textproto => "textproto",
    highlight_latex => "latex",
    highlight_tcl => "tcl",
    highlight_textile => "textile",
    highlight_csv => "csv",
    highlight_todotxt => "todotxt",
    highlight_tsv => "tsv",
    highlight_typst => "typst",
    highlight_strace => "strace",
    highlight_stylus => "stylus",
    highlight_syslog => "syslog",
    highlight_sml => "sml",
    highlight_solidity => "solidity",
    highlight_varlink => "varlink",
    highlight_vyper => "vyper",
    highlight_wgsl => "wgsl",
    highlight_javascript => "javascript",
    highlight_markdown => "markdown",
    highlight_ruby => "ruby",
    highlight_lua => "lua",
    highlight_nix => "nix",
    highlight_dotenv => "dotenv",
    highlight_ini => "ini",
    highlight_xml => "xml",
    highlight_make => "make",
    highlight_cmake => "cmake",
    highlight_ninja => "ninja",
    highlight_jinja => "jinja",
    highlight_eex => "eex",
    highlight_twig => "twig",
    highlight_erb => "erb",
    highlight_jsp => "jsp",
    highlight_asp => "asp",
    highlight_adp => "adp",
    highlight_vhdl => "vhdl",
    highlight_verilog => "verilog",
    highlight_systemverilog => "systemverilog",
    highlight_vim => "vim",
    highlight_vimhelp => "vimhelp",
    highlight_nasm => "nasm",
}

pub fn debug_named_language_tree(language_name: &str, source: &str) -> Result<String> {
    debug_language_tree_impl(canonical_language_name(language_name)?, source)
}

pub fn debug_semantics(language_name: &str, source: &str) -> Result<String> {
    debug_semantics_impl(canonical_language_name(language_name)?, source)
}

pub fn debug_shell_semantics(language_name: &str, source: &str) -> Result<String> {
    debug_semantics(language_name, source)
}

pub(crate) fn highlight_named_language(
    language_name: &str,
    source: &str,
    theme: &Theme,
) -> Result<String> {
    highlight_document_kind(
        named_document_kind(language_name)?,
        None,
        source,
        theme,
        None,
    )
}

pub fn canonical_language_name(language_name: &str) -> Result<&str> {
    let normalized = normalize_language_name(language_name)
        .ok_or_else(|| anyhow!("unknown language override '{language_name}'"))?;
    if supports_runtime(normalized) {
        Ok(normalized)
    } else {
        Err(anyhow!("unknown language override '{language_name}'"))
    }
}

pub(crate) fn highlight_document_kind(
    document_kind: DocumentKind,
    source_path: Option<&Path>,
    source: &str,
    theme: &Theme,
    timings: Option<&mut RenderTimings>,
) -> Result<String> {
    highlight_named_language_with_path(document_kind, source_path, source, theme, timings)
}

pub(crate) fn highlight_named_language_with_path(
    document_kind: DocumentKind,
    source_path: Option<&Path>,
    source: &str,
    theme: &Theme,
    mut timings: Option<&mut RenderTimings>,
) -> Result<String> {
    let analysis = AnalysisDocument::document_kind(
        document_kind,
        source_path,
        source,
        *theme,
        timings.as_deref_mut(),
    )?;
    let visual = VisualDocument::from_analysis(&analysis);
    let render_started_at = Instant::now();
    let output = render_styled_spans(source, visual.spans(), visual.regions(), theme);
    if let Some(timings) = timings {
        timings.record_render_styled_spans(render_started_at);
    }
    Ok(output)
}

pub(crate) fn render_styled_spans(
    source: &str,
    spans: &[StyledSpan],
    regions: &[VisualRegion],
    theme: &Theme,
) -> String {
    let total_lines = source.bytes().filter(|byte| *byte == b'\n').count() + 1;
    progress_log(
        "render",
        format!(
            "begin lines={} spans={} regions={}",
            total_lines,
            spans.len(),
            regions.len(),
        ),
    );
    let layout = LayoutDocument::from_visual(source, spans, regions, *theme, None);
    let rendered =
        RenderPlan::compile(&layout, *theme).encode(*theme, /*hyperlinks_enabled*/ false);
    progress_log(
        "render",
        format!("done lines={} output_bytes={}", total_lines, rendered.len()),
    );
    rendered
}
