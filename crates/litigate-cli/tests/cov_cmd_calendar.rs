//! `litigate calendar`: a matter's computed deadlines as iCalendar, on a
//! real matter and on the coverage fixture (open and every deadline, an
//! uncountable deadline left unscheduled, `--out`, and every failure).
// Test helpers below may unwrap: a panic is the failed assertion.
#![allow(clippy::unwrap_used)]
mod support_cmds;

use serde_json::Value;
use std::path::PathBuf;
use support_cmds::{copy_dir, errors, fixture, in_fixture, litigate_in, repo, scratch};

/// A matter served with the complaint and the amended complaint, with a
/// hearing set; it uses the fixture package plus a deadline counted in
/// business days, which the engine cannot count.
const MATTER: &str = r#"case cal-matter
uses cov-example ^0.1.0
as_of 2025-03-10
summary "Calendar fixture: Widget is served; a hearing is set."

perspective widget

parties {
  acme: Plaintiff Corporation
  widget: Defendant Corporation
}

events {
  2025-03-05 Served(complaint, Personal) on widget
  2025-03-06 Hearing() on widget
  2025-03-07 Served(amended_complaint, Personal) on widget
}

facts { time-barred: not-barred }
"#;

/// The fixture copied to a scratch directory with [`MATTER`] and its lock.
fn matter(name: &str) -> PathBuf {
    let dir = scratch(name);
    copy_dir(&fixture(), &dir);
    let lit = dir.join("lit/cov/cov.lit");
    let text = std::fs::read_to_string(&lit).unwrap();
    std::fs::write(
        &lit,
        format!("{text}\ndeadline in-business-days: 3.business_days from Served(complaint) counted_by frcp6\n"),
    )
    .unwrap();
    std::fs::write(dir.join("cases/cal.case"), MATTER).unwrap();
    assert_eq!(litigate_in(&dir, &["lock", "cases"]).code, 0);
    dir
}

/// The result with its `ics` text taken out, and that text with the
/// compiler version masked.
fn split(result: &Value) -> (Value, String) {
    let mut result = result.clone();
    let ics = result["ics"].as_str().unwrap().to_owned();
    result["ics"] = Value::Null;
    (result, ics.replace(env!("CARGO_PKG_VERSION"), "<version>"))
}

#[test]
fn open_deadlines_are_events_and_an_uncountable_one_is_unscheduled() {
    let dir = matter("calendar-open");
    let run = litigate_in(&dir, &["calendar", "cases/cal.case", "--remind", "3"]);
    assert_eq!(run.code, 0, "{}", run.stdout);
    let (result, ics) = split(&run.json["result"]);
    insta::assert_json_snapshot!("calendar_open", result);
    insta::assert_snapshot!("calendar_open_ics", ics);
}

#[test]
fn all_includes_deadlines_that_are_no_longer_open() {
    let dir = matter("calendar-all");
    let run = litigate_in(&dir, &["calendar", "cases/cal.case", "--all"]);
    assert_eq!(run.code, 0, "{}", run.stdout);
    let statuses: Vec<(&str, &str)> = run.json["result"]["events"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| {
            (
                e["deadline"].as_str().unwrap(),
                e["status"].as_str().unwrap(),
            )
        })
        .collect();
    assert_eq!(
        statuses,
        [
            ("cov-pack::brief-period", "lapsed"),
            ("cov-pack::answer-period", "open"),
            ("cov-pack::amended-answer-period", "open"),
        ]
    );
    let brief = &run.json["result"]["events"][0];
    assert!(brief.get("authority").is_none(), "{brief}");
    assert!(run.json["result"]["ics"]
        .as_str()
        .unwrap()
        .contains("SUMMARY:[lapsed] brief-period due (cal-matter)"));
}

#[test]
fn out_writes_the_calendar_it_reports() {
    let dir = matter("calendar-out");
    let run = litigate_in(
        &dir,
        &["calendar", "cases/cal.case", "--out", "out/cal.ics"],
    );
    assert_eq!(run.code, 0, "{}", run.stdout);
    assert_eq!(run.json["result"]["written"], "out/cal.ics");
    let written = std::fs::read_to_string(dir.join("out/cal.ics")).unwrap();
    assert_eq!(written, run.json["result"]["ics"].as_str().unwrap());
    assert!(written.contains("\r\nEND:VCALENDAR\r\n"));
}

#[test]
fn an_unwritable_out_is_a_failure() {
    let dir = matter("calendar-unwritable");
    let run = litigate_in(
        &dir,
        &[
            "calendar",
            "cases/cal.case",
            "--out",
            "cases/cal.case/x.ics",
        ],
    );
    assert_eq!(run.code, 1, "{}", run.stdout);
    assert!(run.stderr.contains("cal.case"), "{}", run.stderr);
}

#[test]
fn a_missing_case_file_is_a_failure() {
    let run = in_fixture(&["calendar", "cases/none.case"]);
    assert_eq!(run.code, 1, "{}", run.stdout);
    assert!(run.stderr.contains("cases/none.case"), "{}", run.stderr);
}

#[test]
fn a_package_is_not_a_case() {
    let run = in_fixture(&["calendar", "lit/cov"]);
    assert_eq!(run.code, 1, "{}", run.stdout);
    assert!(
        run.stderr.contains("lit/cov: not a .case file"),
        "{}",
        run.stderr
    );
}

#[test]
fn a_case_with_errors_has_no_calendar() {
    let dir = scratch("calendar-broken-case");
    std::fs::write(
        dir.join("broken.case"),
        "case broken\nuses nowhere ^0.1.0\nas_of 2025-06-01\nsummary \"No such package.\"\n",
    )
    .unwrap();
    let run = litigate_in(&dir, &["calendar", "broken.case"]);
    assert_eq!(run.code, 2, "{}", run.stdout);
    assert!(run.json.get("result").is_none(), "{}", run.stdout);
    assert!(!errors(&run.json).is_empty());
}

#[test]
fn the_ptab_matter_calendars_its_open_preliminary_response_period() {
    let run = litigate_in(&repo(), &["calendar", "cases/ptab-ipr-defense.case"]);
    assert_eq!(run.code, 0, "{}", run.stdout);
    let events = run.json["result"]["events"].as_array().unwrap();
    let popr = events
        .iter()
        .find(|e| e["deadline"] == "ptab-patent-trial-appeal-board::popr-period")
        .unwrap();
    assert_eq!(popr["due"], "2026-09-18");
    assert!(popr["authority"]
        .as_str()
        .unwrap()
        .contains("37 C.F.R. § 42.107(b)"));
    assert!(events.iter().all(|e| e["status"] == "open"));
}
