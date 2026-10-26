use std::{
    fs,
    io::{self, IsTerminal, Read, Write},
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};
use termwiz::input::{InputEvent, InputParser, KeyCode, KeyEvent, Modifiers};
use unicode_segmentation::UnicodeSegmentation;

use crate::{
    CliOptions, ViewerDocument, ViewerSegment, content_fingerprint, pager,
    pager::{PagerInputOutcome, PagerSize, PagerView, TerminalSession},
    terminal_image,
    ui_theme::visible_text,
};

mod filter;
mod style;
use filter::FilenameFilter;
use style::BrowserTheme;

// Automatic previews must not read arbitrarily large files. Right/l explicitly
// opens the complete file; special files are never read as documents.
const PREVIEW_BYTES: u64 = 8 * 1024 * 1024;

pub(crate) fn run(path: &Path, options: &CliOptions) -> Result<()> {
    if !io::stdin().is_terminal() || !io::stdout().is_terminal() {
        bail!("directory browsing requires a terminal on stdin and stdout");
    }
    let mut browser = Browser::new(path, options.icons)?;
    // Probe before raw mode, once for the entire browsing session.
    let render_options = kat::RenderOptions::terminal(crate::render_hyperlinks_enabled(options));
    browser.theme = BrowserTheme::new(render_options.color_mode(), options.icons);
    let terminal = TerminalSession::enter()?;
    let _image = ImagePlacement;
    let mut parser = InputParser::default();
    let mut size = pager::terminal_size();
    browser.load_preview(options, render_options, Some(PREVIEW_BYTES));
    let mut reader: Option<Reader> = None;
    let mut full_image = false;
    browser.render(size, options, false)?;

    loop {
        if terminal.resized() {
            let new_size = pager::terminal_size();
            if let Some(reader) = reader.as_mut() {
                if new_size.cols != size.cols {
                    reader
                        .view
                        .replace_source(reader.document.render(new_size.cols), new_size);
                }
                reader.view.render(new_size)?;
            } else {
                browser.render(new_size, options, full_image)?;
            }
            size = new_size;
        }
        let events = terminal.read_events(&mut parser)?;
        if let Some(reader) = reader.as_mut()
            && reader.view.expire_message()
        {
            reader.view.render(size)?;
        }
        if events.is_empty() {
            continue;
        }
        for event in events {
            if is_interrupt(&event) {
                return Ok(());
            }
            if let Some(active) = reader.as_mut() {
                match active.view.handle(event, size) {
                    PagerInputOutcome::Quit => {
                        browser.preview = Preview::Text(reader.take().unwrap().document);
                    }
                    PagerInputOutcome::Reload => {
                        let result = active
                            .document
                            .reload(options)
                            .map(|changed| (active.document.render(size.cols), changed));
                        active.view.finish_reload(result, size);
                    }
                    PagerInputOutcome::Continue => {}
                }
            } else if full_image {
                if is_parent_key(&event)
                    || matches!(
                        event,
                        InputEvent::Key(KeyEvent {
                            key: KeyCode::Char('q') | KeyCode::Escape | KeyCode::Enter,
                            ..
                        })
                    )
                {
                    full_image = false;
                }
            } else if browser.filter.editing {
                browser.edit_filter(event);
                if !browser.preview_loaded {
                    browser.load_preview(options, render_options, Some(PREVIEW_BYTES));
                }
            } else if let InputEvent::Key(key) = event {
                let Some(action) = Action::parse(&key) else {
                    continue;
                };
                browser.message = None;
                match action {
                    Action::Quit
                        if key.key == KeyCode::Escape && !browser.filter.query.is_empty() =>
                    {
                        browser.filter.query.clear();
                        browser.refilter(true);
                    }
                    Action::Quit => return Ok(()),
                    Action::Down => browser.select(browser.selected.saturating_add(1)),
                    Action::Up => browser.select(browser.selected.saturating_sub(1)),
                    Action::First => browser.select(0),
                    Action::Last => browser.select(browser.filter.matches.len().saturating_sub(1)),
                    Action::PageDown => browser.select(
                        browser
                            .selected
                            .saturating_add(size.rows.saturating_sub(1).max(1)),
                    ),
                    Action::PageUp => browser.select(
                        browser
                            .selected
                            .saturating_sub(size.rows.saturating_sub(1).max(1)),
                    ),
                    Action::Parent => browser.parent(),
                    Action::OpenCurrentDirectory => {
                        let path = browser.directory.clone();
                        if browser.open_shell_directory(&path, options) {
                            return Ok(());
                        }
                    }
                    Action::Filter => browser.filter.editing = true,
                    Action::OpenDirectory | Action::Navigate => {
                        if let Some(entry) = browser.current() {
                            if entry.directory {
                                let path = entry.path.clone();
                                if matches!(action, Action::OpenDirectory) {
                                    if browser.open_shell_directory(&path, options) {
                                        return Ok(());
                                    }
                                } else {
                                    browser.change_directory(&path, None);
                                }
                            } else if matches!(action, Action::Navigate) {
                                browser.load_preview(options, render_options, None);
                                match std::mem::replace(&mut browser.preview, Preview::Empty) {
                                    Preview::Text(document) => {
                                        let chrome = document.chrome(pager::ViewerExit::Back);
                                        let view = PagerView::new(
                                            document.render(size.cols),
                                            size,
                                            chrome,
                                        );
                                        reader = Some(Reader { document, view });
                                    }
                                    preview @ Preview::Image { .. } => {
                                        browser.preview = preview;
                                        full_image = true;
                                    }
                                    preview => browser.preview = preview,
                                }
                            }
                        }
                    }
                    Action::Reload => browser.refresh(),
                }
                if !browser.preview_loaded && reader.is_none() {
                    browser.load_preview(options, render_options, Some(PREVIEW_BYTES));
                }
            }
            // Draw each state transition: later events from this same read must
            // act on the new view, including l followed immediately by q.
            if let Some(reader) = reader.as_ref() {
                reader.view.render(size)?;
            } else {
                browser.render(size, options, full_image)?;
            }
        }
    }
}

struct Reader {
    document: ViewerDocument,
    view: PagerView,
}

struct ImagePlacement;

impl Drop for ImagePlacement {
    fn drop(&mut self) {
        let mut stdout = io::stdout().lock();
        let _ = stdout.write_all(terminal_image::CLEAR_BROWSER_IMAGE.as_bytes());
        let _ = stdout.flush();
    }
}

struct Entry {
    path: PathBuf,
    directory: bool,
    icon: style::EntryIcon,
}

impl Entry {
    fn label(&self) -> String {
        let name = self.path.file_name().unwrap_or_default().to_string_lossy();
        format!(
            "{}{}",
            visible_text(&name).replace('\n', "\\n"),
            if self.directory { "/" } else { "" }
        )
    }
}

fn read_entries(path: &Path) -> Result<Vec<Entry>> {
    let mut entries = fs::read_dir(path)
        .with_context(|| format!("cannot read directory {}", path.display()))?
        .map(|item| {
            let item = item?;
            let path = item.path();
            let directory = path.is_dir();
            Ok(Entry {
                icon: style::EntryIcon::for_path(&path, directory),
                directory,
                path,
            })
        })
        .collect::<io::Result<Vec<_>>>()?;
    entries.sort_by(|a, b| {
        b.directory
            .cmp(&a.directory)
            .then_with(|| a.path.file_name().cmp(&b.path.file_name()))
    });
    Ok(entries)
}

enum Preview {
    Empty,
    Directory(Vec<Entry>),
    Text(ViewerDocument),
    Image { path: PathBuf, bytes: Vec<u8> },
    Message(String),
}

struct Browser {
    directory: PathBuf,
    entries: Vec<Entry>,
    filter: FilenameFilter,
    selected: usize,
    top: usize,
    preview: Preview,
    preview_loaded: bool,
    message: Option<String>,
    theme: BrowserTheme,
    preferred_list_width: usize,
}

impl Browser {
    fn new(path: &Path, icons: crate::IconMode) -> Result<Self> {
        let directory = std::path::absolute(path)?;
        let entries = read_entries(&directory)?;
        let theme = BrowserTheme::new(kat::ColorMode::NoColor, icons);
        let preferred_list_width = theme.preferred_list_width(&entries);
        let filter = FilenameFilter::new(&entries);
        Ok(Self {
            directory,
            entries,
            filter,
            selected: 0,
            top: 0,
            preview: Preview::Empty,
            preview_loaded: false,
            message: None,
            theme,
            preferred_list_width,
        })
    }

    fn current(&self) -> Option<&Entry> {
        self.filter
            .matches
            .get(self.selected)
            .map(|item| &self.entries[item.index])
    }

    fn select(&mut self, index: usize) {
        let index = index.min(self.filter.matches.len().saturating_sub(1));
        if self.selected != index {
            self.selected = index;
            self.preview_loaded = false;
        }
    }

    fn open_shell_directory(&mut self, path: &Path, options: &CliOptions) -> bool {
        self.message = Some(match options.cwd_file.as_ref() {
            Some(output) => match fs::write(output, path.as_os_str().as_encoded_bytes()) {
                Ok(()) => return true,
                Err(error) => format!("Cannot open directory: {error}"),
            },
            None => "Enable shell integration: kat --install-shell-integration <shell>".to_owned(),
        });
        false
    }

    fn refilter(&mut self, preserve_selection: bool) {
        let previous = self.current().map(|entry| entry.path.clone());
        self.filter.rebuild(&self.entries);
        self.selected = self
            .filter
            .matches
            .iter()
            .position(|item| {
                preserve_selection
                    && previous
                        .as_ref()
                        .is_some_and(|path| self.entries[item.index].path == *path)
            })
            .unwrap_or(0);
        self.top = 0;
        if self.current().map(|entry| entry.path.as_path()) != previous.as_deref() {
            self.preview_loaded = false;
        }
        self.message = None;
    }

    fn edit_filter(&mut self, event: InputEvent) {
        match event {
            InputEvent::Paste(text) => self
                .filter
                .query
                .extend(text.chars().filter(|c| !c.is_control())),
            InputEvent::Key(key) => match key.key {
                KeyCode::Enter => {
                    self.filter.editing = false;
                    return;
                }
                KeyCode::Escape => {
                    self.filter.editing = false;
                    self.filter.query.clear();
                    self.refilter(true);
                    return;
                }
                KeyCode::DownArrow | KeyCode::ApplicationDownArrow => {
                    self.select(self.selected.saturating_add(1));
                    return;
                }
                KeyCode::UpArrow | KeyCode::ApplicationUpArrow => {
                    self.select(self.selected.saturating_sub(1));
                    return;
                }
                KeyCode::Backspace => {
                    let end = self
                        .filter
                        .query
                        .grapheme_indices(true)
                        .next_back()
                        .map_or(0, |(index, _)| index);
                    self.filter.query.truncate(end);
                }
                KeyCode::Char('u' | 'U') if key.modifiers.contains(Modifiers::CTRL) => {
                    self.filter.query.clear()
                }
                KeyCode::Char(c)
                    if !c.is_control()
                        && !key.modifiers.intersects(Modifiers::CTRL | Modifiers::ALT) =>
                {
                    self.filter.query.push(c)
                }
                _ => return,
            },
            _ => return,
        }
        self.refilter(false);
    }

    fn change_directory(&mut self, path: &Path, select: Option<&Path>) -> bool {
        match read_entries(path) {
            Ok(entries) => {
                self.preferred_list_width = self.theme.preferred_list_width(&entries);
                self.selected = select
                    .and_then(|path| entries.iter().position(|entry| entry.path == path))
                    .unwrap_or(0);
                self.directory = path.to_path_buf();
                self.entries = entries;
                self.filter.query.clear();
                self.filter.editing = false;
                self.filter.rebuild(&self.entries);
                self.top = 0;
                self.preview_loaded = false;
                true
            }
            Err(error) => {
                self.message = Some(format!("{error:#}"));
                false
            }
        }
    }

    fn parent(&mut self) {
        if let Some(parent) = self.directory.parent() {
            let parent = parent.to_path_buf();
            let child = self.directory.clone();
            self.change_directory(&parent, Some(&child));
        }
    }

    fn refresh(&mut self) {
        let query = self.filter.query.clone();
        let index = self.selected;
        let selected = self.current().map(|entry| entry.path.clone());
        let directory = self.directory.clone();
        if !self.change_directory(&directory, selected.as_deref()) {
            return;
        }
        self.filter.query = query;
        self.refilter(false);
        if let Some(index) = self.filter.matches.iter().position(|item| {
            selected
                .as_ref()
                .is_some_and(|path| self.entries[item.index].path == *path)
        }) {
            self.select(index);
        }
        if selected.is_some_and(|path| !self.entries.iter().any(|entry| entry.path == path)) {
            self.select(index);
        }
    }

    fn load_preview(
        &mut self,
        options: &CliOptions,
        render_options: kat::RenderOptions,
        limit: Option<u64>,
    ) {
        self.preview = match self.current() {
            None => Preview::Message(
                if self.entries.is_empty() {
                    "Empty directory"
                } else {
                    "No matching files"
                }
                .to_owned(),
            ),
            Some(entry) => match prepare_preview(entry, options, render_options, limit) {
                Ok(preview) => preview,
                Err(error) => Preview::Message(format!("Cannot preview: {error:#}")),
            },
        };
        self.preview_loaded = true;
    }

    fn render(&mut self, size: PagerSize, options: &CliOptions, full_image: bool) -> Result<()> {
        let mut stdout = io::stdout().lock();
        // Clear our Kitty image as well as text cells when replacing a preview.
        stdout.write_all(terminal_image::CLEAR_BROWSER_IMAGE.as_bytes())?;
        stdout.write_all(b"\x1b[0m\x1b[H\x1b[2J")?;
        if full_image {
            let title = match &self.preview {
                Preview::Image { path, .. } => crate::ui_theme::display_path(path),
                _ => String::new(),
            };
            self.render_image(
                &mut stdout,
                size.cols,
                size.rows.saturating_sub(1),
                1,
                1,
                options,
            )?;
            self.theme.write_status(
                &mut stdout,
                (size.rows, size.cols),
                &title,
                "q:back  h/←:back  Enter:back",
                kat::DraculaColor::Cyan,
            )?;
        } else if size.cols < 20 || size.rows < 3 {
            write_plain(
                &mut stdout,
                1,
                1,
                "Terminal too small",
                size.cols,
                self.theme.text_style(kat::DraculaColor::Comment),
            )?;
        } else {
            let left = self.preferred_list_width.min((size.cols / 3).max(1));
            let right = size.cols - left - 1;
            let body = size.rows - 1;
            if self.selected < self.top {
                self.top = self.selected;
            }
            if self.selected >= self.top + body {
                self.top = self.selected + 1 - body;
            }
            if self.filter.matches.is_empty() {
                write_plain(
                    &mut stdout,
                    1,
                    1,
                    if self.entries.is_empty() {
                        "(empty)"
                    } else {
                        "No matches"
                    },
                    left,
                    self.theme.text_style(kat::DraculaColor::Comment),
                )?;
            }
            for (row, item) in self
                .filter
                .matches
                .iter()
                .skip(self.top)
                .take(body)
                .enumerate()
            {
                let entry = &self.entries[item.index];
                let selected = self.top + row == self.selected;
                self.theme.write_entry(
                    &mut stdout,
                    entry,
                    (row + 1, 1),
                    left,
                    Some(selected),
                    &item.ranges,
                )?;
            }
            for row in 1..size.rows {
                write!(
                    stdout,
                    "\x1b[{row};{}H{}│\x1b[0m",
                    left + 1,
                    self.theme.text_style(kat::DraculaColor::Comment)
                )?;
            }
            match &self.preview {
                Preview::Directory(entries) => {
                    for (row, entry) in entries.iter().take(body).enumerate() {
                        self.theme.write_entry(
                            &mut stdout,
                            entry,
                            (row + 1, left + 2),
                            right,
                            None,
                            &[],
                        )?;
                    }
                }
                Preview::Text(document) => {
                    // Use layout's display rows directly; never crop ANSI or
                    // wrap the highlighted output a second time.
                    for (row, line) in document.render(right).encoded_rows().take(body).enumerate()
                    {
                        write!(stdout, "\x1b[{};{}H{}\x1b[0m", row + 1, left + 2, line)?;
                    }
                }
                Preview::Message(message) => {
                    let document = kat::PreparedDocument::plain(&visible_text(message));
                    for (row, line) in document
                        .display_rows(right)
                        .into_iter()
                        .take(body)
                        .enumerate()
                    {
                        let color = if message.starts_with("Cannot preview") {
                            kat::DraculaColor::Red
                        } else {
                            kat::DraculaColor::Comment
                        };
                        write!(
                            stdout,
                            "\x1b[{};{}H{}{}\x1b[0m",
                            row + 1,
                            left + 2,
                            self.theme.text_style(color),
                            line.text()
                        )?;
                    }
                }
                Preview::Image { .. } => {
                    self.render_image(&mut stdout, right, body, 1, left + 2, options)?
                }
                Preview::Empty => {}
            }
            let status = if self.filter.editing {
                format!(
                    "/{}  [{}/{}]  Enter:confirm  Esc:clear",
                    self.filter.query,
                    self.filter.matches.len(),
                    self.entries.len()
                )
            } else {
                self.message.clone().unwrap_or_else(|| {
                    format!(
                        "{}/{}{}  q:quit  hjkl/arrows:move  o/O:cd  /:filter  R:reload",
                        if self.filter.matches.is_empty() {
                            0
                        } else {
                            self.selected + 1
                        },
                        self.filter.matches.len(),
                        if self.filter.query.is_empty() {
                            String::new()
                        } else {
                            format!(
                                "  /{} [{}/{}]",
                                self.filter.query,
                                self.filter.matches.len(),
                                self.entries.len()
                            )
                        }
                    )
                })
            };
            self.theme.write_status(
                &mut stdout,
                (size.rows, size.cols),
                &crate::ui_theme::display_path(&self.directory),
                &status,
                if self.message.is_some() {
                    kat::DraculaColor::Red
                } else {
                    kat::DraculaColor::Cyan
                },
            )?;
        }
        stdout.flush()?;
        Ok(())
    }

    fn render_image(
        &self,
        stdout: &mut impl Write,
        cols: usize,
        rows: usize,
        y: usize,
        x: usize,
        options: &CliOptions,
    ) -> Result<()> {
        if cols == 0 || rows == 0 {
            return Ok(());
        }
        if let Preview::Image { path, bytes } = &self.preview {
            let mut image_options = crate::image_render_options(options);
            image_options.width_cells = Some(cols);
            image_options.height_cells = Some(rows);
            image_options.fit = terminal_image::ImageFit::Contain;
            image_options.terminal_columns = Some(cols);
            image_options.terminal_rows = Some(rows);
            match terminal_image::render_browser_image(path, bytes.clone(), image_options) {
                Ok(image) if image.contains_terminal_image => {
                    write!(
                        stdout,
                        "\x1b[{y};{x}H{}",
                        image.output.trim_end_matches('\n')
                    )?;
                }
                result => {
                    let text = match result {
                        Ok(image) => image.output,
                        Err(error) => format!("Cannot preview image: {error:#}"),
                    };
                    let document = kat::PreparedDocument::plain(&visible_text(&text));
                    for (offset, line) in document
                        .display_rows(cols)
                        .into_iter()
                        .take(rows)
                        .enumerate()
                    {
                        write!(
                            stdout,
                            "\x1b[{};{x}H{}{}\x1b[0m",
                            y + offset,
                            self.theme.text_style(kat::DraculaColor::Comment),
                            line.text()
                        )?;
                    }
                }
            }
        }
        Ok(())
    }
}

fn prepare_preview(
    entry: &Entry,
    options: &CliOptions,
    render_options: kat::RenderOptions,
    limit: Option<u64>,
) -> Result<Preview> {
    if entry.directory {
        let entries = read_entries(&entry.path)?;
        return Ok(if entries.is_empty() {
            Preview::Message("Empty directory".to_owned())
        } else {
            Preview::Directory(entries)
        });
    }
    let metadata = fs::metadata(&entry.path)?;
    if !metadata.is_file() {
        return Ok(Preview::Message("Special file (not opened)".to_owned()));
    }
    let mut bytes = Vec::new();
    fs::File::open(&entry.path)?
        .take(limit.map_or(u64::MAX, |limit| limit + 1))
        .read_to_end(&mut bytes)?;
    if limit.is_some_and(|limit| bytes.len() as u64 > limit) {
        return Ok(Preview::Message(format!(
            "Large file ({} bytes). Press Enter, l or Right to open.",
            metadata.len()
        )));
    }
    if crate::should_render_path_as_image(options, &entry.path, &bytes) {
        return Ok(Preview::Image {
            path: entry.path.clone(),
            bytes,
        });
    }
    if bytes.contains(&0) {
        return Ok(Preview::Message(format!(
            "Binary file ({} bytes)",
            bytes.len()
        )));
    }
    let source = String::from_utf8_lossy(&bytes);
    let document = match options.language.as_deref() {
        Some(language) => kat::PreparedDocument::named_language(language, &source, render_options)?,
        None => kat::PreparedDocument::detect(Some(&entry.path), &source, render_options)?,
    };
    Ok(Preview::Text(ViewerDocument {
        segments: vec![ViewerSegment::Prepared {
            path: Some(entry.path.clone()),
            document: Box::new(document),
            render_options,
            fingerprint: content_fingerprint(&bytes),
        }],
    }))
}

fn write_plain(
    stdout: &mut impl Write,
    y: usize,
    x: usize,
    text: &str,
    cols: usize,
    style: anstyle::Style,
) -> Result<()> {
    let document = kat::PreparedDocument::plain(&visible_text(text));
    write!(stdout, "\x1b[{y};{x}H\x1b[0m")?;
    write!(stdout, "{style}")?;
    if let Some(row) = document.display_rows(cols).first() {
        stdout.write_all(row.text().as_bytes())?;
    }
    stdout.write_all(b"\x1b[0m")?;
    Ok(())
}

fn is_interrupt(event: &InputEvent) -> bool {
    matches!(
        event,
        InputEvent::Key(KeyEvent {
            key: KeyCode::Char('\x03'),
            ..
        })
    ) || matches!(event, InputEvent::Key(KeyEvent { key: KeyCode::Char('c' | 'C'), modifiers, .. }) if modifiers.contains(Modifiers::CTRL))
}

fn is_parent_key(event: &InputEvent) -> bool {
    matches!(event, InputEvent::Key(key) if matches!(Action::parse(key), Some(Action::Parent)))
}

#[derive(Clone, Copy)]
enum Action {
    Quit,
    Down,
    Up,
    Parent,
    Navigate,
    OpenDirectory,
    OpenCurrentDirectory,
    Filter,
    First,
    Last,
    PageDown,
    PageUp,
    Reload,
}

impl Action {
    fn parse(key: &KeyEvent) -> Option<Self> {
        if key.modifiers.intersects(Modifiers::CTRL | Modifiers::ALT) {
            return None;
        }
        Some(match key.key {
            KeyCode::Char('q') | KeyCode::Escape => Self::Quit,
            KeyCode::Char('j') | KeyCode::DownArrow | KeyCode::ApplicationDownArrow => Self::Down,
            KeyCode::Char('k') | KeyCode::UpArrow | KeyCode::ApplicationUpArrow => Self::Up,
            KeyCode::Char('h') | KeyCode::LeftArrow | KeyCode::ApplicationLeftArrow => Self::Parent,
            KeyCode::Char('l') | KeyCode::RightArrow | KeyCode::ApplicationRightArrow => {
                Self::Navigate
            }
            KeyCode::Enter => Self::Navigate,
            KeyCode::Char('o') => Self::OpenDirectory,
            KeyCode::Char('O') => Self::OpenCurrentDirectory,
            KeyCode::Char('/') => Self::Filter,
            KeyCode::Char('g') | KeyCode::Home => Self::First,
            KeyCode::Char('G') | KeyCode::End => Self::Last,
            KeyCode::PageDown => Self::PageDown,
            KeyCode::PageUp => Self::PageUp,
            KeyCode::Char('R') => Self::Reload,
            _ => return None,
        })
    }
}
