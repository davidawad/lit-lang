//! `given e by actor on date` in inline tests (lg-pxkt): the actor is
//! resolved like a grant's, and an order given by it extends a period
//! under the grants it holds (`case::grant::extensions`), as replay
//! judges a matter's order.
// Test helpers below may unwrap: a panic is the failed assertion.
#![allow(clippy::unwrap_used)]
mod common;
use common::{package, render, workspace, PRELUDE};
use litigate_sema::case::grant::{extends, extensions, Given};
use litigate_sema::hir::{Declaration, Package, Test};
use litigate_sema::temporal::{Key, World};

/// `p` is FRCP 6(b)(1)'s pair of court grants; `capped` a bounded grant;
/// `locked` may not be extended; `bare` grants nothing.
const LAW: &str = "event Extended(until: Date) extends p to until
event Capped(until: Date) extends capped to until
event Locked(until: Date) extends locked to until
event Bare(until: Date) extends bare to until
event Moved()
deadline p: 20.days from Served(complaint) counted_by c6 {
  extend { by Court, discretion GoodCause }
  extend { by Court, discretion ExcusableNeglect, after_expiry, on Moved() }
}
deadline capped: 20.days from Served(complaint) counted_by c6 {
  extend { by Court, max 10.days }
}
deadline locked: 20.days from Served(complaint) counted_by c6 {
  extend { never cite \"FRCP 6(b)(2)\" }
}
deadline bare: 20.days from Served(complaint) counted_by c6
";

/// The package of `LAW` with one test of `givens`, after its first
/// (`Served`, on 2025-03-03), if it compiles, and every diagnostic.
fn compiled(givens: &str) -> (Option<Package>, Vec<String>) {
    let text = format!(
        "{PRELUDE}{LAW}test \"t\" {{\n  given Served(complaint, Personal) on 2025-03-03\n{givens}}}\n"
    );
    let (db, ws) = workspace(&[package("demo", &[], &[("main.lit", &text)])]);
    let diagnostics = render(&db, ws);
    let pkg = litigate_sema::check_workspace(&db, ws)
        .into_iter()
        .find_map(|c| c.package.clone());
    (pkg, diagnostics)
}

fn test_of(pkg: &Package) -> &Test {
    pkg.modules
        .iter()
        .flat_map(|m| &m.declarations)
        .find_map(|d| match d {
            Declaration::Test(t) => Some(t),
            _ => None,
        })
        .unwrap()
}

/// The extensions of deadline `d` in a test of `givens`, its instance
/// due 2025-03-24 (a 20-day count from 2025-03-03 rolled off a Sunday).
fn extend(d: &str, givens: &str) -> Result<Vec<(usize, String)>, String> {
    let (pkg, diagnostics) = compiled(givens);
    assert_eq!(diagnostics, Vec::<String>::new());
    let pkg = pkg.unwrap();
    let world = World::from_packages(&[&pkg]);
    let deadline = pkg
        .modules
        .iter()
        .flat_map(|m| &m.declarations)
        .find_map(|x| match x {
            Declaration::Deadline(x) if x.meta.id == d => Some(x),
            _ => None,
        })
        .unwrap();
    let test = test_of(&pkg);
    let history: Vec<Given<'_>> = test
        .givens
        .iter()
        .enumerate()
        .map(|(i, (action, date))| Given {
            date,
            action,
            actor: test.actors.get(&i).map(String::as_str),
        })
        .collect();
    let key = Key {
        pack: "demo".to_owned(),
        name: d.to_owned(),
    };
    extensions(deadline, d, &history, 0, "2025-03-24", |action| {
        extends(&world, &pkg, &key, action)
    })
}

fn refused(d: &str, givens: &str) -> String {
    extend(d, givens).unwrap_err()
}

#[test]
fn a_given_records_its_actor_by_index() {
    let (pkg, diagnostics) = compiled(
        "  given Moved() on 2025-03-04\n  given Extended(2025-04-30) by Court on 2025-03-10\n",
    );
    assert_eq!(diagnostics, Vec::<String>::new());
    let actors: Vec<(usize, String)> = test_of(&pkg.unwrap()).actors.clone().into_iter().collect();
    assert_eq!(actors, [(2, "Court".to_owned())]);
}

#[test]
fn an_undeclared_actor_is_l1009() {
    let (_, diagnostics) = compiled("  given Extended(2025-04-30) by Nobody on 2025-03-10\n");
    assert_eq!(
        diagnostics,
        ["pkgs/demo/main.lit:28:33 L1009 actor `Nobody` is not declared in `[package.roles]`"]
    );
}

#[test]
fn an_order_by_the_grant_holder_extends_the_period() {
    let got = extend("p", "  given Extended(2025-04-30) by Court on 2025-03-10\n");
    assert_eq!(got, Ok(vec![(1, "2025-04-30".to_owned())]));
}

#[test]
fn an_order_naming_no_actor_extends_nothing() {
    assert_eq!(
        refused("p", "  given Extended(2025-04-30) on 2025-03-10\n"),
        "only Court may extend `p`, but `Extended(2025-04-30)` on 2025-03-10 names no actor (`given ... by <actor> on <date>`)"
    );
}

#[test]
fn an_order_by_another_actor_extends_nothing() {
    assert_eq!(
        refused(
            "p",
            "  given Extended(2025-04-30) by Plaintiff on 2025-03-10\n"
        ),
        "only Court may extend `p`, but `Extended(2025-04-30)` on 2025-03-10 is by `Plaintiff`"
    );
}

#[test]
fn after_expiry_the_order_needs_the_motion() {
    assert_eq!(
        refused("p", "  given Extended(2025-04-30) by Court on 2025-04-01\n"),
        "`Extended(2025-04-30)` requires a motion `Moved()` made after the period expired on 2025-03-24, and none precedes the order on 2025-04-01"
    );
    let got = extend(
        "p",
        "  given Moved() on 2025-03-28\n  given Extended(2025-04-30) by Court on 2025-04-01\n",
    );
    assert_eq!(got, Ok(vec![(2, "2025-04-30".to_owned())]));
}

#[test]
fn a_motion_is_spent_by_the_extension_it_licenses() {
    assert_eq!(
        refused(
            "p",
            "  given Moved() on 2025-03-28\n  given Extended(2025-04-15) by Court on 2025-04-01\n  given Extended(2025-05-15) by Court on 2025-04-20\n",
        ),
        "`Extended(2025-05-15)` requires a motion `Moved()` made after the period expired on 2025-04-15 after the one made on 2025-03-28, spent by an earlier extension, and none precedes the order on 2025-04-20"
    );
}

#[test]
fn an_order_is_held_to_the_bound_and_must_extend() {
    assert_eq!(
        refused("capped", "  given Capped(2025-12-31) by Court on 2025-03-10\n"),
        "an extension to 2025-12-31 exceeds the bound of `capped`: prescribed_due + 10.days = 2025-04-03 (prescribed_due 2025-03-24, grant_order_entered 2025-03-10)"
    );
    assert_eq!(
        refused("p", "  given Extended(2025-03-20) by Court on 2025-03-10\n"),
        "an extension to 2025-03-20 does not extend `p`, already due 2025-03-24"
    );
}

#[test]
fn a_locked_or_ungranted_period_is_not_extended() {
    assert_eq!(
        refused("locked", "  given Locked(2025-04-30) by Court on 2025-03-10\n"),
        "`Locked(2025-04-30)` cannot extend `locked`: the deadline may not be extended (`extend { never cite \"FRCP 6(b)(2)\" }`)"
    );
    assert_eq!(
        refused("bare", "  given Bare(2025-04-30) by Court on 2025-03-10\n"),
        "`Bare(2025-04-30)` cannot extend `bare`: the deadline grants no `extend` capability"
    );
}

#[test]
fn an_order_to_no_known_date_moves_nothing() {
    assert_eq!(
        refused("p", "  given Extended(?) by Court on 2025-03-10\n"),
        "`Extended(?)` extends `p` to no known date, which moves nothing"
    );
}

#[test]
fn only_later_orders_extending_the_period_count() {
    // `Capped` extends another deadline; `Moved` extends none.
    let got = extend(
        "p",
        "  given Capped(2025-04-30) by Court on 2025-03-10\n  given Moved() on 2025-03-11\n",
    );
    assert_eq!(got, Ok(Vec::new()));
}
