//! Expression type checking (SPEC §6): literals, names, calls, and the
//! bidirectional `check` that resolves enum variants, `Dist` records,
//! probability literals, and holes against an expected type.
use super::ops;
use super::rational::{self, ArithError};
use super::ty::display;
use crate::db::parse::tree::{Expr, ExprKind, RefPath};
use crate::db::span::{to_span, Report, TextRange};
use crate::hir::{self, Type, Value};
use crate::resolve::{scope::Scope, sig::TypeSig, sig::ValueKind};
use litigate_base::{Applicability, DiagnosticCode as C, FileId, LineIndex};
use std::collections::BTreeSet;

/// A checked expression: its HIR, type (`None` after an error), and
/// constant value when it folds.
#[derive(Debug, Clone)]
pub struct Checked {
    /// Typed HIR.
    pub hir: hir::Expr,
    /// Type; `None` suppresses cascading errors.
    pub ty: Option<Type>,
    /// Exact constant value, when known.
    pub value: Option<Value>,
    /// Exact linear form of a `Probability` term that does not fold
    /// (`refine`), when its operator built one.
    pub form: Option<super::refine::Linear>,
}

/// Expression checker for one module.
pub struct Checker<'s, 'a> {
    /// Name lookup.
    pub scope: &'s Scope<'a>,
    /// Reports produced so far.
    pub reports: Vec<Report>,
    /// Holes seen: range and reason.
    pub holes: Vec<(TextRange, Option<String>)>,
    /// Declared actors (values of type `Role`).
    pub actors: BTreeSet<String>,
    /// File for HIR spans.
    pub file: Option<(&'s FileId, &'s LineIndex)>,
    /// Names bound by the context being checked (`payoff` and `stake` in
    /// a `payoff_transform`), shadowing declarations.
    pub locals: Vec<(String, Type)>,
}

impl<'s, 'a> Checker<'s, 'a> {
    /// A checker with no actors and no span information.
    #[must_use]
    pub fn new(scope: &'s Scope<'a>) -> Self {
        Self {
            scope,
            reports: Vec::new(),
            holes: Vec::new(),
            actors: BTreeSet::new(),
            file: None,
            locals: Vec::new(),
        }
    }

    /// Records a report; a lookup failure already reported at its import
    /// (`Unresolved::AlreadyReported`) records nothing.
    pub fn report(&mut self, report: impl Into<Option<Report>>) {
        self.reports.extend(report.into());
    }

    pub(crate) fn span(&self, range: TextRange) -> Option<litigate_base::Span> {
        self.file.map(|(f, i)| to_span(f, i, range))
    }

    pub(crate) fn node(
        &self,
        kind: hir::ExprKind,
        ty: Option<Type>,
        value: Option<Value>,
        range: TextRange,
    ) -> Checked {
        Checked {
            hir: hir::Expr {
                ty: ty.clone().unwrap_or(Type::Unit),
                kind,
                span: self.span(range),
            },
            ty,
            value,
            form: None,
        }
    }

    pub(crate) fn literal(&self, value: Value, ty: Type, range: TextRange) -> Checked {
        self.node(
            hir::ExprKind::Literal {
                value: value.clone(),
            },
            Some(ty),
            Some(value),
            range,
        )
    }

    pub(crate) fn error(&self, range: TextRange) -> Checked {
        self.node(
            hir::ExprKind::Literal { value: Value::Unit },
            None,
            None,
            range,
        )
    }

    pub(crate) fn arith(&mut self, range: TextRange, e: ArithError) -> Checked {
        self.report(Report::new(C::ArithmeticError, range, e.message()));
        self.error(range)
    }

    /// Infers the type of `e` with no expectation.
    pub fn infer(&mut self, e: &Expr) -> Checked {
        let r = e.range;
        match &e.kind {
            ExprKind::Number(text) => match rational::parse_decimal(text) {
                Ok(v) => self.literal(Value::Number(v), Type::Number, r),
                Err(err) => self.arith(r, err),
            },
            ExprKind::Money(text) => self.money(text, r),
            ExprKind::Duration(text) => self.duration(text, r),
            ExprKind::Str(s) => self.literal(Value::String(s.clone()), Type::String, r),
            ExprKind::Date(d) => self.literal(Value::Date(d.clone()), Type::Date, r),
            ExprKind::Bool(b) => self.literal(Value::Boolean(*b), Type::Boolean, r),
            ExprKind::Unit => self.literal(Value::Unit, Type::Unit, r),
            ExprKind::Cite(c, _) => self.literal(Value::Cite(c.clone()), Type::Cite, r),
            ExprKind::Hole(reason) => {
                self.holes.push((r, reason.clone()));
                self.report(
                    Report::new(C::TypeMismatch, r, "cannot infer the type of this hole")
                        .note("holes are allowed only where the expected type is known"),
                );
                self.error(r)
            }
            ExprKind::Ref(path) => self.reference(path, None),
            ExprKind::Call(callee, args) => self.call(callee, args, r),
            ExprKind::Unary(op, inner) => ops::unary(self, *op, inner, r),
            ExprKind::Binary(op, lhs, rhs) => ops::binary(self, *op, lhs, rhs, r),
            ExprKind::Paren(inner) => self.infer(inner),
            ExprKind::Record(fields) => super::shape::record(self, fields, r),
            ExprKind::List(items) => super::shape::list(self, items, None, r),
            ExprKind::Field(base, name) => super::shape::field(self, base, name, r),
            ExprKind::Match(scrutinee, arms) => {
                super::shape::matching(self, scrutinee, arms, None, r)
            }
        }
    }

    /// Checks `e` against `expected`, reporting L2001 on a mismatch.
    pub fn check(&mut self, e: &Expr, expected: &Type) -> Checked {
        let r = e.range;
        let got = match (&e.kind, expected) {
            (ExprKind::Hole(reason), _) => {
                self.holes.push((r, reason.clone()));
                return self.literal(Value::Hole(reason.clone()), expected.clone(), r);
            }
            (ExprKind::Paren(inner), _) => return self.check(inner, expected),
            (ExprKind::Ref(path), Type::Named(_)) => self.reference(path, Some(expected)),
            (ExprKind::Record(fields), Type::Distribution(en)) => {
                return super::shape::distribution(self, fields, en, r)
            }
            (ExprKind::List(items), Type::List(inner)) => {
                super::shape::list(self, items, Some(inner), r)
            }
            (ExprKind::Match(s, arms), _) => {
                super::shape::matching(self, s, arms, Some(expected), r)
            }
            _ => self.infer(e),
        };
        self.coerce(got, expected, r)
    }

    /// Accepts `got` where `expected` is required: equal types, or a
    /// constant `Number` in `[0,1]` as a `Probability`.
    pub fn coerce(&mut self, got: Checked, expected: &Type, r: TextRange) -> Checked {
        match (&got.ty, expected) {
            (None, _) => got,
            (Some(t), e) if t == e => got,
            (Some(Type::Number), Type::Probability) => match got.value {
                Some(Value::Number(v)) if rational::is_probability(v) => {
                    self.literal(Value::Probability(v), Type::Probability, r)
                }
                Some(Value::Number(v)) => {
                    self.report(Report::new(
                        C::ArithmeticError,
                        r,
                        format!("probability {} is outside [0, 1]", rational::display(v)),
                    ));
                    self.error(r)
                }
                _ => {
                    self.report(Report::new(
                        C::TypeMismatch,
                        r,
                        "expected a constant probability in [0, 1]",
                    ));
                    self.error(r)
                }
            },
            (Some(t), e) => {
                let mut report = Report::new(
                    C::TypeMismatch,
                    r,
                    format!("expected `{}`, found `{}`", display(e), display(t)),
                );
                if let (Type::Days(a), Type::Days(b)) = (t, e) {
                    report = Report::new(
                        C::UnitMismatch,
                        r,
                        format!("expected `{}`, found `{}`", display(e), display(t)),
                    );
                    report = ops::counting_hint(self, report, *a, *b, r);
                }
                self.report(report);
                self.error(r)
            }
        }
    }

    /// Names that denote literals: enum variants (by expected type or
    /// `Enum.Variant`) and actors. (`true`/`false` in expression position
    /// always parse as literals.)
    fn literal_name(&mut self, path: &RefPath, expected: Option<&Type>) -> Option<Checked> {
        let r = path.range;
        if let Some(Type::Named(en)) = expected {
            if let Some(variants) = self.scope.enum_variants(en) {
                if path.is_simple() && variants.contains(&path.head.text) {
                    return Some(self.literal(
                        Value::Variant(format!("{en}::{}", path.head.text)),
                        Type::Named(en.clone()),
                        r,
                    ));
                }
            }
        }
        if path.is_simple() && self.actors.contains(&path.head.text) {
            return Some(self.node(
                hir::ExprKind::Ref {
                    target: path.head.text.clone(),
                },
                Some(Type::Role),
                None,
                r,
            ));
        }
        if path.rest.len() == 1 && path.instance.is_none() {
            if let Ok((en, found)) = self.scope.user_type(&path.head, &[]) {
                // `Alias.Variant` when the alias names an enum.
                let (en, variants) = match found.item {
                    TypeSig::Enum(variants) => (en, Some(variants.as_slice())),
                    TypeSig::Alias(Some(Type::Named(target))) => {
                        let target = self.scope.localize_name(found.sig, target);
                        let variants = self.scope.enum_variants(&target);
                        (target, variants)
                    }
                    _ => (en, None),
                };
                if let Some(variants) = variants {
                    let variant = &path.rest[0].1.text;
                    if variants.contains(variant) {
                        return Some(self.literal(
                            Value::Variant(format!("{en}::{variant}")),
                            Type::Named(en),
                            r,
                        ));
                    }
                    self.report(
                        Report::new(
                            C::UnresolvedName,
                            path.rest[0].1.range,
                            format!("`{en}` has no variant `{variant}`"),
                        )
                        .note(format!("variants: {}", variants.join(", "))),
                    );
                    return Some(self.error(r));
                }
            }
        }
        None
    }

    /// Resolves a name in expression position.
    pub fn reference(&mut self, path: &RefPath, expected: Option<&Type>) -> Checked {
        let r = path.range;
        let local = self.locals.iter().find(|(name, _)| *name == path.display());
        if let Some((name, ty)) = local {
            let (target, ty) = (name.clone(), ty.clone());
            return self.node(hir::ExprKind::Ref { target }, Some(ty), None, r);
        }
        if let Some(checked) = self.literal_name(path, expected) {
            return checked;
        }
        match self.scope.value(path) {
            Ok((target, found)) => match &found.item.kind {
                ValueKind::Default(ty) => {
                    let ty = ty.as_ref().map(|t| self.scope.localize(found.sig, t));
                    self.node(hir::ExprKind::Ref { target }, ty, None, r)
                }
                kind => {
                    let mut report = Report::new(
                        C::TypeMismatch,
                        r,
                        format!("{} `{}` is not a value", kind.noun(), path.display()),
                    );
                    if matches!(kind, ValueKind::Deadline) {
                        report = report.note("its due date is `deadline(..)`").fix(
                            r,
                            format!("deadline({})", path.display()),
                            Applicability::MachineApplicable,
                        );
                    }
                    self.report(report);
                    self.error(r)
                }
            },
            Err(crate::resolve::scope::Unresolved::AlreadyReported) => self.error(r),
            Err(crate::resolve::scope::Unresolved::Report(report)) => {
                let report = match expected {
                    Some(Type::Named(en))
                        if path.is_simple() && self.scope.enum_variants(en).is_some() =>
                    {
                        let variants = self.scope.enum_variants(en).unwrap_or_default();
                        let mut r2 = Report::new(
                            C::TypeMismatch,
                            r,
                            format!("expected a `{en}` variant, found `{}`", path.head.text),
                        )
                        .note(format!("variants: {}", variants.join(", ")));
                        if let Some(best) = crate::resolve::suggest::nearest(
                            &path.head.text,
                            variants.iter().map(String::as_str),
                        ) {
                            r2 = r2.fix(r, best, Applicability::MaybeIncorrect);
                        }
                        r2
                    }
                    _ => self.hyphen_fix(path, report),
                };
                self.report(report);
                self.error(r)
            }
        }
    }

    /// `a-b` that does not resolve while `a` and `b` do: offer `a - b`.
    fn hyphen_fix(&self, path: &RefPath, report: Report) -> Report {
        if !path.is_simple() {
            return report;
        }
        let names = self.scope.value_names();
        let known = |s: &str| names.contains(&s) || self.actors.contains(s);
        let text = &path.head.text;
        let split = text
            .match_indices('-')
            .map(|(i, _)| (&text[..i], &text[i + 1..]))
            .find(|(a, b)| known(a) && known(b));
        match split {
            Some((a, b)) => report
                .note("identifiers may contain `-`; subtraction needs spaces")
                .fix(
                    path.head.range,
                    format!("{a} - {b}"),
                    Applicability::MachineApplicable,
                ),
            None => report,
        }
    }

    fn call(&mut self, callee: &RefPath, args: &[Expr], r: TextRange) -> Checked {
        if callee.is_simple() {
            if let Some(checked) = super::builtins::call(self, &callee.head.text, args, r) {
                return checked;
            }
        }
        self.event_call(callee, args, false, r)
    }

    /// An event constructor call; `partial` permits omitted trailing
    /// arguments (patterns in deadline triggers and edge `event`).
    pub fn event_call(
        &mut self,
        callee: &RefPath,
        args: &[Expr],
        partial: bool,
        r: TextRange,
    ) -> Checked {
        let lookup = if callee.is_simple() {
            self.scope.user_type(&callee.head, &super::builtins::NAMES)
        } else {
            Err(Report::new(
                C::UnresolvedName,
                callee.range,
                format!("cannot find function `{}`", callee.display()),
            )
            .into())
        };
        let (name, params) = match lookup {
            Ok((name, found)) => {
                if let TypeSig::Event(params) = found.item {
                    (name, self.scope.localize_params(found.sig, params))
                } else {
                    self.report(Report::new(
                        C::InvalidOperation,
                        callee.range,
                        format!(
                            "`{}` is not an event constructor or builtin",
                            callee.display()
                        ),
                    ));
                    return self.error(r);
                }
            }
            Err(crate::resolve::scope::Unresolved::AlreadyReported) => return self.error(r),
            Err(crate::resolve::scope::Unresolved::Report(report)) => {
                let report = Report {
                    message: report
                        .message
                        .replace("cannot find type", "cannot find function or event"),
                    ..report
                };
                self.report(report);
                return self.error(r);
            }
        };
        let arity_ok = if partial {
            args.len() <= params.len()
        } else {
            args.len() == params.len()
        };
        if !arity_ok {
            let need = if partial {
                format!("at most {}", params.len())
            } else {
                params.len().to_string()
            };
            self.report(
                Report::new(
                    C::TypeMismatch,
                    r,
                    format!(
                        "`{name}` takes {need} argument{}, found {}",
                        if params.len() == 1 { "" } else { "s" },
                        args.len()
                    ),
                )
                .note(format!(
                    "`{name}({})`",
                    params
                        .iter()
                        .map(|(p, t)| format!("{p}: {}", t.as_ref().map_or("?".into(), display)))
                        .collect::<Vec<_>>()
                        .join(", ")
                )),
            );
        }
        let args: Vec<hir::Expr> = args
            .iter()
            .zip(params.iter().map(Some).chain(std::iter::repeat(None)))
            .map(|(a, p)| match p.and_then(|(_, t)| t.as_ref()) {
                Some(t) => self.check(a, t).hir,
                None => self.infer(a).hir,
            })
            .collect();
        let ty = arity_ok.then(|| Type::Named(name.clone()));
        self.node(hir::ExprKind::Call { callee: name, args }, ty, None, r)
    }
}
