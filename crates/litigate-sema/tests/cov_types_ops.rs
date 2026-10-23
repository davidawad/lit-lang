//! Operator typing and exact folding (SPEC §6 permitted-operations table),
//! with diagnostics snapshotted.
mod common;
use common::{check_body, package, render, workspace};

#[test]
fn every_operator_outside_the_table_is_rejected() {
    let out = check_body(
        "default a when true || 1 => 1\ndefault b when true && 1 => 1\ndefault c when 1 == true => 1\ndefault d when [1] != [1] => 1\ndefault e when \"a\" < \"a\" => 1\ndefault f when 1 <= true => 1\ndefault g when \"a\" > \"b\" => 1\ndefault h when \"a\" >= \"b\" => 1\ndefault i when true => true + true\ndefault j when true => 1 - true\ndefault k when true => \"a\" * 2\ndefault l when true => \"a\" / 2\ndefault m when true => 1.usd % 2\ndefault n when 1 != true => 1\ndefault o when [1] == [1] => 1\ndefault p when 1 < true => 1\ndefault q when 1 > true => 1\ndefault r when 1 >= true => 1",
    );
    insta::assert_debug_snapshot!(out);
}

#[test]
fn dates_move_only_through_counting_rules() {
    let out = check_body(
        "default a when true => 2025-01-01 + 1.days\ndefault b when true => 8.hours + 2025-01-01",
    );
    insta::assert_debug_snapshot!(out);
}

#[test]
fn units_and_currencies_do_not_mix() {
    let out = check_body(
        "default a when true => 1.court_days - 1.business_days\ndefault b when 1.usd < 1.usd => 1",
    );
    insta::assert_debug_snapshot!(out);
}

#[test]
fn unit_mixing_without_a_counting_rule_uses_a_placeholder() {
    let main =
        "module main\nstart s\nterminal s: \"S\"\ndefault d when true => 21.days + 5.court_days\n";
    let (db, ws) = workspace(&[package("demo", &[], &[("main.lit", main)])]);
    let checks = litigate_sema::check_workspace(&db, ws);
    let notes: Vec<&str> = checks
        .iter()
        .flat_map(|c| c.diagnostics.iter())
        .flat_map(|d| d.notes.iter().map(|n| n.message.as_str()))
        .collect();
    assert!(
        notes
            .iter()
            .any(|n| n.contains("counted_by <counting-rule>")),
        "{notes:?}"
    );
    insta::assert_debug_snapshot!(render(&db, ws));
}

#[test]
fn unary_operators_fold() {
    let out = check_body(
        "default a when !true => -(3.days)\ndefault b when !nope => -(8.hours)\ndefault c when true => -(1.usd)\ndefault d when true => -(-9223372036854775807 - 1)\ndefault e when true => -(2025-01-01)\ndefault f when true => -nope",
    );
    insta::assert_debug_snapshot!(out);
}

#[test]
fn numbers_meet_probabilities_as_probabilities() {
    let out = check_body(
        "default d when true => { Personal: 1/4, Mail: 3/4 }\ndefault a when 1/4 == prob({ Personal: 1/4, Mail: 3/4 }, Personal) => 1\ndefault b when 3/2 == prob({ Personal: 1/4, Mail: 3/4 }, Personal) => 1\ndefault c when nope == prob({ Personal: 1/4, Mail: 3/4 }, Personal) => 1\ndefault e when true => prob({ Personal: 1/4, Mail: 3/4 }, Mail) + prob({ Personal: 1/4, Mail: 3/4 }, Mail)\ndefault f when true => prob({ Personal: 1/4, Mail: 3/4 }, Mail) * prob({ Personal: 1/4, Mail: 3/4 }, Personal)\ndefault g when true => prob({ Personal: 1, Mail: 0 }, Mail) / prob({ Personal: 1, Mail: 0 }, Mail)\ndefault h when -1/2 == prob({ Personal: 1/4, Mail: 3/4 }, Personal) => 1\ndefault i when 2 == prob({ Personal: 1/4, Mail: 3/4 }, Personal) => 1",
    );
    insta::assert_debug_snapshot!(out);
}

#[test]
fn constants_fold_exactly() {
    let out = check_body(
        "default a when 1 + 2 == 3 && 3 - 1 != 1 => 2 * 3\ndefault b when 6 / 3 <= 2 || 7 % 2 > 1 => 1\ndefault c when 1 >= 2 => 1\ndefault i when false && true => 1\ndefault j when true || false => 1\ndefault m when false || true => 1\ndefault k when true => 7 % 1.5\ndefault l when true => 1234567890123456789012345678901234567890\ndefault d when 2025-01-01 < 2025-02-01 => 1\ndefault e when \"a\" == \"a\" => 1\ndefault f when true => 1 / 0\ndefault g when true => 7 % 0\ndefault h when true => 1.5 % 1",
    );
    insta::assert_debug_snapshot!(out);
}

#[test]
fn quantities_fold_in_whole_units() {
    let out = check_body(
        "default a when 8.hours + 1.hours > 2 * 4.hours => 8.hours / 2\ndefault b when 2 * 1.usd == 1.usd * 2 => 1.usd + 1.usd\ndefault c when true => 10.usd / 3\ndefault d when true => 1.usd / 0.usd\ndefault e when 3.days / 3 == 1.days => 3.days * 2\ndefault f when true => 3.days / 2\ndefault g when true => 3.usd / 2.usd\ndefault h when true => 8.hours / 0",
    );
    insta::assert_debug_snapshot!(out);
}

#[test]
fn fold_leaves_operands_outside_the_table_unfolded() {
    use litigate_sema::hir::{BinaryOp, DayUnit, Money, Value};
    use litigate_sema::types::ops::fold;
    let s = Value::String("a".into());
    let usd = Value::Money(Money {
        minor_units: 1,
        currency: "usd".into(),
    });
    let days = Value::Days {
        amount: 1,
        unit: DayUnit::Calendar,
    };
    assert_eq!(fold(BinaryOp::Lt, &s, &s), Ok(None));
    assert_eq!(fold(BinaryOp::Add, &s, &s), Ok(None));
    assert_eq!(fold(BinaryOp::Add, &usd, &s), Ok(None));
    assert_eq!(fold(BinaryOp::Add, &s, &days), Ok(None));
}
