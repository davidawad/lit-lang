//! Option-removing events (SPEC §5.6): a cited `Forfeited`/`Struck` event
//! removes the edge it names for the rest of the matter, sema records it
//! in `Case::removals`, and lowering emits it as `scenario.remove_edges`.
mod case_support;

use case_support::{check_2009, frcp2009, matter_2009, render};
use litigate_lower::{render as render_json, render_scenario, Lowerer};
use litigate_sema::hir::Package;
use std::collections::BTreeMap;

const SERVED: &str = "2009-12-04 Served(complaint, Personal) on widget";
const FORFEIT: &str = "2009-12-10 Forfeited(\"move-dismiss\") by widget cite FRCP 12(h)(1)";

fn row(events: &[&str]) -> Vec<String> {
    let checked = check_2009(&matter_2009(events));
    let mut out = render(&checked.diagnostics);
    if checked.ok() {
        let case = checked.case.unwrap_or_else(|| panic!("resolved"));
        out.push(format!("ok: start {}", case.start.unwrap_or_default()));
        out.extend(case.removals.iter().map(|r| {
            format!(
                "removed {} on {} (event {}, cite {})",
                r.edge, r.date, r.event, r.authority
            )
        }));
    }
    out
}

#[test]
fn removal_replay_table() {
    let rows: Vec<(&str, Vec<&str>)> = vec![
        (
            "ok: a forfeited motion leaves the answer",
            vec![SERVED, FORFEIT, "2009-12-28 Answered(complaint) by widget"],
        ),
        (
            "ok: a package-qualified option, removed before service",
            vec![
                "2009-12-01 Forfeited(\"frcp::move-dismiss\") by widget cite FRCP 12(h)(1)",
                SERVED,
            ],
        ),
        (
            "ok: the court strikes an outcome of a later stage",
            vec![
                SERVED,
                "2009-12-07 Struck(\"granted\") by court cite FRCP 12(f)",
                "2009-12-10 MovedToDismiss(complaint) by widget",
            ],
        ),
        (
            "removed: the forfeited motion cannot be taken",
            vec![
                SERVED,
                FORFEIT,
                "2009-12-11 MovedToDismiss(complaint) by widget",
            ],
        ),
        (
            "removed: an outcome struck before the ruling",
            vec![
                SERVED,
                "2009-12-07 Struck(\"denied\") by court cite FRCP 12(f)",
                "2009-12-10 MovedToDismiss(complaint) by widget",
                "2010-01-04 Denied(motion) by court",
            ],
        ),
        (
            "uncited: a removal must cite its authority",
            vec![SERVED, "2009-12-10 Forfeited(\"move-dismiss\") by widget"],
        ),
        (
            "unresolved: no such edge, nearest suggested",
            vec![
                SERVED,
                "2009-12-10 Forfeited(\"move-dismis\") by widget cite FRCP 12(h)(1)",
            ],
        ),
        (
            "duplicate: the same option removed twice",
            vec![
                SERVED,
                FORFEIT,
                "2009-12-11 Struck(\"move-dismiss\") by court cite FRCP 12(f)",
            ],
        ),
    ];
    let table: BTreeMap<&str, Vec<String>> = rows
        .iter()
        .map(|(name, events)| (*name, row(events)))
        .collect();
    insta::assert_debug_snapshot!(table);
}

/// Checking replaces asserted removals with the derived ones.
#[test]
fn removals_are_derived_never_asserted() {
    let checked = check_2009(&matter_2009(&[SERVED]));
    let mut case = checked.case.unwrap_or_else(|| panic!("resolved"));
    case.removals.push(litigate_sema::hir::Removal {
        edge: "frcp::answer".into(),
        date: "2009-12-05".into(),
        event: 0,
        authority: "FRCP 1".into(),
    });
    let refs: Vec<&Package> = frcp2009().iter().collect();
    let lock = case_support::lock(frcp2009());
    let rechecked = litigate_sema::case::check(
        case,
        &litigate_sema::case::CaseSpans::default(),
        litigate_sema::case::Env {
            packages: &refs,
            lock: Some(&lock),
            clock: case_support::frcp2009_clock(),
            profile: litigate_base::Profile::Dev,
        },
    );
    assert!(rechecked.ok(), "{:#?}", render(&rechecked.diagnostics));
    assert_eq!(rechecked.case.map(|c| c.removals), Some(Vec::new()));
}

/// The checked matter lowers its removals to `scenario.remove_edges`, each
/// mapped back to its event and cite.
#[test]
fn removals_lower_to_remove_edges() {
    let checked = check_2009(&matter_2009(&[
        SERVED,
        "2009-12-07 Struck(\"granted\") by court cite FRCP 12(f)",
        FORFEIT,
    ]));
    assert!(checked.ok(), "{:#?}", render(&checked.diagnostics));
    let case = checked.case.unwrap_or_else(|| panic!("resolved"));
    let refs: Vec<&Package> = frcp2009().iter().collect();
    let lowered = Lowerer::new(&refs)
        .case(&case, "cases/t.case")
        .unwrap_or_else(|d| panic!("{d:#?}"));
    assert_eq!(
        lowered.scenario.scenario.remove_edges,
        ["frcp::granted", "frcp::move-dismiss"]
    );
    insta::assert_snapshot!(
        "removal_scenario",
        render_scenario(&lowered.scenario).unwrap_or_default()
    );
    insta::assert_snapshot!(
        "removal_map",
        render_json(&lowered.source_map).unwrap_or_default()
    );
}
