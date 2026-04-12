use std::{env, fmt::Display, sync::LazyLock, time::Instant};

static ENABLED: LazyLock<bool> = LazyLock::new(|| {
    env::var("KAT_DEBUG_PROGRESS")
        .ok()
        .is_some_and(|value| !matches!(value.as_str(), "" | "0" | "false" | "FALSE" | "False"))
});
static STARTED_AT: LazyLock<Instant> = LazyLock::new(Instant::now);

pub(crate) fn enabled() -> bool {
    *ENABLED
}

pub(crate) fn log(stage: &str, detail: impl Display) {
    if !enabled() {
        return;
    }

    eprintln!(
        "kat progress t={:.3}ms stage={} {}",
        STARTED_AT.elapsed().as_secs_f64() * 1_000.0,
        stage,
        detail
    );
}
