//! `tests-pass` (SPEC §7): every inline `test` assertion holds.
//!
//! `deadline(d)` is computed by litgraph's own clock interpreter
//! (`clock::compute_due_date`) running the counting rule exactly as
//! lowering emits it for the engine (SPEC §8.2), from the latest `given`
//! event matching `d`'s trigger, with that event's service method, under
//! the versions of `d` and of its counting rule in force on the trigger
//! date (the rule lowered at its own `from`, as `litigate test` does), so
//! a test can never pass here and the engine compute a different date.
//! Both are selected by sema's one query (`precedence::select` over the
//! versions `temporal::select` finds: a version, or a claimant displacing
//! it) in the law the package sees, among those recorded by the build's
//! `known_at`, as lowering selects them; a counting rule no loaded
//! package declares is the shared
//! library's. `has_event(p)` is true when a `given`
//! matches `p`. An assertion this evaluator cannot compute fails: an
//! unevaluated test is not a passing one.
use crate::diag::{emit, note};
use crate::{same_expr, Context};
use litgraph::clock::{self, ClockConfig, ClockLibrary, DayDeadline, Unit};
use litigate_base::{Diagnostic, DiagnosticCode as C};
use litigate_lower::Lowerer;
use litigate_sema::hir::{self, BinaryOp, Declaration, Expr, ExprKind, Value};
use litigate_sema::precedence::select::select;
use litigate_sema::temporal::{Key, World};
use std::cell::RefCell;
use std::collections::BTreeMap;

/// L5006 for every assertion that does not hold or cannot be evaluated,
/// with versions selected among those recorded by `known_at`.
pub(crate) fn failures(cx: &Context<'_>, known_at: &str) -> Vec<Diagnostic> {
    let clocks = Clocks::new(cx, known_at);
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
            due_field(cx, clocks, test, args, "adjusted")
        }
        ExprKind::Field { base, field } => match &base.kind {
            ExprKind::Call { callee, args } if callee == "deadline" => {
                due_field(cx, clocks, test, args, field)
            }
            _ => Err("only a due date's fields are evaluated".to_owned()),
        },
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
fn matches(pattern: &Expr, event: &Expr) -> bool {
    arguments(pattern, event).is_some()
}

/// The arguments of `event` when `pattern` matches it.
fn arguments<'e>(pattern: &Expr, event: &'e Expr) -> Option<&'e [Expr]> {
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
        ) => (p == e && pa.len() <= ea.len() && pa.iter().zip(ea).all(|(a, b)| same_expr(a, b)))
            .then_some(ea.as_slice()),
        _ => None,
    }
}

/// `deadline(d).field`: the `nominal` or (otherwise) `adjusted` date.
fn due_field(
    cx: &Context<'_>,
    clocks: &Clocks,
    test: &hir::Test,
    args: &[Expr],
    field: &str,
) -> Result<Value, String> {
    match args.first().map(|a| &a.kind) {
        Some(ExprKind::Ref { target }) => {
            let (nominal, adjusted) = due(cx, clocks, test, target)?;
            Ok(Value::Date(if field == "nominal" {
                nominal
            } else {
                adjusted
            }))
        }
        _ => Err("`deadline` takes a deadline name".to_owned()),
    }
}

/// The nominal and adjusted due dates of deadline `target` from the
/// matching `given` selected by its trigger policy, using the shared
/// version-selection query and the build's `known_at` date.
fn due(
    cx: &Context<'_>,
    clocks: &Clocks,
    test: &hir::Test,
    target: &str,
) -> Result<(String, String), String> {
    let versions = cx.deadline_versions(target);
    let first = versions
        .first()
        .ok_or_else(|| format!("`{target}` is not a deadline"))?;
    let matching = test
        .givens
        .iter()
        .enumerate()
        .filter_map(|(i, (g, d))| Some((i, g, arguments(&first.trigger, g)?, d.as_str())));
    type Given<'g> = (usize, &'g hir::Expr, &'g [hir::Expr], &'g str);
    let order = |(i, _, _, a): &Given<'_>, (j, _, _, b): &Given<'_>| a.cmp(b).then(i.cmp(j));
    // The trigger policy picks the event that runs the period (SPEC §5.3).
    let picked = if first.policy.is_first() {
        matching.min_by(order)
    } else {
        matching.max_by(order)
    };
    let (given, event, date) = picked
        .map(|(_, g, args, d)| (g, args, d))
        .ok_or_else(|| format!("no `given` event matches the trigger of `{target}`"))?;
    let (pkg, id) = cx.owner(target);
    let key = Key {
        pack: pkg.manifest.pack_id.clone(),
        name: id.to_owned(),
    };
    let Some((owner, Declaration::Deadline(deadline))) = clocks.select(&key, date) else {
        return Err(format!(
            "no version of `{target}` recorded by {} is in force on {date}",
            clocks.known_at
        ));
    };
    // A displacing claimant counts from its own trigger.
    if !matches(&deadline.trigger, given) {
        return Err(format!(
            "the given event does not match the trigger of `{}`",
            deadline.meta.id
        ));
    }
    let counting = counting_of(clocks, owner, deadline, date)?;
    let library = match counting {
        Some((pkg, c)) => clocks.get(pkg, c)?,
        None => ClockLibrary::embedded().0.clone(),
    };
    // A displacing counting rule counts under its own name.
    let rule = counting.map_or_else(
        || {
            deadline
                .counting
                .rsplit("::")
                .next()
                .unwrap_or(&deadline.counting)
        },
        |(_, c)| c.meta.id.as_str(),
    );
    let trigger = clock::Date::parse_iso(date).map_err(|e| e.to_string())?;
    let request = DayDeadline {
        trigger,
        days: i64::try_from(deadline.amount)
            .map_err(|_| format!("`{target}` is too long for the clock to count"))?,
        direction: match deadline.direction {
            hir::Direction::Forward => clock::Direction::Forward,
            hir::Direction::Backward => clock::Direction::Backward,
        },
        unit: match deadline.unit {
            hir::DayUnit::Calendar => Unit::Calendar,
            hir::DayUnit::Court | hir::DayUnit::Business => Unit::Court,
            hir::DayUnit::Week => Unit::Week,
            hir::DayUnit::Month => Unit::Month,
            hir::DayUnit::Year => Unit::Year,
        },
        service_method: service_method(counting.map(|(_, c)| c), event),
    };
    library
        .clock(rule)
        .and_then(|clock| clock::compute_due_date(&request, &clock, &ClockConfig::default()))
        .map(|c| (c.nominal_date, c.due_date))
        .map_err(|e| e.to_string())
}

/// The counting rule `deadline` (of `owner`) counts under on `date`, with
/// its package; `None` for one no loaded package declares (the shared
/// library's).
fn counting_of<'a>(
    clocks: &Clocks<'a>,
    owner: &hir::Package,
    deadline: &hir::Deadline,
    date: &str,
) -> Result<Option<(&'a hir::Package, &'a hir::Counting)>, String> {
    let key = clocks
        .world
        .target(&owner.manifest.pack_id, &deadline.counting)
        .key;
    if clocks.world.versions(&key).versions.is_empty() {
        return Ok(None);
    }
    match clocks.select(&key, date) {
        Some((pkg, Declaration::Counting(c))) => Ok(Some((pkg, c))),
        _ => Err(format!(
            "no version of counting rule `{}` recorded by {} is in force on {date}",
            deadline.counting, clocks.known_at
        )),
    }
}

/// The engine service method of the first event argument naming one of
/// the counting rule's `service_extension` methods (any engine method for
/// a shared-library rule).
fn service_method(c: Option<&hir::Counting>, args: &[Expr]) -> Option<clock::ServiceMethod> {
    args.iter().find_map(|a| match &a.kind {
        ExprKind::Literal {
            value: Value::Variant(v),
        } => {
            let name = v.rsplit("::").next().unwrap_or(v);
            c.is_none_or(|c| c.service_extensions.contains_key(name))
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

/// The loaded declarations, and engine clock libraries per counting-rule
/// version, lowered once each.
struct Clocks<'a> {
    world: World<'a>,
    packages: Vec<&'a hir::Package>,
    /// The corpus date versions are selected at.
    known_at: &'a str,
    lowered: RefCell<BTreeMap<(String, String), Result<ClockLibrary, String>>>,
}

impl<'a> Clocks<'a> {
    fn new(cx: &Context<'a>, known_at: &'a str) -> Self {
        let mut loaded = cx.packages.to_vec();
        if !loaded.iter().any(|p| std::ptr::eq(*p, cx.package)) {
            loaded.push(cx.package);
        }
        // The law the package sees (SPEC §5.5): a downstream package's
        // claims never change its tests.
        let packages = World::closure_packages(&[cx.package], &loaded);
        Self {
            world: World::from_packages(&packages),
            packages,
            known_at,
            lowered: RefCell::default(),
        }
    }

    /// The declaration governing `key` on `date` among those recorded by
    /// `known_at` (`precedence::select` over the versions
    /// `temporal::select` finds: a version of `key`, or a claimant
    /// displacing it), with its package.
    fn select(&self, key: &Key, date: &str) -> Option<(&'a hir::Package, &'a Declaration)> {
        let i = select(&self.world, key, date, Some(self.known_at)).ok()?;
        let d = &self.world.decls[i];
        Some((self.packages[d.pack], d.decl))
    }

    fn get(&self, pkg: &hir::Package, counting: &hir::Counting) -> Result<ClockLibrary, String> {
        let key = (
            format!("{}::{}", pkg.manifest.pack_id, counting.meta.id),
            counting.meta.effective.from.clone(),
        );
        if let Some(found) = self.lowered.borrow().get(&key) {
            return found.clone();
        }
        let lowered = self.lower(pkg, counting);
        self.lowered.borrow_mut().insert(key, lowered.clone());
        lowered
    }

    /// The clock fragment lowering emits for version `counting` of `pkg`,
    /// lowered at that version's `from` date. A selected version is the
    /// one in force on its own `from`, so it always has a fragment or
    /// diagnostics; were it to have neither, the empty library names no
    /// rule and the count fails there.
    fn lower(&self, pkg: &hir::Package, counting: &hir::Counting) -> Result<ClockLibrary, String> {
        Lowerer::new(&self.packages)
            .clock_version(pkg, counting)
            .map(Option::unwrap_or_default)
            .map_err(|ds| {
                ds.iter()
                    .map(|d| format!("{} {}", d.code.as_str(), d.message))
                    .collect::<Vec<_>>()
                    .join("; ")
            })
    }
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
        assert_eq!(compare(&a, BinaryOp::Le, &a), Some(true));
        assert_eq!(compare(&a, BinaryOp::Gt, &b), Some(false));
        assert_eq!(compare(&a, BinaryOp::Ge, &b), Some(false));
        assert_eq!(compare(&a, BinaryOp::Eq, &a), Some(true));
        assert_eq!(compare(&a, BinaryOp::Ne, &b), Some(true));
        assert_eq!(compare(&Value::Boolean(true), BinaryOp::Lt, &a), None);
    }

    #[test]
    fn only_comparisons_compare_dates() {
        let a = Value::Date("2025-03-24".into());
        for o in [BinaryOp::Add, BinaryOp::Sub, BinaryOp::And, BinaryOp::Or] {
            assert_eq!(compare(&a, o, &a), None, "{o:?}");
        }
    }

    #[test]
    fn operators_render_as_written() {
        let all = [
            BinaryOp::Eq,
            BinaryOp::Ne,
            BinaryOp::Lt,
            BinaryOp::Le,
            BinaryOp::Gt,
            BinaryOp::Ge,
            BinaryOp::Mul,
        ];
        let shown: Vec<&str> = all.into_iter().map(op).collect();
        assert_eq!(shown, ["==", "!=", "<", "<=", ">", ">=", "?"]);
    }

    fn expr(kind: ExprKind) -> Expr {
        Expr {
            ty: hir::Type::Unit,
            kind,
            span: None,
        }
    }

    fn variant(v: &str) -> Expr {
        expr(ExprKind::Literal {
            value: Value::Variant(v.to_owned()),
        })
    }

    fn call(callee: &str, args: &[&str]) -> Expr {
        expr(ExprKind::Call {
            callee: callee.to_owned(),
            args: args.iter().map(|a| variant(a)).collect(),
        })
    }

    #[test]
    fn a_pattern_matches_its_constructor_and_leading_arguments() {
        let event = call("Served", &["Document::complaint", "Method::Mail"]);
        assert!(matches(&call("Served", &[]), &event));
        assert!(matches(&call("Served", &["Document::complaint"]), &event));
        assert!(matches(&event, &event));
        assert!(!matches(
            &call("Answered", &["Document::complaint"]),
            &event
        ));
        assert!(!matches(&call("Served", &["Document::reply"]), &event));
        let longer = call("Served", &["Document::complaint", "Method::Mail", "X::y"]);
        assert!(!matches(&longer, &event));
        let literal = variant("Document::complaint");
        assert!(!matches(&literal, &event));
        assert!(!matches(&event, &literal));
    }

    #[test]
    fn values_render_plainly_and_others_as_json() {
        assert_eq!(render(&Value::Date("2025-03-24".into())), "2025-03-24");
        assert_eq!(render(&Value::Boolean(false)), "false");
        assert_eq!(
            render(&Value::Variant("Document::complaint".into())),
            r#"{"kind":"variant","value":"Document::complaint"}"#
        );
    }

    #[test]
    fn a_call_shows_its_reference_or_its_callee() {
        let v = Value::Boolean(true);
        let target = expr(ExprKind::Ref {
            target: "answer-period".into(),
        });
        let deadline = expr(ExprKind::Call {
            callee: "deadline".into(),
            args: vec![target],
        });
        assert_eq!(show(&deadline, &v), "deadline(answer-period)");
        assert_eq!(show(&call("has_event", &["E::x"]), &v), "has_event");
        assert_eq!(show(&call("has_event", &[]), &v), "has_event");
        assert_eq!(show(&variant("E::x"), &v), "true");
    }
}
