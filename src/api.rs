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

pub fn highlight_json(source: &str) -> Result<String> {
    highlight_named_language("json", source, &detected_theme())
}

pub fn highlight_query(source: &str) -> Result<String> {
    highlight_named_language("query", source, &detected_theme())
}

pub fn highlight_ignore(source: &str) -> Result<String> {
    highlight_named_language("ignore", source, &detected_theme())
}

pub fn highlight_git_link(source: &str) -> Result<String> {
    highlight_named_language("git_link", source, &detected_theme())
}

pub fn highlight_git_mailmap(source: &str) -> Result<String> {
    highlight_named_language("git_mailmap", source, &detected_theme())
}

pub fn highlight_git_log(source: &str) -> Result<String> {
    highlight_named_language("git_log", source, &detected_theme())
}

pub fn highlight_git_config(source: &str) -> Result<String> {
    highlight_named_language("git_config", source, &detected_theme())
}

pub fn highlight_bash(source: &str) -> Result<String> {
    highlight_named_language("bash", source, &detected_theme())
}

pub fn highlight_batch(source: &str) -> Result<String> {
    highlight_named_language("batch", source, &detected_theme())
}

pub fn highlight_dockerfile(source: &str) -> Result<String> {
    highlight_named_language("dockerfile", source, &detected_theme())
}

pub fn highlight_toml(source: &str) -> Result<String> {
    highlight_named_language("toml", source, &detected_theme())
}

pub fn highlight_fish(source: &str) -> Result<String> {
    highlight_named_language("fish", source, &detected_theme())
}

pub fn highlight_zsh(source: &str) -> Result<String> {
    highlight_named_language("zsh", source, &detected_theme())
}

pub fn highlight_powershell(source: &str) -> Result<String> {
    highlight_named_language("powershell", source, &detected_theme())
}

pub fn highlight_yaml(source: &str) -> Result<String> {
    highlight_named_language("yaml", source, &detected_theme())
}

pub fn highlight_hcl(source: &str) -> Result<String> {
    highlight_named_language("hcl", source, &detected_theme())
}

pub fn highlight_rust(source: &str) -> Result<String> {
    highlight_named_language("rust", source, &detected_theme())
}

pub fn highlight_just(source: &str) -> Result<String> {
    highlight_named_language("just", source, &detected_theme())
}

pub fn highlight_python(source: &str) -> Result<String> {
    highlight_named_language("python", source, &detected_theme())
}

pub fn highlight_c(source: &str) -> Result<String> {
    highlight_named_language("c", source, &detected_theme())
}

pub fn highlight_cpp(source: &str) -> Result<String> {
    highlight_named_language("cpp", source, &detected_theme())
}

pub fn highlight_java(source: &str) -> Result<String> {
    highlight_named_language("java", source, &detected_theme())
}

pub fn highlight_kotlin(source: &str) -> Result<String> {
    highlight_named_language("kotlin", source, &detected_theme())
}

pub fn highlight_csharp(source: &str) -> Result<String> {
    highlight_named_language("csharp", source, &detected_theme())
}

pub fn highlight_groovy(source: &str) -> Result<String> {
    highlight_named_language("groovy", source, &detected_theme())
}

pub fn highlight_diff(source: &str) -> Result<String> {
    highlight_named_language("diff", source, &detected_theme())
}

pub fn highlight_properties(source: &str) -> Result<String> {
    highlight_named_language("properties", source, &detected_theme())
}

pub fn highlight_php(source: &str) -> Result<String> {
    highlight_named_language("php", source, &detected_theme())
}

pub fn highlight_scala(source: &str) -> Result<String> {
    highlight_named_language("scala", source, &detected_theme())
}

pub fn highlight_swift(source: &str) -> Result<String> {
    highlight_named_language("swift", source, &detected_theme())
}

pub fn highlight_dart(source: &str) -> Result<String> {
    highlight_named_language("dart", source, &detected_theme())
}

pub fn highlight_elixir(source: &str) -> Result<String> {
    highlight_named_language("elixir", source, &detected_theme())
}

pub fn highlight_zig(source: &str) -> Result<String> {
    highlight_named_language("zig", source, &detected_theme())
}

pub fn highlight_ssh_config(source: &str) -> Result<String> {
    highlight_named_language("ssh_config", source, &detected_theme())
}

pub fn highlight_gitattributes(source: &str) -> Result<String> {
    highlight_named_language("gitattributes", source, &detected_theme())
}

pub fn highlight_git_commit(source: &str) -> Result<String> {
    highlight_named_language("git_commit", source, &detected_theme())
}

pub fn highlight_git_rebase(source: &str) -> Result<String> {
    highlight_named_language("git_rebase", source, &detected_theme())
}

pub fn highlight_requirements(source: &str) -> Result<String> {
    highlight_named_language("requirements", source, &detected_theme())
}

pub fn highlight_apache(source: &str) -> Result<String> {
    highlight_named_language("apache", source, &detected_theme())
}

pub fn highlight_scss(source: &str) -> Result<String> {
    highlight_named_language("scss", source, &detected_theme())
}

pub fn highlight_sass(source: &str) -> Result<String> {
    highlight_named_language("sass", source, &detected_theme())
}

pub fn highlight_jq(source: &str) -> Result<String> {
    highlight_named_language("jq", source, &detected_theme())
}

pub fn highlight_less(source: &str) -> Result<String> {
    highlight_named_language("less", source, &detected_theme())
}

pub fn highlight_dot(source: &str) -> Result<String> {
    highlight_named_language("dot", source, &detected_theme())
}

pub fn highlight_nginx(source: &str) -> Result<String> {
    highlight_named_language("nginx", source, &detected_theme())
}

pub fn highlight_typescript(source: &str) -> Result<String> {
    highlight_named_language("typescript", source, &detected_theme())
}

pub fn highlight_tsx(source: &str) -> Result<String> {
    highlight_named_language("tsx", source, &detected_theme())
}

pub fn highlight_go(source: &str) -> Result<String> {
    highlight_named_language("go", source, &detected_theme())
}

pub fn highlight_gomod(source: &str) -> Result<String> {
    highlight_named_language("gomod", source, &detected_theme())
}

pub fn highlight_gowork(source: &str) -> Result<String> {
    highlight_named_language("gowork", source, &detected_theme())
}

pub fn highlight_gosum(source: &str) -> Result<String> {
    highlight_named_language("gosum", source, &detected_theme())
}

pub fn highlight_sql(source: &str) -> Result<String> {
    highlight_named_language("sql", source, &detected_theme())
}

pub fn highlight_html(source: &str) -> Result<String> {
    highlight_named_language("html", source, &detected_theme())
}

pub fn highlight_vue(source: &str) -> Result<String> {
    highlight_named_language("vue", source, &detected_theme())
}

pub fn highlight_svelte(source: &str) -> Result<String> {
    highlight_named_language("svelte", source, &detected_theme())
}

pub fn highlight_css(source: &str) -> Result<String> {
    highlight_named_language("css", source, &detected_theme())
}

pub fn highlight_graphql(source: &str) -> Result<String> {
    highlight_named_language("graphql", source, &detected_theme())
}

pub fn highlight_proto(source: &str) -> Result<String> {
    highlight_named_language("proto", source, &detected_theme())
}

pub fn highlight_cel(source: &str) -> Result<String> {
    highlight_named_language("cel", source, &detected_theme())
}

pub fn highlight_textproto(source: &str) -> Result<String> {
    highlight_named_language("textproto", source, &detected_theme())
}

pub fn highlight_latex(source: &str) -> Result<String> {
    highlight_named_language("latex", source, &detected_theme())
}

pub fn highlight_tcl(source: &str) -> Result<String> {
    highlight_named_language("tcl", source, &detected_theme())
}

pub fn highlight_textile(source: &str) -> Result<String> {
    highlight_named_language("textile", source, &detected_theme())
}

pub fn highlight_csv(source: &str) -> Result<String> {
    highlight_named_language("csv", source, &detected_theme())
}

pub fn highlight_todotxt(source: &str) -> Result<String> {
    highlight_named_language("todotxt", source, &detected_theme())
}

pub fn highlight_tsv(source: &str) -> Result<String> {
    highlight_named_language("tsv", source, &detected_theme())
}

pub fn highlight_typst(source: &str) -> Result<String> {
    highlight_named_language("typst", source, &detected_theme())
}

pub fn highlight_strace(source: &str) -> Result<String> {
    highlight_named_language("strace", source, &detected_theme())
}

pub fn highlight_stylus(source: &str) -> Result<String> {
    highlight_named_language("stylus", source, &detected_theme())
}

pub fn highlight_syslog(source: &str) -> Result<String> {
    highlight_named_language("syslog", source, &detected_theme())
}

pub fn highlight_sml(source: &str) -> Result<String> {
    highlight_named_language("sml", source, &detected_theme())
}

pub fn highlight_solidity(source: &str) -> Result<String> {
    highlight_named_language("solidity", source, &detected_theme())
}

pub fn highlight_varlink(source: &str) -> Result<String> {
    highlight_named_language("varlink", source, &detected_theme())
}

pub fn highlight_vyper(source: &str) -> Result<String> {
    highlight_named_language("vyper", source, &detected_theme())
}

pub fn highlight_wgsl(source: &str) -> Result<String> {
    highlight_named_language("wgsl", source, &detected_theme())
}

pub fn highlight_javascript(source: &str) -> Result<String> {
    highlight_named_language("javascript", source, &detected_theme())
}

pub fn highlight_markdown(source: &str) -> Result<String> {
    highlight_named_language("markdown", source, &detected_theme())
}

pub fn highlight_ruby(source: &str) -> Result<String> {
    highlight_named_language("ruby", source, &detected_theme())
}

pub fn highlight_lua(source: &str) -> Result<String> {
    highlight_named_language("lua", source, &detected_theme())
}

pub fn highlight_nix(source: &str) -> Result<String> {
    highlight_named_language("nix", source, &detected_theme())
}

pub fn highlight_dotenv(source: &str) -> Result<String> {
    highlight_named_language("dotenv", source, &detected_theme())
}

pub fn highlight_ini(source: &str) -> Result<String> {
    highlight_named_language("ini", source, &detected_theme())
}

pub fn highlight_xml(source: &str) -> Result<String> {
    highlight_named_language("xml", source, &detected_theme())
}

pub fn highlight_make(source: &str) -> Result<String> {
    highlight_named_language("make", source, &detected_theme())
}

pub fn highlight_cmake(source: &str) -> Result<String> {
    highlight_named_language("cmake", source, &detected_theme())
}

pub fn highlight_ninja(source: &str) -> Result<String> {
    highlight_named_language("ninja", source, &detected_theme())
}

pub fn highlight_jinja(source: &str) -> Result<String> {
    highlight_named_language("jinja", source, &detected_theme())
}

pub fn highlight_eex(source: &str) -> Result<String> {
    highlight_named_language("eex", source, &detected_theme())
}

pub fn highlight_twig(source: &str) -> Result<String> {
    highlight_named_language("twig", source, &detected_theme())
}

pub fn highlight_erb(source: &str) -> Result<String> {
    highlight_named_language("erb", source, &detected_theme())
}

pub fn highlight_jsp(source: &str) -> Result<String> {
    highlight_named_language("jsp", source, &detected_theme())
}

pub fn highlight_asp(source: &str) -> Result<String> {
    highlight_named_language("asp", source, &detected_theme())
}

pub fn highlight_adp(source: &str) -> Result<String> {
    highlight_named_language("adp", source, &detected_theme())
}

pub fn highlight_vhdl(source: &str) -> Result<String> {
    highlight_named_language("vhdl", source, &detected_theme())
}

pub fn highlight_verilog(source: &str) -> Result<String> {
    highlight_named_language("verilog", source, &detected_theme())
}

pub fn highlight_systemverilog(source: &str) -> Result<String> {
    highlight_named_language("systemverilog", source, &detected_theme())
}

pub fn highlight_vim(source: &str) -> Result<String> {
    highlight_named_language("vim", source, &detected_theme())
}

pub fn highlight_vimhelp(source: &str) -> Result<String> {
    highlight_named_language("vimhelp", source, &detected_theme())
}

pub fn highlight_nasm(source: &str) -> Result<String> {
    highlight_named_language("nasm", source, &detected_theme())
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
