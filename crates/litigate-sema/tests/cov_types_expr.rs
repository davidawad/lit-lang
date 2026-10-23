//! Expression checking (SPEC §6): literal inference, checking against an
//! expected type, names in expression position, and event constructor
//! calls, with diagnostics (messages, notes, fixes) snapshotted.
mod common;
use common::{package, workspace, PRELUDE};
use std::fmt::Write as _;

/// Every diagnostic of the single-module package `PRELUDE + body`, with
/// its notes and fixes.
fn check(body: &str) -> Vec<String> {
    let text = format!("{PRELUDE}{body}\n");
    diagnostics(&[("main.lit", &text)])
}

fn diagnostics(files: &[(&str, &str)]) -> Vec<String> {
    let (db, ws) = workspace(&[package("demo", &[], files)]);
    litigate_sema::check_workspace(&db, ws)
        .into_iter()
        .flat_map(|c| c.diagnostics.iter())
        .map(|d| {
            let mut line = format!(
                "{}:{}:{} {} {}",
                d.span.file.0,
                d.span.line,
                d.span.col,
                d.code.as_str(),
                d.message
            );
            for n in &d.notes {
                let _ = write!(line, " [note: {}]", n.message);
            }
            for f in &d.fixes {
                let _ = write!(line, " [fix: {:?}]", f.replacement);
            }
            line
        })
        .collect()
}

#[test]
fn literals_infer_their_types() {
    let out = check(
        "default a when true => 99999999999999999999\ndefault b when cite FRCP 6\n => 1\ndefault c when true => ?\ndefault d when (true) => (1)\ndefault e when true => unit\ndefault f when true => \"text\"\ndefault g when true => 2025-01-01\ndefault h when true => 8.hours",
    );
    insta::assert_debug_snapshot!(out);
}

#[test]
fn numbers_coerce_to_constant_probabilities_only() {
    let out = check(
        "default n when true => 1\ndefault d when true => given({ Personal: 1/4, Mail: 3/4 }, [Mail])\ndefault ok when prob(d, Mail) == 1/2 => 1\ndefault high when prob(d, Mail) == 3/2 => 1\ndefault open when prob(d, Mail) == n => 1",
    );
    insta::assert_debug_snapshot!(out);
}

#[test]
fn arguments_are_checked_against_parameter_types() {
    let out = check(
        "event P(d: Dist<Method>, m: Method, b: Days<BusinessDays>, c: Bool)\ndefault a when has_event(P({ Personal: 1/2, Mail: 1/2 }, Method.Mail, 1.business_days, match Method.Mail { Personal => true, Mail => false })) => 1\ndefault b when has_event(P({ Personal: 1/2, Mail: 1/2 }, Mal, 1.days, (true))) => 1\ndefault c when has_event(P({ Personal: 1/2, Mail: 1/2 }, Zzzzzz, 1.court_days, true)) => 1\ndefault d when has_event(P({ Personal: 1/2, Mail: 1/2 }, Foo.Mail, 1.business_days, true)) => 1\nevent Q(s: Served)\ndefault e when has_event(Q(x)) => 1",
    );
    insta::assert_debug_snapshot!(out);
}

#[test]
fn qualified_variants_resolve_through_enums_and_aliases() {
    let out = check(
        "type M = Method\ntype S = Served\ndefault a when true => Method.Mail\ndefault b when true => M.Personal\ndefault c when true => Method.Fax\ndefault d when true => Method@x.Mail\ndefault e when true => Served.Mail\ndefault f when true => S.Mail\ndefault g when true => Nope.Mail",
    );
    insta::assert_debug_snapshot!(out);
}

#[test]
fn names_in_expression_position_must_be_values() {
    let out = check(
        "deadline ok: 21.days from Served(complaint) counted_by c6\ndefault n when true => 1\ndefault a when ok => 1\ndefault b when s => 1\ndefault c when n == 1 => n\ndefault d when Plaintiff == Defendant => 1\ndefault e when true => n-n\ndefault f when true => n-x\ndefault h when true => x-n\ndefault g when true => Method.Mail.x",
    );
    insta::assert_debug_snapshot!(out);
}

#[test]
fn calls_are_builtins_or_event_constructors() {
    let out = check(
        "event One(a: Method)\nevent Bad(x: Nope)\ndefault a when foo.bar(1) => 1\ndefault b when Method(1) => 1\ndefault c when Served(complaint, Mail, 3) => 1\ndefault d when has_event(One()) => 1\ndefault e when has_event(Bad()) => 1\ndefault f when Nope(1) => 1\ndeadline g: 21.days from Served(complaint, Mail, 3) counted_by c6",
    );
    insta::assert_debug_snapshot!(out);
}

#[test]
fn names_whose_import_failed_report_only_at_the_import() {
    let main = "module main\nimport rules.{gone}\nstart s\nterminal s: \"S\"\ndefault a when gone => 1\ndefault b when gone(1) => 1\n";
    let out = diagnostics(&[("main.lit", main), ("rules.lit", "module rules\n")]);
    insta::assert_debug_snapshot!(out);
}
