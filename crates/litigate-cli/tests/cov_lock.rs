//! `lock` for a package and for directories of matters: writing, leaving
//! a current lock alone, `--locked` verification, and failures.
// Test helpers below may unwrap: a panic is the failed assertion.
#![allow(clippy::unwrap_used)]
mod support;

use support::{example, litigate_in, located, scratch};

#[test]
fn a_package_lock_is_written_once_then_left_alone() {
    let dir = example("lock-package", "", None);
    std::fs::remove_file(dir.join("lit/examples/Lit.lock")).unwrap();
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
    std::fs::remove_file(dir.join("lit/examples/Lit.lock")).unwrap();
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

#[test]
fn root_is_computed_from_a_verified_lock() {
    let dir = example("lock-root", "", None);
    assert_eq!(litigate_in(&dir, &["lock", "lit/examples"]).code, 0);
    let run = litigate_in(&dir, &["lock", "--root", "lit/examples"]);
    assert_eq!(run.code, 0, "{}", run.stdout);
    let root = run.json["result"]["root"].as_str().unwrap().to_owned();
    assert!(root.starts_with("sha256:") && root.len() == 71, "{root}");
    // Computed on read, never stored.
    let text = std::fs::read_to_string(dir.join("lit/examples/Lit.lock")).unwrap();
    assert!(!text.contains(&root[7..]), "{text}");
    let matters = litigate_in(&dir, &["lock", "--root", "cases/examples"]);
    assert_eq!(matters.code, 0, "{}", matters.stdout);
    assert!(matters.json["result"]["lockfiles"][0]["root"].is_string());
    // A stale lock has no root: L1007, and nothing is written.
    let module = dir.join("lit/examples/rule12.lit");
    let source = std::fs::read_to_string(&module).unwrap();
    std::fs::write(
        &module,
        format!(
            "{source}
terminal extra: \"Extra\"\n"
        ),
    )
    .unwrap();
    let stale = litigate_in(&dir, &["lock", "--root", "lit/examples"]);
    assert_eq!(stale.code, 2, "{}", stale.stdout);
    assert_eq!(stale.json["result"]["root"], serde_json::Value::Null);
    assert_eq!(
        std::fs::read_to_string(dir.join("lit/examples/Lit.lock")).unwrap(),
        text
    );
    std::fs::write(dir.join("cases/examples/Lit.lock"), "version = 1\n").unwrap();
    let matters = litigate_in(&dir, &["lock", "--root", "cases/examples"]);
    assert_eq!(matters.code, 2, "{}", matters.stdout);
    assert_eq!(
        matters.json["result"]["lockfiles"][0]["root"],
        serde_json::Value::Null
    );
}

#[test]
fn a_pin_naming_no_locked_row_is_stale() {
    let dir = example("lock-dangling", "", None);
    let path = dir.join("cases/examples/Lit.lock");
    let text = std::fs::read_to_string(&path).unwrap();
    let pinned = text.replacen(
        "dependencies = []",
        "dependencies = [\n    \"core 9.9.9\",\n]",
        1,
    );
    assert_ne!(pinned, text);
    std::fs::write(&path, pinned).unwrap();
    let stale = litigate_in(&dir, &["lock", "--locked", "cases/examples"]);
    assert_eq!(stale.code, 2, "{}", stale.stdout);
    insta::assert_snapshot!(
        stale.json["diagnostics"][0]["message"],
        @r#""Lit.lock: `frcp-rule12-example` depends on `core 9.9.9`, which names no locked package""#
    );
}

/// A `Lit.lock` in format 1 (lg-1od3): `check` reports L1007 naming
/// `litigate lock DIR` with a machine-applicable fix, and `fix` writes
/// what that command writes, after which `check` passes. For a package
/// lock and for a matter lock.
#[test]
fn a_format_1_lock_is_upgraded_by_fix() {
    let dir = example("lock-format-1", "", None);
    let v1 = std::fs::read_to_string(
        support::repo().join("crates/litigate-cli/tests/fixtures/lock-format-1/Lit.lock"),
    )
    .unwrap();
    for (target, lock) in [
        ("lit/examples", "lit/examples/Lit.lock"),
        ("cases/examples", "cases/examples/Lit.lock"),
    ] {
        let current = std::fs::read_to_string(dir.join(lock)).unwrap();
        std::fs::write(dir.join(lock), &v1).unwrap();
        let stale = litigate_in(&dir, &["check", target]);
        assert_eq!(stale.code, 2, "{}", stale.stdout);
        assert!(
            stale.stdout.contains(&format!(
                "{lock}:1:1: L1007 Lit.lock format 1 is not supported (expected 2): run `litigate lock {target}`"
            )),
            "{}",
            stale.stdout
        );
        let l1007: Vec<&serde_json::Value> = stale.json["diagnostics"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|d| d["code"] == "L1007" && d["span"]["file"] == lock)
            .collect();
        assert!(!l1007.is_empty(), "{}", stale.stdout);
        for d in &l1007 {
            assert_eq!(
                d["message"],
                format!(
                    "Lit.lock format 1 is not supported (expected 2): run `litigate lock {target}` to upgrade it"
                )
            );
            let fix = &d["fixes"][0];
            assert_eq!(fix["applicability"], "machine_applicable", "{d}");
            assert_eq!(fix["edits"][0]["replacement"], current.as_str(), "{d}");
        }
        let locked = litigate_in(&dir, &["lock", "--locked", target]);
        assert_eq!(locked.code, 2, "{}", locked.stdout);
        assert_eq!(
            locked.json["diagnostics"][0]["message"],
            l1007[0]["message"]
        );
        assert_eq!(locked.json["diagnostics"][0]["fixes"], l1007[0]["fixes"]);
        let fixed = litigate_in(&dir, &["fix", target]);
        assert_eq!(fixed.code, 0, "{}{}", fixed.stdout, fixed.stderr);
        assert_eq!(
            fixed.json["result"]["changed_files"],
            serde_json::json!([lock])
        );
        assert_eq!(std::fs::read_to_string(dir.join(lock)).unwrap(), current);
        let checked = litigate_in(&dir, &["check", target]);
        assert_eq!(checked.code, 0, "{}", checked.stdout);
        let locked = litigate_in(&dir, &["lock", "--locked", target]);
        assert_eq!(locked.code, 0, "{}", locked.stdout);
    }
}
