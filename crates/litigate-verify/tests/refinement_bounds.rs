//! Extension refinements within their delegation bounds (SPEC §3.5;
//! design review §2.2 tier 3, §2.4): an event that `extends d to p` and
//! refines `p` claims every extension it records is within `d`'s
//! `extend` bounds, and the SMT tier checks the claim for every matter at
//! once (`well-typed`, L2012 with a counterexample).
mod support;

use litigate_base::Profile;
use litigate_verify::{Obligation, ObligationStatus};
use support::{edit, outcome, verify_one};

/// `BASE` with the answer period extendable by the court up to 30 days
/// past its prescribed due date, or 14 days past the order, whichever is
/// later, and an order event refined by `refinement`.
fn program(refinement: &str) -> String {
    let lit = edit(
        "  deadline answer-period: 21.days from Served(complaint) counted_by frcp6\n",
        "  deadline answer-period: 21.days from Served(complaint) counted_by frcp6 {\n    \
         extend { by Court, to max(prescribed_due + 30.days, grant_order_entered + 14.days) }\n  }\n",
    );
    lit.replace(
        "event Answered(document: Document)\n",
        &format!(
            "event Answered(document: Document)\nevent Extended(until: Date{refinement}) extends answer-period to until\n"
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
        ["9:33 L2012 `Extended` admits an extension of `answer-period` past its bound: when \
          prescribed_due = 1970-01-01, grant_order_entered = 1970-01-01, current_due = 1970-01-01, \
          until = 1970-02-01 exceeds max(prescribed_due + 30.days, grant_order_entered + 14.days) = 1970-01-31"]
    );
}
