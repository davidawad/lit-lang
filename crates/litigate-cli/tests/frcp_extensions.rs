//! A matter on the real `lit/frcp` package records court-ordered
//! extensions (lg-flu.21, lg-flu.23): FRCP 6(b)(1) grants with no numeric
//! ceiling, before expiry and after it on a motion made after expiry, and
//! FRAP 4(a)(5)'s extension of the notice-of-appeal and cross-appeal
//! periods, on a motion no later than 30 days after the prescribed time,
//! to the later of 30 days after it and 14 days after the order; a
//! Justice's extension of the certiorari period by up to 60 days (28
//! U.S.C. § 2101(c)); and the Director's of the PTAB and TTAB appeal
//! periods (37 C.F.R. §§ 90.3(c), 2.145(e)).
// Test helpers below may unwrap: a panic is the failed assertion.
#![allow(clippy::unwrap_used)]
mod support;

use std::fmt::Write as _;
use support::{copy_dir, litigate_in, repo, scratch, Run};

/// `litigate check` of one matter over copies of `lit/core` and `lit/frcp`.
fn check(name: &str, events: &[&str]) -> Run {
    matter(
        name,
        &["frcp"],
        "frcp-civil-procedure ^2024.12.0",
        "  plaintiff: applicant Corporation\n  judge: court Tribunal\n",
        events,
        "frcp-civil-procedure::appeal-window-blown",
    )
}

/// `litigate check` of one matter over copies of `lit/core` and `pkgs`,
/// using `uses`, with `parties` and `events`, staking `stake`.
fn matter(
    name: &str,
    pkgs: &[&str],
    uses: &str,
    parties: &str,
    events: &[&str],
    stake: &str,
) -> Run {
    let dir = scratch(name);
    for pkg in std::iter::once(&"core").chain(pkgs) {
        copy_dir(&repo().join("lit").join(pkg), &dir.join("lit").join(pkg));
    }
    std::fs::create_dir_all(dir.join("cases")).unwrap();
    let events = events.iter().fold(String::new(), |mut s, e| {
        let _ = writeln!(s, "  {e}");
        s
    });
    std::fs::write(
        dir.join("cases/ext.case"),
        format!(
            "case ext\nuses {uses}\nas_of 2026-06-01\nsummary \"s\"\n\n\
             parties {{\n{parties}}}\n\n\
             events {{\n{events}}}\n\nstakes {{ {stake}: 0.usd }}\n"
        ),
    )
    .unwrap();
    let lock = litigate_in(&dir, &["lock", "cases"]);
    assert_eq!(lock.code, 0, "{}", lock.stdout);
    litigate_in(&dir, &["check", "cases/ext.case"])
}

/// The replay effect of each event (none when the check fails), and every
/// error message.
fn effects(run: &Run) -> (Vec<String>, Vec<String>) {
    let replay = run.json["result"]["cases"][0]["replay"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|r| format!("{} {}", r["effect"].as_str().unwrap(), r["event"]))
        .collect();
    let errors = run.json["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|d| d["severity"] == "error")
        .map(|d| d["message"].as_str().unwrap().to_owned())
        .collect();
    (replay, errors)
}

/// `(extended_to, extended_under, request_made)` of each extended
/// obligation of `deadline`.
fn extended(run: &Run, deadline: &str) -> Vec<(String, String, String)> {
    let field = |o: &serde_json::Value, k: &str| o[k].as_str().unwrap_or("-").to_owned();
    run.json["result"]["cases"][0]["deadlines"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|o| o["deadline"] == deadline && !o["extended_to"].is_null())
        .map(|o| {
            (
                field(o, "extended_to"),
                field(o, "extended_under"),
                field(o, "request_made"),
            )
        })
        .collect()
}

#[test]
fn court_ordered_extensions_are_recorded() {
    let run = check(
        "frcp-extended",
        &[
            "2025-01-06 ComplaintFiled() by plaintiff",
            "2025-03-20 ServicePeriodExtended(2025-06-30) by judge cite \"FRCP 6(b)(1)(A)\"",
            "2025-06-02 JudgmentEntered() by judge",
            "2025-07-05 ExtensionMoved(NoticeOfAppealPeriod) by plaintiff",
            "2025-07-10 NoticeOfAppealPeriodExtended(2025-08-01) by judge cite \"FRAP 4(a)(5)\"",
        ],
    );
    assert_eq!(run.code, 0, "{}", run.stdout);
    let (replay, errors) = effects(&run);
    assert_eq!(errors, Vec::<String>::new());
    assert_eq!(
        replay,
        [
            "triggered \"ComplaintFiled()\"",
            "extended \"ServicePeriodExtended(2025-06-30)\"",
            "triggered \"JudgmentEntered()\"",
            "history \"ExtensionMoved(NoticeOfAppealPeriod)\"",
            "extended \"NoticeOfAppealPeriodExtended(2025-08-01)\"",
        ]
    );
}

#[test]
fn rule_6b1_has_no_ceiling_and_reaches_an_expired_period_on_motion() {
    // Due 2025-04-07; extended a year out after it expired (6(b)(1)(B)).
    let run = check(
        "frcp-6b1b",
        &[
            "2025-01-06 ComplaintFiled() by plaintiff",
            "2025-04-20 ExtensionMoved(ServicePeriod) by plaintiff",
            "2025-05-01 ServicePeriodExtended(2026-05-01) by judge cite \"FRCP 6(b)(1)(B)\"",
        ],
    );
    assert_eq!(run.code, 0, "{}", run.stdout);
    assert_eq!(effects(&run).1, Vec::<String>::new());
    assert_eq!(
        extended(&run, "frcp-civil-procedure::service-period"),
        [(
            "2026-05-01".to_owned(),
            "ExcusableNeglect".to_owned(),
            "2025-04-20".to_owned()
        )]
    );
}

#[test]
fn rule_6b1a_turns_on_when_the_request_was_made() {
    // lg-flu.24 acceptance. Served day 0 (2025-03-06), so the 21-day
    // answer period is due day 21 (2025-03-27). The request on day 20 is
    // made before the original time expires; the order on day 25 comes
    // after it, and is good cause under 6(b)(1)(A): no motion after
    // expiry, so no excusable neglect, is demanded.
    let run = check(
        "frcp-6b1a-request",
        &[
            "2025-03-06 ProcessServed() by plaintiff",
            "2025-03-26 ExtensionMoved(AnswerPeriod) by plaintiff",
            "2025-03-31 AnswerPeriodExtended(2025-04-30) by judge cite \"FRCP 6(b)(1)(A)\"",
        ],
    );
    assert_eq!(run.code, 0, "{}", run.stdout);
    let (replay, errors) = effects(&run);
    assert_eq!(errors, Vec::<String>::new());
    assert_eq!(
        replay,
        [
            "triggered \"ProcessServed()\"",
            "history \"ExtensionMoved(AnswerPeriod)\"",
            "extended \"AnswerPeriodExtended(2025-04-30)\"",
        ]
    );
    assert_eq!(
        extended(&run, "frcp-civil-procedure::answer-period"),
        [(
            "2025-04-30".to_owned(),
            "GoodCause".to_owned(),
            "2025-03-26".to_owned()
        )]
    );
}

#[test]
fn rule_6b1_after_expiry_without_a_request_before_or_a_motion_after_is_rejected() {
    // Due 2025-04-07. The motion of 2025-04-01 is a 6(b)(1)(A) request
    // made before expiry, so the order after it is good cause; with no
    // request at all, the order after expiry has neither (A)'s request
    // nor (B)'s motion.
    let run = check(
        "frcp-6b1-request-before",
        &[
            "2025-01-06 ComplaintFiled() by plaintiff",
            "2025-04-01 ExtensionMoved(ServicePeriod) by plaintiff",
            "2025-05-01 ServicePeriodExtended(2026-05-01) by judge cite \"FRCP 6(b)(1)(A)\"",
        ],
    );
    assert_eq!(run.code, 0, "{}", run.stdout);
    assert_eq!(effects(&run).1, Vec::<String>::new());
    let run = check(
        "frcp-6b1-nomotion",
        &[
            "2025-01-06 ComplaintFiled() by plaintiff",
            "2025-05-01 ServicePeriodExtended(2026-05-01) by judge cite \"FRCP 6(b)(1)(B)\"",
        ],
    );
    assert_eq!(run.code, 2, "{}", run.stdout);
    assert_eq!(
        effects(&run).1,
        ["`ServicePeriodExtended(2026-05-01)` requires a motion `ExtensionMoved(ServicePeriod)` made after the period expired on 2025-04-07, and none precedes the order on 2025-05-01"]
    );
}

#[test]
fn a_request_after_the_order_does_not_license_it() {
    // Due 2025-03-27; the order on 2025-03-31 precedes the only request.
    let run = check(
        "frcp-6b1a-late-request",
        &[
            "2025-03-06 ProcessServed() by plaintiff",
            "2025-03-31 AnswerPeriodExtended(2025-04-30) by judge cite \"FRCP 6(b)(1)(A)\"",
            "2025-04-01 ExtensionMoved(AnswerPeriod) by plaintiff",
        ],
    );
    assert_eq!(run.code, 2, "{}", run.stdout);
    assert_eq!(effects(&run).1.len(), 1, "{}", run.stdout);
}

#[test]
fn the_waiver_answer_and_rule_15a1_periods_are_extendable() {
    let run = check(
        "frcp-waiver-15a1",
        &[
            "2025-03-10 WaiverRequestSent() by plaintiff",
            "2025-04-01 WaiverAnswerPeriodExtended(2025-06-30) by judge cite \"FRCP 6(b)(1)(A)\"",
            "2025-04-07 ResponsivePleadingServed(Electronic) by plaintiff",
            "2025-04-20 AmendmentAsOfCoursePeriodExtended(2025-05-30) by judge cite \"FRCP 6(b)(1)(A)\"",
        ],
    );
    assert_eq!(run.code, 0, "{}", run.stdout);
    assert_eq!(effects(&run).1, Vec::<String>::new());
}

#[test]
fn frap_4a5_order_on_day_40_allows_filing_through_day_54() {
    // Due 2025-07-02 (day 0 of the 4(a)(5) clock). The motion comes on day
    // 18; the order on day 40 (2025-08-11) may run to day 54: the later of
    // due + 30 days (2025-08-01) and order + 14 days (2025-08-25).
    let events = |until: &str| {
        [
            "2025-06-02 JudgmentEntered() by judge".to_owned(),
            "2025-07-20 ExtensionMoved(NoticeOfAppealPeriod) by plaintiff".to_owned(),
            format!(
                "2025-08-11 NoticeOfAppealPeriodExtended({until}) by judge cite \"FRAP 4(a)(5)\""
            ),
        ]
    };
    let ok = events("2025-08-25");
    let run = check(
        "frap-4a5-day54",
        &ok.iter().map(String::as_str).collect::<Vec<_>>(),
    );
    assert_eq!(run.code, 0, "{}", run.stdout);
    assert_eq!(effects(&run).1, Vec::<String>::new());
    let late = events("2025-08-26");
    let run = check(
        "frap-4a5-day55",
        &late.iter().map(String::as_str).collect::<Vec<_>>(),
    );
    assert_eq!(run.code, 2, "{}", run.stdout);
    assert_eq!(
        effects(&run).1,
        ["an extension to 2025-08-26 exceeds the bound of `frcp-civil-procedure::notice-of-appeal-period`: max(prescribed_due + 30.days, grant_order_entered + 14.days) = 2025-08-25 (prescribed_due 2025-07-02, grant_order_entered 2025-08-11)"]
    );
}

#[test]
fn frap_4a5_motion_must_come_within_30_days_of_the_prescribed_time() {
    // Due 2025-07-02; a motion on 2025-08-04 is past 4(a)(5)(A)(i)'s
    // 2025-08-01, whenever the order is entered.
    let run = check(
        "frap-4a5-late-motion",
        &[
            "2025-06-02 JudgmentEntered() by judge",
            "2025-08-04 ExtensionMoved(NoticeOfAppealPeriod) by plaintiff",
            "2025-08-05 NoticeOfAppealPeriodExtended(2025-08-15) by judge cite \"FRAP 4(a)(5)\"",
        ],
    );
    assert_eq!(run.code, 2, "{}", run.stdout);
    assert_eq!(
        effects(&run).1,
        ["`NoticeOfAppealPeriodExtended(2025-08-15)` requires a motion `ExtensionMoved(NoticeOfAppealPeriod)` no later than prescribed_due + 30.days = 2025-08-01, and none precedes the order on 2025-08-05"]
    );
}

#[test]
fn frap_4a5_extends_the_cross_appeal_period() {
    // Cross-appeal due 2025-07-15 (FRAP 4(a)(3)); moved after expiry,
    // extended within the later of 2025-08-14 and order + 14 days.
    let run = check(
        "frap-4a5-cross",
        &[
            "2025-07-01 NoticeOfAppealFiled() by judge",
            "2025-07-20 ExtensionMoved(CrossAppealPeriod) by plaintiff",
            "2025-07-25 CrossAppealPeriodExtended(2025-08-10) by judge cite \"FRAP 4(a)(5)\"",
        ],
    );
    assert_eq!(run.code, 0, "{}", run.stdout);
    assert_eq!(effects(&run).1, Vec::<String>::new());
}

/// `(due, floor, status)` of each instance of `deadline`.
fn scheduled(run: &Run, deadline: &str) -> Vec<(String, String, String)> {
    let field = |o: &serde_json::Value, k: &str| o[k].as_str().unwrap_or("-").to_owned();
    run.json["result"]["cases"][0]["deadlines"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|o| o["deadline"] == deadline)
        .map(|o| (field(o, "due"), field(o, "floor"), field(o, "status")))
        .collect()
}

#[test]
fn frap_4a3_cross_appeal_runs_to_the_later_of_14_days_and_the_notice_period() {
    // Judgment 2025-06-02: notice of appeal due 2025-07-02. A first notice
    // on 2025-06-05 counts 14 days to 2025-06-19, a Juneteenth that rolls
    // to 2025-06-20; "whichever period ends later" is 2025-07-02, so the
    // cross-appeal instance absorbs the notice-of-appeal one.
    let run = check(
        "frap-4a3-later",
        &[
            "2025-06-02 JudgmentEntered() by judge",
            "2025-06-05 NoticeOfAppealFiled() by judge",
        ],
    );
    assert_eq!(run.code, 0, "{}", run.stdout);
    assert_eq!(effects(&run).1, Vec::<String>::new());
    assert_eq!(
        scheduled(&run, "frcp-civil-procedure::notice-of-appeal-period"),
        [("2025-07-02".to_owned(), "-".to_owned(), "merged".to_owned())]
    );
    assert_eq!(
        scheduled(&run, "frcp-civil-procedure::cross-appeal-period"),
        [(
            "2025-06-20".to_owned(),
            "2025-07-02".to_owned(),
            "lapsed".to_owned()
        )]
    );
}

#[test]
fn frap_4a5_counts_the_cross_appeal_motion_window_from_the_later_date() {
    // The cross-appeal period above ends 2025-07-02, not 2025-06-20, so a
    // motion on 2025-07-25 is within 4(a)(5)(A)(i)'s 30 days (2025-08-01).
    let run = check(
        "frap-4a5-cross-later",
        &[
            "2025-06-02 JudgmentEntered() by judge",
            "2025-06-05 NoticeOfAppealFiled() by judge",
            "2025-07-25 ExtensionMoved(CrossAppealPeriod) by plaintiff",
            "2025-07-28 CrossAppealPeriodExtended(2025-08-05) by judge cite \"FRAP 4(a)(5)\"",
        ],
    );
    assert_eq!(run.code, 0, "{}", run.stdout);
    assert_eq!(effects(&run).1, Vec::<String>::new());
    assert_eq!(
        extended(&run, "frcp-civil-procedure::cross-appeal-period"),
        [(
            "2025-08-05".to_owned(),
            "GoodCauseOrExcusableNeglect".to_owned(),
            "2025-07-25".to_owned()
        )]
    );
}

#[test]
fn frap_4a5_caps_the_notice_of_appeal_extension() {
    // Due 2025-07-02; FRAP 4(a)(5)(C) allows at most 30 days past it.
    let run = check(
        "frcp-4a5",
        &[
            "2025-06-02 JudgmentEntered() by judge",
            "2025-07-05 ExtensionMoved(NoticeOfAppealPeriod) by plaintiff",
            "2025-07-10 NoticeOfAppealPeriodExtended(2025-08-05) by judge cite \"FRAP 4(a)(5)\"",
        ],
    );
    assert_eq!(run.code, 2, "{}", run.stdout);
    let (_, errors) = effects(&run);
    assert_eq!(errors.len(), 1, "{errors:#?}");
    assert!(
        errors[0]
            .contains("= 2025-08-01 (prescribed_due 2025-07-02, grant_order_entered 2025-07-10)"),
        "{errors:#?}"
    );
}

#[test]
fn only_the_court_extends() {
    let run = check(
        "frcp-party",
        &[
            "2025-01-06 ComplaintFiled() by plaintiff",
            "2025-03-20 ServicePeriodExtended(2025-06-30) by plaintiff cite \"FRCP 6(b)(1)(A)\"",
        ],
    );
    assert_eq!(run.code, 2, "{}", run.stdout);
    assert_eq!(
        effects(&run).1,
        ["only court may extend `frcp-civil-procedure::service-period`, but `plaintiff` is applicant"]
    );
}

/// `litigate check` of a PTAB matter: a final written decision on
/// `decided`, then `events`.
fn ptab(name: &str, decided: &str, events: &[&str]) -> Run {
    let decision = format!("{decided} DecisionEntered(FinalWritten) on owner");
    let events: Vec<&str> = std::iter::once(decision.as_str())
        .chain(events.iter().copied())
        .collect();
    matter(
        name,
        &["mpep", "ptab"],
        "ptab-patent-trial-appeal-board ^0.1.0",
        "  owner: either Corporation\n  director: director Agency\n",
        &events,
        "ptab-patent-trial-appeal-board::appeal-forfeited-unappealed",
    )
}

#[test]
fn the_director_extends_the_ptab_appeal_period_on_a_request_made_before_expiry() {
    // Decided 2025-03-01: day 63 is Saturday 2025-05-03, so the appeal is
    // due Monday 2025-05-05 (35 U.S.C. § 21(b)). A request received on
    // Sunday 2025-05-04 is filed that day (37 C.F.R. § 90.3(c)(2)), before
    // expiry: good cause under § 90.3(c)(1)(i), though the Director acts
    // after the period expired.
    let run = ptab(
        "ptab-90-3-c-1-i",
        "2025-03-01",
        &[
            "2025-05-04 AppealExtensionRequested() by owner",
            "2025-05-07 AppealPeriodExtended(2025-06-06) by director cite \"37 C.F.R. § 90.3(c)(1)(i)\"",
        ],
    );
    assert_eq!(run.code, 0, "{}", run.stdout);
    assert_eq!(effects(&run).1, Vec::<String>::new());
    assert_eq!(
        extended(&run, "ptab-patent-trial-appeal-board::appeal-period"),
        [(
            "2025-06-06".to_owned(),
            "GoodCause".to_owned(),
            "2025-05-04".to_owned()
        )]
    );
}

#[test]
fn a_ptab_request_received_after_expiry_is_excusable_neglect() {
    let run = ptab(
        "ptab-90-3-c-1-ii",
        "2025-03-01",
        &[
            "2025-05-06 AppealExtensionRequested() by owner",
            "2025-05-07 AppealPeriodExtended(2025-06-06) by director cite \"37 C.F.R. § 90.3(c)(1)(ii)\"",
        ],
    );
    assert_eq!(run.code, 0, "{}", run.stdout);
    assert_eq!(
        extended(&run, "ptab-patent-trial-appeal-board::appeal-period"),
        [(
            "2025-06-06".to_owned(),
            "ExcusableNeglect".to_owned(),
            "2025-05-06".to_owned()
        )]
    );
}

#[test]
fn the_director_extends_only_on_written_request() {
    // § 90.3(c)(1) acts "upon written request": no request, no extension,
    // even before expiry.
    let run = ptab(
        "ptab-90-3-no-request",
        "2025-03-01",
        &["2025-04-01 AppealPeriodExtended(2025-06-06) by director cite \"37 C.F.R. § 90.3(c)(1)\""],
    );
    assert_eq!(run.code, 2, "{}", run.stdout);
    assert_eq!(
        effects(&run).1,
        ["`AppealPeriodExtended(2025-06-06)` requires a motion `AppealExtensionRequested()` made before the period expired on 2025-05-05, and none precedes the order on 2025-04-01"]
    );
}

#[test]
fn the_director_extends_the_ptab_appeal_period_under_the_2012_text() {
    // Decided 2020-01-02, under the 2012 version (77 FR 48677): the appeal
    // is due Thursday 2020-03-05. Its § 90.3(c)(1) has the same two
    // grants: a request before expiry is good cause though the Director
    // acts after it, one after expiry excusable neglect.
    for (name, requested, under) in [
        ("ptab-2012-90-3-c-1-i", "2020-03-04", "GoodCause"),
        ("ptab-2012-90-3-c-1-ii", "2020-03-09", "ExcusableNeglect"),
    ] {
        let run = ptab(
            name,
            "2020-01-02",
            &[
                &format!("{requested} AppealExtensionRequested() by owner"),
                "2020-03-10 AppealPeriodExtended(2020-04-06) by director cite \"77 FR 48677\"",
            ],
        );
        assert_eq!(run.code, 0, "{}", run.stdout);
        assert_eq!(effects(&run).1, Vec::<String>::new());
        assert_eq!(
            extended(&run, "ptab-patent-trial-appeal-board::appeal-period"),
            [(
                "2020-04-06".to_owned(),
                under.to_owned(),
                requested.to_owned()
            )]
        );
    }
    // Still only "upon written request".
    let run = ptab(
        "ptab-2012-90-3-no-request",
        "2020-01-02",
        &["2020-02-03 AppealPeriodExtended(2020-04-06) by director cite \"77 FR 48677\""],
    );
    assert_eq!(run.code, 2, "{}", run.stdout);
    assert_eq!(
        effects(&run).1,
        ["`AppealPeriodExtended(2020-04-06)` requires a motion `AppealExtensionRequested()` made before the period expired on 2020-03-05, and none precedes the order on 2020-02-03"]
    );
}

#[test]
fn no_court_extends_the_ptab_appeal_period() {
    // The Director, not the Federal Circuit or a district court, holds the
    // § 90.3(c) power. A matter's parties play its primary package's
    // actors, so the Federal Circuit's package leads (in force from
    // 2025-12-01).
    let run = matter(
        "ptab-90-3-court",
        &["cafc", "mpep", "ptab", "ttab"],
        "cafc ^0.1.0, ptab-patent-trial-appeal-board ^0.1.0",
        "  owner: either Corporation\n  fedcir: court Tribunal\n",
        &[
            "2025-12-02 ptab-patent-trial-appeal-board::DecisionEntered(FinalWritten) on owner",
            "2026-01-05 ptab-patent-trial-appeal-board::AppealExtensionRequested() by owner",
            "2026-01-06 ptab-patent-trial-appeal-board::AppealPeriodExtended(2026-03-06) by fedcir cite \"37 C.F.R. § 90.3(c)(1)\"",
        ],
        "ptab-patent-trial-appeal-board::appeal-forfeited-unappealed",
    );
    assert_eq!(run.code, 2, "{}", run.stdout);
    assert_eq!(
        effects(&run).1,
        ["only director may extend `ptab-patent-trial-appeal-board::appeal-period`, but `fedcir` is court"]
    );
}

#[test]
fn the_director_extends_the_ttab_civil_action_period() {
    // Decided 2025-09-02: the civil action is due 2025-11-04 (37 C.F.R. §
    // 2.145(d)(3)). Requested before, granted after: good cause (§
    // 2.145(e)(1)(i)); the district court (`lit/frcp`) may not grant it.
    let run = |name: &str, pkgs: &[&str], uses: &str, parties: &str, by: &str| {
        // Unqualified in the TTAB's own matter, `pack::` qualified where
        // it is not the primary package.
        let q = if pkgs[0] == "ttab" {
            ""
        } else {
            "ttab-trademark-trial-and-appeal-board::"
        };
        let events = [
            format!("2025-09-02 {q}FinalDecision() on applicant"),
            format!("2025-11-03 {q}CivilActionExtensionRequested() by applicant"),
            format!(
                "2025-11-10 {q}CivilActionPeriodExtended(2025-12-10) by {by} cite \"37 CFR 2.145(e)(1)(i)\""
            ),
        ];
        matter(
            name,
            pkgs,
            uses,
            parties,
            &events.iter().map(String::as_str).collect::<Vec<_>>(),
            "ttab-trademark-trial-and-appeal-board::registered",
        )
    };
    let ok = run(
        "ttab-2-145-e",
        &["ttab"],
        "ttab-trademark-trial-and-appeal-board ^0.1.0",
        "  applicant: applicant Corporation\n  director: director Agency\n",
        "director",
    );
    assert_eq!(ok.code, 0, "{}", ok.stdout);
    assert_eq!(
        extended(
            &ok,
            "ttab-trademark-trial-and-appeal-board::civil-action-period"
        ),
        [(
            "2025-12-10".to_owned(),
            "GoodCause".to_owned(),
            "2025-11-03".to_owned()
        )]
    );
    let court = run(
        "ttab-2-145-e-court",
        &["frcp", "ttab"],
        "frcp-civil-procedure ^2024.12.0, ttab-trademark-trial-and-appeal-board ^0.1.0",
        "  applicant: applicant Corporation\n  district: court Tribunal\n",
        "district",
    );
    assert_eq!(court.code, 2, "{}", court.stdout);
    assert_eq!(
        effects(&court).1,
        ["only director may extend `ttab-trademark-trial-and-appeal-board::civil-action-period`, but `district` is court"]
    );
}

/// `litigate check` of a FRAP matter: the court of appeals' judgment on
/// 2025-10-15, then `events`. The certiorari petition is due 2026-01-13
/// (28 U.S.C. § 2101(c), counted by Sup. Ct. R. 30.1).
fn cert(name: &str, events: &[&str]) -> Run {
    let judgment = "2025-10-15 EnteredOnAppeal(Judgment) on appellant";
    let events: Vec<&str> = std::iter::once(judgment)
        .chain(events.iter().copied())
        .collect();
    matter(
        name,
        &["frap"],
        "frap-appellate-procedure ^0.1.0",
        "  appellant: applicant Corporation\n  justice: justice Tribunal\n  panel: court Tribunal\n",
        &events,
        "frap-appellate-procedure::cert-not-sought",
    )
}

#[test]
fn a_justice_extends_the_cert_period_by_up_to_60_days() {
    let run = cert(
        "cert-2101c",
        &[
            "2026-01-02 CertExtensionApplied() by appellant",
            "2026-01-07 CertPeriodExtended(2026-03-14) by justice cite \"Sup. Ct. R. 13.5\"",
        ],
    );
    assert_eq!(run.code, 0, "{}", run.stdout);
    assert_eq!(
        extended(&run, "frap-appellate-procedure::cert-period"),
        [(
            "2026-03-14".to_owned(),
            "GoodCause".to_owned(),
            "2026-01-02".to_owned()
        )]
    );
}

#[test]
fn a_cert_extension_past_60_days_or_by_a_court_is_rejected() {
    let long = cert(
        "cert-2101c-long",
        &[
            "2026-01-02 CertExtensionApplied() by appellant",
            "2026-01-07 CertPeriodExtended(2026-03-15) by justice cite \"Sup. Ct. R. 13.5\"",
        ],
    );
    assert_eq!(long.code, 2, "{}", long.stdout);
    assert_eq!(
        effects(&long).1,
        ["an extension to 2026-03-15 exceeds the bound of `frap-appellate-procedure::cert-period`: prescribed_due + 60.days = 2026-03-14 (prescribed_due 2026-01-13, grant_order_entered 2026-01-07)"]
    );
    let court = cert(
        "cert-2101c-court",
        &[
            "2026-01-02 CertExtensionApplied() by appellant",
            "2026-01-07 CertPeriodExtended(2026-02-13) by panel cite \"Sup. Ct. R. 13.5\"",
        ],
    );
    assert_eq!(court.code, 2, "{}", court.stdout);
    assert_eq!(
        effects(&court).1,
        ["only justice may extend `frap-appellate-procedure::cert-period`, but `panel` is court"]
    );
    // The application is filed within the period (Sup. Ct. R. 30.2), and
    // the Justice acts before it expires.
    let late = cert(
        "cert-2101c-late",
        &[
            "2026-01-14 CertExtensionApplied() by appellant",
            "2026-01-15 CertPeriodExtended(2026-02-13) by justice cite \"Sup. Ct. R. 13.5\"",
        ],
    );
    assert_eq!(late.code, 2, "{}", late.stdout);
    assert_eq!(
        effects(&late).1,
        ["`frap-appellate-procedure::cert-period` expired on 2026-01-13, before `CertPeriodExtended(2026-02-13)` on 2026-01-15; the capability does not reach an expired period (no `after_expiry`)"]
    );
}

/// An application less than 10 days before the due date is granted only
/// in extraordinary circumstances (Sup. Ct. R. 13.5, 30.2); once denied it
/// may not be renewed (Sup. Ct. R. 30.3); a second application is timed
/// against the due date as extended, and the 60 days run from the
/// prescribed one (28 U.S.C. § 2101(c)).
#[test]
fn cert_extension_timing_follows_rules_13_5_and_30() {
    let late = cert(
        "cert-30-2-late",
        &[
            "2026-01-05 CertExtensionApplied() by appellant",
            "2026-01-07 CertPeriodExtended(2026-02-13) by justice cite \"Sup. Ct. R. 13.5\"",
        ],
    );
    assert_eq!(late.code, 0, "{}", late.stdout);
    assert_eq!(
        extended(&late, "frap-appellate-procedure::cert-period"),
        [(
            "2026-02-13".to_owned(),
            "ExtraordinaryCircumstances".to_owned(),
            "2026-01-05".to_owned()
        )]
    );
    let renewed = cert(
        "cert-30-3-renewed",
        &[
            "2026-01-02 CertExtensionApplied() by appellant",
            "2026-01-05 CertExtensionDenied() by justice",
            "2026-01-06 CertExtensionApplied() by appellant",
            "2026-01-07 CertPeriodExtended(2026-02-13) by justice cite \"Sup. Ct. R. 13.5\"",
        ],
    );
    assert_eq!(renewed.code, 2, "{}", renewed.stdout);
    assert_eq!(
        effects(&renewed).1,
        ["`CertExtensionDenied()` on 2026-01-05 bars `CertPeriodExtended(2026-02-13)`: once denied, the motion `CertExtensionApplied()` may not be renewed (`unless CertExtensionDenied()`)"]
    );
    let second = cert(
        "cert-13-5-second",
        &[
            "2026-01-02 CertExtensionApplied() by appellant",
            "2026-01-07 CertPeriodExtended(2026-02-12) by justice cite \"Sup. Ct. R. 13.5\"",
            "2026-02-02 CertExtensionApplied() by appellant",
            "2026-02-04 CertPeriodExtended(2026-03-14) by justice cite \"Sup. Ct. R. 13.5\"",
        ],
    );
    assert_eq!(second.code, 0, "{}", second.stdout);
    assert_eq!(
        extended(&second, "frap-appellate-procedure::cert-period"),
        [(
            "2026-03-14".to_owned(),
            "GoodCause".to_owned(),
            "2026-02-02".to_owned()
        )]
    );
}

/// The Federal Circuit's certiorari period has the same two grants: its
/// panel judgment of 2025-12-01 makes the petition due 2026-03-02 (the
/// 90th day a Sunday, Sup. Ct. R. 30.1), so an application on 2026-02-25
/// is late for good cause and is granted only in extraordinary
/// circumstances (Sup. Ct. R. 13.5, 30.2).
#[test]
fn a_late_federal_circuit_cert_application_needs_extraordinary_circumstances() {
    let run = matter(
        "fedcir-cert-30-2",
        &["cafc", "mpep", "ttab"],
        "cafc ^0.1.0",
        "  appellant: applicant Corporation\n  justice: justice Tribunal\n  fedcir: court Tribunal\n",
        &[
            "2025-12-01 Entered(PanelJudgment) on appellant",
            "2026-02-25 CertExtensionApplied() by appellant",
            "2026-02-27 CertPeriodExtended(2026-04-01) by justice cite \"Sup. Ct. R. 13.5\"",
        ],
        "cafc::cert-not-sought",
    );
    assert_eq!(run.code, 0, "{}", run.stdout);
    assert_eq!(
        extended(&run, "cafc::cert-period"),
        [(
            "2026-04-01".to_owned(),
            "ExtraordinaryCircumstances".to_owned(),
            "2026-02-25".to_owned()
#[test]
fn the_federal_circuit_cross_appeal_runs_to_the_later_of_14_days_and_frap_4a1() {
    // A district-court judgment of 2026-03-02: the FRAP 4(a)(1)(A) notice
    // is due 2026-04-01. A first notice on 2026-03-05 counts 14 days to
    // 2026-03-19; "whichever period ends later" is 2026-04-01, so the
    // cross-appeal instance absorbs the 4(a)(1)(A) one as its floor.
    let run = matter(
        "fedcir-4a3-later",
        &["cafc", "mpep", "ttab"],
        "cafc ^0.1.0",
        "  appellant: applicant Corporation\n  fedcir: court Tribunal\n",
        &[
            "2026-03-02 Entered(Judgment) on appellant",
            "2026-03-05 Filed(NoticeOfAppeal) by appellant",
        ],
        "cafc::window-blown-district-court",
    );
    assert_eq!(run.code, 0, "{}", run.stdout);
    assert_eq!(effects(&run).1, Vec::<String>::new());
    assert_eq!(
        scheduled(&run, "cafc::district-court-noa-period"),
        [("2026-04-01".to_owned(), "-".to_owned(), "merged".to_owned())]
    );
    assert_eq!(
        scheduled(&run, "cafc::cross-appeal-period"),
        [(
            "2026-03-19".to_owned(),
            "2026-04-01".to_owned(),
            "lapsed".to_owned()
        )]
    );
}

/// `litigate check` of a Federal Circuit matter on a Board decision of
/// 2026-03-02, whose notice of appeal is due 2026-05-04 (37 C.F.R. §
/// 90.3(a)(1)), then `events`.
fn fedcir_ptab(name: &str, events: &[&str]) -> Run {
    let decision = "2026-03-02 Entered(BoardDecision) on appellant";
    let events: Vec<&str> = std::iter::once(decision)
        .chain(events.iter().copied())
        .collect();
    matter(
        name,
        &["cafc", "mpep", "ttab"],
        "cafc ^0.1.0",
        "  appellant: applicant Corporation\n  director: director Agency\n  fedcir: court Tribunal\n",
        &events,
        "cafc::window-blown-ptab",
    )
}

#[test]
fn the_director_not_the_court_extends_the_federal_circuit_ptab_window() {
    let ok = fedcir_ptab(
        "fedcir-90-3-c",
        &[
            "2026-05-01 PtabAppealExtensionRequested() by appellant",
            "2026-05-06 PtabAppealPeriodExtended(2026-06-03) by director cite \"37 C.F.R. § 90.3(c)(1)(i)\"",
        ],
    );
    assert_eq!(ok.code, 0, "{}", ok.stdout);
    assert_eq!(
        extended(&ok, "cafc::ptab-noa-period"),
        [(
            "2026-06-03".to_owned(),
            "GoodCause".to_owned(),
            "2026-05-01".to_owned()
        )]
    );
    let court = fedcir_ptab(
        "fedcir-90-3-c-court",
        &[
            "2026-05-01 PtabAppealExtensionRequested() by appellant",
            "2026-05-06 PtabAppealPeriodExtended(2026-06-03) by fedcir cite \"37 C.F.R. § 90.3(c)(1)(i)\"",
        ],
    );
    assert_eq!(court.code, 2, "{}", court.stdout);
    assert_eq!(
        effects(&court).1,
        ["only director may extend `cafc::ptab-noa-period`, but `fedcir` is court"]
    );
}
