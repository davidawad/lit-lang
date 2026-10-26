//! `litigate match-docket`: synthetic docket entries matched to the
//! coverage fixture's event forms. An exact match advances the matter, an
//! ambiguous or unmatched entry leaves it as it was, provenance (the
//! source entry and its docket number) is kept, and every failure.
// Test helpers below may unwrap: a panic is the failed assertion.
#![allow(clippy::unwrap_used)]
mod support_cmds;

use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use support_cmds::{copy_dir, fixture, litigate_in, repo, scratch};

/// Widget was served (docket 3).
const MATTER: &str = r#"case dk-matter
uses cov-example ^0.1.0
as_of 2025-03-10
summary "Docket fixture: Widget is served."

perspective widget

parties {
  acme: Plaintiff Corporation
  widget: Defendant Corporation
}

events {
  served-1: 2025-03-05 docket 3 Served(complaint, Personal) on widget
}

facts { time-barred: not-barred }
"#;

/// The fixture copied to a scratch directory with [`MATTER`] and its lock.
fn matter(name: &str, text: &str) -> PathBuf {
    let dir = scratch(name);
    copy_dir(&fixture(), &dir);
    std::fs::write(dir.join("cases/dk.case"), text).unwrap();
    assert_eq!(litigate_in(&dir, &["lock", "cases"]).code, 0);
    dir
}

/// `match-docket` over `docket` (written to `docket.json`).
fn run(dir: &Path, docket: &Value, extra: &[&str]) -> support_cmds::Run {
    std::fs::write(dir.join("docket.json"), docket.to_string()).unwrap();
    let mut args = vec!["match-docket", "cases/dk.case", "docket.json"];
    args.extend(extra);
    litigate_in(dir, &args)
}

fn entry(json: &Value, number: u64) -> &Value {
    json["result"]["entries"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["entry_number"] == number)
        .unwrap()
}

#[test]
fn an_exact_match_advances_and_nothing_else_does() {
    let dir = matter("docket-advance", MATTER);
    let docket = json!({"count": 6, "results": [
        {"entry_number": 3, "date_filed": "2025-03-05", "description": "SUMMONS Returned Executed on widget"},
        {"entry_number": 6, "date_filed": "2025-03-12", "description": "NOTICE of Appearance by Defendant"},
        {"entry_number": 5, "date_filed": "2025-03-07", "description": "ANSWER to Complaint by widget; hearing held"},
        {"entry_number": 4, "date_filed": "2025-03-07", "description": "ANSWER to Complaint by Defendant Widget Corp.",
         "recap_documents": [{"document_number": "4", "is_available": false}]},
        {"entry_number": 8, "date_filed": "2025-03-15", "description": "ANSWER to Complaint by widget"},
        {"entry_number": 9, "date_filed": "2025-03-16", "description": "Hearing held"},
    ]});
    let r = run(&dir, &docket, &["--out", "cases/advanced.case"]);
    assert_eq!(r.code, 0, "{}", r.stdout);
    let result = &r.json["result"];
    let statuses: Vec<(u64, &str)> = result["entries"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| {
            (
                e["entry_number"].as_u64().unwrap(),
                e["status"].as_str().unwrap(),
            )
        })
        .collect();
    // Docket order: by date, then entry number.
    assert_eq!(
        statuses,
        [
            (3, "recorded"),
            (4, "advanced"),
            (5, "ambiguous"),
            (6, "unmatched"),
            (8, "rejected"),
            (9, "incomplete"),
        ]
    );
    assert_eq!(result["advanced"], 1);
    assert_eq!(result["unresolved"], 4);
    // Provenance: the source entry verbatim, its confidence, and the
    // docket number on the event line.
    let four = entry(&r.json, 4);
    assert_eq!(four["source"], docket["results"][3]);
    assert_eq!(four["candidates"][0]["event"], "Answered");
    assert_eq!(four["candidates"][0]["confidence"], 1.0);
    let line = "answered-1: 2025-03-07 docket 4 Answered(complaint) by widget";
    assert_eq!(four["event"]["line"], line);
    assert_eq!(four["discharges"][0]["deadline"], "cov-pack::answer-period");
    // Ambiguous, unmatched and rejected entries do not change the matter.
    let five = entry(&r.json, 5);
    assert_eq!(
        five["reason"],
        "more than one event form fits: Answered, Hearing"
    );
    assert!(five.get("event").is_none());
    assert_eq!(entry(&r.json, 8)["diagnostics"][0]["code"], "L4001");
    let text = result["text"].as_str().unwrap();
    assert_eq!(
        text,
        MATTER.replace("on widget\n}", &format!("on widget\n  {line}\n}}"))
    );
    let written = std::fs::read_to_string(dir.join("cases/advanced.case")).unwrap();
    assert_eq!(written, text);
    assert_eq!(result["written"], "cases/advanced.case");
    // The advanced matter checks clean with the matters' lock.
    assert_eq!(litigate_in(&dir, &["lock", "cases"]).code, 0);
    let check = litigate_in(&dir, &["check", "cases/advanced.case"]);
    assert_eq!(check.code, 0, "{}", check.stdout);
}

#[test]
fn an_undated_or_misdated_supported_entry_is_incomplete() {
    let dir = matter("docket-undated", MATTER);
    let docket = json!([
        {"entry_number": 4, "description": "ANSWER to Complaint by widget"},
        {"entry_number": 5, "date_filed": "2025-02-30", "description": "ANSWER to Complaint by widget"},
    ]);
    let r = run(&dir, &docket, &[]);
    assert_eq!(r.code, 0, "{}", r.stdout);
    assert_eq!(entry(&r.json, 4)["reason"], "no `date_filed`");
    let five = entry(&r.json, 5)["reason"].as_str().unwrap();
    assert!(five.starts_with("`date_filed`: "), "{five}");
    assert_eq!(r.json["result"]["advanced"], 0);
    assert_eq!(r.json["result"]["text"], MATTER);
    assert!(r.json["result"]["written"].is_null());
}

#[test]
fn a_matter_with_errors_is_not_matched() {
    let dir = matter("docket-unclean", &MATTER.replace("on widget", "on nobody"));
    let r = run(&dir, &json!([]), &[]);
    assert_eq!(r.code, 2, "{}", r.stdout);
    assert!(r.json["result"].is_null());
}

#[test]
fn every_failure_is_a_usage_error() {
    let dir = matter("docket-failures", MATTER);
    let missing = litigate_in(&dir, &["match-docket", "cases/dk.case", "absent.json"]);
    assert_eq!(missing.code, 1);
    assert!(missing.stderr.contains("absent.json"), "{}", missing.stderr);
    std::fs::write(dir.join("bad.json"), "{").unwrap();
    let bad = litigate_in(&dir, &["match-docket", "cases/dk.case", "bad.json"]);
    assert_eq!(bad.code, 1);
    for (docket, message) in [
        (json!({"count": 0}), "a docket is an array of entries"),
        (json!([1]), "docket entry 0 is not an object"),
    ] {
        let r = run(&dir, &docket, &[]);
        assert_eq!(r.code, 1);
        assert!(r.stderr.contains(message), "{}", r.stderr);
    }
    std::fs::write(dir.join("docket.json"), "[]").unwrap();
    let not_case = litigate_in(&dir, &["match-docket", "lit/cov", "docket.json"]);
    assert_eq!(not_case.code, 1, "{}", not_case.stdout);
    assert!(
        not_case.stderr.contains("not a .case file"),
        "{}",
        not_case.stderr
    );
    let unwritable = run(&dir, &json!([]), &["--out", "no/such/dir/x.case"]);
    assert_eq!(unwritable.code, 1);
    assert!(
        unwritable.stderr.contains("no/such/dir/x.case"),
        "{}",
        unwritable.stderr
    );
}

#[test]
fn a_dependency_event_is_matched_in_the_matter_view() {
    // The PTAB matter uses `cafc` too: its events are written `cafc::Event`.
    let dir = scratch("docket-ptab");
    let docket = dir.join("docket.json");
    let entries = json!([
        {"entry_number": 13, "date_filed": "2026-08-21", "description": "Appeal docketed on owner"},
        {"entry_number": 14, "date_filed": "2026-08-22", "description": "Cert period extended by owner"},
    ]);
    std::fs::write(&docket, entries.to_string()).unwrap();
    let r = litigate_in(
        &repo(),
        &[
            "match-docket",
            "cases/ptab-ipr-defense.case",
            docket.to_str().unwrap(),
        ],
    );
    assert_eq!(r.code, 0, "{}", r.stdout);
    let thirteen = entry(&r.json, 13);
    assert_eq!(thirteen["status"], "advanced", "{thirteen}");
    assert_eq!(thirteen["event"]["call"], "cafc::Docketed()");
    assert_eq!(thirteen["event"]["mode"], "on");
    let fourteen = entry(&r.json, 14);
    assert_eq!(fourteen["status"], "incomplete");
    assert_eq!(
        fourteen["reason"],
        "argument `until`: a `Date` is not read from docket text"
    );
}
