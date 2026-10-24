//! A matter on the real `lit/ttab` package takes the answer date from the
//! notice of institution (lg-flu.27): the notice designates it, no less than
//! thirty days from its mailing date (37 C.F.R. § 2.105(a)); a notice
//! designating less is rejected; a notice whose date the matter does not
//! have leaves the deadline unknown, never counted in its place, and an
//! answer to it neither timely nor late (a warning, L4010); and a
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
             parties {{\n  applicant: applicant Corporation\n  opposer: opposer Corporation\n  board: board Tribunal\n}}\n\n\
             events {{\n{events}}}\n\nstakes {{ {TTAB}::judgment-2135: 0.usd }}\n"
        ),
    )
    .unwrap();
    let lock = litigate_in(&dir, &["lock", "cases"]);
    assert_eq!(lock.code, 0, "{}", lock.stdout);
    let explained = litigate_in(
        &dir,
        &["explain-value", "cases/answer.case", "answer-period"],
    );
    let mut run = litigate_in(&dir, &["check", "cases/answer.case"]);
    run.json["explained"] = explained.json["result"].clone();
    run
}

/// `explain-value`'s due date and the detail of each step that names the
/// designation or the unknown date.
fn explained(run: &Run) -> (String, Vec<String>) {
    let e = &run.json["explained"];
    let details = e["steps"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|s| s["detail"].as_str())
        .filter(|d| d.contains("designates") || d.contains("unknown"))
        .map(str::to_owned)
        .collect();
    (e["value"].as_str().unwrap_or("-").to_owned(), details)
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

/// The [`schedule`] rows of the answer period alone.
fn answer_rows(run: &Run) -> Vec<[String; 5]> {
    let id = format!("{TTAB}::answer-period");
    schedule(run).into_iter().filter(|r| r[0] == id).collect()
}

/// Every error message.
fn errors(run: &Run) -> Vec<String> {
    messages(run, |d| d["severity"] == "error")
}

/// Every replay (L4xxx) warning message; the package's own holes aside.
fn replay_warnings(run: &Run) -> Vec<String> {
    messages(run, |d| {
        d["severity"] == "warning" && d["code"].as_str().unwrap().starts_with("L4")
    })
}

/// The message of every diagnostic `keep` selects.
fn messages(run: &Run, keep: impl Fn(&serde_json::Value) -> bool) -> Vec<String> {
    run.json["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|d| keep(d))
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
        assert_eq!(
            explained(&run),
            (
                rolled.to_owned(),
                vec!["due on the date InstitutionNoticed() designates (its `due`), no earlier than 30 calendar day(s) after it, rolled by ttab-2-196; in force [2017-01-14, ..)".to_owned()]
            )
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
    let (value, details) = explained(&run);
    assert_eq!(value, "-");
    assert_eq!(details.len(), 2, "{details:#?}");
    assert!(
        details[1].starts_with(
            "triggered by InstitutionNoticed(?) on 2026-07-06; the due date is unknown: "
        ),
        "{details:#?}"
    );
}

#[test]
fn a_board_order_resets_the_answer_date() {
    let run = check(
        "ttab-reset",
        &[
            "2026-07-06 InstitutionNoticed(2026-08-05) on applicant",
            "2026-08-03 ttab-board-order::AnswerTimeReset(2026-09-08) by board cite \"37 CFR 2.106(a)\"",
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

#[test]
fn an_answer_to_a_notice_without_its_due_date_is_neither_timely_nor_late() {
    let run = check(
        "ttab-unknown-answered",
        &[
            "2026-06-02 Published() on applicant",
            "2026-06-30 OppositionFiled() by opposer",
            "2026-07-06 InstitutionNoticed(?) on applicant",
            "2026-08-10 AnswerFiled(Plain) by applicant",
        ],
        false,
    );
    assert_eq!(run.code, 0, "{}", run.stdout);
    assert_eq!(errors(&run), Vec::<String>::new());
    assert_eq!(
        replay_warnings(&run),
        ["`AnswerFiled(Plain)` on 2026-08-10 took deadline `answer-period`, but `InstitutionNoticed(?)` left its due date (`due`) unknown: whether it is timely or late is unknown"]
    );
    assert_eq!(
        answer_rows(&run),
        [row(
            &format!("{TTAB}::answer-period"),
            "-",
            "unjudged",
            "-",
            "`InstitutionNoticed` designates no due date for `answer-period` (its `due` is unknown), so the due date is unknown: it is the notice's to set, never counted in its place"
        )]
    );
}

#[test]
fn an_answer_by_the_designated_date_is_timely_and_after_it_late() {
    // Designated Saturday 2026-08-15, rolled to Monday 2026-08-17.
    let answered = |date: &str| {
        check(
            "ttab-answered",
            &[
                "2026-06-02 Published() on applicant",
                "2026-06-30 OppositionFiled() by opposer",
                "2026-07-06 InstitutionNoticed(2026-08-15) on applicant",
                &format!("{date} AnswerFiled(WithCounterclaim) by applicant"),
            ],
            false,
        )
    };
    let timely = answered("2026-08-17");
    assert_eq!(timely.code, 0, "{}", timely.stdout);
    assert_eq!(replay_warnings(&timely), Vec::<String>::new());
    assert_eq!(
        answer_rows(&timely),
        [row(
            &format!("{TTAB}::answer-period"),
            "2026-08-17",
            "discharged",
            "2026-08-15",
            "-"
        )]
    );
    let late = answered("2026-08-18");
    assert_eq!(late.code, 2, "{}", late.stdout);
    assert_eq!(
        errors(&late),
        ["`AnswerFiled(WithCounterclaim)` on 2026-08-18 missed deadline `answer-period`, due 2026-08-17"]
    );
}
