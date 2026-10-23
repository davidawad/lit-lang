//! A matter on the real `lit/frcp` package records court-ordered
//! extensions (lg-flu.21): FRCP 6(b)(1) grants with no numeric ceiling,
//! before expiry and after it on motion, and FRAP 4(a)(5)'s capped
//! extension of the notice-of-appeal period.
// Test helpers below may unwrap: a panic is the failed assertion.
#![allow(clippy::unwrap_used)]
mod support;

use support::{copy_dir, litigate_in, repo, scratch, Run};

/// `litigate check` of one matter over copies of `lit/core` and `lit/frcp`.
fn check(name: &str, events: &[&str]) -> Run {
    let dir = scratch(name);
    for pkg in ["core", "frcp"] {
        copy_dir(&repo().join("lit").join(pkg), &dir.join("lit").join(pkg));
    }
    std::fs::create_dir_all(dir.join("cases")).unwrap();
    let events: String = events.iter().map(|e| format!("  {e}\n")).collect();
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
            "2025-05-01 ServicePeriodExtended(2026-05-01) by judge cite FRCP 6(b)(1)(B)",
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
            "2025-07-10 NoticeOfAppealPeriodExtended(2025-08-05) by judge cite FRAP 4(a)(5)",
        ],
    );
    assert_eq!(run.code, 2, "{}", run.stdout);
    let (_, errors) = effects(&run);
    assert_eq!(errors.len(), 1, "{errors:#?}");
    assert!(
        errors[0].contains("due 2025-07-02 + 30 days = 2025-08-01"),
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
