//! Refinements (SPEC §3.5): `{x: T | p}` and `T where p`.
//!
//! A predicate is typed into HIR in the qualifier grammar: linear
//! comparisons of numbers, probabilities, hours, money, durations and dates
//! (a date moves only by calendar days), `==`/`!=` on enums and `Bool`s,
//! and `&&`, `||`, `!` over those. Anything else is L2012, so every
//! predicate is quantifier-free linear arithmetic over finitely many
//! variables and every check on it terminates.
//!
//! Discharge has two tiers. Where every value a predicate names is known
//! (a case event's arguments, a pattern's constant arguments) it is
//! evaluated exactly ([`eval`]); where values are open (is any value
//! admitted at all, does an extension's refinement stay within its
//! deadline's bound) the SMT tier decides it ([`smt::decide`]). The
//! solver's `unknown` is a hole (L2015), never a pass.
use super::expr::Checker;
use super::rational as q;
use super::ty::display;
use crate::db::parse::tree::{Expr, ExprKind, Name, Qualifier, RefPath, TypeRef};
use crate::db::span::{Report, TextRange};
use crate::hir::{self, BinaryOp, DateRef, DayUnit, Rational, Refinement, Type, UnaryOp, Value};
use crate::resolve::{scope::Scope, sig::TypeSig};
use litigate_base::{Applicability, DiagnosticCode as C};

pub mod eval;
pub mod smt;

pub use eval::{eval, render};

/// What a predicate may name besides the refined value, for the L2012
/// note listing the names in scope.
const GRAMMAR: &str = "a refinement is linear: comparisons (`<`, `<=`, `>`, `>=`, `==`, `!=`) of \
     numbers, probabilities, hours, money, durations and dates, `+`, `-`, and `*` or `/` by a \
     constant; `==`/`!=` on enums and `Bool`s; `&&`, `||`, `!` over those";

/// The bound dates an `extends` event's refinements may name (SPEC §3.5),
/// the dates its deadline's extension bound is counted from.
#[must_use]
pub fn bound_dates() -> Vec<(String, Type)> {
    DateRef::ALL
        .iter()
        .map(|d| (d.name().to_owned(), Type::Date))
        .collect()
}

/// Whether `pred` names a bound date of an `extends` event: it can be
/// evaluated only once a matter's due dates are counted (replay).
#[must_use]
pub fn names_bound_date(pred: &hir::Expr) -> bool {
    let mut found = false;
    visit_refs(pred, &mut |name| {
        found |= DateRef::ALL.iter().any(|d| d.name() == name);
    });
    found
}

/// Calls `f` with every name `e` references.
pub fn visit_refs(e: &hir::Expr, f: &mut dyn FnMut(&str)) {
    match &e.kind {
        hir::ExprKind::Ref { target } => f(target),
        hir::ExprKind::Unary { operand, .. } => visit_refs(operand, f),
        hir::ExprKind::Binary { lhs, rhs, .. } => {
            visit_refs(lhs, f);
            visit_refs(rhs, f);
        }
        _ => {}
    }
}

/// `e` with every reference to `from` renamed `to` (an alias's binder
/// becomes the parameter it refines).
#[must_use]
pub fn rename(e: &hir::Expr, from: &str, to: &str) -> hir::Expr {
    let mut e = e.clone();
    rename_in(&mut e, from, to);
    e
}

fn rename_in(e: &mut hir::Expr, from: &str, to: &str) {
    match &mut e.kind {
        hir::ExprKind::Ref { target } if target == from => to.clone_into(target),
        hir::ExprKind::Unary { operand, .. } => rename_in(operand, from, to),
        hir::ExprKind::Binary { lhs, rhs, .. } => {
            rename_in(lhs, from, to);
            rename_in(rhs, from, to);
        }
        _ => {}
    }
}

/// `e` with each enum type and variant spelled by `f` (a predicate of
/// another package, read in this one's view).
#[must_use]
pub fn localize(e: &hir::Expr, f: &dyn Fn(&str) -> String) -> hir::Expr {
    let mut e = e.clone();
    localize_in(&mut e, f);
    e
}

fn localize_in(e: &mut hir::Expr, f: &dyn Fn(&str) -> String) {
    if let Type::Named(n) = &mut e.ty {
        *n = f(n);
    }
    match &mut e.kind {
        hir::ExprKind::Literal {
            value: Value::Variant(v),
        } => {
            // `Enum::variant`: the enum's name is what a view spells.
            if let Some((en, variant)) = v.rsplit_once("::") {
                *v = format!("{}::{variant}", f(en));
            }
        }
        hir::ExprKind::Unary { operand, .. } => localize_in(operand, f),
        hir::ExprKind::Binary { lhs, rhs, .. } => {
            localize_in(lhs, f);
            localize_in(rhs, f);
        }
        _ => {}
    }
}

/// Types the predicate of `q` in the qualifier grammar, the names `vars`
/// in scope; `None` after reporting (L2012, or the type error).
pub fn check(
    ck: &mut Checker<'_, '_>,
    q: &Qualifier,
    vars: &[(String, Type)],
) -> Option<Refinement> {
    let predicate = Typer { ck, vars }.pred(&q.predicate)?;
    Some(Refinement {
        binder: q.binder.text.clone(),
        predicate,
        via: None,
    })
}

/// The qualifier-grammar checker over one predicate.
struct Typer<'c, 's, 'a> {
    ck: &'c mut Checker<'s, 'a>,
    vars: &'c [(String, Type)],
}

/// What an expression outside the grammar is, for its L2012.
const fn shape(e: &ExprKind) -> &'static str {
    match e {
        ExprKind::Hole(_) => "a hole",
        ExprKind::Call(..) => "a call",
        ExprKind::Match(..) => "a `match`",
        ExprKind::Field(..) => "a field projection",
        ExprKind::Record(_) | ExprKind::List(_) => "a record or list",
        ExprKind::Binary(BinaryOp::Rem, ..) => "`%`",
        _ => "this expression",
    }
}

/// Whether values of `t` are ordered in the qualifier grammar.
fn ordered(t: &Type) -> bool {
    matches!(
        t,
        Type::Date
            | Type::Days(_)
            | Type::Hours
            | Type::Money(_)
            | Type::Number
            | Type::Probability
    )
}

/// The type `l` and `r` share in a comparison or a sum: equal types, or a
/// number beside a probability.
fn unify(l: &Type, r: &Type) -> Option<Type> {
    match (l, r) {
        (a, b) if a == b => Some(a.clone()),
        (Type::Number, Type::Probability) | (Type::Probability, Type::Number) => {
            Some(Type::Probability)
        }
        _ => None,
    }
}

impl Typer<'_, '_, '_> {
    fn var(&self, path: &RefPath) -> Option<&Type> {
        path.is_simple()
            .then(|| {
                self.vars
                    .iter()
                    .find(|(n, _)| *n == path.head.text)
                    .map(|(_, t)| t)
            })
            .flatten()
    }

    /// The type of `e` when it is (in parentheses) a name in scope.
    fn var_type(&self, e: &Expr) -> Option<Type> {
        match &e.kind {
            ExprKind::Paren(inner) => self.var_type(inner),
            ExprKind::Ref(path) => self.var(path).cloned(),
            _ => None,
        }
    }

    fn node(&self, kind: hir::ExprKind, ty: Type, r: TextRange) -> hir::Expr {
        self.ck.node(kind, Some(ty), None, r).hir
    }

    fn outside(&mut self, r: TextRange, what: &str) -> Option<hir::Expr> {
        self.ck.report(
            Report::new(
                C::InvalidRefinement,
                r,
                format!("{what} is outside the refinement qualifier grammar"),
            )
            .note(GRAMMAR),
        );
        None
    }

    /// A `Bool` predicate.
    fn pred(&mut self, e: &Expr) -> Option<hir::Expr> {
        let r = e.range;
        match &e.kind {
            ExprKind::Paren(inner) => self.pred(inner),
            ExprKind::Bool(b) => Some(self.ck.literal(Value::Boolean(*b), Type::Boolean, r).hir),
            ExprKind::Unary(UnaryOp::Not, inner) => {
                let operand = Box::new(self.pred(inner)?);
                Some(self.node(
                    hir::ExprKind::Unary {
                        op: UnaryOp::Not,
                        operand,
                    },
                    Type::Boolean,
                    r,
                ))
            }
            ExprKind::Binary(op @ (BinaryOp::And | BinaryOp::Or), a, b) => {
                let (a, b) = (self.pred(a), self.pred(b));
                Some(self.node(
                    hir::ExprKind::Binary {
                        op: *op,
                        lhs: Box::new(a?),
                        rhs: Box::new(b?),
                    },
                    Type::Boolean,
                    r,
                ))
            }
            ExprKind::Binary(
                op @ (BinaryOp::Eq
                | BinaryOp::Ne
                | BinaryOp::Lt
                | BinaryOp::Le
                | BinaryOp::Gt
                | BinaryOp::Ge),
                a,
                b,
            ) => self.compare(*op, a, b, r),
            ExprKind::Ref(path) if self.var(path) == Some(&Type::Boolean) => {
                let target = path.head.text.clone();
                Some(self.node(hir::ExprKind::Ref { target }, Type::Boolean, r))
            }
            ExprKind::Ref(path) => match self.var(path) {
                Some(t) => {
                    let message = format!(
                        "`{}` has type `{}`, not `Bool`: a refinement is a condition on it",
                        path.display(),
                        display(t)
                    );
                    self.ck.report(
                        Report::new(C::TypeMismatch, r, message)
                            .note("compare it: `x <= 30.days`, `x == Variant`"),
                    );
                    None
                }
                None => self.unknown_name(path),
            },
            other => self.outside(r, shape(other)),
        }
    }

    fn compare(&mut self, op: BinaryOp, a: &Expr, b: &Expr, r: TextRange) -> Option<hir::Expr> {
        // An enum variant is resolved against the other side's type.
        let expected = self.var_type(a).or_else(|| self.var_type(b));
        let (l, rr) = (
            self.term(a, expected.as_ref()),
            self.term(b, expected.as_ref()),
        );
        let (l, rr) = (l?, rr?);
        let Some(ty) = unify(&l.ty, &rr.ty) else {
            let code = if matches!((&l.ty, &rr.ty), (Type::Days(_), Type::Days(_))) {
                C::UnitMismatch
            } else {
                C::TypeMismatch
            };
            self.ck.report(Report::new(
                code,
                r,
                format!(
                    "cannot compare `{}` with `{}`",
                    display(&l.ty),
                    display(&rr.ty)
                ),
            ));
            return None;
        };
        let equality = matches!(op, BinaryOp::Eq | BinaryOp::Ne);
        let comparable = ordered(&ty) || (equality && matches!(ty, Type::Boolean | Type::Named(_)));
        if !comparable {
            return self.outside(
                r,
                &format!("`{}` on `{}`", super::ops::symbol(op), display(&ty)),
            );
        }
        Some(self.node(
            hir::ExprKind::Binary {
                op,
                lhs: Box::new(l),
                rhs: Box::new(rr),
            },
            Type::Boolean,
            r,
        ))
    }

    fn unknown_name(&mut self, path: &RefPath) -> Option<hir::Expr> {
        let names: Vec<&str> = self.vars.iter().map(|(n, _)| n.as_str()).collect();
        let mut report = Report::new(
            C::InvalidRefinement,
            path.range,
            format!("a refinement cannot name `{}`", path.display()),
        )
        .note(format!(
            "it may name only {}, and constants",
            names
                .iter()
                .map(|n| format!("`{n}`"))
                .collect::<Vec<_>>()
                .join(", ")
        ));
        if let Some(best) = crate::resolve::suggest::nearest(&path.head.text, names.iter().copied())
        {
            report = report.fix(path.range, best, Applicability::MaybeIncorrect);
        }
        self.ck.report(report);
        None
    }

    /// A linear term.
    fn term(&mut self, e: &Expr, expected: Option<&Type>) -> Option<hir::Expr> {
        let r = e.range;
        match &e.kind {
            ExprKind::Paren(inner) => self.term(inner, expected),
            ExprKind::Ref(path) => {
                if let Some(t) = self.var(path).cloned() {
                    let target = path.head.text.clone();
                    return Some(self.node(hir::ExprKind::Ref { target }, t, r));
                }
                let named = expected.filter(|t| matches!(t, Type::Named(_)));
                if named.is_none() && path.is_simple() {
                    return self.unknown_name(path);
                }
                let checked = match named {
                    Some(t) => self.ck.check(e, t),
                    None => self.ck.infer(e),
                };
                checked.ty.as_ref()?;
                if checked.value.is_none() {
                    return self.outside(r, &format!("`{}`, not a constant,", path.display()));
                }
                Some(checked.hir)
            }
            ExprKind::Number(_)
            | ExprKind::Money(_)
            | ExprKind::Duration(_)
            | ExprKind::Date(_)
            | ExprKind::Str(_)
            | ExprKind::Bool(_) => {
                let checked = self.ck.infer(e);
                checked.ty.as_ref()?;
                Some(checked.hir)
            }
            ExprKind::Unary(UnaryOp::Neg, inner) => {
                let operand = self.term(inner, None)?;
                if !ordered(&operand.ty) || operand.ty == Type::Date {
                    return self.outside(r, &format!("`-` on `{}`", display(&operand.ty)));
                }
                let ty = operand.ty.clone();
                Some(self.node(
                    hir::ExprKind::Unary {
                        op: UnaryOp::Neg,
                        operand: Box::new(operand),
                    },
                    ty,
                    r,
                ))
            }
            ExprKind::Binary(op @ (BinaryOp::Add | BinaryOp::Sub), a, b) => {
                let (l, rr) = (self.term(a, None), self.term(b, None));
                let (l, rr) = (l?, rr?);
                let ty = self.sum_type(*op, &l.ty, &rr.ty, r)?;
                Some(self.binary(*op, l, rr, ty, r))
            }
            ExprKind::Binary(op @ (BinaryOp::Mul | BinaryOp::Div), a, b) => {
                let (l, rr) = (self.term(a, None), self.term(b, None));
                let (l, rr) = (l?, rr?);
                let ty = self.product_type(*op, &l, &rr, r)?;
                Some(self.binary(*op, l, rr, ty, r))
            }
            other => self.outside(r, shape(other)),
        }
    }

    fn binary(
        &self,
        op: BinaryOp,
        lhs: hir::Expr,
        rhs: hir::Expr,
        ty: Type,
        r: TextRange,
    ) -> hir::Expr {
        self.node(
            hir::ExprKind::Binary {
                op,
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            },
            ty,
            r,
        )
    }

    /// The type of `l ± r`: a date moved by calendar days, the calendar
    /// days between two dates, or a sum of one type.
    fn sum_type(&mut self, op: BinaryOp, l: &Type, r: &Type, at: TextRange) -> Option<Type> {
        let calendar = Type::Days(DayUnit::Calendar);
        match (op, l, r) {
            (_, Type::Date, t) if *t == calendar => Some(Type::Date),
            (BinaryOp::Add, t, Type::Date) if *t == calendar => Some(Type::Date),
            (BinaryOp::Sub, Type::Date, Type::Date) => Some(calendar),
            (_, Type::Date, Type::Days(_)) | (BinaryOp::Add, Type::Days(_), Type::Date) => {
                self.ck.report(
                    Report::new(
                        C::InvalidRefinement,
                        at,
                        format!(
                            "a date moves by calendar days in a refinement, not by `{}`",
                            display(if *l == Type::Date { r } else { l })
                        ),
                    )
                    .note("other units count through a counting rule, which a refinement does not run"),
                );
                None
            }
            _ => match unify(l, r) {
                Some(t) if ordered(&t) && t != Type::Date => Some(t),
                _ => {
                    self.outside(
                        at,
                        &format!(
                            "`{}` of `{}` and `{}`",
                            super::ops::symbol(op),
                            display(l),
                            display(r)
                        ),
                    );
                    None
                }
            },
        }
    }

    /// The type of `l * r` or `l / r`: one factor (the divisor) must be a
    /// constant number, so the term stays linear.
    fn product_type(
        &mut self,
        op: BinaryOp,
        l: &hir::Expr,
        r: &hir::Expr,
        at: TextRange,
    ) -> Option<Type> {
        let constant = |e: &hir::Expr| e.ty == Type::Number && eval::term(e, &|_| None).is_some();
        let scalable = |t: &Type| ordered(t) && *t != Type::Date;
        let ty = match op {
            BinaryOp::Mul if constant(l) && scalable(&r.ty) => Some(r.ty.clone()),
            BinaryOp::Mul | BinaryOp::Div if constant(r) && scalable(&l.ty) => Some(l.ty.clone()),
            _ => None,
        };
        let Some(ty) = ty else {
            let operation = format!(
                "`{}` of `{}` and `{}`",
                super::ops::symbol(op),
                display(&l.ty),
                display(&r.ty)
            );
            if l.ty == Type::Date || r.ty == Type::Date {
                return self.outside(at, &operation).map(|e| e.ty);
            }
            let what = if op == BinaryOp::Div {
                "divisor"
            } else {
                "factor"
            };
            self.ck.report(
                Report::new(
                    C::InvalidRefinement,
                    at,
                    format!("{operation} is not linear: one {what} must be a constant number"),
                )
                .note(GRAMMAR),
            );
            return None;
        };
        if op == BinaryOp::Div
            && eval::term(r, &|_| None) == Some(eval::Val::Num(Rational::integer(0)))
        {
            self.ck.report(Report::new(
                C::ArithmeticError,
                at,
                q::ArithError::DivisionByZero.message(),
            ));
            return None;
        }
        Some(ty)
    }
}

/// A refinement with the range it is reported at in the declaring file:
/// its own `{x: T | p}` or `where p`, or the alias name that brings it.
pub type Located = (Refinement, TextRange);

/// The refinements a value of declared type `t` gets from the refined
/// alias `t` names, in this view; each names the value `binder` when
/// given, else keeps the alias's own name for it.
#[must_use]
pub fn inherited(scope: &Scope<'_>, t: &TypeRef, binder: Option<&str>) -> Vec<Located> {
    if super::ty::BUILTINS.contains(&t.name.text.as_str()) {
        return Vec::new();
    }
    let Ok((alias, found)) = scope.user_type(&t.name, &[]) else {
        return Vec::new();
    };
    if !matches!(found.item, TypeSig::Alias(_)) {
        return Vec::new();
    }
    let view = |n: &str| scope.localize_name(found.sig, n);
    found
        .sig
        .refinements
        .get(&t.name.text)
        .into_iter()
        .flatten()
        .map(|r| {
            let mut predicate = localize(&r.predicate, &view);
            let mut name = r.binder.clone();
            if let Some(b) = binder {
                predicate = rename(&predicate, &r.binder, b);
                b.clone_into(&mut name);
            }
            let refinement = Refinement {
                binder: name,
                predicate,
                via: Some(alias.clone()),
            };
            (refinement, t.name.range)
        })
        .collect()
}

/// A refinement inside a type argument (`List<{x: T | p}>`): L2012, since
/// a refinement stands only as a parameter's or an alias's type.
pub fn nested(ck: &mut Checker<'_, '_>, t: &TypeRef) {
    for arg in &t.args {
        for q in &arg.refinements {
            ck.report(
                Report::new(
                    C::InvalidRefinement,
                    q.range,
                    "a refinement stands only as a parameter's or a type declaration's type, not as a type argument",
                )
                .note("declare `type T = {x: B | p}` and use `T` as the argument; its values are not checked inside a container"),
            );
        }
        nested(ck, arg);
    }
}

/// The refinements of alias declaration `type A = t`, `t` resolving to
/// `base`: those of the alias `t` names, then its own; `None` when one of
/// its own failed to type (reported).
pub fn alias(ck: &mut Checker<'_, '_>, t: &TypeRef, base: &Type) -> Option<Vec<Located>> {
    nested(ck, t);
    let mut out = inherited(ck.scope, t, None);
    let mut ok = true;
    for q in &t.refinements {
        let vars = [(q.binder.text.clone(), base.clone())];
        match check(ck, q, &vars) {
            Some(r) => out.push((r, q.range)),
            None => ok = false,
        }
    }
    ok.then_some(out)
}

/// The refinements of an event's parameters `params`, resolved to
/// `typed`, in parameter order: each parameter's refined alias, then its
/// own `{x: T | p}` and `where p`. Every parameter is in scope in each, and
/// so are the bound dates when the event `extends` a deadline. `None` when
/// one failed to type (reported).
pub fn event(
    ck: &mut Checker<'_, '_>,
    params: &[(Name, TypeRef)],
    typed: &[(String, Type)],
    extends: bool,
) -> Option<Vec<Located>> {
    let mut scope = typed.to_vec();
    if extends {
        scope.extend(
            bound_dates()
                .into_iter()
                .filter(|(d, _)| typed.iter().all(|(p, _)| p != d)),
        );
    }
    let mut out = Vec::new();
    let mut ok = true;
    for ((p, t), (_, ty)) in params.iter().zip(typed) {
        nested(ck, t);
        out.extend(inherited(ck.scope, t, Some(&p.text)));
        for q in &t.refinements {
            let mut vars = vec![(q.binder.text.clone(), ty.clone())];
            vars.extend(scope.iter().filter(|(n, _)| *n != q.binder.text).cloned());
            match check(ck, q, &vars) {
                Some(mut r) => {
                    if r.binder != p.text {
                        r.predicate = rename(&r.predicate, &r.binder, &p.text);
                        p.text.clone_into(&mut r.binder);
                    }
                    out.push((r, q.range));
                }
                None => ok = false,
            }
        }
    }
    ok.then_some(out)
}

/// One argument of an event call: its parameter's name, its constant
/// value (when it folds), and its range.
pub type Argument<'v> = (&'v str, Option<&'v Value>, TextRange);

/// [`ground`] for each of `refinements`.
pub fn ground_all(
    ck: &mut Checker<'_, '_>,
    event: &str,
    refinements: Option<&Vec<Refinement>>,
    args: &[Argument<'_>],
    call: TextRange,
) {
    for r in refinements.into_iter().flatten() {
        ground(ck, event, r, args, call);
    }
}

/// Tier 1 on an event call `event(args)` written at `call`: L2013 when
/// refinement `r` is false of the arguments' values. A refinement naming
/// an argument that is not constant (a hole, a pattern's open argument)
/// or a bound date is not decided here.
pub fn ground(
    ck: &mut Checker<'_, '_>,
    event: &str,
    r: &Refinement,
    args: &[Argument<'_>],
    call: TextRange,
) {
    let env = |name: &str| {
        args.iter()
            .find(|(p, _, _)| *p == name)
            .and_then(|(_, v, _)| v.cloned())
    };
    if eval(&r.predicate, &env) != Some(false) {
        return;
    }
    let at = args
        .iter()
        .find(|(p, _, _)| *p == r.binder)
        .map_or(call, |(_, _, range)| *range);
    ck.report(violation(event, r, &env, at));
}

/// The L2013 report of refinement `r` of `event`, false under `env`: the
/// values of the names it reads, as a counterexample.
#[must_use]
pub fn violation(
    event: &str,
    r: &Refinement,
    env: &dyn Fn(&str) -> Option<Value>,
    at: TextRange,
) -> Report {
    Report::new(C::RefinementViolated, at, message(event, r, env)).note(origin(event, r))
}

/// The message of an L2013: the values refinement `r` of `event` reads
/// under `env`, and the refinement.
#[must_use]
pub fn message(event: &str, r: &Refinement, env: &dyn Fn(&str) -> Option<Value>) -> String {
    format!(
        "`{event}`: {} does not satisfy the refinement `{}`",
        witnesses(&r.predicate, env),
        render(&r.predicate)
    )
}

/// `name = value` for each name `pred` reads that `env` knows, in order.
#[must_use]
pub fn witnesses(pred: &hir::Expr, env: &dyn Fn(&str) -> Option<Value>) -> String {
    let mut names: Vec<String> = Vec::new();
    visit_refs(pred, &mut |n| {
        if !names.iter().any(|m| m == n) {
            names.push(n.to_owned());
        }
    });
    let mut types = std::collections::BTreeMap::new();
    types_of(pred, &mut types);
    names
        .iter()
        .filter_map(|n| {
            let v = eval::of(&env(n)?)?;
            Some(format!("{n} = {}", eval::show(types.get(n.as_str())?, &v)))
        })
        .collect::<Vec<_>>()
        .join(", ")
}

/// The type of each name `e` reads.
fn types_of<'e>(e: &'e hir::Expr, out: &mut std::collections::BTreeMap<&'e str, Type>) {
    match &e.kind {
        hir::ExprKind::Ref { target } => {
            out.insert(target, e.ty.clone());
        }
        hir::ExprKind::Unary { operand, .. } => types_of(operand, out),
        hir::ExprKind::Binary { lhs, rhs, .. } => {
            types_of(lhs, out);
            types_of(rhs, out);
        }
        _ => {}
    }
}

/// Where refinement `r` of `event` was declared, for a note.
#[must_use]
pub fn origin(event: &str, r: &Refinement) -> String {
    match &r.via {
        Some(alias) => format!(
            "the refinement of parameter `{}` of `{event}`, from its type `{alias}` (SPEC §3.5)",
            r.binder
        ),
        None => format!(
            "the refinement of parameter `{}` of `{event}` (SPEC §3.5)",
            r.binder
        ),
    }
}
