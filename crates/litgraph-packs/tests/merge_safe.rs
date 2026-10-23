// SPDX-License-Identifier: GPL-3.0-or-later
//! Parallel `lit/` changes merge without an integration job (lg-flu.31):
//! the generated packs and scenarios are build outputs, so two branches
//! editing different packages touch only their own `.lit` sources and the
//! per-package `Lit.lock` entries, and rebase cleanly with no merge driver
//! (landing runs where cargo, so the `litlock` driver, is unavailable).
//! The control commits the generated files as the repository used to: the
//! same two edits then conflict on them. Needs `git`; skips without it.
// Test helpers below may unwrap: a panic is the failed assertion.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::path::{Path, PathBuf};
use std::process::Command;

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn skip(test: &str, reason: &str) {
    let line = format!("SKIP merge_safe::{test}: {reason}\n");
    eprint!("{line}");
    if let Ok(log) = std::env::var("TEST_SKIP_LOG") {
        let _ = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(log)
            .and_then(|mut file| std::io::Write::write_all(&mut file, line.as_bytes()));
    }
}

fn has_git() -> bool {
    Command::new("git")
        .arg("--version")
        .output()
        .is_ok_and(|o| o.status.success())
}

/// `git -C dir ARGS` with a fixed identity and no user or system config
/// (so no merge driver is registered); its success and combined output.
fn git(dir: &Path, args: &[&str]) -> (bool, String) {
    let out = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(["-c", "user.name=t", "-c", "user.email=t@example.com"])
        .args([
            "-c",
            "commit.gpgsign=false",
            "-c",
            "init.defaultBranch=main",
        ])
        .args(args)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .output()
        .unwrap();
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    (out.status.success(), text)
}

fn git_ok(dir: &Path, args: &[&str]) -> String {
    let (ok, text) = git(dir, args);
    assert!(ok, "git {args:?}: {text}");
    text
}

/// Copies `from` into `to`, recursively.
fn copy_tree(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for entry in std::fs::read_dir(from).unwrap() {
        let path = entry.unwrap().path();
        let target = to.join(path.file_name().unwrap());
        if path.is_dir() {
            copy_tree(&path, &target);
        } else {
            std::fs::copy(&path, &target).unwrap();
        }
    }
}

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("litgraph-packs-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// `litigate ARGS` (the library, so no binary is needed); exit code and output.
fn litigate(args: &[&str]) -> (u8, String) {
    let printed = litigate_cli::execute(std::iter::once("litigate").chain(args.iter().copied()));
    (
        printed.code,
        format!("{}{}", printed.stdout, printed.stderr),
    )
}

/// The `build.args` arguments, inputs made absolute under `root`.
fn build_args(root: &Path) -> Vec<String> {
    let text = std::fs::read_to_string(root.join("crates/litgraph-packs/build.args")).unwrap();
    text.lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(|l| {
            if l.starts_with("lit/") || l.starts_with("cases/") {
                root.join(l).display().to_string()
            } else {
                l.to_owned()
            }
        })
        .collect()
}

/// What `just lit-sync` does in `root`: relock every package and the
/// matters, then build. `into_tree` also writes the build where the
/// repository used to commit it (`packs/`, `scenarios/`).
fn lit_sync(root: &Path, into_tree: bool) {
    let lit = root.join("lit");
    for entry in std::fs::read_dir(&lit).unwrap() {
        let dir = entry.unwrap().path();
        if dir.join("Lit.lock").is_file() {
            let (code, out) = litigate(&["--format", "human", "lock", dir.to_str().unwrap()]);
            assert_eq!(code, 0, "lock {}: {out}", dir.display());
        }
    }
    let cases = root.join("cases");
    let (code, out) = litigate(&["--format", "human", "lock", cases.to_str().unwrap()]);
    assert_eq!(code, 0, "lock cases: {out}");
    let out_dir = root.join("target/lit-build");
    let _ = std::fs::remove_dir_all(&out_dir);
    let mut args = vec![
        "--format".to_owned(),
        "human".to_owned(),
        "build".to_owned(),
    ];
    args.extend(build_args(root));
    args.extend(["--out".to_owned(), out_dir.display().to_string()]);
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    let (code, out) = litigate(&args);
    assert_eq!(code, 0, "build: {out}");
    if into_tree {
        for dir in ["packs", "packs/clock", "scenarios"] {
            std::fs::create_dir_all(root.join(dir)).unwrap();
            for entry in std::fs::read_dir(out_dir.join(dir)).unwrap() {
                let path = entry.unwrap().path();
                let name = path.file_name().unwrap().to_string_lossy().into_owned();
                if path.extension().is_some_and(|e| e == "json") && !name.ends_with(".map.json") {
                    std::fs::copy(&path, root.join(dir).join(&name)).unwrap();
                }
            }
        }
    }
}

/// Replaces the first `from` in `root/file` with `to`.
fn edit(root: &Path, file: &str, from: &str, to: &str) {
    let path = root.join(file);
    let text = std::fs::read_to_string(&path).unwrap();
    assert!(text.contains(from), "{file} has no {from:?}");
    std::fs::write(&path, text.replacen(from, to, 1)).unwrap();
}

/// A repository holding what this one tracks for `lit/` builds, on `main`,
/// with branch `a` editing `lit/frcp` and branch `b` editing `lit/ttab`,
/// each after a `lit-sync`. `committed` commits the generated files too.
fn two_branches(name: &str, committed: bool) -> PathBuf {
    let root = scratch(name);
    let src = repo();
    for dir in ["lit", "cases", "packs/clock"] {
        copy_tree(&src.join(dir), &root.join(dir));
    }
    std::fs::create_dir_all(root.join("crates/litgraph-packs")).unwrap();
    for file in [
        ".gitignore",
        ".gitattributes",
        "crates/litgraph-packs/build.args",
    ] {
        std::fs::copy(src.join(file), root.join(file)).unwrap();
    }
    git_ok(&root, &["init", "-q"]);
    let add: &[&str] = if committed {
        &[
            "add",
            "-A",
            "-f",
            "lit",
            "cases",
            "packs",
            "scenarios",
            "crates",
            ".gitignore",
            ".gitattributes",
        ]
    } else {
        &["add", "-A"]
    };
    lit_sync(&root, committed);
    git_ok(&root, add);
    git_ok(&root, &["commit", "-q", "-m", "base"]);
    let edits = [
        (
            "a",
            "lit/frcp/frcp.lit",
            "\"Pre-Filing, Jurisdiction & Venue\"",
            "\"Pre-Filing, Jurisdiction and Venue\"",
        ),
        (
            "b",
            "lit/ttab/ttab.lit",
            "\"Pre-opposition\"",
            "\"Before opposition\"",
        ),
    ];
    for (branch, file, from, to) in edits {
        git_ok(&root, &["checkout", "-q", "-b", branch, "main"]);
        edit(&root, file, from, to);
        lit_sync(&root, committed);
        git_ok(&root, add);
        git_ok(&root, &["commit", "-q", "-m", branch]);
    }
    root
}

#[test]
fn two_branches_editing_different_lit_packages_rebase_cleanly() {
    if !has_git() {
        return skip(
            "two_branches_editing_different_lit_packages_rebase_cleanly",
            "no git",
        );
    }
    let root = two_branches("clean", false);
    let tracked = git_ok(&root, &["ls-files"]);
    assert!(
        !tracked.lines().any(|f| f.starts_with("scenarios/")
            || (f.starts_with("packs/") && !f.starts_with("packs/clock/"))
            || f == "packs/clock/mpep-prosecution.json"),
        "a generated file is tracked:\n{tracked}"
    );
    let changed = git_ok(&root, &["diff", "--name-only", "main", "b"]);
    assert!(
        changed
            .lines()
            .all(|f| f.starts_with("lit/") || f.starts_with("cases/")),
        "{changed}"
    );
    let (ok, out) = git(&root, &["rebase", "a"]);
    assert!(ok, "rebase conflicted:\n{out}");
    // The rebased tree holds both edits and builds as is: its locks are
    // current (no L1007), with no `lit-sync` after the rebase.
    for (file, text) in [
        ("lit/frcp/frcp.lit", "Jurisdiction and Venue"),
        ("lit/ttab/ttab.lit", "Before opposition"),
    ] {
        assert!(std::fs::read_to_string(root.join(file))
            .unwrap()
            .contains(text));
    }
    let mut args = vec![
        "--format".to_owned(),
        "human".to_owned(),
        "build".to_owned(),
    ];
    args.extend(build_args(&root));
    args.extend([
        "--out".to_owned(),
        root.join("rebased").display().to_string(),
    ]);
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    let (code, out) = litigate(&args);
    assert_eq!(code, 0, "{out}");
    for lock in [
        root.join("lit/frcp"),
        root.join("lit/ttab"),
        root.join("cases"),
    ] {
        let (code, out) = litigate(&[
            "--format",
            "human",
            "lock",
            "--locked",
            lock.to_str().unwrap(),
        ]);
        assert_eq!(code, 0, "{}: {out}", lock.display());
    }
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn committed_generated_files_conflict_on_the_same_edits() {
    if !has_git() {
        return skip(
            "committed_generated_files_conflict_on_the_same_edits",
            "no git",
        );
    }
    let root = two_branches("committed", true);
    let (ok, out) = git(&root, &["rebase", "a"]);
    assert!(!ok, "expected a conflict:\n{out}");
    let conflicted = git_ok(&root, &["diff", "--name-only", "--diff-filter=U"]);
    assert!(
        conflicted
            .lines()
            .all(|f| f.starts_with("packs/") || f.starts_with("scenarios/")),
        "only generated files conflict:\n{conflicted}"
    );
    assert!(!conflicted.trim().is_empty());
    let _ = git(&root, &["rebase", "--abort"]);
    let _ = std::fs::remove_dir_all(&root);
}

/// No file this crate builds is tracked by, or unignored in, this
/// repository (skipped outside a git checkout, e.g. a packaging build).
#[test]
fn no_generated_file_is_committed_here() {
    let root = repo();
    if !has_git() || !git(&root, &["rev-parse", "--git-dir"]).0 {
        return skip("no_generated_file_is_committed_here", "not a git checkout");
    }
    let hand: Vec<String> = std::fs::read_dir(root.join("packs/clock"))
        .unwrap()
        .map(|e| format!("clock/{}", e.unwrap().file_name().to_string_lossy()))
        .collect();
    let built = litgraph_packs::PACKS
        .iter()
        .filter(|(name, _)| !hand.iter().any(|h| h == name))
        .map(|(name, _)| format!("packs/{name}"))
        .chain(
            litgraph_packs::SCENARIOS
                .iter()
                .map(|(name, _)| format!("scenarios/{name}")),
        );
    for path in built {
        let (tracked, _) = git(&root, &["ls-files", "--error-unmatch", &path]);
        assert!(!tracked, "{path} is committed; it is a build output");
        if !path.starts_with("packs/clock/") {
            let (ignored, _) = git(&root, &["check-ignore", "-q", "--no-index", &path]);
            assert!(ignored, "{path} is not gitignored");
        }
    }
}
