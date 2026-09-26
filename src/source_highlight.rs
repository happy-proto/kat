use std::{ops::Range, path::Path};

use anyhow::Result;

use crate::{
    analysis::AnalysisDocument,
    theme::{ColorMode, Theme, TokenStyle},
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
