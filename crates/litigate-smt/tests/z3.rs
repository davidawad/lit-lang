//! The Z3 backend end to end: models, unsat cores, `unknown`, every
//! relation over both sorts, and reproducibility.
#![allow(clippy::unwrap_used)]
use litigate_smt::{
    Backend, Error, Formula, LinExpr, Model, Outcome, Query, Rational, DEFAULT_RLIMIT, Z3,
};

fn z3() -> Z3 {
    Z3::new().unwrap()
}

fn sat(q: &Query) -> Model {
    match z3().check(q).unwrap() {
        Outcome::Sat(m) => m,
        other => panic!("expected sat, got {other:?}"),
    }
}

fn core(q: &Query) -> Vec<String> {
    match z3().check(q).unwrap() {
        Outcome::Unsat(core) => core,
        other => panic!("expected unsat, got {other:?}"),
    }
}

fn r(n: i64, d: i64) -> Rational {
    Rational::new(n, d).unwrap()
}

#[test]
fn empty_query_is_sat_with_empty_model() {
    assert_eq!(sat(&Query::new()), Model::new());
}

#[test]
fn integer_model_respects_every_relation() {
    let mut q = Query::new();
    let x = q.int("x").unwrap();
    let y = q.int("y").unwrap();
    let z = q.int("z").unwrap();
    q.assert("x>=2", &Formula::ge(&x, 2)).unwrap();
    q.assert("x<=2", &Formula::le(&x, 2)).unwrap();
    q.assert("y>x", &Formula::gt(&y, &x)).unwrap();
    q.assert("y<4", &Formula::lt(&y, 4)).unwrap();
    q.assert("z=x+y", &Formula::eq(&z, LinExpr::from(&x).plus(1, &y)))
        .unwrap();
    q.assert("z!=0", &Formula::ne(&z, 0)).unwrap();
    let m = sat(&q);
    assert_eq!(m["x"], Rational::int(2));
    assert_eq!(m["y"], Rational::int(3));
    assert_eq!(m["z"], Rational::int(5));
}

#[test]
fn integers_are_not_rationals() {
    // 0 < 2x < 2 has a rational solution but no integer one.
    let mut q = Query::new();
    let x = q.int("x").unwrap();
    q.assert("lo", &Formula::lt(0, LinExpr::new().plus(2, &x)))
        .unwrap();
    q.assert("hi", &Formula::lt(LinExpr::new().plus(2, &x), 2))
        .unwrap();
    assert_eq!(core(&q), ["lo", "hi"]);

    let mut q = Query::new();
    let p = q.real("p").unwrap();
    q.assert("lo", &Formula::lt(0, LinExpr::new().plus(2, &p)))
        .unwrap();
    q.assert("hi", &Formula::lt(LinExpr::new().plus(2, &p), 2))
        .unwrap();
    let v = sat(&q)["p"];
    assert!(Rational::ZERO.checked_add(v).unwrap() != Rational::ZERO && !v.is_integer());
}

#[test]
fn rational_model_respects_every_relation() {
    // Probability q with 1/3 <= q, q < 1/2, q != 1/3, and q = 2p.
    let mut q = Query::new();
    let prob = q.real("q").unwrap();
    let p = q.real("p").unwrap();
    q.assert("ge", &Formula::ge(&prob, r(1, 3))).unwrap();
    q.assert("lt", &Formula::lt(&prob, r(1, 2))).unwrap();
    q.assert("ne", &Formula::ne(&prob, r(1, 3))).unwrap();
    q.assert("eq", &Formula::eq(&prob, LinExpr::new().plus(2, &p)))
        .unwrap();
    q.assert("gt", &Formula::gt(&p, 0)).unwrap();
    q.assert("le", &Formula::le(&p, 1)).unwrap();
    let m = sat(&q);
    let (vq, vp) = (m["q"], m["p"]);
    assert_eq!(vp.checked_mul(Rational::int(2)).unwrap(), vq);
    // 1/3 < q < 1/2, checked by cross-multiplication.
    assert!(3 * vq.numer() > vq.denom() && 2 * vq.numer() < vq.denom());
}

#[test]
fn mixed_int_and_real_atoms() {
    // A day count n and a rate: n * 1/2 + rate = 7/2, n >= 3, rate >= 1.
    let mut q = Query::new();
    let n = q.int("n").unwrap();
    let rate = q.real("rate").unwrap();
    q.assert(
        "sum",
        &Formula::eq(LinExpr::new().plus(r(1, 2), &n).plus(1, &rate), r(7, 2)),
    )
    .unwrap();
    q.assert("n", &Formula::ge(&n, 3)).unwrap();
    q.assert("rate", &Formula::ge(&rate, 1)).unwrap();
    let m = sat(&q);
    assert!(m["n"].is_integer());
    assert_eq!(
        m["n"]
            .checked_mul(r(1, 2))
            .unwrap()
            .checked_add(m["rate"])
            .unwrap(),
        r(7, 2)
    );
}

#[test]
fn boolean_structure() {
    // Disjoint intervals: [a, a+2) and [b, b+2) with a = 0 and
    // (a+2 <= b or b+2 <= a) and not b < 0  =>  b >= 2.
    let mut q = Query::new();
    let a = q.int("a").unwrap();
    let b = q.int("b").unwrap();
    q.assert("a=0", &Formula::eq(&a, 0)).unwrap();
    q.assert(
        "disjoint",
        &Formula::Or(vec![
            Formula::le(LinExpr::from(&a).plus_const(2), &b),
            Formula::le(LinExpr::from(&b).plus_const(2), &a),
        ]),
    )
    .unwrap();
    q.assert("b>=0", &!Formula::lt(&b, 0)).unwrap();
    q.assert("vacuous", &Formula::And(vec![])).unwrap();
    q.assert(
        "implies",
        &Formula::implies(Formula::eq(&a, 1), Formula::Or(vec![])),
    )
    .unwrap();
    assert!(sat(&q)["b"].numer() >= 2);

    q.assert("b<2", &Formula::lt(&b, 2)).unwrap();
    assert_eq!(core(&q), ["a=0", "disjoint", "b>=0", "b<2"]);
}

#[test]
fn empty_or_is_false() {
    let mut q = Query::new();
    q.assert("ok", &Formula::And(vec![])).unwrap();
    q.assert("false", &Formula::Or(vec![])).unwrap();
    assert_eq!(core(&q), ["false"]);
}

#[test]
fn unsat_core_is_minimal_and_ordered() {
    // Deadline arithmetic: served + 21 = due, due <= served + 14 conflict;
    // the irrelevant assertions stay out of the core.
    let mut q = Query::new();
    let served = q.int("served").unwrap();
    let due = q.int("due").unwrap();
    let other = q.real("other").unwrap();
    q.assert("other>=0", &Formula::ge(&other, 0)).unwrap();
    q.assert(
        "due=served+21",
        &Formula::eq(&due, LinExpr::from(&served).plus_const(21)),
    )
    .unwrap();
    q.assert("served>=0", &Formula::ge(&served, 0)).unwrap();
    q.assert(
        "due<=served+14",
        &Formula::le(&due, LinExpr::from(&served).plus_const(14)),
    )
    .unwrap();
    assert_eq!(core(&q), ["due=served+21", "due<=served+14"]);
}

#[test]
fn model_values_beyond_i64_are_an_error() {
    let mut q = Query::new();
    let x = q.int("x").unwrap();
    q.assert("x>=max", &Formula::ge(&x, i64::MAX)).unwrap();
    q.assert("x>max", &Formula::ne(&x, i64::MAX)).unwrap();
    assert_eq!(z3().check(&q), Err(Error::ValueOutOfRange("x".into())));

    let mut q = Query::new();
    let p = q.real("p").unwrap();
    q.assert("p>max", &Formula::gt(&p, i64::MAX)).unwrap();
    assert_eq!(z3().check(&q), Err(Error::ValueOutOfRange("p".into())));
}

#[test]
fn resource_limit_gives_unknown() {
    // A pigeonhole-style integer problem that needs real search.
    let mut q = Query::new();
    let xs: Vec<_> = (0..8).map(|i| q.int(&format!("x{i}")).unwrap()).collect();
    for (i, x) in xs.iter().enumerate() {
        q.assert(&format!("lo{i}"), &Formula::ge(x, 0)).unwrap();
        q.assert(&format!("hi{i}"), &Formula::lt(x, 7)).unwrap();
        for (j, y) in xs.iter().enumerate().skip(i + 1) {
            q.assert(&format!("ne{i}.{j}"), &Formula::ne(x, y)).unwrap();
        }
    }
    let Outcome::Unknown(reason) = Z3::with_rlimit(1).unwrap().check(&q).unwrap() else {
        panic!("expected unknown under rlimit 1");
    };
    assert!(!reason.is_empty());
    assert!(matches!(z3().check(&q).unwrap(), Outcome::Unsat(_)));
}

#[test]
fn answers_are_reproducible() {
    let build = || {
        let mut q = Query::new();
        let a = q.real("a").unwrap();
        let b = q.real("b").unwrap();
        let n = q.int("n").unwrap();
        q.assert("sum", &Formula::eq(LinExpr::from(&a).plus(1, &b), 1))
            .unwrap();
        q.assert("a", &Formula::gt(&a, 0)).unwrap();
        q.assert("b", &Formula::gt(&b, 0)).unwrap();
        q.assert("n", &Formula::gt(LinExpr::new().plus(3, &n), &a))
            .unwrap();
        q
    };
    let first = z3().check(&build()).unwrap();
    for _ in 0..5 {
        assert_eq!(z3().check(&build()).unwrap(), first);
    }
    // Also from another thread, in its own context.
    let other = std::thread::spawn(move || z3().check(&build()).unwrap())
        .join()
        .unwrap();
    assert_eq!(other, first);
}

#[test]
fn fingerprint_names_version_and_options() {
    assert_eq!(
        z3().fingerprint(),
        format!("z3 5.1.0 random_seed=0 threads=1 core.minimize=true rlimit={DEFAULT_RLIMIT}")
    );
    assert_ne!(
        Z3::with_rlimit(5).unwrap().fingerprint(),
        z3().fingerprint()
    );
}

#[test]
fn smtlib_text_replays_in_z3() {
    // The canonical text is valid SMT-LIB with the same answer.
    let mut q = Query::new();
    let d = q.int("filed day").unwrap();
    let p = q.real("p").unwrap();
    q.assert("window", &Formula::ge(&d, 30)).unwrap();
    q.assert(
        "prob",
        &Formula::lt(LinExpr::from(&p).plus(r(1, 3), &d), r(1, 2)),
    )
    .unwrap();
    q.assert(
        "p>=0",
        &Formula::Or(vec![Formula::ge(&p, 0), Formula::And(vec![])]),
    )
    .unwrap();
    let single = Formula::Or(vec![Formula::le(&p, 9)]);
    q.assert("one", &!Formula::And(vec![Formula::Or(vec![]), single]))
        .unwrap();
    q.assert("not", &!Formula::eq(&p, 1)).unwrap();
    let answer = |text: &str| {
        // Through the API, `produce-unsat-cores` turns each `:named`
        // assertion into `name => f` with `name` free (the CLI's
        // `check-sat` assumes the names); drop it to assert plainly.
        let solver = z3::Solver::new();
        solver.from_string(text.replace("(set-option :produce-unsat-cores true)\n", ""));
        solver.check()
    };
    assert_eq!(answer(&q.to_smtlib()), z3::SatResult::Sat);
    q.assert("p>=1", &Formula::ge(&p, 1)).unwrap();
    assert_eq!(answer(&q.to_smtlib()), z3::SatResult::Unsat);
    assert!(matches!(z3().check(&q).unwrap(), Outcome::Unsat(_)));
}
