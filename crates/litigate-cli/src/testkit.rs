//! Unit-test fixtures: in-memory packages run through the real loader,
//! checker, and session, starting from the SPEC §12 worked example.
use crate::ws::{CaseText, Session};
use litigate_sema::hir;
use litigate_sema::pkg::load::{LoadedFile, LoadedPackage};

/// The worked example's `Lit.toml`.
pub const MANIFEST: &str = include_str!("../../../lit/examples/Lit.toml");
/// The worked example's module.
pub const RULE12: &str = include_str!("../../../lit/examples/rule12.lit");

/// One in-memory package at `root`.
pub fn package(root: &str, manifest: &str, files: &[(&str, &str)]) -> LoadedPackage {
    LoadedPackage {
        root: root.to_owned(),
        manifest: manifest.to_owned(),
        lock: None,
        files: files
            .iter()
            .map(|(relative, text)| LoadedFile {
                relative: (*relative).to_owned(),
                text: (*text).to_owned(),
            })
            .collect(),
    }
}

/// The worked example with `extra` appended to its module.
pub fn example_with(extra: &str) -> Session {
    example_and(extra, Vec::new())
}

/// The worked example with `extra` appended, and `cases`.
pub fn example_and(extra: &str, cases: Vec<CaseText>) -> Session {
    let module = format!("{RULE12}\n{extra}\n");
    Session::from_loaded(
        &[package(
            "lit/examples",
            MANIFEST,
            &[("rule12.lit", &module)],
        )],
        cases,
    )
}

/// The worked example's HIR with `extra` appended (panics on errors).
pub fn hir_with(extra: &str) -> hir::Package {
    let session = example_with(extra);
    let check = &session.packages[0].check;
    assert!(check.package.is_some(), "{:#?}", check.diagnostics);
    check.package.clone().unwrap()
}

/// The worked example's matter.
pub const ACME: &str = include_str!("../../../cases/examples/acme.case");

/// The worked example with `extra` appended, and one matter with `text`
/// and `lock`, else the lock it needs.
pub fn example_locked(extra: &str, text: &str, lock: Option<&str>) -> Session {
    let unlocked = example_and(extra, vec![case(text, None)]);
    let needed = unlocked.matter_lock(&unlocked.cases).render();
    example_and(extra, vec![case(text, Some(lock.unwrap_or(&needed)))])
}

/// A matter at the worked example's path with `text` and `lock`.
pub fn case(text: &str, lock: Option<&str>) -> CaseText {
    CaseText {
        path: "cases/examples/acme.case".to_owned(),
        text: text.to_owned(),
        lock: lock.map(str::to_owned),
    }
}
