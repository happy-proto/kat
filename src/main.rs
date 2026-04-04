use std::{
    env,
    ffi::OsStr,
    ffi::OsString,
    fs,
    io::{self, IsTerminal, Read, Write},
    path::PathBuf,
    process::{Command, Stdio},
};

use anyhow::{Context, Result, bail};
use terminal_size::{Height, terminal_size};

const DEFAULT_TERMINAL_ROWS: usize = 24;

fn main() -> Result<()> {
    let options = parse_cli_args(env::args_os().skip(1))?;
    let output = build_output(&options)?;
    write_output(&output, &options)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum OutputMode {
    Render,
    DebugAst,
    DebugSemantics,
}

#[derive(Debug, Eq, PartialEq)]
struct CliOptions {
    mode: OutputMode,
    paging: PagingMode,
    language: Option<String>,
    paths: Vec<PathBuf>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum PagingMode {
    Auto,
    Always,
    Never,
}

fn parse_cli_args(args: impl IntoIterator<Item = OsString>) -> Result<CliOptions> {
    let mut mode = OutputMode::Render;
    let mut paging = PagingMode::Auto;
    let mut language = None;
    let mut paths = Vec::new();
    let mut args = args.into_iter();

    while let Some(arg) = args.next() {
        if let Some(value) = arg.to_str() {
            match value {
                "--debug-ast" => {
                    ensure_mode(&mut mode, OutputMode::DebugAst, "--debug-ast")?;
                    continue;
                }
                "--debug-shell-semantics" => {
                    ensure_mode(
                        &mut mode,
                        OutputMode::DebugSemantics,
                        "--debug-shell-semantics",
                    )?;
                    continue;
                }
                "--debug-semantics" => {
                    ensure_mode(&mut mode, OutputMode::DebugSemantics, "--debug-semantics")?;
                    continue;
                }
                "--language" => {
                    let Some(next) = args.next() else {
                        bail!("--language requires a language name");
                    };
                    language = Some(next.to_string_lossy().into_owned());
                    continue;
                }
                "--paging" => {
                    let Some(next) = args.next() else {
                        bail!("--paging requires one of auto, always, or never");
                    };
                    paging = parse_paging_mode(&next)?;
                    continue;
                }
                "--help" | "-h" => {
                    print_help();
                    std::process::exit(0);
                }
                _ => {
                    if let Some(value) = value.strip_prefix("--language=") {
                        language = Some(value.to_owned());
                        continue;
                    }
                    if let Some(value) = value.strip_prefix("--paging=") {
                        paging = parse_paging_mode(value)?;
                        continue;
                    }
                }
            }
        }

        paths.push(PathBuf::from(arg));
    }

    Ok(CliOptions {
        mode,
        paging,
        language,
        paths,
    })
}

fn parse_paging_mode(value: impl AsRef<OsStr>) -> Result<PagingMode> {
    match value.as_ref().to_string_lossy().as_ref() {
        "auto" => Ok(PagingMode::Auto),
        "always" => Ok(PagingMode::Always),
        "never" => Ok(PagingMode::Never),
        other => bail!("unsupported paging mode: {other}; expected auto, always, or never"),
    }
}

fn ensure_mode(current: &mut OutputMode, next: OutputMode, flag: &str) -> Result<()> {
    if *current != OutputMode::Render && *current != next {
        bail!("multiple debug modes provided; keep only one of --debug-ast or --debug-semantics");
    }
    *current = next;
    if matches!(next, OutputMode::Render) {
        bail!("unexpected render mode request from {flag}");
    }
    Ok(())
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

fn print_help() {
    println!(
        "Usage: kat [--debug-ast|--debug-semantics|--debug-shell-semantics] [--paging <auto|always|never>] [--language <name>] [PATH|-]..."
    );
    println!();
    println!("Paging:");
    println!("  auto   page long highlighted output when stdout is a TTY (default)");
    println!("  always always try pager when stdout is a TTY");
    println!("  never  write directly to stdout");
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
        bail!(
            "failed to read {}: path is a directory; pass a file path instead",
            path.display()
        );
    }

    let bytes = fs::read(path).with_context(|| format!("failed to read {}", path.display()))?;
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

    use std::path::Path;

    use super::{
        CliOptions, OutputMode, PagerCommand, PagingMode, page_output_via_command, parse_cli_args,
        read_source_from_path, render_output, resolve_pager_command, should_page_output,
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
    fn rejects_unknown_paging_mode() {
        let error = parse_cli_args([OsString::from("--paging=maybe")])
            .expect_err("unknown paging mode should fail");

        assert!(
            format!("{error:#}").contains("unsupported paging mode"),
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
            format!("{error:#}").contains("multiple debug modes"),
            "unexpected error: {error:#}"
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
