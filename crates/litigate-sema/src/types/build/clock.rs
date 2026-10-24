//! Deadlines, counting rules, holiday calendars, and transitions lowered to
//! HIR (SPEC §5.3-§5.4). Structural checks only; overlap, coverage, and
//! counting semantics are verified downstream.
use super::cx::{first_expr, first_ref, flag, Cx};
use crate::db::parse::tree::{ArmItem, Atom, Expr, ExprKind, Field, Item, Name, RefPath};
use crate::db::span::Report;
use crate::hir::{
    Adoption, Anchor, Chain, Counting, DayUnit, Deadline, Designated, Direction, DueEnd,
    EndOfMonth, RollOn, ServiceTime, ShortPeriod, Transition, TransitionArm, Type, Value,
    SERVICE_METHODS,
};
use crate::resolve::sig::{TypeSig, ValueKind};
use crate::resolve::suggest;
use litigate_base::{Applicability, DiagnosticCode as C};
use std::collections::BTreeMap;

impl Cx<'_, '_> {
    /// Reports `r` as none of the `expected` keywords.
    pub(super) fn bad_counting(&mut self, r: &RefPath, expected: &[&str]) {
        let shown = r.display();
        let mut report = Report::new(
            C::InvalidCounting,
            r.range,
            format!("unknown counting keyword `{shown}`"),
        )
        .note(format!("expected one of {}", expected.join(", ")));
        if let Some(best) = suggest::nearest(&shown, expected.iter().copied()) {
            report = report.fix(r.range, best, Applicability::MaybeIncorrect);
        }
        self.report(report);
    }

    pub(super) fn keyword<T: Copy>(&mut self, r: &RefPath, table: &[(&str, T)]) -> Option<T> {
        let hit = table
            .iter()
            .find(|(k, _)| r.is_simple() && *k == r.head.text)
            .map(|(_, v)| *v);
        if hit.is_none() {
            self.bad_counting(r, &table.iter().map(|(k, _)| *k).collect::<Vec<_>>());
        }
        hit
    }

    pub(super) fn days(&mut self, e: &Expr) -> Option<(u64, DayUnit)> {
        match self.ck.infer(e).value {
            // Day amounts here are unsigned duration literals (SPEC §3.1),
            // so the conversion cannot fail.
            Some(Value::Days {
                amount,
                unit: unit @ (DayUnit::Calendar | DayUnit::Court | DayUnit::Business),
            }) => u64::try_from(amount).ok().map(|n| (n, unit)),
            Some(Value::Hours(_) | Value::Days { .. }) => {
                self.report(Report::new(
                    C::InvalidCounting,
                    e.range,
                    "this period must be counted in days",
                ));
                None
            }
            _ => {
                self.report(Report::new(
                    C::TypeMismatch,
                    e.range,
                    "expected a day count such as `3.days`",
                ));
                None
            }
        }
    }

    /// `deadline d: N.unit from|after|before T counted_by c`.
    #[allow(clippy::too_many_arguments)]
    pub fn deadline(
        &mut self,
        item: &Item,
        rule: Option<&Item>,
        name: &Name,
        amount: &Expr,
        direction: &Name,
        trigger: &Expr,
        counted_by: &RefPath,
        opts: &[Field],
        local_counting: impl Fn(&str, &str) -> Option<bool>,
    ) -> Option<Deadline> {
        let meta = self.meta(name, None, item, rule);
        let counting = self.value_ref(counted_by, "counting rule", |k| {
            matches!(k, ValueKind::Counting(_))
        });
        let period = match self.ck.infer(amount).value {
            Some(Value::Days { amount: n, unit }) => {
                let counted = counting
                    .as_deref()
                    .map(|c| local_counting(c, "end_of_month"));
                if unit.is_monthly() && counted == Some(Some(false)) {
                    self.report(Report::new(
                        C::InvalidCounting,
                        amount.range,
                        "a month or year deadline needs a counting rule with `end_of_month`",
                    ));
                }
                u64::try_from(n).ok().map(|n| (n, unit))
            }
            Some(Value::Hours(_)) => {
                let allowed = counting
                    .as_deref()
                    .is_some_and(|c| local_counting(c, "allow_hours") == Some(true));
                let (code, message) = if allowed {
                    (
                        C::LoweringUnsupported,
                        "hour deadlines have no HIR/litgraph representation in language version 1",
                    )
                } else {
                    (
                        C::InvalidCounting,
                        "an hours deadline needs a counting rule with `allow_hours`",
                    )
                };
                self.report(Report::new(code, amount.range, message));
                None
            }
            _ => {
                self.report(Report::new(
                    C::TypeMismatch,
                    amount.range,
                    "a deadline period is a duration such as `21.days`",
                ));
                None
            }
        };
        let direction = match direction.text.as_str() {
            "before" => Direction::Backward,
            _ => Direction::Forward,
        };
        let o = self.once_except(opts, "extend");
        let mut designated = first_ref(o.get("designated"))
            .and_then(|r| self.designated(name, trigger, r, (direction, period.map(|p| p.1))));
        self.interpretation(name, &o, designated.as_mut());
        let (trigger, chain) = self.trigger(name, trigger);
        let forum = self.forum(name, o.get("forum"), counted_by, counting.is_some());
        let adoption = o.get("forum").and_then(|f| self.adoption(f));
        let obligation = self.obligation(name, opts, &o, rule);
        let (amount, unit) = period?;
        Some(Deadline {
            meta,
            amount,
            unit,
            direction,
            trigger: trigger?,
            chain,
            counting: counting?,
            forum,
            adoption,
            extendable: flag(o.get("extendable")),
            extension_authority: self.string_or_hole(o.get("extension_authority")),
            note: Self::string(o.get("note")),
            bearer: obligation.bearer,
            policy: obligation.policy,
            linear: obligation.linear,
            extend: obligation.extend,
            waive: obligation.waive,
            designated,
        })
    }

    /// Deadline `name`'s forum, its `forum F` option's or its package's;
    /// a resolved counting rule must compute that forum's periods.
    fn forum(
        &mut self,
        name: &Name,
        field: Option<&&Field>,
        counted_by: &RefPath,
        counted: bool,
    ) -> String {
        let forum = first_ref(field).map_or_else(
            || self.manifest.map(|m| m.forum.clone()).unwrap_or_default(),
            RefPath::display,
        );
        if counted {
            self.borrowing(name, counted_by, &forum);
        }
        forum
    }

    /// The adopting rule of `forum F cite "X" [quote "..."]`. A cite hole
    /// is recorded as a hole; a quote needs a cite whose text it quotes,
    /// so one beside a hole is L2007.
    fn adoption(&mut self, field: &Field) -> Option<Adoption> {
        let rest = Field {
            key: field.key.clone(),
            values: field.values.iter().skip(1).cloned().collect(),
            range: field.range,
        };
        let cite = first_expr(&rest)?;
        let hole = matches!(cite.kind, ExprKind::Hole(_));
        let historical = matches!(cite.kind, ExprKind::Cite(_, true));
        let quote = match rest.values.get(1) {
            Some((
                Atom::Expr(Expr {
                    kind: ExprKind::Str(q),
                    range,
                }),
                _,
            )) => Some((q.clone(), *range)),
            _ => None,
        };
        let cite = self.cite_text(Some(&&rest));
        if let (true, Some((_, range))) = (hole, &quote) {
            self.report(Report::new(
                C::InvalidField,
                *range,
                "a `quote` quotes the text of its cite, and a cite hole has none",
            ));
            return None;
        }
        Some(Adoption {
            cite: cite?,
            historical,
            quote: quote.map(|(q, _)| q),
        })
    }

    /// `interpretation "..."` of deadline `name`, qualifying its
    /// `designated` least time (SPEC §5.3): L2007 without one.
    fn interpretation(
        &mut self,
        name: &Name,
        o: &BTreeMap<&str, &Field>,
        designated: Option<&mut Designated>,
    ) {
        let Some(f) = o.get("interpretation") else {
            return;
        };
        match designated {
            Some(d) => d.interpretation = Self::string(Some(f)),
            // A designation already reported leaves nothing to qualify.
            None if o.contains_key("designated") => {}
            None => self.report(Report::new(
                C::InvalidField,
                f.range,
                format!(
                    "`interpretation` qualifies the least time of a designated due date, and `{}` designates none",
                    name.text
                ),
            )),
        }
    }

    /// `designated p` of deadline `name`: `p` is a `Date` parameter of the
    /// event `trigger` matches, and the deadline counts forward in calendar
    /// days (its least time, SPEC §5.3).
    fn designated(
        &mut self,
        name: &Name,
        trigger: &Expr,
        r: &RefPath,
        (direction, unit): (Direction, Option<DayUnit>),
    ) -> Option<Designated> {
        if unit.is_some_and(|u| u != DayUnit::Calendar) {
            self.report(Report::new(
                C::InvalidField,
                r.range,
                format!(
                    "the least time a designated due date of `{}` allows is counted in calendar days",
                    name.text
                ),
            ));
            return None;
        }
        if direction == Direction::Backward {
            self.report(Report::new(
                C::InvalidField,
                r.range,
                format!(
                    "deadline `{}` counts backward: only a forward period's due date is designated",
                    name.text
                ),
            ));
            return None;
        }
        let ExprKind::Call(callee, _) = &trigger.kind else {
            return None;
        };
        // The trigger pattern itself reports a callee that is no event.
        let (event, found) = self.ck.scope.user_type(&callee.head, &[]).ok()?;
        let TypeSig::Event(params) = found.item else {
            return None;
        };
        let Some(param) = params
            .iter()
            .position(|(p, _)| r.is_simple() && *p == r.head.text)
        else {
            self.report(Report::new(
                C::UnresolvedName,
                r.range,
                format!(
                    "the trigger `{event}` of deadline `{}` has no parameter `{}`",
                    name.text,
                    r.display()
                ),
            ));
            return None;
        };
        if params[param].1 != Some(Type::Date) {
            self.report(Report::new(
                C::TypeMismatch,
                r.range,
                format!(
                    "parameter `{}` of `{event}` designates a due date, so it must have type `Date`",
                    params[param].0
                ),
            ));
            return None;
        }
        Some(Designated {
            event: event.clone(),
            param,
            name: params[param].0.clone(),
            interpretation: None,
        })
    }

    /// A deadline's trigger: an event pattern, or the end of another
    /// deadline's period, `deadline(d).nominal` or `deadline(d).adjusted`.
    /// A chain must say which end (design review 2026-10-23, §2.1): a bare
    /// `deadline(d)` is L2001 with both readings as fixes, and a period
    /// cannot run from its own end.
    fn trigger(&mut self, name: &Name, x: &Expr) -> (Option<crate::hir::Expr>, Option<Chain>) {
        if let Some(args) = deadline_call(x) {
            let text = |end: &str| format!("deadline({}).{end}", args_text(args));
            self.report(
                Report::new(
                    C::TypeMismatch,
                    x.range,
                    "a chained period says which end it runs from: `.nominal` (the last day as \
                     counted) or `.adjusted` (after every roll and extension)",
                )
                .fix(x.range, text("nominal"), Applicability::MaybeIncorrect)
                .fix(x.range, text("adjusted"), Applicability::MaybeIncorrect),
            );
            return (None, None);
        }
        let ExprKind::Field(base, end) = &x.kind else {
            return (self.pattern(x), None);
        };
        let Some(args) = deadline_call(base) else {
            return (self.pattern(x), None);
        };
        let end = self.due_end(end);
        let target = self.chain_target(name, base, args);
        let (Some(end), Some(deadline)) = (end, target) else {
            return (None, None);
        };
        (
            Some(chain_expr(&deadline, end)),
            Some(Chain { deadline, end }),
        )
    }

    /// The end `.nominal` or `.adjusted` names; L2001 for any other field.
    fn due_end(&mut self, end: &Name) -> Option<DueEnd> {
        match end.text.as_str() {
            "nominal" => Some(DueEnd::Nominal),
            "adjusted" => Some(DueEnd::Adjusted),
            other => {
                let mut report = Report::new(
                    C::TypeMismatch,
                    end.range,
                    format!("a period ends `nominal` or `adjusted`, not `{other}`"),
                );
                if let Some(best) = suggest::nearest(other, ["nominal", "adjusted"]) {
                    report = report.fix(end.range, best, Applicability::MaybeIncorrect);
                }
                self.report(report);
                None
            }
        }
    }

    /// The deadline `deadline(...)` (`base`, with `args`) names for
    /// deadline `name`'s chain: exactly one, and not `name` itself.
    fn chain_target(&mut self, name: &Name, base: &Expr, args: &[Expr]) -> Option<String> {
        let [Expr {
            kind: ExprKind::Ref(r),
            ..
        }] = args
        else {
            self.report(Report::new(
                C::TypeMismatch,
                base.range,
                "`deadline(d)` names exactly one deadline",
            ));
            return None;
        };
        if r.display() == name.text {
            self.report(Report::new(
                C::TypeMismatch,
                r.range,
                format!("`{}` cannot run from the end of its own period", name.text),
            ));
            return None;
        }
        self.value_ref(r, "deadline", |k| matches!(k, ValueKind::Deadline))
    }

    /// L3009 when deadline `name`, a period of `forum`'s rules, is counted
    /// by `counted_by`, a rule computing other forums' periods (design
    /// review 2026-10-23, L2-L4: no cross-forum borrowing).
    fn borrowing(&mut self, name: &Name, counted_by: &RefPath, forum: &str) {
        // `counted_by` resolved to a counting rule before this is asked.
        let borrowed =
            self.ck
                .scope
                .value(counted_by)
                .ok()
                .and_then(|(_, found)| match &found.item.kind {
                    ValueKind::Counting(forums) if !forums.contains(forum) => Some(forums.clone()),
                    _ => None,
                });
        let Some(forums) = borrowed else {
            return;
        };
        let list = forums.iter().cloned().collect::<Vec<_>>().join(", ");
        self.report(
            Report::new(
                C::InvalidCounting,
                counted_by.range,
                format!(
                    "deadline `{}` is a period of forum `{forum}`, but counting rule `{}` \
                     computes periods of {list}",
                    name.text,
                    counted_by.display()
                ),
            )
            .note(
                "a period is counted by the rule of the forum whose rules set it; count it by \
                 that forum's rule, or name that forum with `{ forum F }`",
            ),
        );
    }

    /// `counting c { ... }`.
    pub fn counting(
        &mut self,
        item: &Item,
        rule: Option<&Item>,
        name: &Name,
        items: &[Field],
    ) -> Option<Counting> {
        let meta = self.meta(name, None, item, rule);
        let f = self.once(items);
        let roll_on = Self::refs(f.get("roll_forward_on"))
            .into_iter()
            .filter_map(|r| {
                self.keyword(
                    r,
                    &[
                        ("weekend", RollOn::Weekend),
                        ("legal_holiday", RollOn::LegalHoliday),
                        ("inaccessible", RollOn::Inaccessible),
                    ],
                )
            })
            .collect::<Vec<_>>();
        let service_extensions = self.service_extensions(f.get("service_extension"));
        let forward_calendar = self.closing_calendar(f.get("forward_holidays"), false);
        let declared_calendar = self.closing_calendar(f.get("declared_holidays"), true);
        let forums = match f.get("forums") {
            Some(field) => Self::refs(Some(field))
                .into_iter()
                .map(RefPath::display)
                .collect(),
            None => vec![self.manifest.map(|m| m.forum.clone()).unwrap_or_default()],
        };
        let calendar = if let Some(r) = first_ref(f.get("holidays")) {
            self.value_ref(r, "calendar", |k| matches!(k, ValueKind::Calendar(_)))
        } else {
            self.report(Report::new(
                C::InvalidCounting,
                name.range,
                format!("counting rule `{}` needs `holidays <calendar>`", name.text),
            ));
            None
        };
        let anchor = if f.contains_key("anchor") {
            first_ref(f.get("anchor")).and_then(|r| {
                self.keyword(
                    r,
                    &[
                        ("trigger_day", Anchor::TriggerDay),
                        ("next_business_day", Anchor::NextBusinessDay),
                    ],
                )
            })
        } else {
            Some(Anchor::TriggerDay)
        };
        let short_period = self.short_period(f.get("short_period"));
        let end_of_month = first_ref(f.get("end_of_month")).and_then(|r| {
            self.keyword(
                r,
                &[("clamp", EndOfMonth::Clamp), ("roll", EndOfMonth::Roll)],
            )
        });
        Some(Counting {
            meta,
            exclude_trigger_day: f.contains_key("exclude_trigger_day"),
            include_last_day: f.contains_key("include_last_day"),
            roll_on,
            service_extensions,
            calendar: calendar?,
            forward_calendar,
            declared_calendar,
            forums,
            anchor: anchor?,
            short_period,
            hours: f.contains_key("allow_hours"),
            end_of_month,
        })
    }

    /// `service_extension { Method: N.days | unrecognized, ... }`, total
    /// over [`SERVICE_METHODS`] (L3009 otherwise). A rule without one names
    /// no service method: every method is unrecognized, so a period served
    /// by any of them is refused, never counted as if it added nothing.
    fn service_extensions(&mut self, f: Option<&&Field>) -> BTreeMap<String, ServiceTime> {
        let mut out = BTreeMap::new();
        let Some(field) = f else {
            return SERVICE_METHODS
                .iter()
                .map(|m| ((*m).to_owned(), ServiceTime::Unrecognized))
                .collect();
        };
        let Some(ExprKind::Record(methods)) = first_expr(field).map(|e| &e.kind) else {
            return out;
        };
        for (method, e) in methods {
            if !SERVICE_METHODS.contains(&method.text.as_str()) {
                let mut report = Report::new(
                    C::InvalidCounting,
                    method.range,
                    format!("`{}` is not a service method", method.text),
                )
                .note(format!(
                    "the service methods are {}",
                    SERVICE_METHODS.join(", ")
                ));
                if let Some(best) = suggest::nearest(&method.text, SERVICE_METHODS) {
                    report = report.fix(method.range, best, Applicability::MaybeIncorrect);
                }
                self.report(report);
                continue;
            }
            let time = match &e.kind {
                ExprKind::Ref(r) if r.is_simple() && r.head.text == "unrecognized" => {
                    Some(ServiceTime::Unrecognized)
                }
                ExprKind::Ref(r) => {
                    self.bad_counting(r, &["unrecognized"]);
                    None
                }
                _ => self.days(e).map(|(n, unit)| ServiceTime::Adds(n, unit)),
            };
            if let Some(time) = time {
                if out.insert(method.text.clone(), time).is_some() {
                    self.report(Report::new(
                        C::InvalidCounting,
                        method.range,
                        format!("service method `{}` is listed twice", method.text),
                    ));
                }
            }
        }
        let missing: Vec<&str> = SERVICE_METHODS
            .iter()
            .copied()
            .filter(|m| !methods.iter().any(|(k, _)| k.text == *m))
            .collect();
        if !missing.is_empty() {
            self.report(
                Report::new(
                    C::InvalidCounting,
                    field.key.range,
                    format!(
                        "`service_extension` is not total: it does not say what {} add",
                        missing.join(", ")
                    ),
                )
                .note("give each `N.days` (`0.days`: it adds none) or `unrecognized`"),
            );
        }
        out
    }

    /// The calendar of `forward_holidays C cite X` or `declared_holidays C
    /// cite X`: `X` is the rule that makes `C`'s days ones the forum closes
    /// on, and is required. A `declared_holidays` calendar lists declared
    /// days: each of its holidays is a `once(date)` with no observance.
    fn closing_calendar(&mut self, f: Option<&&Field>, declared: bool) -> Option<String> {
        let field = f?;
        let cited = field.values.iter().any(|(a, _)| {
            matches!(
                a,
                Atom::Expr(Expr {
                    kind: ExprKind::Cite(..) | ExprKind::Hole(_),
                    ..
                })
            )
        });
        if !cited {
            self.report(
                Report::new(
                    C::InvalidCounting,
                    field.key.range,
                    format!("`{}` needs the `cite` of the rule closing on it", field.key.text),
                )
                .note(if declared {
                    "whether a declared day closes the forum is a legal question its rule answers (FRCP 6(a)(6)(B))"
                } else {
                    "name the rule closing these days for forward periods (FRCP 6(a)(6)(C))"
                }),
            );
        }
        let r = first_ref(f)?;
        let id = self.value_ref(r, "calendar", |k| matches!(k, ValueKind::Calendar(_)))?;
        let lists_days = matches!(
            self.ck.scope.value(r),
            Ok((_, found)) if found.item.kind == ValueKind::Calendar(true)
        );
        if declared && !lists_days {
            self.report(
                Report::new(
                    C::InvalidCounting,
                    r.range,
                    format!("`declared_holidays {}` names a calendar of rules, not of declared days", r.display()),
                )
                .note("a declared-closure calendar lists each declared day as `once(date)`, with no `observed`"),
            );
        }
        Some(id)
    }

    fn short_period(&mut self, f: Option<&&Field>) -> Option<ShortPeriod> {
        match f.and_then(|x| first_expr(x)).map(|e| (&e.kind, e.range)) {
            Some((ExprKind::Record(parts), range)) => {
                let below = parts
                    .iter()
                    .find(|(k, _)| k.text == "below")
                    .and_then(|(_, e)| self.days(e));
                let count = match parts.iter().find(|(k, _)| k.text == "count") {
                    Some((
                        _,
                        Expr {
                            kind: ExprKind::Ref(r),
                            ..
                        },
                    )) => self.keyword(
                        r,
                        &[
                            ("calendar_days", DayUnit::Calendar),
                            ("court_days", DayUnit::Court),
                            ("business_days", DayUnit::Business),
                        ],
                    ),
                    _ => None,
                };
                if parts.len() != 2 || below.is_none() || count.is_none() {
                    self.report(Report::new(
                        C::InvalidCounting,
                        range,
                        "`short_period` is `{ below: N.days, count: <unit> }`",
                    ));
                }
                below
                    .zip(count)
                    .map(|((threshold, _), unit)| ShortPeriod { threshold, unit })
            }
            _ => None,
        }
    }

    /// `transition t amends d@D { unless C -> v, otherwise -> w }`.
    pub fn transition(
        &mut self,
        item: &Item,
        rule: Option<&Item>,
        name: &Name,
        amends: &RefPath,
        arms: &[ArmItem],
    ) -> Option<Transition> {
        let meta = self.meta(name, None, item, rule);
        let versioned = |cx: &mut Self, r: &RefPath| {
            if r.version.is_none() {
                cx.report(Report::new(
                    C::InvalidTransition,
                    r.range,
                    format!(
                        "`{}` must name a version (`{}@YYYY-MM-DD`)",
                        r.display(),
                        r.display()
                    ),
                ));
                return None;
            }
            cx.value_ref(r, "declaration", |_| true)
        };
        let amendment = versioned(self, amends);
        let mut out = Vec::new();
        for (i, arm) in arms.iter().enumerate() {
            let last = i + 1 == arms.len();
            if arm.unless.is_none() != last {
                let message = if last {
                    "the last arm must be `otherwise -> <version>`"
                } else {
                    "`otherwise` must be the last arm"
                };
                self.report(Report::new(C::InvalidTransition, arm.range, message));
            }
            let unless = arm
                .unless
                .as_ref()
                .map(|c| self.ck.condition(c).hir);
            if let Some(selects) = versioned(self, &arm.target) {
                out.push(TransitionArm { unless, selects });
            }
        }
        if arms.is_empty() {
            self.report(Report::new(
                C::InvalidTransition,
                name.range,
                "a transition needs at least an `otherwise` arm",
            ));
        }
        Some(Transition {
            meta,
            amendment: amendment?,
            arms: out,
        })
    }
}

/// The HIR trigger of a chain on `deadline`'s `end`: `deadline(d).end`,
/// which no event pattern matches.
fn chain_expr(deadline: &str, end: DueEnd) -> crate::hir::Expr {
    let call = crate::hir::Expr {
        ty: Type::Unit,
        kind: crate::hir::ExprKind::Call {
            callee: "deadline".to_owned(),
            args: vec![crate::hir::Expr {
                ty: Type::Unit,
                kind: crate::hir::ExprKind::Ref {
                    target: deadline.to_owned(),
                },
                span: None,
            }],
        },
        span: None,
    };
    crate::hir::Expr {
        ty: Type::Date,
        kind: crate::hir::ExprKind::Field {
            base: Box::new(call),
            field: end.field().to_owned(),
        },
        span: None,
    }
}

/// The arguments of `e` when it is a `deadline(...)` call.
fn deadline_call(e: &Expr) -> Option<&Vec<Expr>> {
    match &e.kind {
        ExprKind::Call(f, args) if f.display() == "deadline" => Some(args),
        _ => None,
    }
}

/// The source text of `deadline(...)`'s arguments, for a fix.
fn args_text(args: &[Expr]) -> String {
    args.iter()
        .map(|a| match &a.kind {
            ExprKind::Ref(r) => r.display(),
            _ => "d".to_owned(),
        })
        .collect::<Vec<_>>()
        .join(", ")
}
