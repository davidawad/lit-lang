//! Guard: no committed insta snapshot carries the compiler version, so a
//! release bump cannot leave snapshots stale. Redact it (`[version]`) and
//! any hash over output that embeds it.
// Test helpers below may unwrap: a panic is the failed assertion.
#![allow(clippy::unwrap_used)]
mod support;

use std::path::Path;

fn snapshots(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            snapshots(&path, out);
        } else if path.extension().is_some_and(|e| e == "snap") {
            out.push(path);
        }
    }
}

#[test]
fn no_snapshot_pins_the_compiler_version() {
    let mut files = Vec::new();
    snapshots(&support::repo().join("crates"), &mut files);
    assert!(!files.is_empty());
    let needle = format!("\"{}\"", env!("CARGO_PKG_VERSION"));
    let stale: Vec<_> = files
        .iter()
        .filter(|f| std::fs::read_to_string(f).unwrap().contains(&needle))
        .collect();
    assert!(stale.is_empty(), "snapshots pin {needle}: {stale:#?}");
}
