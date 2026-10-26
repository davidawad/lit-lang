//! One minimal passing and one minimal failing program per SPEC §7
//! obligation, checked in `strict`. Each snapshot holds the obligation's
//! finding and the diagnostics verification emitted for it; findings that
//! come from sema carry its diagnostic in their examples.
mod support;

use litigate_base::Profile;
use litigate_sema::pkg::load::LoadedPackage;
use litigate_verify::{Obligation, ObligationStatus};
use serde_json::json;
use support::{edit, outcome, pkg, verify, with, BASE};

fn one(lit: &str) -> Vec<LoadedPackage> {
    vec![pkg("demo", "FederalRule", &[], lit)]
}

/// Snapshots `name`'s outcome for both programs and checks the verdicts.
fn pair(name: Obligation, pass: &[LoadedPackage], fail: &[LoadedPackage]) -> serde_json::Value {
    let passed = verify(pass, Profile::Strict);
    let failed = verify(fail, Profile::Strict);
    let (p, f) = (outcome(&passed, name), outcome(&failed, name));
    assert_eq!(
        p.finding.map(|x| x.status),
        Some(ObligationStatus::Passed),
        "{name:?} passing program: {p:#?}"
    );
    assert_eq!(
        f.finding.map(|x| x.status),
        Some(ObligationStatus::Failed),
        "{name:?} failing program: {f:#?}"
    );
    assert!(
        !f.accepted,
        "{name:?}: a failed obligation rejects the program"
    );
    json!({ "pass": p, "fail": f })
}

macro_rules! obligation {
    ($test:ident, $name:expr, $pass:expr, $fail:expr) => {
        #[test]
        fn $test() {
            insta::assert_json_snapshot!(pair($name, &$pass, &$fail));
        }
    };
}

obligation!(
    well_typed,
    Obligation::WellTyped,
    one(BASE),
    one(&edit("== 2025-03-24", "== 21.days"))
);

obligation!(
    cite_resolves_to_a_vendored_heading,
    Obligation::CiteResolves,
    one(BASE),
    one(&edit(
        "by Defendant cite \"FRCP 12(a)\"",
        "by Defendant cite \"FRCP 99(a)\""
    ))
);

obligation!(
    cite_resolves_through_a_binding,
    Obligation::CiteResolves,
    one(BASE),
    one(&edit(
        "by Defendant cite \"FRCP 12(a)\"",
        "by Defendant cite \"35 U.S.C. § 101\""
    ))
);

/// A deadline delegating, characterized, and reached by a grant, each
/// citing `cite`.
fn capability_cites(cite: &str) -> String {
    with(&format!(
        "deadline reply-period: 14.days from Answered(complaint) counted_by frcp6 delegable {{ to LocalRule, op set, cite \"{cite}\" }} cite \"FRCP 12(a)\"\n\
         {{\n  characterization claim_processing cite \"{cite}\"\n}}\n\
         deadline appeal-period: 30.days from Answered(complaint) counted_by frcp6 cite \"FRCP 12(a)\"\n\
         {{\n  characterization jurisdictional\n}}\n\
         grant emergency on [reply-period] effective [2024-03-15, 2024-06-13) cite \"FRCP 87(c)(2)\" {{ delegable {{ to LocalRule, op extend, cite \"{cite}\" }}, delegable {{ to LocalRule, op shorten }} }}"
    ))
}

obligation!(
    cite_resolves_for_capabilities,
    Obligation::CiteResolves,
    one(&capability_cites("FRCP 6(b)")),
    one(&capability_cites("FRCP 99(b)"))
);

/// `demo` and a later package recharacterizing and invalidating its
/// answer period, each citing `cite`.
fn change_cites(cite: &str) -> Vec<LoadedPackage> {
    let later = format!(
        "module later.main\n\
         import demo.main.{{answer-period}}\n\
         start l\n\
         terminal l: \"L\" {{ payoff 0.usd }}\n\
         recharacterize held: answer-period characterization claim_processing cite \"{cite}\"\n\
         invalidate gone: answer-period cite \"{cite}\"\n"
    );
    vec![
        pkg("demo", "FederalRule", &[], BASE),
        pkg("later", "FederalRule", &["demo"], &later),
    ]
}

obligation!(
    cite_resolves_for_recharacterizations_and_invalidations,
    Obligation::CiteResolves,
    change_cites("FRCP 6(b)"),
    change_cites("FRCP 99(b)")
);

/// An `entry` island that routes to two versions of `late`.
fn versions(first: &str, second: &str) -> String {
    with(&format!(
        "terminal late: \"Late\" effective {first} {{ payoff 0.usd }}\n\
         terminal late: \"Late\" effective {second} {{ payoff 0.usd }}\n\
         state island: \"Island\" {{ tags [entry], _ \"Go\" -> late }}"
    ))
}

obligation!(
    effective_disjoint,
    Obligation::EffectiveDisjoint,
    one(&versions("[1990-01-01, 2000-01-01)", "[2000-01-01, ..)")),
    one(&versions("[1990-01-01, ..)", "[2000-01-01, ..)"))
);

obligation!(
    effective_covered,
    Obligation::EffectiveCovered,
    one(&versions("[1990-01-01, 2000-01-01)", "[2000-01-01, ..)")),
    one(&versions("[1990-01-01, 2000-01-01)", "[2001-01-01, ..)"))
);

obligation!(
    precedence_acyclic,
    Obligation::PrecedenceAcyclic,
    one(BASE),
    one(&with("authority LocalRule > Statute"))
);

/// `demo` and a `LocalRule` package overriding its answer period.
fn delegation(delegable: &str) -> Vec<LoadedPackage> {
    let local = "module local.main\n\
        import demo.main.{answer-period, frcp6, Served, Document}\n\
        start l\n\
        state l: \"L\" { _ \"Go\" -> t }\n\
        terminal t: \"T\" { payoff 0.usd }\n\
        deadline local-period: 14.days from Served(complaint) counted_by frcp6 \
        effective [2012-01-01, ..) override answer-period\n";
    vec![
        pkg(
            "demo",
            "FederalRule",
            &[],
            &edit(
                "counted_by frcp6\n",
                &format!("counted_by frcp6{delegable}\n"),
            ),
        ),
        pkg("local", "LocalRule", &["demo"], local),
    ]
}

obligation!(
    delegation_respected,
    Obligation::DelegationRespected,
    delegation(" delegable { to LocalRule, op set }"),
    delegation("")
);

const DEFAULT: &str = "default d when has_event(Served(complaint, Personal)) => 21.days";

obligation!(
    exceptions_ordered,
    Obligation::ExceptionsOrdered,
    one(&with(&format!(
        "{DEFAULT}\nexception e1 rebuts d when has_event(Answered(complaint)) => 7.days priority 2\n\
         exception e2 undercuts d when has_event(Answered(complaint)) priority 1"
    ))),
    one(&with(&format!(
        "{DEFAULT}\nexception e1 rebuts d when has_event(Answered(complaint)) => 7.days\n\
         exception e2 undercuts d when has_event(Answered(complaint))"
    )))
);

fn chance(edges: &str) -> String {
    with(&format!(
        "chance odds: \"Odds\" by Court {{ tags [entry]{edges} }}"
    ))
}

obligation!(
    dist_exact,
    Obligation::DistExact,
    one(&chance(
        ", a \"A\" -> answered p 1/2, b \"B\" -> answered p 1/2"
    )),
    one(&chance(
        ", a \"A\" -> answered p 1/2, b \"B\" -> answered p 1/3"
    ))
);

obligation!(
    dist_exact_rejects_an_empty_distribution,
    Obligation::DistExact,
    one(&chance(", a \"A\" -> answered p 1")),
    one(&chance(""))
);

/// `served` gains an edge to `next`, declared as `decl`.
fn onward(decl: &str) -> String {
    let edge = "within answer-period }";
    with(decl).replacen(edge, &format!("{edge}\n  onward \"Go on\" -> next"), 1)
}

obligation!(
    graph_exits,
    Obligation::GraphExits,
    one(&onward("state next: \"Next\" { _ \"Finish\" -> answered }")),
    one(&onward("state next: \"Next\""))
);

obligation!(
    graph_reachable,
    Obligation::GraphReachable,
    one(&with(
        "state island: \"Island\" { tags [entry], _ \"Go\" -> lone }\n\
         terminal lone: \"Lone\" { payoff 0.usd }"
    )),
    one(&with(
        "state island: \"Island\" { _ \"Go\" -> lone }\n\
         terminal lone: \"Lone\" { payoff 0.usd }"
    ))
);

/// `answer-period` in force from `from`, and a second duty in force
/// before 2000 bounding another edge of `served`.
fn two_duties(from: &str) -> String {
    let old = "\nrule duty old-duty effective [1990-01-01, 2000-01-01) subject_to answer-duty \
        cite \"FRCP 12(a)\"\n{\n  deadline old-period: 20.days from Served(complaint) counted_by \
        frcp6 effective [1990-01-01, 2000-01-01) subject_to answer-period\n}\n\
        test \"old answer period\" {\n  given Served(complaint, Personal) on 1995-03-03\n  \
        expect deadline(old-period) == 1995-03-23\n}";
    with(old)
        .replacen(
            "counted_by frcp6\n",
            &format!("counted_by frcp6 effective [{from}, ..)\n"),
            1,
        )
        .replacen(
            "within answer-period }",
            "within answer-period }\n  old \"Answer (old rule)\" -> answered { within old-period }",
            1,
        )
}

#[test]
fn deadline_satisfiable() {
    // Both programs fail to lower at the default `as_of` (L3007: the old
    // duty is repealed by then); verification still checks the HIR. Only
    // the second has no day on which both duties are in force.
    insta::assert_json_snapshot!(pair(
        Obligation::DeadlineSatisfiable,
        &one(&two_duties("1990-01-01")),
        &one(&two_duties("2000-01-01")),
    ));
}

obligation!(
    tests_pass,
    Obligation::TestsPass,
    one(BASE),
    one(&edit("== 2025-03-27", "== 2025-03-24"))
);

obligation!(
    tests_pass_rejects_a_test_without_its_trigger,
    Obligation::TestsPass,
    one(BASE),
    one(&edit(
        "given Served(complaint, Mail) on 2025-03-03",
        "given Answered(complaint) on 2025-03-03"
    ))
);

obligation!(
    holes,
    Obligation::Holes,
    one(BASE),
    one(&edit("payoff 0.usd", "payoff ?(\"no estimate yet\")"))
);

obligation!(
    rule_without_test,
    Obligation::RuleWithoutTest,
    one(BASE),
    one(&with(
        "rule duty reply-duty cite \"FRCP 12(a)\"\n{\n  deadline reply-period: 14.days from \
         Answered(complaint) counted_by frcp6\n}"
    ))
);

#[test]
fn a_test_cite_alone_does_not_test_a_rule() {
    let cited_only = edit(
        "expect deadline(answer-period) == 2025-03-24",
        "expect has_event(Served(complaint, Personal)) == true",
    )
    .replace(
        "expect deadline(answer-period) == 2025-03-27",
        "expect has_event(Served(complaint, Mail)) == true",
    );
    let checked = verify(&one(&cited_only), Profile::Strict);
    let found = outcome(&checked, Obligation::RuleWithoutTest);
    let codes: Vec<&str> = found.diagnostics.iter().map(|d| d.code.as_str()).collect();
    assert_eq!(codes, ["L9001", "L9001"], "{found:#?}");
    let tests = checked.verification.report.finding(Obligation::TestsPass);
    assert_eq!(tests.map(|f| f.status), Some(ObligationStatus::Passed));
}
