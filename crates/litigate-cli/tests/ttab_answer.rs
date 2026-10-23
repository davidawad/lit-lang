//! A matter on the real `lit/ttab` package takes the answer date from the
//! notice of institution (lg-flu.27): the notice designates it, no less than
//! thirty days from its mailing date (37 C.F.R. § 2.105(a)); a notice
//! designating less is rejected; a notice whose date the matter does not
//! have leaves the deadline unknown, never counted in its place; and a
//! Board order resetting it (§ 2.106(a)) is a `CaseOrder` layer that
//! replaces it.
// Test helpers below may unwrap: a panic is the failed assertion.
#![allow(clippy::unwrap_used)]
mod support;

use std::fmt::Write as _;
use support::{copy_dir, litigate_in, repo, scratch, Run};

const TTAB: &str = "ttab-trademark-trial-and-appeal-board";

/// `litigate check` of one TTAB matter with `events`, over copies of
/// `lit/core` and `lit/ttab` and, with `order`, the Board-order fixture.
fn check(name: &str, events: &[&str], order: bool) -> Run {
    let dir = scratch(name);
    for pkg in ["core", "ttab"] {
        copy_dir(&repo().join("lit").join(pkg), &dir.join("lit").join(pkg));
    }
    let mut uses = format!("uses {TTAB} ^0.1.0");
    if order {
        copy_dir(
            &repo().join("crates/litigate-cli/tests/fixtures/ttab-board-order"),
            &dir.join("lit/ttab-board-order"),
        );
        uses.push_str(", ttab-board-order ^0.1.0");
        let lock = litigate_in(&dir, &["lock", "lit/ttab-board-order"]);
        assert_eq!(lock.code, 0, "{}", lock.stdout);
    }
    std::fs::create_dir_all(dir.join("cases")).unwrap();
    let events = events.iter().fold(String::new(), |mut s, e| {
        let _ = writeln!(s, "  {e}");
        s
    });
    std::fs::write(
        dir.join("cases/answer.case"),
        format!(
            "case answer\n{uses}\nas_of 2026-08-20\nsummary \"s\"\n\n\
             parties {{\n  applicant: applicant Corporation\n  board: board Tribunal\n}}\n\n\
             events {{\n{events}}}\n\nstakes {{ {TTAB}::judgment-2135: 0.usd }}\n"
        ),
    )
    .unwrap();
    let lock = litigate_in(&dir, &["lock", "cases"]);
    assert_eq!(lock.code, 0, "{}", lock.stdout);
    litigate_in(&dir, &["check", "cases/answer.case"])
}

/// `(deadline, due, status, designated, unknown)` of every scheduled
/// obligation.
fn schedule(run: &Run) -> Vec<[String; 5]> {
    let field = |o: &serde_json::Value, k: &str| o[k].as_str().unwrap_or("-").to_owned();
    run.json["result"]["cases"][0]["deadlines"]
        .as_array()
        .unwrap()
        .iter()
        .map(|o| {
            [
                field(o, "deadline"),
                field(o, "due"),
                field(o, "status"),
                field(o, "designated"),
                field(o, "unknown"),
            ]
        })
        .collect()
}

/// Every error message.
fn errors(run: &Run) -> Vec<String> {
    run.json["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|d| d["severity"] == "error")
        .map(|d| d["message"].as_str().unwrap().to_owned())
        .collect()
}

fn row(deadline: &str, due: &str, status: &str, designated: &str, unknown: &str) -> [String; 5] {
    [deadline, due, status, designated, unknown].map(str::to_owned)
}

#[test]
fn the_notice_designates_the_answer_date() {
    // Mailed 2026-07-06, designating 30, 40 and 60 days.
    for (due, rolled) in [
        ("2026-08-05", "2026-08-05"),
        ("2026-08-15", "2026-08-17"),
        ("2026-09-04", "2026-09-04"),
    ] {
        let run = check(
            "ttab-designated",
            &[&format!(
                "2026-07-06 InstitutionNoticed({due}) on applicant"
            )],
            false,
        );
        assert_eq!(run.code, 0, "{}", run.stdout);
        assert_eq!(errors(&run), Vec::<String>::new());
        // No party bears the period (`bearer`), so past due it lapses.
        let status = if rolled < "2026-08-20" {
            "lapsed"
        } else {
            "open"
        };
        assert_eq!(
            schedule(&run),
            [row(
                &format!("{TTAB}::answer-period"),
                rolled,
                status,
                due,
                "-"
            )]
        );
    }
}

#[test]
fn a_notice_designating_less_than_thirty_days_is_rejected() {
    let run = check(
        "ttab-short",
        &["2026-07-06 InstitutionNoticed(2026-08-04) on applicant"],
        false,
    );
    assert_eq!(run.code, 2, "{}", run.stdout);
    assert_eq!(
        errors(&run),
        ["`InstitutionNoticed` on 2026-07-06 designates `answer-period` due 2026-08-04, earlier than the least time the rule allows, 30 calendar days from 2026-07-06 (2026-08-05) (37 CFR 2.105(a))"]
    );
}

#[test]
fn a_notice_without_its_due_date_leaves_the_answer_deadline_unknown() {
    let run = check(
        "ttab-unknown",
        &["2026-07-06 InstitutionNoticed(?) on applicant"],
        false,
    );
    assert_eq!(run.code, 0, "{}", run.stdout);
    assert_eq!(errors(&run), Vec::<String>::new());
    assert_eq!(
        schedule(&run),
        [row(
            &format!("{TTAB}::answer-period"),
            "-",
            "open",
            "-",
            "`InstitutionNoticed` designates no due date for `answer-period` (its `due` is unknown), so the due date is unknown: it is the notice's to set, never counted in its place"
        )]
    );
}

#[test]
fn a_board_order_resets_the_answer_date() {
    let run = check(
        "ttab-reset",
        &[
            "2026-07-06 InstitutionNoticed(2026-08-05) on applicant",
            "2026-08-03 ttab-board-order::AnswerTimeReset(2026-09-08) by board cite 37 CFR 2.106(a)",
        ],
        true,
    );
    assert_eq!(run.code, 0, "{}", run.stdout);
    assert_eq!(errors(&run), Vec::<String>::new());
    assert_eq!(
        schedule(&run),
        [
            row(
                &format!("{TTAB}::answer-period"),
                "2026-08-05",
                "superseded",
                "2026-08-05",
                "-"
            ),
            row(
                &format!("{TTAB}::answer-period"),
                "2026-09-08",
                "open",
                "2026-09-08",
                "-"
            ),
        ]
    );
}
