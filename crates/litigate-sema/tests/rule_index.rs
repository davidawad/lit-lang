//! The indexed rule type `Rule[a, s, v, k, ap]` (SPEC §3.1, design review
//! §2.3, §6 item 20): surface sugar for a rule's `effective`, `known_at`
//! and `commenced` clauses, with its authority and instrument class
//! checked against its package's.
#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use common::{check_body, package, workspace, PRELUDE};
use litigate_sema::hir::{Declaration, Interval};
use std::collections::BTreeMap;

const CHAIN: &str = "authority FederalRule > LocalRule > CaseOrder\n";

/// Diagnostics of a rule `r` with header tail `header` holding one
/// deadline `d`.
fn rule(header: &str) -> Vec<String> {
    check_body(&format!(
        "{CHAIN}rule duty r {header} cite \"FRCP 12(a)\" {{\n  deadline d: 21.days from Served(complaint) counted_by c6\n}}"
    ))
}

/// The in-force interval of deadline `d` in a rule with header tail
/// `header`.
fn interval(header: &str) -> Interval {
    let text = format!(
        "{PRELUDE}{CHAIN}rule duty r {header} cite \"FRCP 12(a)\" {{\n  deadline d: 21.days from Served(complaint) counted_by c6\n}}\n"
    );
    let (db, ws) = workspace(&[package("demo", &[], &[("main.lit", &text)])]);
    let checked = litigate_sema::check_workspace(&db, ws);
    let hir = checked[0].package.clone().expect("the package builds");
    hir.modules
        .iter()
        .flat_map(|m| &m.declarations)
        .find_map(|d| match d {
            Declaration::Deadline(d) if d.meta.id == "d" => Some(d.meta.effective.clone()),
            _ => None,
        })
        .expect("deadline d")
}

#[test]
fn the_index_desugars_to_its_version_clauses() {
    let sugar =
        "Rule[FederalRule, base, [2010-01-01, ..), [2009-06-01, ..), [2010-01-01, 2020-01-01)]";
    let clauses =
        "effective [2010-01-01, ..) known_at 2009-06-01 commenced [2010-01-01, 2020-01-01)";
    assert_eq!(rule(sugar), Vec::<String>::new());
    assert_eq!(interval(sugar), interval(clauses));
    assert_eq!(interval(sugar).known_at, "2009-06-01");
    // `_` leaves every index to its default: the package's.
    let defaults = "Rule[_, _, _, _, _]";
    assert_eq!(rule(defaults), Vec::<String>::new());
    assert_eq!(interval(defaults), interval(""));
}

#[test]
fn each_ill_formed_index_is_reported() {
    let rows: BTreeMap<&str, Vec<String>> = [
        ("four indices", "Rule[_, _, _, _]"),
        ("another authority", "Rule[LocalRule, _, _, _, _]"),
        ("an undeclared authority", "Rule[Nobody, _, _, _, _]"),
        ("another class", "Rule[_, local-rule, _, _, _]"),
        ("not a class", "Rule[_, fuzzy, _, _, _]"),
        (
            "an interval for a name",
            "Rule[[2010-01-01, ..), _, _, _, _]",
        ),
        ("a qualified name", "Rule[_, a.b, _, _, _]"),
        ("a qualified authority", "Rule[a.b, _, _, _, _]"),
        ("a name for an interval", "Rule[_, _, base, _, _]"),
        (
            "a closed transaction interval",
            "Rule[_, _, _, [2009-06-01, 2011-01-01), _]",
        ),
        (
            "an index and its clause",
            "Rule[_, _, [2010-01-01, ..), _, _] effective [2011-01-01, ..)",
        ),
    ]
    .into_iter()
    .map(|(name, header)| (name, rule(header)))
    .collect();
    insta::assert_debug_snapshot!(rows);
}
