use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

// Shared terminal-geometry helpers. Byte offsets remain the source-of-truth for slicing,
// but any width/column math must pass through this module instead of ad-hoc len/count logic.
pub(crate) const DEFAULT_TAB_WIDTH: usize = 8;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct ByteOffset(pub(crate) usize);

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct DisplayColumn(pub(crate) usize);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct DisplayProfile {
    tab_width: usize,
}

impl DisplayProfile {
    pub(crate) const fn new(tab_width: usize) -> Self {
        Self { tab_width }
    }

    pub(crate) fn display_width(self, text: &str) -> DisplayColumn {
        self.advance_column(DisplayColumn(0), text)
    }

    pub(crate) fn display_width_from_column(
        self,
        start_column: DisplayColumn,
        text: &str,
    ) -> DisplayColumn {
        let end_column = self.advance_column(start_column, text);
        DisplayColumn(end_column.0 - start_column.0)
    }

    pub(crate) fn column_for_byte_offset(
        self,
        text: &str,
        byte_offset: ByteOffset,
    ) -> DisplayColumn {
        self.display_width(&text[..byte_offset.0])
    }

    fn advance_column(self, mut column: DisplayColumn, text: &str) -> DisplayColumn {
        for grapheme in text.graphemes(true) {
            column = if grapheme == "\t" {
                next_tab_stop(column, self.tab_width)
            } else {
                column + DisplayColumn(UnicodeWidthStr::width(grapheme))
            };
        }
        column
    }
}

impl Default for DisplayProfile {
    fn default() -> Self {
        Self::new(DEFAULT_TAB_WIDTH)
    }
}

#[cfg(test)]
pub(crate) fn display_width(text: &str) -> DisplayColumn {
    DisplayProfile::default().display_width(text)
}

pub(crate) fn display_width_from_column(start_column: DisplayColumn, text: &str) -> DisplayColumn {
    DisplayProfile::default().display_width_from_column(start_column, text)
}

pub(crate) fn display_column_for_byte_offset(text: &str, byte_offset: ByteOffset) -> DisplayColumn {
    DisplayProfile::default().column_for_byte_offset(text, byte_offset)
}

#[cfg(test)]
pub(crate) fn strip_ansi(text: &str) -> String {
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

fn next_tab_stop(column: DisplayColumn, tab_width: usize) -> DisplayColumn {
    let remainder = column.0 % tab_width;
    if remainder == 0 {
        column + DisplayColumn(tab_width)
    } else {
        column + DisplayColumn(tab_width - remainder)
    }
}

impl ByteOffset {
    pub(crate) const fn new(value: usize) -> Self {
        Self(value)
    }
}

impl DisplayColumn {
    pub(crate) const fn new(value: usize) -> Self {
        Self(value)
    }

    pub(crate) const fn as_usize(self) -> usize {
        self.0
    }
}

impl std::ops::Add for DisplayColumn {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl std::ops::Sub for DisplayColumn {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ByteOffset, DEFAULT_TAB_WIDTH, DisplayColumn, DisplayProfile,
        display_column_for_byte_offset, display_width, display_width_from_column, strip_ansi,
    };

    #[test]
    fn display_width_handles_cjk_and_emoji_sequences() {
        assert_eq!(display_width("短描述。"), DisplayColumn(8));
        assert_eq!(display_width("👩‍💻"), DisplayColumn(2));
    }

    #[test]
    fn display_width_from_column_respects_tab_stops() {
        let profile = DisplayProfile::new(DEFAULT_TAB_WIDTH);
        assert_eq!(profile.display_width("a\tb"), DisplayColumn(9));
        assert_eq!(
            display_width_from_column(DisplayColumn(4), "\tvalue"),
            DisplayColumn(9)
        );
    }

    #[test]
    fn display_column_for_byte_offset_measures_prefix_width() {
        let byte_offset = "前缀\t值".find('值').expect("expected value marker");
        assert_eq!(
            display_column_for_byte_offset("前缀\t值", ByteOffset(byte_offset)),
            DisplayColumn(8)
        );
    }

    #[test]
    fn strip_ansi_removes_control_sequences_without_touching_text() {
        assert_eq!(strip_ansi("\x1b[31m红色\x1b[0m text"), "红色 text");
    }
}
