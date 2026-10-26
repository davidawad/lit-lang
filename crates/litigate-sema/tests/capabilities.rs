//! Typed delegation capabilities (SPEC §5.3, §5.5; design review
//! 2026-10-23 §2.4): `delegable { to L, op O, max N.unit, discretion S,
//! on M() }` licenses a lower authority's displacing claim only for the
//! operations it grants and within its bound, for a closed cause standard
//! and on an event motion (`cov_case_procedure.rs` replays it); an emergency `grant` adds capabilities for a
//! bounded window and can lift a lock; a jurisdictional deadline admits
//! no waiver, no unbounded extension and no delegation. Each obligation
//! moved into the type checker has a negative test here.
mod common;
mod legal;

use common::check_body;
use legal::{pkg, render, workspace, CORE};
use litigate_sema::hir::Package;
use litigate_sema::pkg::load::LoadedPackage;
use litigate_sema::precedence::delegation::{license, operations, Refusal};
use litigate_sema::precedence::{select, Unselected};
use litigate_sema::temporal::{Key, World};
use std::collections::BTreeMap;

const START: &str = "module core.start\nstart z\nstate z: \"Z\"\n";

fn core() -> LoadedPackage {
    let chain = format!("{CORE}authority Statute > Regulation > CaseOrder\n");
    pkg(
        "core",
        "Statute",
        "1990-01-01",
        &[],
        &[("authority.lit", &chain), ("start.lit", START)],
    )
}

/// `fed` (`FederalRule`): `answer-period` (21 calendar days) with
/// `clauses` and `opts`, and `extra` declarations.
fn fed(clauses: &str, opts: &str, extra: &str) -> LoadedPackage {
    let text = format!(
        "module fed.rules
import core.authority.{{Statute, LocalRule, StandingOrder, CaseOrder, Regulation}}
start s
enum Document {{ complaint, motion }}
event Served(document: Document)
event Answered(document: Document)
counting c6 {{ holidays cal }}
counting c7 {{ holidays cal }}
calendar cal {{ new-year: fixed(1, 1) effective [1870-06-28, ..) }}
deadline answer-period: 21.days from Served() counted_by c6 effective [2009-12-01, ..){clauses} {{ {opts} }}
state s: \"S\" by Plaintiff {{ serve \"Serve\" -> r {{ event Served(complaint) }} }}
decision r: \"R\" by Defendant {{ answer \"Answer\" -> done {{ event Answered(complaint), within answer-period }} }}
terminal done: \"Done\" {{ payoff 0.usd }}
{extra}
"
    );
    pkg(
        "fed",
        "FederalRule",
        "2009-12-01",
        &["core"],
        &[("rules.lit", &text)],
    )
}

/// A package `name` of `authority` over `fed` declaring `body`.
fn over(name: &str, authority: &str, body: &str) -> LoadedPackage {
    let text = format!(
        "module {name}.rules
import core.authority.{{CaseOrder}}
import fed.rules.{{answer-period, c6, c7, Served, Document}}
start l
state l: \"L\"
{body}
"
    );
    pkg(
        name,
        authority,
        "2009-12-01",
        &["core", "fed"],
        &[("rules.lit", &text)],
    )
}

fn diagnostics(packages: &[LoadedPackage]) -> Vec<String> {
    let (db, ws) = workspace(packages);
    render(&db, ws)
}

fn compile(packages: &[LoadedPackage]) -> Vec<Package> {
    let (db, ws) = workspace(packages);
    litigate_sema::check_workspace(&db, ws)
        .into_iter()
        .filter_map(|c| c.package.clone())
        .collect()
}

/// Who governs `fed::answer-period` on 2012-06-01 among `hir`.
fn who(hir: &[Package]) -> String {
    let refs: Vec<&Package> = hir.iter().collect();
    let world = World::from_packages(&refs);
    let key = Key {
        pack: "fed".into(),
        name: "answer-period".into(),
    };
    match select(&world, &key, "2012-06-01", None) {
        Ok(w) => format!("{}::{}", world.pack_of(w), world.decls[w].meta.id),
        Err(Unselected::Unresolved(pairs)) => format!("unresolved {}", pairs.len()),
        Err(Unselected::Inactive(_)) => "inactive".to_owned(),
    }
}

/// A local rule overriding `answer-period` with `decl` (a deadline header
/// tail after `local-answer: `).
fn local(decl: &str) -> LoadedPackage {
    local_with(decl, "")
}

/// [`local`] with deadline options `opts`.
fn local_with(decl: &str, opts: &str) -> LoadedPackage {
    over(
        "local",
        "LocalRule",
        &format!("deadline local-answer: {decl} override answer-period {{ {opts} }}\n"),
    )
}

#[test]
#[allow(clippy::too_many_lines)] // one table row per case
fn a_claim_performs_the_operations_its_value_amounts_to() {
    // Each row: the delegation, the local rule's claim, and whether the
    // claim governs (licensed) or `answer-period` still does.
    let rows: &[(&str, &str, &str)] = &[
        // A longer period is an extension.
        ("op extend", "30.days from Served() counted_by c6", "local"),
        ("op shorten", "30.days from Served() counted_by c6", "fed"),
        ("op set", "30.days from Served() counted_by c6", "local"),
        // A shorter one is a shortening, which `extend` does not cover.
        ("op shorten", "14.days from Served() counted_by c6", "local"),
        ("op extend", "14.days from Served() counted_by c6", "fed"),
        // Another counting rule is a change of method; `set` does not cover it.
        (
            "op compute_method",
            "21.days from Served() counted_by c7",
            "local",
        ),
        ("op set", "21.days from Served() counted_by c7", "fed"),
        // Another unit or trigger replaces the value.
        (
            "op set",
            "21.court_days from Served() counted_by c6",
            "local",
        ),
        (
            "op extend",
            "21.court_days from Served() counted_by c6",
            "fed",
        ),
        (
            "op set",
            "21.days from Served(motion) counted_by c6",
            "local",
        ),
        (
            "op [extend, shorten]",
            "21.days from Served(motion) counted_by c6",
            "fed",
        ),
        // A claim that changes nothing needs only some capability.
        ("op shorten", "21.days from Served() counted_by c6", "local"),
        // A bound caps an extension past the declared period.
        (
            "op extend, max 9.days",
            "30.days from Served() counted_by c6",
            "local",
        ),
        (
            "op extend, max 8.days",
            "30.days from Served() counted_by c6",
            "fed",
        ),
        (
            "op set, max 8.days",
            "14.days from Served() counted_by c6",
            "local",
        ),
        // Units never convert: a calendar-day bound admits no court days.
        (
            "op set, max 30.days",
            "21.court_days from Served() counted_by c6",
            "fed",
        ),
        // Two capabilities: one bounded too low, one unbounded.
        (
            "op extend, max 1.days } delegable { to LocalRule, op extend",
            "30.days from Served() counted_by c6",
            "local",
        ),
    ];
    let mut out = BTreeMap::new();
    for (i, (cap, claim, expect)) in rows.iter().enumerate() {
        let clause = format!(" delegable {{ to LocalRule, {cap} }}");
        let packages = [core(), fed(&clause, "", ""), local(claim)];
        let found: Vec<String> = diagnostics(&packages)
            .into_iter()
            .filter(|d| !d.contains(" L9"))
            .collect();
        let governs = who(&compile(&packages));
        let want = if *expect == "local" {
            "local::local-answer"
        } else {
            "fed::answer-period"
        };
        assert_eq!(governs, want, "{cap} / {claim}: {found:#?}");
        assert_eq!(
            found.is_empty(),
            *expect == "local",
            "{cap} / {claim}: {found:#?}"
        );
        out.insert(format!("{i:02} {cap} / {claim}"), found);
    }
    insta::assert_debug_snapshot!(out);
}

#[test]
#[allow(clippy::too_many_lines)] // one fixture, one assertion per operation
fn operations_name_suspension_and_non_deadlines() {
    let packages = [
        core(),
        fed(
            " delegable { to LocalRule, op suspend }",
            "",
            "state stage: \"Stage\" delegable { to LocalRule, op set }\n\
             counting c8 delegable { to LocalRule, op compute_method } { holidays cal }\n",
        ),
        pkg(
            "local",
            "LocalRule",
            "2009-12-01",
            &["core", "fed"],
            &[(
                "rules.lit",
                "module local.rules\n\
                 import fed.rules.{answer-period, c6, c8, Served, stage}\n\
                 start l2\n\
                 deadline pause: 3.days from Served() counted_by c6 effective [2011-01-01, ..) supersedes answer-period\n\
                 counting l8 override c8 { holidays lcal }\n\
                 calendar lcal { new-year: fixed(1, 1) effective [1870-06-28, ..) }\n\
                 state l2: \"L2\" override stage\n",
            )],
        ),
    ];
    let found: Vec<String> = diagnostics(&packages)
        .into_iter()
        .filter(|d| !d.contains(" L9"))
        .collect();
    assert_eq!(found, Vec::<String>::new());
    let mut hir = compile(&packages);
    let names = |ops: Vec<litigate_sema::hir::DelegatedOp>| -> Vec<&str> {
        ops.into_iter()
            .map(litigate_sema::hir::DelegatedOp::name)
            .collect()
    };
    {
        let refs: Vec<&Package> = hir.iter().collect();
        let world = World::from_packages(&refs);
        let find = |pack: &str, id: &str| {
            (0..world.decls.len())
                .find(|&i| world.pack_of(i) == pack && world.decls[i].meta.id == id)
                .unwrap_or_else(|| panic!("{pack}::{id}"))
        };
        let (pause, period) = (find("local", "pause"), find("fed", "answer-period"));
        assert_eq!(names(operations(&world, pause, period, true)), ["suspend"]);
        assert_eq!(license(&world, pause, period, true), Ok(()));
        let (l8, c8) = (find("local", "l8"), find("fed", "c8"));
        assert_eq!(names(operations(&world, l8, c8, false)), ["compute_method"]);
        assert_eq!(license(&world, l8, c8, false), Ok(()));
        let (l2, stage) = (find("local", "l2"), find("fed", "stage"));
        assert_eq!(names(operations(&world, l2, stage, false)), ["set"]);
        assert_eq!(license(&world, l2, stage, false), Ok(()));
    }
    // Without its delegation, the counting rule refuses the claim.
    for p in hir.iter_mut().filter(|p| p.manifest.name == "fed") {
        for d in p.modules.iter_mut().flat_map(|m| m.declarations.iter_mut()) {
            if let litigate_sema::hir::Declaration::Counting(c) = d {
                c.meta.delegations.clear();
            }
        }
    }
    let refs: Vec<&Package> = hir.iter().collect();
    let world = World::from_packages(&refs);
    let find = |pack: &str, id: &str| {
        (0..world.decls.len())
            .find(|&i| world.pack_of(i) == pack && world.decls[i].meta.id == id)
            .unwrap_or_else(|| panic!("{pack}::{id}"))
    };
    assert_eq!(
        license(&world, find("local", "l8"), find("fed", "c8"), false),
        Err(Refusal::Missing)
    );
    // A bound admits only a deadline claim.
    let found = diagnostics(&[
        core(),
        fed("", "", "state stage: \"Stage\" delegable { to LocalRule, op set, max 3.days }\n"),
        pkg(
            "local",
            "LocalRule",
            "2009-12-01",
            &["core", "fed"],
            &[(
                "rules.lit",
                "module local.rules\nimport fed.rules.{stage}\nstart l2\nstate l2: \"L2\" override stage\n",
            )],
        ),
    ]);
    assert!(
        found
            .iter()
            .any(|d| d.contains("L3004") && d.contains("beyond its delegation's bound")),
        "{found:#?}"
    );
}

#[test]
fn delegable_clauses_report_each_bad_value() {
    let out = check_body(
        "authority FederalRule > LocalRule > CaseOrder\n\
deadline a: 21.days from Served(complaint) counted_by c6 delegable_to [CaseOrder]\n\
deadline b: 21.days from Served(complaint) counted_by c6 delegable_to [LocalRule, CaseOrder]\n\
deadline c: 21.days from Served(complaint) counted_by c6 delegable { op set }\n\
deadline d: 21.days from Served(complaint) counted_by c6 delegable { to CaseOrder }\n\
deadline e: 21.days from Served(complaint) counted_by c6 delegable { to CaseOrder, op stretch }\n\
deadline f: 21.days from Served(complaint) counted_by c6 delegable { to CaseOrder, op shorten, max 3.days }\n\
deadline g: 21.days from Served(complaint) counted_by c6 delegable { to Nobody, op set }\n\
deadline h: 21.days from Served(complaint) counted_by c6 delegable { to CaseOrder, op [extend, extend], cite \"FRCP 6(b)\" }\n\
deadline i: 21.days from Served(complaint) counted_by c6 delegable { to [LocalRule, CaseOrder], op extend, max 3.days, cite ?(\"no text\") }\n\
deadline k: 21.days from Served(complaint) counted_by c6 delegable { to CaseOrder, op set, to LocalRule }\n\
deadline l: 21.days from Served(complaint) counted_by c6 delegable { to, op set }",
    );
    insta::assert_debug_snapshot!(out);
    // A delegation must point downward (checked once the module builds).
    let upward = check_body(
        "authority FederalRule > LocalRule > CaseOrder\n\
deadline j: 21.days from Served(complaint) counted_by c6 delegable { to FederalRule, op set }",
    );
    assert!(
        upward.iter().any(|d| d.contains(
            "L3004 `j` (FederalRule) delegates to `FederalRule`, which is not a lower authority"
        )),
        "{upward:#?}"
    );
}

#[test]
fn a_delegation_names_a_cause_standard_and_an_event_motion() {
    let out = check_body(
        "authority FederalRule > LocalRule > CaseOrder\n\
deadline a: 21.days from Served(complaint) counted_by c6 delegable { to CaseOrder, op extend, discretion Whim }\n\
deadline b: 21.days from Served(complaint) counted_by c6 delegable { to CaseOrder, op extend, on Moved() }\n\
deadline c: 21.days from Served(complaint) counted_by c6 delegable { to CaseOrder, op extend, on max(1) }\n\
deadline d: 21.days from Served(complaint) counted_by c6 delegable { to CaseOrder, op extend, discretion GoodCause, on Served(answer) }\n\
deadline e: 21.days from Served(complaint) counted_by c6 delegable { to CaseOrder, op extend, discretion }\n\
deadline f: 21.days from Served(complaint) counted_by c6 delegable { to CaseOrder, op extend, on }",
    );
    insta::assert_debug_snapshot!(out);
}

#[test]
fn a_jurisdictional_deadline_admits_no_waiver_unbounded_extension_or_delegation() {
    let out = check_body(
        "authority FederalRule > CaseOrder\n\
deadline a: 30.days from Served(complaint) counted_by c6 { characterization jurisdictional cite \"28 U.S.C. § 2107(a)\", waive { by Court } }\n\
deadline b: 30.days from Served(complaint) counted_by c6 { characterization jurisdictional, extend { by Court, discretion GoodCause } }\n\
deadline c: 30.days from Served(complaint) counted_by c6 delegable { to CaseOrder, op extend } { characterization jurisdictional }\n\
deadline d: 30.days from Served(complaint) counted_by c6 { characterization jurisdictional, extend { by Court, max 30.days }, extend { never cite \"FRAP 26(b)(1)\" } }\n\
deadline e: 30.days from Served(complaint) counted_by c6 { characterization claim_processing cite ?(\"Hamer, not vendored\"), waive { by Court }, extend { by Court, discretion GoodCause } }\n\
deadline f: 30.days from Served(complaint) counted_by c6 { characterization fuzzy }\n\
deadline g: 30.days from Served(complaint) counted_by c6 { characterization jurisdictional cite \"1\" }\n\
deadline h: 30.days from Served(complaint) counted_by c6 { characterization }",
    );
    insta::assert_debug_snapshot!(out);
}

#[test]
fn grants_report_each_bad_shape() {
    let out = check_body(
        "authority FederalRule > LocalRule > CaseOrder\n\
deadline a: 21.days from Served(complaint) counted_by c6 { extend { never cite \"FRCP 6(b)(2)\" } }\n\
deadline j: 21.days from Served(complaint) counted_by c6 { characterization jurisdictional }\n\
grant open-ended on [a] cite \"FRCP 87\" { unlock }\n\
grant empty on [a] effective [2020-03-15, 2020-06-13) {}\n\
grant not-a-deadline on [s] effective [2020-03-15, 2020-06-13) { unlock }\n\
grant twice on [a] effective [2020-03-15, 2020-06-13) { unlock, unlock }\n\
grant missing on [nope] effective [2020-03-15, 2020-06-13) { unlock }",
    );
    insta::assert_debug_snapshot!(out);
    // What a grant gives its targets, checked once the module builds.
    let world = check_body(
        "authority FederalRule > LocalRule > CaseOrder\n\
deadline a: 21.days from Served(complaint) counted_by c6 { extend { never cite \"FRCP 6(b)(2)\" } }\n\
deadline j: 21.days from Served(complaint) counted_by c6 { characterization jurisdictional }\n\
grant fine on [a] effective [2020-03-15, 2020-06-13) cite \"FRCP 87(c)(2)\" { unlock, extend { by Court, discretion GoodCause }, delegable { to CaseOrder, op extend }, delegable { to LocalRule, op set } }\n\
grant locked-out on [a] effective [2020-03-15, 2020-06-13) { extend { by Court, max 30.days } }\n\
grant upward on [a] effective [2020-03-15, 2020-06-13) { delegable { to FederalRule, op set } }\n\
grant forbidden on [j] effective [2020-03-15, 2020-06-13) { waive { by Court }, extend { by Court, discretion GoodCause }, delegable { to CaseOrder, op set } }\n\
grant waiver-only on [j] effective [2020-03-15, 2020-06-13) { waive { by Court } }\n\
grant bounded on [j] effective [2020-03-15, 2020-06-13) { extend { by Court, max 30.days } }\n\
grant before on [a] effective [2001-03-15, 2001-06-13) { delegable { to FederalRule, op set } }",
    );
    insta::assert_debug_snapshot!("grants_give_their_targets", world);
}

#[test]
fn an_emergency_grant_delegates_and_unlocks_only_inside_its_window() {
    let locked = "extend { by Court, max 30.days }, extend { never cite \"FRAP 26(b)(1)\" }";
    let grant = "grant emergency on [answer-period] effective [2020-03-15, 2020-06-13) cite \"FRCP 87(c)(2)\" { unlock, delegable { to LocalRule, op set } }\n";
    let claim = |window: &str| {
        local_with(
            &format!("30.days from Served() counted_by c6 effective {window}"),
            "extend { by Court, max 60.days }",
        )
    };
    let rows: &[(&str, &str, &str)] = &[
        ("inside the window", grant, "[2020-04-01, 2020-05-01)"),
        ("past the window", grant, "[2020-04-01, 2020-07-01)"),
        (
            "a grant without `unlock` delegates but leaves the lock",
            "grant emergency on [answer-period] effective [2020-03-15, 2020-06-13) { delegable { to LocalRule, op set } }\n",
            "[2020-04-01, 2020-05-01)",
        ),
    ];
    let mut out = BTreeMap::new();
    for (name, grant, window) in rows {
        let found = diagnostics(&[core(), fed("", locked, grant), claim(window)]);
        out.insert(*name, found);
    }
    insta::assert_debug_snapshot!(out);
}

#[test]
fn a_grant_never_widens_its_own_power() {
    // A local rule may not grant itself what the federal rule withholds.
    let found = diagnostics(&[
        core(),
        fed("", "", ""),
        over(
            "local",
            "LocalRule",
            "grant self-help on [answer-period] effective [2020-03-15, 2020-06-13) { delegable { to CaseOrder, op set } }\n\
             deadline local-answer: 30.days from Served() counted_by c6 effective [2020-04-01, 2020-05-01) override answer-period\n",
        ),
    ]);
    insta::assert_debug_snapshot!(found);
}

#[test]
fn any_other_difference_replaces_the_value() {
    // Another direction, bearer, trigger policy or waiver is a `set`,
    // which `extend` does not cover.
    let rows: &[(&str, &str)] = &[
        ("21.days before Served() counted_by c6", ""),
        ("21.days from Served() counted_by c6", "bearer Defendant"),
        ("21.days from Served() counted_by c6", "trigger latest"),
        ("21.days from Served() counted_by c6", "waive { by Court }"),
    ];
    for (decl, opts) in rows {
        for (cap, governs) in [
            ("op extend", "fed::answer-period"),
            ("op set", "local::local-answer"),
        ] {
            let clause = format!(" delegable {{ to LocalRule, {cap} }}");
            let packages = [core(), fed(&clause, "", ""), local_with(decl, opts)];
            let found: Vec<String> = diagnostics(&packages)
                .into_iter()
                .filter(|d| d.contains("L3004"))
                .collect();
            assert_eq!(
                who(&compile(&packages)),
                governs,
                "{decl} {opts} under {cap}: {found:#?}"
            );
            if cap == "op extend" {
                assert!(
                    found.len() == 1 && found[0].contains("by `set`"),
                    "{decl} {opts}: {found:#?}"
                );
            }
        }
    }
    // A claim that changes nothing, with no delegation at all, is told to
    // ask for `set`.
    let found = diagnostics(&[
        core(),
        fed("", "", ""),
        local("21.days from Served() counted_by c6"),
    ]);
    assert!(
        found
            .iter()
            .any(|d| d.contains("missing `delegable { to LocalRule, op set }`")),
        "{found:#?}"
    );
}

#[test]
fn a_bound_needs_the_same_unit_and_direction_on_both_sides() {
    let extra = "deadline cd: 21.court_days from Served() counted_by c6 effective [2009-12-01, ..) delegable { to LocalRule, op set, max 30.days }\n\
                 deadline bk: 21.days before Served() counted_by c6 effective [2009-12-01, ..) delegable { to LocalRule, op set, max 30.days }\n";
    let found = diagnostics(&[
        core(),
        pkg(
            "fed",
            "FederalRule",
            "2009-12-01",
            &["core"],
            &[(
                "rules.lit",
                &format!(
                    "module fed.rules\nimport core.authority.{{LocalRule}}\nstart s\nevent Served()\ncounting c6 {{ holidays cal }}\ncalendar cal {{ new-year: fixed(1, 1) effective [1870-06-28, ..) }}\n{extra}state s: \"S\"\n"
                ),
            )],
        ),
        pkg(
            "local",
            "LocalRule",
            "2009-12-01",
            &["core", "fed"],
            &[(
                "rules.lit",
                "module local.rules\nimport fed.rules.{cd, bk, c6, Served}\nstart l\nstate l: \"L\"\n\
                 deadline lc: 30.days from Served() counted_by c6 override cd\n\
                 deadline lb: 30.days from Served() counted_by c6 override bk\n",
            )],
        ),
    ]);
    let bounded: Vec<&String> = found
        .iter()
        .filter(|d| d.contains("beyond its delegation's bound"))
        .collect();
    assert_eq!(bounded.len(), 2, "{found:#?}");
}

#[test]
fn one_refusal_per_kind_whatever_the_versions() {
    // Two versions refuse the same claim for the same reason: one error.
    let extra = "deadline two: 21.days from Served() counted_by c6 effective [2009-12-01, 2015-01-01) delegable { to LocalRule, op shorten }\n\
                 deadline two: 21.days from Served() counted_by c6 effective [2015-01-01, ..) delegable { to LocalRule, op shorten }\n";
    let fed2 = pkg(
        "fed",
        "FederalRule",
        "2009-12-01",
        &["core"],
        &[(
            "rules.lit",
            &format!(
                "module fed.rules\nimport core.authority.{{LocalRule}}\nstart s\nevent Served()\ncounting c6 {{ holidays cal }}\ncalendar cal {{ new-year: fixed(1, 1) effective [1870-06-28, ..) }}\n{extra}state s: \"S\"\n"
            ),
        )],
    );
    let local2 = |body: &str| {
        pkg(
            "local",
            "LocalRule",
            "2009-12-01",
            &["core", "fed"],
            &[(
                "rules.lit",
                &format!("module local.rules\nimport fed.rules.{{two, c6, Served}}\nstart l\nstate l: \"L\"\n{body}"),
            )],
        )
    };
    let found = diagnostics(&[
        core(),
        fed2.clone(),
        local2("deadline lt: 30.days from Served() counted_by c6 override two\n"),
    ]);
    let refused: Vec<&String> = found.iter().filter(|d| d.contains("L3004")).collect();
    assert_eq!(refused.len(), 1, "{found:#?}");
    // A grant below both versions' authority: one error too.
    let found = diagnostics(&[
        core(),
        fed2,
        local2("grant g on [two] effective [2014-06-01, 2015-06-01) { unlock }\n"),
    ]);
    let refused: Vec<&String> = found.iter().filter(|d| d.contains("L3004")).collect();
    assert_eq!(refused.len(), 1, "{found:#?}");
}

#[test]
fn a_grant_is_not_a_counting_rule() {
    let found = check_body(
        "deadline a: 21.days from Served(complaint) counted_by c6\n\
grant g on [a] effective [2020-03-15, 2020-06-13) { unlock }\n\
deadline z: 1.days from Served(complaint) counted_by g",
    );
    assert!(
        found
            .iter()
            .any(|d| d.contains("L2001 `g` is a grant, expected a counting rule")),
        "{found:#?}"
    );
}

#[test]
fn grants_check_over_hir_without_spans_or_deadlines() {
    use litigate_sema::hir::Declaration;
    use litigate_sema::precedence::delegation::check;
    use litigate_sema::temporal::world::NoSource;
    let mut hir = compile(&[
        core(),
        fed(
            "",
            "",
            "grant g on [answer-period] effective [2020-03-15, 2020-06-13) { delegable { to CaseOrder, op set } }\n",
        ),
    ]);
    // `check` rejects an upward delegation; built HIR with no spans still
    // names the grant, and a grant reaching a node gives it nothing.
    for d in hir
        .iter_mut()
        .flat_map(|p| p.modules.iter_mut())
        .flat_map(|m| m.declarations.iter_mut())
    {
        if let Declaration::Grant(g) = d {
            g.meta.span = None;
            g.targets.push("s".into());
            g.delegations[0].to = "FederalRule".into();
        }
    }
    let refs: Vec<&Package> = hir.iter().collect();
    let world = World::from_packages(&refs);
    let pack = world
        .packs
        .iter()
        .position(|p| p.pack == "fed")
        .unwrap_or(0);
    let found: Vec<String> = check(&world, world.lattice(), &NoSource, pack)
        .into_iter()
        .map(|d| format!("{} {} {}", d.span.file.0, d.code.as_str(), d.message))
        .collect();
    assert_eq!(
        found,
        [
            "g L3004 grant `g` delegates `answer-period` (FederalRule) to `FederalRule`, which is not a lower authority",
            "g L3004 grant `g` delegates `s` (FederalRule) to `FederalRule`, which is not a lower authority",
        ]
    );
}

#[test]
fn named_locks_report_each_bad_shape() {
    let out = check_body(
        "authority FederalRule > LocalRule > CaseOrder\n\
deadline a: 21.days from Served(complaint) counted_by c6 { lock against [] cite \"FRAP 26(b)(1)\" }\n\
deadline b: 21.days from Served(complaint) counted_by c6 { lock against [LocalRule] }\n\
deadline c: 21.days from Served(complaint) counted_by c6 { extend { never cite \"FRCP 6(b)(2)\" }, lock against [LocalRule] cite \"FRAP 26(b)(1)\" }\n\
deadline d: 21.days from Served(complaint) counted_by c6 { lock against [Nobody, CaseOrder] cite \"FRAP 26(b)(1)\" }\n\
deadline e: 21.days from Served(complaint) counted_by c6 { lock against [CaseOrder] cite \"X\", lock against [LocalRule] cite \"Y\" }\n\
deadline f: 21.days from Served(complaint) counted_by c6 { lock [CaseOrder] cite \"X\" }\n\
deadline g: 21.days from Served(complaint) counted_by c6 { extendable true, lock against [CaseOrder] cite \"X\" }",
    );
    insta::assert_debug_snapshot!(out);
    // Outside a deadline's options `lock` is still reserved (L0007).
    let reserved = check_body("group lock: \"G\"");
    assert!(
        reserved.iter().any(|d| d.contains("L0007")),
        "{reserved:#?}"
    );
}

#[test]
fn a_named_lock_binds_the_deadlines_level_and_those_below() {
    let body = |against: &str| {
        check_body(&format!(
            "authority Statute > FederalRule > LocalRule > CaseOrder\n\
authority Statute > Regulation > CaseOrder\n\
deadline a: 21.days from Served(complaint) counted_by c6 {{ lock against {against} cite \"FRAP 26(b)(1)\" }}"
        ))
    };
    let rows: BTreeMap<&str, Vec<String>> = [
        (
            "its own level and lower ones",
            "[FederalRule, LocalRule, CaseOrder, CaseOrder]",
        ),
        ("a higher authority", "[Statute]"),
        ("an unranked one", "[Regulation]"),
    ]
    .into_iter()
    .map(|(name, against)| (name, body(against)))
    .collect();
    insta::assert_debug_snapshot!(rows);
}

#[test]
fn a_named_lock_refuses_only_the_authorities_it_names() {
    // FRAP 26(b)(1) binds the courts: a local rule the federal rule
    // delegates to may not add an extension a lock against it forbids,
    // and may when the lock names only case orders.
    let claim = local_with(
        "30.days from Served() counted_by c6 effective [2012-01-01, ..)",
        "extend { by Court, max 60.days }",
    );
    let rows: BTreeMap<&str, Vec<String>> = [
        ("against the local rule", "[LocalRule, CaseOrder]"),
        ("against case orders only", "[CaseOrder]"),
    ]
    .into_iter()
    .map(|(name, against)| {
        let locked = format!(
            "extend {{ by Court, max 30.days }}, lock against {against} cite \"FRAP 26(b)(1)\""
        );
        let fed = fed(" delegable { to LocalRule, op set }", &locked, "");
        (name, diagnostics(&[core(), fed, claim.clone()]))
    })
    .collect();
    insta::assert_debug_snapshot!(rows);
}

#[test]
fn an_emergency_grant_of_an_unnamed_authority_needs_no_unlock() {
    let rows: BTreeMap<&str, Vec<String>> = [
        ("a lock against lower authorities", "[LocalRule, CaseOrder]"),
        ("a lock against the grant's own level", "[FederalRule]"),
    ]
    .into_iter()
    .map(|(name, against)| {
        let body = format!(
            "authority FederalRule > LocalRule > CaseOrder\n\
deadline a: 21.days from Served(complaint) counted_by c6 {{ lock against {against} cite \"FRCP 6(b)(2)\" }}\n\
grant emergency on [a] effective [2020-03-15, 2020-06-13) cite \"FRCP 87(c)(2)\" {{ extend {{ by Court, max 30.days }} }}"
        );
        (name, check_body(&body))
    })
    .collect();
    insta::assert_debug_snapshot!(rows);
}
