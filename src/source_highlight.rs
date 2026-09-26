use std::ops::Range;

use crate::theme::TokenStyle;

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
