//! Structural equality, event patterns, evaluation, and satisfiability
//! over hand-built HIR expressions (every expression kind and connective).
use super::*;
use crate::hir::Pattern;

fn ex(ty: Type, kind: ExprKind) -> Expr {
    Expr {
        ty,
        kind,
        span: None,
    }
}

fn lit(b: bool) -> Expr {
    ex(
        Type::Boolean,
        ExprKind::Literal {
            value: Value::Boolean(b),
        },
    )
}

fn text(s: &str) -> Expr {
    ex(
        Type::String,
        ExprKind::Literal {
            value: Value::String(s.into()),
        },
    )
}

fn reference(target: &str) -> Expr {
    ex(
        Type::Boolean,
        ExprKind::Ref {
            target: target.into(),
        },
    )
}

fn event(name: &str, args: &[&str]) -> Expr {
    ex(
        Type::Named(name.into()),
        ExprKind::Call {
            callee: name.into(),
            args: args
                .iter()
                .map(|a| {
                    ex(
                        Type::Named("E".into()),
                        ExprKind::Literal {
                            value: Value::Variant((*a).into()),
                        },
                    )
                })
                .collect(),
        },
    )
}

fn has(name: &str, args: &[&str]) -> Expr {
    ex(
        Type::Boolean,
        ExprKind::Call {
            callee: "has_event".into(),
            args: vec![event(name, args)],
        },
    )
}

/// `e` (a call) with its callee renamed.
fn renamed(mut e: Expr, callee: &str) -> Expr {
    if let ExprKind::Call { callee: c, .. } = &mut e.kind {
        callee.clone_into(c);
    }
    e
}

fn bin(op: BinaryOp, a: Expr, b: Expr) -> Expr {
    ex(
        Type::Boolean,
        ExprKind::Binary {
            op,
            lhs: Box::new(a),
            rhs: Box::new(b),
        },
    )
}

fn unary(op: UnaryOp, a: Expr) -> Expr {
    ex(
        Type::Boolean,
        ExprKind::Unary {
            op,
            operand: Box::new(a),
        },
    )
}

fn not(a: Expr) -> Expr {
    unary(UnaryOp::Not, a)
}

fn record(fields: &[(&str, Expr)]) -> Expr {
    ex(
        Type::Named("R".into()),
        ExprKind::Record {
            fields: fields
                .iter()
                .map(|(n, e)| ((*n).to_owned(), e.clone()))
                .collect(),
        },
    )
}

fn list(items: Vec<Expr>) -> Expr {
    ex(Type::String, ExprKind::List { items })
}

fn field(base: Expr, name: &str) -> Expr {
    ex(
        Type::String,
        ExprKind::Field {
            base: Box::new(base),
            field: name.into(),
        },
    )
}

fn matching(scrutinee: Expr, arms: &[(Pattern, Expr)]) -> Expr {
    ex(
        Type::String,
        ExprKind::Match {
            scrutinee: Box::new(scrutinee),
            arms: arms.to_vec(),
        },
    )
}

/// Evaluates `e` with `atom` answering every atom by whether it is in
/// `facts` (and `None` for atoms named `unknown`).
fn run(e: &Expr, facts: &[&Expr]) -> Option<bool> {
    eval(e, &mut |a| {
        if same(a, &reference("unknown")) {
            None
        } else {
            Some(facts.iter().any(|f| same(f, a)))
        }
    })
}

#[test]
fn patterns_match_by_prefix() {
    assert!(matches(
        &event("Served", &["c"]),
        &event("Served", &["c", "Mail"])
    ));
    assert!(!matches(
        &event("Served", &["a"]),
        &event("Served", &["c", "Mail"])
    ));
    assert!(!matches(&event("Filed", &[]), &event("Served", &["c"])));
    assert!(!matches(
        &event("Served", &["c", "Mail", "x"]),
        &event("Served", &["c", "Mail"])
    ));
    assert!(!matches(&lit(true), &event("Served", &["c"])));
    assert!(!matches(&event("Served", &[]), &lit(true)));
}

#[test]
fn has_event_takes_exactly_one_pattern() {
    let served = event("Served", &["c"]);
    assert_eq!(has_event(&has("Served", &["c"])), Some(&served));
    let two = ex(
        Type::Boolean,
        ExprKind::Call {
            callee: "has_event".into(),
            args: vec![served.clone(), served.clone()],
        },
    );
    assert_eq!(has_event(&two), None);
    assert_eq!(has_event(&served), None);
    assert_eq!(has_event(&lit(true)), None);
}

#[test]
#[allow(clippy::too_many_lines)] // one row per kind and outcome
fn structural_equality_compares_every_kind_and_ignores_spans() {
    let a = has("Served", &["c"]);
    let mut spanned = a.clone();
    spanned.span = Some(crate::temporal::world::nowhere("elsewhere"));
    let r = |n: i64| record(&[("n", text(&n.to_string()))]);
    let arm = |p: &str, e: Expr| (Pattern::Variant(p.into()), e);
    let m = matching(
        text("x"),
        &[arm("a", text("1")), (Pattern::Wildcard, text("2"))],
    );
    // (left, right, equal)
    let cases: Vec<(Expr, Expr, bool)> = vec![
        (a.clone(), spanned, true),
        (lit(true), text("true"), false),
        (text("a"), text("a"), true),
        (text("a"), text("b"), false),
        (reference("x"), reference("x"), true),
        (reference("x"), reference("y"), false),
        (has("Served", &["c"]), has("Served", &["c"]), true),
        (event("Served", &["c"]), event("Filed", &["c"]), false),
        (event("Served", &["c"]), event("Served", &["c", "d"]), false),
        (event("Served", &["c"]), event("Served", &["d"]), false),
        (
            has("Served", &["c"]),
            renamed(has("Served", &["c"]), "has_no_event"),
            false,
        ),
        (not(lit(true)), not(lit(true)), true),
        (not(lit(true)), unary(UnaryOp::Neg, lit(true)), false),
        (not(lit(true)), not(lit(false)), false),
        (
            bin(BinaryOp::And, lit(true), lit(false)),
            bin(BinaryOp::And, lit(true), lit(false)),
            true,
        ),
        (
            bin(BinaryOp::And, lit(true), lit(false)),
            bin(BinaryOp::Or, lit(true), lit(false)),
            false,
        ),
        (
            bin(BinaryOp::And, lit(true), lit(false)),
            bin(BinaryOp::And, lit(false), lit(false)),
            false,
        ),
        (
            bin(BinaryOp::And, lit(true), lit(false)),
            bin(BinaryOp::And, lit(true), lit(true)),
            false,
        ),
        (r(1), r(1), true),
        (r(1), r(2), false),
        (r(1), record(&[("m", text("1"))]), false),
        (r(1), record(&[]), false),
        (list(vec![text("a")]), list(vec![text("a")]), true),
        (list(vec![text("a")]), list(vec![]), false),
        (field(r(1), "n"), field(r(1), "n"), true),
        (field(r(1), "n"), field(r(1), "m"), false),
        (field(r(1), "n"), field(r(2), "n"), false),
        (m.clone(), m.clone(), true),
        (m.clone(), matching(text("y"), &[]), false),
        (
            m.clone(),
            matching(text("x"), &[arm("a", text("1"))]),
            false,
        ),
        (
            m.clone(),
            matching(
                text("x"),
                &[arm("b", text("1")), (Pattern::Wildcard, text("2"))],
            ),
            false,
        ),
        (
            m.clone(),
            matching(
                text("x"),
                &[arm("a", text("9")), (Pattern::Wildcard, text("2"))],
            ),
            false,
        ),
        (m, list(vec![]), false),
    ];
    for (left, right, equal) in &cases {
        assert_eq!(same(left, right), *equal, "{left:?} vs {right:?}");
        assert_eq!(same(right, left), *equal, "{right:?} vs {left:?}");
    }
}

#[test]
fn evaluation_follows_the_connectives_and_asks_for_atoms() {
    let a = has("Ordered", &[]);
    let b = has("Served", &["c"]);
    let unknown = reference("unknown");
    let one = text("1");
    let lt = bin(BinaryOp::Lt, lit(true), lit(false));
    let mixed = ex(
        Type::Boolean,
        ExprKind::Binary {
            op: BinaryOp::Eq,
            lhs: Box::new(one.clone()),
            rhs: Box::new(one.clone()),
        },
    );
    let half = bin(BinaryOp::Eq, lit(true), one.clone());
    let cases: Vec<(Expr, Vec<&Expr>, Option<bool>)> = vec![
        (lit(true), vec![], Some(true)),
        (not(a.clone()), vec![], Some(true)),
        (not(unknown.clone()), vec![], None),
        (
            bin(BinaryOp::And, a.clone(), b.clone()),
            vec![&a, &b],
            Some(true),
        ),
        (
            bin(BinaryOp::And, a.clone(), b.clone()),
            vec![&b],
            Some(false),
        ),
        (
            bin(BinaryOp::And, a.clone(), b.clone()),
            vec![&a],
            Some(false),
        ),
        (
            bin(BinaryOp::Or, a.clone(), b.clone()),
            vec![&a],
            Some(true),
        ),
        (
            bin(BinaryOp::Or, a.clone(), b.clone()),
            vec![&b],
            Some(true),
        ),
        (bin(BinaryOp::Or, a.clone(), b.clone()), vec![], Some(false)),
        (bin(BinaryOp::Eq, a.clone(), b.clone()), vec![], Some(true)),
        (
            bin(BinaryOp::Eq, a.clone(), b.clone()),
            vec![&a],
            Some(false),
        ),
        (
            bin(BinaryOp::Ne, a.clone(), b.clone()),
            vec![&a],
            Some(true),
        ),
        (bin(BinaryOp::Ne, a.clone(), b.clone()), vec![], Some(false)),
        (
            bin(BinaryOp::And, unknown.clone(), b.clone()),
            vec![&b],
            None,
        ),
        (
            bin(BinaryOp::And, a.clone(), unknown.clone()),
            vec![&a],
            None,
        ),
        // Not a connective: the whole comparison is one atom.
        (lt.clone(), vec![&lt], Some(true)),
        (lt.clone(), vec![], Some(false)),
        (mixed.clone(), vec![&mixed], Some(true)),
        (half.clone(), vec![], Some(false)),
        (unary(UnaryOp::Neg, a.clone()), vec![], Some(false)),
        (b.clone(), vec![&b], Some(true)),
    ];
    for (e, facts, want) in &cases {
        assert_eq!(run(e, facts), *want, "{e:?} with {facts:?}");
    }
}

#[test]
fn unknown_atoms_follow_kleene_logic() {
    let a = has("Ordered", &[]);
    let b = has("Served", &["c"]);
    let unknown = reference("unknown");
    let cases: Vec<(Expr, Vec<&Expr>, Option<bool>)> = vec![
        (
            bin(BinaryOp::And, a.clone(), unknown.clone()),
            vec![],
            Some(false),
        ),
        (
            bin(BinaryOp::And, unknown.clone(), b.clone()),
            vec![],
            Some(false),
        ),
        (
            bin(BinaryOp::Or, a.clone(), unknown.clone()),
            vec![&a],
            Some(true),
        ),
        (
            bin(BinaryOp::Or, unknown.clone(), b.clone()),
            vec![&b],
            Some(true),
        ),
        (bin(BinaryOp::Or, unknown.clone(), b.clone()), vec![], None),
        (bin(BinaryOp::Eq, unknown.clone(), b.clone()), vec![], None),
        (bin(BinaryOp::Eq, a.clone(), unknown.clone()), vec![], None),
        (bin(BinaryOp::Ne, unknown.clone(), b.clone()), vec![], None),
        (bin(BinaryOp::Ne, a.clone(), unknown.clone()), vec![], None),
    ];
    for (e, facts, want) in &cases {
        assert_eq!(run(e, facts), *want, "{e:?} with {facts:?}");
    }
}

#[test]
fn atoms_are_distinct_and_stop_at_non_boolean_operands() {
    let a = has("Ordered", &[]);
    let b = has("Served", &["c"]);
    for op in [BinaryOp::And, BinaryOp::Or, BinaryOp::Eq, BinaryOp::Ne] {
        let both = bin(op, a.clone(), not(b.clone()));
        assert_eq!(atoms(&[&both, &a, &lit(true)]), vec![&a, &b], "{op:?}");
        let left = bin(op, text("x"), a.clone());
        assert_eq!(atoms(&[&left]), vec![&left], "{op:?}");
        let right = bin(op, a.clone(), text("x"));
        assert_eq!(atoms(&[&right]), vec![&right], "{op:?}");
    }
    let lt = bin(BinaryOp::Lt, a.clone(), b.clone());
    assert_eq!(atoms(&[&lt]), vec![&lt]);
}

#[test]
fn satisfiability_over_has_event_atoms() {
    let a = has("Ordered", &[]);
    let b = has("Served", &["c"]);
    let not_a = not(a.clone());
    assert_eq!(jointly_satisfiable(&[&a, &not_a]), Some(false));
    assert_eq!(jointly_satisfiable(&[&a, &b]), Some(true));
    let both = bin(BinaryOp::And, a.clone(), b.clone());
    let neither = not(bin(BinaryOp::Or, a.clone(), b));
    assert_eq!(jointly_satisfiable(&[&both, &neither]), Some(false));
    assert_eq!(jointly_satisfiable(&[&lit(false)]), Some(false));
    assert_eq!(atoms(&[&both, &a]).len(), 2);
    assert!(first_foreign(&both).is_none());
    let foreign = bin(BinaryOp::Or, a, reference("x"));
    assert_eq!(first_foreign(&foreign), Some(&reference("x")));
}

#[test]
fn too_many_atoms_are_not_enumerated() {
    let names: Vec<String> = (0..=MAX_ATOMS).map(|i| format!("E{i}")).collect();
    let many: Vec<Expr> = names.iter().map(|n| has(n, &[])).collect();
    let refs: Vec<&Expr> = many.iter().collect();
    assert_eq!(jointly_satisfiable(&refs), None);
    assert_eq!(jointly_satisfiable(&refs[1..]), Some(true));
}

fn date(d: &str) -> Expr {
    ex(
        Type::Date,
        ExprKind::Literal {
            value: Value::Date(d.into()),
        },
    )
}

fn call(callee: &str, arg: Expr) -> Expr {
    ex(
        Type::Date,
        ExprKind::Call {
            callee: callee.into(),
            args: vec![arg],
        },
    )
}

#[test]
fn fact_guards_read_events_and_compare_dates() {
    let filed = call("date_of", event("Filed", &[]));
    let due = call("deadline", reference("bar"));
    let ops = [
        BinaryOp::Eq,
        BinaryOp::Ne,
        BinaryOp::Lt,
        BinaryOp::Le,
        BinaryOp::Gt,
        BinaryOp::Ge,
    ];
    for op in ops {
        let g = bin(op, filed.clone(), due.clone());
        assert!(first_foreign_guard(&g).is_none(), "{op:?}");
        let g = bin(op, date("2020-01-01"), filed.clone());
        assert!(first_foreign_guard(&g).is_none(), "{op:?}");
    }
    let guard = bin(
        BinaryOp::And,
        not(has("Fixed", &[])),
        bin(BinaryOp::Le, filed.clone(), due.clone()),
    );
    assert!(first_foreign_guard(&guard).is_none());
    // Anything else is foreign: another atom, a non-date operand on
    // either side, or an operator that is not a comparison.
    let other = bin(BinaryOp::Or, has("Fixed", &[]), reference("x"));
    assert_eq!(first_foreign_guard(&other), Some(&reference("x")));
    let left = bin(BinaryOp::Lt, text("x"), due.clone());
    assert_eq!(first_foreign_guard(&left), Some(&left));
    let right = bin(BinaryOp::Lt, due.clone(), text("x"));
    assert_eq!(first_foreign_guard(&right), Some(&right));
    let not_date = bin(BinaryOp::Lt, call("len", due.clone()), due.clone());
    assert_eq!(first_foreign_guard(&not_date), Some(&not_date));
    let sum = bin(BinaryOp::Add, filed, due);
    assert_eq!(first_foreign_guard(&sum), Some(&sum));
}
