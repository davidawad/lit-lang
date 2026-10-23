//! Calendar period units (weeks, months, years), the counting rule's
//! `end_of_month` policy, and the `Due` fields of `deadline(d)` (design
//! review 2026-10-23 §2.1; SPEC §5.3, §6).
mod common;
use common::{check_body, package, workspace};
use litigate_sema::hir::{
    DayUnit, DeadlineWindow, Declaration, EndOfMonth, Rational, TriangularDuration, Type,
};

/// The declarations of an error-free single-module package with `body`.
fn declarations(body: &str) -> Vec<Declaration> {
    let text = format!("{}{body}\n", common::PRELUDE);
    let (db, ws) = workspace(&[package("demo", &[], &[("main.lit", &text)])]);
    assert_eq!(common::render(&db, ws), Vec::<String>::new());
    litigate_sema::check_workspace(&db, ws)
        .into_iter()
        .flat_map(|c| c.package.iter())
        .flat_map(|p| p.modules.iter())
        .flat_map(|m| m.declarations.iter().cloned())
        .collect()
}

#[test]
fn counting_rules_declare_an_end_of_month_policy() {
    let policies: Vec<_> = declarations(
        "counting clamped { holidays cal end_of_month clamp }\n\
         counting rolled { holidays cal end_of_month roll }",
    )
    .into_iter()
    .filter_map(|d| match d {
        Declaration::Counting(c) => Some((c.meta.id, c.end_of_month)),
        _ => None,
    })
    .collect();
    assert_eq!(
        policies,
        [
            ("c6".to_owned(), None),
            ("clamped".to_owned(), Some(EndOfMonth::Clamp)),
            ("rolled".to_owned(), Some(EndOfMonth::Roll)),
        ]
    );
}

#[test]
fn deadlines_and_edges_carry_week_month_and_year_units() {
    let decls = declarations(
        "counting m { holidays cal end_of_month clamp }\n\
         deadline w: 2.weeks after Served() counted_by m\n\
         deadline mo: 9.months after Served() counted_by m\n\
         deadline y: 1.years after Served() counted_by m\n\
         state u: \"U\" { _ \"go\" -> t { deadline 3.months } }",
    );
    let units: Vec<_> = decls
        .iter()
        .filter_map(|d| match d {
            Declaration::Deadline(d) => Some((d.amount, d.unit)),
            _ => None,
        })
        .collect();
    assert_eq!(
        units,
        [(2, DayUnit::Week), (9, DayUnit::Month), (1, DayUnit::Year)]
    );
    let inline = decls.iter().find_map(|d| match d {
        Declaration::Node(n) if n.meta.id == "u" => n.edges[0].deadline.clone(),
        _ => None,
    });
    assert_eq!(
        inline.map(|d| d.window),
        Some(DeadlineWindow::Inline {
            length: 3,
            unit: DayUnit::Month
        })
    );
}

#[test]
fn a_month_deadline_needs_its_rule_to_declare_end_of_month() {
    let out = check_body(
        "deadline mo: 9.months after Served() counted_by c6\n\
         deadline y: 1.years after Served() counted_by nope\n\
         deadline w: 2.weeks after Served() counted_by c6\n\
         counting bad { holidays cal end_of_month sideways service_extension { Mail: 1.months } }",
    );
    insta::assert_debug_snapshot!(out);
}

#[test]
fn period_types_name_their_unit() {
    let out = check_body(
        "type W = Days<Weeks>\ntype M = Days<Months>\ntype Y = Days<Years>\n\
         state u: \"U\" {\n\
           _ \"a\" -> t { effort 2.weeks }\n\
           _ \"b\" -> t { effort 1.months }\n\
           _ \"c\" -> t { effort 1.years }\n\
         }\n\
         deadline q: 1.5.months after Served() counted_by c6",
    );
    insta::assert_debug_snapshot!(out);
}

#[test]
fn estimates_in_weeks_months_and_years_are_mean_gregorian_days() {
    let durations: Vec<_> = declarations(
        "state u: \"U\" {\n\
           _ \"a\" -> t { duration 2.weeks }\n\
           _ \"b\" -> t { duration { min 1.months, mode 1.years, max 6.years } }\n\
         }",
    )
    .into_iter()
    .filter_map(|d| match d {
        Declaration::Node(n) if n.meta.id == "u" => Some(n),
        _ => None,
    })
    .flat_map(|n| n.edges.into_iter().filter_map(|e| e.duration))
    .collect();
    let r = |numerator, denominator| Rational {
        numerator,
        denominator,
    };
    assert_eq!(
        durations,
        [
            TriangularDuration {
                min: None,
                mode: r(14, 1),
                max: None
            },
            TriangularDuration {
                min: Some(r(48_699, 1600)),
                mode: r(146_097, 400),
                max: Some(r(438_291, 200)),
            },
        ]
    );
}

#[test]
fn estimates_in_calendar_units_report_bad_amounts() {
    let out = check_body(
        "state u: \"U\" {\n\
           _ \"a\" -> t { duration 1.5.years }\n\
           _ \"b\" -> t { duration 100_000_000_000_000_000.years }\n\
         }",
    );
    insta::assert_debug_snapshot!(out);
}

#[test]
fn a_due_date_has_nominal_and_adjusted_fields() {
    let tests: Vec<_> = declarations(
        "deadline d: 21.days after Served() counted_by c6\n\
         test \"due\" {\n\
           given Served(complaint, Mail) on 2025-03-03\n\
           expect deadline(d).nominal == 2025-03-24\n\
           expect deadline(d).adjusted == 2025-03-24\n\
         }",
    )
    .into_iter()
    .filter_map(|d| match d {
        Declaration::Test(t) => Some(t),
        _ => None,
    })
    .collect();
    let types: Vec<_> = tests[0].expects.iter().map(|e| e.lhs.ty.clone()).collect();
    assert_eq!(types, [Type::Date, Type::Date]);
    let out = check_body(
        "deadline d: 21.days after Served() counted_by c6\n\
         test \"due\" {\n\
           given Served(complaint, Mail) on 2025-03-03\n\
           expect deadline(d).nominl == 2025-03-24\n\
           expect deadline(d).zzzzzz == 2025-03-24\n\
           expect has_event(Served(complaint, Mail)).nominal == true\n\
           expect { a: 2025-03-24 }.a.nominal == 2025-03-24\n\
         }",
    );
    insta::assert_debug_snapshot!(out);
}

#[test]
fn units_read_as_prose() {
    let prose: Vec<_> = [
        DayUnit::Calendar,
        DayUnit::Court,
        DayUnit::Business,
        DayUnit::Week,
        DayUnit::Month,
        DayUnit::Year,
    ]
    .iter()
    .flat_map(|u| [u.prose(1), u.prose(2)])
    .collect();
    assert_eq!(
        prose,
        [
            "1 calendar day",
            "2 calendar days",
            "1 court day",
            "2 court days",
            "1 business day",
            "2 business days",
            "1 week",
            "2 weeks",
            "1 month",
            "2 months",
            "1 year",
            "2 years",
        ]
    );
    assert!(DayUnit::Year.is_monthly() && !DayUnit::Week.is_monthly());
}
