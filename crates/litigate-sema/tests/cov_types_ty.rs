//! Type expressions (`type X = T`, event parameter types) and type display
//! in diagnostics (SPEC §3.1 `Type`, §6), with diagnostics snapshotted.
mod common;
use common::check_body;

#[test]
fn every_builtin_type_constructor_resolves() {
    let out = check_body(
        "type A = Date\ntype B = Hours\ntype C = Probability\ntype D = Cite\ntype E = Role\ntype F = Bool\ntype G = Number\ntype H = String\ntype I = Unit\ntype J = Days<CalendarDays>\ntype K = Days<CourtDays>\ntype L = Days<BusinessDays>\ntype M = Money<usd>\ntype N = Interval<Date>\ntype O = List<Number>\ntype P = Dist<Method>\ntype Q = Method\ntype R = Served\ntype S = T2\ntype T2 = Date",
    );
    assert_eq!(out, Vec::<String>::new());
}

#[test]
fn type_constructors_check_their_argument_count() {
    insta::assert_debug_snapshot!(check_body(
        "type A = Date<Number>\ntype B = Days\ntype C = Days<CourtDays<Number>>\ntype D = Money\ntype E = Money<usd<Number>>\ntype F = Interval\ntype G = List\ntype H = Dist\ntype I = Method<Number>\ntype J = Nope<Number>"
    ));
}

#[test]
fn type_arguments_must_be_of_the_right_kind() {
    insta::assert_debug_snapshot!(check_body(
        "type A = Days<Foo>\ntype B = Money<USD>\ntype C = Money<eur>\ntype D = Interval<Number>\ntype E = Interval<Nope>\ntype F = List<Nope>\ntype G = Dist<Number>\ntype H = Dist<Served>\ntype I = Dist<Nope>\ntype J = Nope"
    ));
}

#[test]
fn types_display_as_source_syntax() {
    let out = check_body(
        "event P(span: Interval<Date>, l: List<Number>, d: Dist<Method>, r: Role, u: Unit, m: Money<usd>, b: Days<BusinessDays>)\ndefault a when has_event(P(1, 2, 3, 4, 5, 6, 7)) => 1\ndefault b when !{ x: 1, y: true } => 1\ndefault c when !unit => 1\ndefault d when true => given({ Personal: 1/4, Mail: 3/4 }, [Mail])\ndefault e when !d => 1\ndefault f when !Plaintiff => 1\ndefault g when ![1] => 1",
    );
    insta::assert_debug_snapshot!(out);
}
