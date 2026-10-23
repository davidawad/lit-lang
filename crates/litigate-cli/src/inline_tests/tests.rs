//! Unit tests of `test`-block evaluation over hand-built expressions.
use super::*;
use crate::testkit::hir_with;
use litigate_sema::hir::Type;
use serde_json::json;

fn expr(kind: ExprKind) -> Expr {
    Expr {
        ty: Type::Unit,
        kind,
        span: None,
    }
}
fn lit(value: Value) -> Expr {
    expr(ExprKind::Literal { value })
}
fn num(n: i64) -> Expr {
    lit(Value::Number(hir::Rational {
        numerator: n,
        denominator: 1,
    }))
}
fn date(d: &str) -> Expr {
    lit(Value::Date(d.into()))
}
fn yes(b: bool) -> Expr {
    lit(Value::Boolean(b))
}
fn call(callee: &str, args: Vec<Expr>) -> Expr {
    expr(ExprKind::Call {
        callee: callee.into(),
        args,
    })
}
fn bin(op: BinaryOp, lhs: Expr, rhs: Expr) -> Expr {
    expr(ExprKind::Binary {
        op,
        lhs: Box::new(lhs),
        rhs: Box::new(rhs),
    })
}
fn not(operand: Expr) -> Expr {
    expr(ExprKind::Unary {
        op: UnaryOp::Not,
        operand: Box::new(operand),
    })
}
fn deadline(target: Expr) -> Expr {
    call("deadline", vec![target])
}
fn named(target: &str) -> Expr {
    expr(ExprKind::Ref {
        target: target.into(),
    })
}
fn bad() -> Expr {
    call("frobnicate", Vec::new())
}
fn served(on: &str) -> (Expr, String) {
    let complaint = lit(Value::Variant("Document::complaint".into()));
    let personal = lit(Value::Variant("ServiceMethod::Personal".into()));
    (call("Served", vec![complaint, personal]), on.into())
}

/// A labelled expression and the `given` events it is evaluated under.
type Form = (&'static str, Vec<(Expr, String)>, Expr);

/// `Served(complaint, Personal)` on each of `dates`.
fn on(dates: &[&str]) -> Vec<(Expr, String)> {
    dates.iter().map(|d| served(d)).collect()
}

fn forms() -> Vec<Form> {
    let pattern = call(
        "Served",
        vec![lit(Value::Variant("Document::complaint".into()))],
    );
    let period = || deadline(named("answer-period"));
    let (t, f) = (|| yes(true), || yes(false));
    let (d1, d2) = (|| date("2025-01-01"), || date("2025-01-02"));
    vec![
        ("bool", on(&[]), t()),
        ("number", on(&[]), num(3)),
        ("date", on(&[]), d2()),
        ("other literal", on(&[]), lit(Value::Unit)),
        (
            "has_event",
            on(&["2025-03-03"]),
            call("has_event", vec![pattern]),
        ),
        ("unknown call", on(&[]), bad()),
        ("deadline of a literal", on(&[]), deadline(num(1))),
        (
            "latest trigger",
            on(&["2025-03-01", "2025-03-03"]),
            period(),
        ),
        (
            "not a deadline",
            on(&["2025-03-03"]),
            deadline(named("nope")),
        ),
        ("no trigger", on(&[]), period()),
        ("no version", on(&["2000-01-01"]), period()),
        ("bad date", on(&["2025-13-45"]), period()),
        ("not", on(&[]), not(t())),
        ("not a number", on(&[]), not(num(1))),
        ("not an error", on(&[]), not(bad())),
        ("lhs error", on(&[]), bin(BinaryOp::Eq, bad(), num(1))),
        ("rhs error", on(&[]), bin(BinaryOp::Eq, num(1), bad())),
        ("true and false", on(&[]), bin(BinaryOp::And, t(), f())),
        ("false and true", on(&[]), bin(BinaryOp::And, f(), t())),
        ("true or false", on(&[]), bin(BinaryOp::Or, t(), f())),
        ("false or false", on(&[]), bin(BinaryOp::Or, f(), f())),
        ("eq", on(&[]), bin(BinaryOp::Eq, num(1), num(1))),
        ("ne", on(&[]), bin(BinaryOp::Ne, d1(), d2())),
        ("lt", on(&[]), bin(BinaryOp::Lt, num(1), num(2))),
        ("le", on(&[]), bin(BinaryOp::Le, d2(), d1())),
        ("gt", on(&[]), bin(BinaryOp::Gt, num(2), num(1))),
        ("ge", on(&[]), bin(BinaryOp::Ge, d1(), d1())),
        ("unordered", on(&[]), bin(BinaryOp::Lt, t(), f())),
        ("arithmetic", on(&[]), bin(BinaryOp::Add, num(1), num(2))),
        (
            "other form",
            on(&[]),
            expr(ExprKind::List { items: Vec::new() }),
        ),
    ]
}

#[test]
fn every_expression_form_evaluates_or_says_why() {
    let pkg = hir_with("");
    let clocks = Clocks::new(&[&pkg]);
    let world = clocks.scope(&[&pkg.manifest.name]);
    let results: Vec<(&str, Result<serde_json::Value, String>)> = forms()
        .into_iter()
        .map(|(name, givens, e)| {
            let mut cx = Cx {
                clocks: &clocks,
                world: &world,
                pkg: &pkg,
                known_at: &pkg.manifest.in_force.known_at,
                givens: &givens,
                steps: Vec::new(),
            };
            (name, cx.eval(&e).map(|v| v.json()))
        })
        .collect();
    insta::assert_debug_snapshot!(results);
}

fn spanned(e: Expr, line: u32) -> Expr {
    Expr {
        span: Some(Span {
            file: "t.lit".into(),
            line,
            col: 1,
            end_line: line,
            end_col: 4,
        }),
        ..e
    }
}

#[test]
fn expectations_carry_their_source_text_or_error() {
    let pkg = hir_with("");
    let clocks = Clocks::new(&[&pkg]);
    let world = clocks.scope(&[&pkg.manifest.name]);
    let mut cx = Cx {
        clocks: &clocks,
        world: &world,
        pkg: &pkg,
        known_at: &pkg.manifest.in_force.known_at,
        givens: &[],
        steps: Vec::new(),
    };
    let sources = [("t.lit".to_owned(), "abcdef\nghijkl\n".to_owned())];
    let check =
        |cx: &mut Cx<'_>, lhs: Expr, op: BinaryOp, rhs: Expr, sources: &[(String, String)]| {
            let (x, at) = expectation(cx, &hir::Expectation { lhs, op, rhs }, sources);
            (x.text, x.passed, x.error, at.map(|s| (s.line, s.end_line)))
        };
    assert_eq!(
        check(
            &mut cx,
            spanned(num(1), 1),
            BinaryOp::Eq,
            spanned(num(1), 2),
            &sources
        ),
        ("abcdef\nghi".to_owned(), true, None, Some((1, 2)))
    );
    assert_eq!(
        check(
            &mut cx,
            spanned(num(1), 1),
            BinaryOp::Eq,
            spanned(bad(), 1),
            &[]
        ),
        (
            String::new(),
            false,
            Some("`frobnicate(..)` cannot be evaluated in a test".to_owned()),
            Some((1, 1))
        )
    );
    assert_eq!(
        check(&mut cx, spanned(num(1), 1), BinaryOp::Add, num(1), &sources),
        (
            String::new(),
            false,
            Some("`Add` is not a test assertion operator".to_owned()),
            None
        )
    );
    assert_eq!(
        check(&mut cx, bad(), BinaryOp::Eq, num(1), &sources),
        (
            String::new(),
            false,
            Some("`frobnicate(..)` cannot be evaluated in a test".to_owned()),
            None
        )
    );
}

#[test]
fn run_filters_by_name_and_reports_spanless_failures_at_the_manifest() {
    let mut pkg = hir_with("");
    let (results, failures) = run(&Clocks::new(&[&pkg]), &pkg, &[], Some("mail"), None);
    let names: Vec<&str> = results.iter().map(|t| t.name.as_str()).collect();
    assert_eq!(
        (names, failures.len()),
        (vec!["mail service adds three days"], 0)
    );
    for decl in &mut pkg.modules[0].declarations {
        if let Declaration::Test(t) = decl {
            t.span = None;
            t.expects.push(hir::Expectation {
                lhs: bad(),
                op: BinaryOp::Eq,
                rhs: num(1),
            });
            t.expects.push(hir::Expectation {
                lhs: num(1),
                op: BinaryOp::Eq,
                rhs: num(2),
            });
        }
    }
    let (_, failures) = run(&Clocks::new(&[&pkg]), &pkg, &[], Some("mail"), None);
    let found: Vec<(&str, &str)> = failures
        .iter()
        .map(|d| (d.span.file.0.as_str(), d.message.as_str()))
        .collect();
    insta::assert_debug_snapshot!(found);
}

#[test]
fn the_manifest_beside_the_first_declaration() {
    let mut pkg = hir_with("");
    assert_eq!(nowhere(&pkg).file.0, "lit/examples/Lit.toml");
    let mut d = hirx::deadlines(&pkg, "answer-period")
        .next()
        .unwrap()
        .clone();
    d.meta.span = d.meta.span.map(|s| Span {
        file: "".into(),
        ..s
    });
    pkg.modules[0].declarations = vec![Declaration::Deadline(d)];
    assert_eq!(nowhere(&pkg).file.0, "Lit.toml");
    pkg.modules.clear();
    assert_eq!(nowhere(&pkg).file.0, "Lit.toml");
}

#[test]
fn values_as_json() {
    assert_eq!(V::Bool(true).json(), json!(true));
    assert_eq!(V::Other(json!({"a": 1})).json(), json!({"a": 1}));
}

/// The results of the tests of the worked example plus `extra` whose name
/// contains `filter`: (passed, first error).
fn outcomes(extra: &str, filter: &str) -> Vec<(bool, Option<String>)> {
    let pkg = hir_with(extra);
    let clocks = Clocks::new(&[&pkg]);
    let (results, _) = run(&clocks, &[&pkg], &pkg, &[], Some(filter));
    results
        .iter()
        .flat_map(|t| t.expectations.iter())
        .map(|x| (x.passed, x.error.clone()))
        .collect()
}

#[test]
fn deadline_counts_under_the_claimant_that_prevails() {
    // SPEC §5.5: `quick` displaces `answer-period`, so `deadline(..)` of
    // the latter is 14 days, not 21.
    let quick = "
deadline quick: 14.days from Served(complaint) counted_by frcp6 notwithstanding answer-period

test \"selected\"
{
  given Served(complaint, Personal) on 2025-03-03
  expect deadline(answer-period) == 2025-03-17
}
";
    assert_eq!(outcomes(quick, "selected"), [(true, None)]);
    // The selected claimant runs from its own trigger.
    let moved = "
deadline moved: 7.days from MovedToDismiss(complaint) counted_by frcp6 notwithstanding answer-period

test \"selected\"
{
  given Served(complaint, Personal) on 2025-03-03
  expect deadline(answer-period) == 2025-03-10
}
";
    assert_eq!(
        outcomes(moved, "selected"),
        [(
            false,
            Some(
                "no `given` event matches the trigger MovedToDismiss(complaint) of `answer-period`"
                    .to_owned()
            )
        )]
    );
}
