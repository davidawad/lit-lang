//! Loading package trees from disk (SPEC §4.1-§4.2): discovery order,
//! `path` dependencies, skipped directories, and I/O failures.
#![cfg(unix)]
mod common;

use litigate_sema::pkg::load::{load_package, load_tree, LoadedPackage};
use std::fs;
use std::path::{Path, PathBuf};

/// A fresh, empty directory unique to `test`.
fn scratch(test: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("cov_front_load")
        .join(test);
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap_or_else(|e| panic!("{e}"));
    dir
}

/// Writes `files` (`relative path`, `text`) under `dir`.
fn write(dir: &Path, files: &[(&str, &str)]) {
    for (relative, text) in files {
        let path = dir.join(relative);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap_or_else(|e| panic!("{e}"));
        }
        fs::write(path, text).unwrap_or_else(|e| panic!("{e}"));
    }
}

fn symlink(target: &str, link: &Path) {
    std::os::unix::fs::symlink(target, link).unwrap_or_else(|e| panic!("{e}"));
}

/// `(root, files, has lock)` per loaded package.
fn summary(pkgs: &[LoadedPackage]) -> Vec<(String, Vec<String>, bool)> {
    pkgs.iter()
        .map(|p| {
            let files = p.files.iter().map(|f| f.relative.clone()).collect();
            (p.root.clone(), files, p.lock.is_some())
        })
        .collect()
}

#[test]
fn trees_follow_path_dependencies_once_in_discovery_order() {
    let repo = scratch("tree");
    let app = common::manifest("app", &["lib", "gone"]);
    write(
        &repo,
        &[
            ("app/Lit.toml", &app),
            ("app/Lit.lock", "version = 1\n"),
            ("app/b.lit", "module b\n"),
            ("app/a.lit", "module a\n"),
            ("app/sub/deeper/c.lit", "module c\n"),
            ("app/sub/notes.txt", "not a source"),
            ("app/.hidden/x.lit", "module hidden\n"),
            ("app/nested/Lit.toml", &common::manifest("nested", &[])),
            ("app/nested/n.lit", "module n\n"),
            ("lib/Lit.toml", &common::manifest("lib", &["app"])),
            ("lib/lib.lit", "module lib\n"),
            ("std/Lit.toml", "not toml ["),
            ("std/core.lit", "module core\n"),
            ("plain/readme.lit", "module plain\n"),
        ],
    );
    let roots = [
        repo.join("app"),
        repo.join("app"),
        repo.join("plain"),
        repo.join("missing"),
    ];
    let pkgs = load_tree(&repo, &roots, Some(&repo.join("std"))).unwrap_or_else(|e| panic!("{e}"));
    let own = |files: &[&str]| files.iter().map(|f| (*f).to_owned()).collect::<Vec<_>>();
    assert_eq!(
        summary(&pkgs),
        [
            (
                "app".to_owned(),
                own(&["a.lit", "b.lit", "sub/deeper/c.lit"]),
                true
            ),
            ("std".to_owned(), own(&["core.lit"]), false),
            // A dependency's root is its path as reached (not normalized).
            ("app/../lib".to_owned(), own(&["lib.lit"]), false),
        ]
    );
    assert_eq!(pkgs[1].manifest, "not toml [");
}

#[test]
fn a_package_loaded_from_its_own_directory_is_the_root() {
    let repo = scratch("own-root");
    write(
        &repo,
        &[
            ("Lit.toml", &common::manifest("solo", &[])),
            ("main.lit", "module main\nstart s\nterminal s: \"S\"\n"),
        ],
    );
    let (db, ws) = litigate_sema::load(&repo, std::slice::from_ref(&repo), None)
        .unwrap_or_else(|e| panic!("{e}"));
    let pkg = ws.packages(&db)[0];
    assert_eq!(pkg.root(&db), ".");
    assert_eq!(pkg.manifest_path(&db), "Lit.toml");
    assert_eq!(pkg.files(&db)[0].path(&db), "main.lit");
}

#[test]
fn an_empty_root_names_files_by_their_relative_path() {
    let mut pkg = common::package("solo", &[], &[("main.lit", "module main\n")]);
    pkg.root = String::new();
    let (db, ws) = common::workspace(&[pkg]);
    let pkg = ws.packages(&db)[0];
    assert_eq!(pkg.manifest_path(&db), "Lit.toml");
    assert_eq!(pkg.files(&db)[0].path(&db), "main.lit");
}

#[test]
fn unreadable_sources_fail_the_load() {
    let repo = scratch("dangling");
    write(
        &repo,
        &[
            ("app/Lit.toml", &common::manifest("app", &[])),
            ("app/sub/ok.lit", "module ok\n"),
        ],
    );
    symlink("nowhere.lit", &repo.join("app/sub/broken.lit"));
    let roots = [repo.join("app")];
    let err = load_tree(&repo, &roots, None).err();
    assert_eq!(err.map(|e| e.kind()), Some(std::io::ErrorKind::NotFound));
    assert!(litigate_sema::load(&repo, &roots, None).is_err());
}

#[test]
fn an_unreadable_lock_fails_the_load() {
    let repo = scratch("lock-dir");
    write(&repo, &[("app/Lit.toml", &common::manifest("app", &[]))]);
    fs::create_dir_all(repo.join("app/Lit.lock")).unwrap_or_else(|e| panic!("{e}"));
    assert!(load_package(&repo, &repo.join("app")).is_err());
}

#[test]
fn a_directory_without_a_manifest_is_not_a_package() {
    let repo = scratch("no-manifest");
    let err = load_package(&repo, &repo).err();
    assert_eq!(err.map(|e| e.kind()), Some(std::io::ErrorKind::NotFound));
}

#[test]
fn an_unreadable_directory_fails_the_load() {
    use std::os::unix::fs::PermissionsExt;
    let repo = scratch("locked");
    write(
        &repo,
        &[
            ("app/Lit.toml", &common::manifest("app", &[])),
            ("app/locked/x.lit", "module x\n"),
        ],
    );
    let locked = repo.join("app/locked");
    let set = |mode| fs::set_permissions(&locked, fs::Permissions::from_mode(mode));
    set(0o000).unwrap_or_else(|e| panic!("{e}"));
    let readable_anyway = fs::read_dir(&locked).is_ok();
    let result = load_package(&repo, &repo.join("app"));
    set(0o755).unwrap_or_else(|e| panic!("{e}"));
    // Permissions do not bind a privileged user; the failure is then moot.
    assert_eq!(result.is_err(), !readable_anyway);
}
