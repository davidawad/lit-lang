//! `beliefs` observe world draws (SPEC §5.6, L4006): a `chance` node, or a
//! `decision` whose every edge is taken by an actor playing `nature` under
//! the matter's roles, which is litgraph's own rule for `observe`.
mod case_support;

use case_support::{acme, check_example};
use litigate_base::{DiagnosticCode as C, Profile};

/// `acme.case` observing `respond` (a `decision by Defendant`) under
/// `perspective`, with `parties` replaced when given.
fn observing_respond(perspective: &str) -> String {
    let text = acme()
        .replacen("perspective widget", perspective, 1)
        .replacen(
            "observe mtd-ruling: { granted: 3, denied: 1 }",
            "observe respond: { answer: 3, move-dismiss: 1 }",
            1,
        );
    assert!(text.contains("observe respond"));
    text
}

fn belief_errors(text: &str) -> usize {
    check_example(text, Profile::Dev)
        .diagnostics
        .iter()
        .filter(|d| d.code == C::InvalidBelief)
        .count()
}

#[test]
fn a_chance_node_is_observable() {
    assert_eq!(belief_errors(&acme()), 0);
}

#[test]
fn a_decision_every_actor_of_which_plays_nature_is_observable() {
    let text = observing_respond("perspective { Defendant: nature }");
    let checked = check_example(&text, Profile::Dev);
    assert!(
        checked
            .diagnostics
            .iter()
            .all(|d| d.code != C::InvalidBelief),
        "{:#?}",
        checked.diagnostics
    );
}

#[test]
fn a_decision_someone_in_the_matter_takes_is_not() {
    for perspective in [
        // widget plays Defendant: its own choice.
        "perspective widget",
        // acme is self, so the other party's actor is the opponent.
        "perspective acme",
        // The package role: Defendant is the opponent.
        "perspective { Plaintiff: self }",
        "perspective { Defendant: opponent }",
    ] {
        let text = observing_respond(perspective);
        assert_eq!(belief_errors(&text), 1, "{perspective}");
    }
}

#[test]
fn the_rejection_says_what_can_be_observed() {
    let checked = check_example(&observing_respond("perspective widget"), Profile::Dev);
    let d = checked
        .diagnostics
        .iter()
        .find(|d| d.code == C::InvalidBelief)
        .unwrap_or_else(|| panic!("L4006"));
    assert!(d
        .notes
        .iter()
        .any(|n| n.message.contains("actor playing nature")));
}
