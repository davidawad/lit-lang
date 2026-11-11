//! Opt-in wall-clock phase timings for investigating `litigate check`.

use std::sync::OnceLock;
use std::time::Instant;

static ENABLED: OnceLock<bool> = OnceLock::new();

pub(crate) fn enabled() -> bool {
    *ENABLED.get_or_init(|| {
        std::env::var_os("LITIGATE_PHASE_TIMINGS").is_some_and(|value| value == "1")
    })
}

pub(crate) fn measure<T>(package: &str, phase: &str, f: impl FnOnce() -> T) -> T {
    if !enabled() {
        return f();
    }
    let started = Instant::now();
    let result = f();
    eprintln!(
        "LITIGATE_PHASE\t{package}\t{phase}\t{:.3}ms",
        started.elapsed().as_secs_f64() * 1000.0
    );
    result
}
