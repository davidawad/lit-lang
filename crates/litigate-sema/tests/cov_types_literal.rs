//! Typed literals (SPEC §2, §6): money in whole cents and integral day
//! amounts, with diagnostics snapshotted.
mod common;
use common::check_body;

#[test]
fn money_literals_are_whole_cents() {
    let out = check_body(
        "default a when true => 1_000.50.usd\ndefault b when true => 1.5_0.usd\ndefault c when true => 1.5_0_1.usd\ndefault d when true => 99999999999999999999.usd",
    );
    insta::assert_debug_snapshot!(out);
}

#[test]
fn day_literals_carry_their_unit() {
    let out = check_body(
        "default a when 3.business_days == 3.business_days => 1\ndefault b when true => 99999999999999999999.days\ndefault c when 1.5.hours < 2.hours => 1",
    );
    insta::assert_debug_snapshot!(out);
}

#[test]
fn day_amounts_outside_estimates_must_be_integers() {
    // The parser admits decimals in `duration`; only calendar days may be
    // fractional estimates (SPEC §6), so court days are checked here.
    let out = check_body("decision x: \"X\" {\n  a \"A\" -> t { duration 4.5.court_days }\n  b \"B\" -> t { duration 4.5.days }\n}");
    insta::assert_debug_snapshot!(out);
}
