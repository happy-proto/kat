use std::{ops::Range, path::Path};

use anyhow::Result;

use crate::{
    analysis::AnalysisDocument,
    layout::LayoutDocument,
    terminal_background::derive_nested_region_tint,
    theme::{ColorMode, Theme, TokenStyle},
    visual::VisualDocument,
};

/// A Dracula syntax style, independent of terminal capabilities and layout.
///
/// Backgrounds for nested blocks are deliberately excluded: they depend on
/// the viewer's width and belong to its layout rather than a source range.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HighlightStyle {
    pub foreground: (u8, u8, u8),
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub strikethrough: bool,
}

impl From<TokenStyle> for HighlightStyle {
    fn from(style: TokenStyle) -> Self {
        let rgb = style.foreground_rgb();
        Self {
            foreground: (rgb.0, rgb.1, rgb.2),
            bold: style.is_bold(),
            italic: style.is_italic(),
            underline: style.is_underlined(),
            strikethrough: style.is_strikethrough(),
        }
    }
}

/// A styled range in the original source's UTF-8 byte offsets.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HighlightSpan {
    pub range: Range<usize>,
    pub style: HighlightStyle,
}

/// A painted background interval in display columns of an unwrapped source line.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HighlightBackgroundRun {
    pub start_column: usize,
    pub end_column: usize,
    pub background: (u8, u8, u8),
}

/// Parsed syntax and nested visual regions retained for layout at the viewer's tab width.
#[derive(Clone, Debug)]
pub struct HighlightDocument {
    source: String,
    spans: Vec<HighlightSpan>,
    visual: VisualDocument,
}

impl HighlightDocument {
    pub fn new(source_path: Option<&Path>, source: &str) -> Result<Self> {
        let theme = Theme::new(ColorMode::TrueColor, None);
        let analysis = AnalysisDocument::detect(source_path, source, theme, None)?;
        let spans = analysis
            .spans()
            .iter()
            .filter_map(|span| {
                span.style.map(|style| HighlightSpan {
                    range: span.range.clone(),
                    style: style.into(),
                })
            })
            .collect();
        Ok(Self {
            source: source.to_owned(),
            spans,
            visual: VisualDocument::from_analysis(&analysis),
        })
    }

    pub fn spans(&self) -> &[HighlightSpan] {
        &self.spans
    }

    /// Compute every nested background with kat's visual-region geometry.
    /// The tint is derived from the viewer's terminal colors without probing the terminal.
    pub fn background_lines(
        &self,
        tab_width: usize,
        foreground: (u8, u8, u8),
        background: (u8, u8, u8),
    ) -> Vec<Vec<HighlightBackgroundRun>> {
        use anstyle::RgbColor;

        let to_rgb = |(r, g, b)| RgbColor(r, g, b);
        let tint = derive_nested_region_tint(to_rgb(foreground), to_rgb(background));
        let theme = Theme::new(ColorMode::TrueColor, Some(tint));
        let layout = LayoutDocument::from_visual_with_tab_width(
            &self.source,
            self.visual.spans(),
            self.visual.regions(),
            theme,
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
}

/// Analyze a whole source document without probing a terminal or wrapping text.
///
/// Nested language and semantic overlays are already merged into these ranges.
/// An unrecognized file returns an empty vector. The caller retains ownership
/// of line splitting, diff backgrounds, wrapping, and rendering.
pub fn highlight_source_spans(
    source_path: Option<&Path>,
    source: &str,
) -> Result<Vec<HighlightSpan>> {
    let theme = Theme::new(ColorMode::TrueColor, None);
    let analysis = AnalysisDocument::detect(source_path, source, theme, None)?;
    Ok(analysis
        .spans()
        .iter()
        .filter_map(|span| {
            span.style.map(|style| HighlightSpan {
                range: span.range.clone(),
                style: style.into(),
            })
        })
        .collect())
}
