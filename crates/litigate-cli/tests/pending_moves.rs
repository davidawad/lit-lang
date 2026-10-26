//! A filed request that a deadline's grant reads (lg-9iww): a stipulation
//! or a motion to extend the FRCP 54(d)(2)(B) fee clock on the real
//! `lit/frcp` package shows as the deadline's pending move, filed and
//! awaiting order, in `check`, `timeline` and `lint-filing`, until the
//! order arrives. Dates are synthetic: judgment is entered 2025-06-02, so
//! the fee motion is due 2025-06-16.
// Test helpers below may unwrap: a panic is the failed assertion.
#![allow(clippy::unwrap_used)]
mod support;

use serde_json::{json, Value};
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use support::{copy_dir, litigate_in, repo, scratch};

const FEE: &str = "frcp-civil-procedure::fee-motion-period";
const STIPULATED: &str = "2025-06-12 TimeChangeStipulated(FeeMotionPeriod) by movant";
const ORDERED: &str =
    "2025-06-13 FeeMotionPeriodExtended(2025-07-14) by judge cite \"FRCP 6(b)(1)(A)\"";

/// A matter after final judgment over copies of `lit/core` and
/// `lit/frcp`, as of `as_of`, with judgment entered 2025-06-02 and then
/// `events`.
fn matter(name: &str, as_of: &str, events: &[&str]) -> PathBuf {
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
             start frcp-civil-procedure::final-judgment-entered\n\n\
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

/// The fee instance of the matter's `check` schedule.
fn fee(dir: &Path) -> Value {
    let run = litigate_in(dir, &["check", "cases/fee.case"]);
    assert_eq!(run.code, 0, "{}", run.stdout);
    run.json["result"]["cases"][0]["deadlines"]
        .as_array()
        .unwrap()
        .iter()
        .find(|o| o["deadline"] == FEE)
        .cloned()
        .unwrap()
}

/// `(label, status)` of the matter's timeline rows whose label starts with
/// one of `labels`.
fn rows(dir: &Path, labels: &[&str]) -> Vec<(String, Value)> {
    let run = litigate_in(dir, &["timeline", "cases/fee.case"]);
    assert_eq!(run.code, 0, "{}", run.stdout);
    run.json["result"]["rows"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|r| {
            let label = r["label"].as_str().unwrap();
            labels
                .iter()
                .any(|l| label.starts_with(l))
                .then(|| (label.to_owned(), r["status"].clone()))
        })
        .collect()
}

#[test]
fn a_stipulation_awaiting_its_order_is_the_fee_clocks_pending_move() {
    let dir = matter("pending-stipulation", "2025-06-12", &[STIPULATED]);
    let fee = fee(&dir);
    assert_eq!(fee["status"], "open");
    assert_eq!(
        fee["pending"],
        json!({"request": "TimeChangeStipulated(FeeMotionPeriod)", "date": "2025-06-12"})
    );
    let rows = rows(&dir, &["TimeChangeStipulated", "fee-motion-period"]);
    assert_eq!(
        rows.iter()
            .map(|(_, s)| (
                s["state"].as_str().unwrap(),
                s["reason"].as_str().unwrap_or("-")
            ))
            .collect::<Vec<_>>(),
        [
            (
                "none",
                "pending move of frcp-civil-procedure::fee-motion-period: filed, awaiting order"
            ),
            (
                "open",
                "filed, awaiting order: TimeChangeStipulated(FeeMotionPeriod) on 2025-06-12"
            ),
        ],
        "{rows:?}"
    );
}

#[test]
fn the_order_ends_the_pending_move() {
    let dir = matter("pending-ordered", "2025-06-14", &[STIPULATED, ORDERED]);
    let fee = fee(&dir);
    assert_eq!(fee["extended_to"], "2025-07-14");
    assert_eq!(fee.get("pending"), None, "{fee}");
    let rows = rows(&dir, &["TimeChangeStipulated"]);
    assert_eq!(
        rows[0].1,
        json!({"state": "none", "reason": null}),
        "{rows:?}"
    );
}

#[test]
fn a_motion_still_awaiting_its_order_past_the_due_date_reads_on_the_lapsed_row() {
    let dir = matter(
        "pending-lapsed",
        "2025-06-20",
        &["2025-06-12 ExtensionMoved(FeeMotionPeriod) by movant"],
    );
    let rows = rows(&dir, &["fee-motion-period"]);
    assert_eq!(
        rows[0].1,
        json!({
            "state": "lapsed",
            "reason": "past due at 2025-06-20 with no act; filed, awaiting order: ExtensionMoved(FeeMotionPeriod) on 2025-06-12",
        }),
        "{rows:?}"
    );
}

#[test]
fn lint_filing_a_stipulation_lists_the_move_it_makes_pending() {
    let dir = matter("pending-lint", "2025-06-10", &[]);
    let lint = |dir: &Path, event: &str, date: &str| {
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
    };
    let run = lint(&dir, "TimeChangeStipulated(FeeMotionPeriod)", "2025-06-12");
    assert_eq!(run.json["result"]["accepted"], true, "{}", run.stdout);
    let pending = run.json["result"]["pending"].as_array().unwrap();
    assert_eq!(
        pending
            .iter()
            .map(|p| (p["deadline"].as_str().unwrap(), p["pending"].clone()))
            .collect::<Vec<_>>(),
        [(
            FEE,
            json!({
                "request": "TimeChangeStipulated(FeeMotionPeriod)",
                "date": "2025-06-12",
                "state": "filed, awaiting order",
            })
        )]
    );
    // Once filed, the open fee clock carries it into the next filing's
    // report, and that filing makes nothing newly pending.
    let dir = matter("pending-lint-filed", "2025-06-12", &[STIPULATED]);
    let run = lint(&dir, "FeeMotionFiled()", "2025-06-13");
    assert_eq!(run.json["result"]["accepted"], true, "{}", run.stdout);
    assert_eq!(run.json["result"]["pending"], json!([]));
    let open = run.json["result"]["open_before"].as_array().unwrap();
    let fee = open.iter().find(|o| o["deadline"] == FEE).unwrap();
    assert_eq!(
        fee["pending"]["request"],
        "TimeChangeStipulated(FeeMotionPeriod)"
    );
    let other = open.iter().find(|o| o["deadline"] != FEE).unwrap();
    assert_eq!(other["pending"], Value::Null);
}
