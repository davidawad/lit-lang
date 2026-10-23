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
