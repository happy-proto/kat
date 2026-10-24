use std::{io::Write, path::Path};

use anstyle::Style;
use anyhow::Result;
use kat::{ColorMode, DraculaColor, PreparedDocument};

use super::{Entry, visible_text};
use crate::IconMode;

#[derive(Clone, Copy)]
pub(super) struct EntryIcon {
    glyph: char,
    color: DraculaColor,
}

impl EntryIcon {
    pub(super) fn for_path(path: &Path, directory: bool) -> Self {
        if directory {
            return Self {
                glyph: '\u{f07b}',
                color: DraculaColor::Purple,
            };
        }
        let icon = match path.file_name().and_then(|name| name.to_str()) {
            // Cargo manifests and lockfiles belong to the Rust toolchain.
            Some("Cargo.toml" | "Cargo.lock") => devicons::FileIcon::from("rs"),
            _ => devicons::FileIcon::from(path),
        };
        if icon == devicons::FileIcon::default() {
            return Self {
                glyph: '\u{f15b}',
                color: DraculaColor::Comment,
            };
        }
        Self {
            glyph: icon.icon,
            color: dracula_accent(icon.color),
        }
    }
}

// Keep the upstream file associations, but harmonize their colors with kat's
// existing Dracula palette instead of maintaining a second per-file theme.
fn dracula_accent(hex: &str) -> DraculaColor {
    let rgb = u32::from_str_radix(hex.trim_start_matches('#'), 16).unwrap_or(0xffffff);
    let channel = |shift: u32| ((rgb >> shift) & 255_u32) as i32;
    [
        DraculaColor::Red,
        DraculaColor::Orange,
        DraculaColor::Yellow,
        DraculaColor::Green,
        DraculaColor::Purple,
        DraculaColor::Cyan,
        DraculaColor::Pink,
        DraculaColor::Foreground,
    ]
    .into_iter()
    .min_by_key(|color| {
        let rgb = color.to_rgb_color();
        (channel(16) - i32::from(rgb.0)).pow(2)
            + (channel(8) - i32::from(rgb.1)).pow(2)
            + (channel(0) - i32::from(rgb.2)).pow(2)
    })
    .unwrap()
}

pub(super) struct BrowserTheme {
    mode: ColorMode,
    icons: IconMode,
}

impl BrowserTheme {
    pub(super) fn new(mode: ColorMode, icons: IconMode) -> Self {
        Self { mode, icons }
    }

    fn style(&self, color: DraculaColor, surface: bool, bold: bool) -> Style {
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

    pub(super) fn text_style(&self, color: DraculaColor) -> Style {
        self.style(color, false, false)
    }

    pub(super) fn write_bar(
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

    pub(super) fn write_entry(
        &self,
        stdout: &mut impl Write,
        entry: &Entry,
        (y, x): (usize, usize),
        cols: usize,
        selection: Option<bool>,
    ) -> Result<()> {
        let selected = selection == Some(true);
        let marker = match selection {
            Some(true) => "> ",
            Some(false) => "  ",
            None => "",
        };
        let mut text = marker.to_owned();
        let icon_start = text.len();
        if self.icons == IconMode::Always {
            let glyph = entry.icon.glyph.to_string();
            // Layout owns glyph widths. Reserve two cells for a Nerd Font
            // symbol and one separating cell, including on terminals that
            // enlarge a symbol into the following blank cell.
            let width = PreparedDocument::plain(&glyph).display_rows(4)[0].display_width();
            text.push_str(&glyph);
            text.push_str(&" ".repeat(3_usize.saturating_sub(width)));
        }
        let name_start = text.len();
        text.push_str(&entry.label());
        let document = PreparedDocument::plain(&text);
        if let Some(row) = document.display_rows(cols).first() {
            let text = row.text();
            let icon_start = icon_start.min(text.len());
            let name_start = name_start.min(text.len());
            write!(
                stdout,
                "\x1b[{y};{x}H\x1b[0m{}{}",
                self.style(DraculaColor::Foreground, selected, selected),
                &text[..icon_start]
            )?;
            write!(
                stdout,
                "{}{}",
                self.style(entry.icon.color, selected, entry.directory),
                &text[icon_start..name_start]
            )?;
            write!(
                stdout,
                "{}{}{}\x1b[0m",
                self.style(
                    if entry.directory {
                        DraculaColor::Purple
                    } else {
                        DraculaColor::Foreground
                    },
                    selected,
                    entry.directory
                ),
                &text[name_start..],
                " ".repeat(cols.saturating_sub(row.display_width()))
            )?;
        }
        Ok(())
    }
}
