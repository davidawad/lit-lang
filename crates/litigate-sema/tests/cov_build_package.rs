//! Package-level checks and lockfiles of `types::build` (SPEC §4.1-§4.3):
//! manifests, dependencies, package shape, namespaces, and `Lit.lock`.
mod common;
use common::{manifest, package, render, workspace};
use litigate_sema::pkg::load::LoadedPackage;

const PACK: &str = "module m\nstart s\nterminal s: \"S\"\n";

/// Package `name` at version `version` with path dependencies `deps`.
fn versioned(name: &str, version: &str, deps: &[&str], text: &str) -> LoadedPackage {
    LoadedPackage {
        texts: std::collections::BTreeMap::new(),
        manifest: manifest(name, deps)
            .replace("version = \"1.0.0\"", &format!("version = \"{version}\"")),
        ..package(name, deps, &[(&format!("{name}.lit"), text)])
    }
}

#[test]
fn dependencies_must_resolve_to_a_satisfying_version() {
    let (db, ws) = workspace(&[
        package("root", &["old", "gone"], &[("root.lit", PACK)]),
        versioned(
            "old",
            "2.0.0",
            &[],
            "module old\nstart s\nterminal s: \"S\"\n",
        ),
    ]);
    insta::assert_debug_snapshot!(render(&db, ws));
}

#[test]
fn an_invalid_manifest_reports_and_builds_no_package() {
    let pkg = LoadedPackage {
        manifest: "[package]\nname = 3\n".to_owned(),
        ..package(
            "bad",
            &[],
            &[("bad.lit", "module bad\ndefault d when true => 1\n")],
        )
    };
    let (db, ws) = workspace(&[pkg]);
    let checks = litigate_sema::check_workspace(&db, ws);
    assert!(checks.iter().all(|c| c.package.is_none()));
    insta::assert_debug_snapshot!(render(&db, ws));
    // Without a manifest no dependency resolves, so the lock holds no entry.
    let lock = litigate_sema::render_lock(&db, ws, ws.packages(&db)[0]);
    assert!(!lock.contains("[[package]]"), "{lock}");
}

#[test]
fn render_lock_handles_a_valid_package() {
    let (db, ws) = workspace(&[package("root", &[], &[("root.lit", PACK)])]);
    let lock = litigate_sema::render_lock(&db, ws, ws.packages(&db)[0]);
    assert!(lock.contains("name = \"root\""), "{lock}");
}

#[test]
fn a_package_without_files_needs_a_start() {
    let (db, ws) = workspace(&[package("empty", &[], &[])]);
    insta::assert_debug_snapshot!(render(&db, ws));
}

#[test]
fn a_links_package_declares_no_nodes() {
    let base = "module base\nstart s\nstate s: \"S\" { go \"Go\" -> t }\nterminal t: \"T\"\n";
    let links = "module links\nlink up \"Up\": base::t -> base::s\nstate stray: \"Stray\"\n";
    let (db, ws) = workspace(&[
        package("base", &[], &[("base.lit", base)]),
        package("links", &["base"], &[("links.lit", links)]),
    ]);
    insta::assert_debug_snapshot!(render(&db, ws));
}

#[test]
fn two_files_of_one_module_share_its_namespace() {
    let a = "module m\nstart s\nterminal s: \"S\"\ndefault x when true => 1\n";
    let b = "module m\ndefault x when true => 2\n";
    let (db, ws) = workspace(&[package("demo", &[], &[("a.lit", a), ("b.lit", b)])]);
    insta::assert_debug_snapshot!(render(&db, ws));
}

#[test]
fn the_lock_lists_each_transitive_dependency_once() {
    // root -> left -> shared and root -> right -> shared (a diamond).
    let (db, ws) = workspace(&[
        package("root", &["left", "right"], &[("root.lit", PACK)]),
        package(
            "left",
            &["shared"],
            &[("left.lit", "module left\nstart s\nterminal s: \"S\"\n")],
        ),
        package(
            "right",
            &["shared"],
            &[("right.lit", "module right\nstart s\nterminal s: \"S\"\n")],
        ),
        package(
            "shared",
            &[],
            &[("shared.lit", "module shared\nstart s\nterminal s: \"S\"\n")],
        ),
    ]);
    let lock = litigate_sema::render_lock(&db, ws, ws.packages(&db)[0]);
    let parsed = litigate_sema::pkg::lock::LockFile::parse(&lock).expect("a lock");
    let names: Vec<String> = parsed
        .packages
        .iter()
        .map(|p| format!("{} <- {}", p.name, p.dependencies.join(", ")))
        .collect();
    insta::assert_debug_snapshot!(names);
    // Each dependency is pinned by name and version; its Merkle digest is
    // computed on read, once per row of the diamond.
    let root = parsed.packages.iter().find(|p| p.name == "root");
    assert_eq!(
        root.map(|r| r.dependencies.clone()),
        Some(vec!["left 1.0.0".to_owned(), "right 1.0.0".to_owned()])
    );
    assert_eq!(parsed.merkle().len(), 4);
}
