use std::{
    io::{self, IsTerminal, Read, Write},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

use anyhow::{Context, Result};
use signal_hook::{consts::signal::SIGWINCH, flag};

const ENTER_ALTERNATE_SCREEN: &[u8] = b"\x1b[?1049h";
const LEAVE_ALTERNATE_SCREEN: &[u8] = b"\x1b[?1049l";
const CLEAR_SCREEN: &[u8] = b"\x1b[H\x1b[2J";
const CLEAR_LINE: &[u8] = b"\x1b[2K";
const RESET_STYLE: &[u8] = b"\x1b[0m";

pub(crate) fn run(output: &str) -> Result<()> {
    if !io::stdin().is_terminal() || !io::stdout().is_terminal() {
        return write_direct(output);
    }

    let _terminal = TerminalSession::enter()?;
    let resize_requested = Arc::new(AtomicBool::new(true));
    flag::register(SIGWINCH, resize_requested.clone()).context("failed to register SIGWINCH")?;

    let document = PagerDocument::new(output);
    let mut state = PagerState::default();
    let mut stdin = io::stdin().lock();

    loop {
        if resize_requested.swap(false, Ordering::Relaxed) {
            state.clamp(document.line_count());
            render(&document, state.top_line, terminal_size())?;
        }

        let mut buf = [0_u8; 16];
        let len = stdin.read(&mut buf).context("failed to read pager input")?;
        if len == 0 {
            continue;
        }

        let Some(action) = PagerAction::parse(&buf[..len]) else {
            continue;
        };
        if matches!(action, PagerAction::Quit) {
            return Ok(());
        }

        state.apply(action, document.line_count(), terminal_size().rows);
        render(&document, state.top_line, terminal_size())?;
    }
}

fn write_direct(output: &str) -> Result<()> {
    let mut stdout = io::stdout().lock();
    stdout
        .write_all(output.as_bytes())
        .context("failed to write output")?;
    stdout.flush().context("failed to flush stdout")?;
    Ok(())
}

struct PagerDocument {
    lines: Vec<String>,
}

impl PagerDocument {
    fn new(output: &str) -> Self {
        let mut lines = output
            .split('\n')
            .map(str::to_owned)
            .collect::<Vec<String>>();
        if output.ends_with('\n') {
            lines.pop();
        }
        Self { lines }
    }

    fn line_count(&self) -> usize {
        self.lines.len()
    }
}

#[derive(Default)]
struct PagerState {
    top_line: usize,
}

impl PagerState {
    fn apply(&mut self, action: PagerAction, line_count: usize, rows: usize) {
        let page = rows.saturating_sub(1).max(1);
        match action {
            PagerAction::Quit => {}
            PagerAction::LineDown => self.top_line = self.top_line.saturating_add(1),
            PagerAction::LineUp => self.top_line = self.top_line.saturating_sub(1),
            PagerAction::PageDown => self.top_line = self.top_line.saturating_add(page),
            PagerAction::PageUp => self.top_line = self.top_line.saturating_sub(page),
            PagerAction::Home => self.top_line = 0,
            PagerAction::End => self.top_line = line_count.saturating_sub(page),
        }
        self.clamp(line_count);
    }

    fn clamp(&mut self, line_count: usize) {
        self.top_line = self.top_line.min(line_count.saturating_sub(1));
    }
}

#[derive(Clone, Copy)]
enum PagerAction {
    Quit,
    LineDown,
    LineUp,
    PageDown,
    PageUp,
    Home,
    End,
}

impl PagerAction {
    fn parse(input: &[u8]) -> Option<Self> {
        match input {
            [b'q' | b'Q'] | [0x03] | [0x1b] => Some(Self::Quit),
            [b'j'] => Some(Self::LineDown),
            [b'k'] => Some(Self::LineUp),
            [b' '] => Some(Self::PageDown),
            [b'b'] => Some(Self::PageUp),
            [b'g'] => Some(Self::Home),
            [b'G'] => Some(Self::End),
            [0x1b, b'[', b'A', ..] => Some(Self::LineUp),
            [0x1b, b'[', b'B', ..] => Some(Self::LineDown),
            [0x1b, b'[', b'H', ..] => Some(Self::Home),
            [0x1b, b'[', b'F', ..] => Some(Self::End),
            [0x1b, b'[', b'5', b'~', ..] => Some(Self::PageUp),
            [0x1b, b'[', b'6', b'~', ..] => Some(Self::PageDown),
            _ => None,
        }
    }
}

#[derive(Clone, Copy)]
struct PagerSize {
    cols: usize,
    rows: usize,
}

fn terminal_size() -> PagerSize {
    terminal_size::terminal_size()
        .map(
            |(terminal_size::Width(cols), terminal_size::Height(rows))| PagerSize {
                cols: usize::from(cols).max(1),
                rows: usize::from(rows).max(1),
            },
        )
        .unwrap_or(PagerSize { cols: 80, rows: 24 })
}

fn render(document: &PagerDocument, top_line: usize, size: PagerSize) -> Result<()> {
    let body_rows = size.rows.saturating_sub(1);
    let mut stdout = io::stdout().lock();
    stdout.write_all(CLEAR_SCREEN)?;

    for line in document.lines.iter().skip(top_line).take(body_rows) {
        stdout.write_all(line.as_bytes())?;
        stdout.write_all(RESET_STYLE)?;
        stdout.write_all(b"\r\n")?;
    }

    write!(stdout, "\x1b[{};1H", size.rows)?;
    stdout.write_all(RESET_STYLE)?;
    stdout.write_all(CLEAR_LINE)?;

    let end_line = if document.line_count() == 0 {
        0
    } else {
        (top_line + body_rows).min(document.line_count())
    };
    let mut status = format!(
        "kat {}/{}  q:quit  j/k:line  PgUp/PgDn:page  Home/End",
        end_line,
        document.line_count()
    );
    if status.chars().count() > size.cols {
        status = status.chars().take(size.cols).collect();
    }
    stdout.write_all(b"\x1b[7m")?;
    stdout.write_all(status.as_bytes())?;
    stdout.write_all(RESET_STYLE)?;
    stdout.flush()?;
    Ok(())
}

struct TerminalSession {
    _raw_mode: RawTerminalMode,
}

impl TerminalSession {
    fn enter() -> Result<Self> {
        let raw_mode = RawTerminalMode::enable().context("failed to enter pager raw mode")?;
        let mut stdout = io::stdout().lock();
        stdout
            .write_all(ENTER_ALTERNATE_SCREEN)
            .context("failed to enter alternate screen")?;
        stdout.flush().context("failed to flush stdout")?;
        Ok(Self {
            _raw_mode: raw_mode,
        })
    }
}

impl Drop for TerminalSession {
    fn drop(&mut self) {
        let mut stdout = io::stdout().lock();
        let _ = stdout.write_all(RESET_STYLE);
        let _ = stdout.write_all(LEAVE_ALTERNATE_SCREEN);
        let _ = stdout.flush();
    }
}

#[cfg(unix)]
struct RawTerminalMode {
    fd: std::os::fd::RawFd,
    original: termios::Termios,
}

#[cfg(unix)]
impl RawTerminalMode {
    fn enable() -> Result<Self> {
        use std::os::fd::AsRawFd;

        let stdin = io::stdin();
        let fd = stdin.as_raw_fd();
        let original = termios::Termios::from_fd(fd)?;
        let mut raw = original;
        raw.c_lflag &= !(termios::ICANON | termios::ECHO | termios::ISIG);
        raw.c_cc[termios::VMIN] = 0;
        raw.c_cc[termios::VTIME] = 1;
        termios::tcsetattr(fd, termios::TCSANOW, &raw)?;
        Ok(Self { fd, original })
    }
}

#[cfg(unix)]
impl Drop for RawTerminalMode {
    fn drop(&mut self) {
        let _ = termios::tcsetattr(self.fd, termios::TCSANOW, &self.original);
    }
}

#[cfg(not(unix))]
struct RawTerminalMode;

#[cfg(not(unix))]
impl RawTerminalMode {
    fn enable() -> Result<Self> {
        Ok(Self)
    }
}
