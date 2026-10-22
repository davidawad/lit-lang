//! Performance budget (bead acceptance): parsing a stdlib-sized corpus
//! takes under 50 ms in release builds. The corpus is the worked example's
//! items repeated past 1 MiB, larger than all nine packs' JSON (~0.7 MB).
//! Debug builds only report the timing (and log a skip to
//! `TEST_SKIP_LOG` when set), since the budget is a release property.
use litigate_syntax::parse_lit;
use std::time::{Duration, Instant};

const BUDGET: Duration = Duration::from_millis(50);

fn corpus() -> String {
    let path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../lit/examples/rule12.lit");
    let example = std::fs::read_to_string(path).unwrap_or_default();
    let items: String = example
        .lines()
        .filter(|line| !line.starts_with("module") && !line.starts_with("package"))
        .map(|line| format!("{line}\n"))
        .collect();
    let mut text = String::from("module stdlib.synthetic\n");
    while text.len() < (1 << 20) {
        text.push_str(&items);
    }
    text
}

#[test]
fn parses_a_stdlib_sized_corpus_within_budget() {
    let text = corpus();
    let best = (0..5)
        .map(|_| {
            let start = Instant::now();
            let parsed = parse_lit(&text);
            let elapsed = start.elapsed();
            assert!(parsed.is_ok(), "{:?}", parsed.errors().first());
            elapsed
        })
        .min()
        .unwrap_or_default();
    eprintln!(
        "parsed {} bytes in {best:?} (budget {BUDGET:?})",
        text.len()
    );
    if cfg!(debug_assertions) {
        if let Ok(log) = std::env::var("TEST_SKIP_LOG") {
            let line = "SKIP perf::parses_a_stdlib_sized_corpus_within_budget: timing budget is asserted only in release builds\n";
            let _ = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(log)
                .and_then(|mut file| std::io::Write::write_all(&mut file, line.as_bytes()));
        }
        return;
    }
    assert!(best < BUDGET, "parse took {best:?}, budget {BUDGET:?}");
}
