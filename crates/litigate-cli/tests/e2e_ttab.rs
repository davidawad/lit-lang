//! The end-to-end TTAB matters (lg-flu.46): one notice of institution,
//! mailed 2026-10-21, designating the answer date 2026-11-30 (open), a
//! date short of the thirty days 37 C.F.R. § 2.105(a) allows (L4001), or
//! `?`. A designated date the matter does not record leaves the answer
//! deadline with status `unknown` and the reason, naming the notice, in
//! the schedule, in `explain-value` and on the timeline: never `open`,
//! never `lapsed`.
// Test helpers below may unwrap: a panic is the failed assertion.
#![allow(clippy::unwrap_used)]
mod support;

use serde_json::{json, Value};
use support::litigate;

/// What `litigate check` and `explain-value` report for
/// `cases/e2e-ttab-{name}.case`: the exit code, the matter's own
/// diagnostics, its deadline schedule, the answer period's trace head,
/// and its timeline rows.
fn golden(name: &str) -> Value {
    let case = format!("cases/e2e-ttab-{name}.case");
    let run = litigate(&["check", &case]);
    let own: Vec<Value> = run.json["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|d| d["span"]["file"] == case.as_str())
        .map(|d| json!({"code": d["code"], "severity": d["severity"], "message": d["message"]}))
        .collect();
    let explained = litigate(&["explain-value", &case, "answer-period"]);
    let e = &explained.json["result"];
    let timeline = litigate(&["timeline", &case]);
    let answer: Vec<Value> = timeline.json["result"]["rows"]
        .as_array()
        .map(Vec::as_slice)
        .unwrap_or_default()
        .iter()
        .filter(|r| r["key"].as_str().unwrap().contains(".answer-period@"))
        .map(|r| json!({"key": r["key"], "due": r["due"], "status": r["status"]}))
        .collect();
    json!({
        "code": run.code,
        "diagnostics": own,
        "deadlines": run.json["result"]["cases"][0]["deadlines"],
        "explain_value": {"value": e["value"], "status": e["status"], "reason": e["reason"]},
        "timeline": {"code": timeline.code, "answer_period": answer},
    })
}

#[test]
fn a_notice_designating_thirty_days_or_more_is_open() {
    let g = golden("ok");
    assert_eq!(g["code"], 0);
    assert_eq!(g["deadlines"][0]["due"], "2026-11-30");
    assert_eq!(g["deadlines"][0]["status"], "open");
    insta::assert_json_snapshot!("e2e_ttab_ok", g);
}

#[test]
fn a_notice_designating_less_is_rejected() {
    let g = golden("short");
    assert_eq!(g["code"], 2);
    assert_eq!(g["diagnostics"][0]["code"], "L4001");
    insta::assert_json_snapshot!("e2e_ttab_short", g);
}

#[test]
fn a_notice_without_its_due_date_is_unknown_with_the_reason() {
    let g = golden("unknown");
    assert_eq!(g["code"], 0);
    let d = &g["deadlines"][0];
    assert_eq!(d["due"], Value::Null);
    assert_eq!(d["status"], "unknown");
    let reason = d["unknown"].as_str().unwrap();
    assert!(reason.starts_with("notice due date not recorded: `InstitutionNoticed` on 2026-10-21"));
    assert_eq!(g["explain_value"]["status"], "unknown");
    assert_eq!(g["explain_value"]["reason"], reason);
    assert_eq!(
        g["timeline"]["answer_period"][0]["status"],
        json!({"state": "unknown", "reason": "InstitutionNoticed due date not recorded"})
    );
    insta::assert_json_snapshot!("e2e_ttab_unknown", g);
}
