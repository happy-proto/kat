#![cfg(feature = "ghostty-e2e")]

use std::{
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::mpsc::{self, Receiver},
    thread,
    time::{Duration, Instant},
};

use libghostty_vt::{
    RenderState, Terminal, TerminalOptions,
    render::{CellIterator, RowIterator},
    screen::Screen,
    style::Underline,
    terminal::{Point, PointCoordinate},
};
use portable_pty::{Child, CommandBuilder, MasterPty, PtySize, native_pty_system};

const COLS: u16 = 32;
const ROWS: u16 = 12;

type TestResult = Result<(), Box<dyn std::error::Error>>;

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

#[test]
fn kat_builtin_pager_ignores_external_pager_and_uses_alternate_screen() -> TestResult {
    let fixture = temp_plain_fixture(
        "ghostty-builtin-pager",
        "builtin pager sentinel\n".repeat(20).as_str(),
    )?;
    let mut session = KatPtySession::spawn(
        &[
            "--hyperlinks=never",
            fixture.to_str().expect("fixture path should be UTF-8"),
        ],
        COLS,
        ROWS,
        &[("PAGER", "sh -c 'printf EXTERNAL_PAGER_MARKER; exit 42'")],
    )?;
    let rendered = session.wait_for_screen(COLS, ROWS, |rendered| {
        rendered.screen_text().contains("q:quit")
    })?;

    rendered.assert_active_screen(Screen::Alternate)?;
    rendered.assert_raw_output_contains("\x1b[?1049h");
    rendered.assert_raw_output_lacks("EXTERNAL_PAGER_MARKER");
    rendered.assert_screen_contains("builtin pager sentinel");
    rendered.assert_screen_contains("q:quit");

    session.write_input(b"q")?;
    session.wait_success()?;

    Ok(())
}

#[test]
fn kat_builtin_pager_reflows_alternate_screen_on_resize() -> TestResult {
    let fixture = temp_plain_fixture(
        "ghostty-builtin-pager-reflow",
        "REFLOW-SENTINEL alpha beta gamma delta epsilon zeta eta theta\n",
    )?;
    let mut session = KatPtySession::spawn(
        &[
            "--hyperlinks=never",
            fixture.to_str().expect("fixture path should be UTF-8"),
        ],
        32,
        ROWS,
        &[],
    )?;
    let mut rendered = session.wait_for_screen(32, ROWS, |rendered| {
        rendered.screen_text().contains("q:quit")
    })?;

    rendered.assert_screen_contains("REFLOW-SENTINEL alpha beta");
    assert!(
        !rendered.screen.iter().any(|line| {
            line.trim_end() == "REFLOW-SENTINEL alpha beta gamma delta epsilon zeta eta theta"
        }),
        "expected narrow Ghostty screen to wrap the sentinel line:\n{}",
        rendered.screen_text()
    );

    session.resize(72, ROWS)?;
    rendered = session.wait_for_screen(72, ROWS, |rendered| {
        rendered.screen.iter().any(|line| {
            line.trim_end() == "REFLOW-SENTINEL alpha beta gamma delta epsilon zeta eta theta"
        })
    })?;
    rendered.assert_screen_line("resized alternate-screen reflow", |line| {
        line.trim_end() == "REFLOW-SENTINEL alpha beta gamma delta epsilon zeta eta theta"
    });

    session.write_input(b"q")?;
    session.wait_success()?;

    Ok(())
}

#[test]
fn kat_builtin_pager_supports_page_home_and_end_navigation() -> TestResult {
    let fixture = temp_plain_fixture(
        "ghostty-builtin-pager-navigation",
        &(1..=40)
            .map(|line| format!("NAV-LINE-{line:02}\n"))
            .collect::<String>(),
    )?;
    let mut session = KatPtySession::spawn(
        &[
            "--hyperlinks=never",
            fixture.to_str().expect("fixture path should be UTF-8"),
        ],
        COLS,
        ROWS,
        &[],
    )?;

    let rendered = session.wait_for_screen(COLS, ROWS, |rendered| {
        rendered.screen_text().contains("NAV-LINE-01")
    })?;
    rendered.assert_active_screen(Screen::Alternate)?;
    rendered.assert_screen_contains("NAV-LINE-01");

    session.write_input(b"\x1b[6~")?;
    let rendered = session.wait_for_screen(COLS, ROWS, |rendered| {
        rendered.screen_text().contains("NAV-LINE-12")
    })?;
    rendered.assert_screen_contains("NAV-LINE-12");

    session.write_input(b"g")?;
    let rendered = session.wait_for_screen(COLS, ROWS, |rendered| {
        rendered.screen_text().contains("NAV-LINE-01")
    })?;
    rendered.assert_screen_contains("NAV-LINE-01");

    session.write_input(b"G")?;
    let rendered = session.wait_for_screen(COLS, ROWS, |rendered| {
        rendered.screen_text().contains("NAV-LINE-40")
    })?;
    rendered.assert_screen_contains("NAV-LINE-40");

    session.write_input(b"q")?;
    let output = session.wait_success()?;
    assert!(
        String::from_utf8_lossy(&output).contains("\x1b[?1049l"),
        "expected pager to leave alternate screen"
    );

    Ok(())
}

#[test]
fn kat_builtin_pager_pages_by_display_rows() -> TestResult {
    let fixture = temp_plain_fixture(
        "ghostty-builtin-pager-display-rows",
        "DISPLAY-ROW-01 alpha beta gamma delta epsilon zeta eta theta iota kappa lambda mu nu xi omicron pi rho sigma tau\nDISPLAY-ROW-02 after wrapped first line\n",
    )?;
    let mut session = KatPtySession::spawn(
        &[
            "--hyperlinks=never",
            fixture.to_str().expect("fixture path should be UTF-8"),
        ],
        24,
        6,
        &[],
    )?;

    let rendered = session.wait_for_screen(24, 6, |rendered| {
        rendered.screen_text().contains("DISPLAY-ROW-01 alpha")
    })?;
    rendered.assert_screen_contains("DISPLAY-ROW-01 alpha");
    assert!(
        !rendered.screen_text().contains("DISPLAY-ROW-02"),
        "expected wrapped first logical line to fill the first page:\n{}",
        rendered.screen_text()
    );

    session.write_input(b"\x1b[6~")?;
    let rendered = session.wait_for_screen(24, 6, |rendered| {
        rendered.screen_text().contains("DISPLAY-ROW-02")
    })?;
    rendered.assert_screen_contains("DISPLAY-ROW-02 after");

    session.write_input(b"q")?;
    session.wait_success()?;

    Ok(())
}

#[test]
fn kat_builtin_pager_preserves_display_row_anchor_on_resize() -> TestResult {
    let fixture = temp_plain_fixture(
        "ghostty-builtin-pager-resize-anchor",
        "ANCHOR-LINE-01 before\nANCHOR-LINE-02 before\nANCHOR-LINE-03 alpha beta gamma delta epsilon zeta eta theta iota kappa lambda mu nu xi omicron pi rho sigma tau\nANCHOR-LINE-04 after\n",
    )?;
    let mut session = KatPtySession::spawn(
        &[
            "--hyperlinks=never",
            fixture.to_str().expect("fixture path should be UTF-8"),
        ],
        28,
        8,
        &[],
    )?;
    session.wait_for_screen(28, 8, |rendered| {
        rendered.screen_text().contains("ANCHOR-LINE-01")
    })?;

    session.write_input(b"j")?;
    session.write_input(b"j")?;
    let rendered = session.wait_for_screen(28, 8, |rendered| {
        rendered
            .screen
            .first()
            .is_some_and(|line| line.contains("ANCHOR-LINE-03"))
    })?;
    rendered.assert_screen_contains("ANCHOR-LINE-03 alpha");

    session.resize(56, 8)?;
    let rendered = session.wait_for_screen(56, 8, |rendered| {
        rendered
            .screen
            .first()
            .is_some_and(|line| line.contains("ANCHOR-LINE-03"))
    })?;
    rendered.assert_screen_contains("ANCHOR-LINE-03 alpha beta");

    session.write_input(b"q")?;
    session.wait_success()?;

    Ok(())
}

#[test]
fn kat_builtin_pager_supports_plain_text_search() -> TestResult {
    let fixture = temp_plain_fixture(
        "ghostty-builtin-pager-search",
        &(1..=30)
            .map(|line| {
                if line == 22 {
                    "SEARCH-HIT visible target\n".to_owned()
                } else {
                    format!("SEARCH-LINE-{line:02}\n")
                }
            })
            .collect::<String>(),
    )?;
    let mut session = KatPtySession::spawn(
        &[
            "--hyperlinks=never",
            fixture.to_str().expect("fixture path should be UTF-8"),
        ],
        COLS,
        ROWS,
        &[],
    )?;
    session.wait_for_screen(COLS, ROWS, |rendered| {
        rendered.screen_text().contains("SEARCH-LINE-01")
    })?;

    session.write_input(b"/SEARCH-HIT\r")?;
    let rendered = session.wait_for_screen(COLS, ROWS, |rendered| {
        rendered.screen_text().contains("SEARCH-HIT visible target")
    })?;
    rendered.assert_screen_contains("SEARCH-HIT visible target");
    rendered.assert_search_highlight_style("SEARCH-HIT", Underline::Single)?;

    session.write_input(b"q")?;
    session.wait_success()?;

    Ok(())
}

#[test]
fn kat_builtin_pager_highlights_all_search_matches_and_current_match() -> TestResult {
    let fixture = temp_plain_fixture(
        "ghostty-builtin-pager-search-highlight",
        "alpha needle one\nbeta needle two\ngamma needle three\n",
    )?;
    let mut session = KatPtySession::spawn(
        &[
            "--hyperlinks=never",
            fixture.to_str().expect("fixture path should be UTF-8"),
        ],
        COLS,
        ROWS,
        &[],
    )?;
    session.wait_for_screen(COLS, ROWS, |rendered| {
        rendered.screen_text().contains("alpha needle one")
    })?;

    session.write_input(b"/needle\r")?;
    let rendered = session.wait_for_screen(COLS, ROWS, |rendered| {
        rendered.raw_output_string().contains("\x1b[7;4m")
            && rendered.raw_output_string().contains("\x1b[7m")
    })?;
    rendered.assert_search_highlight_style("alpha needle", Underline::Single)?;
    rendered.assert_search_highlight_style("beta needle", Underline::None)?;

    session.write_input(b"n")?;
    let rendered = session.wait_for_screen(COLS, ROWS, |rendered| {
        rendered.raw_output_string().matches("\x1b[7;4m").count() >= 2
    })?;
    rendered.assert_search_highlight_style("beta needle", Underline::Single)?;
    rendered.assert_search_highlight_style("gamma needle", Underline::None)?;

    session.write_input(b"q")?;
    session.wait_success()?;

    Ok(())
}

struct RenderedTerminal {
    terminal: Terminal<'static, 'static>,
    screen: Vec<String>,
    raw_output: Vec<u8>,
}

impl RenderedTerminal {
    fn screen_text(&self) -> String {
        self.screen.join("\n")
    }

    fn raw_output_string(&self) -> String {
        String::from_utf8_lossy(&self.raw_output).into_owned()
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

    fn assert_raw_output_lacks(&self, needle: &str) {
        let output = String::from_utf8_lossy(&self.raw_output);
        assert!(
            !output.contains(needle),
            "expected PTY output to omit {needle:?}, got:\n{output}"
        );
    }

    fn assert_raw_output_contains(&self, needle: &str) {
        let output = String::from_utf8_lossy(&self.raw_output);
        assert!(
            output.contains(needle),
            "expected PTY output to contain {needle:?}, got:\n{output}"
        );
    }

    fn assert_active_screen(&self, screen: Screen) -> Result<(), Box<dyn std::error::Error>> {
        assert_eq!(
            self.terminal.active_screen()?,
            screen,
            "unexpected active Ghostty screen:\n{}",
            self.screen_text()
        );
        Ok(())
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

    fn assert_search_highlight_style(
        &self,
        text_before_match: &str,
        expected_underline: Underline,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let y = self.line_index(text_before_match) as u32;
        let line = &self.screen[y as usize];
        let x = line
            .find("needle")
            .or_else(|| line.find("SEARCH-HIT"))
            .unwrap_or_else(|| {
                panic!(
                    "expected highlighted search text in line {line:?}:\n{}",
                    self.screen_text()
                )
            }) as u16;
        let grid_ref = self
            .terminal
            .grid_ref(Point::Active(PointCoordinate { x, y }))?;
        let style = grid_ref.style()?;
        assert!(
            style.inverse,
            "expected inverse search highlight at {x},{y} in Ghostty screen:\n{}",
            self.screen_text()
        );
        assert_eq!(
            style.underline,
            expected_underline,
            "unexpected search highlight underline at {x},{y} in Ghostty screen:\n{}",
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
    let mut session = KatPtySession::spawn(
        &[
            hyperlink_arg.as_str(),
            path.to_str().expect("fixture path should be UTF-8"),
        ],
        cols,
        rows,
        &[],
    )?;
    let rendered = session.wait_for_screen(cols, rows, |rendered| {
        rendered.screen_text().contains("q:quit")
    })?;
    session.write_input(b"q")?;
    session.wait_success()?;
    Ok(rendered)
}

fn ghostty_terminal(cols: u16, rows: u16) -> Terminal<'static, 'static> {
    Terminal::new(TerminalOptions {
        cols,
        rows,
        max_scrollback: 100,
    })
    .expect("Ghostty terminal should initialize")
}

struct KatPtySession {
    master: Box<dyn MasterPty>,
    writer: Box<dyn Write + Send>,
    child: Box<dyn Child + Send + Sync>,
    chunks: Receiver<Vec<u8>>,
    reader_thread: Option<thread::JoinHandle<std::io::Result<()>>>,
    output: Vec<u8>,
}

impl KatPtySession {
    fn spawn(
        args: &[&str],
        cols: u16,
        rows: u16,
        envs: &[(&str, &str)],
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let pty_system = native_pty_system();
        let pair = pty_system.openpty(PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        })?;
        let mut reader = pair.master.try_clone_reader()?;
        let writer = pair.master.take_writer()?;

        let mut command = CommandBuilder::new(env!("CARGO_BIN_EXE_kat"));
        command.args(args);
        command.env("TERM", "xterm-256color");
        command.env_remove("KAT_HYPERLINKS");
        command.env_remove("NO_COLOR");
        for (key, value) in envs {
            command.env(key, value);
        }
        let child = pair.slave.spawn_command(command)?;
        drop(pair.slave);

        let (tx, chunks) = mpsc::channel();
        let reader_thread = thread::spawn(move || {
            let mut buf = [0_u8; 4096];
            loop {
                match reader.read(&mut buf) {
                    Ok(0) => return Ok(()),
                    Ok(len) => {
                        if tx.send(buf[..len].to_vec()).is_err() {
                            return Ok(());
                        }
                    }
                    Err(error) => return Err(error),
                }
            }
        });

        Ok(Self {
            master: pair.master,
            writer,
            child,
            chunks,
            reader_thread: Some(reader_thread),
            output: Vec::new(),
        })
    }

    fn wait_for_screen(
        &mut self,
        cols: u16,
        rows: u16,
        predicate: impl Fn(&RenderedTerminal) -> bool,
    ) -> Result<RenderedTerminal, Box<dyn std::error::Error>> {
        let deadline = Instant::now() + Duration::from_secs(5);
        let mut terminal = ghostty_terminal(cols, rows);
        terminal.vt_write(&self.output);

        loop {
            while let Ok(chunk) = self.chunks.try_recv() {
                terminal.vt_write(&chunk);
                self.output.extend(chunk);
            }

            let screen = visible_screen_lines(&terminal)?;
            let rendered = RenderedTerminal {
                terminal,
                screen,
                raw_output: self.output.clone(),
            };
            if predicate(&rendered) {
                return Ok(rendered);
            }
            terminal = rendered.terminal;

            if Instant::now() >= deadline {
                panic!(
                    "timed out waiting for Ghostty screen; PTY output:\n{}",
                    String::from_utf8_lossy(&self.output)
                );
            }

            match self.chunks.recv_timeout(Duration::from_millis(25)) {
                Ok(chunk) => {
                    terminal.vt_write(&chunk);
                    self.output.extend(chunk);
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {}
                Err(mpsc::RecvTimeoutError::Disconnected) => {}
            }
        }
    }

    fn resize(&mut self, cols: u16, rows: u16) -> Result<(), Box<dyn std::error::Error>> {
        self.master.resize(PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        })?;
        Ok(())
    }

    fn write_input(&mut self, bytes: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
        self.writer.write_all(bytes)?;
        self.writer.flush()?;
        Ok(())
    }

    fn wait_success(mut self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let status = self.child.wait()?;
        drop(self.writer);
        while let Ok(chunk) = self.chunks.recv_timeout(Duration::from_millis(25)) {
            self.output.extend(chunk);
        }
        if let Some(reader_thread) = self.reader_thread.take() {
            reader_thread
                .join()
                .map_err(|_| "PTY reader thread panicked")??;
        }

        assert!(
            status.success(),
            "kat exited with {status:?}; output:\n{}",
            String::from_utf8_lossy(&self.output)
        );
        Ok(self.output)
    }
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

fn temp_plain_fixture(name: &str, source: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let path = std::env::temp_dir().join(format!("{name}-{}.txt", std::process::id()));
    fs::write(&path, source)?;
    Ok(path)
}
