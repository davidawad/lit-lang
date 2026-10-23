//! The obligation a deadline creates and the effects an event declares
//! (SPEC §5.2, §5.3): bearer, trigger policy, linearity, the `extend` and
//! `waive` capabilities, and `removes`/`extends`/`waives` on an event.
use super::cx::{first_expr, first_ref, nested, Cx};
use crate::db::parse::tree::{ExprKind, Field, Item, ItemKind, Name, RefPath};
use crate::db::span::Report;
use crate::hir::{Capability, DayUnit, EventEffect, TriggerPolicy, Type};
use crate::resolve::sig::ValueKind;
use litigate_base::DiagnosticCode as C;
use std::collections::BTreeMap;

/// The obligation options of one deadline declaration.
pub(super) struct Obligation {
    pub bearer: Option<String>,
    pub policy: TriggerPolicy,
    pub linear: bool,
    pub extend: Option<Capability>,
    pub waive: Option<Capability>,
}

/// The event effect keywords that each declare an effect.
const EFFECTS: [&str; 4] = ["removes", "elects", "extends", "waives"];

impl Cx<'_, '_> {
    /// `bearer`, `trigger`, `extend`, and `waive` of deadline `name`
    /// (options `o`, declared in `rule`, if any).
    pub(super) fn obligation(
        &mut self,
        name: &Name,
        o: &BTreeMap<&str, &Field>,
        rule: Option<&Item>,
    ) -> Obligation {
        let bearer = first_ref(o.get("bearer")).map(|r| self.actor(r));
        let policy = o
            .get("trigger")
            .and_then(|f| self.policy(name, f))
            .unwrap_or_default();
        let linear =
            rule.is_some_and(|r| matches!(&r.kind, ItemKind::Rule(k, ..) if k.text == "duty"));
        let extend = o.get("extend").and_then(|f| self.capability(f, true));
        let waive = o.get("waive").and_then(|f| self.capability(f, false));
        if let (Some(f), Some(ExprKind::Bool(false))) = (
            o.get("extend"),
            o.get("extendable")
                .and_then(|f| first_expr(f))
                .map(|e| &e.kind),
        ) {
            self.report(Report::new(
                C::InvalidField,
                f.key.range,
                format!(
                    "deadline `{}` is `extendable false` but grants an `extend` capability",
                    name.text
                ),
            ));
        }
        Obligation {
            bearer,
            policy,
            linear,
            extend,
            waive,
        }
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

    /// `{ by A, max N.days, after_expiry }`; `max` is required for an
    /// extension (it is bounded) and not allowed on a waiver.
    fn capability(&mut self, f: &Field, extend: bool) -> Option<Capability> {
        let fields = self.once(nested(f));
        let max_days = match (fields.get("max"), extend) {
            (Some(m), false) => {
                self.not_allowed(m, "on a `waive` capability: a waiver has no extent");
                None
            }
            (Some(m), true) => first_expr(m).and_then(|e| {
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
            }),
            (None, true) => {
                self.report(Report::new(
                    C::InvalidField,
                    f.key.range,
                    "an `extend` capability is bounded: give `max N.days`",
                ));
                None
            }
            (None, false) => None,
        };
        let Some(by) = first_ref(fields.get("by")).map(|r| self.actor(r)) else {
            self.report(Report::new(
                C::InvalidField,
                f.key.range,
                format!("`{}` names the actor holding it: `by <Actor>`", f.key.text),
            ));
            return None;
        };
        if extend && max_days.is_none() {
            return None;
        }
        Some(Capability {
            by,
            max_days,
            after_expiry: fields.contains_key("after_expiry"),
        })
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
                .param(name, params, target, &Type::String)
                .map(|param| EventEffect::Removes { param }),
            "elects" => self
                .param(name, params, target, &Type::String)
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
                    if *ty == Type::Date { "Date" } else { "String" }
                ),
            ));
            return None;
        }
        Some(i)
    }
}
