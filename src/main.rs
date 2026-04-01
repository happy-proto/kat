use std::{
    env,
    ffi::OsStr,
    fs,
    io::{self, Read},
    path::PathBuf,
};

use anyhow::{Context, Result, bail};

fn main() -> Result<()> {
    let paths: Vec<PathBuf> = env::args_os().skip(1).map(PathBuf::from).collect();

    if paths.is_empty() {
        let stdin = read_stdin().context("failed to read stdin")?;
        print!("{}", kat::render(None, &stdin)?);
        return Ok(());
    }

    for path in paths {
        if path.as_os_str() == OsStr::new("-") {
            let stdin = read_stdin().context("failed to read stdin")?;
            print!("{}", kat::render(None, &stdin)?);
            continue;
        }

        let source = read_source_from_path(&path)?;
        print!("{}", kat::render(Some(&path), &source)?);
    }

    Ok(())
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
        fs,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    use super::read_source_from_path;

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
}
