//! The `facts`, `stakes`, and `beliefs` blocks of a matter.
use super::{child, children, field_value, money, sig, R};
use crate::db::parse::ref_path;
use crate::db::span::{Report, TextRange};
use crate::hir::{Observation, Prior, Rational, Stake};
use crate::types::rational;
use litigate_base::{Applicability, DiagnosticCode as C};
use litigate_syntax::{SyntaxKind as K, SyntaxNode};
use std::collections::BTreeMap;

impl R<'_, '_> {
    /// Record entries keyed by a node reference: (node, key range, value node).
    fn keyed(&mut self, block: &SyntaxNode) -> Vec<(String, TextRange, SyntaxNode)> {
        self.entries(block)
            .into_iter()
            .filter_map(|(node, key, f)| Some((node, key, field_value(&f)?)))
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
                let key = child(&f, K::Ref).and_then(|r| ref_path(&r))?;
                if field_value(&f).is_none() && child(&f, K::Field).is_none() {
                    return None;
                }
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
}
