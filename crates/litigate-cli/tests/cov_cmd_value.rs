//! `litigate explain-value`: backward, delegable, untriggered and
//! uncited deadlines, deadlines outside a rule, edges without a deadline,
//! pack prefixes by name or pack id, and every failure.
// Test helpers below may unwrap: a panic is the failed assertion.
#![allow(clippy::unwrap_used)]
mod support_cmds;

use serde_json::json;
use support_cmds::{errors, in_fixture, litigate_in, scratch, stable};

#[test]
fn a_case_without_a_value_is_a_failure() {
    let run = in_fixture(&["explain-value", "cases/cov.case"]);
    assert_eq!(run.code, 1, "{}", run.stdout);
    insta::assert_snapshot!("value_missing_id", run.stderr);
}

#[test]
fn a_missing_case_file_is_a_failure() {
    let run = in_fixture(&["explain-value", "cases/none.case", "answer-period"]);
    assert_eq!(run.code, 1, "{}", run.stdout);
    assert!(run.stderr.contains("cases/none.case"), "{}", run.stderr);
}

#[test]
fn a_package_is_not_a_case() {
    let run = in_fixture(&["explain-value", "lit/cov", "answer-period"]);
    assert_eq!(run.code, 1, "{}", run.stdout);
    insta::assert_snapshot!("value_not_a_case", run.stderr);
}

#[test]
fn a_case_with_errors_reports_them_without_a_value() {
    let dir = scratch("value-broken-case");
    std::fs::write(
        dir.join("broken.case"),
        "case broken\nuses nowhere ^0.1.0\nas_of 2025-06-01\nsummary \"No such package.\"\n",
    )
    .unwrap();
    let run = litigate_in(&dir, &["explain-value", "broken.case", "answer-period"]);
    assert_eq!(run.code, 2, "{}", run.stdout);
    assert!(run.json.get("result").is_none(), "{}", run.stdout);
    insta::assert_json_snapshot!("value_case_with_errors", errors(&run.json));
}

#[test]
fn an_untriggered_backward_delegable_deadline_has_no_due_date() {
    let run = in_fixture(&["explain-value", "cases/cov.case", "brief-period"]);
    assert_eq!(run.code, 0, "{}", run.stdout);
    let result = &run.json["result"];
    assert!(result["value"].is_null());
    assert_eq!(result["type"], "Date");
    insta::assert_json_snapshot!("value_untriggered_backward_deadline", result);
}

#[test]
fn a_deadline_outside_any_rule_traces_its_own_cite() {
    let run = in_fixture(&["explain-value", "cases/cov.case", "reply-period"]);
    assert_eq!(run.code, 0, "{}", run.stdout);
    let result = &run.json["result"];
    assert_eq!(result["value"], "2025-03-19");
    insta::assert_json_snapshot!("value_deadline_outside_a_rule", result["steps"]);
}

#[test]
fn a_deadline_names_the_rule_declaring_it_among_others() {
    let run = in_fixture(&["explain-value", "cases/cov.case", "cov-pack::answer-period"]);
    assert_eq!(run.code, 0, "{}", run.stdout);
    let result = &run.json["result"];
    // `reply-period notwithstanding answer-period` governs it (SPEC §5.5):
    // 14 days, not answer-period's own 21, and no version of its own is
    // the selected one.
    assert_eq!(result["value"], "2025-03-19");
    assert_eq!(result["steps"][0]["rule"], "reply-period@2009-12-01");
    assert!(result["versions"]
        .as_array()
        .unwrap()
        .iter()
        .all(|v| v["selected"] == false));
    assert_eq!(result["steps"][1]["rule"], "answer-duty");
    assert_eq!(result["overrides"][0]["id"], "reply-period");
}

#[test]
fn an_edge_without_a_deadline_by_package_name() {
    let run = in_fixture(&["explain-value", "cases/cov.case", "cov-example::default"]);
    assert_eq!(run.code, 0, "{}", run.stdout);
    let result = &run.json["result"];
    assert_eq!(result["type"], "Edge");
    assert_eq!(result["id"], "cov-pack::default");
    assert!(result["value"]["due"].is_null());
    assert_eq!(result["steps"].as_array().unwrap().len(), 1);
    insta::assert_json_snapshot!("value_edge_without_deadline", stable(&run.json));
}

#[test]
fn an_edge_with_a_deadline_by_pack_id() {
    let run = in_fixture(&["explain-value", "cases/cov.case", "cov-pack::answer"]);
    assert_eq!(run.code, 0, "{}", run.stdout);
    // `within answer-period` runs under the claimant that displaces it.
    assert_eq!(run.json["result"]["value"]["due"], json!("2025-03-19"));
}

#[test]
fn an_id_that_is_neither_deadline_nor_edge_is_a_failure() {
    let run = in_fixture(&["explain-value", "cases/cov.case", "time-barred"]);
    assert_eq!(run.code, 1, "{}", run.stdout);
    insta::assert_snapshot!("value_neither", run.stderr);
    let other = in_fixture(&["explain-value", "cases/cov.case", "other-pack::answer"]);
    assert_eq!(other.code, 1, "{}", other.stdout);
}
