use std::{
    env,
    ffi::OsStr,
    ffi::OsString,
    fs,
    io::{self, IsTerminal, Read, Write},
    path::PathBuf,
    process::{Command, ExitCode, Stdio},
    time::{Duration, Instant},
};

use anyhow::{Context, Result, bail};
use clap::{ArgAction, CommandFactory, FromArgMatches, Parser, ValueEnum};
use clap_complete::engine::{ArgValueCompleter, CompletionCandidate};
use clap_complete::env::{Bash, Elvish, EnvCompleter, Fish, Powershell, Zsh};
use miette::{Report, miette};
use shadow_rs::shadow;
use terminal_size::{Height, Width, terminal_size};

const DEFAULT_TERMINAL_ROWS: usize = 24;

shadow!(build);

fn main() -> ExitCode {
    if let Some(exit_code) = complete_env() {
        return exit_code;
    }

    match try_main() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            if let Some(clap_error) = error.downcast_ref::<clap::Error>() {
                clap_error
                    .print()
                    .expect("failed to print clap parsing error");
                return ExitCode::from(clap_error.exit_code() as u8);
            }

            eprintln!("{}", format_cli_error(&error));
            ExitCode::FAILURE
        }
    }
}

fn complete_env() -> Option<ExitCode> {
    let shell_name = std::env::var_os("COMPLETE")?;
    if shell_name.is_empty() || shell_name == "0" {
        return None;
    }

    let shell = match env_completer(std::path::Path::new(&shell_name)) {
        Ok(shell) => shell,
        Err(error) => {
            error.print().expect("failed to print completion error");
            return Some(ExitCode::from(error.exit_code() as u8));
        }
    };

    // SAFETY: mirrors clap_complete's initialization behavior and runs before app logic.
    unsafe {
        std::env::remove_var("COMPLETE");
    }

    let mut argv = std::env::args_os().collect::<Vec<_>>();
    let completer = argv.remove(0);
    let escape_index = argv
        .iter()
        .position(|arg| *arg == "--")
        .map(|index| index + 1)
        .unwrap_or(argv.len());
    argv.drain(0..escape_index);

    let current_dir = std::env::current_dir().ok();
    let mut buf = Vec::new();
    if argv.is_empty() {
        let cmd = completion_command();
        let name = cmd.get_name().to_owned();
        let bin = cmd.get_bin_name().unwrap_or(&name).to_owned();
        let completer = completer.to_string_lossy().into_owned();
        shell
            .write_registration("COMPLETE", &name, &bin, &completer, &mut buf)
            .expect("failed to write completion registration");
    } else {
        let mut cmd = completion_command_for(&argv);
        cmd.build();
        shell
            .write_complete(&mut cmd, argv, current_dir.as_deref(), &mut buf)
            .expect("failed to write dynamic completions");
    }
    std::io::stdout()
        .write_all(&buf)
        .expect("failed to write completion output");

    Some(ExitCode::SUCCESS)
}

fn env_completer(name: &std::path::Path) -> clap::error::Result<Box<dyn EnvCompleter>> {
    let name = name
        .file_stem()
        .unwrap_or(name.as_os_str())
        .to_string_lossy();
    let shell: Box<dyn EnvCompleter> = if Bash.is(&name) {
        Box::new(Bash)
    } else if Elvish.is(&name) {
        Box::new(Elvish)
    } else if Fish.is(&name) {
        Box::new(Fish)
    } else if Powershell.is(&name) {
        Box::new(Powershell)
    } else if Zsh.is(&name) {
        Box::new(Zsh)
    } else {
        return Err(clap::Error::raw(
            clap::error::ErrorKind::InvalidValue,
            format!("unsupported completion shell `{name}`"),
        ));
    };
    Ok(shell)
}

fn try_main() -> Result<()> {
    let options = parse_cli_args(env::args_os().skip(1))?;
    let total_started_at = Instant::now();
    let mut built_output = build_output(&options)?;
    let write_started_at = Instant::now();
    write_output(&built_output.output, &options)?;

    if let Some(timings) = built_output.timings.as_mut() {
        timings.write_output += write_started_at.elapsed();
        timings.total = total_started_at.elapsed();
        eprintln!("{}", timings.format());
    }

    Ok(())
}

#[derive(Debug)]
enum ReadSourceErrorKind {
    Directory,
    Io(io::Error),
}

#[derive(Debug)]
struct ReadSourceError {
    path: PathBuf,
    kind: ReadSourceErrorKind,
}

impl ReadSourceError {
    fn directory(path: PathBuf) -> Self {
        Self {
            path,
            kind: ReadSourceErrorKind::Directory,
        }
    }

    fn io(path: PathBuf, error: io::Error) -> Self {
        Self {
            path,
            kind: ReadSourceErrorKind::Io(error),
        }
    }

    fn path(&self) -> &std::path::Path {
        &self.path
    }

    fn io_error(&self) -> Option<&io::Error> {
        match &self.kind {
            ReadSourceErrorKind::Directory => None,
            ReadSourceErrorKind::Io(error) => Some(error),
        }
    }
}

impl std::fmt::Display for ReadSourceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            ReadSourceErrorKind::Directory => write!(
                f,
                "failed to read {}: path is a directory; pass a file path instead",
                self.path.display()
            ),
            ReadSourceErrorKind::Io(_) => write!(f, "failed to read {}", self.path.display()),
        }
    }
}

impl std::error::Error for ReadSourceError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.io_error()
            .map(|error| error as &(dyn std::error::Error + 'static))
    }
}

fn format_cli_error(error: &anyhow::Error) -> String {
    if let Some(read_error) = error.downcast_ref::<ReadSourceError>() {
        if let Some(io_error) = read_error.io_error() {
            let report = miette!("'{}': {}", read_error.path().display(), io_error);
            return format!("{report:?}");
        }

        let report = miette!(
            help = "pass a file path instead",
            "'{}' is a directory",
            read_error.path().display()
        );
        return format!("{report:?}");
    }

    format!("{:?}", Report::msg(error.to_string()))
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum OutputMode {
    Render,
    DebugAst,
    DebugAnalysis,
    DebugSemantics,
    DebugVisual,
    DebugLayout,
    DebugRenderOps,
    DebugTerminal,
    Version,
}

#[derive(Debug, Eq, PartialEq)]
struct CliOptions {
    mode: OutputMode,
    paging: PagingMode,
    debug_timing: bool,
    language: Option<String>,
    paths: Vec<PathBuf>,
}

#[derive(Debug, Default)]
struct DebugTimingStats {
    files: usize,
    input_bytes: usize,
    output_bytes: usize,
    read_source: Duration,
    render_pipeline: kat::RenderTimings,
    write_output: Duration,
    total: Duration,
}

impl DebugTimingStats {
    fn format(&self) -> String {
        format!(
            concat!(
                "kat timing",
                " files={}",
                " input_bytes={}",
                " output_bytes={}",
                " read={}",
                " detect={}",
                " highlight={}",
                " semantic={}",
                " injections={}",
                " nested_overlays={}",
                " ansi_render={}",
                " pipeline={}",
                " write={}",
                " total={}"
            ),
            self.files,
            self.input_bytes,
            self.output_bytes,
            format_duration(self.read_source),
            format_duration(self.render_pipeline.detect_document_kind),
            format_duration(self.render_pipeline.highlight),
            format_duration(self.render_pipeline.semantic_overlays),
            format_duration(self.render_pipeline.injection_regions),
            format_duration(self.render_pipeline.nested_region_overlays),
            format_duration(self.render_pipeline.render_styled_spans),
            format_duration(self.render_pipeline.total_render_pipeline()),
            format_duration(self.write_output),
            format_duration(self.total),
        )
    }
}

#[derive(Debug)]
struct BuiltOutput {
    output: String,
    timings: Option<DebugTimingStats>,
}

fn format_duration(duration: Duration) -> String {
    format!("{:.3}ms", duration.as_secs_f64() * 1_000.0)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
enum PagingMode {
    Auto,
    Always,
    Never,
}

#[derive(Debug, Parser)]
#[command(
    name = "kat",
    disable_help_flag = true,
    disable_help_subcommand = true,
    disable_version_flag = true
)]
struct CliArgs {
    #[arg(long = "debug-ast", group = "mode")]
    debug_ast: bool,
    #[arg(long = "debug-analysis", group = "mode")]
    debug_analysis: bool,
    #[arg(long = "debug-semantics", group = "mode")]
    debug_semantics: bool,
    #[arg(long = "debug-shell-semantics", group = "mode")]
    debug_shell_semantics: bool,
    #[arg(long = "debug-visual", group = "mode")]
    debug_visual: bool,
    #[arg(long = "debug-layout", group = "mode")]
    debug_layout: bool,
    #[arg(long = "debug-render-ops", group = "mode")]
    debug_render_ops: bool,
    #[arg(long = "debug-terminal", group = "mode")]
    debug_terminal: bool,
    #[arg(long = "debug-timing")]
    debug_timing: bool,
    #[arg(long = "help", short = 'h', action = ArgAction::Help)]
    help: Option<bool>,
    #[arg(long = "version", short = 'V', group = "mode")]
    version: bool,
    #[arg(long)]
    language: Option<String>,
    #[arg(long, value_enum, default_value_t = PagingMode::Auto)]
    paging: PagingMode,
    #[arg(
        value_name = "PATH|-",
        add = ArgValueCompleter::new(complete_input_paths)
    )]
    paths: Vec<PathBuf>,
}

fn cli_command() -> clap::Command {
    CliArgs::command()
}

fn completion_command() -> clap::Command {
    completion_command_for(&[])
}

fn completion_command_for(args: &[OsString]) -> clap::Command {
    let mut command = cli_command();
    for arg_id in [
        "debug_ast",
        "debug_analysis",
        "debug_semantics",
        "debug_shell_semantics",
        "debug_visual",
        "debug_render_ops",
        "debug_terminal",
        "debug_timing",
    ] {
        command = command.mut_arg(arg_id, |arg| arg.hide(true));
    }

    let current_token = args.last().and_then(|arg| arg.to_str()).unwrap_or_default();
    if current_token.starts_with('-') {
        command = command.mut_arg("paths", |arg| arg.hide(true));
    } else {
        for arg_id in ["help", "version", "language", "paging"] {
            command = command.mut_arg(arg_id, |arg| arg.hide(true));
        }
    }

    command
}

fn complete_input_paths(current: &OsStr) -> Vec<CompletionCandidate> {
    let Some((display_prefix, search_root, fragment)) = resolve_completion_root(current) else {
        return Vec::new();
    };

    let fragment = fragment.to_string_lossy();
    let mut prefix_matches = Vec::new();
    let mut substring_matches = Vec::new();
    for entry in fs::read_dir(search_root)
        .ok()
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
    {
        let file_name = entry.file_name();
        let file_name_text = file_name.to_string_lossy();

        let mut suggestion = display_prefix.join(&file_name);
        let is_dir = entry.path().is_dir();
        if is_dir {
            suggestion.push("");
        } else if !entry.path().is_file() {
            continue;
        }

        let candidate =
            CompletionCandidate::new(suggestion.into_os_string()).hide(is_hidden_path(&file_name));
        if case_insensitive_starts_with(&file_name_text, &fragment) {
            prefix_matches.push((completion_kind_rank(is_dir), candidate));
            continue;
        }

        if should_include_substring_match(&file_name_text, &fragment) {
            substring_matches.push((completion_kind_rank(is_dir), candidate));
        }
    }

    prefix_matches.sort_by(|left, right| left.0.cmp(&right.0).then_with(|| left.1.cmp(&right.1)));
    substring_matches
        .sort_by(|left, right| left.0.cmp(&right.0).then_with(|| left.1.cmp(&right.1)));

    let mut candidates = prefix_matches
        .into_iter()
        .map(|(_, candidate)| candidate)
        .collect::<Vec<_>>();
    candidates.extend(
        substring_matches
            .into_iter()
            .map(|(_, candidate)| candidate),
    );
    candidates
}

fn case_insensitive_starts_with(candidate: &str, prefix: &str) -> bool {
    if prefix.is_empty() {
        return true;
    }

    let candidate = candidate.to_lowercase();
    let prefix = prefix.to_lowercase();
    candidate.starts_with(&prefix)
}

fn should_include_substring_match(candidate: &str, fragment: &str) -> bool {
    if fragment.is_empty() || fragment.starts_with('.') {
        return false;
    }

    let candidate = candidate.to_lowercase();
    let fragment = fragment.to_lowercase();
    candidate.contains(&fragment)
}

fn completion_kind_rank(is_dir: bool) -> u8 {
    if is_dir { 1 } else { 0 }
}

fn resolve_completion_root(current: &OsStr) -> Option<(PathBuf, PathBuf, OsString)> {
    let (prefix, fragment) = split_completion_path(current);

    if prefix.is_absolute() {
        return Some((
            prefix.to_path_buf(),
            prefix.to_path_buf(),
            fragment.to_os_string(),
        ));
    }

    if prefix.iter().next() == Some(OsStr::new("~")) {
        let home = env::var_os("HOME").map(PathBuf::from)?;
        let home_relative = prefix.strip_prefix("~").ok()?;
        return Some((
            prefix.to_path_buf(),
            home.join(home_relative),
            fragment.to_os_string(),
        ));
    }

    let current_dir = env::current_dir().ok()?;
    Some((prefix.clone(), current_dir.join(&prefix), fragment))
}

fn split_completion_path(path: &OsStr) -> (PathBuf, OsString) {
    let path = path.to_string_lossy();
    if path.is_empty() {
        return (PathBuf::new(), OsString::new());
    }

    let separator = std::path::MAIN_SEPARATOR;
    if path == "." || path.ends_with(&format!("{separator}.")) {
        let prefix = if path == "." {
            PathBuf::new()
        } else {
            PathBuf::from(&path[..path.len() - 2])
        };
        return (prefix, OsString::from("."));
    }

    if path == ".." || path.ends_with(&format!("{separator}..")) {
        let prefix = if path == ".." {
            PathBuf::new()
        } else {
            PathBuf::from(&path[..path.len() - 3])
        };
        return (prefix, OsString::from(".."));
    }

    if path.ends_with(separator) {
        return (PathBuf::from(path.as_ref()), OsString::new());
    }

    if let Some(index) = path.rfind(separator) {
        let prefix = if index == 0 {
            separator.to_string()
        } else {
            path[..index].to_owned()
        };
        return (PathBuf::from(prefix), OsString::from(&path[index + 1..]));
    }

    (PathBuf::new(), OsString::from(path.as_ref()))
}

fn is_hidden_path(file_name: &OsStr) -> bool {
    file_name.to_string_lossy().starts_with('.')
}

fn parse_cli_args(args: impl IntoIterator<Item = OsString>) -> Result<CliOptions> {
    let cli = CliArgs::from_arg_matches_mut(
        &mut cli_command()
            .try_get_matches_from_mut(std::iter::once(OsString::from("kat")).chain(args))?,
    )?;

    let mode = if cli.version {
        OutputMode::Version
    } else if cli.debug_ast {
        OutputMode::DebugAst
    } else if cli.debug_analysis {
        OutputMode::DebugAnalysis
    } else if cli.debug_semantics || cli.debug_shell_semantics {
        OutputMode::DebugSemantics
    } else if cli.debug_visual {
        OutputMode::DebugVisual
    } else if cli.debug_layout {
        OutputMode::DebugLayout
    } else if cli.debug_render_ops {
        OutputMode::DebugRenderOps
    } else if cli.debug_terminal {
        OutputMode::DebugTerminal
    } else {
        OutputMode::Render
    };

    Ok(CliOptions {
        mode,
        paging: cli.paging,
        debug_timing: cli.debug_timing,
        language: cli.language,
        paths: cli.paths,
    })
}

fn render_output(
    mode: OutputMode,
    language: Option<&str>,
    source_path: Option<&std::path::Path>,
    source: &str,
) -> Result<String> {
    match mode {
        OutputMode::Render => kat::render(source_path, source),
        OutputMode::DebugAst => {
            let language_name = resolve_debug_language_name(language, source_path, source)?;
            let mut output = kat::debug_named_language_tree(&language_name, source)?;
            if !output.ends_with('\n') {
                output.push('\n');
            }
            Ok(output)
        }
        OutputMode::DebugAnalysis => render_debug_json(
            language,
            source_path,
            source,
            kat::debug_analysis_json,
            kat::debug_named_language_analysis_json,
        ),
        OutputMode::DebugSemantics => {
            let language_name = resolve_debug_language_name(language, source_path, source)?;
            let mut output = kat::debug_semantics(&language_name, source)?;
            if !output.ends_with('\n') {
                output.push('\n');
            }
            Ok(output)
        }
        OutputMode::DebugVisual => render_debug_json(
            language,
            source_path,
            source,
            kat::debug_visual_json,
            kat::debug_named_language_visual_json,
        ),
        OutputMode::DebugLayout => render_debug_json(
            language,
            source_path,
            source,
            kat::debug_layout_json,
            kat::debug_named_language_layout_json,
        ),
        OutputMode::DebugRenderOps => render_debug_json(
            language,
            source_path,
            source,
            kat::debug_render_ops_json,
            kat::debug_named_language_render_ops_json,
        ),
        OutputMode::DebugTerminal => render_debug_json(
            language,
            source_path,
            source,
            kat::debug_terminal_json,
            kat::debug_named_language_terminal_json,
        ),
        OutputMode::Version => Ok(version_output()),
    }
}

fn render_debug_json(
    language: Option<&str>,
    source_path: Option<&std::path::Path>,
    source: &str,
    auto_debug: impl Fn(Option<&std::path::Path>, &str) -> Result<String>,
    named_debug: impl Fn(&str, &str) -> Result<String>,
) -> Result<String> {
    let mut output = match language {
        Some(language_name) => named_debug(language_name, source)?,
        None => auto_debug(source_path, source)?,
    };
    if !output.ends_with('\n') {
        output.push('\n');
    }
    Ok(output)
}

fn render_output_with_timing(
    options: &CliOptions,
    source_path: Option<&std::path::Path>,
    source: &str,
    timings: Option<&mut DebugTimingStats>,
) -> Result<String> {
    if matches!(options.mode, OutputMode::Render) {
        let terminal_width = io::stdout().is_terminal().then(terminal_columns).flatten();
        let render_output =
            kat::render_with_timing_and_terminal_width(source_path, source, terminal_width)?;
        if let Some(timings) = timings {
            timings.render_pipeline.detect_document_kind +=
                render_output.timings.detect_document_kind;
            timings.render_pipeline.highlight += render_output.timings.highlight;
            timings.render_pipeline.semantic_overlays += render_output.timings.semantic_overlays;
            timings.render_pipeline.injection_regions += render_output.timings.injection_regions;
            timings.render_pipeline.nested_region_overlays +=
                render_output.timings.nested_region_overlays;
            timings.render_pipeline.render_styled_spans +=
                render_output.timings.render_styled_spans;
        }
        return Ok(render_output.output);
    }

    render_output(
        options.mode,
        options.language.as_deref(),
        source_path,
        source,
    )
}

fn resolve_debug_language_name(
    explicit_language: Option<&str>,
    source_path: Option<&std::path::Path>,
    source: &str,
) -> Result<String> {
    if let Some(language) = explicit_language {
        return Ok(language.to_owned());
    }

    kat::detected_language_name(source_path, source)
        .map(str::to_owned)
        .ok_or_else(|| {
            anyhow::anyhow!("could not detect language for debug output; pass --language <name>")
        })
}

fn version_output() -> String {
    let mut output = String::new();
    output.push_str(&format!("kat {}\n", build::PKG_VERSION));
    output.push_str(&format!("branch: {}\n", build::BRANCH));
    output.push_str(&format!("tag: {}\n", build::TAG));
    output.push_str(&format!("commit: {}\n", build::COMMIT_HASH));
    output.push_str(&format!("short-commit: {}\n", build::SHORT_COMMIT));
    output.push_str(&format!("commit-date: {}\n", build::COMMIT_DATE));
    output.push_str(&format!(
        "commit-author: {} <{}>\n",
        build::COMMIT_AUTHOR,
        build::COMMIT_EMAIL
    ));
    output.push_str(&format!("build-time: {}\n", build::BUILD_TIME));
    output.push_str(&format!("build-channel: {}\n", build::BUILD_RUST_CHANNEL));
    output.push_str(&format!("build-os: {}\n", build::BUILD_OS));
    output.push_str(&format!("build-target: {}\n", build::BUILD_TARGET));
    output.push_str(&format!("rust-version: {}\n", build::RUST_VERSION));
    output.push_str(&format!("rust-channel: {}\n", build::RUST_CHANNEL));
    output.push_str(&format!("cargo-version: {}\n", build::CARGO_VERSION));
    output.push_str(&format!("git-clean: {}\n", build::GIT_CLEAN));

    let git_status = build::GIT_STATUS_FILE.trim();
    if git_status.is_empty() {
        output.push_str("git-status: clean\n");
    } else {
        output.push_str("git-status:\n");
        for line in git_status.lines() {
            output.push_str("  ");
            output.push_str(line.trim_start());
            output.push('\n');
        }
    }

    output
}

#[derive(Debug, Eq, PartialEq)]
struct PagerCommand {
    program: OsString,
    args: Vec<OsString>,
}

fn should_page_output(output: &str, paging: PagingMode, stdout_is_terminal: bool) -> bool {
    should_page_output_with_rows(output, paging, stdout_is_terminal, terminal_rows())
}

fn should_page_output_with_rows(
    output: &str,
    paging: PagingMode,
    stdout_is_terminal: bool,
    terminal_rows: Option<usize>,
) -> bool {
    if !stdout_is_terminal {
        return false;
    }

    match paging {
        PagingMode::Never => false,
        PagingMode::Always => true,
        PagingMode::Auto => {
            count_output_lines(output) > terminal_rows.unwrap_or(DEFAULT_TERMINAL_ROWS)
        }
    }
}

fn resolve_pager_command(pager: Option<&OsStr>) -> Result<Option<PagerCommand>> {
    let Some(pager) = pager else {
        return Ok(Some(PagerCommand {
            program: OsString::from("less"),
            args: vec![
                OsString::from("-R"),
                OsString::from("-F"),
                OsString::from("-X"),
            ],
        }));
    };

    let Some(pager) = pager.to_str() else {
        bail!("PAGER contains invalid UTF-8");
    };
    let Some(parts) = shlex::split(pager) else {
        bail!("failed to parse PAGER command");
    };
    if parts.is_empty() {
        return Ok(None);
    }

    let mut parts = parts.into_iter();
    let program = OsString::from(parts.next().expect("pager command should not be empty"));
    Ok(Some(PagerCommand {
        program,
        args: parts.map(OsString::from).collect(),
    }))
}

fn page_output_via_command(output: &str, pager: &PagerCommand) -> Result<()> {
    let mut child = Command::new(&pager.program)
        .args(&pager.args)
        .stdin(Stdio::piped())
        .spawn()
        .with_context(|| format!("failed to start pager {:?}", pager.program))?;
    let mut stdin = child.stdin.take().context("failed to open pager stdin")?;
    stdin.write_all(output.as_bytes())?;
    drop(stdin);
    let status = child.wait().context("failed to wait for pager")?;
    if !status.success() {
        bail!("pager exited with status {status}");
    }
    Ok(())
}

fn build_output(options: &CliOptions) -> Result<BuiltOutput> {
    if matches!(options.mode, OutputMode::Version) {
        return Ok(BuiltOutput {
            output: version_output(),
            timings: options.debug_timing.then_some(DebugTimingStats::default()),
        });
    }

    let mut timings = options.debug_timing.then_some(DebugTimingStats::default());

    if options.paths.is_empty() {
        let stdin = read_stdin().context("failed to read stdin")?;
        if let Some(timings) = timings.as_mut() {
            timings.files += 1;
            timings.input_bytes += stdin.len();
        }
        let output = render_output_with_timing(options, None, &stdin, timings.as_mut())?;
        if let Some(timings) = timings.as_mut() {
            timings.output_bytes = output.len();
        }
        return Ok(BuiltOutput { output, timings });
    }

    let mut output = String::new();
    let multiple_paths = options.paths.len() > 1;

    for (index, path) in options.paths.iter().enumerate() {
        if path.as_os_str() == OsStr::new("-") {
            let stdin = read_stdin().context("failed to read stdin")?;
            if let Some(timings) = timings.as_mut() {
                timings.files += 1;
                timings.input_bytes += stdin.len();
            }
            if multiple_paths {
                push_header(&mut output, "-", index > 0);
            }
            output.push_str(&render_output_with_timing(
                options,
                None,
                &stdin,
                timings.as_mut(),
            )?);
            continue;
        }

        let read_started_at = Instant::now();
        let source = read_source_from_path(path)?;
        if let Some(timings) = timings.as_mut() {
            timings.files += 1;
            timings.input_bytes += source.len();
            timings.read_source += read_started_at.elapsed();
        }
        if multiple_paths {
            push_header(&mut output, &path.display().to_string(), index > 0);
        }
        output.push_str(&render_output_with_timing(
            options,
            Some(path.as_path()),
            &source,
            timings.as_mut(),
        )?);
    }

    if let Some(timings) = timings.as_mut() {
        timings.output_bytes = output.len();
    }

    Ok(BuiltOutput { output, timings })
}

fn write_output(output: &str, options: &CliOptions) -> Result<()> {
    let should_page = matches!(options.mode, OutputMode::Render)
        && should_page_output(output, options.paging, io::stdout().is_terminal());
    if !should_page {
        return write_output_direct(output);
    }

    let pager = resolve_pager_command(env::var_os("PAGER").as_deref())?;
    let Some(pager) = pager else {
        return write_output_direct(output);
    };

    match page_output_via_command(output, &pager) {
        Ok(()) => Ok(()),
        Err(error) if matches!(options.paging, PagingMode::Always) => Err(error),
        Err(_) => write_output_direct(output),
    }
}

fn write_output_direct(output: &str) -> Result<()> {
    let mut stdout = io::stdout().lock();
    stdout
        .write_all(output.as_bytes())
        .context("failed to write output")?;
    stdout.flush().context("failed to flush stdout")?;
    Ok(())
}

fn push_header(output: &mut String, label: &str, add_leading_newline: bool) {
    if add_leading_newline {
        output.push('\n');
    }
    output.push_str("==> ");
    output.push_str(label);
    output.push_str(" <==\n");
}

fn terminal_rows() -> Option<usize> {
    if let Some((_, Height(rows))) = terminal_size() {
        return Some(usize::from(rows).max(1));
    }

    env::var("LINES")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .filter(|rows| *rows > 0)
}

fn terminal_columns() -> Option<usize> {
    if let Some((Width(columns), _)) = terminal_size() {
        return Some(usize::from(columns).max(1));
    }

    env::var("COLUMNS")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .filter(|columns| *columns > 0)
}

fn count_output_lines(output: &str) -> usize {
    if output.is_empty() {
        return 0;
    }

    output.bytes().filter(|byte| *byte == b'\n').count() + usize::from(!output.ends_with('\n'))
}

fn read_source_from_path(path: &PathBuf) -> Result<String> {
    if path.is_dir() {
        return Err(ReadSourceError::directory(path.clone()).into());
    }

    let bytes = fs::read(path).map_err(|error| ReadSourceError::io(path.clone(), error))?;
    Ok(String::from_utf8_lossy(&bytes).into_owned())
}

fn read_stdin() -> io::Result<String> {
    let mut buffer = String::new();
    io::stdin().read_to_string(&mut buffer)?;
    Ok(buffer)
}

#[cfg(test)]
mod tests {
    use std::{
        ffi::OsStr,
        ffi::OsString,
        fs,
        path::PathBuf,
        time::{Duration, SystemTime, UNIX_EPOCH},
    };

    use clap_complete::engine::complete;
    use std::path::Path;

    use super::{
        CliOptions, DebugTimingStats, OutputMode, PagerCommand, PagingMode, cli_command,
        completion_command_for, format_cli_error, page_output_via_command, parse_cli_args,
        read_source_from_path, render_output, resolve_pager_command, should_page_output,
        version_output,
    };

    #[test]
    fn directory_path_reports_actionable_error() {
        let dir = unique_temp_dir("kat-directory-error");
        fs::create_dir_all(&dir)
            .unwrap_or_else(|error| panic!("failed to create temp dir {}: {error}", dir.display()));

        let error = read_source_from_path(&dir).expect_err("directory path should fail");
        let message = format!("{error:#}");
        assert!(
            message.contains("is a directory") && message.contains("pass a file path instead"),
            "unexpected error message: {message}"
        );

        fs::remove_dir_all(&dir)
            .unwrap_or_else(|error| panic!("failed to clean temp dir {}: {error}", dir.display()));
    }

    #[test]
    fn missing_file_formats_as_single_line_cli_error() {
        let missing = unique_temp_dir("kat-missing-file").join("con");
        let error = read_source_from_path(&missing).expect_err("missing file should fail");
        let message = format_cli_error(&error);

        assert!(
            message.contains("con':")
                && message.contains("No such file or directory")
                && message.contains("(os error 2)"),
            "unexpected missing-file cli error: {message}"
        );
    }

    #[test]
    fn directory_error_keeps_actionable_cli_message() {
        let dir = unique_temp_dir("kat-directory-cli-error");
        fs::create_dir_all(&dir)
            .unwrap_or_else(|error| panic!("failed to create temp dir {}: {error}", dir.display()));

        let error = read_source_from_path(&dir).expect_err("directory path should fail");
        let message = format_cli_error(&error);
        assert!(
            message.contains("is a directory") && message.contains("pass a file path instead"),
            "unexpected cli error message: {message}"
        );

        fs::remove_dir_all(&dir)
            .unwrap_or_else(|error| panic!("failed to clean temp dir {}: {error}", dir.display()));
    }

    fn unique_temp_dir(prefix: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time before unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("{prefix}-{nanos}"))
    }

    #[test]
    fn parses_debug_shell_semantics_language_and_path() {
        let options = parse_cli_args([
            OsString::from("--debug-shell-semantics"),
            OsString::from("--language"),
            OsString::from("fish"),
            OsString::from("testdata/fixtures/fish/rich.fish"),
        ])
        .expect("failed to parse cli args");

        assert_eq!(
            options,
            CliOptions {
                mode: OutputMode::DebugSemantics,
                paging: PagingMode::Auto,
                debug_timing: false,
                language: Some("fish".to_owned()),
                paths: vec![PathBuf::from("testdata/fixtures/fish/rich.fish")],
            }
        );
    }

    #[test]
    fn parses_debug_semantics_language_and_path() {
        let options = parse_cli_args([
            OsString::from("--debug-semantics"),
            OsString::from("--language=regex"),
            OsString::from("pattern.re"),
        ])
        .expect("failed to parse cli args");

        assert_eq!(
            options,
            CliOptions {
                mode: OutputMode::DebugSemantics,
                paging: PagingMode::Auto,
                debug_timing: false,
                language: Some("regex".to_owned()),
                paths: vec![PathBuf::from("pattern.re")],
            }
        );
    }

    #[test]
    fn parses_debug_analysis_flag() {
        let options = parse_cli_args([
            OsString::from("--debug-analysis"),
            OsString::from("docs/architecture.md"),
        ])
        .expect("failed to parse debug analysis flag");

        assert_eq!(
            options,
            CliOptions {
                mode: OutputMode::DebugAnalysis,
                paging: PagingMode::Auto,
                debug_timing: false,
                language: None,
                paths: vec![PathBuf::from("docs/architecture.md")],
            }
        );
    }

    #[test]
    fn parses_debug_visual_flag() {
        let options = parse_cli_args([
            OsString::from("--debug-visual"),
            OsString::from("--language"),
            OsString::from("markdown"),
            OsString::from("notes.md"),
        ])
        .expect("failed to parse debug visual flag");

        assert_eq!(
            options,
            CliOptions {
                mode: OutputMode::DebugVisual,
                paging: PagingMode::Auto,
                debug_timing: false,
                language: Some("markdown".to_owned()),
                paths: vec![PathBuf::from("notes.md")],
            }
        );
    }

    #[test]
    fn parses_debug_layout_flag() {
        let options =
            parse_cli_args([OsString::from("--debug-layout"), OsString::from("notes.md")])
                .expect("failed to parse debug layout flag");

        assert_eq!(
            options,
            CliOptions {
                mode: OutputMode::DebugLayout,
                paging: PagingMode::Auto,
                debug_timing: false,
                language: None,
                paths: vec![PathBuf::from("notes.md")],
            }
        );
    }

    #[test]
    fn parses_debug_render_ops_flag() {
        let options = parse_cli_args([
            OsString::from("--debug-render-ops"),
            OsString::from("notes.md"),
        ])
        .expect("failed to parse debug render ops flag");

        assert_eq!(
            options,
            CliOptions {
                mode: OutputMode::DebugRenderOps,
                paging: PagingMode::Auto,
                debug_timing: false,
                language: None,
                paths: vec![PathBuf::from("notes.md")],
            }
        );
    }

    #[test]
    fn parses_debug_terminal_flag() {
        let options = parse_cli_args([
            OsString::from("--debug-terminal"),
            OsString::from("notes.md"),
        ])
        .expect("failed to parse debug terminal flag");

        assert_eq!(
            options,
            CliOptions {
                mode: OutputMode::DebugTerminal,
                paging: PagingMode::Auto,
                debug_timing: false,
                language: None,
                paths: vec![PathBuf::from("notes.md")],
            }
        );
    }

    #[test]
    fn parses_paging_flag_and_language() {
        let options = parse_cli_args([
            OsString::from("--paging=never"),
            OsString::from("--language"),
            OsString::from("rust"),
            OsString::from("src/main.rs"),
        ])
        .expect("failed to parse cli args");

        assert_eq!(
            options,
            CliOptions {
                mode: OutputMode::Render,
                paging: PagingMode::Never,
                debug_timing: false,
                language: Some("rust".to_owned()),
                paths: vec![PathBuf::from("src/main.rs")],
            }
        );
    }

    #[test]
    fn parses_version_flag() {
        let options =
            parse_cli_args([OsString::from("--version")]).expect("failed to parse version flag");

        assert_eq!(
            options,
            CliOptions {
                mode: OutputMode::Version,
                paging: PagingMode::Auto,
                debug_timing: false,
                language: None,
                paths: vec![],
            }
        );
    }

    #[test]
    fn parses_debug_timing_flag() {
        let options = parse_cli_args([
            OsString::from("--debug-timing"),
            OsString::from("src/main.rs"),
        ])
        .expect("failed to parse debug timing flag");

        assert_eq!(
            options,
            CliOptions {
                mode: OutputMode::Render,
                paging: PagingMode::Auto,
                debug_timing: true,
                language: None,
                paths: vec![PathBuf::from("src/main.rs")],
            }
        );
    }

    #[test]
    fn version_output_includes_build_metadata() {
        let output = version_output();

        assert!(
            output.contains("kat ")
                && output.contains("commit:")
                && output.contains("build-time:")
                && output.contains("rust-version:")
                && output.contains("git-clean:"),
            "unexpected version output: {output}"
        );
    }

    #[test]
    fn debug_timing_format_is_machine_readable() {
        let report = DebugTimingStats {
            files: 2,
            input_bytes: 128,
            output_bytes: 256,
            read_source: Duration::from_millis(1),
            render_pipeline: kat::RenderTimings {
                detect_document_kind: Duration::from_millis(2),
                highlight: Duration::from_millis(3),
                semantic_overlays: Duration::from_millis(4),
                injection_regions: Duration::from_millis(5),
                nested_region_overlays: Duration::from_millis(6),
                render_styled_spans: Duration::from_millis(7),
            },
            write_output: Duration::from_millis(8),
            total: Duration::from_millis(36),
        };

        let output = report.format();
        for fragment in [
            "kat timing",
            "files=2",
            "input_bytes=128",
            "output_bytes=256",
            "read=1.000ms",
            "detect=2.000ms",
            "highlight=3.000ms",
            "semantic=4.000ms",
            "injections=5.000ms",
            "nested_overlays=6.000ms",
            "ansi_render=7.000ms",
            "pipeline=27.000ms",
            "write=8.000ms",
            "total=36.000ms",
        ] {
            assert!(
                output.contains(fragment),
                "expected debug timing output to contain {fragment}: {output}"
            );
        }
    }

    #[test]
    fn rejects_unknown_paging_mode() {
        let error = parse_cli_args([OsString::from("--paging=maybe")])
            .expect_err("unknown paging mode should fail");

        assert!(
            format!("{error:#}").contains("invalid value"),
            "unexpected error: {error:#}"
        );
    }

    #[test]
    fn debug_modes_require_only_one_flag() {
        let error = parse_cli_args([
            OsString::from("--debug-ast"),
            OsString::from("--debug-shell-semantics"),
        ])
        .expect_err("multiple debug flags should fail");

        assert!(
            format!("{error:#}").contains("cannot be used with"),
            "unexpected error: {error:#}"
        );
    }

    #[test]
    fn clap_command_configuration_is_valid() {
        cli_command().debug_assert();
    }

    #[test]
    fn positional_path_completion_includes_files_and_directories() {
        let dir = unique_temp_dir("kat-path-completion");
        fs::create_dir_all(&dir)
            .unwrap_or_else(|error| panic!("failed to create temp dir {}: {error}", dir.display()));
        let nested_dir = dir.join("nested");
        fs::create_dir_all(&nested_dir).unwrap_or_else(|error| {
            panic!(
                "failed to create nested temp dir {}: {error}",
                nested_dir.display()
            )
        });
        let nested_file = dir.join("notes.txt");
        fs::write(&nested_file, "kat\n").unwrap_or_else(|error| {
            panic!(
                "failed to write completion fixture {}: {error}",
                nested_file.display()
            )
        });

        let current = dir.join("n");
        let args = vec![OsString::from("kat"), current.into_os_string()];
        let completions = complete(&mut completion_command_for(&args), args.clone(), 1, None)
            .expect("failed to compute completions");

        let values = completions
            .iter()
            .map(|candidate| candidate.get_value().to_string_lossy().into_owned())
            .collect::<Vec<_>>();

        assert!(
            values.iter().any(|value| value.ends_with("notes.txt")),
            "expected file completion, got {values:?}"
        );
        assert!(
            values.iter().any(|value| value.ends_with("nested/")),
            "expected directory completion, got {values:?}"
        );
        assert!(
            values
                .iter()
                .all(|value| !value.ends_with("/.") && !value.ends_with("/..")),
            "expected dot navigation entries to stay hidden, got {values:?}"
        );

        fs::remove_dir_all(&dir)
            .unwrap_or_else(|error| panic!("failed to clean temp dir {}: {error}", dir.display()));
    }

    #[test]
    fn path_completion_matches_case_insensitively() {
        let dir = unique_temp_dir("kat-casefold-path-completion");
        fs::create_dir_all(&dir)
            .unwrap_or_else(|error| panic!("failed to create temp dir {}: {error}", dir.display()));
        let readme = dir.join("README.md");
        fs::write(&readme, "# kat\n").unwrap_or_else(|error| {
            panic!(
                "failed to write completion fixture {}: {error}",
                readme.display()
            )
        });

        let current = dir.join("read");
        let args = vec![OsString::from("kat"), current.into_os_string()];
        let completions = complete(&mut completion_command_for(&args), args.clone(), 1, None)
            .expect("failed to compute case-insensitive completions");
        let values = completions
            .iter()
            .map(|candidate| candidate.get_value().to_string_lossy().into_owned())
            .collect::<Vec<_>>();

        assert!(
            values.iter().any(|value| value.ends_with("README.md")),
            "expected lowercase prefix to match uppercase file names, got {values:?}"
        );

        fs::remove_dir_all(&dir)
            .unwrap_or_else(|error| panic!("failed to clean temp dir {}: {error}", dir.display()));
    }

    #[test]
    fn path_completion_falls_back_to_case_insensitive_substring_matches() {
        let dir = unique_temp_dir("kat-substring-path-completion");
        fs::create_dir_all(&dir)
            .unwrap_or_else(|error| panic!("failed to create temp dir {}: {error}", dir.display()));
        let cargo_toml = dir.join("Cargo.toml");
        fs::write(&cargo_toml, "[package]\nname = \"kat\"\n").unwrap_or_else(|error| {
            panic!(
                "failed to write completion fixture {}: {error}",
                cargo_toml.display()
            )
        });

        let current = dir.join("toml");
        let args = vec![OsString::from("kat"), current.into_os_string()];
        let completions = complete(&mut completion_command_for(&args), args.clone(), 1, None)
            .expect("failed to compute substring completions");
        let values = completions
            .iter()
            .map(|candidate| candidate.get_value().to_string_lossy().into_owned())
            .collect::<Vec<_>>();

        assert!(
            values.iter().any(|value| value.ends_with("Cargo.toml")),
            "expected substring completion to match Cargo.toml, got {values:?}"
        );

        fs::remove_dir_all(&dir)
            .unwrap_or_else(|error| panic!("failed to clean temp dir {}: {error}", dir.display()));
    }

    #[test]
    fn path_completion_keeps_prefix_matches_ahead_of_substring_matches() {
        let dir = unique_temp_dir("kat-path-completion-priority");
        fs::create_dir_all(&dir)
            .unwrap_or_else(|error| panic!("failed to create temp dir {}: {error}", dir.display()));
        let prefix_match = dir.join("tom-preview.txt");
        fs::write(&prefix_match, "theme = true\n").unwrap_or_else(|error| {
            panic!(
                "failed to write completion fixture {}: {error}",
                prefix_match.display()
            )
        });
        let substring_match = dir.join("Cargo.toml");
        fs::write(&substring_match, "[package]\nname = \"kat\"\n").unwrap_or_else(|error| {
            panic!(
                "failed to write completion fixture {}: {error}",
                substring_match.display()
            )
        });

        let current = dir.join("tom");
        let args = vec![OsString::from("kat"), current.into_os_string()];
        let completions = complete(&mut completion_command_for(&args), args.clone(), 1, None)
            .expect("failed to compute prioritized completions");
        let values = completions
            .iter()
            .map(|candidate| candidate.get_value().to_string_lossy().into_owned())
            .collect::<Vec<_>>();

        let prefix_index = values
            .iter()
            .position(|value| value.ends_with("tom-preview.txt"))
            .expect("expected prefix match to be present");
        let substring_index = values
            .iter()
            .position(|value| value.ends_with("Cargo.toml"))
            .expect("expected substring match to be present");
        assert!(
            prefix_index < substring_index,
            "expected prefix matches to outrank substring matches, got {values:?}"
        );

        fs::remove_dir_all(&dir)
            .unwrap_or_else(|error| panic!("failed to clean temp dir {}: {error}", dir.display()));
    }

    #[test]
    fn dot_path_completion_does_not_panic() {
        let args = vec![OsString::from("kat"), OsString::from(".")];
        let completions = complete(&mut completion_command_for(&args), args.clone(), 1, None)
            .expect("dot completion should work");
        let values = completions
            .iter()
            .map(|candidate| candidate.get_value().to_string_lossy().into_owned())
            .collect::<Vec<_>>();

        assert!(
            values.iter().all(|value| value != "." && value != "../"),
            "expected dot navigation entries to stay hidden, got {values:?}"
        );
    }

    #[test]
    fn dot_prefix_completion_prefers_hidden_entries() {
        let dir = unique_temp_dir("kat-dot-path-completion");
        fs::create_dir_all(&dir)
            .unwrap_or_else(|error| panic!("failed to create temp dir {}: {error}", dir.display()));
        let hidden_file = dir.join(".gitignore");
        fs::write(&hidden_file, "*\n").unwrap_or_else(|error| {
            panic!(
                "failed to write hidden completion fixture {}: {error}",
                hidden_file.display()
            )
        });
        let visible_file = dir.join("visible.txt");
        fs::write(&visible_file, "kat\n").unwrap_or_else(|error| {
            panic!(
                "failed to write visible completion fixture {}: {error}",
                visible_file.display()
            )
        });

        let current = dir.join(".");
        let args = vec![OsString::from("kat"), current.into_os_string()];
        let completions = complete(&mut completion_command_for(&args), args.clone(), 1, None)
            .expect("dot-prefix completion should work");
        let values = completions
            .iter()
            .map(|candidate| candidate.get_value().to_string_lossy().into_owned())
            .collect::<Vec<_>>();

        assert!(
            values.iter().any(|value| value.ends_with(".gitignore")),
            "expected hidden file completion, got {values:?}"
        );
        assert!(
            values.iter().all(|value| !value.ends_with("visible.txt")),
            "expected visible files to stay out of dot-prefix completion, got {values:?}"
        );
        assert!(
            values
                .iter()
                .all(|value| !value.ends_with("/.") && !value.ends_with("/./")),
            "expected current-directory navigation entries to stay hidden, got {values:?}"
        );

        fs::remove_dir_all(&dir)
            .unwrap_or_else(|error| panic!("failed to clean temp dir {}: {error}", dir.display()));
    }

    #[test]
    fn empty_input_completion_prefers_paths_over_options() {
        let args = vec![OsString::from("kat"), OsString::from("")];
        let completions = complete(&mut completion_command_for(&args), args.clone(), 1, None)
            .expect("failed to compute completions for empty input");
        let values = completions
            .iter()
            .map(|candidate| candidate.get_value().to_string_lossy().into_owned())
            .collect::<Vec<_>>();

        assert!(
            values.iter().all(|value| {
                value != "--help"
                    && value != "--version"
                    && value != "--language"
                    && value != "--paging"
                    && value != "--debug-ast"
                    && value != "--debug-semantics"
                    && value != "--debug-shell-semantics"
                    && value != "-"
            }),
            "expected empty-input completion to stay path-oriented, got {values:?}"
        );
    }

    #[test]
    fn dash_prefixed_input_completion_includes_options() {
        let args = vec![OsString::from("kat"), OsString::from("-")];
        let completions = complete(&mut completion_command_for(&args), args.clone(), 1, None)
            .expect("failed to compute option completions");
        let values = completions
            .iter()
            .map(|candidate| candidate.get_value().to_string_lossy().into_owned())
            .collect::<Vec<_>>();

        assert!(
            values.iter().any(|value| value == "-h") && values.iter().any(|value| value == "-V"),
            "expected dash-prefixed completion to include options, got {values:?}"
        );
        assert!(
            values.iter().all(|value| {
                value != "--debug-ast"
                    && value != "--debug-semantics"
                    && value != "--debug-shell-semantics"
            }),
            "expected debug flags to stay hidden from option completion, got {values:?}"
        );
    }

    #[test]
    fn debug_semantics_renders_structured_output() {
        let output = render_output(
            OutputMode::DebugSemantics,
            Some("fish"),
            None,
            "function watch --argument-names theme\n    status current-filename\nend\n",
        )
        .expect("failed to render semantic overlay output");

        assert!(
            output.contains("variable.parameter") && output.contains("keyword.directive"),
            "unexpected debug semantics output: {output}"
        );
    }

    #[test]
    fn debug_ast_renders_tree_output() {
        let output = render_output(
            OutputMode::DebugAst,
            Some("bash"),
            None,
            "printf '%s\\n' \"$HOME\"\n",
        )
        .expect("failed to render ast debug output");

        assert!(
            output.contains("(program") && output.contains("(command"),
            "unexpected debug ast output: {output}"
        );
    }

    #[test]
    fn auto_paging_requires_terminal_and_long_output() {
        assert!(
            should_page_output(&"line\n".repeat(40), PagingMode::Auto, true),
            "long tty output should use pager in auto mode"
        );
        assert!(
            !should_page_output(&"line\n".repeat(2), PagingMode::Auto, true),
            "short tty output should not use pager in auto mode"
        );
        assert!(
            !should_page_output(&"line\n".repeat(40), PagingMode::Auto, false),
            "non-tty output should not use pager in auto mode"
        );
    }

    #[test]
    fn resolves_default_and_env_pager_commands() {
        assert_eq!(
            resolve_pager_command(None).expect("default pager should resolve"),
            Some(PagerCommand {
                program: OsString::from("less"),
                args: vec![
                    OsString::from("-R"),
                    OsString::from("-F"),
                    OsString::from("-X"),
                ],
            })
        );
        assert_eq!(
            resolve_pager_command(Some(OsStr::new("less -R --quit-if-one-screen")))
                .expect("env pager should resolve"),
            Some(PagerCommand {
                program: OsString::from("less"),
                args: vec![OsString::from("-R"), OsString::from("--quit-if-one-screen")],
            })
        );
    }

    #[test]
    fn page_output_command_receives_rendered_text() {
        let dir = unique_temp_dir("kat-pager-output");
        fs::create_dir_all(&dir)
            .unwrap_or_else(|error| panic!("failed to create temp dir {}: {error}", dir.display()));
        let output_path = dir.join("pager-output.txt");
        let pager = PagerCommand {
            program: OsString::from("sh"),
            args: vec![
                OsString::from("-c"),
                OsString::from(format!(
                    "cat > {}",
                    shell_single_quote(output_path.as_path())
                )),
            ],
        };

        page_output_via_command("hello from pager\n", &pager)
            .expect("pager command should receive output");

        let saved = fs::read_to_string(&output_path)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", output_path.display()));
        assert_eq!(saved, "hello from pager\n");

        fs::remove_dir_all(&dir)
            .unwrap_or_else(|error| panic!("failed to clean temp dir {}: {error}", dir.display()));
    }

    fn shell_single_quote(path: &Path) -> String {
        let path = path.display().to_string();
        format!("'{}'", path.replace('\'', "'\"'\"'"))
    }
}
