//! Graph nodes and their edges (SPEC §5.1, field types from §6).
use super::cx::{first_expr, Cx};
use crate::db::parse::tree::{Expr, ExprKind, Field, Item, NodeItem};
use crate::db::span::Report;
use crate::hir::{Authored, Edge, Money, Node, NodeKind, Rational, Type, Valence, Value};
use crate::resolve::suggest;
use crate::types::rational as q;
use litigate_base::{Applicability, DiagnosticCode as C};
use std::collections::{BTreeMap, BTreeSet};

type Fields<'f> = BTreeMap<&'f str, &'f Field>;

const VALENCES: [&str; 4] = ["good", "caution", "bad", "neutral"];

impl Cx<'_, '_> {
    pub(super) fn valence(&mut self, f: Option<&&Field>) -> Option<Valence> {
        let Some(ExprKind::Ref(r)) = first_expr(f?).map(|e| &e.kind) else {
            return None;
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

    pub(super) fn attrs(&mut self, f: Option<&&Field>) -> BTreeMap<String, Rational> {
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

    /// Fields and clauses that only some node kinds allow (L2007).
    fn placement(
        &mut self,
        clauses: &Fields<'_>,
        fields: &Fields<'_>,
        chance: bool,
        terminal: bool,
    ) {
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
    }

    fn terminal(&mut self, clauses: &Fields<'_>, fields: &Fields<'_>) -> NodeKind {
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

    /// Builds one node.
    pub fn node(&mut self, n: &NodeItem, item: &Item, rule: Option<&Item>) -> Node {
        let meta = self.meta(&n.name, Some(&n.label), item, rule);
        let clauses = self.once(&n.clauses);
        let fields = self.once(&n.fields);
        let kw = n.keyword.text.as_str();
        let chance = kw == "chance";
        let terminal = kw == "terminal";
        self.placement(&clauses, &fields, chance, terminal);
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
            "terminal" => self.terminal(&clauses, &fields),
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
}
