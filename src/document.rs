use crate::analysis::AnalysisDocument;
use crate::layout::LayoutDocument;
use crate::render_ops::RenderPlan;
use crate::source_highlight::HighlightBackgroundRun;
use crate::source_highlight::HighlightSpan;
use crate::terminal::TerminalCapabilities;
use crate::terminal_background;
use crate::theme::ColorMode;
use crate::theme::Theme;
use crate::timing::RenderTimings;
use crate::visual::VisualDocument;
use anyhow::Result;
use std::path::Path;
use std::time::Instant;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
/// Stable source position represented by one rendered terminal row.
pub struct RenderedRowAnchor {
    /// Byte offset where the original source line starts.
    pub source_line_start: usize,
    /// Display column where this row starts after source-aware wrapping.
    pub wrapped_from_column: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// Terminal output and source positions produced for one viewport width.
pub struct RenderedDocument {
    /// ANSI-encoded terminal output.
    pub output: String,
    /// Source position for each rendered row in `output`.
    pub row_anchors: Vec<RenderedRowAnchor>,
    /// Measurements collected while preparing and rendering the document.
    pub timings: RenderTimings,
}

/// One display row produced by layout. Encoding never changes its geometry.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DisplayRow {
    pub text: String,
    pub display_width: usize,
    pub anchor: RenderedRowAnchor,
    plan: RenderPlan,
    theme: Theme,
    hyperlinks_enabled: bool,
}

impl DisplayRow {
    /// Highlight sorted, non-overlapping UTF-8 ranges in this row without parsing ANSI.
    /// Range endpoints must lie on text character boundaries.
    pub fn encode(
        &self,
        matches: &[std::ops::Range<usize>],
        current: Option<&std::ops::Range<usize>>,
    ) -> String {
        self.plan
            .encode_marked(self.theme, self.hyperlinks_enabled, matches, current)
    }
}

#[derive(Clone, Debug)]
/// Source-backed render state that can be laid out repeatedly at new widths.
pub struct PreparedDocument {
    source: String,
    visual: Option<VisualDocument>,
    theme: Theme,
    hyperlinks_enabled: bool,
    timings: RenderTimings,
}

/// Terminal presentation chosen by the caller before preparing a document.
#[derive(Clone, Copy, Debug)]
pub struct RenderOptions {
    theme: Theme,
    hyperlinks_enabled: bool,
}

impl RenderOptions {
    /// Probe the terminal palette for the kat CLI's rendering behavior.
    pub fn terminal(hyperlinks_enabled: bool) -> Self {
        Self {
            theme: detected_theme(),
            hyperlinks_enabled,
        }
    }

    /// Use explicit colors in an embedding viewer, without terminal queries.
    pub fn with_colors(foreground: (u8, u8, u8), background: (u8, u8, u8)) -> Self {
        use anstyle::RgbColor;
        let to_rgb = |(r, g, b)| RgbColor(r, g, b);
        let tint =
            terminal_background::derive_nested_region_tint(to_rgb(foreground), to_rgb(background));
        Self {
            theme: Theme::new(ColorMode::TrueColor, Some(tint)),
            hyperlinks_enabled: false,
        }
    }
}

impl PreparedDocument {
    /// Prepare unstyled text with the same display geometry as syntax-highlighted text.
    pub fn plain(source: &str) -> Self {
        Self {
            source: source.to_owned(),
            visual: None,
            theme: Theme::new(ColorMode::NoColor, None),
            hyperlinks_enabled: false,
            timings: RenderTimings::default(),
        }
    }

    pub fn source(&self) -> &str {
        &self.source
    }

    /// Layout is the sole owner of wrapping, widths and source anchors.
    pub fn display_rows(&self, terminal_width: usize) -> Vec<DisplayRow> {
        let (spans, regions) = self
            .visual
            .as_ref()
            .map(|v| (v.spans(), v.regions()))
            .unwrap_or((&[], &[]));
        let layout = LayoutDocument::from_visual(
            &self.source,
            spans,
            regions,
            self.theme,
            Some(terminal_width.max(1)),
        );
        let mut rows = layout
            .rows()
            .iter()
            .map(|row| {
                let plan = RenderPlan::compile_row(row, self.theme);
                DisplayRow {
                    text: plan.text(),
                    display_width: row.display_width,
                    anchor: RenderedRowAnchor {
                        source_line_start: row.source_line_start,
                        wrapped_from_column: row.wrapped_from_column,
                    },
                    plan,
                    theme: self.theme,
                    hyperlinks_enabled: self.hyperlinks_enabled,
                }
            })
            .collect::<Vec<_>>();
        // A final newline terminates the last source line; it is not another blank line.
        if self.source.ends_with('\n') {
            rows.pop();
        }
        rows
    }

    /// Detects the document kind and prepares width-independent visual data.
    pub fn detect(
        source_path: Option<&Path>,
        source: &str,
        options: RenderOptions,
    ) -> Result<Self> {
        let theme = options.theme;
        let mut timings = RenderTimings::default();
        let analysis = AnalysisDocument::detect(source_path, source, theme, Some(&mut timings))?;
        Ok(Self::from_analysis(
            source,
            analysis,
            theme,
            options.hyperlinks_enabled,
            timings,
        ))
    }

    /// Prepares width-independent visual data using an explicit language.
    pub fn named_language(
        language_name: &str,
        source: &str,
        options: RenderOptions,
    ) -> Result<Self> {
        let theme = options.theme;
        let mut timings = RenderTimings::default();
        let analysis =
            AnalysisDocument::named_language(language_name, source, theme, Some(&mut timings))?;
        Ok(Self::from_analysis(
            source,
            analysis,
            theme,
            options.hyperlinks_enabled,
            timings,
        ))
    }

    fn from_analysis(
        source: &str,
        analysis: AnalysisDocument,
        theme: Theme,
        hyperlinks_enabled: bool,
        timings: RenderTimings,
    ) -> Self {
        let visual =
            (!analysis.is_plain_passthrough()).then(|| VisualDocument::from_analysis(&analysis));
        Self {
            source: source.to_owned(),
            visual,
            theme,
            hyperlinks_enabled,
            timings,
        }
    }

    /// Syntax styles in original UTF-8 byte offsets.
    pub fn spans(&self) -> Vec<HighlightSpan> {
        self.visual
            .as_ref()
            .map(|visual| {
                visual
                    .spans()
                    .iter()
                    .filter_map(|span| {
                        span.style.map(|style| HighlightSpan {
                            range: span.range.clone(),
                            style: style.into(),
                        })
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Nested-region backgrounds in columns of each unwrapped source line.
    pub fn background_lines(&self, tab_width: usize) -> Vec<Vec<HighlightBackgroundRun>> {
        let Some(visual) = self.visual.as_ref() else {
            return Vec::new();
        };
        let layout = LayoutDocument::from_visual_with_tab_width(
            &self.source,
            visual.spans(),
            visual.regions(),
            self.theme,
            None,
            tab_width.max(1),
        );
        layout
            .rows()
            .iter()
            .map(|row| {
                row.background_runs
                    .iter()
                    .filter_map(|run| {
                        let color = run.style.background_rgb()?;
                        Some(HighlightBackgroundRun {
                            start_column: run.start_column,
                            end_column: run.end_column,
                            background: (color.0, color.1, color.2),
                        })
                    })
                    .collect()
            })
            .collect()
    }

    /// Lays out and encodes the prepared document for `terminal_width`.
    pub fn render(&self, terminal_width: Option<usize>) -> RenderedDocument {
        let mut timings = self.timings.clone();
        let Some(visual) = self.visual.as_ref() else {
            return RenderedDocument {
                output: self.source.clone(),
                row_anchors: plain_row_anchors(&self.source),
                timings,
            };
        };

        let layout_started_at = Instant::now();
        let layout = LayoutDocument::from_visual(
            &self.source,
            visual.spans(),
            visual.regions(),
            self.theme,
            terminal_width,
        );
        timings.layout += layout_started_at.elapsed();
        let mut row_anchors = layout
            .rows()
            .iter()
            .map(|row| RenderedRowAnchor {
                source_line_start: row.source_line_start,
                wrapped_from_column: row.wrapped_from_column,
            })
            .collect::<Vec<_>>();
        let render_started_at = Instant::now();
        let output =
            RenderPlan::compile(&layout, self.theme).encode(self.theme, self.hyperlinks_enabled);
        timings.record_render_styled_spans(render_started_at);
        row_anchors.truncate(rendered_line_count(&output));

        RenderedDocument {
            output,
            row_anchors,
            timings,
        }
    }
}

pub(crate) fn plain_row_anchors(source: &str) -> Vec<RenderedRowAnchor> {
    let mut anchors = Vec::new();
    let mut line_start = 0usize;
    for line in source.split_inclusive('\n') {
        anchors.push(RenderedRowAnchor {
            source_line_start: line_start,
            wrapped_from_column: 0,
        });
        line_start += line.len();
    }
    if source.is_empty() {
        anchors.push(RenderedRowAnchor::default());
    }
    anchors.truncate(rendered_line_count(source));
    anchors
}

pub(crate) fn rendered_line_count(output: &str) -> usize {
    let line_count = output.split('\n').count();
    line_count.saturating_sub(usize::from(output.ends_with('\n')))
}

pub(crate) fn detected_theme() -> Theme {
    TerminalCapabilities::detect().theme()
}

#[cfg(test)]
pub(crate) fn render_with_theme(
    source_path: Option<&Path>,
    source: &str,
    theme: &Theme,
) -> Result<String> {
    let options = RenderOptions {
        theme: *theme,
        hyperlinks_enabled: false,
    };
    Ok(PreparedDocument::detect(source_path, source, options)?
        .render(None)
        .output)
}
