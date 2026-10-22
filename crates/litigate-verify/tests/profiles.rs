//! `dev` versus `strict` (SPEC §7): profiles change severity and
//! acceptance, never values.
mod support;

use litigate_base::{Profile, Severity};
use litigate_verify::{Checked, Obligation, ObligationStatus};
use support::{edit, verify_one, with, BASE};

fn status(c: &Checked, name: Obligation) -> Option<ObligationStatus> {
    c.verification.report.finding(name).map(|f| f.status)
}

fn codes(c: &Checked) -> Vec<(&str, Severity)> {
    c.verification
        .diagnostics
        .iter()
        .map(|d| (d.code.as_str(), d.severity))
        .collect()
}

#[test]
fn a_hole_warns_in_dev_and_fails_in_strict() {
    let lit = edit("payoff 0.usd", "payoff ?(\"no estimate yet\")");
    let dev = verify_one(&lit, Profile::Dev);
    let strict = verify_one(&lit, Profile::Strict);
    assert_eq!(
        status(&dev, Obligation::Holes),
        Some(ObligationStatus::Warning)
    );
    assert!(dev.verification.is_accepted());
    assert_eq!(codes(&dev), [("L9002", Severity::Warning)]);
    assert_eq!(
        status(&strict, Obligation::Holes),
        Some(ObligationStatus::Failed)
    );
    assert!(!strict.verification.is_accepted());
    assert_eq!(codes(&strict), [("L5002", Severity::Error)]);
}

#[test]
fn an_untested_rule_warns_in_dev_and_fails_in_strict() {
    let lit = with(
        "rule duty reply-duty cite FRCP 12(a)\n{\n  deadline reply-period: 14.days from \
         Answered(complaint) counted_by frcp6\n}",
    );
    let dev = verify_one(&lit, Profile::Dev);
    let strict = verify_one(&lit, Profile::Strict);
    assert_eq!(
        status(&dev, Obligation::RuleWithoutTest),
        Some(ObligationStatus::Warning)
    );
    assert!(dev.verification.is_accepted());
    assert_eq!(codes(&dev), [("L9001", Severity::Warning)]);
    assert_eq!(
        status(&strict, Obligation::RuleWithoutTest),
        Some(ObligationStatus::Failed)
    );
    assert_eq!(codes(&strict), [("L9001", Severity::Error)]);
}

#[test]
fn other_failures_are_errors_in_both_profiles() {
    let lit = with("terminal island: \"Island\" { payoff 0.usd }");
    for profile in [Profile::Dev, Profile::Strict] {
        let c = verify_one(&lit, profile);
        assert_eq!(
            status(&c, Obligation::GraphReachable),
            Some(ObligationStatus::Failed)
        );
        assert!(!c.verification.is_accepted());
        assert_eq!(codes(&c), [("L5003", Severity::Error)]);
    }
}

#[test]
fn profiles_never_change_counts_or_values() {
    let lit = edit("payoff 0.usd", "payoff ?").replace(
        "expect deadline(answer-period) == 2025-03-27",
        "expect deadline(answer-period) == 2025-03-28",
    );
    let dev = verify_one(&lit, Profile::Dev);
    let strict = verify_one(&lit, Profile::Strict);
    let counts = |c: &Checked| -> Vec<(Obligation, usize)> {
        c.verification
            .report
            .obligations
            .iter()
            .map(|f| (f.name, f.count))
            .collect()
    };
    assert_eq!(counts(&dev), counts(&strict));
    assert_eq!(dev.lowered, strict.lowered);
    assert_eq!(dev.verification.report.profile, Profile::Dev);
    assert_eq!(strict.verification.report.profile, Profile::Strict);
}

#[test]
fn a_clean_program_is_accepted_in_both_profiles() {
    for profile in [Profile::Dev, Profile::Strict] {
        let c = verify_one(BASE, profile);
        assert!(c.verification.is_accepted(), "{profile:?}");
        assert!(c.verification.diagnostics.is_empty(), "{profile:?}");
        let skipped: Vec<Obligation> = c
            .verification
            .report
            .obligations
            .iter()
            .filter(|f| f.status != ObligationStatus::Passed)
            .map(|f| f.name)
            .collect();
        assert_eq!(skipped, [Obligation::CaseReplay]);
    }
}

/// SPEC §7: `cite-resolves` on a `/// imported` declaration (its own cite
/// and its edges' authorities) is a warning in `dev`; `strict` rejects it,
/// and an unmarked declaration fails in both.
#[test]
fn an_imported_cite_warns_in_dev_and_fails_in_strict() {
    let from = "state served: \"Complaint served\" by Defendant cite FRCP 12(a)";
    let bad = "state served: \"Complaint served\" by Defendant cite FRCP 99(a)";
    let imported = edit(from, &format!("/// imported\n{bad}"));
    let plain = edit(from, bad);
    let dev = verify_one(&imported, Profile::Dev);
    let strict = verify_one(&imported, Profile::Strict);
    let unmarked = verify_one(&plain, Profile::Dev);
    assert_eq!(
        status(&dev, Obligation::CiteResolves),
        Some(ObligationStatus::Warning)
    );
    assert!(dev.verification.is_accepted());
    assert_eq!(codes(&dev), [("L1010", Severity::Warning)]);
    assert_eq!(
        status(&strict, Obligation::CiteResolves),
        Some(ObligationStatus::Failed)
    );
    assert!(!strict.verification.is_accepted());
    assert_eq!(codes(&strict), [("L1010", Severity::Error)]);
    assert_eq!(
        status(&unmarked, Obligation::CiteResolves),
        Some(ObligationStatus::Failed)
    );
    assert_eq!(codes(&unmarked), [("L1010", Severity::Error)]);
    insta::assert_json_snapshot!(dev.verification.diagnostics);
}
