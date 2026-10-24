//! `run` refusals: an `--op` that is not JSON or not a litgraph op, a path
//! with no matter, a missing path, and a matter that fails its check.
// Test helpers below may unwrap: a panic is the failed assertion.
#![allow(clippy::unwrap_used)]
mod support;

use support::{acme, example, litigate, litigate_in, located};

#[test]
fn ops_that_are_not_json_or_not_litgraph_ops_are_usage_errors() {
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
            .starts_with("litigate run: invalid litgraph op: "),
        "{}",
        unknown.stderr
    );
}

#[test]
fn run_needs_an_existing_matter() {
    let none = litigate(&["run", "lit/examples", "--op", "chain"]);
    assert_eq!(none.code, 1);
    insta::assert_snapshot!(none.stderr, @"litigate run: lit/examples: not a .case file");
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
