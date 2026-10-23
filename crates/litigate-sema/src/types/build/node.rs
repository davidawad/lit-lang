//! Graph nodes and their edges (SPEC §5.1, field types from §6).
use super::cx::{first_expr, first_ref, Cx};
use crate::db::parse::tree::{Expr, ExprKind, Field, Item, NodeItem, RefPath};
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
        let r = first_ref(f)?;
        let v = match r.head.text.as_str() {
            "good" => Some(Valence::Good),
            "caution" => Some(Valence::Caution),
            "bad" => Some(Valence::Bad),
            "neutral" => Some(Valence::Neutral),
            _ => None,
        }
        .filter(|_| r.is_simple());
        if v.is_none() {
            let shown = r.display();
            let mut report = Report::new(
                C::UnresolvedName,
                r.range,
                format!("unknown valence `{shown}`"),
            )
            .note("expected `good`, `caution`, `bad`, or `neutral`");
            if let Some(best) = suggest::nearest(&shown, VALENCES) {
                report = report.fix(r.range, best, Applicability::MaybeIncorrect);
            }
            self.report(report);
        }
        v
    }

    /// Money<usd> or a hole. The grammar admits only (signed) money
    /// literals and holes here, so a well-typed value is always constant.
    pub fn money(&mut self, e: &Expr) -> Option<Authored<Money>> {
        match self.ck.check(e, &Type::Money("usd".into())).value {
            Some(Value::Money(m)) => Some(Authored::Known(m)),
            Some(Value::Hole(reason)) => Some(Authored::Hole(reason)),
            _ => None,
        }
    }

    /// A constant of type `ty` (attrs, hours, days): the grammar admits
    /// only literals here, so a well-typed value is always constant.
    pub fn constant(&mut self, e: &Expr, ty: &Type) -> Option<Value> {
        self.ck.check(e, ty).value
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
            // Values are signed money literals (no holes, SPEC §3.1).
            for (flag, e) in rs {
                if let Some(Authored::Known(m)) = self.money(e) {
                    by_flag.insert(flag.text.clone(), m);
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
        // A fact is a draw nature controls: only a `chance` node (SPEC §5.1).
        let misplaced_fact = !chance && tags.iter().any(|t| t == "fact");
        if let Some(f) = fields.get("tags").filter(|_| misplaced_fact) {
            self.report(Report::new(
                C::InvalidField,
                f.range,
                "the `fact` tag is allowed only on `chance` nodes",
            ));
        }
        let group = first_ref(fields.get("group"));
        if let Some(Err(report)) = group.map(|r| self.ck.scope.group(&r.head)) {
            self.report(report);
        }
        let actor = first_ref(clauses.get("by")).map(|r| self.actor(r));
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
        self.probability_sum(n, &edges, chance);
        Node {
            meta,
            kind,
            group: group.map(RefPath::display),
            note: Self::string(fields.get("note")),
            valence: self.valence(fields.get("valence")),
            court_listener_url: Self::string(fields.get("court_listener_url")),
            tags,
            attrs: self.attrs(fields.get("attrs")),
            actor,
            edges,
        }
    }

    /// `dist-exact` (SPEC §5.1, §13 O8): the `p` edges of a node are
    /// `Dist<Edges(n)>` minus its self edges, which carry no `p` (L2008).
    /// A chance node's known probabilities sum to exactly 1, or to at most
    /// 1 while any is a hole. Any other node with `p` edges draws among
    /// them: at most 1, and exactly 1 when every edge but a self edge
    /// carries a known `p` (the engine would otherwise renormalise).
    fn probability_sum(&mut self, n: &NodeItem, edges: &[Edge], chance: bool) {
        let drawn: Vec<&Edge> = edges.iter().filter(|e| e.to != n.name.text).collect();
        if !chance && drawn.iter().all(|e| e.probability.is_none()) {
            return;
        }
        let known: Vec<Rational> = drawn
            .iter()
            .filter_map(|e| match &e.probability {
                Some(Authored::Known(p)) => Some(*p),
                _ => None,
            })
            .collect();
        let open = drawn
            .iter()
            .any(|e| matches!(e.probability, Some(Authored::Hole(_)) | None));
        let one = Rational::integer(1);
        let refinement = if open {
            "while a `p` is open the known ones need `Σ p <= 1`"
        } else {
            "refinement `Dist<Edges(n)> = {m | Σ m(e) = 1}` over the edges other than self edges"
        };
        match q::sum(known) {
            Ok(total) if !open && total != one && !edges.is_empty() => {
                self.report(
                    Report::new(
                        C::InvalidDistribution,
                        n.name.range,
                        format!(
                            "probabilities of {} `{}` sum to {}, not exactly 1",
                            if chance { "chance node" } else { "node" },
                            n.name.text,
                            q::display(total)
                        ),
                    )
                    .note(refinement),
                );
            }
            Ok(total) if open && q::cmp(total, one).is_gt() => {
                self.report(
                    Report::new(
                        C::InvalidDistribution,
                        n.name.range,
                        format!(
                            "known probabilities of `{}` already sum to {}",
                            n.name.text,
                            q::display(total)
                        ),
                    )
                    .note(refinement),
                );
            }
            Ok(_) => {}
            Err(e) => self.report(Report::new(C::ArithmeticError, n.name.range, e.message())),
        }
    }
}
