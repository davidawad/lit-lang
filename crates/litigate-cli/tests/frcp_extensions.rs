//! A matter on the real `lit/frcp` package records court-ordered
//! extensions (lg-flu.21, lg-flu.23): FRCP 6(b)(1) grants with no numeric
//! ceiling, before expiry and after it on a motion made after expiry, and
//! FRAP 4(a)(5)'s extension of the notice-of-appeal and cross-appeal
//! periods, on a motion no later than 30 days after the prescribed time,
//! to the later of 30 days after it and 14 days after the order.
// Test helpers below may unwrap: a panic is the failed assertion.
#![allow(clippy::unwrap_used)]
mod support;

use std::fmt::Write as _;
use support::{copy_dir, litigate_in, repo, scratch, Run};

/// `litigate check` of one matter over copies of `lit/core` and `lit/frcp`.
fn check(name: &str, events: &[&str]) -> Run {
    let dir = scratch(name);
    for pkg in ["core", "frcp"] {
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
            "case ext\nuses frcp-civil-procedure ^2024.12.0\nas_of 2025-12-01\nsummary \"s\"\n\n\
             parties {{\n  plaintiff: applicant Corporation\n  judge: court Tribunal\n}}\n\n\
             events {{\n{events}}}\n\nstakes {{ frcp-civil-procedure::appeal-window-blown: 0.usd }}\n"
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

#[test]
fn court_ordered_extensions_are_recorded() {
    let run = check(
        "frcp-extended",
        &[
            "2025-01-06 ComplaintFiled() by plaintiff",
            "2025-03-20 ServicePeriodExtended(2025-06-30) by judge cite FRCP 6(b)(1)(A)",
            "2025-06-02 JudgmentEntered() by judge",
            "2025-07-05 ExtensionMoved(NoticeOfAppealPeriod) by plaintiff",
            "2025-07-10 NoticeOfAppealPeriodExtended(2025-08-01) by judge cite FRAP 4(a)(5)",
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
            "2025-05-01 ServicePeriodExtended(2026-05-01) by judge cite FRCP 6(b)(1)(B)",
        ],
    );
    assert_eq!(run.code, 0, "{}", run.stdout);
    assert_eq!(effects(&run).1, Vec::<String>::new());
}

#[test]
fn rule_6b1b_without_a_motion_is_rejected() {
    // Due 2025-04-07; the only motion came before expiry, so it is not
    // the 6(b)(1)(B) "motion made after the time has expired".
    let run = check(
        "frcp-6b1b-nomotion",
        &[
            "2025-01-06 ComplaintFiled() by plaintiff",
            "2025-04-01 ExtensionMoved(ServicePeriod) by plaintiff",
            "2025-05-01 ServicePeriodExtended(2026-05-01) by judge cite FRCP 6(b)(1)(B)",
        ],
    );
    assert_eq!(run.code, 2, "{}", run.stdout);
    assert_eq!(
        effects(&run).1,
        ["`ServicePeriodExtended(2026-05-01)` requires a motion `ExtensionMoved(ServicePeriod)` made after the period expired on 2025-04-07, and none precedes the order on 2025-05-01"]
    );
}

#[test]
fn the_waiver_answer_and_rule_15a1_periods_are_extendable() {
    let run = check(
        "frcp-waiver-15a1",
        &[
            "2025-03-10 WaiverRequestSent() by plaintiff",
            "2025-04-01 WaiverAnswerPeriodExtended(2025-06-30) by judge cite FRCP 6(b)(1)(A)",
            "2025-04-07 ResponsivePleadingServed(Electronic) by plaintiff",
            "2025-04-20 AmendmentAsOfCoursePeriodExtended(2025-05-30) by judge cite FRCP 6(b)(1)(A)",
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
            format!("2025-08-11 NoticeOfAppealPeriodExtended({until}) by judge cite FRAP 4(a)(5)"),
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
            "2025-08-05 NoticeOfAppealPeriodExtended(2025-08-15) by judge cite FRAP 4(a)(5)",
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
            "2025-07-25 CrossAppealPeriodExtended(2025-08-10) by judge cite FRAP 4(a)(5)",
        ],
    );
    assert_eq!(run.code, 0, "{}", run.stdout);
    assert_eq!(effects(&run).1, Vec::<String>::new());
}

#[test]
fn frap_4a5_caps_the_notice_of_appeal_extension() {
    // Due 2025-07-02; FRAP 4(a)(5)(C) allows at most 30 days past it.
    let run = check(
        "frcp-4a5",
        &[
            "2025-06-02 JudgmentEntered() by judge",
            "2025-07-05 ExtensionMoved(NoticeOfAppealPeriod) by plaintiff",
            "2025-07-10 NoticeOfAppealPeriodExtended(2025-08-05) by judge cite FRAP 4(a)(5)",
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
            "2025-03-20 ServicePeriodExtended(2025-06-30) by plaintiff cite FRCP 6(b)(1)(A)",
        ],
    );
    assert_eq!(run.code, 2, "{}", run.stdout);
    assert_eq!(
        effects(&run).1,
        ["only court may extend `frcp-civil-procedure::service-period`, but `plaintiff` is applicant"]
    );
}
