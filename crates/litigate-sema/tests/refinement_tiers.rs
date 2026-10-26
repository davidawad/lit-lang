//! The two discharge tiers of refinements (SPEC §3.5) through their public
//! API, on predicates built directly: exact evaluation (`eval`) and the
//! SMT tier (`smt::decide`). Sema only hands them predicates in the
//! qualifier grammar; anything else is not decided (`None`, or `unknown`
//! with the reason), never a pass.
use litigate_sema::hir::{
    BinaryOp, DayUnit, Expr, ExprKind, Money, Rational, Type, UnaryOp, Value,
};
use litigate_sema::types::refinement::eval::{self, Val};
use litigate_sema::types::refinement::smt::{self, Decision, Var};

fn expr(ty: Type, kind: ExprKind) -> Expr {
    Expr {
        ty,
        kind,
        span: None,
    }
}

fn lit(ty: Type, value: Value) -> Expr {
    expr(ty, ExprKind::Literal { value })
}

fn num(n: i64) -> Expr {
    lit(Type::Number, Value::Number(Rational::integer(n)))
}

fn var(name: &str, ty: Type) -> Expr {
    expr(
        ty,
        ExprKind::Ref {
            target: name.to_owned(),
        },
    )
}

fn bin(op: BinaryOp, lhs: Expr, rhs: Expr) -> Expr {
    let ty = match op {
        BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div | BinaryOp::Rem => {
            lhs.ty.clone()
        }
        _ => Type::Boolean,
    };
    expr(
        ty,
        ExprKind::Binary {
            op,
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
        },
    )
}

fn un(op: UnaryOp, operand: Expr) -> Expr {
    let ty = operand.ty.clone();
    expr(
        ty,
        ExprKind::Unary {
            op,
            operand: Box::new(operand),
        },
    )
}

/// A field projection: outside the qualifier grammar.
fn field() -> Expr {
    expr(
        Type::Number,
        ExprKind::Field {
            base: Box::new(num(1)),
            field: "f".into(),
        },
    )
}

fn none(_: &str) -> Option<Val> {
    None
}

#[test]
#[allow(clippy::too_many_lines)] // one table row per operator
fn every_operator_evaluates_exactly() {
    let x = |n: i64| Some(Val::Num(Rational::integer(n)));
    let env = |name: &str| match name {
        "a" => x(3),
        "b" => x(4),
        "t" => Some(Val::Bool(true)),
        "m" => Some(Val::Variant("Mail".into())),
        _ => None,
    };
    let a = || var("a", Type::Number);
    let b = || var("b", Type::Number);
    let rows: Vec<(Expr, Option<bool>)> = vec![
        (bin(BinaryOp::Eq, a(), num(3)), Some(true)),
        (bin(BinaryOp::Ne, a(), num(3)), Some(false)),
        (bin(BinaryOp::Lt, a(), b()), Some(true)),
        (bin(BinaryOp::Le, b(), a()), Some(false)),
        (bin(BinaryOp::Gt, a(), b()), Some(false)),
        (bin(BinaryOp::Ge, b(), a()), Some(true)),
        (
            bin(BinaryOp::Eq, bin(BinaryOp::Add, a(), b()), num(7)),
            Some(true),
        ),
        (
            bin(BinaryOp::Eq, bin(BinaryOp::Sub, a(), b()), num(-1)),
            Some(true),
        ),
        (
            bin(BinaryOp::Eq, bin(BinaryOp::Mul, a(), b()), num(12)),
            Some(true),
        ),
        (
            bin(BinaryOp::Gt, bin(BinaryOp::Div, b(), a()), num(1)),
            Some(true),
        ),
        (bin(BinaryOp::Lt, un(UnaryOp::Neg, a()), num(0)), Some(true)),
        (un(UnaryOp::Not, var("t", Type::Boolean)), Some(false)),
        (
            bin(
                BinaryOp::Ne,
                var("m", Type::Named("M".into())),
                lit(
                    Type::Named("M".into()),
                    Value::Variant("pack::M::Mail".into()),
                ),
            ),
            Some(false),
        ),
        // `||` and `&&` decide from either side, and are open otherwise.
        (
            bin(
                BinaryOp::Or,
                var("u", Type::Boolean),
                var("t", Type::Boolean),
            ),
            Some(true),
        ),
        (
            bin(
                BinaryOp::And,
                var("t", Type::Boolean),
                var("u", Type::Boolean),
            ),
            None,
        ),
        (
            bin(
                BinaryOp::And,
                var("t", Type::Boolean),
                var("t", Type::Boolean),
            ),
            Some(true),
        ),
        // An unknown name, overflow, division by zero, and anything outside
        // the grammar are not decided.
        (bin(BinaryOp::Gt, var("u", Type::Number), num(0)), None),
        (bin(BinaryOp::Gt, a(), var("u", Type::Number)), None),
        (
            bin(
                BinaryOp::Gt,
                bin(BinaryOp::Mul, num(i64::MAX), num(2)),
                num(0),
            ),
            None,
        ),
        (
            bin(BinaryOp::Gt, bin(BinaryOp::Div, a(), num(0)), num(0)),
            None,
        ),
        (
            bin(BinaryOp::Gt, bin(BinaryOp::Rem, a(), b()), num(0)),
            None,
        ),
        (bin(BinaryOp::Lt, var("m", Type::Number), a()), None),
        (un(UnaryOp::Not, a()), None),
        (un(UnaryOp::Not, var("u", Type::Boolean)), None),
        (
            bin(BinaryOp::Lt, un(UnaryOp::Neg, var("u", Type::Number)), a()),
            None,
        ),
        (bin(BinaryOp::Lt, a(), var("m", Type::Number)), None),
        (
            bin(BinaryOp::Lt, un(UnaryOp::Neg, var("t", Type::Number)), a()),
            None,
        ),
        (bin(BinaryOp::Gt, field(), num(0)), None),
        (a(), None),
        (lit(Type::String, Value::String("s".into())), None),
        (lit(Type::Date, Value::Date("2024-13-01".into())), None),
    ];
    for (pred, want) in rows {
        assert_eq!(eval::eval(&pred, &env), want, "{}", eval::render(&pred));
    }
    assert_eq!(eval::eval(&num(1), &none), None);
}

#[test]
fn values_and_predicates_are_spelled_as_source() {
    let show = |ty: Type, v: Val| eval::show(&ty, &v);
    let n = |n: i64| Val::Num(Rational::integer(n));
    assert_eq!(show(Type::Date, n(0)), "1970-01-01");
    for (unit, text) in [
        (DayUnit::Calendar, "3.days"),
        (DayUnit::Court, "3.court_days"),
        (DayUnit::Business, "3.business_days"),
        (DayUnit::Week, "3.weeks"),
        (DayUnit::Month, "3.months"),
        (DayUnit::Year, "3.years"),
    ] {
        assert_eq!(show(Type::Days(unit), n(3)), text);
    }
    assert_eq!(show(Type::Hours, n(8)), "8.hours");
    let usd = || Type::Money("usd".into());
    assert_eq!(show(usd(), n(125_050)), "1250.50.usd");
    assert_eq!(show(usd(), n(-300)), "-3.usd");
    let half = Rational {
        numerator: 1,
        denominator: 2,
    };
    assert_eq!(show(usd(), Val::Num(half)), "1/2 minor units of usd");
    assert_eq!(show(Type::Number, Val::Num(half)), "1/2");
    assert_eq!(show(Type::Boolean, Val::Bool(true)), "true");
    assert_eq!(
        show(Type::Named("M".into()), Val::Variant("Mail".into())),
        "Mail"
    );
    let money = lit(
        usd(),
        Value::Money(Money {
            minor_units: 100,
            currency: "usd".into(),
        }),
    );
    let a = || var("a", Type::Number);
    let rows = [
        (bin(BinaryOp::Le, money, a()), "1.usd <= a"),
        (
            bin(
                BinaryOp::Lt,
                lit(Type::Date, Value::Date("2024-02-29".into())),
                var("d", Type::Date),
            ),
            "2024-02-29 < d",
        ),
        (
            bin(
                BinaryOp::Gt,
                bin(BinaryOp::Mul, num(2), bin(BinaryOp::Add, a(), num(1))),
                bin(BinaryOp::Sub, a(), bin(BinaryOp::Sub, a(), num(1))),
            ),
            "2 * (a + 1) > a - (a - 1)",
        ),
        (
            bin(
                BinaryOp::And,
                bin(
                    BinaryOp::Or,
                    var("p", Type::Boolean),
                    var("q", Type::Boolean),
                ),
                un(UnaryOp::Not, var("p", Type::Boolean)),
            ),
            "(p || q) && !p",
        ),
        (
            bin(
                BinaryOp::Lt,
                un(UnaryOp::Neg, a()),
                bin(BinaryOp::Rem, a(), num(2)),
            ),
            "-a < a % 2",
        ),
        (field(), "?"),
        (lit(Type::String, Value::String("s".into())), "?"),
    ];
    for (e, text) in rows {
        assert_eq!(eval::render(&e), text);
    }
}

fn v<'a>(name: &'a str, ty: &'a Type) -> Var<'a> {
    Var {
        name,
        ty,
        variants: None,
    }
}

#[test]
fn the_solver_tier_reports_a_witness_by_type() {
    let variants = ["Personal".to_owned(), "Mail".to_owned()];
    let (number, flag, method, plain) = (
        Type::Number,
        Type::Boolean,
        Type::Named("Method".into()),
        Type::Named("Other".into()),
    );
    let vars = [
        v("n", &number),
        v("f", &flag),
        Var {
            name: "m",
            ty: &method,
            variants: Some(&variants),
        },
        v("o", &plain),
    ];
    let preds = [
        bin(BinaryOp::Gt, var("n", Type::Number), num(2)),
        bin(BinaryOp::Lt, var("n", Type::Number), num(4)),
        var("f", Type::Boolean),
        bin(
            BinaryOp::Eq,
            var("f", Type::Boolean),
            lit(Type::Boolean, Value::Boolean(true)),
        ),
        bin(
            BinaryOp::Eq,
            lit(method.clone(), Value::Variant("Method::Mail".into())),
            var("m", method.clone()),
        ),
        bin(
            BinaryOp::Or,
            bin(
                BinaryOp::And,
                lit(Type::Boolean, Value::Boolean(true)),
                var("f", Type::Boolean),
            ),
            lit(Type::Boolean, Value::Boolean(false)),
        ),
    ];
    let Decision::Sat(model) = smt::decide(&vars, &preds) else {
        panic!("satisfiable");
    };
    assert_eq!(
        model,
        [
            ("n".to_owned(), "3".to_owned()),
            ("f".to_owned(), "true".to_owned()),
            ("m".to_owned(), "Mail".to_owned()),
            // An enum whose variants are not given has no spelling.
            ("o".to_owned(), String::new()),
        ]
    );
}

#[test]
#[allow(clippy::too_many_lines)] // one table row per case
fn a_question_outside_the_grammar_is_unknown_with_its_reason() {
    let (number, flag, date, method) = (
        Type::Number,
        Type::Boolean,
        Type::Date,
        Type::Named("Method".into()),
    );
    let n = || var("n", Type::Number);
    let big = |k: i64| num(k);
    let min = lit(
        Type::Number,
        Value::Number(Rational {
            numerator: i64::MIN,
            denominator: 1,
        }),
    );
    let huge = lit(
        Type::Number,
        Value::Number(Rational {
            numerator: 1,
            denominator: u64::MAX,
        }),
    );
    let mail = lit(method.clone(), Value::Variant("Method::Mail".into()));
    let gt0 = |e: Expr| bin(BinaryOp::Gt, e, num(0));
    let cases: Vec<(Vec<Var<'_>>, Expr)> = vec![
        // Not a predicate, not a term, an undeclared name.
        (vec![v("n", &number)], field()),
        (vec![v("n", &number)], gt0(field())),
        (vec![v("n", &number)], bin(BinaryOp::Gt, num(0), field())),
        (vec![v("n", &number)], un(UnaryOp::Not, field())),
        (
            vec![v("n", &number)],
            bin(BinaryOp::And, field(), var("f", Type::Boolean)),
        ),
        (
            vec![v("f", &flag)],
            bin(BinaryOp::Or, var("f", Type::Boolean), field()),
        ),
        (vec![], var("f", Type::Boolean)),
        (vec![], gt0(n())),
        (vec![v("n", &number)], gt0(un(UnaryOp::Neg, field()))),
        (vec![v("n", &number)], gt0(bin(BinaryOp::Add, field(), n()))),
        (vec![v("n", &number)], gt0(bin(BinaryOp::Add, n(), field()))),
        (
            vec![v("n", &number)],
            gt0(lit(Type::String, Value::String("s".into()))),
        ),
        // An enum constant without its variants, or not among them.
        (
            vec![v("m", &method)],
            bin(BinaryOp::Eq, var("m", method.clone()), mail.clone()),
        ),
        (
            vec![Var {
                name: "m",
                ty: &method,
                variants: Some(&[]),
            }],
            bin(BinaryOp::Eq, mail.clone(), var("m", method.clone())),
        ),
        // Not linear.
        (vec![v("n", &number)], gt0(bin(BinaryOp::Mul, n(), n()))),
        (vec![v("n", &number)], gt0(bin(BinaryOp::Div, num(1), n()))),
        (vec![v("n", &number)], gt0(bin(BinaryOp::Div, n(), num(0)))),
        // Exact arithmetic past i64, in each operation.
        (vec![v("n", &number)], gt0(huge)),
        (vec![v("n", &number)], gt0(un(UnaryOp::Neg, min.clone()))),
        (
            vec![v("n", &number)],
            gt0(bin(
                BinaryOp::Mul,
                big(i64::MAX),
                bin(BinaryOp::Add, n(), num(2)),
            )),
        ),
        (
            vec![v("n", &number)],
            gt0(bin(
                BinaryOp::Mul,
                bin(BinaryOp::Add, n(), num(2)),
                big(i64::MAX),
            )),
        ),
        (
            vec![v("n", &number)],
            gt0(bin(
                BinaryOp::Div,
                bin(BinaryOp::Add, n(), num(2)),
                lit(
                    Type::Number,
                    Value::Number(Rational {
                        numerator: 1,
                        denominator: 9_223_372_036_854_775_807,
                    }),
                ),
            )),
        ),
        (
            vec![v("n", &number)],
            gt0(bin(BinaryOp::Add, big(i64::MAX), big(i64::MAX))),
        ),
        (
            vec![v("n", &number)],
            gt0(bin(
                BinaryOp::Add,
                bin(BinaryOp::Mul, big(i64::MAX), n()),
                bin(BinaryOp::Mul, big(i64::MAX), n()),
            )),
        ),
        (
            vec![v("n", &number)],
            gt0(bin(BinaryOp::Sub, n(), min.clone())),
        ),
        (
            vec![v("n", &number)],
            gt0(bin(BinaryOp::Sub, big(-2), big(i64::MAX))),
        ),
        (vec![v("n", &number)], bin(BinaryOp::Le, n(), min)),
        // A name the solver cannot take, declared twice.
        (
            vec![v("a|b", &number)],
            lit(Type::Boolean, Value::Boolean(true)),
        ),
        (
            vec![v("a|b", &date)],
            lit(Type::Boolean, Value::Boolean(true)),
        ),
        (
            vec![v("d", &date), v("d", &date)],
            lit(Type::Boolean, Value::Boolean(true)),
        ),
    ];
    for (vars, pred) in cases {
        let text = eval::render(&pred);
        let decided = smt::decide(&vars, std::slice::from_ref(&pred));
        assert!(
            matches!(decided, Decision::Unknown(_)),
            "{text}: {decided:?}"
        );
    }
    // Out of its work budget, the solver answers unknown.
    let decided = smt::decide_within(1, &[v("n", &number)], &[gt0(n())]);
    assert!(matches!(decided, Decision::Unknown(_)), "{decided:?}");
}
