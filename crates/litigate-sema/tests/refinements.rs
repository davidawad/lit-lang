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

const GRAMMAR: &str = "a refinement is linear: comparisons (`<`, `<=`, `>`, `>=`, `==`, `!=`) of numbers, probabilities, hours, money, durations and dates, `+`, `-`, and `*` or `/` by a constant; `==`/`!=` on enums and `Bool`s; `&&`, `||`, `!` over those";

/// Each body's diagnostics, with the grammar note abbreviated.
fn table(rows: &[(&str, &[&str])]) {
    for (body, want) in rows {
        let got: Vec<String> = diagnostics(body)
            .into_iter()
            .map(|d| d.replace(GRAMMAR, "<grammar>"))
            .collect();
        assert_eq!(got, *want, "{body}");
    }
}

#[test]
fn predicates_outside_the_qualifier_grammar_are_l2011() {
    table(&[
        (
            "event E(a: Days<CalendarDays> where a * a > 1.days)",
            &["10:37 L2011 `*` of `Days<CalendarDays>` and `Days<CalendarDays>` is not linear: one factor must be a constant number | <grammar>"],
        ),
        (
            "event E(a: Days<CalendarDays> where a / a > 1.days)",
            &["10:37 L2011 `/` of `Days<CalendarDays>` and `Days<CalendarDays>` is not linear: one divisor must be a constant number | <grammar>"],
        ),
        (
            "event E(a: Days<CalendarDays> where deadline(x) > 1.days)",
            &["10:37 L2011 a call is outside the refinement qualifier grammar | <grammar>"],
        ),
        (
            "event E(a: Days<CalendarDays> where a > ?)",
            &["10:41 L2011 a hole is outside the refinement qualifier grammar | <grammar>"],
        ),
        (
            "event E(a: Number where a % 2 == 0)",
            &["10:25 L2011 `%` is outside the refinement qualifier grammar | <grammar>"],
        ),
        (
            "event E(m: Method where match m { Mail => true, _ => false })",
            &["10:25 L2011 a `match` is outside the refinement qualifier grammar | <grammar>"],
        ),
        (
            "event E(a: String where a == \"x\")",
            &["10:25 L2011 `==` on `String` is outside the refinement qualifier grammar | <grammar>"],
        ),
        (
            "event E(a: Bool where a < true)",
            &["10:23 L2011 `<` on `Bool` is outside the refinement qualifier grammar | <grammar>"],
        ),
        (
            "event E(a: Date where -a < 2024-01-01)",
            &["10:23 L2011 `-` on `Date` is outside the refinement qualifier grammar | <grammar>"],
        ),
        (
            "event E(a: Date where a + a < 2024-01-01)",
            &["10:23 L2011 `+` of `Date` and `Date` is outside the refinement qualifier grammar | <grammar>"],
        ),
        (
            "event E(a: Days<CalendarDays> where b > 1.days)",
            &["10:37 L2011 a refinement cannot name `b` | it may name only `a`, and constants"],
        ),
        (
            "event E(aa: Days<CalendarDays>, b: Days<CalendarDays> where ab > b)",
            &["10:61 L2011 a refinement cannot name `ab` | it may name only `b`, `aa`, and constants"],
        ),
        (
            "event E(m: Method where m == Method.Mail || m == r)",
            &["10:50 L2001 expected a `Method` variant, found `r` | variants: Personal, Mail"],
        ),
        (
            "event E(a: Date where a + 3.court_days > 2024-01-01)",
            &["10:23 L2011 a date moves by calendar days in a refinement, not by `Days<CourtDays>` | other units count through a counting rule, which a refinement does not run"],
        ),
        (
            "type L = List<{d: Days<CalendarDays> | d > 0.days}>",
            &["10:15 L2011 a refinement stands only as a parameter's or a type declaration's type, not as a type argument | declare `type T = {x: B | p}` and use `T` as the argument; its values are not checked inside a container"],
        ),
    ]);
}

#[test]
fn ill_typed_predicates_report_their_type_errors() {
    table(&[
        (
            "event E(a: Days<CalendarDays> where a > 1.court_days)",
            &["10:37 L2004 cannot compare `Days<CalendarDays>` with `Days<CourtDays>` | "],
        ),
        (
            "event E(a: Days<CalendarDays> where a > 2024-01-01)",
            &["10:37 L2001 cannot compare `Days<CalendarDays>` with `Date` | "],
        ),
        (
            "event E(a: Days<CalendarDays> where a)",
            &["10:37 L2001 `a` has type `Days<CalendarDays>`, not `Bool`: a refinement is a condition on it | compare it: `x <= 30.days`, `x == Variant`"],
        ),
        (
            "event E(a: Days<CalendarDays> where a / 0 > 1.days)",
            &["10:37 L2006 division by zero | "],
        ),
        (
            "event E(m: Method where m == Fax)",
            &["10:30 L2001 expected a `Method` variant, found `Fax` | variants: Personal, Mail"],
        ),
    ]);
}

#[test]
fn a_refinement_no_value_satisfies_is_l2013_with_an_unsat_core() {
    table(&[
        (
            "type Never = {d: Days<CalendarDays> | d > 60.days && d < 30.days}",
            &["10:14 L2013 no type `Never` satisfies its refinements: `d < 30.days` contradicts `d > 60.days` | 10:14 contradicted: `d > 60.days` | a value no program can write is a declaration error: relax or remove one of these refinements"],
        ),
        (
            "type Empty = Days<CalendarDays> where false",
            &["10:33 L2013 no type `Empty` satisfies its refinement `false`: the type is empty | a value no program can write is a declaration error: relax or remove one of these refinements"],
        ),
        (
            "event E(a: Days<CalendarDays> where a > 10.days, b: Days<CalendarDays> where b > a && a > 20.days && b < 15.days)",
            &["10:72 L2013 no `E(..)` satisfies its refinements: `b < 15.days` contradicts `b > a` and `a > 20.days` | 10:72 contradicted: `b > a` | 10:72 contradicted: `a > 20.days` | a value no program can write is a declaration error: relax or remove one of these refinements"],
        ),
        (
            "type Short = {d: Days<CalendarDays> | d <= 10.days}\nevent E(a: Short where a > 20.days)",
            &["11:18 L2013 no `E(..)` satisfies its refinements: `a > 20.days` contradicts `a <= 10.days` (from type `Short`) | 11:12 contradicted: `a <= 10.days` (from type `Short`) | a value no program can write is a declaration error: relax or remove one of these refinements"],
        ),
        (
            // An enum has only its variants.
            "event E(m: Method where m != Mail && m != Personal)",
            &["10:19 L2013 no `E(..)` satisfies its refinements: `m != Personal` contradicts `m != Mail` | 10:19 contradicted: `m != Mail` | a value no program can write is a declaration error: relax or remove one of these refinements"],
        ),
        (
            // A probability is in [0, 1], a `Bool` true or false, a date
            // one of a four-digit year.
            "event E(p: Probability where p > 1, b: Bool where b && !b, d: Date where d < 0001-01-01)",
            &["10:24 L2013 no `E(..)` satisfies its refinement `p > 1`: the type is empty | a value no program can write is a declaration error: relax or remove one of these refinements"],
        ),
    ]);
}

#[test]
fn an_alias_refinement_reaches_every_parameter_of_its_type() {
    // Chained aliases inherit, and a refinement of a parameter may read
    // the event's other parameters.
    table(&[
        (
            "type Short = {d: Days<CalendarDays> | d <= 10.days}
type Shorter = Short where it >= 8.days
event E(a: Shorter, b: Days<CalendarDays> where b >= a + 3.days && b < 11.days)",
            &["12:43 L2013 no `E(..)` satisfies its refinements: `b < 11.days` contradicts `a >= 8.days` (from type `Shorter`) and `b >= a + 3.days` | 12:12 contradicted: `a >= 8.days` (from type `Shorter`) | 12:43 contradicted: `b >= a + 3.days` | a value no program can write is a declaration error: relax or remove one of these refinements"],
        ),
        (
            "type Late = {d: Date | d >= 2024-01-01}\nevent E(a: Late, b: Date where b < a - 1_000.days && b > 2021-06-01)",
            &[],
        ),
    ]);
}

/// A case-free check of ground arguments: a pattern's or a test's event
/// call in a `.lit` file whose constant arguments violate a refinement.
#[test]
fn constant_arguments_of_a_refined_event_are_checked_exactly() {
    table(&[
        (
            "event Extended(days: Days<CalendarDays> where days <= 30.days, method: Method)
state x: \"X\" { _ \"go\" -> t { event Extended(45.days, Mail) } }",
            &["11:45 L2012 `Extended`: days = 45.days does not satisfy the refinement `days <= 30.days` | the refinement of parameter `days` of `Extended` (SPEC §3.5)"],
        ),
        (
            "type Cap = {d: Days<CalendarDays> | d <= 30.days}
event Extended(days: Cap, method: Method where method == Mail || days <= 3.days)
state x: \"X\" { _ \"go\" -> t { event Extended(10.days, Personal) } }
state y: \"Y\" { _ \"go\" -> t { event Extended(31.days) } }",
            &[
                "12:54 L2012 `Extended`: method = Personal, days = 10.days does not satisfy the refinement `method == Mail || days <= 3.days` | the refinement of parameter `method` of `Extended` (SPEC §3.5)",
                "13:45 L2012 `Extended`: days = 31.days does not satisfy the refinement `days <= 30.days` | the refinement of parameter `days` of `Extended`, from its type `Cap` (SPEC §3.5)",
            ],
        ),
        (
            // A pattern's omitted argument is open: the refinement reading it
            // is not decided there.
            "event Extended(days: Days<CalendarDays>, method: Method where method == Mail || days <= 3.days)
state x: \"X\" { _ \"go\" -> t { event Extended(10.days) } }",
            &[],
        ),
    ]);
}

/// A refined alias of another package keeps its refinement, its enum
/// variants read in the using package's view.
#[test]
fn a_refined_alias_of_a_dependency_is_checked_in_its_users_view() {
    let base = "module base
start s
enum Method { Personal, Mail }
type MailOnly = {m: Method | m == Mail}
type Cap = Number where it <= 30
default ceiling when true => 3
state s: \"S\" { _ \"go\" -> t }
terminal t: \"T\"
";
    let user = "module user
import base.{ Method, MailOnly, Cap }
import base
start s
event Sent(how: MailOnly, days: Cap)
event Both(how: MailOnly where how != Mail)
event Over(n: Number where n < base.ceiling)
state s: \"S\" { a \"A\" -> t { event Sent(Personal, 40) }, b \"B\" -> t { event Sent(Mail, 3) } }
terminal t: \"T\"
";
    let (db, ws) = workspace(&[
        package("base", &[], &[("main.lit", base)]),
        package("user", &["base"], &[("main.lit", user)]),
    ]);
    assert_eq!(
        common::render(&db, ws),
        [
            "pkgs/user/main.lit:6:26 L2013 no `Both(..)` satisfies its refinements: `how != Mail` contradicts `how == Mail` (from type `base::MailOnly`)",
            "pkgs/user/main.lit:7:32 L2011 `base.ceiling`, not a constant, is outside the refinement qualifier grammar",
            "pkgs/user/main.lit:8:40 L2012 `Sent`: how = Personal does not satisfy the refinement `how == Mail`",
            "pkgs/user/main.lit:8:50 L2012 `Sent`: days = 40 does not satisfy the refinement `days <= 30`",
        ]
    );
}

/// A question the SMT tier does not decide (here, exact arithmetic past
/// `i64` while building it) is a hole: L2014, a warning in the dev
/// profile and an error in strict, never a pass.
#[test]
fn an_undecided_refinement_is_a_hole() {
    let found = diagnostics("event E(a: Number where (a * 9_223_372_036_854_775_807) * 2 > 1)");
    assert_eq!(
        found,
        ["10:19 L2014 the SMT tier did not decide whether any `E(..)` satisfies its refinements: exact arithmetic overflowed i64 | an undecided refinement is a hole, never a pass: simplify the predicates (fewer disjunctions, smaller constants)"]
    );
    let text =
        format!("{PRELUDE}event E(a: Number where (a * 9_223_372_036_854_775_807) * 2 > 1)\n");
    let (db, ws) = workspace(&[package("demo", &[], &[("main.lit", &text)])]);
    let checks = litigate_sema::check_workspace(&db, ws);
    let d = &checks[0].diagnostics[0];
    assert_eq!(d.severity, litigate_base::Severity::Warning);
    assert!(
        checks[0].package.is_some(),
        "a hole does not stop the build"
    );
}
