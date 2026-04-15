use std::{
    env,
    fs::{self, OpenOptions},
    io::Write,
    path::PathBuf,
    sync::{LazyLock, Mutex},
    time::Instant,
};

use serde::Serialize;

static STARTED_AT: LazyLock<Instant> = LazyLock::new(Instant::now);
static WRITE_LOCK: Mutex<()> = Mutex::new(());

#[derive(Serialize)]
struct LogEvent<'a, T> {
    t_ms: u64,
    pid: u32,
    event: &'a str,
    payload: &'a T,
}

pub(crate) fn log<T: Serialize>(event: &str, payload: &T) {
    let Some(path) = log_path() else {
        return;
    };

    let entry = LogEvent {
        t_ms: STARTED_AT.elapsed().as_millis() as u64,
        pid: std::process::id(),
        event,
        payload,
    };

    let Ok(serialized) = serde_json::to_string(&entry) else {
        return;
    };

    let _guard = WRITE_LOCK.lock().ok();
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        && fs::create_dir_all(parent).is_err()
    {
        return;
    }

    let Ok(mut file) = OpenOptions::new().create(true).append(true).open(&path) else {
        return;
    };

    let _ = writeln!(file, "{serialized}");
}

fn log_path() -> Option<PathBuf> {
    let path = env::var_os("KAT_DEBUG_LOG_PATH")?;
    if path.is_empty() {
        return None;
    }
    Some(PathBuf::from(path))
}

#[cfg(test)]
mod tests {
    use super::log;
    use serde::Serialize;
    use std::{
        fs,
        path::PathBuf,
        sync::atomic::{AtomicU64, Ordering},
        time::{SystemTime, UNIX_EPOCH},
    };

    #[derive(Serialize)]
    struct SamplePayload<'a> {
        value: &'a str,
    }

    static UNIQUE_COUNTER: AtomicU64 = AtomicU64::new(0);

    fn unique_log_path() -> PathBuf {
        let mut path = std::env::temp_dir();
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time before unix epoch")
            .as_nanos();
        let counter = UNIQUE_COUNTER.fetch_add(1, Ordering::Relaxed);
        path.push("kat-tests");
        path.push(format!(
            "runtime-debug-log-{}-{}-{}.jsonl",
            std::process::id(),
            nanos,
            counter
        ));
        path
    }

    #[test]
    fn writes_jsonl_events_when_debug_log_path_is_set() {
        let path = unique_log_path();
        // SAFETY: test-scoped env mutation.
        unsafe {
            std::env::set_var("KAT_DEBUG_LOG_PATH", &path);
        }
        log(
            "sample_event",
            &SamplePayload {
                value: "expected-value",
            },
        );
        // SAFETY: test-scoped env mutation.
        unsafe {
            std::env::remove_var("KAT_DEBUG_LOG_PATH");
        }

        let contents = fs::read_to_string(&path).expect("debug log file should be created");
        assert!(contents.contains("\"event\":\"sample_event\""));
        assert!(contents.contains("\"value\":\"expected-value\""));

        let _ = fs::remove_file(path);
    }
}
