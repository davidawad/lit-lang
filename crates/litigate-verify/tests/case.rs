//! `case-replay` for matters: `verify_case` runs sema's case checker on the
//! SPEC §12 matter (`cases/examples/acme.case` over `lit/examples`, with
//! its lock and the engine clock) and reports its diagnostics.
use litigate_base::Profile;
use litigate_lower::{EngineClock, Lowerer};
use litigate_sema::case::{resolve, CaseSpans, Env};
use litigate_sema::hir::Package;
use litigate_sema::pkg::lock::{LockFile, LockedPackage};
use litigate_verify::{
    verify_case, verify_case_source, Obligation, ObligationStatus, VerifiedCase,
};
use serde_json::json;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

const FILE: &str = "cases/examples/acme.case";

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn packages() -> &'static [Package] {
    static P: OnceLock<Vec<Package>> = OnceLock::new();
    P.get_or_init(|| {
        let root = root();
        let (db, ws) = litigate_sema::load(&root, &[root.join("lit/examples")], None)
            .unwrap_or_else(|e| panic!("{e}"));
        litigate_sema::check_workspace(&db, ws)
            .iter()
            .filter_map(|c| c.package.clone())
            .collect()
    })
}

fn lock() -> LockFile {
    LockFile::new(
        packages()
            .iter()
            .map(|p| LockedPackage {
                name: p.manifest.name.clone(),
                version: p.manifest.version.clone(),
                source: format!("sha256:{}", p.digest),
                dependencies: Vec::new(),
                language: 1,
                ..Default::default()
            })
            .collect(),
    )
}

fn clock() -> &'static EngineClock {
    static C: OnceLock<EngineClock> = OnceLock::new();
    C.get_or_init(|| {
        let refs: Vec<&Package> = packages().iter().collect();
        EngineClock::lowered(&Lowerer::new(&refs), &refs).unwrap_or_else(|d| panic!("{d:?}"))
    })
}

fn acme() -> String {
    std::fs::read_to_string(root().join(FILE)).unwrap_or_else(|e| panic!("{e}"))
}

/// Runs `f` with an [`Env`] over the example under `profile`.
fn with_env<T>(profile: Profile, f: impl FnOnce(Env<'_>) -> T) -> T {
    let refs: Vec<&Package> = packages().iter().collect();
    let lock = lock();
    f(Env {
        packages: &refs,
        lock: Some(&lock),
        clock: clock(),
        profile,
    })
}

fn source(text: &str, profile: Profile) -> VerifiedCase {
    with_env(profile, |env| verify_case_source(FILE, text, env))
}

fn status(v: &VerifiedCase, name: Obligation) -> Option<ObligationStatus> {
    v.verification.report.finding(name).map(|f| f.status)
}

/// The `case-replay` finding, acceptance, the derived posture, and every
/// diagnostic as `line:col CODE message`.
fn summary(v: &VerifiedCase) -> serde_json::Value {
    json!({
        "finding": v.verification.report.finding(Obligation::CaseReplay),
        "accepted": v.is_accepted(),
        "posture": v.checked.case.as_ref().and_then(|c| c.start.clone()),
        "diagnostics": v.diagnostics().iter().map(|d| format!(
            "{}:{}:{} {} {}", d.span.file.0, d.span.line, d.span.col, d.code.as_str(), d.message
        )).collect::<Vec<_>>(),
    })
}

#[test]
fn the_worked_example_replays_to_its_posture() {
    let dev = source(&acme(), Profile::Dev);
    let strict = source(&acme(), Profile::Strict);
    assert!(dev.is_accepted());
    assert!(strict.is_accepted());
    assert_eq!(dev.verification.report, {
        let mut r = strict.verification.report.clone();
        r.profile = Profile::Dev;
        r
    });
    insta::assert_json_snapshot!(summary(&strict));
}

#[test]
fn a_missed_deadline_fails_case_replay() {
    let late = acme().replace("2025-03-20 MovedToDismiss", "2025-03-30 MovedToDismiss");
    let v = source(&late, Profile::Dev);
    assert!(!v.is_accepted());
    assert_eq!(
        status(&v, Obligation::CaseReplay),
        Some(ObligationStatus::Failed)
    );
    insta::assert_json_snapshot!(summary(&v));
}

#[test]
fn events_out_of_order_fail_case_replay() {
    let swapped = acme().replace(
        "  2025-03-05 Served(complaint, Personal) on widget\n  2025-03-20 MovedToDismiss(complaint) by widget\n",
        "  2025-03-20 MovedToDismiss(complaint) by widget\n  2025-03-05 Served(complaint, Personal) on widget\n",
    );
    assert_ne!(swapped, acme());
    let v = source(&swapped, Profile::Strict);
    assert!(!v.is_accepted());
    insta::assert_json_snapshot!(summary(&v));
}

#[test]
fn unvalued_stakes_warn_in_dev_and_fail_in_strict() {
    let unvalued = acme().replace("dismissed: 0.usd", "dismissed: unvalued");
    assert_ne!(unvalued, acme());
    let dev = source(&unvalued, Profile::Dev);
    let strict = source(&unvalued, Profile::Strict);
    assert_eq!(
        status(&dev, Obligation::CaseReplay),
        Some(ObligationStatus::Warning)
    );
    assert!(dev.is_accepted());
    assert_eq!(
        status(&strict, Obligation::CaseReplay),
        Some(ObligationStatus::Failed)
    );
    assert!(!strict.is_accepted());
    insta::assert_json_snapshot!(json!({ "dev": summary(&dev), "strict": summary(&strict) }));
}

#[test]
fn a_matter_that_does_not_resolve_skips_replay() {
    let broken = acme().replace("uses frcp-rule12-example", "uses no-such-package");
    let v = source(&broken, Profile::Dev);
    assert!(!v.is_accepted());
    insta::assert_json_snapshot!(json!({
        "summary": summary(&v),
        "well-typed": v.verification.report.finding(Obligation::WellTyped),
    }));
}

#[test]
fn hir_without_source_reports_at_the_case() {
    let refs: Vec<&Package> = packages().iter().collect();
    let resolved = resolve(FILE, &acme(), &refs, Some(&lock()));
    let mut case = resolved
        .case
        .unwrap_or_else(|| panic!("{:?}", resolved.diagnostics));
    case.events[1].date = "2025-03-27".into();
    let v = with_env(Profile::Dev, |env| {
        verify_case(case, &CaseSpans::default(), env)
    });
    assert!(!v.is_accepted());
    insta::assert_json_snapshot!(summary(&v));
}

#[test]
fn a_case_report_skips_package_obligations() {
    let v = source(&acme(), Profile::Strict);
    let report = &v.verification.report;
    let passed: Vec<Obligation> = report
        .obligations
        .iter()
        .filter(|f| f.status == ObligationStatus::Passed)
        .map(|f| f.name)
        .collect();
    let skipped = report
        .obligations
        .iter()
        .filter(|f| f.status == ObligationStatus::Skipped)
        .count();
    assert_eq!(skipped + passed.len(), Obligation::ALL.len());
    // A matter's own checks pass on evidence; sema obligations of the
    // packages it uses are not passed again by the absence of findings.
    assert_eq!(passed, [Obligation::WellTyped, Obligation::CaseReplay]);
    for name in passed {
        let evidence = report.finding(name).and_then(|f| f.evidence.clone());
        assert!(evidence.is_some_and(|e| e.checked > 0), "{name:?}");
    }
    assert_eq!(
        status(&v, Obligation::GraphReachable),
        Some(ObligationStatus::Skipped)
    );
}
