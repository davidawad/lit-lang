//! `tests-pass` (SPEC §7): every inline `test` assertion holds.
//!
//! `deadline(d)` is computed by litgraph's own clock interpreter
//! (`clock::compute_due_date`) running the counting rule exactly as
//! lowering emits it for the engine (SPEC §8.2), from the latest `given`
//! event matching `d`'s trigger, with that event's service method. The
//! counting rule's lowering is reused by lowering its package with
//! `Pack.counting` naming the rule, so a test can never pass here and the
//! engine compute a different date. `has_event(p)` is true when a `given`
//! matches `p`. An assertion this evaluator cannot compute fails: an
//! unevaluated test is not a passing one.
use crate::diag::{emit, note};
use crate::{same_expr, Context};
use litgraph::clock::{self, ClockConfig, ClockLibrary, DayDeadline, Unit};
use litigate_base::{Diagnostic, DiagnosticCode as C};
use litigate_lower::Lowerer;
use litigate_sema::hir::{self, BinaryOp, Declaration, Expr, ExprKind, Value};
use std::cell::RefCell;
use std::collections::BTreeMap;

/// L5006 for every assertion that does not hold or cannot be evaluated.
pub(crate) fn failures(cx: &Context<'_>) -> Vec<Diagnostic> {
    let clocks = Clocks::default();
    cx.declarations()
        .filter_map(|d| match d {
            Declaration::Test(t) => Some(t),
            _ => None,
        })
        .flat_map(|t| {
            t.expects
                .iter()
                .filter_map(|e| expectation(cx, &clocks, t, e))
                .collect::<Vec<_>>()
        })
        .collect()
}

fn expectation(
    cx: &Context<'_>,
    clocks: &Clocks,
    test: &hir::Test,
    e: &hir::Expectation,
) -> Option<Diagnostic> {
    let span = cx.span_or_manifest(e.lhs.span.as_ref().or(test.span.as_ref()));
    let fail = |message: String| {
        note(
            emit(C::TestFailed, cx.profile, span.clone(), message),
            test.span.as_ref(),
            format!("in test \"{}\"", test.name),
        )
    };
    let lhs = eval(cx, clocks, test, &e.lhs);
    let rhs = eval(cx, clocks, test, &e.rhs);
    match (lhs, rhs) {
        (Ok(l), Ok(r)) => match compare(&l, e.op, &r) {
            Some(true) => None,
            Some(false) => Some(fail(format!(
                "expected {} {} {}, but it is {}",
                show(&e.lhs, &l),
                op(e.op),
                show(&e.rhs, &r),
                render(&l)
            ))),
            None => Some(fail(format!(
                "cannot compare {} {} {}",
                render(&l),
                op(e.op),
                render(&r)
            ))),
        },
        (Err(why), _) | (_, Err(why)) => {
            Some(fail(format!("cannot evaluate the assertion: {why}")))
        }
    }
}

/// Evaluates a test-side expression to a literal value.
fn eval(cx: &Context<'_>, clocks: &Clocks, test: &hir::Test, e: &Expr) -> Result<Value, String> {
    match &e.kind {
        ExprKind::Literal { value } => Ok(value.clone()),
        ExprKind::Call { callee, args } if callee == "deadline" => {
            match args.first().map(|a| &a.kind) {
                Some(ExprKind::Ref { target }) => due(cx, clocks, test, target).map(Value::Date),
                _ => Err("`deadline` takes a deadline name".to_owned()),
            }
        }
        ExprKind::Call { callee, args } if callee == "has_event" => match args.first() {
            Some(p) => Ok(Value::Boolean(
                test.givens.iter().any(|(g, _)| matches(p, g)),
            )),
            None => Err("`has_event` takes an event pattern".to_owned()),
        },
        _ => Err("only literals, `deadline(d)`, and `has_event(p)` are evaluated".to_owned()),
    }
}

fn compare(l: &Value, o: BinaryOp, r: &Value) -> Option<bool> {
    match (o, l, r) {
        (BinaryOp::Eq, _, _) => Some(l == r),
        (BinaryOp::Ne, _, _) => Some(l != r),
        (_, Value::Date(a), Value::Date(b)) => match o {
            BinaryOp::Lt => Some(a < b),
            BinaryOp::Le => Some(a <= b),
            BinaryOp::Gt => Some(a > b),
            BinaryOp::Ge => Some(a >= b),
            _ => None,
        },
        _ => None,
    }
}

/// `pattern` matches `event`: same constructor, and each argument the
/// pattern gives equals the event's (omitted trailing arguments match
/// anything).
pub(crate) fn matches(pattern: &Expr, event: &Expr) -> bool {
    match (&pattern.kind, &event.kind) {
        (
            ExprKind::Call {
                callee: p,
                args: pa,
            },
            ExprKind::Call {
                callee: e,
                args: ea,
            },
        ) => p == e && pa.len() <= ea.len() && pa.iter().zip(ea).all(|(a, b)| same_expr(a, b)),
        _ => false,
    }
}

/// The due date of deadline `target` from the test's latest matching
/// `given`.
fn due(
    cx: &Context<'_>,
    clocks: &Clocks,
    test: &hir::Test,
    target: &str,
) -> Result<String, String> {
    let versions = cx.deadline_versions(target);
    let first = versions
        .first()
        .ok_or_else(|| format!("`{target}` is not a deadline"))?;
    let (event, date) = test
        .givens
        .iter()
        .enumerate()
        .filter(|(_, (g, _))| matches(&first.trigger, g))
        .max_by(|(i, (_, a)), (j, (_, b))| a.cmp(b).then(i.cmp(j)))
        .map(|(_, (g, d))| (g, d.as_str()))
        .ok_or_else(|| format!("no `given` event matches the trigger of `{target}`"))?;
    let deadline = versions
        .iter()
        .find(|v| in_force(&v.meta.effective, date))
        .ok_or_else(|| format!("no version of `{target}` is in force on {date}"))?;
    let counting = cx
        .counting(&deadline.counting)
        .ok_or_else(|| format!("counting rule `{}` is not declared", deadline.counting))?;
    let library = clocks.get(cx, &deadline.counting)?;
    let clock = library
        .clock(&counting.meta.id)
        .map_err(|e| e.to_string())?;
    let trigger = clock::Date::parse_iso(date).map_err(|e| e.to_string())?;
    let request = DayDeadline {
        trigger,
        days: i64::try_from(deadline.amount).map_err(|e| e.to_string())?,
        direction: match deadline.direction {
            hir::Direction::Forward => clock::Direction::Forward,
            hir::Direction::Backward => clock::Direction::Backward,
        },
        unit: match deadline.unit {
            hir::DayUnit::Calendar => Unit::Calendar,
            hir::DayUnit::Court | hir::DayUnit::Business => Unit::Court,
        },
        service_method: service_method(counting, event),
    };
    clock::compute_due_date(&request, &clock, &ClockConfig::default())
        .map(|c| c.due_date)
        .map_err(|e| e.to_string())
}

fn in_force(i: &hir::Interval, date: &str) -> bool {
    i.from.as_str() <= date && i.to.as_deref().is_none_or(|to| date < to)
}

/// The engine service method of the first `event` argument naming one of
/// the counting rule's `service_extension` methods.
fn service_method(c: &hir::Counting, event: &Expr) -> Option<clock::ServiceMethod> {
    let ExprKind::Call { args, .. } = &event.kind else {
        return None;
    };
    args.iter().find_map(|a| match &a.kind {
        ExprKind::Literal {
            value: Value::Variant(v),
        } => {
            let name = v.rsplit("::").next().unwrap_or(v);
            c.service_extensions
                .contains_key(name)
                .then(|| kebab(name))
                .and_then(|k| serde_json::from_value(serde_json::Value::String(k)).ok())
        }
        _ => None,
    })
}

fn kebab(name: &str) -> String {
    name.chars()
        .enumerate()
        .flat_map(|(i, ch)| {
            let dash = (i > 0 && ch.is_ascii_uppercase()).then_some('-');
            dash.into_iter()
                .chain(std::iter::once(ch.to_ascii_lowercase()))
        })
        .collect()
}

/// Engine clock libraries per counting rule, lowered once each.
#[derive(Default)]
struct Clocks {
    lowered: RefCell<BTreeMap<String, Result<ClockLibrary, String>>>,
}

impl Clocks {
    fn get(&self, cx: &Context<'_>, counting: &str) -> Result<ClockLibrary, String> {
        if let Some(found) = self.lowered.borrow().get(counting) {
            return found.clone();
        }
        let lowered = lower_clock(cx, counting);
        self.lowered
            .borrow_mut()
            .insert(counting.to_owned(), lowered.clone());
        lowered
    }
}

/// The clock fragment lowering emits for the package declaring `counting`
/// when its pack names that rule.
fn lower_clock(cx: &Context<'_>, counting: &str) -> Result<ClockLibrary, String> {
    let (pkg, local) = cx.owner(counting);
    let mut named = pkg.clone();
    named.manifest.counting = Some(local.to_owned());
    // Lowering emits a declaration by identity, so the renamed copy takes
    // the original's place among the loaded packages.
    let packages: Vec<&hir::Package> = cx
        .packages
        .iter()
        .map(|p| {
            if p.manifest.pack_id == named.manifest.pack_id {
                &named
            } else {
                *p
            }
        })
        .collect();
    Lowerer::new(&packages)
        .package(&named, cx.options)
        .map_err(|ds| {
            ds.iter()
                .map(|d| format!("{} {}", d.code.as_str(), d.message))
                .collect::<Vec<_>>()
                .join("; ")
        })?
        .clock
        .map(|(library, _)| library)
        .ok_or_else(|| format!("counting rule `{counting}` has no engine lowering"))
}

fn show(e: &Expr, v: &Value) -> String {
    match &e.kind {
        ExprKind::Call { callee, args } => match args.first().map(|a| &a.kind) {
            Some(ExprKind::Ref { target }) => format!("{callee}({target})"),
            _ => callee.clone(),
        },
        _ => render(v),
    }
}

fn render(v: &Value) -> String {
    match v {
        Value::Date(d) => d.clone(),
        Value::Boolean(b) => b.to_string(),
        other => serde_json::to_string(other).unwrap_or_default(),
    }
}

fn op(o: BinaryOp) -> &'static str {
    match o {
        BinaryOp::Eq => "==",
        BinaryOp::Ne => "!=",
        BinaryOp::Lt => "<",
        BinaryOp::Le => "<=",
        BinaryOp::Gt => ">",
        BinaryOp::Ge => ">=",
        _ => "?",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn method_names_map_to_engine_spellings() {
        assert_eq!(kebab("Mail"), "mail");
        assert_eq!(kebab("OtherConsented"), "other-consented");
    }

    #[test]
    fn date_order_comparisons_are_iso_ordered() {
        let a = Value::Date("2025-03-24".into());
        let b = Value::Date("2025-03-27".into());
        assert_eq!(compare(&a, BinaryOp::Lt, &b), Some(true));
        assert_eq!(compare(&a, BinaryOp::Ge, &b), Some(false));
        assert_eq!(compare(&a, BinaryOp::Ne, &b), Some(true));
        assert_eq!(compare(&Value::Boolean(true), BinaryOp::Lt, &a), None);
    }
}
