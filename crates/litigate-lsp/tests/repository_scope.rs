//! Full repository protocol contract: the root analysis covers every
//! `Lit.toml`, `.lit`, and `.case` source under `lit/` and `cases/`.
#![allow(clippy::unwrap_used, clippy::expect_used)]

mod support;

use std::path::{Path, PathBuf};

/// Every `Lit.toml`, `.lit` and `.case` file under `roots`, repo-relative
/// and sorted: what `litigate check` analyses (`lit/core` included).
fn sources(roots: &[&str]) -> Vec<String> {
    fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
        for entry in std::fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                walk(&path, out);
            } else {
                out.push(path);
            }
        }
    }
    let root = support::repo();
    let mut files = Vec::new();
    for path in roots {
        walk(&root.join(path), &mut files);
    }
    let mut out: Vec<String> = files
        .iter()
        .filter(|path| {
            path.file_name().is_some_and(|name| name == "Lit.toml")
                || path
                    .extension()
                    .is_some_and(|ext| ext == "lit" || ext == "case")
        })
        .map(|path| {
            path.strip_prefix(&root)
                .unwrap()
                .to_string_lossy()
                .into_owned()
        })
        .collect();
    out.sort();
    out
}

#[tokio::test]
async fn repository_analysis_covers_lit_and_cases_sources() {
    let mut client = support::Client::repo().await;
    let analyzed: Vec<String> = client.status_draining_notifications().await["files"]
        .as_array()
        .unwrap()
        .iter()
        .map(|path| path.as_str().unwrap().to_owned())
        .collect();
    assert_eq!(
        analyzed,
        sources(&["lit", "cases"]),
        "root analysis must cover lit/ and cases/"
    );
}
