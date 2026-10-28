//! The `facts`, `stakes`, `beliefs`, and `opponents` blocks of a matter.
use super::{child, children, field_value, money, sig, R};
use crate::db::parse::ref_path;
use crate::db::span::{Report, TextRange};
use crate::hir::{NodeKind, Observation, OpponentModel, Prior, Rational, Stake};
use crate::types::rational;
use litigate_base::{Applicability, DiagnosticCode as C};
use litigate_syntax::{SyntaxKind as K, SyntaxNode};
use std::collections::BTreeMap;

impl R<'_, '_> {
    /// Record entries keyed by a node reference: (node, key range, value node).
    fn keyed(&mut self, block: &SyntaxNode) -> Vec<(String, TextRange, SyntaxNode)> {
        self.entries(block)
            .into_iter()
            .filter_map(|(node, key, f)| field_value(&f).map(|v| (node, key, v)))
            .collect()
    }

    /// Record fields keyed by a node reference that have a value or a
    /// `fee_shift` field: (node, key range, field).
    fn entries(&mut self, block: &SyntaxNode) -> Vec<(String, TextRange, SyntaxNode)> {
        let Some(record) = child(block, K::RecordExpr) else {
            return Vec::new();
        };
        children(&record, K::RecordField)
            .filter_map(|f| {
                // A field starts at its key; only its value can be missing.
                let given = child(&f, K::Field).is_some() || field_value(&f).is_some();
                let (Some(key), true) = (child(&f, K::Ref).and_then(|r| ref_path(&r)), given)
                else {
                    return None;
                };
                Some((self.node(&key)?, key.range, f))
            })
            .collect()
    }

    /// A fact's edge, qualified by its node's pack unless written
    /// `pack::edge`; an edge's former id (`formerly`) resolves to its
    /// current one (L9007).
    fn edge_of(&mut self, node: &str, value: &SyntaxNode) -> String {
        let edge = value.text().to_string();
        let edge = edge.trim();
        if edge.contains("::") {
            return edge.to_owned();
        }
        let pack = node.split_once("::").map_or("", |(p, _)| p);
        let current = self.graph.as_ref().and_then(|g| g.renamed_edge(node, edge));
        if let Some(current) = current {
            let range = sig(value);
            self.out.push(
                Report::new(
                    C::FormerName,
                    range,
                    format!("edge `{edge}` was renamed `{current}`"),
                )
                .fix(range, current.clone(), Applicability::MachineApplicable),
            );
            return format!("{pack}::{current}");
        }
        format!("{pack}::{edge}")
    }

    pub(super) fn facts(&mut self, block: &SyntaxNode) -> BTreeMap<String, String> {
        let mut facts = BTreeMap::new();
        for (node, key, value) in self.keyed(block) {
            self.spans
                .facts
                .insert(node.clone(), (self.s(key), self.s(sig(&value))));
            let edge = self.edge_of(&node, &value);
            facts.insert(node.clone(), edge);
        }
        facts
    }

    /// The `stakes` block: each terminal's stake, and the fee-shift
    /// fractions its entries give (`j: 1.usd fee_shift 0.5`).
    pub(super) fn stakes(
        &mut self,
        block: &SyntaxNode,
        text: &str,
    ) -> (BTreeMap<String, Stake>, BTreeMap<String, Rational>) {
        if let Some(close) = child(block, K::RecordExpr)
            .and_then(|r| r.last_token())
            .filter(|t| t.kind() == K::RBrace)
        {
            let at = crate::db::parse::range(close.text_range());
            let before = text.get(..at.start as usize).unwrap_or_default();
            let multiline = before
                .rsplit('\n')
                .next()
                .is_some_and(|l| l.trim().is_empty());
            let (insert, prefix, suffix) = if multiline {
                let line_start =
                    u32::try_from(before.rfind('\n').map_or(0, |i| i + 1)).unwrap_or(at.start);
                (
                    TextRange::new(line_start, line_start),
                    "  ".to_owned(),
                    "\n".to_owned(),
                )
            } else {
                let last = before.trim_end();
                let end = u32::try_from(last.len()).unwrap_or(at.start);
                let sep = if last.ends_with('{') { " " } else { ", " };
                (TextRange::new(end, end), sep.to_owned(), String::new())
            };
            self.spans.stakes_end = Some((self.s(insert), prefix, suffix));
        }
        let mut stakes = BTreeMap::new();
        let mut fee_shifts = BTreeMap::new();
        for (node, key, f) in self.entries(block) {
            if let Some(v) = child(&f, K::Field).and_then(|s| self.fee_shift(&s)) {
                fee_shifts.insert(node.clone(), v);
            }
            let Some(value) = field_value(&f) else {
                self.spans.stakes.insert(node, self.s(key));
                continue;
            };
            let range = sig(&value);
            self.spans.stakes.insert(node.clone(), self.s(range));
            let written: String = value.text().to_string().split_whitespace().collect();
            let stake = if written == "unvalued" {
                Some(Stake::Unvalued)
            } else {
                money(&written).map(Stake::Value)
            };
            match stake {
                Some(s) => {
                    stakes.insert(node, s);
                }
                None => self.out.push(Report::new(
                    C::TypeMismatch,
                    range,
                    "expected a money amount like `-2_000.usd`, or `unvalued`",
                )),
            }
        }
        (stakes, fee_shifts)
    }

    /// A stake's `fee_shift` fraction, in `[0, 1]`.
    fn fee_shift(&mut self, field: &SyntaxNode) -> Option<Rational> {
        let (text, range) = super::token_text(field, K::Number)?;
        let value = rational::parse_decimal(&text)
            .ok()
            .filter(|r| rational::is_probability(*r));
        if value.is_none() {
            self.out.push(Report::new(
                C::ArithmeticError,
                range,
                format!("fee_shift {text} is not a fraction in [0, 1]"),
            ));
        }
        value
    }

    /// The `observe` and `prior` entries of `beliefs`, each with its node
    /// and outcome spans.
    pub(super) fn beliefs(&mut self, block: &SyntaxNode) -> (Vec<Observation>, Vec<Prior>) {
        let (mut observations, mut priors) = (Vec::new(), Vec::new());
        for decl in block
            .children()
            .filter(|c| matches!(c.kind(), K::ObserveDecl | K::PriorDecl))
        {
            let prior = decl.kind() == K::PriorDecl;
            let Some(node_ref) = child(&decl, K::Ref).and_then(|r| ref_path(&r)) else {
                continue;
            };
            let node_span = Some(self.s(node_ref.range));
            let Some(node) = self.node(&node_ref) else {
                continue;
            };
            let (mut counts, mut outcome_spans) = (Vec::new(), Vec::new());
            let record = child(&decl, K::RecordExpr);
            for f in record.iter().flat_map(|r| children(r, K::RecordField)) {
                let (Some(edge), Some(value)) = (child(&f, K::Ref), field_value(&f)) else {
                    continue;
                };
                outcome_spans.push(self.s(sig(&edge)));
                let text = value.text().to_string();
                let count = if prior {
                    rational::parse_decimal(text.trim())
                        .ok()
                        .filter(|r| r.numerator > 0)
                } else {
                    text.trim()
                        .replace('_', "")
                        .parse::<i64>()
                        .ok()
                        .filter(|n| *n >= 0)
                        .map(Rational::integer)
                };
                match count {
                    Some(n) => {
                        let edge = self.edge_of(&node, &edge);
                        counts.push((edge, n));
                    }
                    None => self.out.push(Report::new(
                        C::TypeMismatch,
                        sig(&value),
                        if prior {
                            "a prior pseudo-count is a positive number"
                        } else {
                            "an observation count is a non-negative integer"
                        },
                    )),
                }
            }
            if prior {
                self.spans.priors.push((node_span, outcome_spans));
                priors.push(Prior {
                    node,
                    pseudo_counts: counts,
                });
            } else {
                self.spans.observations.push((node_span, outcome_spans));
                let outcomes = counts
                    .into_iter()
                    .map(|(e, n)| (e, n.numerator.unsigned_abs()))
                    .collect();
                observations.push(Observation { node, outcomes });
            }
        }
        (observations, priors)
    }

    /// The `opponents` block: each decision's opponent model, its node a
    /// declared decision and each edge one of that decision's own.
    pub(super) fn opponents(&mut self, block: &SyntaxNode) -> BTreeMap<String, OpponentModel> {
        let mut models = BTreeMap::new();
        for decl in children(block, K::OpponentModelDecl) {
            let Some(kw) = decl
                .children_with_tokens()
                .filter_map(rowan::NodeOrToken::into_token)
                .map(|t| t.kind())
                .find(|k| matches!(k, K::KwAdversarial | K::KwChance | K::KwOwnObjective))
            else {
                continue;
            };
            let Some(node_ref) = child(&decl, K::Ref).and_then(|r| ref_path(&r)) else {
                continue;
            };
            let Some(node) = self.node(&node_ref) else {
                continue;
            };
            if !self.is_decision(&node, node_ref.range) {
                continue;
            }
            if models.contains_key(&node) {
                self.out.push(Report::new(
                    C::DuplicateName,
                    node_ref.range,
                    format!("`{node}` already has an opponent model"),
                ));
                continue;
            }
            let entries = self.model_entries(&node, &decl);
            let model = match kw {
                K::KwAdversarial => OpponentModel::Adversarial,
                K::KwChance => OpponentModel::Chance {
                    probabilities: entries
                        .into_iter()
                        .filter_map(|(edge, text, range)| {
                            let p = rational::parse_decimal(&text)
                                .ok()
                                .filter(|r| rational::is_probability(*r));
                            if p.is_none() {
                                self.out.push(Report::new(
                                    C::TypeMismatch,
                                    range,
                                    format!("{text} is not a probability in [0, 1]"),
                                ));
                            }
                            Some((edge, p?))
                        })
                        .collect(),
                },
                _ => OpponentModel::OwnObjective {
                    payoffs: entries
                        .into_iter()
                        .filter_map(|(edge, text, range)| {
                            let m = super::money(&text);
                            if m.is_none() {
                                self.out.push(Report::new(
                                    C::TypeMismatch,
                                    range,
                                    "expected the opponent's payoff as a money amount like `-2_000.usd`",
                                ));
                            }
                            Some((edge, m?))
                        })
                        .collect(),
                },
            };
            models.insert(node, model);
        }
        models
    }

    /// Whether qualified `node` is a `decision` (reported at `at` if not).
    fn is_decision(&mut self, node: &str, at: TextRange) -> bool {
        let Some(graph) = self.graph.as_ref() else {
            return false;
        };
        let kinds: Vec<&NodeKind> = graph.node_versions(node).iter().map(|n| &n.kind).collect();
        if kinds.is_empty() || kinds.iter().any(|k| matches!(k, NodeKind::Decision)) {
            return true;
        }
        let kind = match kinds[0] {
            NodeKind::State => "a state",
            NodeKind::Chance { .. } => "a chance node",
            NodeKind::Decision => "a decision",
            NodeKind::Terminal { .. } => "a terminal",
        };
        self.out.push(
            Report::new(
                C::TypeMismatch,
                at,
                format!("`{node}` is {kind}, not a decision: an opponent model names a decision"),
            )
            .note(
                "`opponents` says how the opponent decides at a `decision` where only they choose",
            ),
        );
        false
    }

    /// A model's `{ edge: value }` entries: (qualified edge, value text,
    /// value range), each edge one of `node`'s (L1002 otherwise).
    fn model_entries(&mut self, node: &str, decl: &SyntaxNode) -> Vec<(String, String, TextRange)> {
        let own: Vec<String> = self.graph.as_ref().map_or_else(Vec::new, |g| {
            g.node_versions(node)
                .into_iter()
                .flat_map(|n| g.edges(node, n))
                .filter_map(|(pack, e)| e.id.as_ref().map(|id| format!("{pack}::{id}")))
                .collect()
        });
        let mut out = Vec::new();
        let record = child(decl, K::RecordExpr);
        for f in record.iter().flat_map(|r| children(r, K::RecordField)) {
            let (Some(edge), Some(value)) = (child(&f, K::Ref), field_value(&f)) else {
                continue;
            };
            let qualified = self.edge_of(node, &edge);
            if !own.contains(&qualified) {
                let local = qualified.split_once("::").map_or("", |(_, l)| l);
                let mut r = Report::new(
                    C::UnresolvedName,
                    sig(&edge),
                    format!("decision `{node}` has no edge `{local}`"),
                );
                let locals = own
                    .iter()
                    .filter_map(|e| e.split_once("::").map(|(_, l)| l));
                if let Some(best) = crate::resolve::suggest::nearest(local, locals) {
                    r = r.fix(sig(&edge), best, Applicability::MaybeIncorrect);
                }
                self.out.push(r);
                continue;
            }
            let text: String = value.text().to_string().split_whitespace().collect();
            out.push((qualified, text, sig(&value)));
        }
        out
    }
}
