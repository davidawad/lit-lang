//! Erie composition (SPEC §5.6; lg-fg2h.3): a matter names its `forum` and
//! its `governing_law`, and the resolver composes the forum's procedure
//! with the substantive layers of the governing law's tree. The fixture
//! (`fixtures/erie`) is a contract action in the Southern District of New
//! York in diversity under New Jersey limitations law: FRCP 12(a)'s 21-day
//! answer governs, New Jersey's 35 days under R. 4:6-1(a) never applies,
//! and the six-year period of N.J.S.A. 2A:14-1 bars a late commencement.
mod case_support;

use case_support::{clock, lock, render, repo_root};
use litigate_base::{DiagnosticCode, Profile};
use litigate_sema::case::{check_source, resolve, CheckedCase, Env, Status};
use litigate_sema::hir::Package;
use std::sync::OnceLock;

fn fixture(path: &str) -> std::path::PathBuf {
    repo_root()
        .join("crates/litigate-sema/tests/fixtures/erie")
        .join(path)
}

/// The fixture's packages and `lit/core`.
fn packages() -> &'static [Package] {
    static P: OnceLock<Vec<Package>> = OnceLock::new();
    P.get_or_init(|| {
        let root = repo_root();
        let dirs = ["erie-federal", "erie-nj-court-rules", "erie-nj-limitations"]
            .map(|d| fixture(&format!("lit/{d}")));
        let (db, ws) = litigate_sema::load(&root, &dirs, None).unwrap_or_else(|e| panic!("{e}"));
        litigate_sema::check_workspace(&db, ws)
            .iter()
            .map(|c| {
                c.package
                    .clone()
                    .unwrap_or_else(|| panic!("{:?}", c.diagnostics))
            })
            .collect()
    })
}

/// The fixture matter.
fn matter() -> String {
    std::fs::read_to_string(fixture("cases/sdny-diversity-nj-limitations.case")).unwrap_or_default()
}

fn check(text: &str) -> CheckedCase {
    let refs: Vec<&Package> = packages().iter().collect();
    let lock = lock(packages());
    let clock = clock(packages());
    check_source(
        "cases/t.case",
        text,
        Env {
            packages: &refs,
            lock: Some(&lock),
            clock: &clock,
            profile: Profile::Dev,
        },
    )
}

/// The packages the resolved matter uses, in order.
fn uses(text: &str) -> Vec<String> {
    let refs: Vec<&Package> = packages().iter().collect();
    let lock = lock(packages());
    let resolved = resolve("cases/t.case", text, &refs, Some(&lock));
    let case = resolved
        .case
        .unwrap_or_else(|| panic!("{:?}", render(&resolved.diagnostics)));
    case.uses.into_iter().map(|u| u.name).collect()
}

/// `(deadline, trigger date, status)` of every obligation.
fn obligations(checked: &CheckedCase) -> Vec<(String, String, Status)> {
    checked
        .obligations
        .iter()
        .map(|o| (o.deadline.clone(), o.date.clone(), o.status))
        .collect()
}

/// The rendered diagnostics of `code`.
fn with_code(checked: &CheckedCase, code: DiagnosticCode) -> Vec<String> {
    let found: Vec<_> = checked
        .diagnostics
        .iter()
        .filter(|d| d.code == code)
        .cloned()
        .collect();
    render(&found)
}

#[test]
fn the_forum_and_the_governing_laws_substantive_layers_are_the_matter() {
    // The forum is primary; New Jersey's limitations layer joins it, and
    // its Rules of Court, the tree's root, do not.
    assert_eq!(uses(&matter()), ["erie-federal", "erie-nj-limitations"]);
}

#[test]
fn a_timely_diversity_action_answers_the_state_period_under_federal_procedure() {
    let checked = check(&matter());
    assert!(checked.ok(), "{:?}", render(&checked.diagnostics));
    assert_eq!(
        obligations(&checked),
        [
            (
                "erie-nj-limitations::contract-period".to_owned(),
                "2018-01-10".to_owned(),
                Status::Discharged
            ),
            (
                "erie-federal::answer-period".to_owned(),
                "2023-11-27".to_owned(),
                Status::Lapsed
            ),
        ]
    );
}

#[test]
fn an_action_commenced_after_six_years_is_barred_by_new_jersey_law() {
    let text = matter()
        .replace("2023-11-20 docket 1", "2024-03-01 docket 1")
        .replace("2023-11-20 docket 2", "2024-03-01 docket 2")
        .replace("2023-11-27 docket 3", "2024-03-05 docket 3");
    let checked = check(&text);
    let late = with_code(&checked, DiagnosticCode::DeadlineMissed);
    assert_eq!(late.len(), 1, "{late:?}");
    assert!(
        late[0].contains("missed deadline `contract-period`, due 2024-01-10"),
        "{late:?}"
    );
    assert_eq!(
        obligations(&checked)[0],
        (
            "erie-nj-limitations::contract-period".to_owned(),
            "2018-01-10".to_owned(),
            Status::Late
        )
    );
}

/// The L4015 diagnostics of resolving `text`.
fn erie_errors(text: &str) -> Vec<String> {
    let refs: Vec<&Package> = packages().iter().collect();
    let lock = lock(packages());
    let resolved = resolve("cases/t.case", text, &refs, Some(&lock));
    assert!(
        resolved.case.is_none(),
        "an L4015 leaves the matter unresolved"
    );
    let found: Vec<_> = resolved
        .diagnostics
        .into_iter()
        .filter(|d| d.code == DiagnosticCode::InvalidGoverningLaw)
        .collect();
    render(&found)
}

#[test]
fn the_governing_laws_procedure_never_governs_in_the_forum() {
    let text = matter().replace(
        "governing_law erie-nj-court-rules ^1.0.0\n",
        "governing_law erie-nj-court-rules ^1.0.0\nuses erie-nj-court-rules ^1.0.0\n",
    );
    assert_eq!(
        erie_errors(&text),
        [
            "4:6 L4015 `erie-nj-court-rules` is procedure of the governing law `erie-nj-court-rules`: the forum applies its own procedure\n    note: only the governing law's substantive layers join the matter",
        ]
    );
}

#[test]
fn a_governing_law_needs_substantive_law_and_a_forum() {
    let none = matter().replace(
        "governing_law erie-nj-court-rules ^1.0.0",
        "governing_law erie-federal ^1.0.0",
    );
    assert_eq!(
        erie_errors(&none),
        ["3:15 L4015 governing law `erie-federal` states no substantive law: no package of its tree has `law = \"substantive\"`"]
    );
    let no_forum = matter()
        .replace("forum erie-federal ^1.0.0\n", "")
        .replace("as_of", "uses erie-federal ^1.0.0\nas_of");
    assert_eq!(
        erie_errors(&no_forum),
        ["2:1 L4015 `governing_law erie-nj-court-rules` needs a `forum`: the forum's procedure governs with it"]
    );
}

#[test]
fn a_forum_is_procedure() {
    let text = matter().replace(
        "forum erie-federal ^1.0.0",
        "forum erie-nj-limitations ^1.0.0",
    );
    assert_eq!(
        erie_errors(&text),
        ["2:7 L4015 forum `erie-nj-limitations` states substantive law: a forum is the procedure the court applies"]
    );
}
