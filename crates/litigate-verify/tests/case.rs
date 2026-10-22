//! `case-replay` for matters. Sema does not yet replay `.case` files (bead
//! "sema case"), so these feed its diagnostics in directly; `unvalued`
//! stakes are checked here.
use litigate_base::{Diagnostic, DiagnosticCode, FileId, Profile, Span};
use litigate_sema::hir;
use litigate_verify::{verify_case, Obligation, ObligationStatus, Verification};
use serde_json::json;

const FILE: &str = "cases/examples/acme.case";

fn case(stake: &serde_json::Value) -> hir::Case {
    serde_json::from_value(json!({
        "id": "acme-v-widget",
        "summary": "Acme sues Widget.",
        "notes": null,
        "sources": [],
        "uses": [],
        "as_of": "2025-06-01",
        "known_at": null,
        "start": null,
        "perspective": null,
        "parties": [],
        "events": [],
        "facts": {},
        "stakes": { "judgment-entered": stake },
        "observations": [],
        "analysis": { "fields": {} }
    }))
    .unwrap_or_else(|e| panic!("{e}"))
}

fn valued() -> hir::Case {
    case(&json!({ "stake": "value", "value": { "minor_units": -200_000_000, "currency": "usd" } }))
}

fn missed() -> Diagnostic {
    Diagnostic::error(
        DiagnosticCode::DeadlineMissed,
        Span {
            file: FileId(FILE.to_owned()),
            line: 22,
            col: 3,
            end_line: 22,
            end_col: 45,
        },
        "`MovedToDismiss(complaint)` on 2025-03-30 missed `answer-period` (due 2025-03-26)",
    )
}

fn replay(v: &Verification) -> serde_json::Value {
    json!({
        "finding": v.report.finding(Obligation::CaseReplay),
        "accepted": v.is_accepted(),
        "diagnostics": v.diagnostics,
    })
}

#[test]
fn case_replay() {
    let pass = verify_case(&valued(), FILE, &[], Profile::Strict);
    let fail = verify_case(&valued(), FILE, &[missed()], Profile::Strict);
    assert!(pass.is_accepted());
    assert!(!fail.is_accepted());
    insta::assert_json_snapshot!(json!({ "pass": replay(&pass), "fail": replay(&fail) }));
}

#[test]
fn unvalued_stakes_warn_in_dev_and_fail_in_strict() {
    let unvalued = case(&json!({ "stake": "unvalued" }));
    let dev = verify_case(&unvalued, FILE, &[], Profile::Dev);
    let strict = verify_case(&unvalued, FILE, &[], Profile::Strict);
    let status = |v: &Verification| v.report.finding(Obligation::CaseReplay).map(|f| f.status);
    assert_eq!(status(&dev), Some(ObligationStatus::Warning));
    assert!(dev.is_accepted());
    assert_eq!(status(&strict), Some(ObligationStatus::Failed));
    insta::assert_json_snapshot!(replay(&strict));
}

#[test]
fn a_case_report_skips_package_obligations() {
    let v = verify_case(&valued(), FILE, &[], Profile::Strict);
    let skipped = v
        .report
        .obligations
        .iter()
        .filter(|f| f.status == ObligationStatus::Skipped)
        .count();
    let passed: Vec<Obligation> = v
        .report
        .obligations
        .iter()
        .filter(|f| f.status == ObligationStatus::Passed)
        .map(|f| f.name)
        .collect();
    assert_eq!(skipped + passed.len(), Obligation::ALL.len());
    assert!(passed.contains(&Obligation::CaseReplay));
    assert!(passed.contains(&Obligation::WellTyped));
}
