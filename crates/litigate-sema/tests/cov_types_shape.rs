//! Records, lists, projections, exhaustive `match`, and exact `Dist`
//! literals (SPEC §5.2, §6), with diagnostics snapshotted.
mod common;
use common::check_body;

#[test]
fn records_have_distinct_fields() {
    let out = check_body(
        "default a when true => { x: 1, y: 2 }\ndefault b when true => { x: 1, x: 2 }\ndefault c when !{ x: nope } => 1",
    );
    insta::assert_debug_snapshot!(out);
}

#[test]
fn list_elements_share_the_first_elements_type() {
    let out = check_body(
        "default a when true => [1, 2]\ndefault b when true => [1, true]\ndefault c when true => [nope, 1]\ndefault d when true => []\ndefault f when true => [nope]\ndefault e when contains([], 1) => 1",
    );
    insta::assert_debug_snapshot!(out);
}

#[test]
fn fields_project_from_records_only() {
    let out = check_body(
        "default a when true => { x: 1 }.x\ndefault b when true => { count: 1 }.cont\ndefault c when true => { x: 1 }.zzzzzz\ndefault d when true => (1).x\ndefault e when true => (nope).x",
    );
    insta::assert_debug_snapshot!(out);
}

#[test]
fn match_scrutinees_are_enums_or_bools() {
    let out = check_body(
        "default m when true => Method.Mail\ndefault a when true => match true { true => 1, false => 2 }\ndefault b when true => match 1 { _ => 1 }\ndefault c when true => match nope { _ => 1 }\ndefault e when true => match Served(complaint, Mail) { _ => 1 }\ndefault d when true => match m { Personal => 1, Mail => 2 }",
    );
    insta::assert_debug_snapshot!(out);
}

#[test]
fn match_patterns_name_a_variant_or_enum_variant() {
    let out = check_body(
        "default a when true => match Method.Mail { Method.Personal => 1, Method.Mail => 2 }\ndefault b when true => match Method.Mail { Personal => 1, Mal => 2 }\ndefault c when true => match Method.Mail { Personal => 1, Mail.x => 2, _ => 3 }\ndefault d when true => match Method.Mail { Personal => 1, Method@i.Mail => 2, _ => 3 }\ndefault e when true => match Method.Mail { Personal => 1, Method.Mail@2025-01-01 => 2, _ => 3 }\ndefault f when true => match Method.Mail { Personal => 1, Zzzzzz => 2, _ => 3 }",
    );
    insta::assert_debug_snapshot!(out);
}

#[test]
fn match_arms_share_a_type_and_fold_on_constants() {
    let out = check_body(
        "default a when match Method.Mail { Personal => false, _ => true } => match Method.Personal { Mail => 1, _ => 2 }\ndefault b when true => match Method.Mail { Personal => 1, Mail => true }\ndefault c when true => match Method.Mail { Personal => nope, Mail => 1 }\ndefault d when true => match Method.Mail { _ => 1, _ => 2 }\nexception e rebuts a when true => match Method.Mail { Personal => 1, Mail => 2 }",
    );
    insta::assert_debug_snapshot!(out);
}

#[test]
fn dist_records_name_each_variant_once() {
    let out = check_body(
        "event P(d: Dist<Method>)\ndefault a when has_event(P({ Personal: 1/2, Mail: 1/2 })) => 1\ndefault b when has_event(P({ Personal: 1/2, Fax: 1/2, Mail: 1/2 })) => 1\ndefault c when has_event(P({ Personal: 1/2, Personal: 1/2, Mail: 1/2 })) => 1",
    );
    insta::assert_debug_snapshot!(out);
}

#[test]
fn dist_weights_are_constant_probabilities() {
    let out = check_body(
        "event P(d: Dist<Method>)\ndefault n when true => 1/2\ndefault d when true => given({ Personal: 1/4, Mail: 3/4 }, [Mail])\ndefault a when has_event(P({ Personal: ?, Mail: 1/2 })) => 1\ndefault b when has_event(P({ Personal: n, Mail: 1/2 })) => 1\ndefault c when has_event(P({ Personal: prob(d, Mail), Mail: 0 })) => 1\ndefault e when has_event(P({ Personal: ?, Mail: ? })) => 1",
    );
    insta::assert_debug_snapshot!(out);
}

#[test]
fn dist_weights_sum_to_exactly_one() {
    let out = check_body(
        "enum Three { a, b, c }\nevent P(d: Dist<Three>)\ndefault x when has_event(P({ a: ?, b: 2/3, c: 2/3 })) => 1\ndefault y when has_event(P({ a: ?, b: 1/3, c: 1/3 })) => 1\ndefault z when has_event(P({ a: 1/5000000000, b: 1/5000000001, c: 0 })) => 1\ndefault w when has_event(P({ a: 1/2, b: 1/3, c: 1/7 })) => 1",
    );
    insta::assert_debug_snapshot!(out);
}
