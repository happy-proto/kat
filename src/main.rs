use std::{
    env,
    ffi::OsStr,
    ffi::OsString,
    fs,
    io::{self, IsTerminal, Read, Write},
    path::PathBuf,
    process::{Command, ExitCode, Stdio},
};

use anyhow::{Context, Result, bail};
use clap::{ArgAction, CommandFactory, FromArgMatches, Parser, ValueEnum};
use clap_complete::engine::{ArgValueCompleter, CompletionCandidate};
use clap_complete::env::{Bash, Elvish, EnvCompleter, Fish, Powershell, Zsh};
use miette::{Report, miette};
use shadow_rs::shadow;
use terminal_size::{Height, terminal_size};

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
    let output = build_output(&options)?;
    write_output(&output, &options)
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
    DebugSemantics,
    Version,
}

#[derive(Debug, Eq, PartialEq)]
struct CliOptions {
    mode: OutputMode,
    paging: PagingMode,
    language: Option<String>,
    paths: Vec<PathBuf>,
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
    #[arg(long = "debug-semantics", group = "mode")]
    debug_semantics: bool,
    #[arg(long = "debug-shell-semantics", group = "mode")]
    debug_shell_semantics: bool,
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
    for arg_id in ["debug_ast", "debug_semantics", "debug_shell_semantics"] {
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
    let mut candidates = Vec::new();
    let Some((display_prefix, search_root, fragment)) = resolve_completion_root(current) else {
        return candidates;
    };

    let fragment = fragment.to_string_lossy();
    for entry in fs::read_dir(search_root)
        .ok()
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
    {
        let file_name = entry.file_name();
        if !file_name.to_string_lossy().starts_with(&*fragment) {
            continue;
        }

        let mut suggestion = display_prefix.join(&file_name);
        if entry.path().is_dir() {
            suggestion.push("");
        } else if !entry.path().is_file() {
            continue;
        }

        candidates.push(
            CompletionCandidate::new(suggestion.into_os_string()).hide(is_hidden_path(&file_name)),
        );
    }

    candidates.sort();
    candidates
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
    } else if cli.debug_semantics || cli.debug_shell_semantics {
        OutputMode::DebugSemantics
    } else {
        OutputMode::Render
    };

    Ok(CliOptions {
        mode,
        paging: cli.paging,
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
        OutputMode::DebugSemantics => {
            let language_name = resolve_debug_language_name(language, source_path, source)?;
            let mut output = kat::debug_semantics(&language_name, source)?;
            if !output.ends_with('\n') {
                output.push('\n');
            }
            Ok(output)
        }
        OutputMode::Version => Ok(version_output()),
    }
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

fn build_output(options: &CliOptions) -> Result<String> {
    if matches!(options.mode, OutputMode::Version) {
        return Ok(version_output());
    }

    if options.paths.is_empty() {
        let stdin = read_stdin().context("failed to read stdin")?;
        return render_output(options.mode, options.language.as_deref(), None, &stdin);
    }

    let mut output = String::new();
    let multiple_paths = options.paths.len() > 1;

    for (index, path) in options.paths.iter().enumerate() {
        if path.as_os_str() == OsStr::new("-") {
            let stdin = read_stdin().context("failed to read stdin")?;
            if multiple_paths {
                push_header(&mut output, "-", index > 0);
            }
            output.push_str(&render_output(
                options.mode,
                options.language.as_deref(),
                None,
                &stdin,
            )?);
            continue;
        }

        let source = read_source_from_path(path)?;
        if multiple_paths {
            push_header(&mut output, &path.display().to_string(), index > 0);
        }
        output.push_str(&render_output(
            options.mode,
            options.language.as_deref(),
            Some(path.as_path()),
            &source,
        )?);
    }

    Ok(output)
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
        time::{SystemTime, UNIX_EPOCH},
    };

    use clap_complete::engine::complete;
    use std::path::Path;

    use super::{
        CliOptions, OutputMode, PagerCommand, PagingMode, cli_command, completion_command_for,
        format_cli_error, page_output_via_command, parse_cli_args, read_source_from_path,
        render_output, resolve_pager_command, should_page_output, version_output,
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
                language: Some("regex".to_owned()),
                paths: vec![PathBuf::from("pattern.re")],
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
                language: None,
                paths: vec![],
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
