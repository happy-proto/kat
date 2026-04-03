use std::{
    env,
    ffi::OsStr,
    ffi::OsString,
    fs,
    io::{self, Read},
    path::PathBuf,
};

use anyhow::{Context, Result, bail};

fn main() -> Result<()> {
    let options = parse_cli_args(env::args_os().skip(1))?;

    if options.paths.is_empty() {
        let stdin = read_stdin().context("failed to read stdin")?;
        print!(
            "{}",
            render_output(options.mode, options.language.as_deref(), None, &stdin)?
        );
        return Ok(());
    }

    let multiple_paths = options.paths.len() > 1;
    for (index, path) in options.paths.iter().enumerate() {
        if path.as_os_str() == OsStr::new("-") {
            let stdin = read_stdin().context("failed to read stdin")?;
            if multiple_paths {
                print_header("-", index > 0);
            }
            print!(
                "{}",
                render_output(options.mode, options.language.as_deref(), None, &stdin)?
            );
            continue;
        }

        let source = read_source_from_path(&path)?;
        if multiple_paths {
            print_header(&path.display().to_string(), index > 0);
        }
        print!(
            "{}",
            render_output(
                options.mode,
                options.language.as_deref(),
                Some(path.as_path()),
                &source
            )?
        );
    }

    Ok(())
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
    language: Option<String>,
    paths: Vec<PathBuf>,
}

fn parse_cli_args(args: impl IntoIterator<Item = OsString>) -> Result<CliOptions> {
    let mut mode = OutputMode::Render;
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
                "--help" | "-h" => {
                    print_help();
                    std::process::exit(0);
                }
                _ => {
                    if let Some(value) = value.strip_prefix("--language=") {
                        language = Some(value.to_owned());
                        continue;
                    }
                }
            }
        }

        paths.push(PathBuf::from(arg));
    }

    Ok(CliOptions {
        mode,
        language,
        paths,
    })
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

fn print_header(label: &str, add_leading_newline: bool) {
    if add_leading_newline {
        println!();
    }
    println!("==> {label} <==");
}

fn print_help() {
    println!(
        "Usage: kat [--debug-ast|--debug-semantics|--debug-shell-semantics] [--language <name>] [PATH|-]..."
    );
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
        ffi::OsString,
        fs,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    use super::{CliOptions, OutputMode, parse_cli_args, read_source_from_path, render_output};

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
                language: Some("regex".to_owned()),
                paths: vec![PathBuf::from("pattern.re")],
            }
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
}
