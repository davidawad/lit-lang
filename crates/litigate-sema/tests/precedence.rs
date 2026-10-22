//! Authority, declared relations, canons, delegation, exceptions, rule
//! kinds, and deadline direction (SPEC §5.5), one small program per rule.
mod legal;
use legal::{pkg, render, table, with_world, workspace, CORE};
use litigate_sema::pkg::load::LoadedPackage;
use litigate_sema::precedence::{conclude, winner, Conclusion, Lattice};
use litigate_sema::temporal::Key;
use std::collections::BTreeMap;

const START: &str = "module core.start\nstart z\nstate z: \"Z\"\n";

fn core(extra: &str) -> LoadedPackage {
    let chain = format!("{CORE}{extra}");
    pkg(
        "core",
        "Statute",
        "1990-01-01",
        &[],
        &[("authority.lit", &chain), ("start.lit", START)],
    )
}

/// `frcp`: a FederalRule answer period; `clauses` go on the deadline.
fn frcp(clauses: &str) -> LoadedPackage {
    let text = format!(
        "module frcp.rules\nimport core.authority.{{Statute, LocalRule, StandingOrder, CaseOrder}}\nstart s\nenum Document {{ complaint }}\nevent Served(document: Document)\ncounting c6 {{ holidays cal }}\ncalendar cal {{ new-year: fixed(1, 1) effective [1870-06-28, ..) }}\nstate s: \"S\"\ndeadline answer-period: 21.days from Served(complaint) counted_by c6 effective [2009-12-01, ..){clauses}\n"
    );
    pkg(
        "frcp",
        "FederalRule",
        "2009-12-01",
        &["core"],
        &[("rules.lit", &text)],
    )
}

/// A package `name` of `authority` whose `body` may override frcp's rules.
fn local(name: &str, authority: &str, body: &str) -> LoadedPackage {
    local_from(name, authority, "2009-12-01", body)
}

/// [`local`] with a package lifetime starting `in_force`.
fn local_from(name: &str, authority: &str, in_force: &str, body: &str) -> LoadedPackage {
    let text = format!(
        "module {name}.rules\nimport frcp.rules.{{answer-period, c6}}\nstart l\nenum Paper {{ complaint }}\nevent Filed(paper: Paper)\nstate l: \"L\"\n{body}\n"
    );
    pkg(
        name,
        authority,
        in_force,
        &["core", "frcp"],
        &[("rules.lit", &text)],
    )
}

fn override_decl(id: &str, clauses: &str) -> String {
    format!("deadline {id}: 14.days from Filed(complaint) counted_by c6 {clauses}")
}

fn run(packages: &[LoadedPackage]) -> Vec<String> {
    let (db, ws) = workspace(packages);
    render(&db, ws)
}

#[test]
fn overrides_need_delegation_and_conflicts_need_an_order() {
    let ov = |id: &str, extra: &str| {
        override_decl(
            id,
            &format!("effective [2012-01-01, ..) override answer-period{extra}"),
        )
    };
    let delegated = " delegable_to [LocalRule, StandingOrder]";
    let cases: Vec<(&str, Vec<LoadedPackage>)> = vec![
        (
            "01 override without delegation",
            vec![
                core(""),
                frcp(""),
                local("local", "LocalRule", &ov("local-answer", "")),
            ],
        ),
        (
            "02 delegated override",
            vec![
                core(""),
                frcp(delegated),
                local("local", "LocalRule", &ov("local-answer", "")),
            ],
        ),
        (
            "03 two overrides at one level tie",
            vec![
                core(""),
                frcp(delegated),
                local("local", "LocalRule", &(ov("a", "") + "\n" + &ov("b", ""))),
            ],
        ),
        (
            "04 notwithstanding orders them",
            vec![
                core(""),
                frcp(delegated),
                local(
                    "local",
                    "LocalRule",
                    &(ov("a", "") + "\n" + &ov("b", " notwithstanding a")),
                ),
            ],
        ),
        (
            "05 higher authority wins",
            vec![
                core(""),
                frcp(delegated),
                local("local", "LocalRule", &ov("a", "")),
                local("standing", "StandingOrder", &ov("b", "")),
            ],
        ),
        (
            "06 inside a rule beats package level",
            vec![
                core(""),
                frcp(delegated),
                local(
                    "local",
                    "LocalRule",
                    &(ov("a", "") + "\nrule duty d {\n  " + &ov("b", "") + "\n}"),
                ),
            ],
        ),
        (
            "07 narrower window wins",
            vec![
                core(""),
                frcp(delegated),
                local(
                    "local",
                    "LocalRule",
                    &(ov("a", "")
                        + "\n"
                        + &override_decl(
                            "b",
                            "effective [2012-01-01, 2013-01-01) override answer-period",
                        )),
                ),
            ],
        ),
        (
            "08 later from wins",
            vec![
                core(""),
                frcp(delegated),
                local(
                    "local",
                    "LocalRule",
                    &(ov("a", "")
                        + "\n"
                        + &override_decl("b", "effective [2014-01-01, ..) override answer-period")),
                ),
            ],
        ),
        (
            "09 delegation must point downward",
            vec![core(""), frcp(" delegable_to [Statute]")],
        ),
        (
            "10 override of a rule not yet in force",
            vec![
                core(""),
                frcp(delegated),
                local_from(
                    "local",
                    "LocalRule",
                    "2005-01-01",
                    &override_decl(
                        "a",
                        "effective [2005-01-01, 2006-01-01) override answer-period",
                    ),
                ),
            ],
        ),
        (
            "11 authority cycle",
            vec![core("authority CaseOrder > Statute\n"), frcp("")],
        ),
    ];
    let got: BTreeMap<&str, Vec<String>> = cases.iter().map(|(n, p)| (*n, run(p))).collect();
    insta::assert_debug_snapshot!(got);
}

fn key(pack: &str, name: &str) -> Key {
    Key {
        pack: pack.into(),
        name: name.into(),
    }
}

/// Who prevails over `frcp::answer-period` on each of `dates`.
fn winners(packages: &[LoadedPackage], dates: &[&str]) -> Vec<String> {
    let (db, ws) = workspace(packages);
    assert_eq!(render(&db, ws), Vec::<String>::new());
    with_world(&db, ws, |world| {
        let lattice = Lattice::new(&world.ranks);
        let t = key("frcp", "answer-period");
        dates
            .iter()
            .map(|d| match winner(world, &lattice, &t, d) {
                Ok(w) => format!("{d}: {}::{}", world.pack_of(w), world.decls[w].meta.id),
                Err(pairs) => format!("{d}: unresolved {pairs:?}"),
            })
            .collect()
    })
}

#[test]
fn the_winner_follows_relations_then_canons() {
    let delegated = " delegable_to [LocalRule, StandingOrder]";
    let ov = |id: &str, window: &str| {
        override_decl(id, &format!("effective {window} override answer-period"))
    };
    let by_authority = [
        core(""),
        frcp(delegated),
        local(
            "local",
            "LocalRule",
            &ov("local-answer", "[2012-01-01, ..)"),
        ),
        local(
            "standing",
            "StandingOrder",
            &ov("standing-answer", "[2011-01-01, ..)"),
        ),
    ];
    assert_eq!(
        winners(&by_authority, &["2010-06-01", "2011-06-01", "2012-06-01"]),
        [
            "2010-06-01: frcp::answer-period",
            "2011-06-01: standing::standing-answer",
            "2012-06-01: local::local-answer"
        ]
    );
    let specific = [
        core(""),
        frcp(delegated),
        local(
            "local",
            "LocalRule",
            &[
                ov("wide", "[2012-01-01, ..)"),
                ov("narrow", "[2013-01-01, 2014-01-01)"),
                format!(
                    "rule duty d {{\n  {}\n}}",
                    ov("nested", "[2015-01-01, 2016-01-01)")
                ),
                ov("wider-later", "[2017-01-01, ..)"),
            ]
            .join("\n"),
        ),
    ];
    assert_eq!(
        winners(
            &specific,
            &["2012-06-01", "2013-06-01", "2015-06-01", "2018-01-01"]
        ),
        [
            "2012-06-01: local::wide",
            "2013-06-01: local::narrow",
            "2015-06-01: local::nested",
            "2018-01-01: local::wider-later"
        ]
    );
}

#[test]
fn relation_cycles_are_rejected() {
    let d = |id: &str, rel: &str| {
        format!("deadline {id}: 1.days from Served(complaint) counted_by c6 {rel}\n")
    };
    insta::assert_debug_snapshot!(table(&[
        ("1 notwithstanding both ways", &(d("a", "notwithstanding b") + &d("b", "notwithstanding a"))),
        ("2 subject_to agrees with notwithstanding", &(d("a", "notwithstanding b") + &d("b", "subject_to a"))),
        ("3 supersession cycle", &(d("a", "supersedes b") + &d("b", "supersedes a"))),
        (
            "4 exceptions targeting each other",
            "default d when true => 1\nexception e1 rebuts e2 when true => 2\nexception e2 rebuts e1 when true => 3",
        ),
    ]));
}

const DEFAULT: &str = "default d when has_event(Served(complaint, Personal)) => 21.days\n";

#[test]
fn exceptions_are_disjoint_or_prioritized() {
    let e = |id: &str, rest: &str| format!("exception {id} {rest}\n");
    let ordered = DEFAULT.to_owned()
        + &e(
            "e1",
            "rebuts d when has_event(Ordered()) => 7.days priority 2",
        )
        + &e(
            "e2",
            "undercuts d when has_event(Answered(complaint)) priority 1",
        );
    insta::assert_debug_snapshot!(table(&[
        (
            "1 overlapping and unordered",
            &(DEFAULT.to_owned()
                + &e("e1", "rebuts d when has_event(Ordered()) => 7.days")
                + &e("e2", "undercuts d when has_event(Answered(complaint))")),
        ),
        (
            "2 provably disjoint",
            &(DEFAULT.to_owned()
                + &e("e1", "rebuts d when has_event(Ordered()) => 7.days")
                + &e("e2", "undercuts d when !has_event(Ordered()) && has_event(Answered(complaint))")),
        ),
        ("3 distinct priorities", &ordered),
        (
            "4 equal priorities",
            &(DEFAULT.to_owned()
                + &e("e1", "rebuts d when has_event(Ordered()) => 7.days priority 1")
                + &e("e2", "undercuts d when has_event(Answered(complaint)) priority 1")),
        ),
        (
            "5 exception outside its target's force",
            "default d effective [2012-01-01, ..) when has_event(Served(complaint, Personal)) => 21.days\n\
             rule duty old effective [2009-12-01, 2011-01-01) {\n\
               deadline z: 1.days from Served(complaint) counted_by c6\n\
               exception e1 rebuts d when true => 7.days\n\
             }",
        ),
    ]));
}

#[test]
fn rebutting_concludes_and_undercutting_only_disables() {
    let body = DEFAULT.to_owned()
        + "exception shorter rebuts d when has_event(Ordered()) => 7.days priority 2\n\
           exception moot undercuts d when has_event(Answered(complaint)) priority 1\n\
           exception unless-shortened undercuts shorter when has_event(Hearing(motion))\n";
    let text = format!("{}{body}\n", legal::PRELUDE);
    let (db, ws) = workspace(&[pkg(
        "demo",
        "FederalRule",
        "2009-12-01",
        &[],
        &[("main.lit", &text)],
    )]);
    assert_eq!(render(&db, ws), Vec::<String>::new());
    with_world(&db, ws, |world| {
        let k = key("demo", "d");
        let outcome = |facts: &[&str]| {
            let mut atom = |e: &litigate_sema::hir::Expr| {
                let p = litigate_sema::temporal::condition::has_event(e)?;
                let litigate_sema::hir::ExprKind::Call { callee, .. } = &p.kind else {
                    return None;
                };
                Some(facts.contains(&callee.as_str()))
            };
            match conclude(world, &k, "2012-01-01", &mut atom) {
                Ok(Conclusion::Value { value, by }) => {
                    format!("{:?} via {}", value.kind, by.join(" > "))
                }
                Ok(Conclusion::Undercut { by }) => format!("disabled via {}", by.join(" > ")),
                Ok(Conclusion::NotApplicable) => "not applicable".to_owned(),
                Err(e) => e,
            }
        };
        insta::assert_debug_snapshot!([
            outcome(&[]),
            outcome(&["Served"]),
            outcome(&["Served", "Ordered"]),
            outcome(&["Served", "Answered"]),
            outcome(&["Served", "Ordered", "Answered"]),
            outcome(&["Served", "Ordered", "Hearing"]),
            outcome(&["Served", "Ordered", "Hearing", "Answered"]),
        ]);
    });
}

#[test]
fn rule_kinds_have_their_shapes() {
    let deadline = "deadline p: 21.days from Served(complaint) counted_by c6";
    insta::assert_debug_snapshot!(table(&[
        ("1 duty with its deadline", &format!("rule duty answer {{\n  {deadline}\n}}")),
        ("2 duty without a deadline", "rule duty answer {\n  state x: \"X\" { _ \"go\" -> t }\n}"),
        ("3 power granting a decision", "rule power move {\n  decision m: \"Move?\" by Defendant { _ \"yes\" -> t }\n}"),
        ("4 power without a decision", &format!("rule power move {{\n  {deadline}\n}}")),
        ("5 liability with consequences", "rule liability defaulted {\n  state dr: \"Default\" { _ \"entered\" -> t { sets [defaulted] } }\n}"),
        (
            "6 liability imposing a deadline",
            &format!("rule liability defaulted {{\n  state dr: \"Default\" {{ _ \"entered\" -> t }}\n  {deadline}\n}}"),
        ),
    ]));
}

#[test]
fn deadline_direction_is_part_of_its_type() {
    let before = "deadline brief: 14.days before Hearing(motion) counted_by c6";
    insta::assert_debug_snapshot!(table(&[
        ("1 backward deadline on its own", before),
        ("2 within a backward deadline", &format!("{before}\nstate w: \"W\" {{ file \"File\" -> t {{ within brief }} }}")),
        (
            "3 versions change direction",
            "deadline q: 14.days before Hearing(motion) counted_by c6 effective [2009-12-01, 2016-12-01)\n\
             deadline q: 14.days from Hearing(motion) counted_by c6 effective [2016-12-01, ..)",
        ),
        (
            "4 backward with a next-business-day anchor",
            "counting itc { anchor next_business_day, holidays cal }\ndeadline r: 7.days before Hearing(motion) counted_by itc",
        ),
    ]));
}
