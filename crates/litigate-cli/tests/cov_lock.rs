//! `lock` for a package and for directories of matters: writing, leaving
//! a current lock alone, `--locked` verification, and failures.
// Test helpers below may unwrap: a panic is the failed assertion.
#![allow(clippy::unwrap_used)]
mod support;

use support::{example, litigate_in, located, scratch};

#[test]
fn a_package_lock_is_written_once_then_left_alone() {
    let dir = example("lock-package", "", None);
    let first = litigate_in(&dir, &["lock", "lit/examples"]);
    assert_eq!(first.code, 0, "{}", first.stderr);
    assert_eq!(first.json["result"]["changed"], true);
    assert!(dir.join("lit/examples/Lit.lock").is_file());
    let second = litigate_in(&dir, &["lock", "lit/examples"]);
    assert_eq!(second.json["result"]["changed"], false);
    let locked = litigate_in(&dir, &["lock", "--locked", "lit/examples"]);
    assert_eq!(locked.code, 0, "{}", locked.stdout);
    assert_eq!(located(&locked.json), []);
}

#[test]
fn locked_reports_a_package_lock_that_is_stale() {
    let dir = example("lock-package-stale", "", None);
    assert_eq!(litigate_in(&dir, &["lock", "lit/examples"]).code, 0);
    let module = dir.join("lit/examples/rule12.lit");
    let text = std::fs::read_to_string(&module).unwrap();
    std::fs::write(&module, format!("{text}\n// edited\n")).unwrap();
    let locked = litigate_in(&dir, &["lock", "--locked", "lit/examples"]);
    assert_eq!(locked.code, 2, "{}", locked.stdout);
    insta::assert_debug_snapshot!(located(&locked.json));
}

#[test]
fn a_directory_of_packages_locks_the_first() {
    let dir = example("lock-nested", "", None);
    let run = litigate_in(&dir, &["lock", "lit"]);
    assert_eq!(run.code, 0, "{}", run.stderr);
    assert_eq!(run.json["result"]["lockfile"], "lit/examples/Lit.lock");
}

#[test]
fn a_directory_with_nothing_to_lock_is_a_usage_error() {
    let dir = scratch("lock-empty");
    std::fs::create_dir_all(dir.join("empty")).unwrap();
    let run = litigate_in(&dir, &["lock", "empty"]);
    assert_eq!(run.code, 1);
    insta::assert_snapshot!(run.stderr, @"litigate lock: empty: no Lit.toml and no .case files");
}

#[test]
fn matter_locks_are_left_alone_when_current_and_reported_when_stale() {
    let dir = example("lock-matters", "", None);
    let again = litigate_in(&dir, &["lock", "cases/examples"]);
    assert_eq!(again.json["result"]["lockfiles"][0]["changed"], false);
    std::fs::write(dir.join("cases/examples/Lit.lock"), "version = 1\n").unwrap();
    let stale = litigate_in(&dir, &["lock", "--locked", "cases/examples"]);
    assert_eq!(stale.code, 2, "{}", stale.stdout);
    insta::assert_debug_snapshot!(located(&stale.json));
}

/// A matter reads only the `Lit.lock` in its own directory: neither a
/// parent directory's matter lock nor the package's own lock pins it, and
/// `lock` writes one per directory of matters (SPEC §5.6).
#[test]
fn a_matter_reads_only_the_lock_beside_it() {
    let dir = example("lock-beside", "", None);
    assert_eq!(litigate_in(&dir, &["lock", "lit/examples"]).code, 0);
    std::fs::create_dir_all(dir.join("cases/examples/nested")).unwrap();
    std::fs::copy(
        dir.join("cases/examples/acme.case"),
        dir.join("cases/examples/nested/acme.case"),
    )
    .unwrap();
    let nested = "cases/examples/nested/acme.case";
    let unpinned = litigate_in(&dir, &["check", nested]);
    assert_eq!(unpinned.code, 2, "{}", unpinned.stdout);
    let errors: Vec<(String, String, String, u64)> = located(&unpinned.json)
        .into_iter()
        .filter(|d| d.1 == "error")
        .collect();
    assert_eq!(
        errors,
        [("L1007".into(), "error".into(), nested.into(), 2)],
        "{}",
        unpinned.stdout
    );
    assert!(
        unpinned.stdout.contains("the matter has no Lit.lock"),
        "{}",
        unpinned.stdout
    );
    let lock = litigate_in(&dir, &["lock", "cases/examples"]);
    assert_eq!(lock.code, 0, "{}", lock.stderr);
    let files: Vec<(&str, bool)> = lock.json["result"]["lockfiles"]
        .as_array()
        .unwrap()
        .iter()
        .map(|f| (f["lockfile"].as_str().unwrap(), f["changed"] == true))
        .collect();
    assert_eq!(
        files,
        [
            ("cases/examples/Lit.lock", false),
            ("cases/examples/nested/Lit.lock", true),
        ]
    );
    let pinned = litigate_in(&dir, &["check", nested]);
    assert_eq!(pinned.code, 0, "{}", pinned.stdout);
}

#[cfg(unix)]
#[test]
fn unwritable_and_unreadable_lock_inputs_fail() {
    let dir = example("lock-unwritable", "", None);
    support::chmod(&dir.join("lit/examples"), 0o555);
    let package = litigate_in(&dir, &["lock", "lit/examples"]);
    support::chmod(&dir.join("lit/examples"), 0o755);
    assert_eq!(package.code, 1);
    insta::assert_snapshot!(package.stderr, @"litigate lock: lit/examples/Lit.litigate-tmp: Permission denied (os error 13)");
    std::fs::write(dir.join("cases/examples/Lit.lock"), "version = 1\n").unwrap();
    support::chmod(&dir.join("cases/examples"), 0o555);
    let matters = litigate_in(&dir, &["lock", "cases/examples"]);
    support::chmod(&dir.join("cases/examples"), 0o755);
    assert_eq!(matters.code, 1);
    insta::assert_snapshot!(matters.stderr, @"litigate lock: cases/examples/Lit.litigate-tmp: Permission denied (os error 13)");
    support::chmod(&dir.join("cases/examples/acme.case"), 0o000);
    let unreadable = litigate_in(&dir, &["lock", "cases/examples"]);
    support::chmod(&dir.join("cases/examples/acme.case"), 0o644);
    assert_eq!(unreadable.code, 1);
    insta::assert_snapshot!(unreadable.stderr, @"litigate lock: cases/examples/acme.case: Permission denied (os error 13)");
}

/// Runs `git ARGS` in `dir` with a throwaway identity, returning success.
fn git(dir: &std::path::Path, args: &[&str]) -> bool {
    std::process::Command::new("git")
        .current_dir(dir)
        .args([
            "-c",
            "user.name=t",
            "-c",
            "user.email=t@t",
            "-c",
            "commit.gpgsign=false",
        ])
        .args(args)
        .output()
        .unwrap()
        .status
        .success()
}

#[test]
fn the_merge_driver_lets_two_edits_to_one_package_merge_then_fail_closed() {
    let dir = example("lock-merge-git", "", None);
    assert_eq!(litigate_in(&dir, &["lock", "lit/examples"]).code, 0);
    std::fs::copy(
        support::repo().join(".gitattributes"),
        dir.join(".gitattributes"),
    )
    .unwrap();
    let driver = format!("{} lock --merge %O %A %B", env!("CARGO_BIN_EXE_litigate"));
    let module = dir.join("lit/examples/rule12.lit");
    let text = std::fs::read_to_string(&module).unwrap();
    let edit = |branch: &str, edited: String| {
        assert!(git(&dir, &["checkout", "-q", "-B", branch, "main"]));
        std::fs::write(&module, edited).unwrap();
        assert_eq!(litigate_in(&dir, &["lock", "lit/examples"]).code, 0);
        assert_eq!(litigate_in(&dir, &["lock", "cases/examples"]).code, 0);
        assert!(git(&dir, &["commit", "-q", "-am", branch]));
    };
    assert!(git(&dir, &["init", "-q", "-b", "main"]));
    assert!(git(&dir, &["config", "merge.litlock.driver", &driver]));
    assert!(git(&dir, &["add", "."]));
    assert!(git(&dir, &["commit", "-q", "-m", "base"]));
    edit("top", format!("// top\n{text}"));
    edit("bottom", format!("{text}// bottom\n"));
    assert!(git(&dir, &["merge", "-q", "--no-edit", "top"]));
    let merged = std::fs::read_to_string(dir.join("cases/examples/Lit.lock")).unwrap();
    assert!(
        merged.contains("digest = \"unmerged: run `litigate lock`\""),
        "{merged}"
    );
    for lockfile in ["lit/examples", "cases/examples"] {
        let stale = litigate_in(&dir, &["lock", "--locked", lockfile]);
        assert_eq!(stale.code, 2, "{lockfile}: {}", stale.stdout);
        assert_eq!(litigate_in(&dir, &["lock", lockfile]).code, 0);
        assert_eq!(litigate_in(&dir, &["lock", "--locked", lockfile]).code, 0);
    }
}

#[test]
fn merge_reports_what_it_wrote_and_fails_on_bad_inputs() {
    let dir = scratch("lock-merge");
    let lock = "version = 1\n";
    let other = "version = 1\n\n[[package]]\nname = \"a\"\nversion = \"1.0.0\"\ndigest = \"d\"\ndependencies = []\nlanguage = 1\n";
    for (name, text) in [("o", lock), ("a", lock), ("b", other), ("bad", "x")] {
        std::fs::write(dir.join(name), text).unwrap();
    }
    let run = litigate_in(&dir, &["lock", "--merge", "o", "a", "b"]);
    assert_eq!(run.code, 0, "{}", run.stderr);
    insta::assert_snapshot!(run.json["result"], @r#"{"lockfile":"a","changed":true,"unresolved":[]}"#);
    assert_eq!(std::fs::read_to_string(dir.join("a")).unwrap(), other);
    let again = litigate_in(&dir, &["lock", "--merge", "o", "a", "b"]);
    assert_eq!(again.json["result"]["changed"], false);
    let missing = litigate_in(&dir, &["lock", "--merge", "o", "a", "nope"]);
    assert_eq!(missing.code, 1);
    insta::assert_snapshot!(missing.stderr, @"litigate lock: nope: No such file or directory (os error 2)");
    let base = litigate_in(&dir, &["lock", "--merge", "bad", "a", "b"]);
    assert_eq!(
        (base.code, base.stderr.starts_with("litigate lock: bad: ")),
        (1, true)
    );
    let bad = litigate_in(&dir, &["lock", "--merge", "o", "bad", "b"]);
    assert_eq!(bad.code, 1);
    insta::assert_snapshot!(bad.stderr, @"litigate lock: bad: Lit.lock is invalid: key with no value, expected `=`");
}

#[cfg(unix)]
#[test]
fn merge_into_an_unwritable_directory_fails() {
    let dir = scratch("lock-merge-unwritable");
    for name in ["o", "a", "b"] {
        std::fs::write(dir.join(name), "version = 1\n").unwrap();
    }
    support::chmod(&dir, 0o555);
    let run = litigate_in(&dir, &["lock", "--merge", "o", "a", "b"]);
    support::chmod(&dir, 0o755);
    assert_eq!(run.code, 1);
    insta::assert_snapshot!(run.stderr, @"litigate lock: a.litigate-tmp: Permission denied (os error 13)");
}
