//! Refinements (SPEC §3.5): `{x: T | p}` and `T where p`, typed in the
//! qualifier grammar and discharged by exact evaluation (tier 1) and the
//! SMT tier (tier 3). Each rejection is a diagnostic with a span.
mod common;
use common::{package, workspace, PRELUDE};

/// `line:col code message | notes` for each diagnostic of a one-module
/// package whose body is `body` (after the 9-line prelude).
fn diagnostics(body: &str) -> Vec<String> {
    let text = format!("{PRELUDE}{body}\n");
    let (db, ws) = workspace(&[package("demo", &[], &[("main.lit", &text)])]);
    litigate_sema::check_workspace(&db, ws)
        .iter()
        .flat_map(|c| c.diagnostics.iter())
        .map(|d| {
            let notes: Vec<String> = d
                .labels
                .iter()
                .map(|l| format!("{}:{} {}", l.span.line, l.span.col, l.message))
                .chain(d.notes.iter().map(|n| n.message.clone()))
                .collect();
            format!(
                "{}:{} {} {} | {}",
                d.span.line,
                d.span.col,
                d.code.as_str(),
                d.message,
                notes.join(" | ")
            )
        })
        .collect()
}

#[test]
fn well_formed_refinements_are_accepted() {
    let out = diagnostics(
        "type Short = {d: Days<CalendarDays> | d > 0.days && d <= 60.days}
type Recent = Date where it >= 2020-01-01
event Extended(days: Short where days != 45.days, on: Recent, method: Method where method == Mail || on > 2024-01-01)
event Paid(amount: Money<usd> where amount >= 100.usd && 2 * amount <= 1_000.usd, odds: Probability where odds <= 1/2)",
    );
    assert_eq!(out, Vec::<String>::new());
}

#[test]
fn probe() {
    for body in [
        "type Never = {d: Days<CalendarDays> | d > 60.days && d < 30.days}",
        "event E(a: Days<CalendarDays> where a > 10.days, b: Days<CalendarDays> where b > a && a > 20.days && b < 15.days)",
        "type Short = {d: Days<CalendarDays> | d <= 10.days}\nevent E(a: Short where a > 20.days)",
        "event E(m: Method where m != Mail && m != Personal)",
        "event E(a: Days<CalendarDays> where a * a > 1.days)",
        "event E(a: Days<CalendarDays> where deadline(x) > 1.days)",
        "event E(a: Days<CalendarDays> where b > 1.days)",
        "event E(a: Days<CalendarDays> where a > 1.court_days)",
        "event E(a: Date where a + 3.court_days > 2024-01-01)",
        "event E(a: Days<CalendarDays> where a)",
        "event E(a: String where a == \"x\")",
        "type L = List<{d: Days<CalendarDays> | d > 0.days}>",
        "event E(a: Days<CalendarDays> where a / 0 > 1.days)",
        "event E(a: Days<CalendarDays> where a > ?)",
        "event E(a: Days<CalendarDays> where a > 1.days) \nstate x: \"X\" { _ \"go\" -> t { event E(45.days) } }",
    ] {
        eprintln!("== {body}\n{:#?}", diagnostics(body));
    }
    panic!();
}
