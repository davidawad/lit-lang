//! Worked example: FRCP 6 and 12 across the December 1, 2009
//! time-computation amendments (adopted by the Supreme Court on March 26,
//! 2009), with FRCP 86(a)(2) written as `transition` declarations
//! (`fixtures/frcp-2009/`).
//!
//! - 6(a) as restyled in 2007 (effective 2007-12-01): periods under 11 days
//!   skip intermediate weekends and legal holidays (6(a)(2)); 6(d) adds 3
//!   days after mail service. From 2009-12-01 every day counts (6(a)(1)(B)).
//! - 12(a)(1)(A)(i): 20 days, then 21. 12(a)(4)(A): 10 days, then 14.
//!
//! The fixture's `test` blocks only state due dates inside one regime.
//! Whether a period that straddles the amendment is recomputed is what the
//! transition decides for a given matter, and courts differed on it; here
//! the matter's own history (an `OrderedPriorRules` order, FRCP
//! 86(a)(2)(B)) selects the version.
mod legal;
use legal::{pkg, render, workspace, CORE};
use litigate_base::Severity;
use litigate_sema::hir::{Declaration, Expr, Package};
use litigate_sema::temporal::{self, governing, Key, MatterEvent, World};

const TIME: &str = include_str!("fixtures/frcp-2009/time.lit");
const RULE12: &str = include_str!("fixtures/frcp-2009/rule12.lit");

fn compile() -> (litigate_sema::LitDatabase, litigate_sema::Workspace) {
    workspace(&[
        pkg(
            "core",
            "Statute",
            "1990-01-01",
            &[],
            &[
                ("authority.lit", CORE),
                ("start.lit", "module core.start\nstart z\nstate z: \"Z\"\n"),
            ],
        ),
        pkg(
            "frcp",
            "FederalRule",
            "2007-12-01",
            &["core"],
            &[("rule12.lit", RULE12), ("time.lit", TIME)],
        ),
    ])
}

fn key(name: &str) -> Key {
    Key {
        pack: "frcp".into(),
        name: name.into(),
    }
}

/// What version `i` says, in one line.
fn describe(world: &World<'_>, i: usize) -> String {
    let d = &world.decls[i];
    let what = match d.decl {
        Declaration::Deadline(x) => format!("{}.days", x.amount),
        Declaration::Counting(c) => match c.short_period {
            Some(s) => format!("under {} days skip weekends and holidays", s.threshold),
            None => "every day counts".to_owned(),
        },
        _ => String::new(),
    };
    format!("{}@{} ({what})", d.meta.id, d.meta.effective.from)
}

#[test]
fn the_package_checks_without_diagnostics() {
    let (db, ws) = compile();
    let checks = litigate_sema::check_workspace(&db, ws);
    let errors: Vec<&litigate_base::Diagnostic> = checks
        .iter()
        .flat_map(|c| &c.diagnostics)
        .filter(|d| d.severity == Severity::Error)
        .collect();
    assert!(errors.is_empty(), "{:#?}", render(&db, ws));
    insta::assert_debug_snapshot!(render(&db, ws));
}

#[test]
fn as_of_selects_the_rules_in_force_and_known() {
    let (db, ws) = compile();
    legal::with_world(&db, ws, |world| {
        let at = temporal::world::nowhere("query");
        let row = |name: &str, date: &str, known: Option<&str>| {
            let got = match temporal::as_of(world, &key(name), date, known, at.clone()) {
                Ok(i) => describe(world, i),
                Err(d) => format!("{} {}", d.code.as_str(), d.message),
            };
            format!(
                "{name} on {date} known {}: {got}",
                known.unwrap_or("latest")
            )
        };
        insta::assert_debug_snapshot!([
            row("answer-period", "2009-11-30", None),
            row("answer-period", "2009-12-01", None),
            row("after-denial", "2009-11-30", None),
            row("after-denial", "2009-12-01", None),
            row("frcp6", "2009-11-30", None),
            row("frcp6", "2009-12-01", None),
            row("answer-period", "2010-01-04", Some("2009-01-01")),
            row("answer-period", "2010-01-04", Some("2009-03-26")),
        ]);
    });
}

/// The edge event patterns and test givens of the package, by callee.
fn event(package: &Package, callee: &str, arg: Option<&str>) -> Expr {
    let modules = package.modules.iter().flat_map(|m| &m.declarations);
    let mut found = modules.flat_map(|d| match d {
        Declaration::Node(n) => n
            .edges
            .iter()
            .filter_map(|e| e.event.clone())
            .collect::<Vec<_>>(),
        Declaration::Test(t) => t.givens.iter().map(|(e, _)| e.clone()).collect(),
        _ => Vec::new(),
    });
    found
        .find(|e| {
            let text = format!("{:?}", e.kind);
            text.contains(&format!("callee: \"{callee}\"")) && arg.is_none_or(|a| text.contains(a))
        })
        .unwrap_or_else(|| panic!("no {callee} event"))
}

#[test]
fn pending_matters_follow_rule_86a_unless_the_court_orders_otherwise() {
    let (db, ws) = compile();
    let checks = litigate_sema::check_workspace(&db, ws);
    let packages: Vec<&Package> = checks.iter().filter_map(|c| c.package.as_ref()).collect();
    let frcp = packages
        .iter()
        .find(|p| p.manifest.name == "frcp")
        .expect("frcp compiles");
    let world = World::from_packages(&packages);
    let served_mail = event(frcp, "Served", Some("Mail"));
    let served = event(frcp, "Served", Some("Personal"));
    let answered = event(frcp, "Answered", None);
    let ordered = event(frcp, "OrderedPriorRules", None);
    let matters: [(&str, Vec<(&str, &Expr)>); 5] = [
        (
            "served by mail before, answered after",
            vec![("2009-11-20", &served_mail), ("2009-12-11", &answered)],
        ),
        (
            "same, with an order keeping the prior rules",
            vec![
                ("2009-11-20", &served_mail),
                ("2009-12-03", &ordered),
                ("2009-12-11", &answered),
            ],
        ),
        (
            "the order comes after the answer",
            vec![
                ("2009-11-20", &served_mail),
                ("2009-12-11", &answered),
                ("2009-12-15", &ordered),
            ],
        ),
        (
            "commenced after the amendments",
            vec![("2009-12-04", &served), ("2009-12-28", &answered)],
        ),
        (
            "over before the amendments",
            vec![("2009-10-01", &served), ("2009-10-21", &answered)],
        ),
    ];
    let mut rows = Vec::new();
    for (label, history) in &matters {
        let events: Vec<MatterEvent<'_>> = history
            .iter()
            .map(|(d, e)| MatterEvent { date: d, action: e })
            .collect();
        let at = events
            .iter()
            .position(|e| e.action.kind == answered.kind)
            .unwrap_or(0);
        for rule in ["answer-period", "frcp6"] {
            let got = match governing(&world, &key(rule), &events, at, None) {
                Ok(g) => {
                    let via = g.via.map_or_else(
                        || "in force on the answer date".to_owned(),
                        |(t, arm)| format!("{} arm {arm}", world.decls[t].meta.id),
                    );
                    format!("{} via {via}", describe(&world, g.version))
                }
                Err(e) => format!("{e:?}"),
            };
            rows.push(format!("{label}: {rule} = {got}"));
        }
    }
    insta::assert_debug_snapshot!(rows);
}
