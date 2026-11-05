// SPDX-License-Identifier: GPL-3.0-or-later
//! Parallel `lit/` changes merge without an integration job (lg-flu.31):
//! generated packs and scenarios are build outputs, so branches editing
//! different `lit/core` modules touch independent module digests and clock
//! fragments, and merge cleanly with no merge driver.
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
    let core = root.join("lit/core");
    let (code, out) = litigate(&["--format", "human", "lock", core.to_str().unwrap()]);
    assert_eq!(code, 0, "lock core: {out}");
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
    let core_out = root.join("target/core-clock-build");
    let (code, out) = litigate(&[
        "--format",
        "human",
        "build",
        core.to_str().unwrap(),
        "--out",
        core_out.to_str().unwrap(),
    ]);
    assert_eq!(code, 0, "build core clock: {out}");
    let split = Command::new("python3")
        .arg(root.join("scripts/split-core-clock.py"))
        .arg(core_out.join("packs/clock/litigate-core.json"))
        .current_dir(root)
        .output()
        .unwrap();
    assert!(
        split.status.success(),
        "split core clock: {}",
        String::from_utf8_lossy(&split.stderr)
    );
    let _ = std::fs::remove_file(root.join("crates/litigate-engine/clock/litigate-core.json"));
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
/// with branches editing distinct `lit/core` state modules,
/// each after a `lit-sync`. `committed` commits the generated files too.
fn two_branches(name: &str, committed: bool, same_module: bool) -> PathBuf {
    let root = scratch(name);
    let src = repo();
    for dir in ["lit", "cases", "crates/litigate-engine/clock"] {
        copy_tree(&src.join(dir), &root.join(dir));
    }
    #[cfg(unix)]
    std::os::unix::fs::symlink(src.join("sources"), root.join("sources")).unwrap();
    std::fs::create_dir_all(root.join("crates/litgraph-packs")).unwrap();
    std::fs::create_dir_all(root.join("scripts")).unwrap();
    for file in [
        ".gitignore",
        "crates/litgraph-packs/build.args",
        "scripts/split-core-clock.py",
    ] {
        std::fs::copy(src.join(file), root.join(file)).unwrap();
    }
    std::env::set_current_dir(&root).unwrap();
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
        ]
    } else {
        &["add", "-A"]
    };
    lit_sync(&root, committed);
    git_ok(&root, add);
    git_ok(&root, &["commit", "-q", "-m", "base"]);
    let edits = if same_module {
        [
            (
                "a",
                "lit/core/states/hawaii.lit",
                "Hawaii state holidays",
                "Hawaii holidays",
            ),
            (
                "b",
                "lit/core/states/hawaii.lit",
                "Hawaii state holidays",
                "Hawaii legal holidays",
            ),
        ]
    } else {
        [
            (
                "a",
                "lit/core/states/delaware.lit",
                "Delaware (1 Del. C. § 501(a))",
                "Delaware holidays (1 Del. C. § 501(a))",
            ),
            (
                "b",
                "lit/core/states/hawaii.lit",
                "Hawaii state holidays",
                "Hawaii holidays",
            ),
        ]
    };
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
fn two_branches_editing_different_core_modules_merge_cleanly_without_regen() {
    if !has_git() {
        return skip(
            "two_branches_editing_different_core_modules_merge_cleanly_without_regen",
            "no git",
        );
    }
    let root = two_branches("clean", false, false);
    let tracked = git_ok(&root, &["ls-files"]);
    assert!(
        !tracked.lines().any(|f| f.starts_with("scenarios/")
            || (f.starts_with("packs/") && Path::new(f).extension().is_some_and(|e| e == "json"))),
        "a generated file is tracked:\n{tracked}"
    );
    let changed = git_ok(&root, &["diff", "--name-only", "main", "b"]);
    assert!(
        changed.lines().all(|f| f.starts_with("lit/")
            || f.starts_with("cases/")
            || f.starts_with("crates/litigate-engine/clock/core/")),
        "{changed}"
    );
    git_ok(&root, &["checkout", "-q", "a"]);
    let (ok, out) = git(&root, &["merge", "--no-edit", "b"]);
    assert!(ok, "merge conflicted:\n{out}");
    // The rebased tree holds both edits and builds as is: its locks are
    // current (no L1007), with no `lit-sync` after the rebase.
    for (file, text) in [
        ("lit/core/states/delaware.lit", "Delaware holidays"),
        ("lit/core/states/hawaii.lit", "Hawaii holidays"),
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
        root.join("target/lit-build").display().to_string(),
    ]);
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    let (code, out) = litigate(&args);
    assert_eq!(code, 0, "post-merge build without regeneration: {out}");
    for lock in [root.join("lit/core"), root.join("cases")] {
        let (code, out) = litigate(&[
            "--format",
            "human",
            "lock",
            "--locked",
            lock.to_str().unwrap(),
        ]);
        assert_eq!(code, 0, "{}: {out}", lock.display());
    }
    let status = git_ok(&root, &["status", "--porcelain", "--untracked-files=all"]);
    assert!(
        status.trim().is_empty(),
        "merge/build changed files; regeneration was required:\n{status}"
    );
    std::env::set_current_dir(repo()).unwrap();
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn two_branches_editing_the_same_core_module_conflict_or_fail_locked_check() {
    if !has_git() {
        return skip(
            "two_branches_editing_the_same_core_module_conflict_or_fail_locked_check",
            "no git",
        );
    }
    let root = two_branches("same-module", false, true);
    git_ok(&root, &["checkout", "-q", "a"]);
    let (merged, out) = git(&root, &["merge", "--no-edit", "b"]);
    if merged {
        let (code, output) = litigate(&["--format", "human", "lock", "--locked", "lit/core"]);
        assert_ne!(
            code, 0,
            "same-module merge unexpectedly passed lock verification: {output}"
        );
    } else {
        let conflicts = git_ok(&root, &["diff", "--name-only", "--diff-filter=U"]);
        assert!(
            conflicts.lines().any(|f| f.contains("hawaii")),
            "{out}\n{conflicts}"
        );
        let _ = git(&root, &["merge", "--abort"]);
    }
    std::env::set_current_dir(repo()).unwrap();
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
    let built = litgraph_packs::PACKS
        .iter()
        .map(|(name, _)| format!("packs/{name}"))
        .chain(
            litgraph_packs::SCENARIOS
                .iter()
                .map(|(name, _)| format!("scenarios/{name}")),
        );
    for path in built {
        if path.starts_with("packs/clock/core/") {
            let source = format!(
                "crates/litigate-engine/clock/{}",
                path.strip_prefix("packs/clock/").unwrap()
            );
            let (tracked, _) = git(&root, &["ls-files", "--error-unmatch", &source]);
            assert!(tracked, "{path} is a committed shared clock module");
            continue;
        }
        let (tracked, _) = git(&root, &["ls-files", "--error-unmatch", &path]);
        assert!(!tracked, "{path} is committed; it is a build output");
        let (ignored, _) = git(&root, &["check-ignore", "-q", "--no-index", &path]);
        assert!(ignored, "{path} is not gitignored");
    }
}
