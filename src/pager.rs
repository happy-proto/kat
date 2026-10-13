use std::{
    io::{self, IsTerminal, Read, Write},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};

use anyhow::{Context, Result};
#[cfg(unix)]
use signal_hook::{consts::signal::SIGWINCH, flag};
use termwiz::input::{InputEvent, InputParser, KeyCode, KeyEvent, Modifiers};

const ENTER_ALTERNATE_SCREEN: &[u8] = b"\x1b[?1049h";
const LEAVE_ALTERNATE_SCREEN: &[u8] = b"\x1b[?1049l";
const CLEAR_SCREEN: &[u8] = b"\x1b[H\x1b[2J";
const CLEAR_LINE: &[u8] = b"\x1b[2K";
const RESET_STYLE: &[u8] = b"\x1b[0m";
const MESSAGE_DURATION: Duration = Duration::from_secs(3);

pub(crate) fn run(
    mut render_source: impl FnMut(usize, bool) -> Result<(PagerSource, bool)>,
) -> Result<()> {
    if !io::stdin().is_terminal() || !io::stdout().is_terminal() {
        let (source, _) = render_source(terminal_size().cols, false)?;
        return write_direct(
            &source
                .rows
                .iter()
                .map(|row| row.rendered.encode(&[], None))
                .collect::<Vec<_>>()
                .join("\n"),
        );
    }

    let _terminal = TerminalSession::enter()?;
    let resize_requested = Arc::new(AtomicBool::new(false));
    register_resize_signal(resize_requested.clone())?;

    let mut size = terminal_size();
    let mut document = PagerDocument::from_source(render_source(size.cols, false)?.0);
    let mut state = PagerState::default();
    let mut mode = PagerMode::Normal;
    let mut stdin = io::stdin().lock();
    let mut viewport = PagerViewport::build(&document, size.cols);
    state.restore_anchor(&viewport);
    render(&document, &viewport, &state, &mode, size)?;

    loop {
        if resize_requested.swap(false, Ordering::Relaxed) {
            let new_size = terminal_size();
            if new_size.cols != size.cols {
                document = PagerDocument::from_source(render_source(new_size.cols, false)?.0);
                viewport = PagerViewport::build(&document, new_size.cols);
                state.restore_anchor(&viewport);
                state.rebuild_search(&document);
            }
            size = new_size;
            render(&document, &viewport, &state, &mode, size)?;
        }

        let mut buf = [0_u8; 16];
        let len = stdin.read(&mut buf).context("failed to read pager input")?;
        let events = state.input_parser.parse_as_vec(&buf[..len], len != 0);
        if state.expire_message(Instant::now()) {
            render(&document, &viewport, &state, &mode, size)?;
        }
        if events.is_empty() {
            continue;
        }

        for event in events {
            let outcome = handle_events([event], &document, &viewport, &mut state, &mut mode, size);
            match outcome {
                PagerInputOutcome::Quit => return Ok(()),
                PagerInputOutcome::Reload => match render_source(size.cols, true) {
                    Ok((source, changed)) => {
                        document = PagerDocument::from_source(source);
                        viewport = PagerViewport::build(&document, size.cols);
                        state.restore_anchor(&viewport);
                        state.rebuild_search(&document);
                        state.show_message(format!(
                            "Reloaded: {}  R:reload  q:quit",
                            if changed { "changed" } else { "unchanged" }
                        ));
                    }
                    Err(error) => state.show_message(format!("Reload failed: {error:#}")),
                },
                PagerInputOutcome::Continue => {}
            }
        }
        render(&document, &viewport, &state, &mode, size)?;
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct PagerRowAnchor {
    pub(crate) source_id: usize,
    pub(crate) source_line_start: usize,
    pub(crate) wrapped_from_column: usize,
}

#[derive(Debug)]
pub(crate) struct PagerSource {
    rows: Vec<DisplayRow>,
}

#[derive(Debug, Default)]
pub(crate) struct PagerSourceBuilder {
    rows: Vec<DisplayRow>,
    open_line: bool,
}

impl PagerSourceBuilder {
    pub(crate) fn append(
        &mut self,
        source_id: usize,
        document: &kat::PreparedDocument,
        cols: usize,
    ) {
        if document.source().is_empty() {
            return;
        }
        let mut rows = document.display_rows(cols);
        // The leading separator newline terminates an unfinished previous segment.
        if self.open_line && document.source().starts_with('\n') {
            rows.remove(0);
        }
        self.open_line = !document.source().ends_with('\n');
        self.rows
            .extend(rows.into_iter().map(|rendered| DisplayRow {
                line_index: 0,
                source_id,
                source_line_start: rendered.anchor.source_line_start,
                source_column_start: rendered.anchor.wrapped_from_column,
                start_column: 0,
                end_column: rendered.display_width,
                start_plain: 0,
                end_plain: rendered.text.len(),
                plain: rendered.text.clone(),
                rendered,
            }));
    }

    pub(crate) fn append_plain(&mut self, source_id: usize, source: &str, cols: usize) {
        self.append(source_id, &kat::PreparedDocument::plain(source), cols);
    }

    pub(crate) fn finish(self) -> PagerSource {
        PagerSource { rows: self.rows }
    }
}

#[cfg(unix)]
fn register_resize_signal(resize_requested: Arc<AtomicBool>) -> Result<()> {
    flag::register(SIGWINCH, resize_requested).context("failed to register SIGWINCH")?;
    Ok(())
}

#[cfg(not(unix))]
fn register_resize_signal(_: Arc<AtomicBool>) -> Result<()> {
    Ok(())
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
    rows: Arc<[DisplayRow]>,
    plain_lines: Vec<String>,
    anchors: Vec<PagerRowAnchor>,
}

impl PagerDocument {
    #[cfg(test)]
    fn new(source: &str) -> Self {
        let mut builder = PagerSourceBuilder::default();
        builder.append_plain(0, source, 80);
        Self::from_source(builder.finish())
    }

    fn from_source(mut source: PagerSource) -> Self {
        let mut plain_lines: Vec<String> = Vec::new();
        let mut anchors = Vec::new();
        for row in &mut source.rows {
            let anchor = PagerRowAnchor {
                source_id: row.source_id,
                source_line_start: row.source_line_start,
                wrapped_from_column: 0,
            };
            if anchors.last() != Some(&anchor) {
                anchors.push(anchor);
                plain_lines.push(String::new());
            }
            row.line_index = plain_lines.len() - 1;
            let line = plain_lines.last_mut().unwrap();
            row.start_plain = line.len();
            line.push_str(&row.plain);
            row.end_plain = line.len();
        }
        Self {
            rows: source.rows.into(),
            plain_lines,
            anchors,
        }
    }

    fn line_index_for_anchor(&self, anchor: RowAnchor) -> Option<usize> {
        self.anchors.iter().position(|candidate| {
            candidate.source_id == anchor.source_id
                && candidate.source_line_start == anchor.source_line_start
        })
    }
}

#[derive(Default)]
struct PagerState {
    input_parser: InputParser,
    top_row: usize,
    anchor: RowAnchor,
    search: Option<SearchState>,
    message: Option<PagerMessage>,
}

struct PagerMessage {
    text: String,
    expires_at: Instant,
}

impl PagerState {
    fn show_message(&mut self, text: String) {
        self.message = Some(PagerMessage {
            text,
            expires_at: Instant::now() + MESSAGE_DURATION,
        });
    }

    fn expire_message(&mut self, now: Instant) -> bool {
        if self
            .message
            .as_ref()
            .is_some_and(|message| now >= message.expires_at)
        {
            self.message = None;
            true
        } else {
            false
        }
    }

    fn apply(&mut self, action: PagerAction, viewport: &PagerViewport, rows: usize) {
        let page = rows.saturating_sub(1).max(1);
        let half_page = page.div_ceil(2);
        match action {
            PagerAction::Quit | PagerAction::Reload => {}
            PagerAction::LineDown => self.top_row = self.top_row.saturating_add(1),
            PagerAction::LineUp => self.top_row = self.top_row.saturating_sub(1),
            PagerAction::PageDown => self.top_row = self.top_row.saturating_add(page),
            PagerAction::PageUp => self.top_row = self.top_row.saturating_sub(page),
            PagerAction::HalfPageDown => self.top_row = self.top_row.saturating_add(half_page),
            PagerAction::HalfPageUp => self.top_row = self.top_row.saturating_sub(half_page),
            PagerAction::Home => self.top_row = 0,
            PagerAction::End => self.top_row = viewport.row_count().saturating_sub(page),
            PagerAction::SearchStart | PagerAction::SearchNext | PagerAction::SearchPrevious => {}
        }
        self.clamp(viewport);
    }

    fn search(&mut self, document: &PagerDocument, viewport: &PagerViewport, query: &str) -> bool {
        let matches = SearchMatches::new(document, query);
        if matches.is_empty() {
            return false;
        }

        let start_line = document
            .line_index_for_anchor(self.anchor)
            .unwrap_or_default();
        let selected = matches
            .items
            .iter()
            .position(|item| item.line_index >= start_line)
            .unwrap_or(0);
        self.search = Some(SearchState {
            query: query.to_owned(),
            matches,
            selected,
        });
        self.top_row = viewport.row_for_match(self.current_search_match());
        self.clamp(viewport);
        true
    }

    fn repeat_search(&mut self, viewport: &PagerViewport, direction: SearchDirection) -> bool {
        let Some(search) = self.search.as_mut() else {
            return false;
        };
        match direction {
            SearchDirection::Forward => search.selected = (search.selected + 1) % search.len(),
            SearchDirection::Backward => {
                search.selected = search.selected.checked_sub(1).unwrap_or(search.len() - 1);
            }
        }
        self.top_row = viewport.row_for_match(self.current_search_match());
        self.clamp(viewport);
        true
    }

    fn search_query(&self) -> Option<&str> {
        self.search.as_ref().map(|search| search.query.as_str())
    }

    fn current_search_match(&self) -> SearchMatch {
        self.search
            .as_ref()
            .and_then(SearchState::selected_match)
            .expect("current search match should exist when search state is present")
    }

    fn restore_anchor(&mut self, viewport: &PagerViewport) {
        self.top_row = viewport.row_for_anchor(self.anchor);
        self.clamp(viewport);
    }

    fn rebuild_search(&mut self, document: &PagerDocument) {
        let Some(query) = self.search_query().map(str::to_owned) else {
            return;
        };
        let matches = SearchMatches::new(document, &query);
        if matches.is_empty() {
            self.search = None;
            return;
        }
        let start_line = document
            .line_index_for_anchor(self.anchor)
            .unwrap_or_default();
        let selected = matches
            .items
            .iter()
            .position(|item| item.line_index >= start_line)
            .unwrap_or_default();
        self.search = Some(SearchState {
            query,
            matches,
            selected,
        });
    }

    fn clamp(&mut self, viewport: &PagerViewport) {
        self.top_row = self.top_row.min(viewport.row_count().saturating_sub(1));
        self.anchor = viewport
            .rows
            .get(self.top_row)
            .map(DisplayRow::anchor)
            .unwrap_or_default();
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct RowAnchor {
    source_id: usize,
    source_line_start: usize,
    display_column: usize,
}

impl RowAnchor {
    fn source_line(self) -> PagerRowAnchor {
        PagerRowAnchor {
            source_id: self.source_id,
            source_line_start: self.source_line_start,
            wrapped_from_column: 0,
        }
    }
}

struct SearchState {
    query: String,
    matches: SearchMatches,
    selected: usize,
}

impl SearchState {
    fn len(&self) -> usize {
        self.matches.items.len()
    }

    fn selected_match(&self) -> Option<SearchMatch> {
        self.matches.items.get(self.selected).copied()
    }
}

struct SearchMatches {
    items: Vec<SearchMatch>,
}

impl SearchMatches {
    fn new(document: &PagerDocument, query: &str) -> Self {
        if query.is_empty() {
            return Self { items: Vec::new() };
        }

        let mut items = Vec::new();
        for (line_index, line) in document.plain_lines.iter().enumerate() {
            let mut offset = 0;
            while let Some(relative_start) = line[offset..].find(query) {
                let start_plain = offset + relative_start;
                let end_plain = start_plain + query.len();
                items.push(SearchMatch {
                    line_index,
                    start_plain,
                    end_plain,
                });
                offset = end_plain;
            }
        }
        Self { items }
    }

    fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

#[derive(Clone, Copy)]
struct SearchMatch {
    line_index: usize,
    start_plain: usize,
    end_plain: usize,
}

#[derive(Clone, Copy)]
enum PagerAction {
    Quit,
    Reload,
    LineDown,
    LineUp,
    PageDown,
    PageUp,
    HalfPageDown,
    HalfPageUp,
    Home,
    End,
    SearchStart,
    SearchNext,
    SearchPrevious,
}

impl PagerAction {
    fn parse(input: &KeyEvent) -> Option<Self> {
        if input.modifiers.contains(Modifiers::ALT) {
            return None;
        }
        if input.modifiers.contains(Modifiers::CTRL) {
            return match input.key {
                KeyCode::Char('c' | 'C') => Some(Self::Quit),
                KeyCode::Char('d' | 'D') => Some(Self::HalfPageDown),
                KeyCode::Char('u' | 'U') => Some(Self::HalfPageUp),
                _ => None,
            };
        }
        match input.key {
            KeyCode::Char('q' | 'Q' | '\x03') | KeyCode::Escape => Some(Self::Quit),
            KeyCode::Char('j') | KeyCode::DownArrow | KeyCode::ApplicationDownArrow => {
                Some(Self::LineDown)
            }
            KeyCode::Char('k') | KeyCode::UpArrow | KeyCode::ApplicationUpArrow => {
                Some(Self::LineUp)
            }
            KeyCode::Char(' ') | KeyCode::PageDown => Some(Self::PageDown),
            KeyCode::Char('b') | KeyCode::PageUp => Some(Self::PageUp),
            KeyCode::Char('\x04') => Some(Self::HalfPageDown),
            KeyCode::Char('\x15') => Some(Self::HalfPageUp),
            KeyCode::Char('g') | KeyCode::Home => Some(Self::Home),
            KeyCode::Char('G') | KeyCode::End => Some(Self::End),
            KeyCode::Char('/') => Some(Self::SearchStart),
            KeyCode::Char('n') => Some(Self::SearchNext),
            KeyCode::Char('N') => Some(Self::SearchPrevious),
            KeyCode::Char('R') => Some(Self::Reload),
            _ => None,
        }
    }
}

enum PagerMode {
    Normal,
    Search { query: String },
}

enum PagerInputOutcome {
    Continue,
    Quit,
    Reload,
}

enum SearchDirection {
    Forward,
    Backward,
}

#[cfg(test)]
fn handle_input(
    input: &[u8],
    document: &PagerDocument,
    viewport: &PagerViewport,
    state: &mut PagerState,
    mode: &mut PagerMode,
    size: PagerSize,
) -> PagerInputOutcome {
    let events = state.input_parser.parse_as_vec(input, !input.is_empty());
    handle_events(events, document, viewport, state, mode, size)
}

fn handle_events(
    events: impl IntoIterator<Item = InputEvent>,
    document: &PagerDocument,
    viewport: &PagerViewport,
    state: &mut PagerState,
    mode: &mut PagerMode,
    size: PagerSize,
) -> PagerInputOutcome {
    for event in events {
        let outcome = match mode {
            PagerMode::Normal => match event {
                InputEvent::Key(key) => handle_normal_input(&key, viewport, state, mode, size),
                _ => PagerInputOutcome::Continue,
            },
            PagerMode::Search { query } => {
                let outcome = handle_search_input(event, document, viewport, state, query);
                if outcome.close_search {
                    *mode = PagerMode::Normal;
                }
                if outcome.quit {
                    PagerInputOutcome::Quit
                } else {
                    PagerInputOutcome::Continue
                }
            }
        };
        if !matches!(outcome, PagerInputOutcome::Continue) {
            return outcome;
        }
    }
    PagerInputOutcome::Continue
}

fn handle_normal_input(
    input: &KeyEvent,
    viewport: &PagerViewport,
    state: &mut PagerState,
    mode: &mut PagerMode,
    size: PagerSize,
) -> PagerInputOutcome {
    let Some(action) = PagerAction::parse(input) else {
        return PagerInputOutcome::Continue;
    };
    state.message = None;
    match action {
        PagerAction::Quit => PagerInputOutcome::Quit,
        PagerAction::Reload => PagerInputOutcome::Reload,
        PagerAction::SearchStart => {
            *mode = PagerMode::Search {
                query: String::new(),
            };
            PagerInputOutcome::Continue
        }
        PagerAction::SearchNext => {
            state.repeat_search(viewport, SearchDirection::Forward);
            PagerInputOutcome::Continue
        }
        PagerAction::SearchPrevious => {
            state.repeat_search(viewport, SearchDirection::Backward);
            PagerInputOutcome::Continue
        }
        action => {
            state.apply(action, viewport, size.rows);
            PagerInputOutcome::Continue
        }
    }
}

fn handle_search_input(
    input: InputEvent,
    document: &PagerDocument,
    viewport: &PagerViewport,
    state: &mut PagerState,
    query: &mut String,
) -> SearchInputOutcome {
    match input {
        InputEvent::Paste(text) => query.push_str(&text),
        InputEvent::Key(key) if !key.modifiers.contains(Modifiers::ALT) => match key.key {
            KeyCode::Char('\x03') => {
                return SearchInputOutcome {
                    close_search: true,
                    quit: true,
                };
            }
            KeyCode::Char('c' | 'C') if key.modifiers.contains(Modifiers::CTRL) => {
                return SearchInputOutcome {
                    close_search: true,
                    quit: true,
                };
            }
            KeyCode::Escape => {
                return SearchInputOutcome {
                    close_search: true,
                    quit: false,
                };
            }
            KeyCode::Enter => {
                state.search(document, viewport, query);
                return SearchInputOutcome {
                    close_search: true,
                    quit: false,
                };
            }
            KeyCode::Backspace => {
                query.pop();
            }
            KeyCode::Char(c) if !c.is_control() && !key.modifiers.contains(Modifiers::CTRL) => {
                query.push(c)
            }
            _ => {}
        },
        _ => {}
    }
    SearchInputOutcome {
        close_search: false,
        quit: false,
    }
}

struct SearchInputOutcome {
    close_search: bool,
    quit: bool,
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

struct PagerViewport {
    rows: Arc<[DisplayRow]>,
}

impl PagerViewport {
    fn build(document: &PagerDocument, _cols: usize) -> Self {
        Self {
            rows: document.rows.clone(),
        }
    }

    fn row_count(&self) -> usize {
        self.rows.len()
    }

    fn row_for_anchor(&self, anchor: RowAnchor) -> usize {
        self.rows
            .iter()
            .position(|row| row.contains_anchor(anchor))
            .or_else(|| {
                self.rows
                    .iter()
                    .position(|row| row.source_anchor() >= anchor.source_line())
            })
            .unwrap_or_else(|| self.rows.len().saturating_sub(1))
    }

    fn row_for_match(&self, item: SearchMatch) -> usize {
        self.rows
            .iter()
            .position(|row| {
                row.line_index == item.line_index
                    && row.start_plain <= item.start_plain
                    && item.start_plain < row.end_plain
            })
            .unwrap_or_else(|| self.rows.len().saturating_sub(1))
    }
}

#[derive(Debug)]
struct DisplayRow {
    line_index: usize,
    source_id: usize,
    source_line_start: usize,
    source_column_start: usize,
    start_column: usize,
    end_column: usize,
    start_plain: usize,
    end_plain: usize,
    rendered: kat::DisplayRow,
    plain: String,
}

impl DisplayRow {
    fn anchor(&self) -> RowAnchor {
        RowAnchor {
            source_id: self.source_id,
            source_line_start: self.source_line_start,
            display_column: self.source_column_start + self.start_column,
        }
    }

    fn contains_anchor(&self, anchor: RowAnchor) -> bool {
        let start_column = self.source_column_start + self.start_column;
        let end_column = self.source_column_start + self.end_column;
        self.source_id == anchor.source_id
            && self.source_line_start == anchor.source_line_start
            && start_column <= anchor.display_column
            && (anchor.display_column < end_column
                || start_column == end_column
                || anchor.display_column == start_column)
    }

    fn source_anchor(&self) -> PagerRowAnchor {
        PagerRowAnchor {
            source_id: self.source_id,
            source_line_start: self.source_line_start,
            wrapped_from_column: self.source_column_start,
        }
    }
}

fn render(
    _document: &PagerDocument,
    viewport: &PagerViewport,
    state: &PagerState,
    mode: &PagerMode,
    size: PagerSize,
) -> Result<()> {
    let body_rows = size.rows.saturating_sub(1);
    let mut stdout = io::stdout().lock();
    stdout.write_all(CLEAR_SCREEN)?;

    for (screen_row, row) in viewport
        .rows
        .iter()
        .skip(state.top_row)
        .take(body_rows)
        .enumerate()
    {
        write!(stdout, "\x1b[{};1H", screen_row + 1)?;
        write_row(&mut stdout, row, state)?;
        stdout.write_all(RESET_STYLE)?;
    }

    write!(stdout, "\x1b[{};1H", size.rows)?;
    stdout.write_all(RESET_STYLE)?;
    stdout.write_all(CLEAR_LINE)?;

    let end_line = (state.top_row + body_rows).min(viewport.row_count());
    let mut status = match mode {
        PagerMode::Normal => state.message.as_ref().map(|message| message.text.clone()).unwrap_or_else(|| format!(
            "kat {}/{}  q:quit  R:reload  /:search  n/N:next  j/k:row  PgUp/PgDn:page  ^D/^U:half",
            end_line,
            viewport.row_count()
        )),
        PagerMode::Search { query, .. } => format!("/{query}"),
    };
    if status.chars().count() > size.cols {
        status = status.chars().take(size.cols).collect();
    }
    stdout.write_all(b"\x1b[7m")?;
    stdout.write_all(status.as_bytes())?;
    stdout.write_all(RESET_STYLE)?;
    stdout.flush()?;
    Ok(())
}

fn write_row(mut stdout: impl Write, row: &DisplayRow, state: &PagerState) -> Result<()> {
    let matches = state
        .search
        .as_ref()
        .map(|search| {
            search
                .matches
                .items
                .iter()
                .filter(|item| {
                    item.line_index == row.line_index
                        && item.start_plain < row.end_plain
                        && item.end_plain > row.start_plain
                })
                .map(|item| {
                    item.start_plain.max(row.start_plain) - row.start_plain
                        ..item.end_plain.min(row.end_plain) - row.start_plain
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let current = state
        .search
        .as_ref()
        .and_then(SearchState::selected_match)
        .filter(|item| {
            item.line_index == row.line_index
                && item.start_plain < row.end_plain
                && item.end_plain > row.start_plain
        })
        .map(|item| {
            item.start_plain.max(row.start_plain) - row.start_plain
                ..item.end_plain.min(row.end_plain) - row.start_plain
        });
    stdout.write_all(row.rendered.encode(&matches, current.as_ref()).as_bytes())?;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn colored_full_width_rows_have_one_navigation_slot_each() {
        let source = format!("{{\"instructions_template\":\"{}\"}}\n", "a".repeat(400));
        let prepared = kat::PreparedDocument::named_language(
            "json",
            &source,
            kat::RenderOptions::with_colors((255, 255, 255), (0, 0, 0)),
        )
        .unwrap();
        let rendered = prepared.render(Some(40));
        let mut builder = PagerSourceBuilder::default();
        builder.append(0, &prepared, 40);
        let document = PagerDocument::from_source(builder.finish());
        let viewport = PagerViewport::build(&document, 40);
        assert_eq!(viewport.row_count(), rendered.row_anchors.len());
    }

    #[test]
    fn status_message_expires_and_repeated_refresh_restarts_timeout() {
        let mut state = PagerState::default();
        state.show_message("Reloaded: changed".to_owned());
        let first_deadline = state.message.as_ref().unwrap().expires_at;
        assert!(!state.expire_message(first_deadline - Duration::from_millis(1)));
        state.show_message("Reloaded: unchanged".to_owned());
        let next_deadline = state.message.as_ref().unwrap().expires_at;
        assert!(!state.expire_message(next_deadline - Duration::from_millis(1)));
        assert!(state.expire_message(next_deadline));
        assert!(state.message.is_none());
        assert!(!state.expire_message(next_deadline));
    }

    #[test]
    fn reload_key_is_a_command_only_in_normal_mode() {
        let document = PagerDocument::new("R\n");
        let viewport = PagerViewport::build(&document, 80);
        let size = PagerSize { cols: 80, rows: 12 };
        let mut state = PagerState::default();
        let mut mode = PagerMode::Normal;
        assert!(matches!(
            handle_input(b"R", &document, &viewport, &mut state, &mut mode, size),
            PagerInputOutcome::Reload
        ));
        assert!(matches!(
            handle_input(b"/R", &document, &viewport, &mut state, &mut mode, size),
            PagerInputOutcome::Continue
        ));
        assert!(matches!(mode, PagerMode::Search { ref query } if query == "R"));
    }

    #[test]
    fn reload_rebuilds_search_and_clamps_position_for_shorter_content() {
        let old = PagerDocument::new("prefix\nneedle\nlast\n");
        let old_viewport = PagerViewport::build(&old, 80);
        let mut state = PagerState::default();
        assert!(state.search(&old, &old_viewport, "needle"));
        state.apply(PagerAction::LineDown, &old_viewport, 2);
        let new = PagerDocument::new("needle needle\n");
        let viewport = PagerViewport::build(&new, 80);
        state.restore_anchor(&viewport);
        state.rebuild_search(&new);
        assert_eq!(state.top_row, 0);
        assert_eq!(state.search.as_ref().unwrap().len(), 2);
        let empty = PagerDocument::new("");
        state.restore_anchor(&PagerViewport::build(&empty, 80));
        state.rebuild_search(&empty);
        assert!(state.search.is_none());
    }

    #[test]
    fn dense_cursor_input_does_not_quit_or_drop_steps() {
        let document = PagerDocument::new(&"line\n".repeat(100));
        let viewport = PagerViewport::build(&document, 80);
        let size = PagerSize { cols: 80, rows: 12 };
        let mut state = PagerState::default();
        let mut mode = PagerMode::Normal;
        let input = b"\x1b[B".repeat(10);
        for chunk in input.chunks(16) {
            assert!(matches!(
                handle_input(chunk, &document, &viewport, &mut state, &mut mode, size),
                PagerInputOutcome::Continue
            ));
        }
        assert_eq!(state.top_row, 10);
    }

    #[test]
    fn cursor_sequences_survive_every_read_chunk_size() {
        let document = PagerDocument::new(&"line\n".repeat(100));
        let viewport = PagerViewport::build(&document, 80);
        let size = PagerSize { cols: 80, rows: 12 };
        let input = b"\x1b[B\x1bOB".repeat(10);
        for chunk_size in 1..=input.len() {
            let mut state = PagerState::default();
            let mut mode = PagerMode::Normal;
            for chunk in input.chunks(chunk_size) {
                assert!(matches!(
                    handle_input(chunk, &document, &viewport, &mut state, &mut mode, size),
                    PagerInputOutcome::Continue
                ));
            }
            assert_eq!(state.top_row, 20, "chunk size {chunk_size}");
        }
    }

    #[test]
    fn lone_escape_waits_for_idle_before_quitting() {
        let document = PagerDocument::new("line\n");
        let viewport = PagerViewport::build(&document, 80);
        let size = PagerSize { cols: 80, rows: 12 };
        let mut state = PagerState::default();
        let mut mode = PagerMode::Normal;
        assert!(matches!(
            handle_input(b"\x1b", &document, &viewport, &mut state, &mut mode, size),
            PagerInputOutcome::Continue
        ));
        assert!(matches!(
            handle_input(b"", &document, &viewport, &mut state, &mut mode, size),
            PagerInputOutcome::Quit
        ));
    }

    #[test]
    fn search_handles_split_unicode_and_cursor_keys_without_cancelling() {
        let document = PagerDocument::new("before\n中文\nafter\n");
        let viewport = PagerViewport::build(&document, 80);
        let size = PagerSize { cols: 80, rows: 12 };
        let mut state = PagerState::default();
        let mut mode = PagerMode::Normal;
        for byte in "/中文\x1b[A".as_bytes() {
            handle_input(&[*byte], &document, &viewport, &mut state, &mut mode, size);
        }
        assert!(matches!(&mode, PagerMode::Search { query } if query == "中文"));
        handle_input(b"\rj", &document, &viewport, &mut state, &mut mode, size);
        assert!(matches!(mode, PagerMode::Normal));
        assert_eq!(state.top_row, 2);
        assert_eq!(state.search.as_ref().unwrap().query, "中文");
    }

    #[test]
    fn source_anchor_survives_source_aware_relayout() {
        let source = kat::PreparedDocument::plain("first half second half");
        let mut narrow = PagerSourceBuilder::default();
        narrow.append(7, &source, 10);
        let narrow_doc = PagerDocument::from_source(narrow.finish());
        let anchor = PagerViewport::build(&narrow_doc, 10).rows[1].anchor();
        let mut wide = PagerSourceBuilder::default();
        wide.append(7, &source, 80);
        let wide_doc = PagerDocument::from_source(wide.finish());
        assert_eq!(
            PagerViewport::build(&wide_doc, 80).row_for_anchor(anchor),
            0
        );
    }

    #[test]
    fn ctrl_d_and_ctrl_u_move_by_half_a_viewport() {
        let document = PagerDocument::new(
            &(1..=20)
                .map(|line| format!("line {line}\n"))
                .collect::<String>(),
        );
        let viewport = PagerViewport::build(&document, 80);
        let size = PagerSize { cols: 80, rows: 12 };
        let mut state = PagerState::default();
        let mut mode = PagerMode::Normal;

        handle_input(b"\x04", &document, &viewport, &mut state, &mut mode, size);
        assert_eq!(state.top_row, 6);

        handle_input(b"\x15", &document, &viewport, &mut state, &mut mode, size);
        assert_eq!(state.top_row, 0);
    }
}
