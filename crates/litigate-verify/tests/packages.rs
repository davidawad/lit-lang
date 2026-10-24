//! Package shapes beyond one rule package bound to the FRCP: a links
//! package, a dependent package counting with an imported rule, other
//! source bindings and citation forms, and edge windows that are not duty
//! deadlines.
mod support;

use litigate_base::Profile;
use litigate_verify::{Obligation, ObligationStatus};
use serde_json::json;
use support::{
    edit, links_pkg, outcome, pkg, pkg_with_sources, verify, verify_one, with, BASE, FRCP,
};

fn status(c: &litigate_verify::Checked, name: Obligation) -> Option<ObligationStatus> {
    c.verification.report.finding(name).map(|f| f.status)
}

#[test]
fn a_links_package_runs_only_its_tests() {
    let links = "module links\n\
                 link go \"Go\": demo::served -> demo::answered by Defendant\n";
    let c = verify(
        &[
            pkg("demo", "FederalRule", &[], BASE),
            links_pkg("links", &["demo"], links),
        ],
        Profile::Strict,
    );
    assert!(c.lowered.is_some());
    assert!(c.litigatable().is_none(), "a link file is not a pack");
    assert!(c.verification.is_accepted());
    let statuses: Vec<(Obligation, ObligationStatus)> = c
        .verification
        .report
        .obligations
        .iter()
        .map(|f| (f.name, f.status))
        .collect();
    insta::assert_json_snapshot!(json!({
        "statuses": statuses,
        "graph-exits": c.verification.report.finding(Obligation::GraphExits),
    }));
}

#[test]
fn a_dependent_package_counts_with_an_imported_rule() {
    // `local` tests its own deadline, counted by `demo`'s `frcp6`: the
    // counting rule is found in `demo` and counts as tested.
    let local = "module local.main\n\
        import demo.main.{frcp6, Served, Document, ServiceMethod}\n\
        start l\n\
        state l: \"L\" { _ \"Go\" -> t }\n\
        terminal t: \"T\" { payoff 0.usd }\n\
        rule duty local-duty cite \"FRCP 12(a)\"\n{\n  deadline local-period: 14.days from \
        Served(complaint) counted_by frcp6\n}\n\
        test \"local period by mail\" cite \"FRCP 6(d)\"\n{\n  \
        given Served(complaint, Mail) on 2025-03-03\n  \
        expect deadline(local-period) == 2025-03-20\n}\n";
    let c = verify(
        &[
            pkg("demo", "FederalRule", &[], BASE),
            pkg("local", "LocalRule", &["demo"], local),
        ],
        Profile::Strict,
    );
    assert!(c.verification.is_accepted(), "{:#?}", c.verification);
    assert_eq!(
        status(&c, Obligation::TestsPass),
        Some(ObligationStatus::Passed)
    );
    assert_eq!(
        status(&c, Obligation::RuleWithoutTest),
        Some(ObligationStatus::Passed)
    );
}

const URL_ONLY: &str = "[sources.local]\ntitle = \"Local Rules\"\n\
                        url = \"https://example.org/local-rules\"\n\
                        cite_prefixes = [\"LR\"]\nsection_heading = \"Local Rule {n}\"\n";

const UNVENDORED: &str = "[sources.local]\ntitle = \"Local Rules\"\n\
                          url = \"https://example.org/local-rules\"\n\
                          path = \"sources/local-rules.txt\"\n\
                          cite_prefixes = [\"LR\"]\nsection_heading = \"Local Rule {n}\"\n";

/// `BASE` with `served` cited as `cite`, under `sources` plus the FRCP.
fn cited(sources: &str, cite: &str) -> litigate_verify::Checked {
    let lit = edit(
        "by Defendant cite \"FRCP 12(a)\"",
        &format!("by Defendant cite \"{cite}\""),
    );
    verify(
        &[pkg_with_sources(
            "demo",
            "FederalRule",
            &[],
            &format!("{FRCP}\n{sources}"),
            &lit,
        )],
        Profile::Strict,
    )
}

#[test]
fn a_binding_without_a_vendored_path_resolves_by_its_prefix() {
    let c = cited(URL_ONLY, "LR 7");
    assert_eq!(
        status(&c, Obligation::CiteResolves),
        Some(ObligationStatus::Passed)
    );
}

#[test]
fn cite_resolves_rejects_a_path_outside_the_corpus() {
    let c = cited(UNVENDORED, "LR 7");
    insta::assert_json_snapshot!(outcome(&c, Obligation::CiteResolves));
}

#[test]
fn cite_resolves_rejects_each_cite_with_no_heading() {
    // Both pieces are reported at the same span with the same code; the
    // message orders them.
    let c = verify_one(
        &edit(
            "by Defendant cite \"FRCP 12(a)\"",
            "by Defendant cite \"0 U.S.C. § 102; 0 U.S.C. § 101\"",
        ),
        Profile::Strict,
    );
    insta::assert_json_snapshot!(outcome(&c, Obligation::CiteResolves));
}

#[test]
fn text_with_no_rule_shape_is_not_a_rule_cite() {
    let c = verify_one(
        &edit(
            "by Defendant cite \"FRCP 12(a)\"",
            "by Defendant cite \"FRCP 12(a); Standing Order 12\"",
        ),
        Profile::Strict,
    );
    assert_eq!(
        status(&c, Obligation::CiteResolves),
        Some(ObligationStatus::Passed)
    );
}

#[test]
fn a_package_sema_rejects_has_no_lowering_or_litigatable() {
    let c = verify_one(&edit("== 2025-03-24", "== 21.days"), Profile::Strict);
    assert!(c.lowered.is_none());
    assert!(c.litigatable().is_none());
}

#[test]
fn cite_resolves_checks_edge_authorities() {
    let lit = edit(
        "{ event Answered(complaint), within answer-period }",
        "{ event Answered(complaint), within answer-period, authority cite \"FRCP 99(a)\" }",
    );
    let c = verify_one(&lit, Profile::Strict);
    insta::assert_json_snapshot!(outcome(&c, Obligation::CiteResolves));
}

/// The cite of a `basis sourced(..)` on an edge or a node resolves like
/// any other (design review 2026-10-23 §2.10).
#[test]
fn cite_resolves_checks_sourced_bases() {
    let lit = edit(
        "{ event Answered(complaint), within answer-period }",
        "{ event Answered(complaint), within answer-period, basis sourced(\"FRCP 98(a)\") }",
    )
    .replacen(
        "{ payoff 0.usd }",
        "{ payoff 0.usd, basis sourced(\"FRCP 97(a)\") }",
        1,
    )
    .replacen(
        "{\n  answer \"File",
        "{\n  basis estimate\n  answer \"File",
        1,
    );
    let c = verify_one(&lit, Profile::Strict);
    let found = outcome(&c, Obligation::CiteResolves);
    let text = serde_json::to_string(&found).unwrap_or_default();
    assert!(
        text.contains("FRCP 98(a)") && text.contains("FRCP 97(a)"),
        "{text}"
    );
}

#[test]
fn only_duty_deadlines_constrain_a_node() {
    // An inline window and a top-level deadline beside a `power` rule:
    // neither is a duty, so neither is checked for satisfiability.
    let lit = with(
        "deadline free-period: 30.days from Served(complaint) counted_by frcp6\n\
         test \"free period\" cite \"FRCP 6(a)\"\n{\n  given Served(complaint, Personal) on 2025-03-03\n  \
         expect deadline(free-period) == 2025-04-02\n}\n\
         rule power move-duty cite \"FRCP 12(a)\"\n{\n  decision m: \"Move?\" by Defendant \
         { tags [entry], _ \"Yes\" -> answered }\n}",
    )
    .replacen(
        "within answer-period }",
        "within answer-period }\n  late \"Late answer\" -> answered { deadline 30.days }\n  \
         later \"Later answer\" -> answered { within free-period }",
        1,
    );
    let c = verify_one(&lit, Profile::Strict);
    assert!(c.verification.is_accepted(), "{:#?}", c.verification);
    assert_eq!(
        status(&c, Obligation::DeadlineSatisfiable),
        Some(ObligationStatus::Passed)
    );
}
