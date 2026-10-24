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
    RenderState, Terminal,
    render::{CellIterator, RowIterator},
    screen::Screen,
    style::{StyleColor, Underline},
    terminal::{Point, PointCoordinate},
};
use portable_pty::{Child, CommandBuilder, MasterPty, PtySize, native_pty_system};

const COLS: u16 = 32;
const ROWS: u16 = 12;
const FOREGROUND_QUERY: &[u8] = b"\x1b]10;?\x1b\\";
const FOREGROUND_RESPONSE: &[u8] = b"\x1b]10;rgb:f8f8/f8f8/f2f2\x1b\\";
const BACKGROUND_QUERY: &[u8] = b"\x1b]11;?\x1b\\";
const BACKGROUND_RESPONSE: &[u8] = b"\x1b]11;rgb:2828/2a2a/3636\x1b\\";

type TestResult = Result<(), Box<dyn std::error::Error>>;

struct BrowserFixture(PathBuf);

impl BrowserFixture {
    fn new(name: &str) -> Self {
        let root = std::env::temp_dir().join(format!("kat-browser-{name}-{}", std::process::id()));
        fs::create_dir_all(&root).unwrap();
        Self(root)
    }

    fn write(&self, name: &str, contents: impl AsRef<[u8]>) -> PathBuf {
        let path = self.0.join(name);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, contents).unwrap();
        path
    }

    fn spawn(
        &self,
        args: &[&str],
        cols: u16,
        rows: u16,
    ) -> Result<KatPtySession, Box<dyn std::error::Error>> {
        KatPtySession::spawn_in(
            args,
            cols,
            rows,
            &[
                ("KAT_COLOR_MODE", "truecolor"),
                ("KAT_DISABLE_TERMINAL_QUERIES", "1"),
                ("KAT_IMAGE_PROTOCOL", "none"),
            ],
            Some(&self.0),
        )
    }
}

impl Drop for BrowserFixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn browser_open_and_back_keys_share_fullscreen_dracula_chrome() -> TestResult {
    let fixture = BrowserFixture::new("open-and-back");
    fixture.write("file.txt", "first line\nhere is the search target\n");
    fixture.write("other.txt", "other preview\n");
    for (open, back) in [
        (b"l".as_slice(), b"h".as_slice()),
        (b"\x1b[C", b"q"),
        (b"l", b"\x1b[D"),
        (b"\r", b"h"),
    ] {
        let mut session = fixture.spawn(&["."], 100, 12)?;
        session.wait_for_screen(100, 12, |frame| frame.screen_text().contains("hjkl/arrows"))?;
        // o must leave us in the browser so j selects the next file.
        session.write_input(b"oj")?;
        session.wait_for_screen(100, 12, |frame| {
            frame.screen_text().contains("hjkl/arrows")
                && frame.screen[2].starts_with("> ")
                && frame.screen[2].contains("other.txt")
        })?;
        session.write_input(b"k")?;
        session.write_input(open)?;
        let frame =
            session.wait_for_screen(100, 12, |frame| frame.screen_text().contains("q:back"))?;
        assert!(frame.screen[0].contains("file.txt"));
        assert_eq!(frame.screen[1].trim_end(), "first line");
        for y in [0, 11] {
            let style = frame
                .terminal
                .grid_ref(Point::Active(PointCoordinate { x: 0, y }))?
                .style()?;
            let StyleColor::Rgb(bg) = style.bg_color else {
                panic!("expected unified surface")
            };
            assert_eq!((bg.r, bg.g, bg.b), (68, 71, 90));
        }
        session.write_input(b"/h")?;
        session.wait_for_screen(100, 12, |frame| {
            frame.screen.last().unwrap().starts_with("/h")
        })?;
        session.write_input(b"\x1b")?;
        session.wait_for_screen(100, 12, |frame| {
            frame.screen.last().unwrap().contains("q:back")
        })?;
        session.write_input(back)?;
        let frame = session
            .wait_for_screen(100, 12, |frame| frame.screen_text().contains("hjkl/arrows"))?;
        assert!(frame.screen[1].starts_with("> ") && frame.screen[1].contains("file.txt"));
        session.write_input(b"q")?;
        session.wait_success()?;
    }
    Ok(())
}

#[cfg(unix)]
#[test]
fn shell_wrappers_change_directory_only_when_o_opens_a_directory() -> TestResult {
    let fixture = BrowserFixture::new("shell-cd");
    fixture.write("workers with spaces/read.txt", "inside workers\n");
    fixture.write("z.txt", "last file\n");
    let temporary = fixture.0.join("z-temp");
    fs::create_dir_all(&temporary)?;
    let binary_dir = Path::new(env!("CARGO_BIN_EXE_kat")).parent().unwrap();
    let path = format!("{}:{}", binary_dir.display(), std::env::var("PATH")?);
    for shell in ["bash", "zsh", "fish"] {
        if !std::process::Command::new(shell)
            .arg("--version")
            .output()
            .is_ok_and(|result| result.status.success())
        {
            assert_ne!(
                shell, "bash",
                "bash is required for shell integration tests"
            );
            continue;
        }
        for key in [b"o".as_slice(), b"O", b"lO", b"GO", b"/missing\rO", b"q"] {
            let mut command = CommandBuilder::new(shell);
            let script = if shell == "fish" {
                "source $argv[1]; kat $argv[2]; set -l kat_result $status; printf '\\nKAT_CWD=%s\\n' \"$PWD\"; exit $kat_result"
            } else {
                "source \"$1\"; kat \"$2\"; kat_result=$?; printf '\\nKAT_CWD=%s\\n' \"$PWD\"; exit \"$kat_result\""
            };
            if shell == "fish" {
                command.arg("--no-config");
            } else if shell == "bash" {
                command.args(["--noprofile", "--norc"]);
            } else {
                command.arg("-f");
            }
            command.args(["-c", script]);
            if shell != "fish" {
                command.arg("kat-shell-test");
            }
            command.arg(
                Path::new(env!("CARGO_MANIFEST_DIR")).join(if shell == "fish" {
                    "shell/kat.fish"
                } else {
                    "shell/kat.sh"
                }),
            );
            command.arg(&fixture.0);
            let mut session = KatPtySession::spawn_command(
                command,
                100,
                12,
                &[
                    ("PATH", &path),
                    ("TMPDIR", temporary.to_str().unwrap()),
                    ("KAT_COLOR_MODE", "truecolor"),
                    ("KAT_DISABLE_TERMINAL_QUERIES", "1"),
                ],
                Some(&fixture.0),
            )?;
            session
                .wait_for_screen(100, 12, |frame| frame.screen_text().contains("hjkl/arrows"))?;
            session.write_input(key)?;
            let output = session.wait_success()?;
            let expected = if key == b"o" || key == b"lO" {
                fixture.0.join("workers with spaces")
            } else {
                fixture.0.clone()
            };
            let transcript = String::from_utf8_lossy(&output);
            let reported = transcript
                .lines()
                .find_map(|line| line.strip_prefix("KAT_CWD="))
                .unwrap_or_else(|| panic!("{shell}: {transcript}"));
            assert_eq!(
                fs::canonicalize(reported.trim_end_matches('\r'))?,
                fs::canonicalize(expected)?,
                "{shell}: {transcript}"
            );
            assert_eq!(
                fs::read_dir(&temporary)?.count(),
                0,
                "{shell} must clean up its cwd file"
            );
        }
    }
    Ok(())
}

#[test]
fn browser_filters_filenames_with_fzf_ranking_and_highlights_matches() -> TestResult {
    let fixture = BrowserFixture::new("filename-filter");
    fixture.write("abc.txt", "best match preview\n");
    fixture.write("a_b_c.txt", "sparse match preview\n");
    fixture.write("other.txt", "other preview\n");
    fixture.write("dir-abc/inside.txt", "inside directory\n");
    fixture.write("报告🦀.md", "unicode preview\n");
    let mut session = fixture.spawn(&["--icons=never", "."], 100, 12)?;
    session.wait_for_screen(100, 12, |frame| frame.screen_text().contains("hjkl/arrows"))?;
    session.write_input(b"/abc")?;
    let frame = session.wait_for_screen(100, 12, |frame| {
        frame.screen[11].starts_with("/abc") && frame.screen[11].contains("[3/5]")
    })?;
    assert!(
        frame.screen[1].starts_with("> abc.txt"),
        "best match must rank first: {}",
        frame.screen_text()
    );
    assert!(!frame.screen_text().contains("other.txt"));
    let sparse_row = frame
        .screen
        .iter()
        .position(|line| line.contains("a_b_c.txt"))
        .unwrap() as u32;
    for (y, offsets) in [(1, [0, 1, 2]), (sparse_row, [0, 2, 4])] {
        for offset in offsets {
            assert_filename_match(&frame, 2 + offset, y)?;
        }
    }
    let style = frame
        .terminal
        .grid_ref(Point::Active(PointCoordinate { x: 5, y: 1 }))?
        .style()?;
    assert_eq!(
        style.underline,
        Underline::None,
        "unmatched extension must keep its normal style"
    );
    session.write_input(b"\x1b[B")?;
    session.wait_for_screen(100, 12, |frame| {
        frame.screen[2].starts_with("> dir-abc/") && frame.screen[11].starts_with("/abc")
    })?;
    session.write_input(b"\x1b[A")?;
    session.wait_for_screen(100, 12, |frame| frame.screen[1].starts_with("> abc.txt"))?;
    session.write_input(b"\r")?;
    session.wait_for_screen(100, 12, |frame| {
        frame.screen[1].starts_with("best match preview") && frame.screen[11].contains("q:back")
    })?;
    session.write_input(b"h")?;
    session.wait_for_screen(100, 12, |frame| {
        frame.screen[11].contains("hjkl/arrows") && frame.screen[11].contains("/abc")
    })?;
    session.resize(60, 12)?;
    session.wait_for_screen(60, 12, |frame| {
        frame.screen[11].starts_with("kat ") && frame.screen[11].contains("/abc [3/5]")
    })?;
    session.resize(100, 12)?;
    session.wait_for_screen(100, 12, |frame| frame.screen[11].contains("/abc [3/5]"))?;
    fixture.write("abc-new.txt", "new matching file\n");
    session.write_input(b"R")?;
    session.wait_for_screen(100, 12, |frame| frame.screen[11].contains("[4/6]"))?;
    session.write_input(b"\x1b")?;
    session.wait_for_screen(100, 12, |frame| {
        frame.screen[11].contains("hjkl/arrows") && frame.screen_text().contains("other.txt")
    })?;
    session.write_input(b"/abc\rh")?;
    session.wait_for_screen(100, 12, |frame| frame.screen[11].contains("/abc [4/6]"))?;
    session.write_input(b"/\x15zzzz")?;
    session.wait_for_screen(100, 12, |frame| {
        frame.screen[11].starts_with("/zzzz") && frame.screen_text().contains("No matches")
    })?;
    session.write_input(b"\x7f\x7f\x7f\x7f")?;
    session.wait_for_screen(100, 12, |frame| frame.screen[11].contains("[6/6]"))?;
    session.write_input("报告🦀".as_bytes())?;
    let frame = session.wait_for_screen(100, 12, |frame| {
        frame.screen[11]
            .split_whitespace()
            .collect::<String>()
            .starts_with("/报告🦀[1/6]")
    })?;
    for x in [2, 4, 6] {
        assert_filename_match(&frame, x, 1)?;
    }
    session.write_input(b"\x7f")?;
    session.wait_for_screen(100, 12, |frame| {
        frame.screen[11]
            .split_whitespace()
            .collect::<String>()
            .starts_with("/报告[1/6]")
    })?;
    session.write_input(b"\x1b")?;
    session.wait_for_screen(100, 12, |frame| {
        frame.screen[11].contains("hjkl/arrows") && frame.screen_text().contains("other.txt")
    })?;
    session.write_input(b"/dir\r")?;
    session.wait_for_screen(100, 12, |frame| {
        frame.screen[1].starts_with("> inside.txt") && !frame.screen[11].contains("/dir")
    })?;
    session.write_input(b"q")?;
    session.wait_success()?;
    Ok(())
}

fn assert_filename_match(frame: &RenderedTerminal, x: u16, y: u32) -> TestResult {
    let style = frame
        .terminal
        .grid_ref(Point::Active(PointCoordinate { x, y }))?
        .style()?;
    let StyleColor::Rgb(color) = style.fg_color else {
        panic!("expected colored filename match: {style:?}")
    };
    assert_eq!((color.r, color.g, color.b), (241, 250, 140));
    assert_eq!(style.underline, Underline::Single);
    Ok(())
}

#[test]
fn browser_list_width_fits_names_and_preserves_preview_space() -> TestResult {
    let fixture = BrowserFixture::new("adaptive-width");
    fixture.write("a.txt", "short preview\n");
    let mut session = fixture.spawn(&["."], 120, 12)?;
    let frame =
        session.wait_for_screen(120, 12, |frame| frame.screen_text().contains("hjkl/arrows"))?;
    assert_eq!(frame.divider_column(1)?, 16);
    let long_name = "x".repeat(160);
    let long_file = fixture.write(&long_name, "long preview\n");
    session.write_input(b"R")?;
    session.wait_for_screen(120, 12, |frame| frame.divider_column(1).ok() == Some(40))?;
    session.resize(240, 12)?;
    session.wait_for_screen(240, 12, |frame| {
        frame.divider_column(1).ok() == Some(40) && frame.screen_text().contains("short preview")
    })?;
    session.resize(60, 12)?;
    session.wait_for_screen(60, 12, |frame| frame.divider_column(1).ok() == Some(20))?;
    session.resize(120, 12)?;
    session.wait_for_screen(120, 12, |frame| frame.divider_column(1).ok() == Some(40))?;
    fs::remove_file(long_file)?;
    session.write_input(b"R")?;
    session.wait_for_screen(120, 12, |frame| frame.divider_column(1).ok() == Some(16))?;
    session.write_input(b"l")?;
    let frame = session.wait_for_screen(120, 12, |frame| frame.screen_text().contains("q:back"))?;
    assert_eq!(frame.screen[1].trim_end(), "short preview");
    session.write_input(b"q")?;
    let frame =
        session.wait_for_screen(120, 12, |frame| frame.screen_text().contains("hjkl/arrows"))?;
    assert_eq!(frame.divider_column(1)?, 16);
    assert!(frame.screen[1].starts_with("> ") && frame.screen[1].contains("a.txt"));
    session.write_input(b"q")?;
    session.wait_success()?;
    // A direct file invocation has no browser to return to.
    let mut session = fixture.spawn(&["a.txt"], 120, 12)?;
    session.wait_for_screen(120, 12, |frame| frame.screen_text().contains("q:quit"))?;
    session.write_input(b"q")?;
    session.wait_success()?;
    Ok(())
}

#[test]
fn browser_uses_dracula_chrome_and_matching_icons_in_both_panes() -> TestResult {
    let fixture = BrowserFixture::new("dracula");
    fixture.write("nested/deeper/doc.txt", "preview\n");
    fixture.write("nested/code.rs", "fn main() {}\n");
    fixture.write("Cargo.toml", "[package]\n");
    fixture.write("Dockerfile", "FROM scratch\n");
    fixture.write("unknown.kat-unknown", "plain\n");
    let mut session = fixture.spawn(&["."], 100, 12)?;
    let frame =
        session.wait_for_screen(100, 12, |frame| frame.screen_text().contains("hjkl/arrows"))?;
    frame.assert_text_style("nested/", (189, 147, 249), false)?;
    frame.assert_text_style("deeper/", (189, 147, 249), false)?;
    frame.assert_text_style("Cargo.toml", (248, 248, 242), false)?;
    frame.assert_text_style("hjkl/arrows", (139, 233, 253), false)?;
    let left = frame.divider_column(1)?;
    for (x, y, glyph) in [
        (2, 1, '\u{f07b}'),
        (left + 1, 1, '\u{f07b}'),
        (2, 2, ''),
        (left + 1, 2, ''),
        (2, 3, '󰡨'),
        (2, 4, '\u{f15b}'),
    ] {
        let cell = frame
            .terminal
            .grid_ref(Point::Active(PointCoordinate { x, y }))?;
        assert_eq!(
            cell.cell()?.codepoint()?,
            glyph as u32,
            "{x},{y}: {}",
            frame.screen_text()
        );
    }
    for (x, y) in [(0, 0), (0, 11), (0, 1), (left - 1, 1)] {
        let style = frame
            .terminal
            .grid_ref(Point::Active(PointCoordinate { x, y }))?
            .style()?;
        let StyleColor::Rgb(bg) = style.bg_color else {
            panic!("expected Dracula surface: {style:?}")
        };
        assert_eq!((bg.r, bg.g, bg.b), (68, 71, 90));
        assert!(
            !style.inverse,
            "selected items should use a colored surface"
        );
    }
    let divider = frame
        .terminal
        .grid_ref(Point::Active(PointCoordinate { x: left, y: 1 }))?
        .style()?;
    let StyleColor::Rgb(fg) = divider.fg_color else {
        panic!("expected divider color")
    };
    assert_eq!((fg.r, fg.g, fg.b), (98, 114, 164));
    session.write_input(b"l")?;
    let frame = session.wait_for_screen(100, 12, |frame| {
        frame.screen[1].starts_with("> ") && frame.screen[1].contains("deeper/")
    })?;
    frame.assert_text_style("deeper/", (189, 147, 249), false)?;
    session.write_input(b"q")?;
    session.wait_success()?;
    Ok(())
}

#[test]
fn browser_icons_can_be_disabled_without_disabling_directory_colors() -> TestResult {
    let fixture = BrowserFixture::new("icons-disabled");
    fixture.write("nested/subdir/read.txt", "plain\n");
    let mut session = fixture.spawn(&["--icons=never", "."], 100, 12)?;
    let frame =
        session.wait_for_screen(100, 12, |frame| frame.screen_text().contains("hjkl/arrows"))?;
    assert!(frame.screen[1].starts_with("> nested/"));
    assert!(frame.screen[1].contains("│subdir/"));
    assert!(!frame.screen_text().contains('\u{f07b}'));
    frame.assert_text_style("nested/", (189, 147, 249), false)?;
    frame.assert_text_style("subdir/", (189, 147, 249), false)?;
    session.write_input(b"q")?;
    session.wait_success()?;
    Ok(())
}

#[test]
fn browser_theme_respects_no_color_and_ansi_modes() -> TestResult {
    let fixture = BrowserFixture::new("color-modes");
    fixture.write("nested/file.txt", "plain\n");
    for mode in ["none", "ansi"] {
        let mut session = KatPtySession::spawn_in(
            &["."],
            100,
            12,
            &[
                ("KAT_COLOR_MODE", mode),
                ("KAT_DISABLE_TERMINAL_QUERIES", "1"),
            ],
            Some(&fixture.0),
        )?;
        let frame = session
            .wait_for_screen(100, 12, |frame| frame.screen_text().contains("hjkl/arrows"))?;
        let left = frame.divider_column(1)?;
        for (x, y) in [(0, 0), (5, 1), (left, 1), (0, 11)] {
            let style = frame
                .terminal
                .grid_ref(Point::Active(PointCoordinate { x, y }))?
                .style()?;
            assert!(!matches!(style.fg_color, StyleColor::Rgb(_)));
            assert!(!matches!(style.bg_color, StyleColor::Rgb(_)));
            if mode == "none" {
                assert!(matches!(style.fg_color, StyleColor::None));
                assert!(matches!(style.bg_color, StyleColor::None));
            }
        }
        assert!(frame.screen[1].starts_with("> ") && frame.screen[1].contains("nested/"));
        session.write_input(b"q")?;
        session.wait_success()?;
    }
    Ok(())
}

#[test]
fn browser_defaults_to_cwd_and_navigates_directories_with_both_key_sets() -> TestResult {
    let fixture = BrowserFixture::new("navigation");
    fixture.write("nested/inside.txt", "nested preview\n");
    fixture.write("a.rs", "fn main() {}\n");
    fixture.write("z.txt", "last preview\n");
    for args in [vec![], vec!["."]] {
        let mut session = fixture.spawn(&args, 100, 12)?;
        let frame = session
            .wait_for_screen(100, 12, |frame| frame.screen_text().contains("hjkl/arrows"))?;
        assert!(frame.screen[1].starts_with("> ") && frame.screen[1].contains("nested/"));
        assert!(frame.screen[1].contains("inside.txt"));

        session.write_input(b"\r")?;
        session.wait_for_screen(100, 12, |frame| frame.screen[1].contains("nested preview"))?;
        session.write_input(b"h")?;
        session.wait_for_screen(100, 12, |frame| {
            frame.screen[1].starts_with("> ") && frame.screen[1].contains("nested/")
        })?;
        // A split escape sequence must not be mistaken for Escape/quit.
        session.write_input(b"\x1b[")?;
        session.write_input(b"B")?;
        let frame =
            session.wait_for_screen(100, 12, |frame| frame.screen_text().contains("fn main()"))?;
        assert!(frame.screen[2].starts_with("> ") && frame.screen[2].contains("a.rs"));
        frame.assert_text_style("fn", (255, 121, 198), false)?;
        session.write_input(b"j")?;
        session.wait_for_screen(100, 12, |frame| {
            frame.screen_text().contains("last preview")
        })?;
        session.write_input(b"k\x1b[Ak\x1b[C")?;
        session.wait_for_screen(100, 12, |frame| frame.screen[1].contains("nested preview"))?;
        session.write_input(b"\x1b[D")?;
        session.wait_for_screen(100, 12, |frame| {
            frame.screen[1].starts_with("> ") && frame.screen[1].contains("nested/")
        })?;
        session.write_input(b"q")?;
        let output = session.wait_success()?;
        assert_eq!(
            output
                .windows(8)
                .filter(|bytes| *bytes == b"\x1b[?1049h")
                .count(),
            1
        );
        assert_eq!(
            output
                .windows(8)
                .filter(|bytes| *bytes == b"\x1b[?1049l")
                .count(),
            1
        );
    }
    Ok(())
}

#[test]
fn browser_reader_searches_reloads_resizes_and_returns_without_losing_keys() -> TestResult {
    let fixture = BrowserFixture::new("reader");
    let source = (0..40)
        .map(|i| format!("line-{i:03}\n"))
        .collect::<String>();
    let file = fixture.write("read.txt", &source);
    let mut session = fixture.spawn(&["."], 100, 12)?;
    session.wait_for_screen(100, 12, |frame| frame.screen_text().contains("hjkl/arrows"))?;
    session.write_input(b"l/line-035\r")?;
    let frame =
        session.wait_for_screen(100, 12, |frame| frame.screen[1].starts_with("line-035"))?;
    assert!(frame.screen_text().contains("q:back"));
    session.resize(60, 10)?;
    session.wait_for_screen(60, 10, |frame| frame.screen[1].starts_with("line-035"))?;
    fs::write(&file, source.replace("line-035", "changed-035"))?;
    session.write_input(b"R")?;
    session.wait_for_screen(60, 10, |frame| {
        frame.screen_text().contains("Reloaded: changed")
    })?;
    session.write_input(b"q")?;
    let frame = session.wait_for_screen(60, 10, |frame| {
        frame.screen.last().unwrap().contains("hjkl/arrows")
    })?;
    assert!(frame.screen[1].starts_with("> ") && frame.screen[1].contains("read.txt"));
    // All three keys may arrive in one read: open reader, return, exit browser.
    session.write_input(b"lqq")?;
    let output = session.wait_success()?;
    assert_eq!(
        output
            .windows(8)
            .filter(|bytes| *bytes == b"\x1b[?1049h")
            .count(),
        1
    );
    Ok(())
}

#[test]
fn browser_preview_preserves_two_panes_with_wide_names_and_resize() -> TestResult {
    let fixture = BrowserFixture::new("geometry");
    fixture.write(
        "中文文件😀.md",
        "---\ntitle: 中文\n---\n\n```rust\nfn main() {}\n```\n",
    );
    let mut session = fixture.spawn(&["."], 90, 12)?;
    session.wait_for_screen(90, 12, |frame| frame.screen_text().contains("fn main()"))?;
    for (width, left) in [(42, 14), (90, 20)] {
        session.resize(width, 12)?;
        let frame = session.wait_for_screen(width, 12, |frame| {
            frame.screen[1].contains('│')
                && frame.screen_text().contains("title:")
                && frame
                    .terminal
                    .grid_ref(Point::Active(PointCoordinate { x: left, y: 1 }))
                    .and_then(|cell| cell.cell())
                    .and_then(|cell| cell.codepoint())
                    .ok()
                    == Some('│' as u32)
        })?;
        for y in 1..11 {
            let cell = frame
                .terminal
                .grid_ref(Point::Active(PointCoordinate { x: left, y }))?;
            // Query Ghostty's physical cell at the divider, rather than counting
            // Unicode scalar values in a line containing wide glyphs.
            assert_eq!(
                cell.cell()?.codepoint()?,
                '│' as u32,
                "{}",
                frame.screen_text()
            );
        }
    }
    session.resize(10, 2)?;
    session.wait_for_screen(10, 2, |frame| frame.screen_text().contains("Terminal"))?;
    session.resize(90, 12)?;
    session.wait_for_screen(90, 12, |frame| frame.screen_text().contains("fn main()"))?;
    session.write_input(b"q")?;
    session.wait_success()?;
    Ok(())
}

#[test]
fn browser_handles_empty_binary_missing_large_and_special_entries() -> TestResult {
    let fixture = BrowserFixture::new("edge-cases");
    fs::create_dir_all(fixture.0.join("empty"))?;
    let binary = fixture.write("a.bin", b"hello\0world");
    let text = fixture.write("b.txt", "before reload\n");
    let large = fs::File::create(fixture.0.join("c.txt"))?;
    large.set_len(9 * 1024 * 1024)?;
    let mut session = fixture.spawn(&["."], 100, 12)?;
    session.wait_for_screen(100, 12, |frame| {
        frame.screen_text().contains("Empty directory")
    })?;
    session.write_input(b"l")?;
    session.wait_for_screen(100, 12, |frame| frame.screen_text().contains("(empty)"))?;
    session.write_input(b"hj")?;
    session.wait_for_screen(100, 12, |frame| frame.screen_text().contains("Binary file"))?;
    fs::remove_file(binary)?;
    session.write_input(b"R")?;
    session.wait_for_screen(100, 12, |frame| {
        frame.screen_text().contains("before reload")
    })?;
    fs::remove_file(text)?;
    session.write_input(b"l")?;
    session.wait_for_screen(100, 12, |frame| {
        frame.screen_text().contains("Cannot preview")
    })?;
    session.write_input(b"j")?;
    session.wait_for_screen(100, 12, |frame| frame.screen_text().contains("Large file"))?;
    #[cfg(unix)]
    {
        let socket = std::os::unix::net::UnixListener::bind(fixture.0.join("d.sock"))?;
        session.write_input(b"RG")?;
        session.wait_for_screen(100, 12, |frame| {
            frame.screen_text().contains("Special file")
        })?;
        drop(socket);
    }
    session.write_input(b"\x03")?;
    session.wait_success()?;
    Ok(())
}

#[test]
fn browser_images_fall_back_to_information_and_return_from_fullscreen() -> TestResult {
    let fixture = BrowserFixture::new("image");
    // A tiny self-contained SVG exercises image detection without adding assets.
    fixture.write("a.svg", "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"8\" height=\"4\"><rect width=\"8\" height=\"4\" fill=\"red\"/></svg>");
    fixture.write("b.txt", "after image\n");
    let mut session = fixture.spawn(&["."], 100, 12)?;
    session.wait_for_screen(100, 12, |frame| {
        frame.screen_text().contains("image protocol")
    })?;
    session.write_input(b"oj")?;
    session.wait_for_screen(100, 12, |frame| {
        frame.screen[2].starts_with("> ") && frame.screen[2].contains("b.txt")
    })?;
    session.write_input(b"k\r")?;
    session.wait_for_screen(100, 12, |frame| frame.screen_text().contains("q:back"))?;
    session.write_input(b"qj")?;
    session.wait_for_screen(100, 12, |frame| frame.screen_text().contains("after image"))?;
    session.write_input(b"q")?;
    session.wait_success()?;
    Ok(())
}

#[test]
fn browser_kitty_images_are_positioned_in_the_preview_and_cleaned_up() -> TestResult {
    let fixture = BrowserFixture::new("kitty");
    fixture.write("a.svg", "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"8\" height=\"4\"><rect width=\"8\" height=\"4\" fill=\"red\"/></svg>");
    fixture.write("b.txt", "after kitty image\n");
    let mut session = KatPtySession::spawn_in(
        &["."],
        100,
        12,
        &[
            ("KAT_COLOR_MODE", "truecolor"),
            ("KAT_DISABLE_TERMINAL_QUERIES", "1"),
            ("KAT_IMAGE_PROTOCOL", "kitty"),
        ],
        Some(&fixture.0),
    )?;
    let frame =
        session.wait_for_screen(100, 12, |frame| frame.screen_text().contains("hjkl/arrows"))?;
    let raw = String::from_utf8_lossy(&frame.raw_output);
    assert!(
        raw.contains(&format!(
            "\x1b[2;{}H\x1b_Ga=T,",
            frame.divider_column(1)? + 2
        )),
        "{raw}"
    );
    assert!(raw.contains(",i=4931924,"), "{raw}");
    session.write_input(b"j")?;
    session.wait_for_screen(100, 12, |frame| {
        frame.screen_text().contains("after kitty image")
    })?;
    session.write_input(b"q")?;
    let raw = session.wait_success()?;
    assert!(raw.ends_with(b"\x1b[0m\x1b[?25h\x1b[?1049l"));
    assert!(String::from_utf8_lossy(&raw).contains("a=d,d=I,i=4931924"));
    Ok(())
}

#[test]
fn browser_scrolls_the_list_and_keeps_its_position_after_reading() -> TestResult {
    let fixture = BrowserFixture::new("scroll");
    for i in 0..30 {
        fixture.write(&format!("{i:02}.txt"), format!("content-{i:02}\n"));
    }
    let mut session = fixture.spawn(&["."], 100, 8)?;
    session.wait_for_screen(100, 8, |frame| frame.screen_text().contains("content-00"))?;
    session.write_input(b"\x1b[6~")?;
    session.wait_for_screen(100, 8, |frame| frame.screen_text().contains("content-06"))?;
    session.write_input(b"G")?;
    let frame =
        session.wait_for_screen(100, 8, |frame| frame.screen_text().contains("content-29"))?;
    assert!(frame.screen[6].starts_with("> ") && frame.screen[6].contains("29.txt"));
    session.write_input(b"lq")?;
    let frame = session.wait_for_screen(100, 8, |frame| {
        frame.screen_text().contains("hjkl/arrows")
            && frame.screen[6].starts_with("> ")
            && frame.screen[6].contains("29.txt")
    })?;
    assert!(frame.screen[1].starts_with("  ") && frame.screen[1].contains("24.txt"));
    session.write_input(b"\x1b[5~g")?;
    session.wait_for_screen(100, 8, |frame| {
        frame.screen[1].starts_with("> ") && frame.screen[1].contains("00.txt")
    })?;
    session.write_input(b"q")?;
    session.wait_success()?;
    Ok(())
}

#[test]
fn kat_renders_tmtheme_as_xml() -> TestResult {
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("testdata/fixtures/xml/theme.tmTheme");
    let mut session = KatPtySession::spawn(
        &[fixture.to_str().unwrap()],
        120,
        32,
        &[("COLORTERM", "truecolor")],
    )?;
    let rendered = session.wait_for_screen(120, 32, |rendered| {
        rendered.screen_text().contains("q:quit")
    })?;
    assert!(rendered.screen_text().contains("#F8F8F2"));
    for (text, expected, italic) in [
        ("DOCTYPE", (255, 121, 198), false),
        ("<key>", (255, 121, 198), false),
        ("Default colors", (98, 114, 164), true),
        ("&amp;", (189, 147, 249), false),
    ] {
        rendered.assert_text_style(text, expected, italic)?;
    }
    session.write_input(b"q")?;
    session.wait_success()?;
    Ok(())
}

#[test]
fn kat_renders_pkl_and_hk_commands_with_host_and_shell_styles() -> TestResult {
    for (file, styles) in [
        (
            "config.pkl",
            vec![
                ("class", (255, 121, 198)),
                ("UInt16", (139, 233, 253)),
                ("8080", (255, 184, 108)),
            ],
        ),
        (
            "hk.local.pkl",
            vec![
                ("echo", (80, 250, 123)),
                ("files", (189, 147, 249)),
                ("set", (139, 233, 253)),
            ],
        ),
    ] {
        let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("testdata/fixtures/pkl")
            .join(file);
        let mut session = KatPtySession::spawn(
            &[fixture.to_str().unwrap()],
            120,
            32,
            &[("COLORTERM", "truecolor")],
        )?;
        let rendered = session.wait_for_screen(120, 32, |rendered| {
            rendered.screen_text().contains("q:quit")
        })?;
        for (text, expected) in styles {
            rendered.assert_text_style(text, expected, text == "files")?;
        }
        session.write_input(b"q")?;
        session.wait_success()?;
    }
    Ok(())
}

#[test]
fn kat_renders_public_key_comments_and_base64_suffixes_consistently() -> TestResult {
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("testdata/fixtures/authorized_keys/id_ed25519.pub");
    let mut session = KatPtySession::spawn(
        &[fixture.to_str().unwrap()],
        100,
        ROWS,
        &[("COLORTERM", "truecolor")],
    )?;
    let rendered = session.wait_for_screen(100, ROWS, |rendered| {
        rendered.screen_text().contains("q:quit")
    })?;
    rendered.assert_text_style("ssh-ed25519", (139, 233, 253), false)?;
    rendered.assert_text_style("dcjanusmacbook-pro tailnet", (98, 114, 164), true)?;
    rendered.assert_text_style("AAAAC3NzaC1lZDI1NTE5AAAAIExample/8", (189, 147, 249), false)?;
    session.write_input(b"q")?;
    session.wait_success()?;
    Ok(())
}

#[test]
fn kat_auto_detects_known_hosts_and_renders_revocation_warning() -> TestResult {
    let fixture =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("testdata/fixtures/known_hosts/known_hosts");
    let mut session = KatPtySession::spawn(
        &[fixture.to_str().unwrap()],
        120,
        ROWS,
        &[("COLORTERM", "truecolor")],
    )?;
    let rendered = session.wait_for_screen(120, ROWS, |rendered| {
        rendered.screen_text().contains("q:quit")
    })?;
    for (needle, expected, bold) in [
        ("@revoked", (255, 184, 108), true),
        ("example.com", (139, 233, 253), false),
        ("|1|c2FsdA==|aGFzaA==", (139, 233, 253), false),
        ("# Public host keys", (98, 114, 164), false),
    ] {
        let y = rendered.line_index(needle);
        let line = &rendered.screen[y];
        assert!(line.starts_with(needle));
        let style = rendered
            .terminal
            .grid_ref(Point::Active(PointCoordinate { x: 0, y: y as u32 }))?
            .style()?;
        let StyleColor::Rgb(color) = style.fg_color else {
            panic!("expected RGB syntax color for {needle}: {style:?}");
        };
        assert_eq!((color.r, color.g, color.b), expected, "{needle}");
        assert_eq!(style.bold, bold, "{needle}");
    }
    session.write_input(b"q")?;
    session.wait_success()?;
    Ok(())
}

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
fn kat_keeps_narrow_markdown_frontmatter_rows_adjacent() -> TestResult {
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("testdata/fixtures/markdown/frontmatter-narrow.md");
    let mut session = KatPtySession::spawn(
        &[
            "--hyperlinks=never",
            fixture.to_str().expect("fixture path should be UTF-8"),
        ],
        48,
        ROWS,
        &[("COLORTERM", "truecolor")],
    )?;
    let rendered = session.wait_for_screen(48, ROWS, |rendered| {
        rendered.has_rect_block_background("---", 47, 5)
    })?;
    session.write_input(b"q")?;
    session.wait_success()?;

    rendered.assert_rect_block_background("---", 47, 5)?;

    Ok(())
}

#[test]
fn kat_relayouts_markdown_frontmatter_when_terminal_grows() -> TestResult {
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("testdata/fixtures/markdown/frontmatter-narrow.md");
    let mut session = KatPtySession::spawn(
        &[
            "--hyperlinks=never",
            fixture.to_str().expect("fixture path should be UTF-8"),
        ],
        48,
        ROWS,
        &[("COLORTERM", "truecolor")],
    )?;
    session.wait_for_screen(48, ROWS, |rendered| {
        rendered.has_rect_block_background("---", 47, 5)
    })?;

    session.resize(80, ROWS)?;
    let rendered = session.wait_for_screen(80, ROWS, |rendered| {
        rendered.has_rect_block_background("---", 69, 4)
    })?;
    rendered.assert_rect_block_background("---", 69, 4)?;

    session.resize(48, ROWS)?;
    let rendered = session.wait_for_screen(48, ROWS, |rendered| {
        rendered.has_rect_block_background("---", 47, 5)
    })?;
    rendered.assert_rect_block_background("---", 47, 5)?;

    session.write_input(b"q")?;
    session.wait_success()?;

    Ok(())
}

#[test]
fn kat_relayouts_markdown_frontmatter_when_terminal_shrinks() -> TestResult {
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("testdata/fixtures/markdown/frontmatter-narrow.md");
    let mut session = KatPtySession::spawn(
        &[
            "--hyperlinks=never",
            fixture.to_str().expect("fixture path should be UTF-8"),
        ],
        80,
        ROWS,
        &[("COLORTERM", "truecolor")],
    )?;
    session.wait_for_screen(80, ROWS, |rendered| {
        rendered.has_rect_block_background("---", 69, 4)
    })?;

    session.resize(32, ROWS)?;
    let rendered = session.wait_for_screen(32, ROWS, |rendered| {
        rendered.has_rect_block_background("---", 32, 6)
    })?;
    rendered.assert_rect_block_background("---", 32, 6)?;

    session.write_input(b"q")?;
    session.wait_success()?;

    Ok(())
}

#[test]
fn kat_relayouts_realistic_frontmatter_between_half_and_full_width() -> TestResult {
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("testdata/fixtures/markdown/frontmatter-realistic-resize.md");
    let mut session = KatPtySession::spawn(
        &[
            "--hyperlinks=never",
            fixture.to_str().expect("fixture path should be UTF-8"),
        ],
        115,
        65,
        &[("COLORTERM", "truecolor")],
    )?;
    let initial_half = session.wait_for_screen(115, 65, |rendered| {
        rendered.has_rect_block_background("---", 114, 5)
    })?;

    session.resize(232, 65)?;
    let resized_full = session.wait_for_screen(232, 65, |rendered| {
        rendered.has_rect_block_background("---", 132, 4)
    })?;
    resized_full.assert_rect_block_background("---", 132, 4)?;
    let direct_full = render_path_in_ghostty(&fixture, 232, 65, "never")?;
    assert_eq!(
        resized_full.screen, direct_full.screen,
        "expected resize-to-full layout to equal a direct full-width render"
    );

    session.resize(115, 65)?;
    let roundtrip_half = session.wait_for_screen(115, 65, |rendered| {
        rendered.has_rect_block_background("---", 114, 5)
    })?;
    roundtrip_half.assert_rect_block_background("---", 114, 5)?;
    assert_eq!(
        roundtrip_half.screen, initial_half.screen,
        "expected full-width roundtrip to recover the direct half-width layout"
    );

    session.write_input(b"q")?;
    session.wait_success()?;

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
fn kat_builtin_pager_reloads_files_and_reports_success_and_failure() -> TestResult {
    let fixture = temp_plain_fixture("ghostty-reload", "before reload\n")?;
    let mut session = KatPtySession::spawn(
        &["--hyperlinks=never", fixture.to_str().unwrap()],
        80,
        ROWS,
        &[],
    )?;
    session.wait_for_screen(80, ROWS, |screen| {
        screen.screen_text().contains("before reload")
    })?;
    fs::write(&fixture, "after reload\n")?;
    session.write_input(b"R")?;
    let screen = session.wait_for_screen(80, ROWS, |screen| {
        screen.screen_text().contains("Reloaded: changed")
    })?;
    screen.assert_screen_contains("after reload");
    assert!(!screen.screen_text().contains("before reload"));

    fs::remove_file(&fixture)?;
    session.write_input(b"R")?;
    let screen = session.wait_for_screen(80, ROWS, |screen| {
        screen.screen_text().contains("Reload failed:")
    })?;
    screen.assert_screen_contains("after reload");

    fs::write(&fixture, "recovered\n")?;
    // The navigation event following R in the same read must also be processed.
    session.write_input(b"Rg")?;
    let screen = session.wait_for_screen(80, ROWS, |screen| {
        screen.screen_text().contains("recovered") && screen.screen_text().contains("q:quit")
    })?;
    assert!(!screen.screen_text().contains("Reload failed:"));
    session.write_input(b"R")?;
    session.wait_for_screen(80, ROWS, |screen| {
        screen.screen_text().contains("Reloaded: unchanged")
    })?;
    // The status must recover without another key press.
    let screen = session.wait_for_screen(80, ROWS, |screen| {
        screen.screen_text().contains("kat 1/1") && !screen.screen_text().contains("Reloaded:")
    })?;
    screen.assert_screen_contains("recovered");
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
fn kat_builtin_pager_handles_burst_and_split_cursor_input() -> TestResult {
    let fixture = temp_plain_fixture(
        "ghostty-builtin-pager-input-stream",
        &(1..=100)
            .map(|line| format!("SCROLL-LINE-{line:03}\n"))
            .collect::<String>(),
    )?;
    let mut session = KatPtySession::spawn(
        &[fixture.to_str().expect("UTF-8 fixture path")],
        COLS,
        ROWS,
        &[],
    )?;
    session.wait_for_screen(COLS, ROWS, |screen| {
        screen.screen_text().contains("SCROLL-LINE-001")
    })?;

    session.write_input(&b"\x1b[B".repeat(10))?;
    let screen = session.wait_for_screen(COLS, ROWS, |screen| {
        screen.screen_text().lines().nth(1).map(str::trim_end) == Some("SCROLL-LINE-011")
    })?;
    screen.assert_active_screen(Screen::Alternate)?;

    session.write_input(b"\x1b")?;
    thread::sleep(Duration::from_millis(30));
    session.write_input(b"[")?;
    thread::sleep(Duration::from_millis(30));
    session.write_input(b"B")?;
    let screen = session.wait_for_screen(COLS, ROWS, |screen| {
        screen.screen_text().lines().nth(1).map(str::trim_end) == Some("SCROLL-LINE-012")
    })?;
    screen.assert_active_screen(Screen::Alternate)?;

    session.write_input(b"G")?;
    session.wait_for_screen(COLS, ROWS, |screen| {
        screen.screen_text().contains("SCROLL-LINE-100")
    })?;
    session.write_input(&b"\x1b[B".repeat(100))?;
    session.write_input(b"g")?;
    let screen = session.wait_for_screen(COLS, ROWS, |screen| {
        screen.screen_text().lines().nth(1).map(str::trim_end) == Some("SCROLL-LINE-001")
    })?;
    screen.assert_active_screen(Screen::Alternate)?;
    session.write_input(b"q")?;
    session.wait_success()?;
    Ok(())
}

#[test]
fn kat_builtin_pager_supports_less_style_half_page_navigation() -> TestResult {
    let fixture = temp_plain_fixture(
        "ghostty-builtin-pager-half-page-navigation",
        &(1..=40)
            .map(|line| format!("HALF-PAGE-LINE-{line:02}\n"))
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
        rendered
            .screen
            .get(1)
            .is_some_and(|line| line.trim_end() == "HALF-PAGE-LINE-01")
    })?;

    session.write_input(b"\x04")?;
    let rendered = session.wait_for_screen(COLS, ROWS, |rendered| {
        rendered
            .screen
            .get(1)
            .is_some_and(|line| line.trim_end() == "HALF-PAGE-LINE-06")
    })?;
    assert_eq!(rendered.screen[1].trim_end(), "HALF-PAGE-LINE-06");

    session.write_input(b"\x15")?;
    let rendered = session.wait_for_screen(COLS, ROWS, |rendered| {
        rendered
            .screen
            .get(1)
            .is_some_and(|line| line.trim_end() == "HALF-PAGE-LINE-01")
    })?;
    assert_eq!(rendered.screen[1].trim_end(), "HALF-PAGE-LINE-01");

    session.write_input(b"q")?;
    session.wait_success()?;

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
            .get(1)
            .is_some_and(|line| line.contains("ANCHOR-LINE-03"))
    })?;
    rendered.assert_screen_contains("ANCHOR-LINE-03 alpha");

    session.resize(56, 8)?;
    let rendered = session.wait_for_screen(56, 8, |rendered| {
        rendered
            .screen
            .get(1)
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
    fn divider_column(&self, y: u32) -> Result<u16, Box<dyn std::error::Error>> {
        for x in 0..self.terminal.cols()? {
            let cell = self
                .terminal
                .grid_ref(Point::Active(PointCoordinate { x, y }))?;
            if cell.cell()?.codepoint()? == '│' as u32 {
                return Ok(x);
            }
        }
        Err("missing browser pane divider".into())
    }

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

    fn assert_text_style(
        &self,
        needle: &str,
        foreground: (u8, u8, u8),
        italic: bool,
    ) -> TestResult {
        let mut state = RenderState::new()?;
        let snapshot = state.update(&self.terminal)?;
        let mut rows = RowIterator::new()?;
        let mut cells = CellIterator::new()?;
        let mut row_iter = rows.update(&snapshot)?;
        let mut found = false;
        while let Some(row) = row_iter.next() {
            let mut text = String::new();
            let mut styles = Vec::new();
            let mut cell_iter = cells.update(row)?;
            while let Some(cell) = cell_iter.next() {
                let mut grapheme = String::new();
                cell.graphemes_utf8(&mut grapheme)?;
                if grapheme.is_empty() {
                    grapheme.push(' ');
                }
                // These are UTF-8 offsets into collected cell text, not display columns.
                let start = text.len();
                text.push_str(&grapheme);
                styles.push((start..text.len(), cell.style()?));
            }
            for (start, _) in text.match_indices(needle) {
                found = true;
                let end = start + needle.len();
                for (_, style) in styles
                    .iter()
                    .filter(|(range, _)| range.start < end && range.end > start)
                {
                    let StyleColor::Rgb(color) = style.fg_color else {
                        panic!("expected RGB color for {needle}: {style:?}");
                    };
                    assert_eq!((color.r, color.g, color.b), foreground, "{needle}");
                    assert_eq!(style.italic, italic, "{needle}");
                }
            }
        }
        assert!(
            found,
            "expected {needle} in Ghostty screen:\n{}",
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

    fn assert_rect_block_background(
        &self,
        delimiter: &str,
        expected_width: u16,
        expected_rows: usize,
    ) -> TestResult {
        let delimiter_rows = self
            .screen
            .iter()
            .enumerate()
            .filter_map(|(index, line)| line.contains(delimiter).then_some(index))
            .collect::<Vec<_>>();
        assert!(
            delimiter_rows.len() >= 2,
            "expected two {delimiter:?} rows around the block:\n{}",
            self.screen_text()
        );
        let first = delimiter_rows[0];
        let last = delimiter_rows[1];
        assert_eq!(
            last - first + 1,
            expected_rows,
            "unexpected wrapped row count for block:\n{}",
            self.screen_text()
        );

        for row in first..=last {
            assert_eq!(
                self.rgb_background_width(row),
                Some(expected_width),
                "expected block row {row} to share a {expected_width}-column background:\n{}",
                self.screen_text()
            );
        }
        Ok(())
    }

    fn has_rect_block_background(
        &self,
        delimiter: &str,
        expected_width: u16,
        expected_rows: usize,
    ) -> bool {
        let delimiter_rows = self
            .screen
            .iter()
            .enumerate()
            .filter_map(|(index, line)| line.contains(delimiter).then_some(index))
            .take(2)
            .collect::<Vec<_>>();
        let [first, last] = delimiter_rows.as_slice() else {
            return false;
        };
        last - first + 1 == expected_rows
            && (*first..=*last).all(|row| self.rgb_background_width(row) == Some(expected_width))
    }

    fn rgb_background_width(&self, row: usize) -> Option<u16> {
        let cols = self.terminal.cols().ok()?;
        let mut width = 0;
        for x in 0..cols {
            let style = self
                .terminal
                .grid_ref(Point::Active(PointCoordinate { x, y: row as u32 }))
                .ok()?
                .style()
                .ok()?;
            if !matches!(style.bg_color, StyleColor::Rgb(_)) {
                break;
            }
            width = x + 1;
        }
        Some(width)
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
    let mut terminal = Terminal::new(cols, rows).expect("Ghostty terminal should initialize");
    terminal
        .set_scrollback_max_lines(Some(100))
        .expect("Ghostty scrollback limit should initialize");
    terminal
}

struct KatPtySession {
    master: Box<dyn MasterPty>,
    writer: Box<dyn Write + Send>,
    child: Box<dyn Child + Send + Sync>,
    chunks: Receiver<Vec<u8>>,
    reader_thread: Option<thread::JoinHandle<std::io::Result<()>>>,
    output: Vec<u8>,
    foreground_query_answered: bool,
    background_query_answered: bool,
}

impl KatPtySession {
    fn spawn(
        args: &[&str],
        cols: u16,
        rows: u16,
        envs: &[(&str, &str)],
    ) -> Result<Self, Box<dyn std::error::Error>> {
        Self::spawn_in(args, cols, rows, envs, None)
    }

    fn spawn_in(
        args: &[&str],
        cols: u16,
        rows: u16,
        envs: &[(&str, &str)],
        cwd: Option<&Path>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let mut command = CommandBuilder::new(env!("CARGO_BIN_EXE_kat"));
        command.args(args);
        Self::spawn_command(command, cols, rows, envs, cwd)
    }

    fn spawn_command(
        mut command: CommandBuilder,
        cols: u16,
        rows: u16,
        envs: &[(&str, &str)],
        cwd: Option<&Path>,
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

        if let Some(cwd) = cwd {
            command.cwd(cwd);
        }
        command.env("TERM", "xterm-256color");
        command.env_remove("COLORTERM");
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
            foreground_query_answered: false,
            background_query_answered: false,
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
            self.answer_dynamic_color_queries()?;

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

    fn answer_dynamic_color_queries(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if !self.foreground_query_answered && contains_bytes(&self.output, FOREGROUND_QUERY) {
            self.writer.write_all(FOREGROUND_RESPONSE)?;
            self.writer.flush()?;
            self.foreground_query_answered = true;
        }
        if !self.background_query_answered && contains_bytes(&self.output, BACKGROUND_QUERY) {
            self.writer.write_all(BACKGROUND_RESPONSE)?;
            self.writer.flush()?;
            self.background_query_answered = true;
        }
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
        // The child has exited, but its final PTY bytes may still be in transit.
        // Drain until the reader closes the channel rather than treating a
        // short scheduling pause as end-of-output.
        while let Ok(chunk) = self.chunks.recv() {
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

fn contains_bytes(haystack: &[u8], needle: &[u8]) -> bool {
    haystack
        .windows(needle.len())
        .any(|window| window == needle)
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

#[test]
fn colored_long_json_uses_the_same_rows_for_painting_navigation_and_resize() -> TestResult {
    // Entirely synthetic: no model configuration or private prompt content.
    let payload = (0..240).map(|i| format!("item{i:04} ")).collect::<String>();
    let source = format!("{{\"instructions_template\":\"{payload}\"}}\n\n");
    let fixture = std::env::temp_dir().join(format!(
        "kat-viewer-long-string-{}.json",
        std::process::id()
    ));
    fs::write(&fixture, &source)?;
    for cols in [39_u16, 40, 41] {
        let expected = |width: u16| {
            source
                .split_terminator('\n')
                .flat_map(|line| {
                    if line.is_empty() {
                        return vec![String::new()];
                    }
                    line.as_bytes()
                        .chunks(usize::from(width))
                        .map(|chunk| String::from_utf8(chunk.to_vec()).unwrap())
                        .collect()
                })
                .collect::<Vec<_>>()
        };
        let check =
            |session: &mut KatPtySession, width: u16, height: u16, top: usize| -> TestResult {
                let rows = expected(width);
                let body = usize::from(height) - 2;
                let status = format!("kat {}/{} ", (top + body).min(rows.len()), rows.len());
                let rendered = session.wait_for_screen(width, height, |frame| {
                    frame
                        .screen
                        .last()
                        .is_some_and(|line| line.starts_with(&status))
                        && (0..body).all(|i| {
                            frame.screen[i + 1].trim_end()
                                == rows
                                    .get(top + i)
                                    .map(String::as_str)
                                    .unwrap_or("")
                                    .trim_end()
                        })
                })?;
                for i in 0..body {
                    assert_eq!(
                        rendered.screen[i + 1].trim_end(),
                        rows.get(top + i)
                            .map(String::as_str)
                            .unwrap_or("")
                            .trim_end(),
                        "display row {}",
                        top + i
                    );
                }
                assert!(
                    rendered
                        .raw_output
                        .windows(b"\x1b[38;2".len())
                        .any(|w| w == b"\x1b[38;2")
                );
                Ok(())
            };
        let mut session = KatPtySession::spawn(
            &["--hyperlinks=never", fixture.to_str().unwrap()],
            cols,
            17,
            &[
                ("KAT_COLOR_MODE", "truecolor"),
                ("KAT_DISABLE_TERMINAL_QUERIES", "1"),
            ],
        )?;
        check(&mut session, cols, 17, 0)?;
        let end = expected(cols).len() - 15;
        session.write_input(b"G")?;
        check(&mut session, cols, 17, end)?;
        let top = end - 18;
        session.write_input(&b"k".repeat(18))?;
        check(&mut session, cols, 17, top)?;
        session.resize(cols, 9)?;
        check(&mut session, cols, 9, top)?;
        let wider = cols + 7;
        let new_top = top * usize::from(cols) / usize::from(wider);
        session.resize(wider, 13)?;
        check(&mut session, wider, 13, new_top)?;
        session.write_input(b"/item0020\r")?;
        let match_row = source.find("item0020").unwrap() / usize::from(wider);
        check(&mut session, wider, 13, match_row)?;
        session.write_input(b"q")?;
        session.wait_success()?;
    }
    fs::remove_file(fixture)?;
    Ok(())
}

#[test]
fn wide_search_status_does_not_wrap_or_scroll_the_body() -> TestResult {
    let source = (0..80)
        .map(|i| format!("line-{i:03}\n"))
        .collect::<String>();
    let fixture = temp_plain_fixture("kat-wide-search-status", &source)?;
    let mut session = KatPtySession::spawn(
        &[fixture.to_str().unwrap()],
        40,
        17,
        &[
            ("KAT_COLOR_MODE", "truecolor"),
            ("KAT_DISABLE_TERMINAL_QUERIES", "1"),
        ],
    )?;
    session.wait_for_screen(40, 17, |frame| {
        frame
            .screen
            .last()
            .is_some_and(|line| line.starts_with("kat "))
    })?;
    session.write_input(format!("/{}", "中".repeat(40)).as_bytes())?;
    let rendered = session.wait_for_screen(40, 17, |frame| {
        frame.screen.last().is_some_and(|line| line.contains("中"))
    })?;
    assert_eq!(
        rendered.screen[1].trim_end(),
        "line-000",
        "{}",
        rendered.screen_text()
    );
    assert_eq!(rendered.screen[15].trim_end(), "line-014");
    assert_eq!(
        rendered.screen[16].replace(' ', ""),
        format!("/{}", "中".repeat(19))
    );
    session.write_input(b"\x03")?;
    session.wait_success()?;
    fs::remove_file(fixture)?;
    Ok(())
}

#[test]
fn source_control_characters_cannot_reposition_viewer_rows() -> TestResult {
    let source = "safe \x1b[2Jcontrol\nleft\rright\nlast\n";
    let fixture = temp_plain_fixture("kat-source-controls", source)?;
    let mut session = KatPtySession::spawn(
        &[fixture.to_str().unwrap()],
        40,
        8,
        &[
            ("KAT_COLOR_MODE", "truecolor"),
            ("KAT_DISABLE_TERMINAL_QUERIES", "1"),
        ],
    )?;
    let rendered = session.wait_for_screen(40, 8, |frame| {
        frame
            .screen
            .last()
            .is_some_and(|line| line.starts_with("kat "))
    })?;
    assert_eq!(rendered.screen[1].trim_end(), "safe ␛[2Jcontrol");
    assert_eq!(rendered.screen[2].trim_end(), "left␍right");
    assert_eq!(rendered.screen[3].trim_end(), "last");
    session.write_input(b"q")?;
    session.wait_success()?;
    fs::remove_file(fixture)?;
    Ok(())
}

#[test]
fn expanded_tabs_fill_narrow_viewer_rows() -> TestResult {
    let fixture = temp_plain_fixture("kat-narrow-tabs", "abcd\t012\n")?;
    let mut session = KatPtySession::spawn(
        &[fixture.to_str().unwrap()],
        3,
        8,
        &[
            ("KAT_COLOR_MODE", "truecolor"),
            ("KAT_DISABLE_TERMINAL_QUERIES", "1"),
        ],
    )?;
    let rendered = session.wait_for_screen(3, 8, |frame| {
        frame.screen.last().is_some_and(|line| line == "kat")
    })?;
    assert_eq!(&rendered.screen[1..5], &["abc", "d  ", "  0", "12 "]);
    session.write_input(b"q")?;
    session.wait_success()?;
    fs::remove_file(fixture)?;
    Ok(())
}
