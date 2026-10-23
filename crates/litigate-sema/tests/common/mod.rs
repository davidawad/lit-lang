//! In-memory workspaces for integration tests.
#![allow(dead_code)]
use litigate_sema::db::inputs::create_workspace;
use litigate_sema::pkg::load::{LoadedFile, LoadedPackage};
use litigate_sema::{LitDatabase, Workspace};
use std::fmt::Write as _;

/// A manifest for package `name` with `deps` (`name = "^1.0.0"` path deps).
pub fn manifest(name: &str, deps: &[&str]) -> String {
    let mut text = format!(
        "[package]\nname = \"{name}\"\nversion = \"1.0.0\"\nlanguage = 1\npack_id = \"{name}\"\nforum = \"demo\"\nauthority = \"FederalRule\"\ntitle = \"T\"\nin_force = \"2009-12-01\"\nknown_at = \"2025-01-01\"\n\n[package.roles]\nPlaintiff = \"self\"\nDefendant = \"opponent\"\nCourt = \"nature\"\n"
    );
    if !deps.is_empty() {
        text.push_str("\n[dependencies]\n");
        for d in deps {
            let _ = writeln!(text, "{d} = {{ version = \"^1.0.0\", path = \"../{d}\" }}");
        }
    }
    text
}

/// One package `name` at `pkgs/<name>` with files `(relative, text)`.
pub fn package(name: &str, deps: &[&str], files: &[(&str, &str)]) -> LoadedPackage {
    LoadedPackage {
        root: format!("pkgs/{name}"),
        manifest: manifest(name, deps),
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

/// A database holding `packages`.
pub fn workspace(packages: &[LoadedPackage]) -> (LitDatabase, Workspace) {
    let mut db = LitDatabase::default();
    let ws = create_workspace(&mut db, packages);
    (db, ws)
}

/// Every diagnostic of every package as `file:line:col CODE message`,
/// with fixes appended as `[fix: replacement]`.
pub fn render(db: &LitDatabase, ws: Workspace) -> Vec<String> {
    litigate_sema::check_workspace(db, ws)
        .into_iter()
        .flat_map(|c| c.diagnostics.iter())
        .map(|d| {
            let fixes = d
                .fixes
                .iter()
                .flat_map(|f| &f.edits)
                .fold(String::new(), |mut s, e| {
                    let _ = write!(s, " [fix: {:?}]", e.replacement);
                    s
                });
            format!(
                "{}:{}:{} {} {}{fixes}",
                d.span.file.0,
                d.span.line,
                d.span.col,
                d.code.as_str(),
                d.message
            )
        })
        .collect()
}

/// Diagnostics of a single-module package whose body is `body` (a module
/// header, the standard enums/events, and a start node are prepended).
pub fn check_body(body: &str) -> Vec<String> {
    let text = format!("{PRELUDE}{body}\n");
    let (db, ws) = workspace(&[package("demo", &[], &[("main.lit", &text)])]);
    render(&db, ws)
}

/// Shared declarations for `check_body` (lines 1-9).
pub const PRELUDE: &str = "module main
start s
enum Document { complaint, answer }
enum Method { Personal, Mail }
event Served(document: Document, method: Method)
counting c6 { holidays cal }
calendar cal { new-year: fixed(1, 1) effective [1870-06-28, ..) }
state s: \"S\" { _ \"go\" -> t }
terminal t: \"T\"
";
