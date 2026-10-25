//! Lock v2 rows (SPEC §4.2): vendored L0 text digests read through the
//! loader, an `unchecked` row, a dependency cycle, and the one-version
//! cross-check of a resolution against its dependencies' own.
#![allow(clippy::unwrap_used)]
mod common;
use common::{manifest, package, render, workspace};
use litigate_sema::pkg::lock::{LockFile, LockedPackage, UNCHECKED};
use litigate_sema::pkg::source::MemoryTree;
use salsa::Setter;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

const PACK: &str = "module m\nstart s\nterminal s: \"S\"\n";

fn row<'l>(lock: &'l LockFile, name: &str) -> &'l LockedPackage {
    lock.packages.iter().find(|p| p.name == name).unwrap()
}

#[test]
fn the_lock_pins_vendored_l0_texts_only() {
    let toml = format!(
        "{}\n[sources.here]\nurl = \"https://example.org\"\npath = \"sources/here.txt\"\n\n\
         [sources.gone]\nurl = \"https://example.org\"\npath = \"sources/gone.txt\"\n\n\
         [sources.pack]\nurl = \"https://example.org\"\npath = \"packs/x.json\"\n",
        manifest("app", &[])
    );
    let tree = MemoryTree::new([
        ("lit/app/Lit.toml", toml),
        ("lit/app/m.lit", PACK.to_owned()),
        ("sources/here.txt", "## Rule 1\ntext\n".to_owned()),
        ("packs/x.json", "{}".to_owned()),
    ]);
    let (db, ws) =
        litigate_sema::load_in(&tree, Path::new("/"), &[PathBuf::from("/lit/app")], None).unwrap();
    let lock = LockFile::parse(&litigate_sema::render_lock(&db, ws, ws.packages(&db)[0])).unwrap();
    insta::assert_debug_snapshot!(row(&lock, "app").texts);
    // The pinned text is part of the Merkle digest: changing it stales the lock.
    let edited = MemoryTree::new([
        ("lit/app/Lit.toml", tree_text(&db, ws)),
        ("lit/app/m.lit", PACK.to_owned()),
        ("sources/here.txt", "## Rule 1\namended\n".to_owned()),
    ]);
    let (db2, ws2) =
        litigate_sema::load_in(&edited, Path::new("/"), &[PathBuf::from("/lit/app")], None)
            .unwrap();
    let again = LockFile::parse(&litigate_sema::render_lock(
        &db2,
        ws2,
        ws2.packages(&db2)[0],
    ))
    .unwrap();
    assert_eq!(row(&again, "app").semantic, row(&lock, "app").semantic);
    assert_ne!(row(&again, "app").texts, row(&lock, "app").texts);
    assert_ne!(again.root(), lock.root());
}

fn tree_text(db: &litigate_sema::LitDatabase, ws: litigate_sema::Workspace) -> String {
    ws.packages(db)[0].manifest_text(db).clone()
}

#[test]
fn a_package_that_does_not_check_locks_its_source_digest() {
    let (db, ws) = workspace(&[package(
        "app",
        &[],
        &[("m.lit", "module m\nstart nowhere\n")],
    )]);
    let lock = LockFile::parse(&litigate_sema::render_lock(&db, ws, ws.packages(&db)[0])).unwrap();
    let app = row(&lock, "app");
    assert_eq!(app.semantic, UNCHECKED);
    assert_eq!(
        lock.merkle()["app 1.0.0"],
        litigate_sema::pkg::lock::merkle("app", "1.0.0", 1, &app.source, &[], &[])
    );
}

#[test]
fn stale_lock_is_reported_with_its_source_location() {
    let (mut db, ws) = workspace(&[package("app", &[], &[("m.lit", PACK)])]);
    let pkg = ws.packages(&db)[0];
    pkg.set_lock_text(&mut db)
        .to(Some("not a valid lock\n".to_owned()));
    let diagnostics = litigate_sema::lock_diagnostics(&db, ws, pkg);
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].span.file.0, "pkgs/app/Lit.lock");
    assert_eq!(diagnostics[0].span.line, 1);
}

#[test]
fn a_dependency_cycle_is_l1011_and_entered_by_name_and_version() {
    let (db, ws) = workspace(&[
        package("a", &["b"], &[("a.lit", PACK)]),
        package(
            "b",
            &["a"],
            &[("b.lit", "module n\nstart t\nterminal t: \"T\"\n")],
        ),
    ]);
    insta::assert_debug_snapshot!(render(&db, ws));
    // Each row pins the other by name and version only; the Merkle digests
    // computed on read cut the edge back to where the walk began.
    let lock = LockFile::parse(&litigate_sema::render_lock(&db, ws, ws.packages(&db)[0])).unwrap();
    let (a, b) = (row(&lock, "a"), row(&lock, "b"));
    assert_eq!(a.dependencies, ["b 1.0.0"]);
    assert_eq!(b.dependencies, ["a 1.0.0"]);
    assert_eq!(lock.merkle().len(), 2);
}

#[test]
fn a_dependency_checked_against_other_versions_is_l1011() {
    let versioned = |name: &str, version: &str, deps: &[(&str, &str)]| {
        let mut p = package(
            name,
            &[],
            &[(
                &format!("{name}.lit"),
                &format!("module {name}\nstart s\nterminal s: \"S\"\n"),
            )],
        );
        p.root = format!("pkgs/{name}-{version}");
        p.manifest =
            manifest(name, &[]).replace("version = \"1.0.0\"", &format!("version = \"{version}\""));
        if !deps.is_empty() {
            p.manifest.push_str("\n[dependencies]\n");
            for (d, req) in deps {
                let _ = writeln!(p.manifest, "{d} = {{ version = \"{req}\" }}");
            }
        }
        p
    };
    let (db, ws) = workspace(&[
        versioned("app", "1.0.0", &[("core", "=1.0.0"), ("frcp", "^1.0.0")]),
        versioned("frcp", "1.0.0", &[("core", "^1.0.0")]),
        versioned("core", "1.0.0", &[]),
        versioned("core", "1.1.0", &[]),
    ]);
    insta::assert_debug_snapshot!(render(&db, ws));
}

#[test]
fn an_undeclared_core_is_the_highest_loaded_version() {
    let core = |version: &str, extra: &str| {
        let mut p = package(
            "core",
            &[],
            &[(
                "core.lit",
                &format!("module core\nstart s\nterminal s: \"S\"\n{extra}"),
            )],
        );
        p.root = format!("pkgs/core-{version}");
        p.manifest = manifest("core", &[])
            .replace("version = \"1.0.0\"", &format!("version = \"{version}\""));
        p
    };
    let app = "module app\nimport core.{ newer }\nstart a\nterminal a: \"A\"\n";
    let (db, ws) = workspace(&[
        package("app", &[], &[("app.lit", app)]),
        core("1.0.0", ""),
        core("1.1.0", "group newer: \"Newer\"\n"),
    ]);
    assert_eq!(render(&db, ws), Vec::<String>::new());
}
