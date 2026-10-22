//! Type expressions (`Days<CourtDays>`, `Dist<Outcome>`) to [`Type`], and
//! type display for diagnostics.
use crate::db::parse::tree::TypeRef;
use crate::db::span::Report;
use crate::hir::{DayUnit, Type};
use crate::resolve::{scope::Scope, sig::TypeSig};
use litigate_base::{Applicability, DiagnosticCode as C};

/// Built-in type constructor names.
pub const BUILTINS: [&str; 14] = [
    "Date",
    "Days",
    "Hours",
    "Money",
    "Probability",
    "Dist",
    "Cite",
    "Interval",
    "Role",
    "Bool",
    "Number",
    "String",
    "List",
    "Unit",
];

/// Spelling of a day unit as a type argument.
#[must_use]
pub const fn unit_name(unit: DayUnit) -> &'static str {
    match unit {
        DayUnit::Calendar => "CalendarDays",
        DayUnit::Court => "CourtDays",
        DayUnit::Business => "BusinessDays",
    }
}

/// Renders a type as source syntax.
#[must_use]
pub fn display(ty: &Type) -> String {
    match ty {
        Type::Date => "Date".into(),
        Type::Days(u) => format!("Days<{}>", unit_name(*u)),
        Type::Hours => "Hours".into(),
        Type::Money(c) => format!("Money<{c}>"),
        Type::Probability => "Probability".into(),
        Type::Distribution(e) => format!("Dist<{e}>"),
        Type::Cite => "Cite".into(),
        Type::DateInterval => "Interval<Date>".into(),
        Type::Role => "Role".into(),
        Type::Boolean => "Bool".into(),
        Type::Number => "Number".into(),
        Type::String => "String".into(),
        Type::List(t) => format!("List<{}>", display(t)),
        Type::Record(fields) => {
            let f: Vec<String> = fields
                .iter()
                .map(|(n, t)| format!("{n}: {}", display(t)))
                .collect();
            format!("{{ {} }}", f.join(", "))
        }
        Type::Named(n) => n.clone(),
        Type::Unit => "Unit".into(),
    }
}

fn arity(t: &TypeRef, n: usize, reports: &mut Vec<Report>) -> bool {
    let ok = t.args.len() == n;
    if !ok {
        reports.push(Report::new(
            C::TypeMismatch,
            t.range,
            format!(
                "`{}` takes {n} type argument{}, found {}",
                t.name.text,
                if n == 1 { "" } else { "s" },
                t.args.len()
            ),
        ));
    }
    ok
}

/// Resolves `t`, pushing L1002/L2001 reports; `None` when ill-formed.
pub fn resolve(scope: &Scope<'_>, t: &TypeRef, reports: &mut Vec<Report>) -> Option<Type> {
    let simple = |ty: Type, reports: &mut Vec<Report>| arity(t, 0, reports).then_some(ty);
    match t.name.text.as_str() {
        "Date" => simple(Type::Date, reports),
        "Hours" => simple(Type::Hours, reports),
        "Probability" => simple(Type::Probability, reports),
        "Cite" => simple(Type::Cite, reports),
        "Role" => simple(Type::Role, reports),
        "Bool" => simple(Type::Boolean, reports),
        "Number" => simple(Type::Number, reports),
        "String" => simple(Type::String, reports),
        "Unit" => simple(Type::Unit, reports),
        "Days" => {
            let arg = arity(t, 1, reports).then(|| &t.args[0])?;
            let unit = match arg.name.text.as_str() {
                "CalendarDays" => DayUnit::Calendar,
                "CourtDays" => DayUnit::Court,
                "BusinessDays" => DayUnit::Business,
                other => {
                    reports.push(
                        Report::new(
                            C::TypeMismatch,
                            arg.range,
                            format!("`{other}` is not a day unit"),
                        )
                        .note("expected `CalendarDays`, `CourtDays`, or `BusinessDays`"),
                    );
                    return None;
                }
            };
            arity(arg, 0, reports).then_some(Type::Days(unit))
        }
        "Money" => {
            let arg = arity(t, 1, reports).then(|| &t.args[0])?;
            if arg.name.text == "usd" && arg.args.is_empty() {
                return Some(Type::Money("usd".into()));
            }
            let mut report = Report::new(
                C::UnresolvedName,
                arg.range,
                format!("unknown currency `{}`", arg.name.text),
            )
            .note("the only currency in language version 1 is `usd`");
            if arg.name.text.eq_ignore_ascii_case("usd") {
                report = report.fix(arg.name.range, "usd", Applicability::MachineApplicable);
            }
            reports.push(report);
            None
        }
        "Interval" => {
            let arg = arity(t, 1, reports).then(|| &t.args[0])?;
            if resolve(scope, arg, reports)? == Type::Date {
                Some(Type::DateInterval)
            } else {
                reports.push(Report::new(
                    C::TypeMismatch,
                    arg.range,
                    "only `Interval<Date>` exists",
                ));
                None
            }
        }
        "List" => {
            let arg = arity(t, 1, reports).then(|| &t.args[0])?;
            resolve(scope, arg, reports).map(|inner| Type::List(Box::new(inner)))
        }
        "Dist" => {
            let arg = arity(t, 1, reports).then(|| &t.args[0])?;
            match resolve(scope, arg, reports)? {
                Type::Named(n) if scope.enum_variants(&n).is_some() => Some(Type::Distribution(n)),
                other => {
                    reports.push(Report::new(
                        C::TypeMismatch,
                        arg.range,
                        format!("`Dist` needs an enum, found `{}`", display(&other)),
                    ));
                    None
                }
            }
        }
        _ => {
            let resolved = scope.user_type(&t.name, &BUILTINS);
            if resolved.is_ok() && !arity(t, 0, reports) {
                return None;
            }
            match resolved {
                Ok((canonical, found)) => match found.item {
                    TypeSig::Alias(target) => target.clone(),
                    TypeSig::Enum(_) | TypeSig::Event(_) => Some(Type::Named(canonical)),
                },
                Err(report) => {
                    reports.push(report);
                    None
                }
            }
        }
    }
}
