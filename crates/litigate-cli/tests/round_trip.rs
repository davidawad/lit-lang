//! Compile -> solve round trip (lg-9lrj): `litigate solve` on every
//! package under `lit/` and every matter under `cases/` compiles it and
//! the engine solves the result, so the compiler and the engine cannot
//! disagree on a shape they exchange.
// Test helpers below may unwrap: a panic is the failed assertion.
#![allow(clippy::unwrap_used)]
mod support;

use std::path::Path;
use support::{litigate, repo};

/// Every package directory under `lit/` and every `.case` under `cases/`,
/// repository-relative.
fn targets() -> Vec<String> {
    fn cases(dir: &Path, out: &mut Vec<String>) {
        for entry in std::fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                cases(&path, out);
            } else if path.extension().is_some_and(|e| e == "case") {
                out.push(rel(&path));
            }
        }
    }
    fn rel(path: &Path) -> String {
        path.strip_prefix(repo())
            .unwrap()
            .to_string_lossy()
            .into_owned()
    }
    let mut out: Vec<String> = std::fs::read_dir(repo().join("lit"))
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.join("Lit.toml").is_file())
        .map(|p| rel(&p))
        .collect();
    cases(&repo().join("cases"), &mut out);
    out.sort();
    out
}

#[test]
fn every_package_and_matter_compiles_and_solves() {
    let targets = targets();
    assert!(targets.iter().any(|t| t == "lit/links"), "{targets:?}");
    assert!(
        targets.iter().any(|t| t == "cases/examples/acme.case"),
        "{targets:?}"
    );
    let runs: Vec<(String, support::Run)> = std::thread::scope(|s| {
        let handles: Vec<_> = targets
            .iter()
            .map(|t| s.spawn(move || (t.clone(), litigate(&["solve", t]))))
            .collect();
        handles.into_iter().map(|h| h.join().unwrap()).collect()
    });
    for (target, run) in runs {
        assert_eq!(run.code, 0, "{target}: {}{}", run.stdout, run.stderr);
        let solved = &run.json["result"];
        assert_eq!(solved["ok"], true, "{target}: {solved}");
        assert_eq!(solved["result"]["converged"], true, "{target}: {solved}");
        assert!(solved["result"]["value"].is_number(), "{target}: {solved}");
    }
}
