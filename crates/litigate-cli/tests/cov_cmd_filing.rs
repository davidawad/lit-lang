//! `litigate lint-filing`: a proposed filing replayed into a matter, on a
//! real matter and on the coverage fixture (timely, late, a service event
//! that triggers a deadline, an undeclared party, a breach the filing
//! does not cause, and every failure).
// Test helpers below may unwrap: a panic is the failed assertion.
#![allow(clippy::unwrap_used)]
mod support_cmds;

use serde_json::Value;
use std::path::{Path, PathBuf};
use support_cmds::{copy_dir, errors, fixture, in_fixture, litigate_in, repo, scratch, stable};

/// Widget has been served; its answer is due 2025-03-19, and a duty it
/// bears falls due on 2025-03-10.
const MATTER: &str = r#"case lf-matter
uses cov-example ^0.1.0
as_of 2025-03-10
summary "Filing fixture: Widget is served."

perspective widget

parties {
  acme: Plaintiff Corporation
  widget: Defendant Corporation
}

events {
  2025-03-05 Served(complaint, Personal) on widget
}

facts { time-barred: not-barred }
"#;

/// The fixture copied to a scratch directory with [`MATTER`], a duty
/// Widget bears, and the lock.
fn matter(name: &str) -> PathBuf {
    let dir = scratch(name);
    copy_dir(&fixture(), &dir);
    let lit = dir.join("lit/cov/cov.lit");
    let text = std::fs::read_to_string(&lit).unwrap();
    std::fs::write(
        &lit,
        format!(
            "{text}\nrule duty borne-duty\n{{\n  deadline borne-period: 3.days from Served(complaint) counted_by frcp6 {{ bearer Defendant }}\n}}\n"
        ),
    )
    .unwrap();
    std::fs::write(dir.join("cases/lf.case"), MATTER).unwrap();
    assert_eq!(litigate_in(&dir, &["lock", "cases"]).code, 0);
    dir
}

fn lint(dir: &Path, args: &[&str]) -> support_cmds::Run {
    let mut all = vec!["lint-filing", "cases/lf.case"];
    all.extend(args);
    litigate_in(dir, &all)
}

fn codes(json: &Value) -> Vec<String> {
    json["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .map(|d| d["code"].as_str().unwrap().to_owned())
        .collect()
}

#[test]
fn a_timely_answer_discharges_the_answer_period() {
    let dir = matter("filing-timely");
    let run = lint(
        &dir,
        &[
            "Answered(complaint)",
            "--date",
            "2025-03-07",
            "--by",
            "widget",
        ],
    );
    assert_eq!(run.code, 0, "{}", run.stdout);
    insta::assert_json_snapshot!("filing_timely", stable(&run.json));
}

#[test]
fn a_late_answer_is_l4002_and_a_breach_it_does_not_cause_is_not_reported() {
    let dir = matter("filing-late");
    let run = lint(
        &dir,
        &[
            "Answered(complaint)",
            "--date",
            "2025-03-25",
            "--by",
            "widget",
        ],
    );
    assert_eq!(run.code, 2, "{}", run.stdout);
    assert_eq!(codes(&run.json), ["L4002"]);
    let result = &run.json["result"];
    assert_eq!(result["accepted"], false);
    assert_eq!(result["as_of"], "2025-03-25");
    assert_eq!(result["triggers"], Value::Array(Vec::new()));
}

#[test]
fn a_service_event_triggers_a_deadline_without_moving_as_of() {
    let dir = matter("filing-service");
    let run = lint(
        &dir,
        &[
            "Served(amended_complaint, Personal)",
            "--date",
            "2025-03-08",
            "--on",
            "widget",
        ],
    );
    assert_eq!(run.code, 0, "{}", run.stdout);
    let result = &run.json["result"];
    assert_eq!(result["as_of"], "2025-03-10");
    assert_eq!(
        result["filing"]["line"],
        "2025-03-08 Served(amended_complaint, Personal) on widget"
    );
    let triggered: Vec<(&str, &str)> = result["triggers"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| (t["deadline"].as_str().unwrap(), t["due"].as_str().unwrap()))
        .collect();
    assert_eq!(
        triggered,
        [("cov-pack::amended-answer-period", "2025-03-24")]
    );
}

#[test]
fn an_undeclared_party_is_an_error_at_the_filing() {
    let dir = matter("filing-party");
    let run = lint(
        &dir,
        &[
            "Answered(complaint)",
            "--date",
            "2025-03-07",
            "--by",
            "nobody",
        ],
    );
    assert_eq!(run.code, 2, "{}", run.stdout);
    let diagnostics = run.json["diagnostics"].as_array().unwrap();
    assert_eq!(diagnostics.len(), 1, "{}", run.stdout);
    assert_eq!(diagnostics[0]["code"], "L4005");
    assert_eq!(diagnostics[0]["span"]["line"], 15);
}

#[test]
fn text_that_is_not_one_event_is_a_failure() {
    let dir = matter("filing-two-events");
    let run = lint(
        &dir,
        &[
            "Answered(complaint) by widget\n  2025-03-08 Hearing()",
            "--date",
            "2025-03-07",
            "--by",
            "widget",
        ],
    );
    assert_eq!(run.code, 1, "{}", run.stdout);
    assert!(run.stderr.contains("is not one event"), "{}", run.stderr);
}

#[test]
fn a_bad_date_is_a_failure() {
    let dir = matter("filing-bad-date");
    let run = lint(
        &dir,
        &[
            "Answered(complaint)",
            "--date",
            "2025-02-30",
            "--by",
            "widget",
        ],
    );
    assert_eq!(run.code, 1, "{}", run.stdout);
    assert!(run.stderr.contains("--date 2025-02-30"), "{}", run.stderr);
}

#[test]
fn a_party_is_required() {
    let dir = matter("filing-no-party");
    let run = lint(&dir, &["Answered(complaint)", "--date", "2025-03-07"]);
    assert_eq!(run.code, 1, "{}", run.stdout);
    assert!(run.stderr.contains("--by"), "{}", run.stderr);
}

#[test]
fn a_missing_case_file_is_a_failure() {
    let run = in_fixture(&[
        "lint-filing",
        "cases/none.case",
        "Answered(complaint)",
        "--date",
        "2025-03-07",
        "--by",
        "widget",
    ]);
    assert_eq!(run.code, 1, "{}", run.stdout);
    assert!(run.stderr.contains("cases/none.case"), "{}", run.stderr);
}

#[test]
fn a_package_is_not_a_case() {
    let run = in_fixture(&[
        "lint-filing",
        "lit/cov",
        "Answered(complaint)",
        "--date",
        "2025-03-07",
        "--by",
        "widget",
    ]);
    assert_eq!(run.code, 1, "{}", run.stdout);
    assert!(
        run.stderr.contains("lit/cov: not a .case file"),
        "{}",
        run.stderr
    );
}

#[test]
fn a_case_with_errors_is_not_linted() {
    let dir = scratch("filing-broken-case");
    std::fs::write(
        dir.join("broken.case"),
        "case broken\nuses nowhere ^0.1.0\nas_of 2025-06-01\nsummary \"No such package.\"\n",
    )
    .unwrap();
    let run = litigate_in(
        &dir,
        &[
            "lint-filing",
            "broken.case",
            "Answered(complaint)",
            "--date",
            "2025-06-02",
            "--by",
            "widget",
        ],
    );
    assert_eq!(run.code, 2, "{}", run.stdout);
    assert!(run.json.get("result").is_none(), "{}", run.stdout);
    assert!(!errors(&run.json).is_empty());
}

#[test]
fn a_timely_preliminary_response_triggers_the_institution_period() {
    let run = litigate_in(
        &repo(),
        &[
            "lint-filing",
            "cases/ptab-ipr-defense.case",
            "PreliminaryResponseFiled()",
            "--date",
            "2026-09-10",
            "--by",
            "owner",
        ],
    );
    assert_eq!(run.code, 0, "{}", run.stdout);
    let result = &run.json["result"];
    let deadline = |list: &str, id: &str| -> Value {
        result[list]
            .as_array()
            .unwrap()
            .iter()
            .find(|e| e["deadline"] == format!("ptab-patent-trial-appeal-board::{id}"))
            .cloned()
            .unwrap_or_default()
    };
    assert_eq!(
        deadline("discharges", "popr-period")["status"],
        "discharged"
    );
    assert_eq!(
        deadline("triggers", "institution-period")["due"],
        "2026-12-10"
    );
}
