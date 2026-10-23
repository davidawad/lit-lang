//! Precedence edge cases (SPEC §5.5) through whole packages: unranked and
//! cyclic claimants, delegation that cannot be repaired by a fix,
//! exceptions too large to prove disjoint, reference cycles through every
//! expression form, authority cycles, rule shapes, and package plumbing.
mod legal;
use legal::{pkg, render, table, with_world, workspace, CORE, PRELUDE};
use litigate_sema::pkg::load::{LoadedFile, LoadedPackage};
use litigate_sema::precedence::{shape, winner, Lattice};
use litigate_sema::temporal::Key;
use std::collections::BTreeMap;
use std::fmt::Write as _;

fn core() -> LoadedPackage {
    pkg(
        "core",
        "Statute",
        "1990-01-01",
        &[],
        &[
            ("authority.lit", CORE),
            ("start.lit", "module core.start\nstart z\nstate z: \"Z\"\n"),
        ],
    )
}

/// Package `name` of `authority` with its own start, `Served`, and `c6`,
/// importing `imports` from `deps`' `<dep>.rules`, then `body`.
fn rules(name: &str, authority: &str, deps: &[&str], imports: &str, body: &str) -> LoadedPackage {
    let mut text = format!("module {name}.rules\n");
    if deps.contains(&"core") {
        text.push_str(
            "import core.authority.{Statute, FederalRule, LocalRule, StandingOrder, CaseOrder}\n",
        );
    }
    for dep in deps.iter().filter(|d| **d != "core") {
        let _ = writeln!(text, "import {dep}.rules.{{{imports}}}");
    }
    let _ = write!(
        text,
        "start s\nenum Document {{ complaint }}\nevent Served(document: Document)\nstate s: \"S\"\n{body}\n"
    );
    pkg(name, authority, "2009-12-01", deps, &[("rules.lit", &text)])
}

const C6: &str = "counting c6 { holidays cal }\ncalendar cal { new-year: fixed(1, 1) effective [1870-06-28, ..) }\n";

fn d(id: &str, days: u32, clauses: &str) -> String {
    format!("deadline {id}: {days}.days from Served(complaint) counted_by c6 {clauses}\n")
}

fn run(packages: &[LoadedPackage]) -> Vec<String> {
    let (db, ws) = workspace(packages);
    render(&db, ws)
}

#[test]
fn claimants_across_packages_and_delegation_table() {
    let target = format!("{C6}{}", d("tg", 1, ""));
    let delegated = format!(
        "{C6}{}",
        d(
            "tg",
            1,
            "effective [2009-12-01, ..) delegable_to [LocalRule]"
        )
    );
    let cases: Vec<(&str, Vec<LoadedPackage>)> = vec![
        (
            "1 unranked authorities tie; a dependency's own tie is its own",
            vec![
                rules(
                    "x",
                    "Alpha",
                    &[],
                    "",
                    &(target.clone()
                        + &d("a", 2, "notwithstanding tg")
                        + &d("a2", 3, "notwithstanding tg")),
                ),
                rules(
                    "y",
                    "Beta",
                    &["x"],
                    "tg, c6",
                    &d("b", 4, "notwithstanding tg"),
                ),
            ],
        ),
        (
            "2 an unranked override needs a delegation the target cannot be fixed to give",
            vec![
                rules("x", "Alpha", &[], "", &target),
                rules("y", "Beta", &["x"], "tg, c6", &d("o", 4, "override tg")),
            ],
        ),
        (
            "3 delegating to one's own level",
            vec![
                core(),
                rules(
                    "x",
                    "FederalRule",
                    &["core"],
                    "",
                    &(C6.to_owned() + &d("tg", 1, "delegable_to [FederalRule]")),
                ),
            ],
        ),
        (
            "4 a versioned override of a delegating rule",
            vec![
                core(),
                rules("x", "FederalRule", &["core"], "", &delegated),
                rules(
                    "y",
                    "LocalRule",
                    &["core", "x"],
                    "tg, c6",
                    &d("o", 4, "override tg@2009-12-01"),
                ),
            ],
        ),
    ];
    let got: BTreeMap<&str, Vec<String>> = cases.iter().map(|(n, p)| (*n, run(p))).collect();
    insta::assert_debug_snapshot!(got);
}

#[test]
fn a_lower_authority_listed_first_loses_on_authority() {
    let delegated = format!(
        "{C6}{}",
        d(
            "tg",
            1,
            "effective [2009-12-01, ..) delegable_to [LocalRule, StandingOrder]"
        )
    );
    let packages = [
        core(),
        rules("frcp", "FederalRule", &["core"], "", &delegated),
        rules(
            "astanding",
            "StandingOrder",
            &["core", "frcp"],
            "tg, c6",
            &d("b", 5, "override tg"),
        ),
        rules(
            "local",
            "LocalRule",
            &["core", "frcp"],
            "tg, c6",
            &d("a", 4, "override tg"),
        ),
    ];
    let (db, ws) = workspace(&packages);
    assert_eq!(render(&db, ws), Vec::<String>::new());
    let got = with_world(&db, ws, |world| {
        let t = Key {
            pack: "frcp".into(),
            name: "tg".into(),
        };
        winner(world, &Lattice::new(&world.ranks), &t, "2013-01-01")
            .map(|w| format!("{}::{}", world.pack_of(w), world.decls[w].meta.id))
    });
    assert_eq!(got, Ok("local::a".to_owned()));
}

/// The world is a set of packages (design review §3.4): every order of
/// loading the same packages selects the same governing declaration.
#[test]
fn every_load_order_of_one_world_selects_alike() {
    let delegated = format!(
        "{C6}{}",
        d(
            "tg",
            1,
            "effective [2009-12-01, ..) delegable_to [LocalRule]"
        )
    );
    let packages = [
        core(),
        rules("frcp", "FederalRule", &["core"], "", &delegated),
        rules(
            "local",
            "LocalRule",
            &["core", "frcp"],
            "tg, c6",
            &d("a", 4, "override tg"),
        ),
    ];
    let (db, ws) = workspace(&packages);
    let checks = litigate_sema::check_workspace(&db, ws);
    let compiled: Vec<&litigate_sema::hir::Package> =
        checks.iter().filter_map(|c| c.package.as_ref()).collect();
    assert_eq!(compiled.len(), 3, "{:?}", render(&db, ws));
    let t = Key {
        pack: "frcp".into(),
        name: "tg".into(),
    };
    for order in [
        [0, 1, 2],
        [0, 2, 1],
        [1, 0, 2],
        [1, 2, 0],
        [2, 0, 1],
        [2, 1, 0],
    ] {
        let world = litigate_sema::temporal::World::from_packages(&order.map(|i| compiled[i]));
        let got = litigate_sema::precedence::select::select(&world, &t, "2013-01-01", None)
            .map(|w| format!("{}::{}", world.pack_of(w), world.decls[w].meta.id));
        assert_eq!(got, Ok("local::a".to_owned()), "{order:?}");
    }
}

#[test]
fn claimant_ties_and_cycles_table() {
    let tg = d("tg", 1, "");
    insta::assert_debug_snapshot!(table(&[
        (
            "1 claimants beating each other in a circle",
            &(tg.clone()
                + &d("a", 2, "notwithstanding tg")
                + &d("b", 3, "subject_to tg notwithstanding a")),
        ),
        (
            "2 a claimant whose first relation is not a claim",
            &(d("old", 1, "")
                + &tg
                + &d("a", 2, "supersedes old notwithstanding tg")
                + &d("b", 3, "notwithstanding tg")),
        ),
        (
            "3 both claimants inside one rule",
            &(tg.clone()
                + "rule duty r {\n  "
                + &d("a", 2, "notwithstanding tg")
                + "  "
                + &d("b", 3, "notwithstanding tg")
                + "}"),
        ),
        (
            "4 one tie across two target versions is reported once",
            &(d("tg", 1, "effective [2009-12-01, 2012-01-01)")
                + &d("tg", 1, "effective [2012-01-01, ..)")
                + &d("a", 2, "notwithstanding tg")
                + &d("b", 3, "notwithstanding tg")),
        ),
        (
            "5 delegating to an unranked level",
            &("authority Alpha > Beta\n".to_owned() + &d("o", 1, "delegable_to [Alpha]")),
        ),
    ]));
}

#[test]
fn exceptions_references_authority_and_shapes_table() {
    let events = (0..18).fold(String::new(), |mut s, i| {
        let _ = writeln!(s, "event E{i}()");
        s
    });
    let any = |range: std::ops::Range<u32>| {
        range
            .map(|i| format!("has_event(E{i}())"))
            .collect::<Vec<_>>()
            .join(" || ")
    };
    let large = format!(
        "{events}default d when true => 1\nexception e1 rebuts d when {} => 2\nexception e2 rebuts d when {} => 3",
        any(0..9),
        any(9..18)
    );
    let disjoint = "default d when true => 1\n\
        rule duty r effective [2009-12-01, 2012-01-01) {\n  deadline z: 1.days from Served(complaint) counted_by c6\n  exception e rebuts d when true => 2\n}\n\
        rule duty r effective [2012-01-01, ..) {\n  deadline z: 2.days from Served(complaint) counted_by c6\n  exception e rebuts d when true => 3\n}";
    let references = "default b when true => true\ndefault a when b => 1\n\
        default c when true => deadline(z)\ndeadline z: 1.days from Served(complaint) counted_by c6\n\
        default f when true => {x: 1}.x\ndefault g when true => match Document.complaint { complaint => 1, motion => 2 }\n\
        default p1 when p2 => true\ndefault p2 when p1 => true";
    insta::assert_debug_snapshot!(table(&[
        ("1 exceptions with too many atoms to prove disjoint", &large),
        ("2 versions of one exception never meet", disjoint),
        ("3 references through refs, deadlines, fields, and matches", references),
        ("4 an authority outranking itself", "authority A > A"),
        ("5 a cycle with a tail below it", "authority A > B\nauthority B > A\nauthority B > C"),
        ("6 a power rule with only a state", "rule power move {\n  state x: \"X\" { _ \"go\" -> t }\n}"),
        (
            "7 a liability rule with only a deadline",
            "rule liability l {\n  deadline p: 21.days from Served(complaint) counted_by c6\n}",
        ),
        (
            "8 a backward deadline and a counting version it never meets",
            "counting itc effective [2009-12-01, 2012-01-01) { anchor next_business_day, holidays cal }\n\
             counting itc effective [2012-01-01, ..) { holidays cal }\n\
             deadline r: 7.days before Hearing(motion) counted_by itc effective [2013-01-01, ..)",
        ),
    ]));
}

#[test]
fn rule_kinds_are_queryable() {
    let body = "rule duty answer {\n  deadline p: 21.days from Served(complaint) counted_by c6\n}\n\
        rule power move {\n  decision m: \"Move?\" by Defendant { _ \"yes\" -> t }\n  deadline q: 1.days from Served(complaint) counted_by c6\n}\n\
        deadline free: 1.days from Served(complaint) counted_by c6";
    let text = format!("{PRELUDE}{body}\n");
    let (db, ws) = workspace(&[pkg(
        "demo",
        "FederalRule",
        "2009-12-01",
        &[],
        &[("main.lit", &text)],
    )]);
    let got = with_world(&db, ws, |world| {
        ["p", "q", "free", "m", "answer", "nothing"]
            .iter()
            .map(|n| {
                let k = Key {
                    pack: "demo".into(),
                    name: (*n).into(),
                };
                let kinds: Vec<_> = world
                    .versions(&k)
                    .versions
                    .iter()
                    .map(|v| shape::rule_kind(world, v.decl))
                    .collect();
                format!("{n}: duty {} kinds {kinds:?}", shape::is_duty(world, &k))
            })
            .collect::<Vec<_>>()
    });
    insta::assert_debug_snapshot!(got);
}

#[test]
fn packages_without_files_or_manifests_and_chains_in_later_files() {
    let empty = pkg("empty", "FederalRule", "2009-12-01", &[], &[]);
    let mut broken = pkg(
        "broken",
        "FederalRule",
        "2009-12-01",
        &[],
        &[("main.lit", PRELUDE)],
    );
    broken.manifest = "[package]\nname = 1\n".into();
    let auth = "module auth\nenum K { k }\nauthority A > B\nauthority B > A\n";
    let two_files = LoadedPackage {
        files: vec![
            LoadedFile {
                relative: "main.lit".into(),
                text: PRELUDE.into(),
            },
            LoadedFile {
                relative: "auth.lit".into(),
                text: auth.into(),
            },
        ],
        ..pkg("demo", "FederalRule", "2009-12-01", &[], &[])
    };
    insta::assert_debug_snapshot!([run(&[empty]), run(&[broken]), run(&[two_files])]);
}
