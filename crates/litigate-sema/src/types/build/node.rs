//! Graph nodes and their edges (SPEC §5.1, field types from §6).
use super::cx::{first_expr, first_ref, holes_of, Cx};
use super::edge::Draw;
use crate::db::parse::tree::{EdgeItem, Expr, ExprKind, Field, Item, NodeItem, RefPath};
use crate::db::span::Report;
use crate::hir::{Authored, Edge, Money, Node, NodeKind, Rational, Type, Valence, Value};
use crate::resolve::suggest;
use crate::temporal::condition;
use crate::types::rational as q;
use litigate_base::{Applicability, DiagnosticCode as C};
use std::collections::{BTreeMap, BTreeSet};

type Fields<'f> = BTreeMap<&'f str, &'f Field>;

/// Most gating flags one chance node's contexts are enumerated over.
const MAX_GATES: usize = 12;

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
        decision: bool,
    ) {
        for (key, ok, why) in [
            ("discretion", chance, "on non-chance nodes"),
            ("info", decision, "on non-decision nodes"),
            ("outcome", terminal, "on non-terminal nodes"),
        ] {
            if let Some(f) = clauses.get(key).filter(|_| !ok) {
                self.not_allowed(f, why);
            }
        }
        for key in ["payoff", "payoff_by_flag", "fee_shift"] {
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
        let fee_shift = match fields
            .get("fee_shift")
            .and_then(|f| first_expr(f))
            .and_then(|e| self.constant(e, &Type::Probability))
        {
            Some(Value::Probability(f)) => Some(f),
            _ => None,
        };
        NodeKind::Terminal {
            outcomes: self.idents(clauses.get("outcome")),
            payoff,
            payoff_by_flag: by_flag,
            fee_shift,
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
        self.placement(&clauses, &fields, chance, terminal, kw == "decision");
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
            let draw = if chance {
                Draw::Required
            } else {
                Draw::Forbidden
            };
            let fact = chance && tags.iter().any(|t| t == "fact");
            edges.push(self.edge(e, &n.name.text, actor.as_deref(), draw, fact));
        }
        self.probability_sum(n, &edges, chance);
        self.disjoint_guards(n, &edges);
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
            info: first_ref(clauses.get("info"))
                .filter(|_| kw == "decision")
                .map(RefPath::display),
            basis: self.basis(fields.get("basis")),
            edges,
            formerly: self.idents(fields.get("formerly")),
            holes: holes_of(n.clauses.iter().chain(&n.fields)),
        }
    }

    /// The `when` guards of a fact node's edges are pairwise disjoint
    /// (L2005): at most one holds on any history, so the fact replay
    /// derives never depends on the order the edges are written in.
    fn disjoint_guards(&mut self, n: &NodeItem, edges: &[Edge]) {
        let guarded: Vec<(&EdgeItem, &Edge, &crate::hir::Expr)> = n
            .edges
            .iter()
            .zip(edges)
            .filter_map(|(item, e)| Some((item, e, e.guard.as_ref()?)))
            .collect();
        for (j, &(item, b, gb)) in guarded.iter().enumerate() {
            for &(_, a, ga) in &guarded[..j] {
                let sat = condition::jointly_satisfiable(&[ga, gb]);
                if sat == Some(false) {
                    continue;
                }
                let why = if sat.is_none() {
                    "their guards have too many atoms to prove disjoint"
                } else {
                    "some history makes both guards true"
                };
                self.report(
                    Report::new(
                        C::NonExhaustiveMatch,
                        item.name_range.cover(item.label.range),
                        format!(
                            "the guards of `{}` and `{}` on fact `{}` can both hold",
                            a.label, b.label, n.name.text
                        ),
                    )
                    .note(why)
                    .note("make the guards exclusive (`c` and `!c`): a fact has one value"),
                );
            }
        }
    }

    /// `dist-exact` (SPEC §5.1, §13 O8): the `p` edges of a chance node
    /// are `Dist<Edges(n)>` minus its self edges, which carry no `p`
    /// (L2008). Its known probabilities sum to exactly 1, or to at most 1
    /// while any is a hole. No other node carries `p` (L2008).
    ///
    /// Edges gated by `requires`/`forbids` make the distribution
    /// conditional on the flags an earlier edge set (`Dist[E | C]`, design
    /// review 2026-10-23 §2.11): the check runs once per context, a set of
    /// the node's gating flags, over the edges that context enables. A
    /// context enabling none draws nothing there, so it is not checked.
    fn probability_sum(&mut self, n: &NodeItem, edges: &[Edge], chance: bool) {
        if !chance {
            return;
        }
        let drawn: Vec<&Edge> = edges.iter().filter(|e| e.to != n.name.text).collect();
        let gates: Vec<&str> = drawn
            .iter()
            .flat_map(|e| e.requires.iter().chain(&e.forbids))
            .map(String::as_str)
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect();
        if gates.len() > MAX_GATES {
            self.report(
                Report::new(
                    C::InvalidDistribution,
                    n.name.range,
                    format!(
                        "chance node `{}` conditions its probabilities on {} flags, more than the {MAX_GATES} whose contexts can be checked",
                        n.name.text,
                        gates.len()
                    ),
                )
                .note("split the node, or gate its edges on fewer flags"),
            );
            return;
        }
        for mask in 0..1_usize << gates.len() {
            let set: Vec<&str> = gates
                .iter()
                .enumerate()
                .filter(|(i, _)| mask >> i & 1 == 1)
                .map(|(_, g)| *g)
                .collect();
            let live: Vec<&Edge> = drawn
                .iter()
                .copied()
                .filter(|e| {
                    e.requires.iter().all(|r| set.contains(&r.as_str()))
                        && !e.forbids.iter().any(|f| set.contains(&f.as_str()))
                })
                .collect();
            let skip = if gates.is_empty() {
                edges.is_empty()
            } else {
                live.is_empty()
            };
            if !skip && !self.context_sum(n, &live, (!gates.is_empty()).then_some(&set)) {
                return;
            }
        }
    }

    /// `dist-exact` for the edges `live` one flag context (`set`, `None`
    /// when no edge is gated) enables; false once it has reported.
    fn context_sum(&mut self, n: &NodeItem, live: &[&Edge], set: Option<&Vec<&str>>) -> bool {
        let known: Vec<Rational> = live
            .iter()
            .filter_map(|e| match &e.probability {
                Some(Authored::Known(p)) => Some(*p),
                _ => None,
            })
            .collect();
        let open = live
            .iter()
            .any(|e| matches!(e.probability, Some(Authored::Hole(_)) | None));
        let one = Rational::integer(1);
        let refinement = if open {
            "while a `p` is open the known ones need `Σ p <= 1`"
        } else {
            "refinement `Dist<Edges(n)> = {m | Σ m(e) = 1}` over the edges other than self edges"
        };
        let context = match set {
            None => String::new(),
            Some(set) if set.is_empty() => " with none of its gating flags set".to_owned(),
            Some(set) => format!(" with only flags {{{}}} set", set.join(", ")),
        };
        let message = match q::sum(known) {
            Ok(total) if !open && total != one => format!(
                "probabilities of chance node `{}` sum to {}, not exactly 1{context}",
                n.name.text,
                q::display(total)
            ),
            Ok(total) if open && q::cmp(total, one).is_gt() => format!(
                "known probabilities of `{}` already sum to {}{context}",
                n.name.text,
                q::display(total)
            ),
            Ok(_) => return true,
            Err(e) => {
                self.report(Report::new(C::ArithmeticError, n.name.range, e.message()));
                return false;
            }
        };
        let mut report =
            Report::new(C::InvalidDistribution, n.name.range, message).note(refinement);
        if set.is_some() {
            report = report.note(
                "each set of the node's `requires`/`forbids` flags draws from the edges it enables; gate the edges so one context enables one distribution (`forbids [x]` on the others)",
            );
        }
        self.report(report);
        false
    }
}
