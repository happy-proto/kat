use std::{io::Write, path::Path};

use anstyle::{AnsiColor, Color, RgbColor, Style};
use anyhow::Result;
use kat::{ColorMode, DraculaColor, PreparedDocument};
use unicode_segmentation::UnicodeSegmentation;

/// Presentation shared by directory browsing and full-screen reading.
#[derive(Clone, Copy)]
pub(crate) struct UiTheme {
    mode: ColorMode,
}

pub(crate) enum StatusLeft<'a> {
    Path(&'a str),
    Input(&'a str),
}

impl UiTheme {
    pub(crate) fn write_status(
        &self,
        stdout: &mut impl Write,
        (y, cols): (usize, usize),
        left: StatusLeft<'_>,
        status: &str,
        color: DraculaColor,
    ) -> Result<()> {
        let status = visible_text(status).replace('\n', "\\n");
        let path = match left {
            StatusLeft::Path(path) => visible_text(path).replace('\n', "\\n"),
            StatusLeft::Input(query) => {
                return self.write_input_status(stdout, (y, cols), query, &status, color);
            }
        };
        // Keep the path and the start of the status (position, query or error)
        // visible. Lower-priority key hints are clipped on narrow terminals.
        let status = prefix(&status, cols.saturating_sub(2) / 2);
        let status_width = text_width(&status);
        let gap = 2.min(cols.saturating_sub(status_width));
        let path = compact_path(&path, cols.saturating_sub(status_width + gap), self.mode);
        let padding =
            cols.saturating_sub(path.iter().map(|part| part.width()).sum::<usize>() + status_width);
        let normal = self.style(DraculaColor::Foreground, true, false);
        let shortened = match self.mode {
            ColorMode::TrueColor => normal.fg_color(Some(Color::Rgb(RgbColor(184, 176, 208)))),
            ColorMode::Ansi => normal.fg_color(Some(Color::Ansi(AnsiColor::Magenta))),
            ColorMode::NoColor => normal,
        };
        write!(stdout, "\x1b[{y};1H\x1b[0m")?;
        for part in path {
            write!(
                stdout,
                "{}{}{}{}",
                if part.shortened { shortened } else { normal },
                part.name,
                normal,
                part.separator
            )?;
        }
        write!(
            stdout,
            "{}{}{}{}\x1b[0m",
            normal,
            " ".repeat(padding),
            self.style(color, true, false),
            status
        )?;
        Ok(())
    }

    fn write_input_status(
        &self,
        stdout: &mut impl Write,
        (y, cols): (usize, usize),
        query: &str,
        status: &str,
        color: DraculaColor,
    ) -> Result<()> {
        let query = visible_text(query).replace('\n', "\\n");
        // Input takes priority over hints, including the tail currently being typed.
        let available = cols.saturating_sub(1 + text_width(&query) + 2);
        let status = prefix(status, available.min(cols.saturating_sub(2) / 2));
        let status_width = text_width(&status);
        let gap = if status_width == 0 { 0 } else { 2 };
        let input = search_input(&query, cols.saturating_sub(status_width + gap));
        let padding = cols.saturating_sub(text_width(&input) + status_width);
        write!(
            stdout,
            "\x1b[{y};1H\x1b[0m{}{}{}{}{}{}\x1b[0m",
            self.style(DraculaColor::Cyan, true, false),
            input,
            self.style(DraculaColor::Foreground, true, false),
            " ".repeat(padding),
            self.style(color, true, false),
            status,
        )?;
        Ok(())
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
}

pub(crate) fn display_path(path: &Path) -> String {
    #[allow(deprecated)]
    let home = std::env::home_dir();
    home.as_deref().map_or_else(
        || path.to_string_lossy().into_owned(),
        |home| abbreviate_home(path, home),
    )
}

fn abbreviate_home(path: &Path, home: &Path) -> String {
    match path.strip_prefix(home) {
        Ok(relative) if relative.as_os_str().is_empty() => "~".to_owned(),
        Ok(relative) => Path::new("~").join(relative).to_string_lossy().into_owned(),
        Err(_) => path.to_string_lossy().into_owned(),
    }
}

fn text_width(text: &str) -> usize {
    PreparedDocument::plain(text)
        .display_rows(usize::MAX)
        .first()
        .map_or(0, |row| row.display_width())
}

fn prefix(text: &str, width: usize) -> String {
    if width == 0 {
        return String::new();
    }
    PreparedDocument::plain(text)
        .display_rows(width)
        .first()
        .map_or_else(String::new, |row| row.text().to_owned())
}

fn search_input(query: &str, width: usize) -> String {
    if width == 0 {
        return String::new();
    }
    let mut start = query.len();
    let mut used = 1; // Keep the search prompt even when the query is clipped.
    for (index, grapheme) in query.grapheme_indices(true).rev() {
        used += text_width(grapheme);
        if used > width {
            break;
        }
        start = index;
    }
    format!("/{}", &query[start..])
}

/// A directory name and its original separator. Only transformed names change color.
struct PathPart {
    name: String,
    separator: String,
    shortened: bool,
}

impl PathPart {
    fn width(&self) -> usize {
        text_width(&self.name) + text_width(&self.separator)
    }

    fn is_root(&self) -> bool {
        matches!(self.name.as_str(), "" | "~" | "." | "..")
            || (cfg!(windows) && self.name.ends_with(':'))
    }
}

fn compact_path(text: &str, width: usize, mode: ColorMode) -> Vec<PathPart> {
    if width == 0 {
        return Vec::new();
    }
    let separator = |c| c == '/' || (cfg!(windows) && c == '\\');
    let mut parts = text
        .split_inclusive(separator)
        .map(|part| {
            let split = if part.ends_with(separator) {
                part.len() - 1
            } else {
                part.len()
            };
            PathPart {
                name: part[..split].to_owned(),
                separator: part[split..].to_owned(),
                shortened: false,
            }
        })
        .collect::<Vec<_>>();
    let fits = |parts: &[PathPart]| parts.iter().map(PathPart::width).sum::<usize>() <= width;
    if fits(&parts) {
        return parts;
    }
    // Shorten distant ancestors first, preserving the final name and its parent.
    for index in 0..parts.len().saturating_sub(2) {
        let part = &mut parts[index];
        if part.is_root() {
            continue;
        }
        let mut graphemes = part.name.graphemes(true);
        let first = graphemes.next().unwrap_or("");
        let abbreviated = if first == "." {
            format!(".{next}", next = graphemes.next().unwrap_or(""))
        } else {
            first.to_owned()
        };
        let abbreviated = if mode == ColorMode::NoColor {
            format!("{abbreviated}…")
        } else {
            abbreviated
        };
        if text_width(&abbreviated) < text_width(&part.name) {
            part.name = abbreviated;
            part.shortened = true;
        }
        if fits(&parts) {
            return parts;
        }
    }
    // Fold ancestors into one explicit omission marker before sacrificing the
    // immediate parent. Keep home/root notation while space permits.
    for keep in [2, 1] {
        let start = parts
            .iter()
            .position(|part| !part.is_root())
            .unwrap_or(parts.len());
        let end = parts.len().saturating_sub(keep);
        if start < end {
            let separator = parts[end - 1].separator.clone();
            parts.splice(
                start..end,
                [PathPart {
                    name: "…".to_owned(),
                    separator,
                    shortened: true,
                }],
            );
        }
        if fits(&parts) {
            return parts;
        }
    }
    let Some(last) = parts.pop() else {
        return parts;
    };
    if last.width() + 2 <= width {
        return vec![
            PathPart {
                name: "…".to_owned(),
                separator: std::path::MAIN_SEPARATOR.to_string(),
                shortened: true,
            },
            last,
        ];
    }
    if last.width() <= width {
        // The filename fits, but even home/root and the omission marker do not.
        return vec![last];
    }
    vec![PathPart {
        name: compact_name(
            &last.name,
            width.saturating_sub(text_width(&last.separator)),
        ),
        separator: prefix(&last.separator, width),
        shortened: true,
    }]
}

fn compact_name(name: &str, width: usize) -> String {
    if width == 0 {
        return String::new();
    }
    // Retain the extension where possible, and keep both ends of extensionless
    // names. All measurements use layout's grapheme/display-column semantics.
    let extension = name
        .rfind('.')
        .filter(|&index| index > 0)
        .map(|index| &name[index..]);
    let suffix_width = extension
        .map(text_width)
        .filter(|&n| n + 2 <= width)
        .unwrap_or((width - 1) / 2);
    let mut start = name.len();
    let mut used = 0;
    for (index, grapheme) in name.grapheme_indices(true).rev() {
        used += text_width(grapheme);
        if used > suffix_width {
            break;
        }
        start = index;
    }
    let suffix = &name[start..];
    format!("{}…{suffix}", prefix(name, width - 1 - text_width(suffix)))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_input_preserves_prompt_and_query_tail_at_grapheme_boundaries() {
        for (query, width, expected) in [
            ("", 0, ""),
            ("", 1, "/"),
            ("path/to/needle", 30, "/path/to/needle"),
            ("path/to/needle", 7, "/needle"),
            ("中文", 4, "/文"),
            ("中文", 2, "/"),
            ("abc👩‍💻", 3, "/👩‍💻"),
            ("abc👩‍💻", 2, "/"),
            ("abc e\u{301}", 2, "/e\u{301}"),
        ] {
            let input = search_input(query, width);
            assert_eq!(input, expected, "query {query:?}, width {width}");
            assert!(text_width(&input) <= width);
        }
    }

    fn shown(parts: &[PathPart]) -> String {
        parts
            .iter()
            .map(|part| format!("{}{}", part.name, part.separator))
            .collect()
    }

    #[test]
    fn path_compression_preserves_nearby_context_and_marks_only_changes() {
        let path = "~/Documents/Projects/happy-proto/kat/Cargo.toml";
        for (width, expected) in [
            (80, path),
            (40, "~/D/Projects/happy-proto/kat/Cargo.toml"),
            (28, "~/D/P/h/kat/Cargo.toml"),
            (20, "~/…/kat/Cargo.toml"),
            (15, "~/…/Cargo.toml"),
            (12, "…/Cargo.toml"),
            (10, "Cargo.toml"),
            (9, "Car….toml"),
            (1, "…"),
            (0, ""),
        ] {
            let parts = compact_path(path, width, ColorMode::TrueColor);
            assert_eq!(shown(&parts), expected, "width {width}");
            assert!(parts.iter().map(PathPart::width).sum::<usize>() <= width);
            for part in parts.iter().filter(|part| part.shortened) {
                assert!(matches!(
                    part.name.as_str(),
                    "D" | "P" | "h" | "…" | "Car….toml"
                ));
            }
        }
    }

    #[test]
    fn path_compression_handles_hidden_unicode_and_plain_text_names() {
        for mode in [ColorMode::TrueColor, ColorMode::Ansi, ColorMode::NoColor] {
            let parts = compact_path(
                "~/.configuration/👩‍💻workspace/中文项目/kat/file.rs",
                30,
                mode,
            );
            let text = shown(&parts);
            assert!(text.ends_with("/kat/file.rs"), "{text}");
            assert!(text.starts_with("~/.c"), "{text}");
            assert!(parts.iter().map(PathPart::width).sum::<usize>() <= 30);
            if mode == ColorMode::NoColor {
                assert!(text.starts_with("~/.c…/"), "{text}");
                assert!(
                    parts
                        .iter()
                        .filter(|part| part.shortened)
                        .all(|part| part.name.ends_with('…'))
                );
            }
        }
        let parts = compact_path("a/b/kat/file.rs", 80, ColorMode::NoColor);
        assert!(parts.iter().all(|part| !part.shortened));
        assert_eq!(shown(&parts), "a/b/kat/file.rs");
    }

    #[test]
    fn home_abbreviation_respects_path_components() {
        let home = Path::new("/users/reader");
        for (path, expected) in [
            ("/users/reader", "~"),
            (
                "/users/reader/project/文件.toml",
                if cfg!(windows) {
                    r"~\project/文件.toml"
                } else {
                    "~/project/文件.toml"
                },
            ),
            ("/users/reader-other/file", "/users/reader-other/file"),
            ("project/file", "project/file"),
        ] {
            assert_eq!(abbreviate_home(Path::new(path), home), expected);
        }
    }
}
