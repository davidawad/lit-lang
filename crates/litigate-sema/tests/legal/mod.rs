//! Helpers for the temporal and precedence tables: packages with a chosen
//! authority and lifetime, and per-case diagnostic rendering.
#![allow(dead_code)]
use litigate_sema::pkg::load::{LoadedFile, LoadedPackage};
use litigate_sema::temporal::World;
use litigate_sema::{LitDatabase, Workspace};
use std::collections::BTreeMap;
use std::fmt::Write as _;

/// A package `name` at `pkgs/<name>` whose declarations default to
/// `authority` and whose lifetime starts `in_force`.
pub fn pkg(
    name: &str,
    authority: &str,
    in_force: &str,
    deps: &[&str],
    files: &[(&str, &str)],
) -> LoadedPackage {
    let mut manifest = format!(
        "[package]\nname = \"{name}\"\nversion = \"1.0.0\"\nlanguage = 1\npack_id = \"{name}\"\nforum = \"demo\"\nauthority = \"{authority}\"\ntitle = \"T\"\nin_force = \"{in_force}\"\nknown_at = \"2025-01-01\"\n\n[package.roles]\nPlaintiff = \"self\"\nDefendant = \"opponent\"\nCourt = \"nature\"\n"
    );
    if !deps.is_empty() {
        manifest.push_str("\n[dependencies]\n");
        for d in deps {
            let _ = writeln!(
                manifest,
                "{d} = {{ version = \"^1.0.0\", path = \"../{d}\" }}"
            );
        }
    }
    LoadedPackage {
        root: format!("pkgs/{name}"),
        manifest,
        lock: None,
        files: files
            .iter()
            .map(|(p, t)| LoadedFile {
                relative: (*p).to_owned(),
                text: (*t).to_owned(),
            })
            .collect(),
    }
}

/// Shared declarations for single-package cases (lines 1-12; start `s`).
pub const PRELUDE: &str = "module main
start s
enum Document { complaint, motion }
enum Method { Personal, Mail }
event Served(document: Document, method: Method)
event Answered(document: Document)
event Ordered()
event Hearing(document: Document)
counting c6 { exclude_trigger_day, include_last_day, roll_forward_on [weekend], holidays cal }
calendar cal { new-year: fixed(1, 1) effective [1870-06-28, ..) }
state s: \"S\" { _ \"go\" -> t }
terminal t: \"T\"
";

/// The authority chain `lit/core` declares.
pub const CORE: &str = "module core.authority\nauthority Statute > FederalRule > LocalRule > StandingOrder > CaseOrder\n";

/// Renders every diagnostic of `ws`, notes and fixes included.
pub fn render(db: &LitDatabase, ws: Workspace) -> Vec<String> {
    litigate_sema::check_workspace(db, ws)
        .into_iter()
        .flat_map(|c| c.diagnostics.iter())
        .map(|d| {
            let mut s = format!(
                "{}:{}:{} {} {}",
                d.span.file.0,
                d.span.line,
                d.span.col,
                d.code.as_str(),
                d.message
            );
            for n in &d.notes {
                let at = n
                    .span
                    .as_ref()
                    .map(|s| format!(" @{}:{}", s.line, s.col))
                    .unwrap_or_default();
                let _ = write!(s, "\n    note{at}: {}", n.message);
            }
            for f in &d.fixes {
                let _ = write!(
                    s,
                    "\n    fix @{}:{}-{}:{}: {:?}",
                    f.span.line, f.span.col, f.span.end_line, f.span.end_col, f.replacement
                );
            }
            s
        })
        .collect()
}

/// A workspace of `packages`.
pub fn workspace(packages: &[LoadedPackage]) -> (LitDatabase, Workspace) {
    let mut db = LitDatabase::default();
    let ws = litigate_sema::db::inputs::create_workspace(&mut db, packages);
    (db, ws)
}

/// Diagnostics of a single-package program `PRELUDE + body` (package
/// lifetime from 2009-12-01).
pub fn single(body: &str) -> Vec<String> {
    let text = format!("{PRELUDE}{body}\n");
    let (db, ws) = workspace(&[pkg(
        "demo",
        "FederalRule",
        "2009-12-01",
        &[],
        &[("main.lit", &text)],
    )]);
    render(&db, ws)
}

/// Runs every `(name, body)` case through [`single`].
pub fn table(cases: &[(&str, &str)]) -> BTreeMap<String, Vec<String>> {
    cases
        .iter()
        .map(|(n, b)| ((*n).to_owned(), single(b)))
        .collect()
}

/// Calls `f` with the world of the compiled packages of `ws` (every
/// package must be error-free).
pub fn with_world<R>(db: &LitDatabase, ws: Workspace, f: impl FnOnce(&World<'_>) -> R) -> R {
    let checks = litigate_sema::check_workspace(db, ws);
    let packages: Vec<&litigate_sema::hir::Package> =
        checks.iter().filter_map(|c| c.package.as_ref()).collect();
    assert_eq!(packages.len(), checks.len(), "{:?}", render(db, ws));
    f(&World::from_packages(&packages))
}
