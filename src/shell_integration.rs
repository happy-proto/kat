use std::{env, fs, path::PathBuf};

use anyhow::{Context, Result, bail};

use crate::CompletionShell;

pub(crate) fn install(shell: CompletionShell) -> Result<String> {
    let home = env::var_os("HOME")
        .map(PathBuf::from)
        .context("could not determine home directory")?;
    let (path, source) = match shell {
        CompletionShell::Fish => (
            env::var_os("XDG_CONFIG_HOME")
                .map(PathBuf::from)
                .unwrap_or_else(|| home.join(".config"))
                .join("fish/functions/kat.fish"),
            include_str!("../shell/kat.fish"),
        ),
        CompletionShell::Bash | CompletionShell::Zsh => (
            env::var_os("XDG_DATA_HOME")
                .map(PathBuf::from)
                .unwrap_or_else(|| home.join(".local/share"))
                .join("kat/shell/kat.sh"),
            include_str!("../shell/kat.sh"),
        ),
    };
    if path.exists() && !fs::read_to_string(&path)?.starts_with("# kat shell integration\n") {
        bail!(
            "{} already exists and is not managed by kat",
            path.display()
        );
    }
    fs::create_dir_all(
        path.parent()
            .context("shell integration path has no parent")?,
    )?;
    fs::write(&path, source)
        .with_context(|| format!("failed to install shell integration to {}", path.display()))?;
    let mut output = format!(
        "installed {} shell integration to {}\n",
        shell.name(),
        path.display()
    );
    match shell {
        CompletionShell::Fish => {
            output.push_str("The kat function will be autoloaded on the next invocation.\n")
        }
        CompletionShell::Bash | CompletionShell::Zsh => output.push_str(&format!(
            "Source this file from your shell configuration:\n{}\n",
            path.display()
        )),
    }
    Ok(output)
}
