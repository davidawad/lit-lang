//! `run` refusals: an `--op` that is not JSON or not an engine op, a bad
//! `--arg` or `--set`, a path with neither a matter nor a pack, a missing
//! path, and a matter that fails its check.
// Test helpers below may unwrap: a panic is the failed assertion.
#![allow(clippy::unwrap_used)]
mod support;

use support::{acme, example, litigate, litigate_in, located, scratch};

#[test]
fn ops_that_are_not_json_or_not_engine_ops_are_usage_errors() {
    let bad_json = litigate(&["run", "cases/examples/acme.case", "--op", "{bad"]);
    assert_eq!(bad_json.code, 1);
    assert!(
        bad_json
            .stderr
            .starts_with("litigate run: --op is not JSON: "),
        "{}",
        bad_json.stderr
    );
    let unknown = litigate(&[
        "run",
        "cases/examples/acme.case",
        "--op",
        "{\"op\": \"frobnicate\"}",
    ]);
    assert_eq!(unknown.code, 1);
    assert!(
        unknown
            .stderr
            .starts_with("litigate run: invalid engine op: "),
        "{}",
        unknown.stderr
    );
}

#[test]
fn bad_op_arguments_are_usage_errors() {
    let arg = litigate(&["solve", "cases/examples/acme.case", "--arg", "from"]);
    assert_eq!(arg.code, 1);
    insta::assert_snapshot!(arg.stderr, @"litigate solve: --arg expects K=V, got `from`");
    let set = litigate(&["solve", "cases/examples/acme.case", "--set", "rate=fast"]);
    assert_eq!(set.code, 1);
    insta::assert_snapshot!(set.stderr, @"litigate solve: --set rate: `fast` is not a number (params are numeric)");
}

#[test]
fn run_needs_a_matter_or_a_pack() {
    let none = litigate_in(&scratch("run-nothing"), &["run", ".", "--op", "chain"]);
    assert_eq!(none.code, 1);
    insta::assert_snapshot!(none.stderr, @"litigate run: .: neither a matter nor a package that lowers to a pack");
    let missing = litigate(&["run", "no/such.case", "--op", "chain"]);
    assert_eq!(missing.code, 1);
    insta::assert_snapshot!(missing.stderr, @"litigate run: no/such.case: no such file or directory");
}

#[test]
fn a_matter_that_fails_its_check_is_not_run() {
    let case = acme().replace(
        "analysis { params: { rate: 650.usd / 1.hours, opp_rate: 550.usd / 1.hours } }",
        "analysis { frobnicate: 1 }",
    );
    let dir = example("run-failing", "", Some(&case));
    let run = litigate_in(&dir, &["run", "cases/examples/acme.case", "--op", "chain"]);
    assert_eq!(run.code, 2, "{}", run.stdout);
    assert!(run.json.get("result").is_none());
    let errors: Vec<_> = located(&run.json)
        .into_iter()
        .filter(|d| d.1 == "error")
        .collect();
    assert_eq!(
        errors,
        [(
            "L4007".to_owned(),
            "error".to_owned(),
            "cases/examples/acme.case".to_owned(),
            27
        )]
    );
}
