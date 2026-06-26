use std::{
    io::{self, IsTerminal, Read, Write},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

use anyhow::{Context, Result};
use signal_hook::{consts::signal::SIGWINCH, flag};
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

const ENTER_ALTERNATE_SCREEN: &[u8] = b"\x1b[?1049h";
const LEAVE_ALTERNATE_SCREEN: &[u8] = b"\x1b[?1049l";
const CLEAR_SCREEN: &[u8] = b"\x1b[H\x1b[2J";
const CLEAR_LINE: &[u8] = b"\x1b[2K";
const RESET_STYLE: &[u8] = b"\x1b[0m";
const SEARCH_MATCH_STYLE: &str = "\x1b[7m";
const SEARCH_MATCH_RESET: &str = "\x1b[27m";
const SEARCH_CURRENT_STYLE: &str = "\x1b[7;4m";
const SEARCH_CURRENT_RESET: &str = "\x1b[24;27m";

pub(crate) fn run(output: &str) -> Result<()> {
    if !io::stdin().is_terminal() || !io::stdout().is_terminal() {
        return write_direct(output);
    }

    let _terminal = TerminalSession::enter()?;
    let resize_requested = Arc::new(AtomicBool::new(true));
    flag::register(SIGWINCH, resize_requested.clone()).context("failed to register SIGWINCH")?;

    let document = PagerDocument::new(output);
    let mut state = PagerState::default();
    let mut mode = PagerMode::Normal;
    let mut stdin = io::stdin().lock();

    loop {
        if resize_requested.swap(false, Ordering::Relaxed) {
            let size = terminal_size();
            let viewport = PagerViewport::build(&document, size.cols);
            state.restore_anchor(&viewport);
            render(&document, &viewport, &state, &mode, size)?;
        }

        let mut buf = [0_u8; 16];
        let len = stdin.read(&mut buf).context("failed to read pager input")?;
        if len == 0 {
            continue;
        }

        let size = terminal_size();
        let viewport = PagerViewport::build(&document, size.cols);
        let outcome = handle_input(
            &buf[..len],
            &document,
            &viewport,
            &mut state,
            &mut mode,
            size,
        );
        if matches!(outcome, PagerInputOutcome::Quit) {
            return Ok(());
        }
        render(&document, &viewport, &state, &mode, size)?;
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
    plain_lines: Vec<String>,
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
        let plain_lines = lines.iter().map(|line| strip_ansi(line)).collect();
        Self { lines, plain_lines }
    }

    fn line_count(&self) -> usize {
        self.lines.len()
    }
}

#[derive(Default)]
struct PagerState {
    top_row: usize,
    anchor: RowAnchor,
    search: Option<SearchState>,
}

impl PagerState {
    fn apply(&mut self, action: PagerAction, viewport: &PagerViewport, rows: usize) {
        let page = rows.saturating_sub(1).max(1);
        match action {
            PagerAction::Quit => {}
            PagerAction::LineDown => self.top_row = self.top_row.saturating_add(1),
            PagerAction::LineUp => self.top_row = self.top_row.saturating_sub(1),
            PagerAction::PageDown => self.top_row = self.top_row.saturating_add(page),
            PagerAction::PageUp => self.top_row = self.top_row.saturating_sub(page),
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

        let start_line = self.anchor.line_index;
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
        self.top_row = viewport.first_row_for_line(self.current_search_match().line_index);
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
        self.top_row = viewport.first_row_for_line(self.current_search_match().line_index);
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

    fn current_search_match_in_row(&self, row: &DisplayRow) -> Option<SearchMatch> {
        let item = self.search.as_ref()?.selected_match()?;
        if item.line_index == row.line_index
            && row.start_plain <= item.start_plain
            && item.start_plain < row.end_plain
        {
            Some(item)
        } else {
            None
        }
    }

    fn restore_anchor(&mut self, viewport: &PagerViewport) {
        self.top_row = viewport.row_for_anchor(self.anchor);
        self.clamp(viewport);
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
    line_index: usize,
    display_column: usize,
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
    LineDown,
    LineUp,
    PageDown,
    PageUp,
    Home,
    End,
    SearchStart,
    SearchNext,
    SearchPrevious,
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
            [b'/'] => Some(Self::SearchStart),
            [b'n'] => Some(Self::SearchNext),
            [b'N'] => Some(Self::SearchPrevious),
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

enum PagerMode {
    Normal,
    Search {
        query: String,
        pending_utf8: Vec<u8>,
    },
}

enum PagerInputOutcome {
    Continue,
    Quit,
}

enum SearchDirection {
    Forward,
    Backward,
}

fn handle_input(
    input: &[u8],
    document: &PagerDocument,
    viewport: &PagerViewport,
    state: &mut PagerState,
    mode: &mut PagerMode,
    size: PagerSize,
) -> PagerInputOutcome {
    match mode {
        PagerMode::Normal if input.first() == Some(&b'/') => {
            *mode = PagerMode::Search {
                query: String::new(),
                pending_utf8: Vec::new(),
            };
            if input.len() == 1 {
                PagerInputOutcome::Continue
            } else {
                handle_input(&input[1..], document, viewport, state, mode, size)
            }
        }
        PagerMode::Normal => handle_normal_input(input, viewport, state, mode, size),
        PagerMode::Search {
            query,
            pending_utf8,
        } => {
            let outcome =
                handle_search_input(input, document, viewport, state, query, pending_utf8);
            if outcome.close_search {
                *mode = PagerMode::Normal;
            }
            if outcome.quit {
                PagerInputOutcome::Quit
            } else {
                PagerInputOutcome::Continue
            }
        }
    }
}

fn handle_normal_input(
    input: &[u8],
    viewport: &PagerViewport,
    state: &mut PagerState,
    mode: &mut PagerMode,
    size: PagerSize,
) -> PagerInputOutcome {
    if input.len() > 1 && input.first() != Some(&0x1b) {
        for byte in input {
            let outcome = handle_normal_input(&[*byte], viewport, state, mode, size);
            if matches!(outcome, PagerInputOutcome::Quit) {
                return PagerInputOutcome::Quit;
            }
            if !matches!(mode, PagerMode::Normal) {
                return PagerInputOutcome::Continue;
            }
        }
        return PagerInputOutcome::Continue;
    }

    let Some(action) = PagerAction::parse(input) else {
        return PagerInputOutcome::Continue;
    };
    match action {
        PagerAction::Quit => PagerInputOutcome::Quit,
        PagerAction::SearchStart => {
            *mode = PagerMode::Search {
                query: String::new(),
                pending_utf8: Vec::new(),
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
    input: &[u8],
    document: &PagerDocument,
    viewport: &PagerViewport,
    state: &mut PagerState,
    query: &mut String,
    pending_utf8: &mut Vec<u8>,
) -> SearchInputOutcome {
    for byte in input {
        match *byte {
            0x03 => {
                return SearchInputOutcome {
                    close_search: true,
                    quit: true,
                };
            }
            0x1b => {
                return SearchInputOutcome {
                    close_search: true,
                    quit: false,
                };
            }
            b'\r' | b'\n' => {
                flush_pending_search_utf8(query, pending_utf8);
                let query = query.clone();
                state.search(document, viewport, &query);
                return SearchInputOutcome {
                    close_search: true,
                    quit: false,
                };
            }
            0x7f | 0x08 => {
                pending_utf8.clear();
                query.pop();
            }
            byte if byte.is_ascii_control() => {}
            byte if byte.is_ascii() => query.push(char::from(byte)),
            byte => push_search_utf8_byte(query, pending_utf8, byte),
        }
    }
    SearchInputOutcome {
        close_search: false,
        quit: false,
    }
}

fn push_search_utf8_byte(query: &mut String, pending_utf8: &mut Vec<u8>, byte: u8) {
    pending_utf8.push(byte);
    match std::str::from_utf8(pending_utf8) {
        Ok(text) => {
            query.push_str(text);
            pending_utf8.clear();
        }
        Err(error) if error.error_len().is_some() => {
            query.push_str(&String::from_utf8_lossy(pending_utf8));
            pending_utf8.clear();
        }
        Err(_) => {}
    }
}

fn flush_pending_search_utf8(query: &mut String, pending_utf8: &mut Vec<u8>) {
    if pending_utf8.is_empty() {
        return;
    }
    query.push_str(&String::from_utf8_lossy(pending_utf8));
    pending_utf8.clear();
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
    rows: Vec<DisplayRow>,
}

impl PagerViewport {
    fn build(document: &PagerDocument, cols: usize) -> Self {
        let mut rows = Vec::new();
        for (line_index, line) in document.lines.iter().enumerate() {
            wrap_line(line_index, line, cols.max(1), &mut rows);
        }
        Self { rows }
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
                    .position(|row| row.line_index >= anchor.line_index)
            })
            .unwrap_or_else(|| self.rows.len().saturating_sub(1))
    }

    fn first_row_for_line(&self, line_index: usize) -> usize {
        self.rows
            .iter()
            .position(|row| row.line_index == line_index)
            .unwrap_or_else(|| self.rows.len().saturating_sub(1))
    }
}

struct DisplayRow {
    line_index: usize,
    start_column: usize,
    end_column: usize,
    start_plain: usize,
    end_plain: usize,
    ansi: String,
    plain: String,
    plain_to_ansi: Vec<(usize, usize)>,
}

impl DisplayRow {
    fn anchor(&self) -> RowAnchor {
        RowAnchor {
            line_index: self.line_index,
            display_column: self.start_column,
        }
    }

    fn contains_anchor(&self, anchor: RowAnchor) -> bool {
        self.line_index == anchor.line_index
            && self.start_column <= anchor.display_column
            && (anchor.display_column < self.end_column
                || self.start_column == self.end_column
                || anchor.display_column == self.start_column)
    }

    fn display_width(&self) -> usize {
        self.end_column.saturating_sub(self.start_column)
    }

    fn ansi_offset_for_plain(&self, plain_offset: usize) -> Option<usize> {
        if plain_offset == self.plain.len() {
            return Some(self.ansi.len());
        }
        self.plain_to_ansi
            .iter()
            .find_map(|(plain, ansi)| (*plain == plain_offset).then_some(*ansi))
    }
}

fn wrap_line(line_index: usize, line: &str, cols: usize, rows: &mut Vec<DisplayRow>) {
    let mut parser = AnsiLineParser::new(line);
    let mut row = String::new();
    let mut plain = String::new();
    let mut plain_to_ansi = Vec::new();
    let mut replay_prefix = String::new();
    let mut column = 0;
    let mut row_start_column = 0;
    let mut row_start_plain = 0;
    let mut plain_offset = 0;
    let mut emitted = false;

    while let Some(token) = parser.next_token() {
        match token {
            AnsiToken::Control(control) => {
                replay_prefix.push_str(control);
                row.push_str(control);
            }
            AnsiToken::Text(text) => {
                for grapheme in text.graphemes(true) {
                    let (rendered, width) = rendered_grapheme(grapheme, column);
                    if column > 0 && column + width > cols {
                        rows.push(DisplayRow {
                            line_index,
                            start_column: row_start_column,
                            end_column: row_start_column + column,
                            start_plain: row_start_plain,
                            end_plain: plain_offset,
                            ansi: row,
                            plain,
                            plain_to_ansi,
                        });
                        row = replay_prefix.clone();
                        plain = String::new();
                        plain_to_ansi = Vec::new();
                        row_start_column += column;
                        row_start_plain = plain_offset;
                        column = 0;
                        emitted = true;
                    }

                    plain_to_ansi.push((plain.len(), row.len()));
                    row.push_str(&rendered);
                    plain.push_str(grapheme);
                    column += width;
                    plain_offset += grapheme.len();

                    if column >= cols {
                        rows.push(DisplayRow {
                            line_index,
                            start_column: row_start_column,
                            end_column: row_start_column + column,
                            start_plain: row_start_plain,
                            end_plain: plain_offset,
                            ansi: row,
                            plain,
                            plain_to_ansi,
                        });
                        row = replay_prefix.clone();
                        plain = String::new();
                        plain_to_ansi = Vec::new();
                        row_start_column += column;
                        row_start_plain = plain_offset;
                        column = 0;
                        emitted = true;
                    }
                }
            }
        }
    }

    if column > 0 || !emitted || !row.is_empty() {
        rows.push(DisplayRow {
            line_index,
            start_column: row_start_column,
            end_column: row_start_column + column,
            start_plain: row_start_plain,
            end_plain: plain_offset,
            ansi: row,
            plain,
            plain_to_ansi,
        });
    }
}

fn rendered_grapheme(grapheme: &str, column: usize) -> (String, usize) {
    if grapheme == "\t" {
        let width = 8 - (column % 8);
        (" ".repeat(width), width)
    } else {
        (grapheme.to_owned(), UnicodeWidthStr::width(grapheme).max(1))
    }
}

enum AnsiToken<'a> {
    Control(&'a str),
    Text(&'a str),
}

struct AnsiLineParser<'a> {
    line: &'a str,
    index: usize,
}

impl<'a> AnsiLineParser<'a> {
    fn new(line: &'a str) -> Self {
        Self { line, index: 0 }
    }

    fn next_token(&mut self) -> Option<AnsiToken<'a>> {
        if self.index >= self.line.len() {
            return None;
        }

        let bytes = self.line.as_bytes();
        if bytes[self.index] == 0x1b {
            let start = self.index;
            self.index = control_sequence_end(self.line, self.index);
            return Some(AnsiToken::Control(&self.line[start..self.index]));
        }

        let start = self.index;
        while self.index < bytes.len() && bytes[self.index] != 0x1b {
            self.index += self.line[self.index..]
                .chars()
                .next()
                .map(char::len_utf8)
                .unwrap_or(1);
        }
        Some(AnsiToken::Text(&self.line[start..self.index]))
    }
}

fn control_sequence_end(text: &str, start: usize) -> usize {
    let bytes = text.as_bytes();
    if bytes.get(start + 1) == Some(&b'[') {
        let mut index = start + 2;
        while index < bytes.len() {
            let byte = bytes[index];
            index += 1;
            if (0x40..=0x7e).contains(&byte) {
                break;
            }
        }
        return index;
    }

    if bytes.get(start + 1) == Some(&b']') {
        let mut index = start + 2;
        while index < bytes.len() {
            if bytes[index] == 0x07 {
                return index + 1;
            }
            if bytes[index] == 0x1b && bytes.get(index + 1) == Some(&b'\\') {
                return index + 2;
            }
            index += 1;
        }
        return index;
    }

    (start + 1).min(text.len())
}

fn strip_ansi(text: &str) -> String {
    let mut parser = AnsiLineParser::new(text);
    let mut stripped = String::with_capacity(text.len());
    while let Some(token) = parser.next_token() {
        if let AnsiToken::Text(text) = token {
            stripped.push_str(text);
        }
    }
    stripped
}

fn render(
    document: &PagerDocument,
    viewport: &PagerViewport,
    state: &PagerState,
    mode: &PagerMode,
    size: PagerSize,
) -> Result<()> {
    let body_rows = size.rows.saturating_sub(1);
    let mut stdout = io::stdout().lock();
    stdout.write_all(CLEAR_SCREEN)?;

    for row in viewport.rows.iter().skip(state.top_row).take(body_rows) {
        write_row(&mut stdout, row, state)?;
        stdout.write_all(RESET_STYLE)?;
        if row.display_width() < size.cols {
            stdout.write_all(b"\r\n")?;
        }
    }

    write!(stdout, "\x1b[{};1H", size.rows)?;
    stdout.write_all(RESET_STYLE)?;
    stdout.write_all(CLEAR_LINE)?;

    let end_line = if viewport.row_count() == 0 {
        0
    } else {
        viewport
            .rows
            .get((state.top_row + body_rows).saturating_sub(1))
            .map(|row| row.line_index + 1)
            .unwrap_or(document.line_count())
            .min(document.line_count())
    };
    let mut status = match mode {
        PagerMode::Normal => format!(
            "kat {}/{}  q:quit  /:search  n/N:next  j/k:row  PgUp/PgDn:page",
            end_line,
            document.line_count()
        ),
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
    let Some(query) = state.search_query() else {
        stdout.write_all(row.ansi.as_bytes())?;
        return Ok(());
    };
    if query.is_empty() {
        stdout.write_all(row.ansi.as_bytes())?;
        return Ok(());
    }

    let current_match = state.current_search_match_in_row(row);
    let mut search_from = 0;
    let mut ansi_from = 0;
    while let Some(relative_start) = row.plain[search_from..].find(query) {
        let plain_start = search_from + relative_start;
        let plain_end = plain_start + query.len();
        if let (Some(ansi_start), Some(ansi_end)) = (
            row.ansi_offset_for_plain(plain_start),
            row.ansi_offset_for_plain(plain_end),
        ) {
            stdout.write_all(&row.ansi.as_bytes()[ansi_from..ansi_start])?;
            let is_current = current_match.is_some_and(|item| {
                item.start_plain == row.start_plain + plain_start
                    && item.end_plain == row.start_plain + plain_end
            });
            let (style, reset) = if is_current {
                (SEARCH_CURRENT_STYLE, SEARCH_CURRENT_RESET)
            } else {
                (SEARCH_MATCH_STYLE, SEARCH_MATCH_RESET)
            };
            stdout.write_all(style.as_bytes())?;
            stdout.write_all(&row.ansi.as_bytes()[ansi_start..ansi_end])?;
            stdout.write_all(reset.as_bytes())?;
            ansi_from = ansi_end;
        }
        search_from = plain_end;
    }

    stdout.write_all(&row.ansi.as_bytes()[ansi_from..])?;
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
