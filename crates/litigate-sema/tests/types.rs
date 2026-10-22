//! Type-checker behavior on real `.lit` sources (SPEC §6), one rule per
//! test, with diagnostics snapshotted.
mod common;
use common::check_body;

#[test]
fn clean_prelude_has_no_diagnostics() {
    assert_eq!(check_body(""), Vec::<String>::new());
}

#[test]
fn day_units_do_not_mix_and_suggest_the_counting_rule() {
    insta::assert_debug_snapshot!(check_body("default d when true => 21.days + 5.court_days"));
}

#[test]
fn dates_move_only_through_counting_rules() {
    insta::assert_debug_snapshot!(check_body("default d when true => 2025-01-01 + 3.days"));
}

#[test]
fn money_is_exact_integer_cents_in_one_currency() {
    let out = check_body("default a when true => 1.005.usd\ndefault b when true => 10.usd / 3\ndefault c when true => 10.usd / 2.usd");
    insta::assert_debug_snapshot!(out);
}

#[test]
fn probabilities_stay_in_the_unit_interval() {
    let out = check_body(
        "chance k: \"K\" { a \"A\" -> t p 3/2, b \"B\" -> t p 1/3 }\nchance j: \"J\" { a \"A\" -> t p 1/3, b \"B\" -> t p 1/3 }\nchance m: \"M\" { a \"A\" -> t }\nstate n: \"N\" { a \"A\" -> n p 1/2 }",
    );
    insta::assert_debug_snapshot!(out);
}

#[test]
fn distributions_are_total_and_exact() {
    let out = check_body(
        "type D = Dist<Method>\ndefault ok when prob({ Personal: 1/3, Mail: 2/3 }, Mail) == 2/3 => 1\nexception e rebuts ok when true => prob(given({ Personal: 1/4, Mail: 3/4 }, [Mail]), Mail)\ndefault short when true => prob({ Personal: 1/3 }, Mail)\ndefault over when true => prob({ Personal: 1/2, Mail: 2/3 }, Mail)\ndefault zero when true => given({ Personal: 1, Mail: 0 }, [Mail])",
    );
    insta::assert_debug_snapshot!(out);
}

#[test]
fn match_is_exhaustive_without_unreachable_arms() {
    let out = check_body(
        "default full when true => match Method.Mail { Personal => 1, Mail => 2 }\ndefault missing when true => match Method.Mail { Personal => 1 }\ndefault late when true => match Method.Mail { _ => 1, Mail => 2 }\ndefault twice when true => match Method.Mail { Mail => 1, Mail => 2, Personal => 3 }",
    );
    insta::assert_debug_snapshot!(out);
}

#[test]
fn no_implicit_conversions_or_unknown_operations() {
    let out = check_body("default a when 1 => 2\ndefault b when true < false => 1\ndefault c when true => \"x\" + \"y\"\ndefault d when !3 => 1");
    insta::assert_debug_snapshot!(out);
}

#[test]
fn event_patterns_and_tests_are_typed() {
    let out = check_body(
        "deadline ok: 21.days from Served(complaint) counted_by c6\ndeadline bad: 21.days from Served(Mail) counted_by c6\ndeadline hours: 8.hours from Served(complaint) counted_by c6\ntest \"t\" {\n  given Served(complaint) on 2025-01-01\n  expect deadline(ok) == 2025-01-22\n  expect deadline(s) == 2025-01-22\n  expect deadline(ok)\n}",
    );
    insta::assert_debug_snapshot!(out);
}

#[test]
fn field_types_and_placement() {
    let out = check_body(
        "decision x: \"X\" by Judge discretion {\n  payoff 1.usd\n  tags [fact]\n  valence goood\n  a \"A\" -> t { effort 3.days, cost 5, deadline 8.hours, duration { min 5.days, mode 2.days, max 9.days } }\n  b \"B\" -> t { within ok }\n}\ndeadline ok: 21.days from Served(complaint) counted_by c6",
    );
    insta::assert_debug_snapshot!(out);
}

#[test]
fn unit_mixing_note_names_an_imported_counting_rule() {
    let rules = "module rules\ncounting frcp6 { holidays cal }\ncalendar cal { new-year: fixed(1, 1) effective [1870-06-28, ..) }\n";
    let main = "module main\nimport rules.{frcp6}\nstart s\nterminal s: \"S\"\ndefault d when true => 21.days + 5.court_days\n";
    let (db, ws) = common::workspace(&[common::package(
        "demo",
        &[],
        &[("main.lit", main), ("rules.lit", rules)],
    )]);
    let checks = litigate_sema::check_workspace(&db, ws);
    let notes: Vec<&str> = checks
        .iter()
        .flat_map(|c| c.diagnostics.iter())
        .filter(|d| d.code == litigate_base::DiagnosticCode::UnitMismatch)
        .flat_map(|d| d.notes.iter().map(|n| n.message.as_str()))
        .collect();
    assert!(
        notes.iter().any(|n| n.contains("counted_by frcp6")),
        "{notes:?}"
    );
}
