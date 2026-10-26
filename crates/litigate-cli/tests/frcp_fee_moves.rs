//! The FRCP 54(d)(2)(B) fee clock and its protective moves on the real
//! `lit/frcp` package (lg-i0eq): asking the opponent to stipulate, moving
//! to extend, and filing protectively, each judged by replay, and
//! `litigate lint-filing` listing the moves an open deadline has; and the
//! full motion supplementing a protective one, counted once (lg-4nma).
//! Dates are synthetic: judgment is entered 2025-06-02, so the fee motion
//! is due 2025-06-16.
// Test helpers below may unwrap: a panic is the failed assertion.
#![allow(clippy::unwrap_used)]
mod support;

use serde_json::Value;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use support::{copy_dir, litigate_in, repo, scratch, Run};

const FEE: &str = "frcp-civil-procedure::fee-motion-period";

/// A matter at `start` over copies of `lit/core` and `lit/frcp`, as of
/// `as_of`, with judgment entered 2025-06-02 and then `events`.
fn matter(name: &str, start: &str, as_of: &str, events: &[&str]) -> PathBuf {
    let dir = scratch(name);
    for pkg in ["core", "frcp"] {
        copy_dir(&repo().join("lit").join(pkg), &dir.join("lit").join(pkg));
    }
    // The copied locks pin the vendored L0 texts (SPEC §4.2).
    #[cfg(unix)]
    std::os::unix::fs::symlink(repo().join("sources"), dir.join("sources")).unwrap();
    #[cfg(not(unix))]
    copy_dir(&repo().join("sources"), &dir.join("sources"));
    std::fs::create_dir_all(dir.join("cases")).unwrap();
    let events = events
        .iter()
        .enumerate()
        .fold(String::new(), |mut s, (i, e)| {
            let _ = writeln!(s, "  e-{i}: {e}");
            s
        });
    std::fs::write(
        dir.join("cases/fee.case"),
        format!(
            "case fee\nuses frcp-civil-procedure ^2024.12.0\nas_of {as_of}\nsummary \"s\"\n\
             start frcp-civil-procedure::{start}\n\n\
             parties {{\n  movant: applicant Corporation\n  judge: court Tribunal\n}}\n\n\
             events {{\n  j-1: 2025-06-02 JudgmentEntered() by judge\n{events}}}\n\n\
             stakes {{ frcp-civil-procedure::appeal-window-blown: 0.usd }}\n"
        ),
    )
    .unwrap();
    let lock = litigate_in(&dir, &["lock", "cases"]);
    assert_eq!(lock.code, 0, "{}", lock.stdout);
    dir
}

/// Every error message of a run.
fn errors(run: &Run) -> Vec<String> {
    run.json["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|d| d["severity"] == "error")
        .map(|d| d["message"].as_str().unwrap().to_owned())
        .collect()
}

/// The fee instance of a `check` run's schedule.
fn fee(run: &Run) -> Value {
    run.json["result"]["cases"][0]["deadlines"]
        .as_array()
        .unwrap()
        .iter()
        .find(|o| o["deadline"] == FEE)
        .cloned()
        .unwrap()
}

/// `lint-filing` of `event` by the movant on `date`.
fn lint(dir: &Path, event: &str, date: &str) -> Run {
    litigate_in(
        dir,
        &[
            "lint-filing",
            "cases/fee.case",
            event,
            "--date",
            date,
            "--by",
            "movant",
        ],
    )
}

/// The moves `lint-filing` lists for the open fee instance.
fn moves(run: &Run) -> Value {
    run.json["result"]["open_before"]
        .as_array()
        .unwrap()
        .iter()
        .find(|o| o["deadline"] == FEE)
        .map(|o| o["moves"].clone())
        .unwrap()
}

#[test]
fn an_open_fee_clock_lists_its_protective_moves() {
    let dir = matter("fee-moves", "final-judgment-entered", "2025-06-10", &[]);
    let run = lint(&dir, "FeeMotionFiled()", "2025-06-12");
    assert_eq!(run.code, 0, "{}", run.stdout);
    assert_eq!(run.json["result"]["accepted"], true);
    let moves = moves(&run);
    assert_eq!(
        moves["file"],
        serde_json::json!([{
            "edge": "frcp-civil-procedure::edge-final-judgment-entered-fee-motion-filed",
            "event": "FeeMotionFiled()",
        }])
    );
    let requests: Vec<(&str, &str, &str)> = moves["extend"]
        .as_array()
        .unwrap()
        .iter()
        .map(|g| {
            (
                g["window"].as_str().unwrap(),
                g["standard"].as_str().unwrap(),
                g["request"].as_str().unwrap_or("-"),
            )
        })
        .collect();
    assert_eq!(
        requests,
        [
            (
                "before_expiry",
                "GoodCause",
                "TimeChangeStipulated(FeeMotionPeriod)"
            ),
            (
                "before_expiry",
                "GoodCause",
                "ExtensionMoved(FeeMotionPeriod)"
            ),
            ("before_expiry", "GoodCause", "-"),
            (
                "after_expiry",
                "ExcusableNeglect",
                "ExtensionMoved(FeeMotionPeriod)"
            ),
        ]
    );
    assert_eq!(
        moves["orders"],
        serde_json::json!(["FeeMotionPeriodExtended"])
    );
    // The protective filing discharges the clock and leaves the posture.
    assert_eq!(run.json["result"]["discharges"][0]["deadline"], FEE);
    assert_eq!(run.json["result"]["discharges"][0]["status"], "discharged");
    assert_eq!(
        run.json["result"]["replay"][0]["to"],
        "frcp-civil-procedure::final-judgment-entered"
    );
    // A Rule 6(b)(2) period is locked: no grant is a move, and the
    // notice of appeal's FRAP 4(a)(5) grant carries its bound.
    let open = run.json["result"]["open_before"].as_array().unwrap();
    let of = |d: &str| {
        open.iter()
            .find(|o| o["deadline"] == format!("frcp-civil-procedure::{d}"))
            .map(|o| o["moves"].clone())
            .unwrap()
    };
    assert_eq!(of("jmol-renewal-period")["extend"], serde_json::json!([]));
    let appeal = of("notice-of-appeal-period");
    assert_eq!(appeal["extend"][0]["window"], "any");
    assert!(appeal["extend"][0]["until"].is_object(), "{appeal}");
}

#[test]
fn a_fee_motion_after_the_fourteenth_day_is_late() {
    let dir = matter("fee-late", "final-judgment-entered", "2025-06-10", &[]);
    let run = lint(&dir, "FeeMotionFiled()", "2025-06-17");
    assert_eq!(run.json["result"]["accepted"], false, "{}", run.stdout);
    assert_eq!(
        errors(&run),
        ["`FeeMotionFiled()` on 2025-06-17 missed deadline `fee-motion-period`, due 2025-06-16"]
    );
}

#[test]
fn a_stipulation_re_anchors_the_fee_clock() {
    let dir = matter(
        "fee-stipulated",
        "final-judgment-entered",
        "2025-06-20",
        &[
            "2025-06-12 TimeChangeStipulated(FeeMotionPeriod) by movant",
            "2025-06-18 FeeMotionPeriodExtended(2025-07-14) by judge cite \"FRCP 6(b)(1)(A)\"",
        ],
    );
    let run = litigate_in(&dir, &["check", "cases/fee.case"]);
    assert_eq!(run.code, 0, "{}", run.stdout);
    let fee = fee(&run);
    assert_eq!(
        (
            &fee["extended_to"],
            &fee["extended_under"],
            &fee["request_made"]
        ),
        (
            &Value::from("2025-07-14"),
            &Value::from("GoodCause"),
            &Value::from("2025-06-12")
        )
    );
    // Past the original 2025-06-16, inside the re-anchored date.
    let filed = lint(&dir, "FeeMotionFiled()", "2025-07-01");
    assert_eq!(filed.json["result"]["accepted"], true, "{}", filed.stdout);
    assert_eq!(filed.json["result"]["discharges"][0]["due"], "2025-07-14");
}

#[test]
fn a_stipulation_after_the_due_date_licenses_no_order() {
    let dir = matter(
        "fee-stipulated-late",
        "final-judgment-entered",
        "2025-06-20",
        &[
            "2025-06-17 TimeChangeStipulated(FeeMotionPeriod) by movant",
            "2025-06-18 FeeMotionPeriodExtended(2025-07-14) by judge cite \"FRCP 6(b)(1)(A)\"",
        ],
    );
    let run = litigate_in(&dir, &["check", "cases/fee.case"]);
    assert_eq!(
        errors(&run),
        ["`FeeMotionPeriodExtended(2025-07-14)` requires a motion `ExtensionMoved(FeeMotionPeriod)` made after the period expired on 2025-06-16, and none precedes the order on 2025-06-18"]
    );
}

#[test]
fn a_motion_after_expiry_extends_for_excusable_neglect() {
    let dir = matter(
        "fee-moved-late",
        "final-judgment-entered",
        "2025-06-20",
        &[
            "2025-06-17 ExtensionMoved(FeeMotionPeriod) by movant",
            "2025-06-19 FeeMotionPeriodExtended(2025-06-30) by judge cite \"FRCP 6(b)(1)(B)\"",
        ],
    );
    let run = litigate_in(&dir, &["check", "cases/fee.case"]);
    assert_eq!(run.code, 0, "{}", run.stdout);
    let fee = fee(&run);
    assert_eq!(
        (&fee["extended_under"], &fee["request_made"]),
        (&Value::from("ExcusableNeglect"), &Value::from("2025-06-17"))
    );
}

#[test]
fn the_post_trial_window_offers_the_fee_filing_beside_the_other_motions() {
    let dir = matter(
        "fee-post-trial",
        "post-trial-motion-window",
        "2025-06-10",
        &[],
    );
    let run = lint(&dir, "FeeMotionFiled()", "2025-06-12");
    assert_eq!(run.json["result"]["accepted"], true, "{}", run.stdout);
    let file = &moves(&run)["file"];
    assert_eq!(
        file,
        &serde_json::json!([{
            "edge": "frcp-civil-procedure::edge-post-trial-motion-window-fee-motion-filed",
            "event": "FeeMotionFiled()",
        }])
    );
}

const SUPPLEMENT: &str = "frcp-civil-procedure::fee-supplement-period";

/// A matter whose protective fee motion is filed 2025-06-12 and whose
/// court sets 2025-07-21 for the full motion supplementing it.
fn protective(name: &str, as_of: &str) -> PathBuf {
    matter(
        name,
        "final-judgment-entered",
        as_of,
        &[
            "2025-06-12 FeeMotionFiled() by movant",
            "2025-06-20 FeeSupplementScheduled(2025-07-21) by judge",
        ],
    )
}

#[test]
fn the_full_motion_supplements_the_protective_one_and_discharges_its_deadline() {
    let dir = protective("fee-supplement", "2025-07-01");
    let run = litigate_in(&dir, &["check", "cases/fee.case"]);
    assert_eq!(run.code, 0, "{}", run.stdout);
    let deadlines = run.json["result"]["cases"][0]["deadlines"]
        .as_array()
        .unwrap();
    let of = |d: &str| {
        deadlines
            .iter()
            .find(|o| o["deadline"] == d)
            .cloned()
            .unwrap()
    };
    assert_eq!(of(FEE)["status"], "discharged");
    assert_eq!(
        (&of(SUPPLEMENT)["status"], &of(SUPPLEMENT)["due"]),
        (&Value::from("open"), &Value::from("2025-07-21"))
    );
    let filed = lint(&dir, "FeeMotionSupplemented()", "2025-07-18");
    assert_eq!(filed.json["result"]["accepted"], true, "{}", filed.stdout);
    let file = filed.json["result"]["open_before"]
        .as_array()
        .unwrap()
        .iter()
        .find(|o| o["deadline"] == SUPPLEMENT)
        .map(|o| o["moves"]["file"].clone())
        .unwrap();
    assert_eq!(
        file,
        serde_json::json!([{
            "edge": "frcp-civil-procedure::edge-final-judgment-entered-fee-motion-supplemented",
            "event": "FeeMotionSupplemented()",
        }])
    );
    let discharged = &filed.json["result"]["discharges"][0];
    assert_eq!(
        (&discharged["deadline"], &discharged["status"]),
        (&Value::from(SUPPLEMENT), &Value::from("discharged"))
    );
    assert_eq!(
        filed.json["result"]["replay"]
            .as_array()
            .unwrap()
            .last()
            .unwrap()["to"],
        "frcp-civil-procedure::final-judgment-entered"
    );
}

#[test]
fn a_supplement_after_the_date_the_court_set_is_late() {
    let dir = protective("fee-supplement-late", "2025-07-01");
    let run = lint(&dir, "FeeMotionSupplemented()", "2025-07-22");
    assert_eq!(run.json["result"]["accepted"], false, "{}", run.stdout);
    assert_eq!(
        errors(&run),
        ["`FeeMotionSupplemented()` on 2025-07-22 missed deadline `fee-supplement-period`, due 2025-07-21"]
    );
}

#[test]
fn a_later_scheduling_order_supersedes_the_supplement_date() {
    let dir = matter(
        "fee-supplement-reset",
        "final-judgment-entered",
        "2025-07-01",
        &[
            "2025-06-12 FeeMotionFiled() by movant",
            "2025-06-20 FeeSupplementScheduled(2025-07-21) by judge",
            "2025-06-27 FeeSupplementScheduled(2025-08-04) by judge",
        ],
    );
    let filed = lint(&dir, "FeeMotionSupplemented()", "2025-07-30");
    assert_eq!(filed.json["result"]["accepted"], true, "{}", filed.stdout);
    assert_eq!(filed.json["result"]["discharges"][0]["due"], "2025-08-04");
}

/// The engine's options at `node` of the matter's compiled graph, as
/// `(edge, target, hours)`.
fn options(dir: &Path, node: &str) -> Vec<(String, String, f64)> {
    let op = format!(r#"{{"op":"explain","node":"frcp-civil-procedure::{node}"}}"#);
    let run = litigate_in(dir, &["run", "cases/fee.case", "--op", &op]);
    assert_eq!(run.code, 0, "{}", run.stdout);
    run.json["result"]["result"]["options"]
        .as_array()
        .unwrap()
        .iter()
        .map(|o| {
            let id = o["id"].as_str().unwrap();
            (
                id.trim_start_matches("frcp-civil-procedure::").to_owned(),
                o["to"]
                    .as_str()
                    .unwrap()
                    .trim_start_matches("frcp-civil-procedure::")
                    .to_owned(),
                o["hours"].as_f64().unwrap(),
            )
        })
        .filter(|(id, _, _)| id.split('{').next().unwrap().contains("fee-motion"))
        .collect()
}

#[test]
fn a_protective_motion_and_its_supplement_count_the_hours_once() {
    let dir = matter(
        "fee-count-once",
        "final-judgment-entered",
        "2025-06-10",
        &[],
    );
    // Before any filing: the motion, with its hours; no supplement yet.
    assert_eq!(
        options(&dir, "final-judgment-entered"),
        [(
            "edge-final-judgment-entered-fee-motion-filed".to_owned(),
            "final-judgment-entered{fee-motion-filed}".to_owned(),
            15.0
        )]
    );
    // After it: no second motion, only the supplement, which adds none.
    for node in ["final-judgment-entered", "post-trial-motion-window"] {
        assert_eq!(
            options(&dir, &format!("{node}{{fee-motion-filed}}")),
            [(
                format!("edge-{node}-fee-motion-supplemented{{fee-motion-filed}}"),
                format!("{node}{{fee-motion-filed,fee-motion-supplemented}}"),
                0.0
            )]
        );
        // Once supplemented, neither is offered again.
        assert_eq!(
            options(
                &dir,
                &format!("{node}{{fee-motion-filed,fee-motion-supplemented}}")
            ),
            []
        );
    }
}
