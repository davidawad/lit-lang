// SPDX-License-Identifier: GPL-3.0-or-later
//! Records the git commit the binary was built from, and whether the
//! working tree had uncommitted changes, so `litigate --version` names the
//! exact build installed (lg-rf4m).
//!
//! Sets `LITIGATE_GIT_COMMIT` (the full sha, or `unknown` outside a git
//! checkout) and `LITIGATE_GIT_TREE` (`clean`, `dirty`, or `unknown`).
//! Builds without a `.git` (a Docker context, a Nix source tarball) can set
//! either variable in the environment to supply it.

use std::path::Path;
use std::process::Command;

fn main() {
    let manifest = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_default();
    let dir = Path::new(&manifest);
    for var in ["LITIGATE_GIT_COMMIT", "LITIGATE_GIT_TREE"] {
        println!("cargo:rerun-if-env-changed={var}");
    }
    // Rerun when HEAD moves (checkout, commit) or the index changes
    // (staging; `git status` refreshes it after an edit too).
    for path in ["HEAD", "index", "packed-refs"] {
        if let Some(p) = git(dir, &["rev-parse", "--git-path", path]) {
            println!("cargo:rerun-if-changed={}", dir.join(p).display());
        }
    }
    if let Some(head) = git(dir, &["symbolic-ref", "-q", "HEAD"]) {
        if let Some(p) = git(dir, &["rev-parse", "--git-path", &head]) {
            println!("cargo:rerun-if-changed={}", dir.join(p).display());
        }
    }

    let commit = std::env::var("LITIGATE_GIT_COMMIT")
        .ok()
        .or_else(|| git(dir, &["rev-parse", "HEAD"]))
        .unwrap_or_else(|| "unknown".to_owned());
    let tree = std::env::var("LITIGATE_GIT_TREE").ok().unwrap_or_else(|| {
        match git(dir, &["status", "--porcelain", "--untracked-files=no"]) {
            Some(s) if s.is_empty() => "clean".to_owned(),
            Some(_) => "dirty".to_owned(),
            None => "unknown".to_owned(),
        }
    });
    println!("cargo:rustc-env=LITIGATE_GIT_COMMIT={commit}");
    println!("cargo:rustc-env=LITIGATE_GIT_TREE={tree}");
}

/// `git ARGS` run in `dir`, its trimmed stdout, or `None` when git is
/// missing or fails (not a checkout).
fn git(dir: &Path, args: &[&str]) -> Option<String> {
    let out = Command::new("git")
        .args(args)
        .current_dir(dir)
        .output()
        .ok()?;
    out.status
        .success()
        .then(|| String::from_utf8_lossy(&out.stdout).trim().to_owned())
}
