//! Graph nodes and their edges (SPEC §5.1, field types from §6).
use super::cx::{first_expr, nested, Cx};
use crate::db::parse::tree::{EdgeItem, Expr, ExprKind, Field, Item, NodeItem};
use crate::db::span::Report;
use crate::hir::{
    Authored, DayUnit, DeadlineWindow, Edge, EdgeDeadline, Money, Node, NodeKind, Rational,
    TriangularDuration, Type, Valence, Value,
};
use crate::resolve::sig::ValueKind;
use crate::resolve::suggest;
use crate::types::rational as q;
use litigate_base::{Applicability, DiagnosticCode as C};
use std::collections::{BTreeMap, BTreeSet};

const VALENCES: [&str; 4] = ["good", "caution", "bad", "neutral"];

impl Cx<'_, '_> {
    fn valence(&mut self, f: Option<&&Field>) -> Option<Valence> {
        let r = match first_expr(f?).map(|e| &e.kind) {
            Some(ExprKind::Ref(r)) => r,
            _ => return None,
        };
        let v = match r.head.text.as_str() {
            "good" => Valence::Good,
            "caution" => Valence::Caution,
            "bad" => Valence::Bad,
            "neutral" => Valence::Neutral,
            other => {
                let mut report = Report::new(
                    C::UnresolvedName,
                    r.range,
                    format!("unknown valence `{other}`"),
                )
                .note("expected `good`, `caution`, `bad`, or `neutral`");
                if let Some(best) = suggest::nearest(other, VALENCES) {
                    report = report.fix(r.range, best, Applicability::MaybeIncorrect);
                }
                self.report(report);
                return None;
            }
        };
        r.is_simple().then_some(v)
    }

    /// Money<usd> or a hole.
    pub fn money(&mut self, e: &Expr) -> Option<Authored<Money>> {
        let ch = self.ck.check(e, &Type::Money("usd".into()));
        match ch.value {
            Some(Value::Money(m)) => Some(Authored::Known(m)),
            Some(Value::Hole(reason)) => Some(Authored::Hole(reason)),
            _ if ch.ty.is_some() => {
                self.report(Report::new(
                    C::TypeMismatch,
                    e.range,
                    "expected a constant amount",
                ));
                None
            }
            _ => None,
        }
    }

    /// A constant of type `ty` (attrs, hours, days).
    pub fn constant(&mut self, e: &Expr, ty: &Type) -> Option<Value> {
        let ch = self.ck.check(e, ty);
        if ch.ty.is_some() && ch.value.is_none() {
            self.report(Report::new(C::TypeMismatch, e.range, "expected a constant"));
        }
        ch.value
    }

    fn attrs(&mut self, f: Option<&&Field>) -> BTreeMap<String, Rational> {
        let Some(ExprKind::Record(fields)) = f.and_then(|f| first_expr(f)).map(|e| &e.kind) else {
            return BTreeMap::new();
        };
        let mut out = BTreeMap::new();
        for (name, e) in fields {
            if let Some(Value::Number(v)) = self.constant(e, &Type::Number) {
                if out.insert(name.text.clone(), v).is_some() {
                    self.report(Report::new(
                        C::InvalidField,
                        name.range,
                        format!("attribute `{}` is given twice", name.text),
                    ));
                }
            }
        }
        out
    }

    /// Builds one node.
    pub fn node(&mut self, n: &NodeItem, item: &Item, rule: Option<&Item>) -> Node {
        let meta = self.meta(&n.name, Some(&n.label), item, rule);
        let clauses = self.once(&n.clauses);
        let fields = self.once(&n.fields);
        let kw = n.keyword.text.as_str();
        let chance = kw == "chance";
        let terminal = kw == "terminal";
        for (key, ok, why) in [
            ("discretion", chance, "on non-chance nodes"),
            ("outcome", terminal, "on non-terminal nodes"),
        ] {
            if let Some(f) = clauses.get(key).filter(|_| !ok) {
                self.not_allowed(f, why);
            }
        }
        for key in ["payoff", "payoff_by_flag"] {
            if let Some(f) = fields.get(key).filter(|_| !terminal) {
                self.not_allowed(f, "on non-terminal nodes");
            }
        }
        let tags = self.idents(fields.get("tags"));
        if !chance && tags.iter().any(|t| t == "fact") {
            if let Some(f) = fields.get("tags") {
                self.report(Report::new(
                    C::InvalidField,
                    f.range,
                    "the `fact` tag is allowed only on `chance` nodes",
                ));
            }
        }
        if let Some(g) = fields.get("group").and_then(|f| first_expr(f)) {
            if let ExprKind::Ref(r) = &g.kind {
                if let Err(report) = self.ck.scope.group(&r.head) {
                    self.report(report);
                }
            }
        }
        let actor = clauses
            .get("by")
            .and_then(|f| first_expr(f))
            .and_then(|e| match &e.kind {
                ExprKind::Ref(r) => Some(r),
                _ => None,
            })
            .map(|r| self.actor(r));
        let kind = match kw {
            "decision" => NodeKind::Decision,
            "chance" => NodeKind::Chance {
                discretion: clauses.contains_key("discretion"),
            },
            "terminal" => {
                let payoff = fields
                    .get("payoff")
                    .and_then(|f| first_expr(f))
                    .and_then(|e| self.money(e));
                let mut by_flag = BTreeMap::new();
                if let Some(ExprKind::Record(rs)) = fields
                    .get("payoff_by_flag")
                    .and_then(|f| first_expr(f))
                    .map(|e| &e.kind)
                {
                    for (flag, e) in rs {
                        match self.money(e) {
                            Some(Authored::Known(m)) => {
                                by_flag.insert(flag.text.clone(), m);
                            }
                            Some(Authored::Hole(_)) => self.report(Report::new(
                                C::TypeMismatch,
                                e.range,
                                "`payoff_by_flag` values cannot be holes",
                            )),
                            None => {}
                        }
                    }
                }
                NodeKind::Terminal {
                    outcomes: self.idents(clauses.get("outcome")),
                    payoff,
                    payoff_by_flag: by_flag,
                }
            }
            _ => NodeKind::State,
        };
        let mut seen = BTreeSet::new();
        let mut edges = Vec::new();
        for e in &n.edges {
            if let Some(id) = &e.name {
                if !seen.insert(id.text.clone()) {
                    self.report(Report::new(
                        C::DuplicateName,
                        id.range,
                        format!("node `{}` already has an edge `{}`", n.name.text, id.text),
                    ));
                }
            }
            edges.push(self.edge(e, &n.name.text, actor.as_deref(), chance));
        }
        if chance {
            self.chance_sum(n, &edges);
        }
        Node {
            meta,
            kind,
            group: fields
                .get("group")
                .and_then(|f| first_expr(f))
                .and_then(|e| match &e.kind {
                    ExprKind::Ref(r) => Some(r.display()),
                    _ => None,
                }),
            note: Self::string(fields.get("note")),
            valence: self.valence(fields.get("valence")),
            court_listener_url: Self::string(fields.get("court_listener_url")),
            tags,
            attrs: self.attrs(fields.get("attrs")),
            actor,
            edges,
        }
    }

    fn chance_sum(&mut self, n: &NodeItem, edges: &[Edge]) {
        let known: Vec<Rational> = edges
            .iter()
            .filter_map(|e| match &e.probability {
                Some(Authored::Known(p)) => Some(*p),
                _ => None,
            })
            .collect();
        let holes = edges
            .iter()
            .any(|e| matches!(e.probability, Some(Authored::Hole(_)) | None));
        match q::sum(known) {
            Ok(total) if !holes && total != Rational::integer(1) && !edges.is_empty() => {
                self.report(Report::new(
                    C::InvalidDistribution,
                    n.name.range,
                    format!(
                        "probabilities of chance node `{}` sum to {}, not exactly 1",
                        n.name.text,
                        q::display(total)
                    ),
                ));
            }
            Ok(total) if holes && q::cmp(total, Rational::integer(1)).is_gt() => {
                self.report(Report::new(
                    C::InvalidDistribution,
                    n.name.range,
                    format!(
                        "known probabilities of `{}` already sum to {}",
                        n.name.text,
                        q::display(total)
                    ),
                ));
            }
            Ok(_) => {}
            Err(e) => self.report(Report::new(C::ArithmeticError, n.name.range, e.message())),
        }
    }

    /// Builds one edge from node `from` (or a link when `from` is qualified).
    pub fn edge(
        &mut self,
        e: &EdgeItem,
        from: &str,
        node_actor: Option<&str>,
        chance: bool,
    ) -> Edge {
        let clauses = self.once(&e.clauses);
        let fields = self.once(&e.fields);
        let to = self
            .value_ref(&e.to, "node", |k| matches!(k, ValueKind::Node { .. }))
            .unwrap_or_else(|| e.to.display());
        let actor = clauses
            .get("by")
            .and_then(|f| first_expr(f))
            .and_then(|x| match &x.kind {
                ExprKind::Ref(r) => Some(r),
                _ => None,
            })
            .map(|r| self.actor(r));
        let actor = actor
            .or_else(|| node_actor.map(str::to_owned))
            .unwrap_or_else(|| "either".to_owned());
        let probability = self.probability(
            e,
            clauses.get("p").and_then(|f| first_expr(f)),
            to == from,
            chance,
        );
        if let Some(f) = fields.get("replaces").filter(|_| e.from.is_none()) {
            self.not_allowed(f, "outside a links package");
        }
        if let (Some(_), Some(f)) = (fields.get("within"), fields.get("deadline")) {
            self.report(Report::new(
                C::InvalidField,
                f.key.range,
                "an edge has either `within` or `deadline`, not both",
            ));
        }
        Edge {
            id: e.name.as_ref().map(|n| n.text.clone()),
            to,
            label: e.label.text.clone(),
            actor,
            authority: Self::string(fields.get("authority")),
            probability,
            hours: fields
                .get("effort")
                .and_then(|f| first_expr(f))
                .and_then(|x| match self.constant(x, &Type::Hours) {
                    Some(Value::Hours(h)) => Some(h),
                    _ => None,
                }),
            cost: fields
                .get("cost")
                .and_then(|f| first_expr(f))
                .and_then(|x| self.money(x)),
            event: fields
                .get("event")
                .and_then(|f| first_expr(f))
                .and_then(|x| self.pattern(x)),
            deadline: self.edge_deadline(fields.get("within"), fields.get("deadline")),
            duration: fields.get("duration").and_then(|f| self.duration(f)),
            valence: self.valence(fields.get("valence")),
            court_listener_url: Self::string(fields.get("court_listener_url")),
            action_id: Self::string(fields.get("action_id")),
            note: Self::string(fields.get("note")),
            tags: self.idents(fields.get("tags")),
            attrs: self.attrs(fields.get("attrs")),
            sets: self.idents(fields.get("sets")),
            clears: self.idents(fields.get("clears")),
            requires: self.idents(fields.get("requires")),
            forbids: self.idents(fields.get("forbids")),
            span: self.ck.span(e.range),
        }
    }

    /// An event pattern (trailing arguments may be omitted).
    pub fn pattern(&mut self, x: &Expr) -> Option<crate::hir::Expr> {
        match &x.kind {
            ExprKind::Call(callee, args) => {
                let ch = self.ck.event_call(callee, args, true, x.range);
                ch.ty.map(|_| ch.hir)
            }
            _ => {
                self.report(Report::new(
                    C::TypeMismatch,
                    x.range,
                    "expected an event pattern such as `Served(complaint)`",
                ));
                None
            }
        }
    }

    fn probability(
        &mut self,
        e: &EdgeItem,
        p: Option<&Expr>,
        self_edge: bool,
        chance: bool,
    ) -> Option<Authored<Rational>> {
        let Some(p) = p else {
            if chance && !self_edge {
                self.report(
                    Report::new(
                        C::InvalidProbability,
                        e.name_range.cover(e.label.range),
                        "every edge of a chance node needs `p`",
                    )
                    .fix(
                        e.to.range.at_end(),
                        " p ?",
                        Applicability::HasPlaceholders,
                    ),
                );
            }
            return None;
        };
        if self_edge {
            self.report(Report::new(
                C::InvalidProbability,
                p.range,
                "a self edge must not carry `p`",
            ));
            return None;
        }
        if let ExprKind::Hole(reason) = &p.kind {
            self.ck.holes.push((p.range, reason.clone()));
            return Some(Authored::Hole(reason.clone()));
        }
        let num = self.ck.infer(p);
        match (&p.kind, num.value) {
            (_, Some(Value::Number(v) | Value::Probability(v))) if q::is_probability(v) => {
                Some(Authored::Known(v))
            }
            (_, Some(Value::Number(v) | Value::Probability(v))) => {
                self.report(Report::new(
                    C::InvalidProbability,
                    p.range,
                    format!("probability {} is outside [0, 1]", q::display(v)),
                ));
                None
            }
            _ => {
                if num.ty.is_some() {
                    self.report(Report::new(
                        C::InvalidProbability,
                        p.range,
                        "`p` must be a constant probability",
                    ));
                }
                None
            }
        }
    }

    fn edge_deadline(
        &mut self,
        within: Option<&&Field>,
        inline: Option<&&Field>,
    ) -> Option<EdgeDeadline> {
        let f = within.or(inline)?;
        let window = if within.is_some() {
            let ExprKind::Ref(r) = &first_expr(f)?.kind else {
                return None;
            };
            DeadlineWindow::Declared {
                deadline: self.value_ref(r, "deadline", |k| matches!(k, ValueKind::Deadline))?,
            }
        } else {
            let x = first_expr(f)?;
            match self.ck.infer(x).value {
                Some(Value::Days {
                    amount,
                    unit: unit @ (DayUnit::Calendar | DayUnit::Court | DayUnit::Business),
                }) => match u64::try_from(amount) {
                    Ok(length) => DeadlineWindow::Inline { length, unit },
                    Err(_) => {
                        self.report(Report::new(
                            C::ArithmeticError,
                            x.range,
                            "a deadline cannot be negative",
                        ));
                        return None;
                    }
                },
                _ => {
                    self.report(Report::new(
                        C::UnitMismatch,
                        x.range,
                        "an edge `deadline` is a day count such as `30.days` or `5.court_days`",
                    ));
                    return None;
                }
            }
        };
        let opts = nested(f);
        let opts = self.once(opts);
        Some(EdgeDeadline {
            window,
            extendable: opts
                .get("extendable")
                .and_then(|o| first_expr(o))
                .and_then(|x| match x.kind {
                    ExprKind::Bool(b) => Some(b),
                    _ => None,
                }),
            extension_authority: Self::string(opts.get("extension_authority")),
            note: Self::string(opts.get("note")),
        })
    }

    fn duration(&mut self, f: &Field) -> Option<TriangularDuration> {
        let days = Type::Days(DayUnit::Calendar);
        let get = |cx: &mut Self, x: &Expr| match cx.constant(x, &days) {
            Some(Value::Days { amount, .. }) => Some(Rational::integer(amount)),
            _ => None,
        };
        if let Some(x) = first_expr(f) {
            return get(self, x).map(|mode| TriangularDuration {
                min: None,
                mode,
                max: None,
            });
        }
        let parts = self.once(nested(f));
        let mut vals = Vec::new();
        for key in ["min", "mode", "max"] {
            match parts.get(key).and_then(|p| first_expr(p)) {
                Some(x) => vals.push(get(self, x)?),
                None => {
                    self.report(Report::new(
                        C::InvalidField,
                        f.key.range,
                        format!("triangular `duration` needs `{key}`"),
                    ));
                    return None;
                }
            }
        }
        let (min, mode, max) = (vals[0], vals[1], vals[2]);
        if q::cmp(min, mode).is_gt() || q::cmp(mode, max).is_gt() {
            self.report(Report::new(
                C::InvalidField,
                f.range,
                "`duration` needs min <= mode <= max",
            ));
        }
        Some(TriangularDuration {
            min: Some(min),
            mode,
            max: Some(max),
        })
    }
}
