#![cfg(feature = "ghostty-e2e")]

use std::{io::Read, path::Path, thread};

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

    assert!(
        narrow
            .screen
            .iter()
            .any(|line| line.contains("This line is intentionally")),
        "expected rendered Markdown body in Ghostty screen:\n{}",
        narrow.screen.join("\n")
    );
    assert_wrapped_body_line(&narrow.screen);
    assert!(
        wide.screen.iter().any(|line| {
            line.trim_end()
                == "This line is intentionally long enough to require terminal wrapping at a narrow width."
        }),
        "expected wider PTY to keep the body line unwrapped:\n{}",
        wide.screen.join("\n")
    );

    Ok(())
}

#[test]
fn kat_preserves_markdown_hyperlinks_on_ghostty_cells() -> Result<(), Box<dyn std::error::Error>> {
    let rendered = render_fixture_in_ghostty("markdown/ghostty-e2e.md", COLS, ROWS)?;

    assert!(
        hyperlink_uris(&rendered.terminal)?.contains(&"https://www.rust-lang.org/".to_string()),
        "expected OSC 8 hyperlink metadata on Ghostty cells:\n{}",
        rendered.screen.join("\n")
    );

    Ok(())
}

#[test]
fn kat_does_not_inject_hyperlinks_for_plain_text() -> Result<(), Box<dyn std::error::Error>> {
    let rendered = render_fixture_in_ghostty("plain/ghostty-url.txt", COLS, ROWS)?;

    assert!(
        rendered
            .screen
            .iter()
            .any(|line| line.contains("https://www.rust-lang.org/")),
        "expected plain URL text in Ghostty screen:\n{}",
        rendered.screen.join("\n")
    );
    assert!(
        hyperlink_uris(&rendered.terminal)?.is_empty(),
        "expected plain text rendering not to synthesize OSC 8 hyperlinks:\n{}",
        rendered.screen.join("\n")
    );

    Ok(())
}

#[test]
fn kat_omits_markdown_hyperlinks_when_disabled() -> Result<(), Box<dyn std::error::Error>> {
    let rendered =
        render_fixture_in_ghostty_with_hyperlinks("markdown/ghostty-e2e.md", COLS, ROWS, "never")?;

    assert!(
        rendered
            .screen
            .iter()
            .any(|line| line.contains("[Rust](https://www.rust-lang.org")),
        "expected Markdown URL text in Ghostty screen:\n{}",
        rendered.screen.join("\n")
    );
    assert!(
        hyperlink_uris(&rendered.terminal)?.is_empty(),
        "expected --hyperlinks=never to omit OSC 8 cell metadata:\n{}",
        rendered.screen.join("\n")
    );

    Ok(())
}

#[test]
fn kat_expands_markdown_html_tabs_on_ghostty_cells() -> Result<(), Box<dyn std::error::Error>> {
    let rendered = render_fixture_in_ghostty("markdown/html_block_tabs.md", 120, 18)?;
    let tabbed_cell_line = rendered
        .screen
        .iter()
        .find(|line| line.contains("<td>值"))
        .unwrap_or_else(|| {
            panic!(
                "expected tabbed HTML cell in Ghostty screen:\n{}",
                rendered.screen.join("\n")
            )
        });
    let compact_line = tabbed_cell_line
        .chars()
        .filter(|ch| !ch.is_whitespace())
        .collect::<String>();

    assert!(
        !tabbed_cell_line.contains('\t') && compact_line.contains("<td>值标签</td>"),
        "expected tabbed HTML cells to render as display spaces:\n{}",
        rendered.screen.join("\n")
    );

    Ok(())
}

#[test]
fn kat_keeps_wrapped_just_recipe_rows_adjacent() -> Result<(), Box<dyn std::error::Error>> {
    let rendered = render_testdata_in_ghostty("showcase/just/recipe-block.just", 80, 18, "always")?;

    assert_adjacent_screen_lines(&rendered.screen, "@cd mdsre", "uv run mdsre mongo query");

    Ok(())
}

struct RenderedTerminal {
    terminal: Terminal<'static, 'static>,
    screen: Vec<String>,
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
    let hyperlink_arg = format!("--hyperlinks={hyperlinks}");
    let output = run_kat_in_pty(
        &[
            "--paging=never",
            hyperlink_arg.as_str(),
            fixture.to_str().expect("fixture path should be UTF-8"),
        ],
        cols,
        rows,
    )?;

    let mut terminal = ghostty_terminal(cols, rows);
    terminal.vt_write(&output);
    let screen = visible_screen_lines(&terminal)?;
    Ok(RenderedTerminal { terminal, screen })
}

fn assert_wrapped_body_line(screen: &[String]) {
    let wrapped = screen.windows(2).any(|rows| {
        rows[0].trim_end() == "This line is intentionally long"
            && rows[1]
                .trim_end()
                .starts_with("enough to require terminal wrapp")
    });
    assert!(
        wrapped,
        "expected Ghostty to apply terminal-width wrapping:\n{}",
        screen.join("\n")
    );
}

fn assert_adjacent_screen_lines(screen: &[String], first: &str, second: &str) {
    let first_index = screen
        .iter()
        .position(|line| line.contains(first))
        .unwrap_or_else(|| {
            panic!(
                "expected Ghostty screen to contain {first:?}:\n{}",
                screen.join("\n")
            )
        });
    let second_index = screen
        .iter()
        .position(|line| line.contains(second))
        .unwrap_or_else(|| {
            panic!(
                "expected Ghostty screen to contain {second:?}:\n{}",
                screen.join("\n")
            )
        });

    assert_eq!(
        second_index,
        first_index + 1,
        "expected no blank screen row between {first:?} and {second:?}:\n{}",
        screen.join("\n")
    );
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

fn hyperlink_uris(terminal: &Terminal<'_, '_>) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut uris = Vec::new();
    for y in 0..u32::from(terminal.rows()?) {
        for x in 0..terminal.cols()? {
            let grid_ref = terminal.grid_ref(Point::Active(PointCoordinate { x, y }))?;
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
