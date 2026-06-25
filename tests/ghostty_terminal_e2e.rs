#![cfg(feature = "ghostty-e2e")]

use std::{
    fs,
    io::Read,
    path::{Path, PathBuf},
    thread,
};

use libghostty_vt::{
    RenderState, Terminal, TerminalOptions,
    render::{CellIterator, RowIterator},
    terminal::{Point, PointCoordinate},
};
use portable_pty::{CommandBuilder, PtySize, native_pty_system};

const COLS: u16 = 32;
const ROWS: u16 = 12;

#[test]
fn kat_uses_pty_width_for_wrapping() -> Result<(), Box<dyn std::error::Error>> {
    let narrow = render_fixture_in_ghostty("markdown/ghostty-e2e.md", COLS, ROWS)?;
    let wide = render_fixture_in_ghostty("markdown/ghostty-e2e.md", 96, ROWS)?;

    narrow.assert_screen_contains("This line is intentionally");
    narrow.assert_wrapped_body_line();
    wide.assert_screen_line("wider PTY to keep the body line unwrapped", |line| {
            line.trim_end()
                == "This line is intentionally long enough to require terminal wrapping at a narrow width."
    });

    Ok(())
}

#[test]
fn kat_preserves_markdown_hyperlinks_on_ghostty_cells() -> Result<(), Box<dyn std::error::Error>> {
    let rendered = render_fixture_in_ghostty("markdown/ghostty-e2e.md", COLS, ROWS)?;

    rendered.assert_hyperlink_uri("https://www.rust-lang.org/")?;

    Ok(())
}

#[test]
fn kat_does_not_inject_hyperlinks_for_plain_text() -> Result<(), Box<dyn std::error::Error>> {
    let rendered = render_fixture_in_ghostty("plain/ghostty-url.txt", COLS, ROWS)?;

    rendered.assert_screen_contains("https://www.rust-lang.org/");
    rendered.assert_no_hyperlinks()?;

    Ok(())
}

#[test]
fn kat_omits_markdown_hyperlinks_when_disabled() -> Result<(), Box<dyn std::error::Error>> {
    let rendered =
        render_fixture_in_ghostty_with_hyperlinks("markdown/ghostty-e2e.md", COLS, ROWS, "never")?;

    rendered.assert_screen_contains("[Rust](https://www.rust-lang.org");
    rendered.assert_no_hyperlinks()?;

    Ok(())
}

#[test]
fn kat_rejects_control_character_markdown_hyperlinks() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = temp_markdown_fixture(
        "ghostty-unsafe-url",
        "See <https://example.com/\x1bpath>.\n",
    )?;
    let rendered = render_path_in_ghostty(&fixture, COLS, ROWS, "always")?;

    rendered.assert_screen_contains("https://example.com/");
    rendered.assert_no_hyperlinks()?;

    Ok(())
}

#[test]
fn kat_expands_markdown_html_tabs_on_ghostty_cells() -> Result<(), Box<dyn std::error::Error>> {
    let rendered = render_fixture_in_ghostty("markdown/html_block_tabs.md", 120, 18)?;

    rendered.assert_compact_screen_line_contains("tabbed HTML cell", "<td>值", "<td>值标签</td>");

    Ok(())
}

#[test]
fn kat_keeps_wrapped_just_recipe_rows_adjacent() -> Result<(), Box<dyn std::error::Error>> {
    let rendered = render_testdata_in_ghostty("showcase/just/recipe-block.just", 80, 18, "always")?;

    rendered.assert_adjacent_screen_lines("@cd mdsre", "uv run mdsre mongo query");

    Ok(())
}

struct RenderedTerminal {
    terminal: Terminal<'static, 'static>,
    screen: Vec<String>,
}

impl RenderedTerminal {
    fn screen_text(&self) -> String {
        self.screen.join("\n")
    }

    fn assert_screen_contains(&self, needle: &str) {
        self.assert_screen_line(needle, |line| line.contains(needle));
    }

    fn assert_screen_line(&self, description: &str, predicate: impl Fn(&str) -> bool) {
        assert!(
            self.screen.iter().any(|line| predicate(line)),
            "expected {description} in Ghostty screen:\n{}",
            self.screen_text()
        );
    }

    fn assert_compact_screen_line_contains(
        &self,
        description: &str,
        line_needle: &str,
        compact_needle: &str,
    ) {
        let line = self
            .screen
            .iter()
            .find(|line| line.contains(line_needle))
            .unwrap_or_else(|| {
                panic!(
                    "expected {description} in Ghostty screen:\n{}",
                    self.screen_text()
                )
            });
        let compact_line = line
            .chars()
            .filter(|ch| !ch.is_whitespace())
            .collect::<String>();

        assert!(
            !line.contains('\t') && compact_line.contains(compact_needle),
            "expected {description} to render as display spaces:\n{}",
            self.screen_text()
        );
    }

    fn assert_wrapped_body_line(&self) {
        let wrapped = self.screen.windows(2).any(|rows| {
            rows[0].trim_end() == "This line is intentionally long"
                && rows[1]
                    .trim_end()
                    .starts_with("enough to require terminal wrapp")
        });
        assert!(
            wrapped,
            "expected Ghostty to apply terminal-width wrapping:\n{}",
            self.screen_text()
        );
    }

    fn assert_adjacent_screen_lines(&self, first: &str, second: &str) {
        let first_index = self.line_index(first);
        let second_index = self.line_index(second);

        assert_eq!(
            second_index,
            first_index + 1,
            "expected no blank screen row between {first:?} and {second:?}:\n{}",
            self.screen_text()
        );
    }

    fn line_index(&self, needle: &str) -> usize {
        self.screen
            .iter()
            .position(|line| line.contains(needle))
            .unwrap_or_else(|| {
                panic!(
                    "expected Ghostty screen to contain {needle:?}:\n{}",
                    self.screen_text()
                )
            })
    }

    fn assert_hyperlink_uri(&self, uri: &str) -> Result<(), Box<dyn std::error::Error>> {
        assert!(
            self.hyperlink_uris()?.contains(&uri.to_string()),
            "expected OSC 8 hyperlink {uri:?} on Ghostty cells:\n{}",
            self.screen_text()
        );
        Ok(())
    }

    fn assert_no_hyperlinks(&self) -> Result<(), Box<dyn std::error::Error>> {
        let uris = self.hyperlink_uris()?;
        assert!(
            uris.is_empty(),
            "expected no OSC 8 hyperlink metadata, got {uris:?}:\n{}",
            self.screen_text()
        );
        Ok(())
    }

    fn hyperlink_uris(&self) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let mut uris = Vec::new();
        for y in 0..u32::from(self.terminal.rows()?) {
            for x in 0..self.terminal.cols()? {
                let grid_ref = self
                    .terminal
                    .grid_ref(Point::Active(PointCoordinate { x, y }))?;
                if !grid_ref.cell()?.has_hyperlink()? {
                    continue;
                }
                let mut buf = vec![0_u8; 512];
                let len = grid_ref.hyperlink_uri(&mut buf)?;
                buf.truncate(len);
                let uri = String::from_utf8(buf)?;
                if !uris.contains(&uri) {
                    uris.push(uri);
                }
            }
        }
        Ok(uris)
    }
}

fn render_fixture_in_ghostty(
    fixture: &str,
    cols: u16,
    rows: u16,
) -> Result<RenderedTerminal, Box<dyn std::error::Error>> {
    render_fixture_in_ghostty_with_hyperlinks(fixture, cols, rows, "always")
}

fn render_fixture_in_ghostty_with_hyperlinks(
    fixture: &str,
    cols: u16,
    rows: u16,
    hyperlinks: &str,
) -> Result<RenderedTerminal, Box<dyn std::error::Error>> {
    render_testdata_in_ghostty(&format!("fixtures/{fixture}"), cols, rows, hyperlinks)
}

fn render_testdata_in_ghostty(
    relative_path: &str,
    cols: u16,
    rows: u16,
    hyperlinks: &str,
) -> Result<RenderedTerminal, Box<dyn std::error::Error>> {
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("testdata")
        .join(relative_path);
    render_path_in_ghostty(&fixture, cols, rows, hyperlinks)
}

fn render_path_in_ghostty(
    path: &Path,
    cols: u16,
    rows: u16,
    hyperlinks: &str,
) -> Result<RenderedTerminal, Box<dyn std::error::Error>> {
    let hyperlink_arg = format!("--hyperlinks={hyperlinks}");
    let output = run_kat_in_pty(
        &[
            "--paging=never",
            hyperlink_arg.as_str(),
            path.to_str().expect("fixture path should be UTF-8"),
        ],
        cols,
        rows,
    )?;

    let mut terminal = ghostty_terminal(cols, rows);
    terminal.vt_write(&output);
    let screen = visible_screen_lines(&terminal)?;
    Ok(RenderedTerminal { terminal, screen })
}

fn run_kat_in_pty(
    args: &[&str],
    cols: u16,
    rows: u16,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let pty_system = native_pty_system();
    let pair = pty_system.openpty(PtySize {
        rows,
        cols,
        pixel_width: 0,
        pixel_height: 0,
    })?;
    let mut reader = pair.master.try_clone_reader()?;

    let mut command = CommandBuilder::new(env!("CARGO_BIN_EXE_kat"));
    command.args(args);
    command.env("TERM", "xterm-256color");
    command.env_remove("KAT_HYPERLINKS");
    command.env_remove("NO_COLOR");
    let mut child = pair.slave.spawn_command(command)?;
    drop(pair.slave);

    let reader_thread = thread::spawn(move || {
        let mut output = Vec::new();
        reader.read_to_end(&mut output).map(|_| output)
    });
    let status = child.wait()?;
    let output = reader_thread
        .join()
        .map_err(|_| "PTY reader thread panicked")??;

    assert!(
        status.success(),
        "kat exited with {status:?}; output:\n{}",
        String::from_utf8_lossy(&output)
    );
    Ok(output)
}

fn ghostty_terminal(cols: u16, rows: u16) -> Terminal<'static, 'static> {
    Terminal::new(TerminalOptions {
        cols,
        rows,
        max_scrollback: 100,
    })
    .expect("Ghostty terminal should initialize")
}

fn visible_screen_lines(
    terminal: &Terminal<'_, '_>,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut render_state = RenderState::new()?;
    let snapshot = render_state.update(terminal)?;
    let mut rows = RowIterator::new()?;
    let mut cells = CellIterator::new()?;
    let mut lines = Vec::new();

    let mut row_iter = rows.update(&snapshot)?;
    while let Some(row) = row_iter.next() {
        let mut line = String::new();
        let mut cell_iter = cells.update(row)?;
        while let Some(cell) = cell_iter.next() {
            let mut grapheme = String::new();
            cell.graphemes_utf8(&mut grapheme)?;
            if grapheme.is_empty() {
                line.push(' ');
            } else {
                line.push_str(&grapheme);
            }
        }
        lines.push(line);
    }

    Ok(lines)
}

fn temp_markdown_fixture(name: &str, source: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let path = std::env::temp_dir().join(format!("{name}-{}.md", std::process::id()));
    fs::write(&path, source)?;
    Ok(path)
}
