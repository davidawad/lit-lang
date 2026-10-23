//! Verification of HIR that sema does not produce from source but that
//! `verify_package` accepts (a linked or future front end, a hand-built
//! package): each edit reaches a guard the evaluator or a checker keeps
//! for such inputs. Each snapshot holds the affected obligation's outcome.
mod support;

use litigate_sema::hir::{BinaryOp, DayUnit, Direction, Expr, ExprKind, Package, Type, Value};
use litigate_verify::{Obligation, ObligationStatus, Verification};
use serde_json::json;
use support::{deadlines, edit, outcome_of, tests, verify_edited, with, BASE};

fn status(v: &Verification, name: Obligation) -> Option<ObligationStatus> {
    v.report.finding(name).map(|f| f.status)
}

/// The first expectation of the first test.
fn first_expect(p: &mut Package) -> &mut litigate_sema::hir::Expectation {
    let Some(e) = tests(p).next().and_then(|t| t.expects.first_mut()) else {
        panic!("no expectation");
    };
    e
}

/// The callee and arguments of `e`, which must be a call.
fn call_parts(e: &mut Expr) -> (&mut String, &mut Vec<Expr>) {
    match &mut e.kind {
        ExprKind::Call { callee, args } => (callee, args),
        other => panic!("not a call: {other:?}"),
    }
}

fn literal(value: Value) -> Expr {
    Expr {
        ty: Type::Unit,
        kind: ExprKind::Literal { value },
        span: None,
    }
}

/// Verifies `lit` with `edit` applied and snapshots `tests-pass`.
fn tests_pass(lit: &str, edit: impl FnOnce(&mut Package)) -> serde_json::Value {
    let v = verify_edited(lit, edit);
    json!(outcome_of(&v, Obligation::TestsPass))
}

const HAS_EVENT: &str = "expect has_event(Served(complaint, Personal)) == true";

fn with_has_event() -> String {
    edit("expect deadline(answer-period) == 2025-03-24", HAS_EVENT)
}

#[test]
fn booleans_have_no_order() {
    insta::assert_json_snapshot!(tests_pass(&with_has_event(), |p| {
        first_expect(p).op = BinaryOp::Lt;
    }));
}

#[test]
fn deadline_needs_a_deadline_name() {
    insta::assert_json_snapshot!(tests_pass(BASE, |p| {
        let e = first_expect(p);
        let date = e.rhs.clone();
        *call_parts(&mut e.lhs).1 = vec![date];
    }));
}

#[test]
fn only_known_calls_are_evaluated() {
    insta::assert_json_snapshot!(tests_pass(BASE, |p| {
        *call_parts(&mut first_expect(p).lhs).0 = "elapsed".to_owned();
    }));
}

#[test]
fn has_event_needs_a_pattern() {
    insta::assert_json_snapshot!(tests_pass(&with_has_event(), |p| {
        call_parts(&mut first_expect(p).lhs).1.clear();
    }));
}

#[test]
fn a_reference_into_an_unknown_pack_is_not_a_deadline() {
    insta::assert_json_snapshot!(tests_pass(BASE, |p| {
        let (_, args) = call_parts(&mut first_expect(p).lhs);
        args[0].kind = ExprKind::Ref {
            target: "nowhere::answer-period".to_owned(),
        };
    }));
}

#[test]
fn a_trigger_date_the_clock_cannot_parse() {
    insta::assert_json_snapshot!(tests_pass(BASE, |p| {
        let test = tests(p).next().expect("a test");
        test.givens[0].1 = "2025-3-3".to_owned();
    }));
}

#[test]
fn a_period_longer_than_the_clock_counts() {
    insta::assert_json_snapshot!(tests_pass(BASE, |p| {
        deadlines(p).next().expect("a deadline").amount = u64::MAX;
    }));
}

#[test]
fn a_backward_period_counts_back_from_its_trigger() {
    insta::assert_json_snapshot!(tests_pass(BASE, |p| {
        deadlines(p).next().expect("a deadline").direction = Direction::Backward;
    }));
}

#[test]
fn only_variant_arguments_name_a_service_method() {
    let v = verify_edited(BASE, |p| {
        for test in tests(p) {
            let (_, args) = call_parts(&mut test.givens[0].0);
            args[1] = literal(Value::Date("2025-03-03".to_owned()));
        }
    });
    // Without its `Mail` argument, the mail test loses its three days.
    insta::assert_json_snapshot!(outcome_of(&v, Obligation::TestsPass));
}

/// `served` gains a second duty `reply-period` (3 days from the same
/// trigger, tested) bounding another edge.
fn two_duties() -> String {
    with(
        "rule duty reply-duty cite FRCP 12(a)\n{\n  deadline reply-period: 3.days from \
         Served(complaint) counted_by frcp6\n}\n\
         test \"reply\" cite FRCP 12(a)\n{\n  given Served(complaint, Personal) on 2025-03-03\n  \
         expect deadline(reply-period) == 2025-03-06\n}",
    )
    .replacen(
        "within answer-period }",
        "within answer-period }\n  reply \"Reply\" -> answered { event Answered(complaint), \
         within reply-period }",
        1,
    )
}

fn reply(p: &mut Package) -> &mut litigate_sema::hir::Deadline {
    let Some(d) = deadlines(p).find(|d| d.meta.id == "reply-period") else {
        panic!("no reply-period");
    };
    d
}

fn satisfiable(edit: impl FnOnce(&mut Package)) -> Verification {
    verify_edited(&two_duties(), edit)
}

#[test]
fn two_duties_on_one_trigger_are_satisfiable() {
    let v = satisfiable(|_| {});
    assert_eq!(
        status(&v, Obligation::DeadlineSatisfiable),
        Some(ObligationStatus::Passed)
    );
    assert!(v.is_accepted(), "{:#?}", v.diagnostics);
}

#[test]
fn a_backward_duty_on_a_forward_duty_trigger_clashes() {
    let clash = |unit: DayUnit| {
        let v = satisfiable(|p| {
            let r = reply(p);
            r.direction = Direction::Backward;
            r.unit = unit;
        });
        json!(outcome_of(&v, Obligation::DeadlineSatisfiable))
    };
    insta::assert_json_snapshot!(json!({
        "days": clash(DayUnit::Calendar),
        "court days": clash(DayUnit::Court),
        "business days": clash(DayUnit::Business),
    }));
}

#[test]
fn a_backward_duty_clashes_only_on_the_same_trigger_and_a_positive_length() {
    let zero = satisfiable(|p| {
        let r = reply(p);
        r.direction = Direction::Backward;
        r.amount = 0;
    });
    let other_trigger = satisfiable(|p| {
        let r = reply(p);
        r.direction = Direction::Backward;
        *call_parts(&mut r.trigger).0 = "Answered".to_owned();
    });
    for v in [zero, other_trigger] {
        assert_eq!(
            status(&v, Obligation::DeadlineSatisfiable),
            Some(ObligationStatus::Passed),
            "{:#?}",
            v.report
        );
    }
}

#[test]
fn a_duty_window_naming_no_deadline_is_not_checked() {
    let v = satisfiable(|p| {
        for d in support::declarations(p) {
            match d {
                litigate_sema::hir::Declaration::Rule(r) if r.meta.id == "reply-duty" => {
                    r.declarations.push("ghost".to_owned());
                }
                litigate_sema::hir::Declaration::Node(n) if n.meta.id == "served" => {
                    for e in &mut n.edges {
                        if let Some(d) = &mut e.deadline {
                            d.window = litigate_sema::hir::DeadlineWindow::Declared {
                                deadline: "ghost".to_owned(),
                            };
                        }
                    }
                }
                _ => {}
            }
        }
    });
    assert_eq!(
        status(&v, Obligation::DeadlineSatisfiable),
        Some(ObligationStatus::Passed)
    );
}

/// `answer-period` and `reply-period` as `copies` versions each, the first
/// in force only from 2000 and the second only before: no day has both.
fn disjoint_versions(copies: usize) -> Verification {
    satisfiable(|p| {
        let Some(module) = p.modules.first_mut() else {
            panic!("no module");
        };
        let mut extra = Vec::new();
        for d in &mut module.declarations {
            if let litigate_sema::hir::Declaration::Deadline(x) = d {
                match x.meta.id.as_str() {
                    "answer-period" => "2000-01-01".clone_into(&mut x.meta.effective.from),
                    "reply-period" => x.meta.effective.to = Some("2000-01-01".to_owned()),
                    _ => continue,
                }
                extra.extend(std::iter::repeat_n(d.clone(), copies - 1));
            }
        }
        module.declarations.extend(extra);
    })
}

#[test]
fn duties_never_in_force_together_are_unsatisfiable() {
    let v = disjoint_versions(2);
    let found = outcome_of(&v, Obligation::DeadlineSatisfiable);
    assert_eq!(
        found.finding.map(|f| f.status),
        Some(ObligationStatus::Failed)
    );
    insta::assert_json_snapshot!(found);
}

#[test]
fn an_exhausted_search_budget_accepts() {
    // 65 x 65 version combinations at one node exceed the 4096 searched:
    // the search stops and accepts rather than report a finding it did
    // not establish.
    let v = disjoint_versions(65);
    assert_eq!(
        status(&v, Obligation::DeadlineSatisfiable),
        Some(ObligationStatus::Passed)
    );
}

#[test]
fn a_finding_without_any_span_is_reported_at_the_manifest() {
    let untested = with(
        "rule duty reply-duty cite FRCP 12(a)\n{\n  deadline reply-period: 14.days from \
         Answered(complaint) counted_by frcp6\n}",
    );
    let v = verify_edited(&untested, |p| {
        let mut value = serde_json::to_value(&*p).expect("HIR serializes");
        strip_spans(&mut value);
        *p = serde_json::from_value(value).expect("HIR deserializes");
    });
    insta::assert_json_snapshot!(outcome_of(&v, Obligation::RuleWithoutTest));
}

fn strip_spans(v: &mut serde_json::Value) {
    match v {
        serde_json::Value::Object(map) => {
            if let Some(span) = map.get_mut("span") {
                *span = serde_json::Value::Null;
            }
            map.values_mut().for_each(strip_spans);
        }
        serde_json::Value::Array(items) => items.iter_mut().for_each(strip_spans),
        _ => {}
    }
}

/// A counting rule no loaded package declares is the shared library's,
/// as replay and `litigate test` count it, offering any engine service
/// method.
#[test]
fn an_undeclared_counting_rule_is_the_shared_librarys() {
    let v = verify_edited(BASE, |p| {
        for d in deadlines(p) {
            d.counting = "core::frcp6".to_owned();
        }
    });
    assert_eq!(
        status(&v, Obligation::TestsPass),
        Some(ObligationStatus::Passed),
        "{:#?}",
        outcome_of(&v, Obligation::TestsPass)
    );
}

/// Versions are selected over the package even when the caller lists only
/// its dependencies.
#[test]
fn tests_select_over_an_unlisted_package() {
    let v = support::verify_edited_in(BASE, |_| (), false);
    assert_eq!(
        status(&v, Obligation::TestsPass),
        Some(ObligationStatus::Passed),
        "{:#?}",
        outcome_of(&v, Obligation::TestsPass)
    );
}
