//! Case event calls are typed against the used packages' event types
//! (SPEC §5.2, §5.6): one row per argument type, per spelling of a callee
//! and of an enum variant, and per diagnostic.
mod cov_case_support;

use cov_case_support::{matter, run, STAKES};
use litigate_base::Profile;
use litigate_sema::case::NoClock;
use std::collections::BTreeMap;

/// `event` alone in a matter over `alpha`, `delta`, and `gamma`.
fn typed(event: &str) -> Vec<String> {
    let text = matter(
        "alpha ^1.0.0, delta ^1.0.0, gamma ^1.0.0",
        "",
        &[event],
        STAKES,
    );
    run(&text, &NoClock, Profile::Dev)
}

#[test]
fn argument_types_table() {
    let rows: Vec<(&str, &str)> = vec![
        (
            "ok: a date, a boolean, an exact number, and an amount",
            "2010-01-04 Ruled(2010-01-01, true, 3.5, 5.usd) by p",
        ),
        (
            "ok: a string after a variant",
            "2010-01-04 Filed(complaint, \"by mail\") by p",
        ),
        (
            "date: not a date literal",
            "2010-01-04 Ruled(true, true, 3, 5.usd) by p",
        ),
        (
            "boolean: not true or false",
            "2010-01-04 Ruled(2010-01-01, 1, 3, 5.usd) by p",
        ),
        (
            "number: not a number literal",
            "2010-01-04 Ruled(2010-01-01, true, \"3\", 5.usd) by p",
        ),
        (
            "number: beyond an exact rational",
            "2010-01-04 Ruled(2010-01-01, true, 99999999999999999999999, 5.usd) by p",
        ),
        (
            "money: not a money literal",
            "2010-01-04 Ruled(2010-01-01, true, 3, 5) by p",
        ),
        (
            "money: another currency is not a literal",
            "2010-01-04 Ruled(2010-01-01, true, 3, 5.eur) by p",
        ),
        (
            "money: a fraction of a cent",
            "2010-01-04 Ruled(2010-01-01, true, 3, 5.001.usd) by p",
        ),
        (
            "string: not a string literal",
            "2010-01-04 Filed(complaint, 3) by p",
        ),
        (
            "hours: cannot be written in a case",
            "2010-01-04 Logged(3.hours) by p",
        ),
        (
            "variant: not a reference",
            "2010-01-04 Benched(\"complaint\") by p",
        ),
    ];
    let table: BTreeMap<&str, Vec<String>> = rows
        .iter()
        .map(|(name, event)| (*name, typed(event)))
        .collect();
    insta::assert_debug_snapshot!(table);
}

#[test]
fn callees_and_variants_table() {
    let rows: Vec<(&str, &str)> = vec![
        (
            "ok: the primary package's event, qualified",
            "2010-01-04 alpha::Benched(complaint) by p",
        ),
        (
            "ok: a variant qualified by its enum",
            "2010-01-04 Benched(Document::complaint) by p",
        ),
        (
            "ok: a variant qualified by its package and enum",
            "2010-01-04 Benched(alpha::Document::complaint) by p",
        ),
        (
            "another package: typed in its spelling, not legal at the primary start",
            "2010-01-04 gamma::Pinged(gamma::Kind::a) by p",
        ),
        (
            "another package: no such event, nearest suggested in its spelling",
            "2010-01-04 gamma::Pinge(a) by p",
        ),
        (
            "callee: no such event and nothing near it",
            "2010-01-04 Zzzzzzzzzzzz(complaint) by p",
        ),
        (
            "callee: qualified by a pack the matter does not use",
            "2010-01-04 omega::Benched(complaint) by p",
        ),
        ("arity: one argument missing", "2010-01-04 Benched() by p"),
        (
            "arity: three arguments missing",
            "2010-01-04 Ruled(2010-01-01) by p",
        ),
        (
            "variant: qualified by another enum",
            "2010-01-04 Benched(Method::complaint) by p",
        ),
        (
            "variant: misspelled, nearest suggested",
            "2010-01-04 Benched(complant) by p",
        ),
        (
            "variant: nothing near it",
            "2010-01-04 Benched(zzzzzzzzzzzz) by p",
        ),
    ];
    let table: BTreeMap<&str, Vec<String>> = rows
        .iter()
        .map(|(name, event)| (*name, typed(event)))
        .collect();
    insta::assert_debug_snapshot!(table);
}
