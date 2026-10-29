//! The obligation a deadline creates and the effects an event declares
//! (SPEC §5.2, §5.3): bearer, trigger policy, linearity, the `extend`
//! grants and the `waive` capability, and `removes`/`extends`/`waives` on
//! an event.
use super::cx::{first_expr, first_ref, flag, nested, Cx};
use crate::db::parse::tree::{Expr, ExprKind, Field, Item, ItemKind, Name, RefPath};
use crate::db::span::Report;
use crate::hir::{
    BinaryOp, Bound, Capability, CauseStandard, DateRef, DayUnit, EventEffect, Extension,
    ExtensionWindow, Motion, TriggerPolicy, Type,
};
use crate::resolve::sig::ValueKind;
use litigate_base::DiagnosticCode as C;
use std::collections::BTreeMap;

/// The obligation options of one deadline declaration.
pub(super) struct Obligation {
    pub bearer: Option<String>,
    pub bearer_other: bool,
    pub policy: TriggerPolicy,
    pub linear: bool,
    pub extend: Vec<Extension>,
    pub waive: Option<Capability>,
}

/// The event effect keywords that each declare an effect.
const EFFECTS: [&str; 4] = ["removes", "elects", "extends", "waives"];

impl Cx<'_, '_> {
    /// `bearer`, `trigger`, `extend`, and `waive` of deadline `name`
    /// (options `opts`, indexed as `o` but for the repeatable `extend`;
    /// declared in `rule`, if any).
    pub(super) fn obligation(
        &mut self,
        name: &Name,
        opts: &[Field],
        o: &BTreeMap<&str, &Field>,
        rule: Option<&Item>,
    ) -> Obligation {
        let (bearer, bearer_other) = o
            .get("bearer")
            .and_then(|f| self.bearer(f))
            .map_or((None, false), |(a, other)| (Some(a), other));
        let policy = o
            .get("trigger")
            .and_then(|f| self.policy(name, f))
            .unwrap_or_default();
        let linear =
            rule.is_some_and(|r| matches!(&r.kind, ItemKind::Rule(k, ..) if k.text == "duty"));
        let grants: Vec<&Field> = opts.iter().filter(|f| f.key.text == "extend").collect();
        let mut built: Vec<Option<Extension>> = grants.iter().map(|f| self.extension(f)).collect();
        if let Some(f) = o.get("lock") {
            built.push(self.named_lock(f, &built));
        }
        let waive = o.get("waive").and_then(|f| self.waiver(f));
        // A lock (`never`) admits the declaration's own grants, so the
        // deadline is extendable exactly when it states one.
        let granted = built.iter().flatten().any(|x| x.by().is_some());
        let locked = built.iter().flatten().any(|x| x.lock().is_some());
        let clash = |b: bool| if b { locked && !granted } else { granted };
        if let Some(b) = flag(o.get("extendable")).filter(|&b| clash(b)) {
            let at = if b {
                grants.first().map_or(name.range, |f| f.key.range)
            } else {
                grants
                    .iter()
                    .zip(&built)
                    .find(|(_, x)| x.as_ref().is_some_and(|x| x.by().is_some()))
                    .map_or(name.range, |(f, _)| f.key.range)
            };
            let what = if b {
                "`extendable true` but grants no extension (`extend { never ... }` alone)"
            } else {
                "`extendable false` but grants an `extend` capability"
            };
            self.report(Report::new(
                C::InvalidField,
                at,
                format!("deadline `{}` is {what}", name.text),
            ));
        }
        let extend = built.into_iter().flatten().collect();
        Obligation {
            bearer,
            bearer_other,
            policy,
            linear,
            extend,
            waive,
        }
    }

    /// `bearer A` or `bearer other(A)`: the actor, and whether the bearer
    /// is a party other than the trigger's.
    fn bearer(&mut self, f: &Field) -> Option<(String, bool)> {
        let e = first_expr(f)?;
        match &e.kind {
            ExprKind::Ref(r) => return Some((self.actor(r), false)),
            ExprKind::Call(callee, args) if callee.is_simple() && callee.head.text == "other" => {
                if let [a] = args.as_slice() {
                    if let ExprKind::Ref(r) = &a.kind {
                        return Some((self.actor(r), true));
                    }
                }
            }
            _ => {}
        }
        self.report(Report::new(
            C::InvalidField,
            e.range,
            "expected `bearer <actor>` or `bearer other(<actor>)`",
        ));
        None
    }

    /// `trigger first | latest | each | later_of(d)`.
    fn policy(&mut self, name: &Name, f: &Field) -> Option<TriggerPolicy> {
        const EXPECTED: &str = "expected `first`, `latest`, `each`, or `later_of(<deadline>)`";
        let e = first_expr(f)?;
        let policy = match &e.kind {
            ExprKind::Ref(r) if r.is_simple() => match r.head.text.as_str() {
                "first" => Some(TriggerPolicy::First),
                "latest" => Some(TriggerPolicy::Latest),
                "each" => Some(TriggerPolicy::Each),
                _ => None,
            },
            ExprKind::Call(callee, args)
                if callee.is_simple() && callee.head.text == "later_of" =>
            {
                match args.as_slice() {
                    [a] => match &a.kind {
                        ExprKind::Ref(r) => return self.later_of(name, r),
                        _ => None,
                    },
                    _ => None,
                }
            }
            _ => None,
        };
        if policy.is_none() {
            self.report(Report::new(
                C::InvalidField,
                e.range,
                format!("unknown trigger policy; {EXPECTED}"),
            ));
        }
        policy
    }

    /// `later_of(r)` on deadline `name`: another deadline.
    fn later_of(&mut self, name: &Name, r: &RefPath) -> Option<TriggerPolicy> {
        if r.is_simple() && r.head.text == name.text {
            self.report(Report::new(
                C::InvalidField,
                r.range,
                format!("`later_of` names deadline `{}` itself", name.text),
            ));
            return None;
        }
        self.value_ref(r, "deadline", |k| matches!(k, ValueKind::Deadline))
            .map(TriggerPolicy::LaterOf)
    }

    /// `prescribed_due`, `grant_order_entered` or `current_due` (L2007
    /// otherwise).
    fn date_ref(&mut self, e: &Expr) -> Option<DateRef> {
        let found = match &e.kind {
            ExprKind::Ref(r) if r.is_simple() => {
                DateRef::ALL.into_iter().find(|d| d.name() == r.head.text)
            }
            _ => None,
        };
        found.or_else(|| self.no_bound(e))
    }

    /// L2007 for `e`, which is not a date bound.
    fn no_bound<T>(&mut self, e: &Expr) -> Option<T> {
        self.report(Report::new(
            C::InvalidField,
            e.range,
            "a date bound is `max(...)`, `min(...)`, or `prescribed_due` / `grant_order_entered` / `current_due` `+ N.days` or `- N.days`",
        ));
        None
    }

    /// One `extend` grant: `{ never cite "X" }`, `{ by A, max N.days | to
    /// B [, discretion S] [, after_expiry] [, on M() [, within W]] }`, or
    /// `{ by A, discretion S [, after_expiry, on M()] }`. Its bound is
    /// required: a date bound, a cause standard, or `never`.
    pub(super) fn extension(&mut self, f: &Field) -> Option<Extension> {
        let fields = self.once(nested(f));
        if let Some(never) = fields.get("never") {
            return self.lock(never, &fields);
        }
        if let (Some(m), Some(_)) = (fields.get("max"), fields.get("to")) {
            self.not_allowed(m, "with `to`: give the bound once");
            return None;
        }
        if let (Some(w), None) = (fields.get("within"), fields.get("on")) {
            self.not_allowed(w, "without `on`: `within` bounds the motion");
            return None;
        }
        if let (Some(u), None) = (fields.get("unless"), fields.get("on")) {
            self.not_allowed(u, "without `on`: `unless` names the motion's denial");
            return None;
        }
        let by = self.by(f, &fields);
        let after_expiry = fields.contains_key("after_expiry");
        let interpretation = Self::string(fields.get("interpretation"));
        let standard = match fields.get("discretion") {
            Some(d) => Some(self.standard(first_ref(Some(d))?)?),
            None => None,
        };
        let motion = match fields.get("on") {
            Some(m) => Some(self.motion(
                m,
                fields.get("within").copied(),
                fields.get("unless").copied(),
            )?),
            None => None,
        };
        let bound = match (fields.get("max"), fields.get("to")) {
            (Some(m), _) => Some(Bound::Offset {
                from: DateRef::PrescribedDue,
                days: self.calendar_days(first_expr(m)?)?,
            }),
            (None, Some(t)) => Some(self.bound(first_expr(t)?)?),
            (None, None) => None,
        };
        if let Some(bound) = bound {
            return Some(Extension::Bounded {
                by: by?,
                until: bound,
                after_expiry,
                standard,
                motion,
                interpretation,
            });
        }
        let Some(standard) = standard else {
            self.report(Report::new(
                C::InvalidField,
                f.key.range,
                "an `extend` capability states its bound: `max N.days`, `to <bound>`, `discretion <Standard>`, or `never cite \"<rule>\"`",
            ));
            return None;
        };
        let window = if after_expiry {
            ExtensionWindow::AfterExpiryOnMotion
        } else {
            ExtensionWindow::BeforeExpiry
        };
        if window == ExtensionWindow::AfterExpiryOnMotion && motion.is_none() {
            self.report(Report::new(
                C::InvalidField,
                f.key.range,
                "a discretionary `after_expiry` grant is on motion (FRCP 6(b)(1)(B)): name the motion, `on <Motion>()`",
            ));
            return None;
        }
        Some(Extension::Discretionary {
            by: by?,
            standard,
            window,
            motion,
            interpretation,
        })
    }

    /// `extend { never cite "X" }`: a lock, which takes no other field.
    fn lock(&mut self, never: &Field, fields: &BTreeMap<&str, &Field>) -> Option<Extension> {
        let others: Vec<&Field> = fields
            .values()
            .copied()
            .filter(|o| o.key.text != "never")
            .collect();
        for other in &others {
            self.not_allowed(
                other,
                "with `never`: a lock has no extender or bound of its own",
            );
        }
        let authority = Self::string(Some(&never));
        others
            .is_empty()
            .then_some(authority)
            .flatten()
            .map(|authority| Extension::NonExtendable {
                authority,
                against: Vec::new(),
            })
    }

    /// `lock against [L, ...] cite "X"`: a lock binding the authorities
    /// `L` only (design review §2.4), each an authority level (L1002),
    /// naming at least one and citing the rule that forbids the extension
    /// (L2007). A deadline has one lock: beside `extend { never ... }`,
    /// which binds every authority, it is L2007.
    fn named_lock(&mut self, f: &Field, built: &[Option<Extension>]) -> Option<Extension> {
        // Without `against [...]` the parser has reported it (L0001).
        if f.values.is_empty() {
            return None;
        }
        let names = Self::refs(Some(&f));
        let mut against: Vec<String> = Vec::new();
        let mut known = true;
        for r in &names {
            match self.ck.scope.authority(&r.head) {
                Ok(()) if !against.contains(&r.head.text) => against.push(r.head.text.clone()),
                Ok(()) => {}
                Err(e) => {
                    self.ck.report(e);
                    known = false;
                }
            }
        }
        let cite = Field {
            key: f.key.clone(),
            values: f.values.iter().skip(1).cloned().collect(),
            range: f.range,
        };
        let message = if names.is_empty() {
            "a `lock` names the authorities it binds: `lock against [LocalRule, ...] cite \"<rule>\"`; `extend { never cite \"<rule>\" }` binds every authority"
        } else if built.iter().flatten().any(|x| x.lock().is_some()) {
            "a deadline has one lock: `extend { never ... }` binds every authority, so `lock against [...]` adds nothing beside it"
        } else if let Some(authority) = Self::string(Some(&&cite)) {
            return known.then_some(Extension::NonExtendable { authority, against });
        } else {
            "a `lock` cites the rule that forbids the extension: `lock against [...] cite \"<rule>\"`"
        };
        self.report(Report::new(C::InvalidField, f.key.range, message));
        None
    }

    /// The calendar days of duration `e` (L3009 for other days).
    fn calendar_days(&mut self, e: &Expr) -> Option<u64> {
        let (n, unit) = self.days(e)?;
        if unit != DayUnit::Calendar {
            self.report(Report::new(
                C::InvalidCounting,
                e.range,
                "an extension bound counts calendar days, such as `30.days`",
            ));
            return None;
        }
        Some(n)
    }

    /// The cause standard `r`: one of the closed set (L2007 otherwise).
    pub(super) fn standard(&mut self, r: &RefPath) -> Option<CauseStandard> {
        let text = r.display();
        let found = CauseStandard::ALL.into_iter().find(|s| s.name() == text);
        if found.is_none() {
            self.report(Report::new(
                C::InvalidField,
                r.range,
                format!(
                    "`{text}` is not a cause standard: expected {}",
                    CauseStandard::ALL
                        .iter()
                        .map(|s| format!("`{}`", s.name()))
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
            ));
        }
        found
    }

    /// The motion `on M()` of a grant, bounded by `within W` and barred by
    /// `unless D()` when given.
    fn motion(
        &mut self,
        on: &Field,
        within: Option<&Field>,
        unless: Option<&Field>,
    ) -> Option<Motion> {
        let event = self.pattern(first_expr(on)?)?;
        let within = match within {
            Some(w) => Some(self.bound(first_expr(w)?)?),
            None => None,
        };
        let unless = match unless {
            Some(u) => Some(self.pattern(first_expr(u)?)?),
            None => None,
        };
        Some(Motion {
            event,
            within,
            unless,
        })
    }

    /// A date bound: `max(b, ...)`, `min(b, ...)`, or `r + N.days` or `r -
    /// N.days` with `r` one of `prescribed_due`, `grant_order_entered`,
    /// `current_due` (L2007 otherwise).
    fn bound(&mut self, e: &Expr) -> Option<Bound> {
        match &e.kind {
            ExprKind::Paren(inner) => self.bound(inner),
            ExprKind::Call(f, args) if f.is_simple() && !args.is_empty() => {
                let ops = args
                    .iter()
                    .map(|a| self.bound(a))
                    .collect::<Vec<_>>()
                    .into_iter()
                    .collect::<Option<Vec<_>>>();
                match f.head.text.as_str() {
                    "max" => Some(Bound::Max(ops?)),
                    "min" => Some(Bound::Min(ops?)),
                    _ => self.no_bound(e),
                }
            }
            ExprKind::Ref(_) => Some(Bound::Offset {
                from: self.date_ref(e)?,
                days: 0,
            }),
            ExprKind::Binary(op @ (BinaryOp::Add | BinaryOp::Sub), l, n) => {
                let from = self.date_ref(l)?;
                let days = self.calendar_days(n)?;
                Some(if *op == BinaryOp::Add {
                    Bound::Offset { from, days }
                } else {
                    Bound::Before { from, days }
                })
            }
            _ => self.no_bound(e),
        }
    }

    /// The `waive` capability `{ by A [, after_expiry] }`: a waiver has no
    /// extent, so no bound.
    pub(super) fn waiver(&mut self, f: &Field) -> Option<Capability> {
        let fields = self.once(nested(f));
        for bound in ["max", "to", "discretion", "on", "within", "never"] {
            if let Some(m) = fields.get(bound) {
                self.not_allowed(m, "on a `waive` capability: a waiver has no extent");
            }
        }
        if let Some(i) = fields.get("interpretation") {
            self.not_allowed(
                i,
                "on a `waive` capability: only an `extend` grant rests on one",
            );
        }
        Some(Capability {
            by: self.by(f, &fields)?,
            after_expiry: fields.contains_key("after_expiry"),
        })
    }

    /// The `by A` of capability `f`, reported when missing.
    fn by(&mut self, f: &Field, fields: &BTreeMap<&str, &Field>) -> Option<String> {
        let by = first_ref(fields.get("by")).map(|r| self.actor(r));
        if by.is_none() {
            self.report(Report::new(
                C::InvalidField,
                f.key.range,
                format!("`{}` names the actor holding it: `by <Actor>`", f.key.text),
            ));
        }
        by
    }

    /// The effect clause of event `name` with resolved `params`.
    pub(super) fn effect(
        &mut self,
        name: &Name,
        params: &[(String, Type)],
        fields: &[Field],
    ) -> Option<EventEffect> {
        let o = self.once(fields);
        let mut declared = fields
            .iter()
            .filter(|f| EFFECTS.contains(&f.key.text.as_str()));
        let first = declared.next();
        for extra in declared {
            self.not_allowed(extra, "here: an event declares at most one effect");
        }
        let to = o.get("to");
        if let (Some(t), false) = (to, first.is_some_and(|f| f.key.text == "extends")) {
            self.not_allowed(t, "without `extends`");
        }
        let first = first?;
        let target = first_ref(Some(&first))?;
        match first.key.text.as_str() {
            "removes" => self
                .param(name, params, target, &Type::Edge)
                .map(|param| EventEffect::Removes { param }),
            "elects" => self
                .param(name, params, target, &Type::Edge)
                .map(|param| EventEffect::Elects { param }),
            "extends" => {
                let deadline = self.effect_deadline(target);
                let Some(to) = first_ref(to) else {
                    self.report(Report::new(
                        C::InvalidField,
                        first.key.range,
                        "`extends` needs `to <param>`: the `Date` parameter holding the new due date",
                    ));
                    return None;
                };
                let param = self.param(name, params, to, &Type::Date);
                Some(EventEffect::Extends {
                    deadline: deadline?,
                    param: param?,
                })
            }
            _ => self
                .effect_deadline(target)
                .map(|deadline| EventEffect::Waives { deadline }),
        }
    }

    fn effect_deadline(&mut self, r: &RefPath) -> Option<String> {
        self.value_ref(r, "deadline", |k| matches!(k, ValueKind::Deadline))
    }

    /// The index of parameter `r` of event `name`, which must have type `ty`.
    fn param(
        &mut self,
        name: &Name,
        params: &[(String, Type)],
        r: &RefPath,
        ty: &Type,
    ) -> Option<usize> {
        let Some(i) = params
            .iter()
            .position(|(p, _)| r.is_simple() && *p == r.head.text)
        else {
            self.report(Report::new(
                C::UnresolvedName,
                r.range,
                format!("event `{}` has no parameter `{}`", name.text, r.display()),
            ));
            return None;
        };
        if params[i].1 != *ty {
            self.report(Report::new(
                C::TypeMismatch,
                r.range,
                format!(
                    "parameter `{}` of `{}` must have type `{}`",
                    params[i].0,
                    name.text,
                    crate::types::ty::display(ty)
                ),
            ));
            return None;
        }
        Some(i)
    }
}
