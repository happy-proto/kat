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
fn kat_output_renders_in_ghostty_screen_model() -> Result<(), Box<dyn std::error::Error>> {
    let fixture =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("testdata/fixtures/markdown/ghostty-e2e.md");
    let output = run_kat_in_pty(
        &[
            "--paging=never",
            "--hyperlinks=always",
            fixture.to_str().expect("fixture path should be UTF-8"),
        ],
        COLS,
        ROWS,
    )?;

    let mut terminal = ghostty_terminal(COLS, ROWS);
    terminal.vt_write(&output);

    let screen = visible_screen_lines(&terminal)?;
    assert!(
        screen
            .iter()
            .any(|line| line.contains("This line is intentionally")),
        "expected rendered Markdown body in Ghostty screen:\n{}",
        screen.join("\n")
    );
    assert!(
        screen
            .iter()
            .any(|line| line.trim_end() == "long enough to require terminal"),
        "expected Ghostty to apply terminal-width wrapping:\n{}",
        screen.join("\n")
    );
    assert!(
        hyperlink_uris(&terminal)?.contains(&"https://www.rust-lang.org/".to_string()),
        "expected OSC 8 hyperlink metadata on Ghostty cells:\n{}",
        screen.join("\n")
    );

    Ok(())
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
    command.env("KAT_HYPERLINKS", "always");
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
