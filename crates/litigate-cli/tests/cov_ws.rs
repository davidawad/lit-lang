//! Input discovery and loading (`litigate_cli::ws`): what each kind of
//! path names, where library packages are found, and how unreadable or
//! missing inputs fail.
// Test helpers below may unwrap: a panic is the failed assertion.
#![allow(clippy::unwrap_used)]
mod support;

use litigate_cli::ws::{self, Session, Targets};
use std::path::{Path, PathBuf};
use support::{chmod, litigate_in, repo, scratch};

fn s(p: &Path) -> String {
    p.to_string_lossy().into_owned()
}

/// `root/{pkg/Lit.toml, pkg/a.lit, pkg/m.case, pkg/w.world, .hidden/x.lit,
/// target/y.lit, notes.txt}`.
fn tree(name: &str) -> PathBuf {
    let dir = scratch(name);
    let manifest = std::fs::read_to_string(repo().join("lit/examples/Lit.toml")).unwrap();
    for (path, text) in [
        ("pkg/Lit.toml", manifest.as_str()),
        ("pkg/a.lit", "module m\n"),
        ("pkg/m.case", "case c\n"),
        ("pkg/w.world", "layers = [\"frcp-rule12-example\"]\n"),
        (".hidden/x.lit", "module x\n"),
        ("target/y.lit", "module y\n"),
        ("notes.txt", "not an input\n"),
    ] {
        std::fs::create_dir_all(dir.join(path).parent().unwrap()).unwrap();
        std::fs::write(dir.join(path), text).unwrap();
    }
    dir
}

#[test]
fn walking_skips_hidden_and_target_directories() {
    let dir = tree("ws-walk");
    let found = ws::discover(&[s(&dir)]).unwrap();
    assert_eq!(
        found,
        Targets {
            packages: vec![dir.join("pkg")],
            cases: vec![dir.join("pkg/m.case")],
            worlds: vec![dir.join("pkg/w.world")],
            files: vec![dir.join("pkg/a.lit"), dir.join("pkg/m.case")],
        }
    );
}

#[test]
fn each_kind_of_file_names_its_inputs() {
    let dir = tree("ws-files");
    let found = ws::discover(&[
        s(&dir.join("pkg/a.lit")),
        s(&dir.join("pkg/Lit.toml")),
        s(&dir.join("pkg/m.case")),
        s(&dir.join("pkg/w.world")),
    ])
    .unwrap();
    assert_eq!(found.packages, [dir.join("pkg")]);
    assert_eq!(found.worlds, [dir.join("pkg/w.world")]);
    assert_eq!(found.files, [dir.join("pkg/a.lit"), dir.join("pkg/m.case")]);
    let bare = litigate_in(&dir.join("pkg"), &["lock", "Lit.toml"]);
    assert_eq!(
        bare.json["result"]["lockfile"], "Lit.lock",
        "{}",
        bare.stderr
    );
}

#[test]
fn files_that_are_not_inputs_are_usage_errors() {
    let dir = tree("ws-errors");
    let message = |path: &Path| ws::discover(&[s(path)]).unwrap_err().0;
    let shown = s(&dir.join(".hidden/x.lit"));
    assert_eq!(
        message(&dir.join(".hidden/x.lit")),
        format!("{shown}: not inside a package (no Lit.toml above it)")
    );
    let shown = s(&dir.join("notes.txt"));
    assert_eq!(
        message(&dir.join("notes.txt")),
        format!("{shown}: not a .lit, .case, .world, or Lit.toml file")
    );
}

#[cfg(unix)]
#[test]
fn an_unreadable_directory_fails_discovery() {
    let dir = tree("ws-unreadable");
    chmod(&dir.join("pkg"), 0o000);
    let failure = ws::discover(&[s(&dir)]).map(|_| ());
    chmod(&dir.join("pkg"), 0o755);
    assert_eq!(
        failure,
        Err(litigate_cli::out::Failure::new(
            "Permission denied (os error 13)"
        ))
    );
}

#[test]
fn library_directories_default_to_lit_and_the_matters_neighbours() {
    // Tests run in the crate directory, which has no `lit/`.
    assert_eq!(ws::library_dirs(&[]), Vec::<PathBuf>::new());
    assert_eq!(
        ws::library_for(&[], &[PathBuf::from("no/such.case")]),
        Vec::<PathBuf>::new()
    );
    let named = ws::library_for(&["no/such/dir".to_owned(), "no/such/dir".to_owned()], &[]);
    assert_eq!(named, [PathBuf::from("no/such/dir")]);
    let case = repo().join("cases/examples/acme.case");
    assert_eq!(ws::library_for(&[], &[case]), [repo().join("lit")]);
}

#[cfg(unix)]
#[test]
fn library_packages_need_a_readable_valid_manifest() {
    let dir = tree("ws-library");
    for (pkg, manifest) in [("bad", "not toml ["), ("locked", "")] {
        std::fs::create_dir_all(dir.join(pkg)).unwrap();
        std::fs::write(dir.join(pkg).join("Lit.toml"), manifest).unwrap();
    }
    chmod(&dir.join("locked/Lit.toml"), 0o000);
    let found = ws::library(std::slice::from_ref(&dir));
    chmod(&dir.join("locked/Lit.toml"), 0o644);
    assert_eq!(found, [("frcp-rule12-example".to_owned(), dir.join("pkg"))]);
}

#[test]
fn display_paths_outside_the_working_directory_stay_as_given() {
    assert_eq!(
        ws::shown(Path::new("/no/such/file.lit")),
        "/no/such/file.lit"
    );
    assert_eq!(ws::shown(Path::new("src/lib.rs")), "src/lib.rs");
}

#[test]
fn diagnostic_files_resolve_root_relative_paths() {
    assert_eq!(
        Session::real_path("src/lib.rs"),
        PathBuf::from("src/lib.rs")
    );
    assert_eq!(Session::real_path("tmp"), PathBuf::from("/tmp"));
    assert_eq!(
        Session::real_path("no/such/file"),
        PathBuf::from("no/such/file")
    );
}

#[cfg(unix)]
#[test]
fn unreadable_inputs_fail_loading() {
    let dir = support::example("ws-load", "", None);
    chmod(&dir.join("cases/examples/acme.case"), 0o000);
    let case = litigate_in(&dir, &["check", "cases/examples/acme.case"]);
    let run = litigate_in(&dir, &["run", "cases/examples/acme.case", "--op", "chain"]);
    chmod(&dir.join("cases/examples/acme.case"), 0o644);
    assert_eq!((case.code, run.code), (1, 1));
    insta::assert_snapshot!(case.stderr, @"litigate check: cases/examples/acme.case: Permission denied (os error 13)");
    chmod(&dir.join("lit/examples/Lit.toml"), 0o000);
    let pkg = litigate_in(&dir, &["check", "lit/examples"]);
    chmod(&dir.join("lit/examples/Lit.toml"), 0o644);
    assert_eq!(pkg.code, 1);
    insta::assert_snapshot!(pkg.stderr, @"litigate check: Permission denied (os error 13)");
    std::fs::write(dir.join("w.world"), "layers = [\"frcp-rule12-example\"]\n").unwrap();
    chmod(&dir.join("w.world"), 0o000);
    let world = litigate_in(&dir, &["check", "w.world"]);
    chmod(&dir.join("w.world"), 0o644);
    assert_eq!(world.code, 1);
    insta::assert_snapshot!(world.stderr, @"litigate check: w.world: Permission denied (os error 13)");
}

/// Runs `litigate --format json ARGS` from a working directory that no
/// longer exists.
#[cfg(unix)]
fn in_deleted_directory(args: &[&str]) -> support::Run {
    let dir = scratch("ws-deleted").join("gone");
    std::fs::create_dir_all(&dir).unwrap();
    let out = assert_cmd::Command::new("sh")
        .arg("-c")
        .arg("cd \"$1\" && rmdir \"$1\" && shift && exec \"$@\"")
        .arg("sh")
        .arg(&dir)
        .arg(assert_cmd::cargo::cargo_bin("litigate"))
        .args(["--format", "json"])
        .args(args)
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
    support::Run {
        json: serde_json::from_str(&stdout).unwrap_or_default(),
        stdout,
        stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
        code: out.status.code().unwrap_or(-1),
    }
}

#[cfg(unix)]
#[test]
fn a_deleted_working_directory_is_an_io_failure() {
    let pkg = repo().join("lit/examples");
    let run = in_deleted_directory(&["test", pkg.to_str().unwrap()]);
    assert_eq!(run.code, 1, "{}", run.stdout);
    insta::assert_snapshot!(run.stderr, @"litigate test: No such file or directory (os error 2)");
    let describe = in_deleted_directory(&["describe"]);
    assert_eq!(describe.code, 0, "{}", describe.stderr);
    assert_eq!(describe.json["result"]["stdlib"], serde_json::json!([]));
}
