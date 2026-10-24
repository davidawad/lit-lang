//! `litigate_sema::guard` over HIR built by hand (SPEC §3.6): the
//! canonical order of guard operations, their composition, the pairs no
//! order decides, respelling into the base pack's view, guard text, the
//! provenance record, and the package checks over HIR without spans.
mod cov_case_support;

use litigate_sema::guard::{self, Applied};
use litigate_sema::hir::{
    Authored, BinaryOp, Declaration, Expr, ExprKind, GuardOp, GuardOpKind, Interval, Type,
    UnaryOp, Value,
};
use litigate_sema::precedence::Lattice;
use litigate_sema::temporal::world::{Rank, World};

fn ex(ty: Type, kind: ExprKind) -> Expr {
    Expr {
        ty,
        kind,
        span: None,
    }
}

fn call(callee: &str, args: Vec<Expr>) -> Expr {
    ex(
        Type::Boolean,
        ExprKind::Call {
            callee: callee.to_owned(),
            args,
        },
    )
}

/// `has_event(<event>())`.
fn has(event: &str) -> Expr {
    call("has_event", vec![call(event, Vec::new())])
}

fn bin(op: BinaryOp, lhs: Expr, rhs: Expr) -> Expr {
    ex(
        Type::Boolean,
        ExprKind::Binary {
            op,
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
        },
    )
}

fn op(kind: GuardOpKind, condition: Expr, by: &str, from: &str, to: Option<&str>) -> GuardOp {
    GuardOp {
        kind,
        condition,
        by: by.to_owned(),
        cite: Authored::Known("Order, Dkt. 12".to_owned()),
        effective: Interval {
            from: from.to_owned(),
            to: to.map(str::to_owned),
            known_at: "2025-01-01".to_owned(),
        },
        span: None,
    }
}

/// `High > Low`; `Other` is incomparable to both.
fn lattice() -> Lattice {
    Lattice::new(&[Rank {
        above: "High".to_owned(),
        below: "Low".to_owned(),
        pack: 0,
    }])
}

#[test]
fn composition_orders_by_authority_then_date() {
    let l = lattice();
    let replace = op(GuardOpKind::Replace, has("A"), "Low", "2011-01-01", None);
    let narrow = op(GuardOpKind::Narrow, has("B"), "High", "2010-01-01", None);
    let widen = op(GuardOpKind::Widen, has("C"), "High", "2012-01-01", Some("2013-01-01"));
    let ops = [&narrow, &widen, &replace];
    assert_eq!(guard::order(&l, &ops), [2, 0, 1]);
    // The base is replaced first, so `B` and `C` apply to `A`.
    let c = guard::compose(&l, Some(&has("Z")), &ops, "2012-06-01", None);
    assert_eq!(c.applied, [2, 0, 1]);
    assert_eq!(
        c.guard.as_ref().map(guard::text),
        Some("(has_event(A()) && has_event(B())) || has_event(C())".to_owned())
    );
    // Out of every interval, or not yet recorded: the base stands.
    let none = guard::compose(&l, Some(&has("Z")), &ops, "2009-06-01", None);
    assert!(none.applied.is_empty());
    assert_eq!(none.guard.as_ref().map(guard::text), Some("has_event(Z())".to_owned()));
    let unknown = guard::compose(&l, None, &ops, "2012-06-01", Some("2024-01-01"));
    assert_eq!(unknown.guard, None);
    // No base guard reads as `true`.
    let fresh = guard::compose(&l, None, &[&narrow], "2010-06-01", Some("2025-01-01"));
    assert_eq!(
        fresh.guard.as_ref().map(guard::text),
        Some("true && has_event(B())".to_owned())
    );
}

#[test]
fn only_unordered_operations_that_do_not_commute_and_meet_are_ambiguous() {
    let l = lattice();
    let a = op(GuardOpKind::Replace, has("A"), "Other", "2010-01-01", Some("2011-01-01"));
    let b = op(GuardOpKind::Narrow, has("B"), "High", "2010-06-01", Some("2012-01-01"));
    let c = op(GuardOpKind::Narrow, has("C"), "High", "2011-01-01", None);
    let d = op(GuardOpKind::Narrow, has("D"), "High", "2011-01-01", None);
    let e = op(GuardOpKind::Widen, has("E"), "Other", "2011-01-01", None);
    // `a`/`b` overlap unordered; `a`/`c` and `a`/`d` never meet; `c`/`d`
    // commute; `e` meets `b`, `c` and `d` unordered and does not commute.
    assert_eq!(
        guard::ambiguous(&l, &[&a, &b, &c, &d, &e]),
        [(0, 1), (1, 4), (2, 4), (3, 4)]
    );
}

#[test]
fn respelling_reads_a_guard_in_the_base_view() {
    let deadline = ex(
        Type::Date,
        ExprKind::Call {
            callee: "deadline".to_owned(),
            args: vec![ex(
                Type::Named("links::Period".to_owned()),
                ExprKind::Ref {
                    target: "grace".to_owned(),
                },
            )],
        },
    );
    let date = ex(
        Type::Date,
        ExprKind::Literal {
            value: Value::Date("2026-01-01".to_owned()),
        },
    );
    let variant = ex(
        Type::Named("base::Doc".to_owned()),
        ExprKind::Literal {
            value: Value::Variant("base::complaint".to_owned()),
        },
    );
    let g = ex(
        Type::Boolean,
        ExprKind::Unary {
            op: UnaryOp::Not,
            operand: Box::new(bin(
                BinaryOp::And,
                call("has_event", vec![call("base::Served", vec![variant])]),
                bin(
                    BinaryOp::Or,
                    has("other::Filed"),
                    bin(BinaryOp::Lt, deadline, date),
                ),
            )),
        },
    );
    let viewed = guard::respell(&g, "links", "base");
    assert_eq!(
        guard::text(&viewed),
        "!(has_event(Served(complaint)) && (has_event(other::Filed()) || \
         (deadline(links::grace) < 2026-01-01)))"
    );
    let ExprKind::Unary { operand, .. } = &viewed.kind else {
        panic!("{viewed:?}");
    };
    let ExprKind::Binary { rhs, .. } = &operand.kind else {
        panic!("{operand:?}");
    };
    let ExprKind::Binary { rhs: lt, .. } = &rhs.kind else {
        panic!("{rhs:?}");
    };
    let ExprKind::Binary { lhs: dl, .. } = &lt.kind else {
        panic!("{lt:?}");
    };
    let ExprKind::Call { args, .. } = &dl.kind else {
        panic!("{dl:?}");
    };
    assert_eq!(args[0].ty, Type::Named("links::Period".to_owned()));
    // A links package without a pack id leaves its own names as written,
    // and a guard written in the base package is already in its view.
    assert_eq!(guard::text(&guard::respell(&has("Filed"), "", "base")), "has_event(Filed())");
    assert_eq!(guard::respell(&has("Filed"), "base", "base"), has("Filed"));
}

#[test]
fn the_provenance_record_names_the_operation_and_its_authority() {
    let mut narrow = op(GuardOpKind::Narrow, has("B"), "High", "2010-01-01", None);
    let applied = Applied::of(&narrow);
    assert_eq!(
        serde_json::to_value(&applied).unwrap_or_default(),
        serde_json::json!({
            "op": "narrow", "condition": "has_event(B())", "by": "High",
            "cite": "Order, Dkt. 12", "from": "2010-01-01", "known_at": "2025-01-01"
        })
    );
    narrow.cite = Authored::Hole(Some("Order, Dkt. 12".to_owned()));
    assert_eq!(Applied::of(&narrow).cite, "Order, Dkt. 12");
    assert!(!guard::in_force(&narrow, "2011-01-01", None));
    narrow.cite = Authored::Hole(None);
    assert_eq!(guard::cite_text(&narrow.cite), "");
    let kinds = [
        GuardOpKind::Narrow,
        GuardOpKind::Widen,
        GuardOpKind::Replace,
    ];
    assert_eq!(kinds.map(GuardOpKind::name), ["narrow", "widen", "replace"]);
    assert_eq!(
        kinds.map(GuardOpKind::keyword),
        ["when and", "when or", "supersedes when"]
    );
}

/// The checks run over HIR without source locations: a diagnostic names
/// the instance instead. A base version that has ended before the
/// operation takes effect needs no capability, an anonymous edge has no
/// guard to compose, and a pair of edges the patch does not touch is not
/// rechecked.
#[test]
fn checks_over_hir_without_spans_locate_the_instance() {
    let base = "module base\nstart s\nauthority FederalRule > LocalRule\n\
                event Moved()\nevent Struck()\n\
                chance s: \"S\" by Court\n{\n  tags [fact]\n  \
                y \"Y\" -> t p 1/3 { when has_event(Moved()) }\n  \
                n \"N\" -> t p 1/3 { when !has_event(Moved()) && has_event(Struck()) }\n  \
                m \"M\" -> t p 1/3 { when !has_event(Moved()) && !has_event(Struck()) }\n}\n\
                terminal t: \"T\" { payoff 0.usd }\n";
    let layer = "module order\nimport base.{ Moved, FederalRule }\ninstance base@order {\n  \
                 pack base\n  patch y {\n    when and has_event(Moved()) by FederalRule effective \
                 [2010-01-01, ..) cite \"Order, Dkt. 12\"\n  }\n}\n";
    let mut packages = cov_case_support::compile(&[
        cov_case_support::package("base", &[], "", base),
        cov_case_support::package("order", &["base"], "", layer),
    ]);
    for m in &mut packages[1].modules {
        for d in &mut m.declarations {
            if let Declaration::Instance(i) = d {
                for p in i.patch_edges.values_mut() {
                    for o in &mut p.guard_ops {
                        o.by = "LocalRule".to_owned();
                        o.span = None;
                    }
                }
            }
        }
    }
    let check = |packages: &[litigate_sema::hir::Package]| {
        let refs: Vec<_> = packages.iter().collect();
        let world = World::from_packages(&refs);
        assert!(guard::check(&world, 0).is_empty());
        guard::check(&world, 1)
    };
    let found = check(&packages);
    assert_eq!(found.len(), 1, "{found:?}");
    assert_eq!(found[0].code.as_str(), "L3004");
    assert_eq!(found[0].span.file.0, "base@order");
    for m in &mut packages[0].modules {
        for d in &mut m.declarations {
            if let Declaration::Node(n) = d {
                if n.meta.id != "s" {
                    continue;
                }
                n.meta.effective.to = Some("2010-01-01".to_owned());
                let mut anonymous = n.edges[0].clone();
                anonymous.id = None;
                anonymous.guard = None;
                n.edges.push(anonymous);
            }
        }
    }
    assert!(check(&packages).is_empty());
}
