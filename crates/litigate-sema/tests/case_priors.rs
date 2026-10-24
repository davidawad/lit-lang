//! `beliefs { prior node: { edge: α } }` (SPEC §5.6, L4006; design review
//! 2026-10-23 §2.11): the Dirichlet prior a matter's observations update,
//! over a world draw, with a positive pseudo-count for every outcome.
mod case_support;

use case_support::{acme, check_2009, check_example, frcp2009_with, lock, matter_2009, render};
use litigate_base::{DiagnosticCode as C, Profile};
use litigate_sema::hir::Rational;

/// `acme.case` with `beliefs` replaced by `beliefs`, and `analysis` by
/// `analysis` when given.
fn with_beliefs(beliefs: &str, analysis: Option<&str>) -> String {
    let observe = "beliefs { observe mtd-ruling: { granted: 3, denied: 1 } }";
    let mut text = acme().replacen(observe, beliefs, 1);
    if let Some(analysis) = analysis {
        let start = text.find("analysis {").unwrap_or(text.len());
        text.replace_range(start.., analysis);
    }
    assert!(text.contains(beliefs));
    text
}

fn belief_errors(text: &str) -> Vec<String> {
    let checked = check_example(text, Profile::Dev);
    let errors: Vec<_> = checked
        .diagnostics
        .into_iter()
        .filter(|d| matches!(d.code, C::InvalidBelief | C::TypeMismatch))
        .collect();
    render(&errors)
}

#[test]
fn a_prior_over_every_outcome_resolves_to_pseudo_counts() {
    let text = with_beliefs(
        "beliefs {\n  prior mtd-ruling: { granted: 1.5, denied: 4 }\n  observe mtd-ruling: { granted: 3, denied: 1 }\n}",
        None,
    );
    let checked = check_example(&text, Profile::Dev);
    assert!(
        belief_errors(&text).is_empty(),
        "{:#?}",
        checked.diagnostics
    );
    let case = checked.case.unwrap_or_else(|| panic!("resolves"));
    let prior = &case.priors[0];
    assert_eq!(prior.node, "frcp-rule12-example::mtd-ruling");
    let q = |n, d| Rational {
        numerator: n,
        denominator: d,
    };
    assert_eq!(
        prior.pseudo_counts,
        [
            ("frcp-rule12-example::granted".to_owned(), q(3, 2)),
            ("frcp-rule12-example::denied".to_owned(), q(4, 1)),
        ]
    );
    assert_eq!(case.observations.len(), 1);
}

#[test]
fn a_prior_is_checked_like_an_observation_and_more() {
    let rows = [
        (
            "two priors on one node",
            "beliefs {\n  prior mtd-ruling: { granted: 1, denied: 1 }\n  prior mtd-ruling: { granted: 1, denied: 1 }\n}",
            None,
        ),
        (
            "a node nobody draws",
            "beliefs { prior respond: { answer: 1, move-dismiss: 1, default: 1 } }",
            None,
        ),
        (
            "outcomes that are not the node's, given twice, or left out",
            "beliefs { prior mtd-ruling: { granted: 1, granted: 2, barred: 1 } }",
            None,
        ),
        (
            "counts that are not positive numbers",
            "beliefs { prior mtd-ruling: { granted: 0, denied: -1, x: \"a\" } }",
            None,
        ),
        (
            "pseudo-counts that overflow",
            "beliefs { prior mtd-ruling: { granted: 9223372036854775807, denied: 1 } }",
            None,
        ),
        (
            "a strength given twice",
            "beliefs { prior mtd-ruling: { granted: 1, denied: 1 } }",
            Some("analysis { uncertainty: { concentration: { mtd-ruling: 4 } } }\n"),
        ),
        (
            "an unresolved node",
            "beliefs { prior nope: { granted: 1 } }",
            None,
        ),
    ];
    let table: Vec<(&str, Vec<String>)> = rows
        .iter()
        .map(|(name, beliefs, analysis)| (*name, belief_errors(&with_beliefs(beliefs, *analysis))))
        .collect();
    insta::assert_debug_snapshot!(table);
}

/// An anonymous outcome cannot be named, so a prior over its node cannot
/// be total.
#[test]
fn an_anonymous_outcome_cannot_be_given_a_pseudo_count() {
    let rule12 = include_str!("fixtures/case-2009/rule12.lit").replacen(
        "granted \"Motion granted\"",
        "_ \"Motion granted\"",
        1,
    );
    let packages = frcp2009_with(&rule12);
    let text = matter_2009(&[]) + "beliefs { prior mtd-ruling: { denied: 1 } }\n";
    let lock = lock(&packages);
    let checked = case_support::check_with(
        &text,
        &packages,
        Some(&lock),
        &case_support::clock(&packages),
        Profile::Dev,
    );
    let errors: Vec<String> = render(&checked.diagnostics);
    assert!(
        errors
            .iter()
            .any(|e| e.contains("gives no pseudo-count to `_ -> dismissed`")),
        "{errors:#?}"
    );
    // The fixture as shipped has both outcomes named.
    let named = check_2009(
        &(matter_2009(&[]) + "beliefs { prior mtd-ruling: { granted: 1, denied: 1 } }\n"),
    );
    assert!(
        named.diagnostics.iter().all(|d| d.code != C::InvalidBelief),
        "{:#?}",
        named.diagnostics
    );
}
