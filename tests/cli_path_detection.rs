use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::atomic::{AtomicUsize, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

struct Fixture(PathBuf);

impl Fixture {
    fn new() -> Self {
        static NEXT_ID: AtomicUsize = AtomicUsize::new(0);
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        let root =
            std::env::temp_dir().join(format!("kat-cli-path-{}-{nanos}-{id}", std::process::id()));
        fs::create_dir_all(&root).unwrap();
        Self(root)
    }

    fn write(&self, path: &str, source: &str) -> PathBuf {
        let path = self.0.join(path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, source).unwrap();
        path
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

fn command(cwd: &Path) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_kat"));
    command
        .current_dir(cwd)
        .env_remove("NO_COLOR")
        .env("TERM", "xterm-256color");
    command
}

#[test]
fn directory_browsing_requires_a_terminal_and_a_single_directory() {
    let fixture = Fixture::new();
    fixture.write("file.txt", "file contents\n");
    for args in [vec!["."], vec![".", "file.txt"]] {
        let output = command(&fixture.0).args(&args).output().unwrap();
        assert!(!output.status.success());
        assert!(output.stdout.is_empty());
        let error = String::from_utf8_lossy(&output.stderr);
        assert!(
            error.contains(if args.len() == 1 {
                "directory browsing requires a terminal"
            } else {
                "directory browsing takes a single directory"
            }),
            "{error}"
        );
    }
}

#[test]
fn no_path_with_piped_stdin_still_reads_the_stream() {
    let fixture = Fixture::new();
    for args in [vec![], vec!["-"]] {
        let mut child = command(&fixture.0)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();
        child
            .stdin
            .take()
            .unwrap()
            .write_all(b"stream contents\n")
            .unwrap();
        let output = child.wait_with_output().unwrap();
        assert!(output.status.success());
        assert_eq!(output.stdout, b"stream contents\n");
    }
}

fn analysis(cwd: &Path, args: &[&str]) -> serde_json::Value {
    let output = command(cwd)
        .arg("--debug-analysis")
        .args(args)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).unwrap()
}

#[test]
fn relative_paths_use_the_working_directory_for_detection() {
    let fixture = Fixture::new();
    for (path, source, runtime) in [
        (
            ".ssh/config",
            "Host example\n    HostName example.com\n",
            "ssh_config",
        ),
        (".git/config", "[core]\n    bare = false\n", "git_config"),
        ("ghostty/config", "font-size = 14\n", "ghostty"),
    ] {
        let path = fixture.write(path, source);
        let absolute = analysis(&fixture.0, &[path.to_str().unwrap()]);
        assert_eq!(absolute["detected_document_kind"]["runtime_name"], runtime);
        for relative in ["config", "./config"] {
            assert_eq!(analysis(path.parent().unwrap(), &[relative]), absolute);
        }
    }
}

#[test]
fn generic_config_and_stdin_do_not_inherit_a_language() {
    let fixture = Fixture::new();
    fixture.write("config", "Host example\n");
    assert!(analysis(&fixture.0, &["config"])["detected_document_kind"].is_null());

    let ssh_config = fixture.write(".ssh/config", "Host example\n");
    let mut child = command(ssh_config.parent().unwrap())
        .args(["--debug-analysis", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(b"Host example\n")
        .unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    let snapshot: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(snapshot["detected_document_kind"].is_null());
}

#[test]
fn explicit_language_overrides_directory_detection() {
    let fixture = Fixture::new();
    let path = fixture.write(".ssh/config", "Host example\n");
    let snapshot = analysis(path.parent().unwrap(), &["--language", "bash", "config"]);
    assert_eq!(snapshot["resolved_document_kind"]["runtime_name"], "bash");
}

#[test]
fn rendering_uses_directory_context_and_headers_keep_input_paths() {
    let fixture = Fixture::new();
    let path = fixture.write(".ssh/config", "Host example\n    HostName example.com\n");
    let cwd = path.parent().unwrap();
    let absolute = command(cwd).arg(&path).output().unwrap();
    let relative = command(cwd).arg("config").output().unwrap();
    assert!(absolute.status.success() && relative.status.success());
    assert!(
        absolute.stdout.contains(&0x1b),
        "expected highlighted output"
    );
    assert_eq!(relative.stdout, absolute.stdout);

    let multiple = command(cwd).args(["config", "./config"]).output().unwrap();
    assert!(multiple.status.success());
    let output = String::from_utf8(multiple.stdout).unwrap();
    assert!(output.starts_with("==> config <==\n"));
    assert!(output.contains("\n==> ./config <==\n"));
    assert!(!output.contains(fixture.0.to_str().unwrap()));
}

#[cfg(unix)]
#[test]
fn symlink_uses_the_requested_path_for_detection() {
    let fixture = Fixture::new();
    let target = fixture.write("dotfiles/ssh-settings", "Host example\n");
    let ssh_dir = fixture.0.join(".ssh");
    fs::create_dir(&ssh_dir).unwrap();
    std::os::unix::fs::symlink(target, ssh_dir.join("config")).unwrap();
    assert_eq!(
        analysis(&ssh_dir, &["config"])["detected_document_kind"]["runtime_name"],
        "ssh_config"
    );
}

#[test]
fn ssh_conf_d_fragments_are_detected_only_in_the_ssh_directory() {
    let fixture = Fixture::new();
    let path = fixture.write(
        ".ssh/conf.d/example.conf",
        "Host example\n    HostName example.com\n",
    );
    for snapshot in [
        analysis(&fixture.0, &[path.to_str().unwrap()]),
        analysis(path.parent().unwrap(), &["example.conf"]),
    ] {
        assert_eq!(
            snapshot["detected_document_kind"]["runtime_name"],
            "ssh_config"
        );
    }
    for name in [
        "conf.d/example.conf",
        ".ssh/other/example.conf",
        ".ssh/conf.d/example.conf.bak",
        ".ssh/conf.d/README",
    ] {
        let path = fixture.write(name, "Host example\n");
        assert_ne!(
            analysis(&fixture.0, &[path.to_str().unwrap()])["detected_document_kind"]["runtime_name"],
            "ssh_config",
            "{name}"
        );
    }
}
