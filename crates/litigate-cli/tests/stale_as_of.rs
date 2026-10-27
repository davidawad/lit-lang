//! A stale `as_of` cannot hide a passed deadline (lg-rpmf): `check` and
//! `timeline` judge deadlines at today (`--today`, else `LITIGATE_TODAY`,
//! else the system date), so one due before today with no act lapses,
//! with an L4016 warning; `--as-of` reproduces the matter as it stood.
#![allow(clippy::unwrap_used)]
mod support;

use serde_json::Value;
use std::path::{Path, PathBuf};
use support::{codes, litigate_in, Run};

/// The worked example served on 2026-10-05 with no answer or motion: its
/// answer period runs to 2026-10-26, and the matter's `as_of` is that day.
fn stale(name: &str) -> PathBuf {
    let case = support::acme()
        .replace("as_of 2025-06-01", "as_of 2026-10-26")
        .replace("served-1: 2025-03-05", "served-1: 2026-10-05")
        .replace(
            "  moved-to-dismiss-1: 2025-03-20 MovedToDismiss(complaint) by widget\n",
            "",
        );
    support::example(name, "", Some(&case))
}

/// `litigate --format json ARGS` in `dir` with `LITIGATE_TODAY` = `today`.
fn on(today: &str, dir: &Path, args: &[&str]) -> Run {
    let out = assert_cmd::Command::cargo_bin("litigate")
        .unwrap()
        .env("LITIGATE_TODAY", today)
        .current_dir(dir)
        .args(["--format", "json"])
        .args(args)
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
    Run {
        json: serde_json::from_str(&stdout).unwrap_or(Value::Null),
        stdout,
        stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
        code: out.status.code().unwrap_or(-1),
    }
}

/// The `(status, judged_at)` of the answer period `check` reports.
fn answer(run: &Run) -> (String, Option<String>) {
    let d = &run.json["result"]["cases"][0]["deadlines"][0];
    assert_eq!(
        d["deadline"], "frcp-rule12-example::answer-period",
        "{}",
        run.stdout
    );
    assert_eq!(d["due"], "2026-10-26");
    (
        d["status"].as_str().unwrap_or_default().to_owned(),
        d["judged_at"].as_str().map(str::to_owned),
    )
}

fn stale_warnings(run: &Run) -> usize {
    codes(&run.json)
        .iter()
        .filter(|(code, severity)| code == "L4016" && severity == "warning")
        .count()
}

#[test]
fn a_deadline_due_yesterday_with_as_of_yesterday_is_not_open() {
    let dir = stale("stale-as-of-check");
    let run = on("2026-10-27", &dir, &["check", "cases/examples/acme.case"]);
    assert_eq!(run.code, 0, "{}{}", run.stdout, run.stderr);
    assert_eq!(
        answer(&run),
        ("lapsed".to_owned(), Some("2026-10-27".to_owned()))
    );
    assert_eq!(stale_warnings(&run), 1, "{}", run.stdout);
    let warning = run.json["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .find(|d| d["code"] == "L4016")
        .unwrap();
    assert_eq!(
        warning["message"],
        "as_of 2026-10-26 is older than today 2026-10-27: deadlines are judged at today; 1 deadline is past due with no act"
    );
    assert_eq!(
        (&warning["span"]["line"], &warning["span"]["col"]),
        (&Value::from(3), &Value::from(7))
    );
    // `--today` says the same, whatever the environment.
    let flag = litigate_in(
        &dir,
        &["check", "--today", "2026-10-27", "cases/examples/acme.case"],
    );
    assert_eq!(answer(&flag), answer(&run));
    assert_eq!(stale_warnings(&flag), 1);
}

#[test]
fn as_of_today_or_an_explicit_date_judges_the_matter_as_it_stood() {
    let dir = stale("stale-as-of-history");
    let current = on("2026-10-26", &dir, &["check", "cases/examples/acme.case"]);
    assert_eq!(answer(&current), ("open".to_owned(), None));
    assert_eq!(stale_warnings(&current), 0);
    for args in [
        &["check", "--as-of", "2026-10-26", "cases/examples/acme.case"][..],
        &["check", "--today", "2026-10-26", "cases/examples/acme.case"][..],
    ] {
        let history = on("2026-10-27", &dir, args);
        assert_eq!(answer(&history), ("open".to_owned(), None), "{args:?}");
        assert_eq!(stale_warnings(&history), 0, "{args:?}");
    }
}

/// The answer period's row status in `timeline`'s document.
fn row_status(run: &Run) -> Value {
    run.json["result"]["rows"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["kind"] == "deadline")
        .map_or_else(
            || panic!("no deadline row: {}", run.stdout),
            |r| r["status"].clone(),
        )
}

#[test]
fn the_timeline_judges_deadlines_at_today() {
    let dir = stale("stale-as-of-timeline");
    let run = on(
        "2026-10-27",
        &dir,
        &["timeline", "cases/examples/acme.case"],
    );
    assert_eq!(run.code, 0, "{}{}", run.stdout, run.stderr);
    assert_eq!(
        row_status(&run),
        serde_json::json!({"state": "lapsed", "reason": "past due at 2026-10-27 with no act"})
    );
    let diagnostics = run.json["result"]["diagnostics"].as_array().unwrap();
    assert!(
        diagnostics
            .iter()
            .any(|d| d["code"] == "L4016" && d["severity"] == "warning"),
        "{diagnostics:?}"
    );
    let history = on(
        "2026-10-27",
        &dir,
        &[
            "timeline",
            "--as-of",
            "2026-10-26",
            "cases/examples/acme.case",
        ],
    );
    assert_eq!(row_status(&history)["state"], "open");
}

#[test]
fn today_must_be_a_date() {
    let dir = stale("stale-as-of-usage");
    for command in ["check", "timeline"] {
        let run = on("10/27/2026", &dir, &[command, "cases/examples/acme.case"]);
        assert_eq!(run.code, 1, "{command}: {}", run.stdout);
        assert!(
            run.stderr.contains("`10/27/2026` is not a YYYY-MM-DD date"),
            "{command}: {}",
            run.stderr
        );
    }
}
