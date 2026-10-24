use std::io::Write;

use anstyle::Style;
use anyhow::Result;
use kat::{ColorMode, DraculaColor, PreparedDocument};
use unicode_segmentation::UnicodeSegmentation;

/// Presentation shared by directory browsing and full-screen reading.
#[derive(Clone, Copy)]
pub(crate) struct UiTheme {
    mode: ColorMode,
}

impl UiTheme {
    pub(crate) fn write_header(
        &self,
        stdout: &mut impl Write,
        title: &str,
        cols: usize,
    ) -> Result<()> {
        let text = visible_text(title).replace('\n', "\\n");
        let fits = |text: &str, width: usize| {
            let rows = PreparedDocument::plain(text).display_rows(width.max(1));
            rows.len() <= 1 && rows.first().is_none_or(|row| row.display_width() <= width)
        };
        let title = if fits(&text, cols) {
            text
        } else if cols < 8 {
            "kat".to_owned()
        } else {
            // Preserve the file name at the right of long paths. Width checks
            // use the same layout as document rows and grapheme-safe slicing.
            let mut start = text.len();
            for (index, _) in text.grapheme_indices(true).rev() {
                if !fits(&text[index..], cols - 6) {
                    break;
                }
                start = index;
            }
            format!("kat  …{}", &text[start..])
        };
        self.write_bar(stdout, 1, &title, cols, DraculaColor::Purple)
    }

    pub(crate) fn new(mode: ColorMode) -> Self {
        Self { mode }
    }

    pub(crate) fn style(&self, color: DraculaColor, surface: bool, bold: bool) -> Style {
        if self.mode == ColorMode::NoColor {
            return if surface {
                Style::new().invert()
            } else {
                Style::new()
            };
        }
        let mut style = Style::new().fg_color(Some(color.to_color(self.mode)));
        if surface {
            style = style.bg_color(Some(DraculaColor::CurrentLine.to_color(self.mode)));
        }
        if bold {
            style = style.bold();
        }
        style
    }

    pub(crate) fn text_style(&self, color: DraculaColor) -> Style {
        self.style(color, false, false)
    }

    pub(crate) fn write_bar(
        &self,
        stdout: &mut impl Write,
        y: usize,
        text: &str,
        cols: usize,
        color: DraculaColor,
    ) -> Result<()> {
        let document = PreparedDocument::plain(&visible_text(text).replace('\n', "\\n"));
        write!(
            stdout,
            "\x1b[{y};1H\x1b[0m{}",
            self.style(color, true, y == 1)
        )?;
        if let Some(row) = document.display_rows(cols).first() {
            write!(
                stdout,
                "{}{}",
                row.text(),
                " ".repeat(cols.saturating_sub(row.display_width()))
            )?;
        }
        write!(stdout, "\x1b[0m")?;
        Ok(())
    }
}

pub(crate) fn visible_text(text: &str) -> String {
    text.chars()
        .flat_map(|c| {
            if c.is_control() && c != '\n' {
                c.escape_default().collect::<Vec<_>>()
            } else {
                vec![c]
            }
        })
        .collect()
}
