//! Extension refinements within their delegation bounds (SPEC §3.5;
//! design review §2.2 tier 3, §2.4): an event that `extends d to p` and
//! refines `p` claims every extension it records is within `d`'s
//! `extend` bounds, and the SMT tier checks the claim for every matter at
//! once (`well-typed`, L2013 with a counterexample).
mod support;

use litigate_base::Profile;
use litigate_verify::{Obligation, ObligationStatus};
use support::{edit, outcome, verify_one};

/// `BASE` with the answer period extendable by the court up to 30 days
/// past its prescribed due date, or 14 days past the order, whichever is
/// later, and an order event refined by `refinement`.
fn program(refinement: &str) -> String {
    with_params(&format!("until: Date{refinement}"))
}

/// [`program`] with the order event's parameters `params`.
fn with_params(params: &str) -> String {
    let lit = edit(
        "  deadline answer-period: 21.days from Served(complaint) counted_by frcp6\n",
        "  deadline answer-period: 21.days from Served(complaint) counted_by frcp6 {\n    \
         extend { by Court, to max(prescribed_due + 30.days, grant_order_entered + 14.days) }\n  }\n",
    );
    lit.replace(
        "event Answered(document: Document)\n",
        &format!(
            "event Answered(document: Document)\nevent Extended({params}) extends answer-period to until\n"
        ),
    )
}

fn found(lit: &str) -> (Option<ObligationStatus>, Vec<String>) {
    let checked = verify_one(lit, Profile::Strict);
    let found = outcome(&checked, Obligation::WellTyped);
    let messages = found
        .diagnostics
        .iter()
        .map(|d| {
            format!(
                "{}:{} {} {}",
                d.span.line,
                d.span.col,
                d.code.as_str(),
                d.message
            )
        })
        .collect();
    (found.finding.map(|f| f.status), messages)
}

#[test]
fn a_refinement_entailing_the_bound_passes() {
    for refinement in [
        " where until <= prescribed_due + 30.days",
        " where until <= grant_order_entered + 14.days && until > current_due",
        " where until <= prescribed_due + 10.days || until <= grant_order_entered",
    ] {
        assert_eq!(
            found(&program(refinement)),
            (Some(ObligationStatus::Passed), vec![]),
            "{refinement}"
        );
    }
}

#[test]
fn an_unrefined_extension_claims_nothing() {
    assert_eq!(
        found(&program("")),
        (Some(ObligationStatus::Passed), vec![])
    );
}

#[test]
fn a_refinement_admitting_an_extension_past_the_bound_is_a_counterexample() {
    let (status, messages) = found(&program(" where until <= prescribed_due + 45.days"));
    assert_eq!(status, Some(ObligationStatus::Failed));
    // The model is the solver's (pinned and deterministic): a matter where
    // every date coincides and the order reaches 31 days out.
    assert_eq!(
        messages,
        ["9:33 L2013 `Extended` admits an extension of `answer-period` past its bound: when \
          prescribed_due = 1970-01-01, grant_order_entered = 1970-01-01, current_due = 1970-01-01, \
          until = 1970-02-01 exceeds max(prescribed_due + 30.days, grant_order_entered + 14.days) = 1970-01-31"]
    );
}

#[test]
fn an_undecided_bound_is_a_hole() {
    let (status, messages) = found(&program(
        " where until <= prescribed_due + 30.days || (until - prescribed_due) * 9_223_372_036_854_775_807 * 2 > 0.days",
    ));
    // Undecided is a hole: an error in strict (sema's own L2015 on the
    // event is aggregated beside it).
    assert_eq!(status, Some(ObligationStatus::Failed));
    assert_eq!(
        messages,
        [
            "9:33 L2015 the SMT tier did not decide whether `Extended` stays within the bound of `answer-period`: exact arithmetic overflowed i64"
        ]
    );
}

/// The counterexample names every value the claim reads, in its type's
/// spelling: an enum's variant, a `Bool`, a number.
#[test]
fn a_counterexample_spells_each_value_by_its_type() {
    let (status, messages) = found(&with_params(
        "until: Date where until <= prescribed_due + 30.days || (consented && how == Mail && odds > 1/2), \
         how: ServiceMethod, consented: Bool, odds: Number",
    ));
    assert_eq!(status, Some(ObligationStatus::Failed));
    assert_eq!(messages.len(), 1, "{messages:?}");
    for value in ["how = Mail", "consented = true", "odds = "] {
        assert!(messages[0].contains(value), "{value}: {messages:?}");
    }
}

/// A discretionary extension has no ceiling, so a refinement cannot exceed
/// it; a refinement of the event's other parameters claims nothing about
/// the new due date.
#[test]
fn only_a_bounded_deadline_and_a_refined_due_date_are_checked() {
    let discretionary = program(" where until <= prescribed_due + 45.days").replace(
        "to max(prescribed_due + 30.days, grant_order_entered + 14.days)",
        "discretion GoodCause",
    );
    assert_eq!(
        found(&discretionary),
        (Some(ObligationStatus::Passed), vec![])
    );
    let other = with_params("until: Date, days: Number where days <= 45");
    assert_eq!(found(&other), (Some(ObligationStatus::Passed), vec![]));
}

/// `min` is a conjunction of ceilings and `x - N.days` counts back: an
/// early order lowers the bound below what the refinement admits.
#[test]
fn a_min_bound_counts_every_ceiling() {
    let lit = program(" where until <= prescribed_due + 20.days").replace(
        "to max(prescribed_due + 30.days, grant_order_entered + 14.days)",
        "to min(prescribed_due + 20.days, grant_order_entered - 1.days)",
    );
    let (status, messages) = found(&lit);
    assert_eq!(status, Some(ObligationStatus::Failed));
    assert_eq!(messages.len(), 1, "{messages:?}");
    assert!(
        messages[0]
            .contains("exceeds min(prescribed_due + 20.days, grant_order_entered - 1.days) = "),
        "{messages:?}"
    );
    let entailed = lit.replace(
        "where until <= prescribed_due + 20.days",
        "where until <= prescribed_due + 20.days && until < grant_order_entered",
    );
    assert_eq!(found(&entailed), (Some(ObligationStatus::Passed), vec![]));
}
