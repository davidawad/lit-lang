//! `.case` lowering edge cases (SPEC §8.3): every case diagnostic as a
//! snapshot, unvalued stakes, and removals mapped to the event that made
//! them. Each case mutates the SPEC §12 matter.
mod support;

use litigate_base::{FileId, Span};
use litigate_lower::{LoweredCase, Lowerer};
use litigate_sema::hir::{
    Case, Event, EventParty, Expr, ExprKind, Money, Perspective, Removal, Stake, Type,
};
use support::{acme, example, render_diagnostics};

fn lower(case: &Case) -> Result<LoweredCase, String> {
    let packages = [example()];
    let refs: Vec<_> = packages.iter().collect();
    Lowerer::new(&refs)
        .case(case, "cases/examples/acme.case")
        .map_err(|d| render_diagnostics(&d))
}

fn errors(case: &Case) -> String {
    lower(case)
        .err()
        .unwrap_or_else(|| panic!("expected case diagnostics"))
}

#[test]
fn the_perspective_must_be_a_party() {
    let mut case = acme();
    case.perspective = Some(Perspective::Party("nobody".into()));
    insta::assert_snapshot!(errors(&case));
}

#[test]
fn a_stake_needs_a_currency_with_a_lowering() {
    let mut case = acme();
    let terminal = "frcp-rule12-example::dismissed".to_owned();
    case.stakes.insert(
        terminal,
        Stake::Value(Money {
            minor_units: 5,
            currency: "jpy".into(),
        }),
    );
    insta::assert_snapshot!(errors(&case));
}

#[test]
fn a_named_scenario_needs_an_id() {
    let mut case = acme();
    case.id = String::new();
    insta::assert_snapshot!(errors(&case));
}

/// An unvalued stake is left to the pack's payoff.
#[test]
fn an_unvalued_stake_has_no_payoff() {
    let mut case = acme();
    let terminal = "frcp-rule12-example::judgment-entered";
    case.stakes.insert(terminal.into(), Stake::Unvalued);
    let lowered = lower(&case).unwrap_or_else(|d| panic!("{d}"));
    let payoffs = &lowered.scenario.scenario.payoffs;
    assert!(!payoffs.contains_key(terminal), "{payoffs:?}");
    assert_eq!(payoffs.len(), 1);
}

/// A removal's source-map entry is located at its event's action; a
/// repeated edge points at its one `remove_edges` index, and an edge of a
/// pack the workspace loads is cited under that pack's syntax.
#[test]
fn removals_are_located_at_their_events() {
    let span = Span {
        file: FileId("cases/examples/acme.case".into()),
        line: 30,
        col: 3,
        end_line: 30,
        end_col: 40,
    };
    let event = Event {
        date: "2025-03-10".into(),
        action: Expr {
            ty: Type::Named("MovedToDismiss".into()),
            kind: ExprKind::Ref {
                target: "frcp-rule12-example::MovedToDismiss".into(),
            },
            span: Some(span.clone()),
        },
        relation: EventParty::By,
        party: "widget".into(),
        authority: Some("FRCP 12(h)(1)".into()),
    };
    let removal = |edge: &str, event: usize| Removal {
        edge: edge.into(),
        date: "2025-03-10".into(),
        event,
        authority: "FRCP 12(h)(1)".into(),
    };
    let mut case = acme();
    case.events = vec![event];
    case.removals = vec![
        removal("frcp-rule12-example::default", 0),
        removal("frcp-rule12-example::default", 1),
    ];
    let lowered = lower(&case).unwrap_or_else(|d| panic!("{d}"));
    assert_eq!(
        lowered.scenario.scenario.remove_edges,
        ["frcp-rule12-example::default"]
    );
    let entries: Vec<_> = lowered.source_map.entries.iter().skip(1).collect();
    let located: Vec<(&str, Option<&Span>)> = entries
        .iter()
        .map(|e| (e.pointer.as_str(), e.span.as_ref()))
        .collect();
    assert_eq!(
        located,
        [
            ("/scenario/remove_edges/0", Some(&span)),
            ("/scenario/remove_edges/0", None),
        ]
    );
    let cite = entries[0].cite.as_ref().map(|c| c.text.as_str());
    assert_eq!(cite, Some("FRCP 12(h)(1)"));
}
