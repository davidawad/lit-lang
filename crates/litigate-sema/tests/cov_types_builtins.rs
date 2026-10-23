//! The closed builtin list (SPEC §6): arity, argument types, and exact
//! folding of `prob`/`given`/`len`, with diagnostics snapshotted.
mod common;
use common::check_body;

#[test]
fn builtins_check_their_argument_count() {
    let out = check_body(
        "default a when true => len()\ndefault b when true => prob(1)\ndefault c when true => contains([1], 1, 2)",
    );
    insta::assert_debug_snapshot!(out);
}

#[test]
fn deadline_takes_a_deadline_name() {
    let out = check_body(
        "deadline ok: 21.days from Served(complaint) counted_by c6\ntest \"t\" {\n  given Served(complaint, Mail) on 2025-01-01\n  expect deadline(ok) == 2025-01-22\n  expect deadline(1) == 2025-01-22\n  expect deadline(nope) == 2025-01-22\n}",
    );
    insta::assert_debug_snapshot!(out);
}

#[test]
fn has_event_takes_an_event_pattern() {
    let out = check_body(
        "default a when has_event(Served(complaint, Mail)) => 1\ndefault b when has_event(1) => 1",
    );
    insta::assert_debug_snapshot!(out);
}

#[test]
fn interval_builtins_take_dates_and_intervals() {
    let out = check_body(
        "default a when in(2025-01-01, ?) => 1\ndefault b when disjoint(?, ?) => 1\ndefault c when true => intersect(?, ?)\ndefault d when in(1, ?) => 1\ndefault e when in(2025-01-01, 2) => 1\ndefault f when !intersect(?, ?) => 1",
    );
    insta::assert_debug_snapshot!(out);
}

#[test]
fn list_builtins_need_lists() {
    let out = check_body(
        "default xs when true => [1, 2]\ndefault a when len([1, 2, 3]) == 3 => 1\ndefault b when len(xs) == 2 => 1\ndefault c when len(1) == 1 => 1\ndefault d when len(nope) == 1 => 1\ndefault e when contains(xs, 3) => 1\ndefault f when contains([1], true) => 1\ndefault g when contains(1, 2) => 1\ndefault h when len([]) == 0 => 1",
    );
    insta::assert_debug_snapshot!(out);
}

#[test]
fn dist_literals_are_typed_by_the_unique_enum_with_their_variants() {
    let out = check_body(
        "enum Other { Personal, Mail }\ndefault a when true => prob({ Foo: 1 }, Mail)\ndefault b when true => prob({ Personal: 1/2, Mail: 1/2 }, Mail)\ndefault c when true => prob({ complaint: 1/2, answer: 1/2 }, answer)",
    );
    insta::assert_debug_snapshot!(out);
}

#[test]
fn prob_and_given_need_a_dist() {
    let out = check_body(
        "default d when true => given({ Personal: 1/4, Mail: 3/4 }, [Mail])\ndefault m when true => Method.Mail\ndefault a when true => prob(1, Mail)\ndefault b when true => prob(nope, Mail)\ndefault c when prob(d, Mail) == 1 => 1\ndefault e when prob({ Personal: 1/4, Mail: 3/4 }, ?) == 1 => 1\ndefault f when true => given(d, [Mail])\ndefault g when true => given({ Personal: 1/4, Mail: 3/4 }, [m])\ndefault h when true => given({ Personal: 1/4, Mail: 3/4 }, ?)\ndefault i when true => given({ Personal: 1/4, Mail: 3/4 }, [Fax])\ndefault j when prob(d, Fax) == 1 => 1",
    );
    insta::assert_debug_snapshot!(out);
}

#[test]
fn prob_and_given_fold_exactly() {
    let out = check_body(
        "default a when prob({ Personal: 1/4, Mail: 3/4 }, Mail) == 3/4 => 1\nexception x rebuts a when prob(given({ Personal: 1/4, Mail: 3/4 }, [Mail, Personal]), Personal) != 1/4 => 2\ndefault z when true => given({ Personal: 1, Mail: 0 }, [Mail])",
    );
    insta::assert_debug_snapshot!(out);
}

#[test]
fn conditioning_reports_exact_arithmetic_overflow() {
    // Each weight and every prefix sum fits, but the mass of the chosen
    // subset `{a, c}` is `1/P + 1/Q` with a denominator `P * Q` > u64::MAX.
    let out = check_body(
        "enum Four { a, b, c, d }\ndefault x when true => given({ a: 1/5000000000, b: 1/2 - 1/5000000000, c: 1/5000000001, d: 1/2 - 1/5000000001 }, [a, c])",
    );
    insta::assert_debug_snapshot!(out);
}
