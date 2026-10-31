//! A matter on the real `lit/ttab` package takes the answer date from the
//! notice of institution (lg-flu.27): the notice designates it, no less than
//! thirty days from its mailing date (37 C.F.R. § 2.105(a)); a notice
//! designating less is rejected; a notice whose date the matter does not
//! have leaves the deadline unknown, never counted in its place, and an
//! answer to it neither timely nor late (a warning, L4010); and a
//! Board order resetting it (§ 2.106(a)) is a `CaseOrder` layer that
//! replaces it. The counterclaim answer (§ 2.106(b)(3)(iii)) names no
//! start for its thirty days, so a designation short of thirty days from
//! mailing stands with a warning naming the interpretation (L4011), and
//! a Board order resetting it (§ 2.106(b)(3)(iv)) is the same layer.
//! The Board's extension of the answer time is Fed. R. Civ. P. 6(b)(1)(A)
//! reached through § 2.116(a), "wherever applicable and appropriate"
//! (lg-flu.45): an interpretation, so an extension under it stands with a
//! warning naming it (L4012), and the reset path is unchanged.
// Test helpers below may unwrap: a panic is the failed assertion.
#![allow(clippy::unwrap_used)]
mod support;

use std::fmt::Write as _;
use support::{copy_dir, litigate_in, repo, scratch, Run};

const TTAB: &str = "ttab-trademark-trial-and-appeal-board";

/// `litigate check` of one TTAB matter with `events`, over copies of
/// `lit/core` and `lit/ttab` (with the vendored L0 texts their locks pin)
/// and, with `order`, the Board-order fixture.
fn check(name: &str, events: &[&str], order: bool) -> Run {
    let dir = scratch(name);
    for pkg in ["core", "ttab"] {
        copy_dir(&repo().join("lit").join(pkg), &dir.join("lit").join(pkg));
    }
    copy_dir(&repo().join("sources"), &dir.join("sources"));
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
    rows(run, "answer-period")
}

/// The [`schedule`] rows of `TTAB`'s deadline `deadline` alone.
fn rows(run: &Run, deadline: &str) -> Vec<[String; 5]> {
    let id = format!("{TTAB}::{deadline}");
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
    // At as_of, past the least time (2026-08-05), the instance is unknown
    // with the reason: never open, never lapsed (lg-flu.46).
    assert_eq!(
        schedule(&run),
        [row(
            &format!("{TTAB}::answer-period"),
            "-",
            "unknown",
            "-",
            "notice due date not recorded: `InstitutionNoticed` on 2026-07-06 designates the due date of `answer-period` (its `due`), and the matter records it as `?`; it is the notice's to set, never counted in its place"
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
    // A reset is the Board's own order under 2.106(a), no interpretation.
    assert_eq!(replay_warnings(&run), Vec::<String>::new());
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
            "notice due date not recorded: `InstitutionNoticed` on 2026-07-06 designates the due date of `answer-period` (its `due`), and the matter records it as `?`; it is the notice's to set, never counted in its place"
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

#[test]
fn a_counterclaim_answer_date_short_of_thirty_days_from_mailing_stands_with_a_warning() {
    // Mailed 2026-07-06; thirty days from it is 2026-08-05.
    let run = check(
        "ttab-counterclaim-short",
        &["2026-07-06 CounterclaimAnswerTimeDesignated(2026-07-27) on applicant"],
        false,
    );
    assert_eq!(run.code, 0, "{}", run.stdout);
    assert_eq!(errors(&run), Vec::<String>::new());
    assert_eq!(
        replay_warnings(&run),
        ["`CounterclaimAnswerTimeDesignated` on 2026-07-06 designates `counterclaim-answer-period` due 2026-07-27, earlier than 30 calendar days from 2026-07-06 (2026-08-05) (37 CFR 2.106(b)(3)(iii)); the date stands: the rule names no start for its least time, and measuring it from 2026-07-06 is an interpretation: 37 CFR 2.106(b)(3)(iii) names no start for its thirty days; measured from the designation's mailing date, as 2.105(a) measures the answer's"]
    );
    let codes: Vec<&str> = run.json["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|d| d["code"].as_str().unwrap().starts_with("L4"))
        .map(|d| d["code"].as_str().unwrap())
        .collect();
    assert_eq!(codes, ["L4011"]);
    assert_eq!(
        rows(&run, "counterclaim-answer-period"),
        [row(
            &format!("{TTAB}::counterclaim-answer-period"),
            "2026-07-27",
            "lapsed",
            "2026-07-27",
            "-"
        )]
    );
    // Thirty days or more from mailing: no warning.
    let run = check(
        "ttab-counterclaim-thirty",
        &["2026-07-06 CounterclaimAnswerTimeDesignated(2026-08-05) on applicant"],
        false,
    );
    assert_eq!(run.code, 0, "{}", run.stdout);
    assert_eq!(replay_warnings(&run), Vec::<String>::new());
}

#[test]
fn a_cancellation_answer_date_short_of_thirty_days_from_mailing_is_rejected() {
    // 37 C.F.R. § 2.113(a) names the start: "from the mailing date of the
    // notice".
    let run = check(
        "ttab-cancellation-short",
        &["2026-07-06 CancellationInstitutionNoticed(2026-08-04) on applicant"],
        false,
    );
    assert_eq!(run.code, 2, "{}", run.stdout);
    assert_eq!(
        errors(&run),
        ["`CancellationInstitutionNoticed` on 2026-07-06 designates `cancellation-answer-period` due 2026-08-04, earlier than the least time the rule allows, 30 calendar days from 2026-07-06 (2026-08-05) (37 CFR 2.113(a))"]
    );
}

#[test]
fn a_board_order_resets_the_counterclaim_answer_date() {
    let run = check(
        "ttab-counterclaim-reset",
        &[
            "2026-07-06 CounterclaimAnswerTimeDesignated(2026-08-05) on applicant",
            "2026-08-03 ttab-board-order::CounterclaimAnswerTimeReset(2026-09-08) by board cite \"37 CFR 2.106(b)(3)(iv)\"",
        ],
        true,
    );
    assert_eq!(run.code, 0, "{}", run.stdout);
    assert_eq!(errors(&run), Vec::<String>::new());
    assert_eq!(replay_warnings(&run), Vec::<String>::new());
    assert_eq!(
        rows(&run, "counterclaim-answer-period"),
        [
            row(
                &format!("{TTAB}::counterclaim-answer-period"),
                "2026-08-05",
                "superseded",
                "2026-08-05",
                "-"
            ),
            row(
                &format!("{TTAB}::counterclaim-answer-period"),
                "2026-09-08",
                "open",
                "2026-09-08",
                "-"
            ),
        ]
    );
}

#[test]
fn the_generated_pack_carries_the_designated_answer_window() {
    // Pack v5 (lg-flu.33): each answer edge's window is the one the notice
    // designates, its 30 days the least time, never a counted `length`;
    // its extension is undeclared (absent), never `extendable: false`.
    let out = scratch("ttab-pack");
    let run = support::litigate(&[
        "build",
        "lit/core",
        "lit/ttab",
        "--out",
        out.to_str().unwrap(),
    ]);
    assert_eq!(run.code, 0, "{}", run.stdout);
    let pack: serde_json::Value =
        serde_json::from_slice(&std::fs::read(out.join(format!("packs/{TTAB}.json"))).unwrap())
            .unwrap();
    assert_eq!(pack["schemaVersion"], 5);
    let edges = pack["edges"].as_array().unwrap();
    let answers: Vec<usize> = (0..edges.len())
        .filter(|&i| edges[i]["from"] == "instituted" && edges[i]["to"] != "talks-instituted")
        .filter(|&i| edges[i]["to"] != "judgment-2135")
        .collect();
    assert_eq!(
        answers
            .iter()
            .map(|&i| edges[i]["to"].as_str().unwrap())
            .collect::<Vec<_>>(),
        ["answered", "answered", "notice-of-default"]
    );
    // The declaration's authority (FRCP 6(b)(1)(A) through 37 CFR
    // 2.116(a), an interpretation) and its two grants are carried with the
    // window, each grant naming the interpretation it rests on (lg-flu.45).
    let authority = "FRCP 6(b)(1)(A), applied through 37 CFR 2.116(a) (interpretation)";
    for &i in &answers {
        let d = &edges[i]["deadline"];
        assert_eq!(
            d["designated"],
            serde_json::json!({
                "minPeriod": 30, "sourceEvent": "InstitutionNoticed", "sourceEventArg": "due"
            })
        );
        assert_eq!(
            (&d["length"], &d["extendable"]),
            (&serde_json::Value::Null, &serde_json::Value::Null)
        );
        assert_eq!(d["extensionAuthority"], authority);
        let grants = d["extensions"].as_array().unwrap();
        assert_eq!(grants.len(), 2, "{d}");
        for g in grants {
            assert_eq!(
                (
                    &g["bound"],
                    &g["standard"],
                    &g["window"],
                    &g["interpretation"]
                ),
                (
                    &serde_json::json!("discretionary"),
                    &serde_json::json!("GoodCause"),
                    &serde_json::json!("beforeExpiry"),
                    &serde_json::json!(RULE_6B)
                ),
                "{g}"
            );
        }
        assert_eq!(grants[1]["motion"]["event"], "AnswerExtensionMoved()");
    }
    // Nothing under a deadline is a hole: the source states it all.
    let holes: Vec<&str> = pack["lowered"]["holes"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|h| h["pointer"].as_str())
        .filter(|p| p.contains("/deadline/"))
        .collect();
    assert_eq!(holes, Vec::<&str>::new());
    // The engine reads the pack's designated window; it has no counted
    // duration, so matters supply the notice's designated date.
    let loaded = litigate_engine::model::Pack::from_json(&pack.to_string()).unwrap();
    let answer_edge = &loaded.edges[answers[0]];
    assert!(answer_edge
        .deadline
        .as_ref()
        .is_some_and(|d| d.days().is_none()));
}

/// The interpretation the answer time's extension grants rest on.
const RULE_6B: &str = "37 CFR 2.116(a) applies the Federal Rules of Civil Procedure 'wherever applicable and appropriate'; that FRCP 6(b)(1)(A) reaches the time to answer is that judgment (TBMP § 509.01 so practises; guidance, not authority)";

/// The codes of every replay (L4xxx) diagnostic.
fn replay_codes(run: &Run) -> Vec<String> {
    run.json["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|d| d["code"].as_str())
        .filter(|c| c.starts_with("L4"))
        .map(str::to_owned)
        .collect()
}

/// `(due, extended_to, status, extended_under, request_made)` of each
/// answer-period row.
fn extended(run: &Run) -> Vec<[String; 5]> {
    let id = format!("{TTAB}::answer-period");
    let field = |o: &serde_json::Value, k: &str| o[k].as_str().unwrap_or("-").to_owned();
    run.json["result"]["cases"][0]["deadlines"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|o| o["deadline"] == id.as_str())
        .map(|o| {
            [
                field(o, "due"),
                field(o, "extended_to"),
                field(o, "status"),
                field(o, "extended_under"),
                field(o, "request_made"),
            ]
        })
        .collect()
}

#[test]
fn an_answer_extension_through_rule_6b_stands_with_the_interpretation_warning() {
    // Designated 2026-08-05; the applicant moves on 2026-08-03 and the
    // Board grants on 2026-08-07, after the date: the request was made
    // before expiry (FRCP 6(b)(1)(A)).
    let run = check(
        "ttab-extended-on-request",
        &[
            "2026-07-06 InstitutionNoticed(2026-08-05) on applicant",
            "2026-08-03 AnswerExtensionMoved() by applicant",
            "2026-08-07 AnswerTimeExtended(2026-09-04) by board cite \"FRCP 6(b)(1)(A); 37 CFR 2.116(a)\"",
        ],
        false,
    );
    assert_eq!(run.code, 0, "{}", run.stdout);
    assert_eq!(errors(&run), Vec::<String>::new());
    assert_eq!(
        replay_warnings(&run),
        [format!("`AnswerTimeExtended(2026-09-04)` on 2026-08-07 extends `{TTAB}::answer-period` to 2026-09-04 under a grant that rests on an interpretation: {RULE_6B}")]
    );
    assert_eq!(replay_codes(&run), ["L4012"]);
    assert_eq!(
        extended(&run),
        [[
            "2026-08-05",
            "2026-09-04",
            "open",
            "GoodCause",
            "2026-08-03"
        ]
        .map(str::to_owned)]
    );
    // The Board acting on its own before the date needs no motion; the
    // warning is the same.
    let run = check(
        "ttab-extended-by-board",
        &[
            "2026-07-06 InstitutionNoticed(2026-08-05) on applicant",
            "2026-08-04 AnswerTimeExtended(2026-09-04) by board cite \"FRCP 6(b)(1)(A); 37 CFR 2.116(a)\"",
        ],
        false,
    );
    assert_eq!(run.code, 0, "{}", run.stdout);
    assert_eq!(replay_codes(&run), ["L4012"]);
    assert_eq!(
        extended(&run),
        [["2026-08-05", "2026-09-04", "open", "GoodCause", "-"].map(str::to_owned)]
    );
    // An answer by the extended date is timely.
    let run = check(
        "ttab-extended-answered",
        &[
            "2026-06-02 Published() on applicant",
            "2026-06-30 OppositionFiled() by opposer",
            "2026-07-06 InstitutionNoticed(2026-08-05) on applicant",
            "2026-08-04 AnswerTimeExtended(2026-08-19) by board cite \"FRCP 6(b)(1)(A); 37 CFR 2.116(a)\"",
            "2026-08-19 AnswerFiled(Plain) by applicant",
        ],
        false,
    );
    assert_eq!(run.code, 0, "{}", run.stdout);
    assert_eq!(errors(&run), Vec::<String>::new());
    assert_eq!(replay_codes(&run), ["L4012"]);
    assert_eq!(
        extended(&run),
        [["2026-08-05", "2026-08-19", "discharged", "GoodCause", "-"].map(str::to_owned)]
    );
}

#[test]
fn an_answer_extension_after_the_date_without_a_prior_request_is_rejected() {
    // Rule 6(b)(1)(B)'s motion after expiry is not declared: an applicant
    // past its answer time is in default, and the Board's reset (2.106(a))
    // is its path.
    let run = check(
        "ttab-extended-late",
        &[
            "2026-07-06 InstitutionNoticed(2026-08-05) on applicant",
            "2026-08-10 AnswerTimeExtended(2026-09-04) by board cite \"FRCP 6(b)(1)(A); 37 CFR 2.116(a)\"",
        ],
        false,
    );
    assert_eq!(run.code, 2, "{}", run.stdout);
    assert_eq!(replay_codes(&run), ["L4001"]);
    assert_eq!(
        errors(&run),
        ["`AnswerTimeExtended(2026-09-04)` requires a motion `AnswerExtensionMoved()` made before the period expired on 2026-08-05, and none precedes the order on 2026-08-10"]
    );
    // Only the Board extends.
    let run = check(
        "ttab-extended-by-applicant",
        &[
            "2026-07-06 InstitutionNoticed(2026-08-05) on applicant",
            "2026-08-04 AnswerTimeExtended(2026-09-04) by applicant cite \"FRCP 6(b)(1)(A)\"",
        ],
        false,
    );
    assert_eq!(run.code, 2, "{}", run.stdout);
    assert_eq!(
        errors(&run),
        [format!(
            "only board may extend `{TTAB}::answer-period`, but `applicant` is applicant"
        )]
    );
}
