//! `check`, `test`, and `describe` over packages and matters that fail in
//! each way the CLI reports itself: a package that does not lower, a lock
//! that does not parse, a matter that does not lower, a deadline the
//! engine cannot count, and a package that fails sema.
// Test helpers below may unwrap: a panic is the failed assertion.
#![allow(clippy::unwrap_used)]
mod support;

use support::{acme, example, litigate_in, located};

/// A counting rule whose cite binds no source URL: it checks, but its
/// package does not lower (L6001).
const UNLOWERABLE: &str = "counting nourl cite XYZ 6(a)
{
  exclude_trigger_day
  include_last_day
  holidays federal-holidays
}";

/// A node edge to a node that does not exist (L1002).
const BROKEN: &str = "state broken: \"Broken\" { _ \"Nowhere\" -> nowhere }";

#[test]
fn a_package_that_does_not_lower_fails_check() {
    let dir = example("check-unlowerable", UNLOWERABLE, None);
    let run = litigate_in(&dir, &["check", "lit/examples"]);
    assert_eq!(run.code, 2, "{}", run.stdout);
    insta::assert_debug_snapshot!(located(&run.json));
}

/// `check --known-at` selects versions among those recorded by then, as
/// `build --known-at` does: before the example's corpus date no version
/// of its start node is visible.
#[test]
fn check_selects_versions_at_its_known_at() {
    let dir = example("check-known-at", "", None);
    let latest = litigate_in(&dir, &["check", "lit/examples"]);
    assert_eq!(latest.code, 0, "{}", latest.stdout);
    let early = litigate_in(&dir, &["check", "--known-at", "2000-01-01", "lit/examples"]);
    assert_eq!(early.code, 2, "{}", early.stdout);
    let errors: Vec<_> = located(&early.json)
        .into_iter()
        .filter(|d| d.1 == "error")
        .collect();
    insta::assert_debug_snapshot!(errors);
}

#[test]
fn an_unparseable_matter_lock_fails_check() {
    let dir = example("check-lock", "", None);
    std::fs::write(dir.join("cases/examples/Lit.lock"), "version = 7\n").unwrap();
    let run = litigate_in(&dir, &["check", "cases/examples/acme.case"]);
    assert_eq!(run.code, 2, "{}", run.stdout);
    insta::assert_debug_snapshot!(located(&run.json));
}

#[test]
fn a_matter_that_does_not_lower_fails_check() {
    // `mixed` is the engine's to type: sema leaves it to lowering.
    let case = acme().replace(
        "analysis { params: { rate: 650, opp_rate: 550 } }",
        "analysis { mixed: \"sideways\" }",
    );
    let dir = example("check-analysis", "", Some(&case));
    let run = litigate_in(&dir, &["check", "cases/examples/acme.case"]);
    assert_eq!(run.code, 2, "{}", run.stdout);
    insta::assert_debug_snapshot!(located(&run.json));
}

#[test]
fn a_deadline_the_engine_cannot_count_is_left_off_the_schedule() {
    let dir = example(
        "check-business",
        "deadline in-business-days: 3.business_days from Served(complaint) counted_by frcp6",
        None,
    );
    let run = litigate_in(&dir, &["check", "cases/examples/acme.case"]);
    assert_eq!(run.code, 0, "{}", run.stdout);
    let deadlines: Vec<&str> = run.json["result"]["cases"][0]["deadlines"]
        .as_array()
        .unwrap()
        .iter()
        .map(|d| d["deadline"].as_str().unwrap())
        .collect();
    assert_eq!(deadlines, ["frcp-rule12-example::answer-period"]);
}

#[test]
fn test_reports_a_package_that_fails_sema() {
    let dir = example("test-broken", BROKEN, None);
    let run = litigate_in(&dir, &["test", "lit/examples"]);
    assert_eq!(run.code, 2, "{}", run.stdout);
    insta::assert_debug_snapshot!(located(&run.json));
}

#[test]
fn test_and_check_fail_on_missing_inputs() {
    let dir = example("test-missing", "", None);
    for command in ["test", "check", "build", "fmt", "lock", "fix"] {
        let run = litigate_in(&dir, &[command, "no/such/dir"]);
        assert_eq!(run.code, 1, "{command}");
        assert_eq!(
            run.stderr,
            format!("litigate {command}: no/such/dir: no such file or directory\n")
        );
    }
}

#[test]
fn describe_counts_the_errors_of_each_library_package() {
    let dir = example("describe-broken", BROKEN, None);
    let run = litigate_in(&dir, &["describe"]);
    assert_eq!(run.code, 0, "{}", run.stderr);
    let stdlib = &run.json["result"]["stdlib"];
    assert_eq!(stdlib[0]["name"], "frcp-rule12-example");
    assert_eq!(stdlib[0]["errors"], 1);
}

#[cfg(unix)]
#[test]
fn describe_without_a_readable_library_lists_none() {
    let dir = example("describe-unreadable", "", None);
    support::chmod(&dir.join("lit/examples"), 0o000);
    let run = litigate_in(&dir, &["describe"]);
    support::chmod(&dir.join("lit/examples"), 0o755);
    assert_eq!(run.code, 0, "{}", run.stderr);
    assert_eq!(run.json["result"]["stdlib"], serde_json::json!([]));
}
