//! Stake expansion from terminal ids and declared outcome tags.
use super::{child, children, field_value, money, sig, R};
use crate::db::span::{Report, TextRange};
use crate::hir::{NodeKind, Rational, Stake};
use litigate_base::DiagnosticCode as C;
use litigate_syntax::{SyntaxKind as K, SyntaxNode};
use std::collections::BTreeMap;

#[derive(Default)]
struct StakeEntries {
    stakes: BTreeMap<String, Stake>,
    fee_shifts: BTreeMap<String, Rational>,
    explicit_stakes: std::collections::BTreeSet<String>,
    explicit_fee_shifts: std::collections::BTreeSet<String>,
}

type OutcomeRow = (String, TextRange, SyntaxNode, Option<SyntaxNode>);

impl R<'_, '_> {
    /// The `stakes` block: each terminal's stake, and the fee-shift
    /// fractions its entries give (`j: 1.usd fee_shift 0.5`).
    pub(super) fn stakes(
        &mut self,
        block: &SyntaxNode,
        text: &str,
        as_of: &str,
        known_at: Option<&str>,
    ) -> (BTreeMap<String, Stake>, BTreeMap<String, Rational>) {
        self.record_stakes_end(block, text);
        let mut entries = StakeEntries::default();
        self.explicit_stakes(block, &mut entries);
        self.outcome_stakes(block, as_of, known_at, &mut entries);
        (entries.stakes, entries.fee_shifts)
    }

    fn record_stakes_end(&mut self, block: &SyntaxNode, text: &str) {
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
    }

    fn explicit_stakes(&mut self, block: &SyntaxNode, entries: &mut StakeEntries) {
        for (node, key, f) in self.entries(block) {
            if let Some(v) = child(&f, K::Field).and_then(|s| self.fee_shift(&s)) {
                entries.explicit_fee_shifts.insert(node.clone());
                entries.fee_shifts.insert(node.clone(), v);
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
                    entries.explicit_stakes.insert(node.clone());
                    entries.stakes.insert(node, s);
                }
                None => self.out.push(Report::new(
                    C::TypeMismatch,
                    range,
                    "expected a money amount like `-2_000.usd`, or `unvalued`",
                )),
            }
        }
    }

    fn outcome_stakes(
        &mut self,
        block: &SyntaxNode,
        as_of: &str,
        known_at: Option<&str>,
        entries: &mut StakeEntries,
    ) {
        let outcome_rows = Self::outcome_rows(block);
        let mut outcome_stakes = BTreeMap::<String, (Stake, String, TextRange)>::new();
        let mut outcome_fees = BTreeMap::<String, (Rational, String, TextRange)>::new();
        for (tag, tag_range, field, value) in outcome_rows {
            let fee = child(&field, K::Field).and_then(|f| self.fee_shift(&f));
            let stake = value.as_ref().and_then(|value| {
                let range = sig(value);
                let written: String = value.text().to_string().split_whitespace().collect();
                if written == "unvalued" {
                    Some((Stake::Unvalued, range))
                } else {
                    super::money(&written).map(|money| (Stake::Value(money), range))
                }
            });
            let terminals = self.terminals_with_outcome(&tag, as_of, known_at);
            if terminals.is_empty() {
                self.out.push(Report::new(
                    C::MissingStake,
                    tag_range,
                    format!("no terminal in force on {as_of} declares outcome `{tag}`"),
                ));
                continue;
            }
            for terminal in terminals {
                if let Some((value, value_range)) = &stake {
                    if let Some((previous, previous_tag, previous_range)) =
                        outcome_stakes.get(&terminal)
                    {
                        if previous != value && !entries.explicit_stakes.contains(&terminal) {
                            self.out.push(
                                Report::new(
                                    C::MissingStake,
                                    *value_range,
                                    format!(
                                        "terminal `{terminal}` matches outcomes `{previous_tag}` and `{tag}` with different stakes"
                                    ),
                                )
                                .note_at(*previous_range, "the other outcome stake is here")
                                .note("add an explicit terminal stake to choose its value"),
                            );
                        }
                    } else {
                        outcome_stakes
                            .insert(terminal.clone(), (value.clone(), tag.clone(), *value_range));
                    }
                }
                if let Some(fee) = fee {
                    if let Some((previous, previous_tag, previous_range)) =
                        outcome_fees.get(&terminal)
                    {
                        if *previous != fee && !entries.explicit_fee_shifts.contains(&terminal) {
                            self.out.push(
                                Report::new(
                                    C::MissingStake,
                                    tag_range,
                                    format!(
                                        "terminal `{terminal}` matches outcomes `{previous_tag}` and `{tag}` with different fee shifts"
                                    ),
                                )
                                .note_at(*previous_range, "the other outcome fee shift is here")
                                .note("add an explicit terminal `fee_shift` to choose its value"),
                            );
                        }
                    } else {
                        outcome_fees.insert(terminal.clone(), (fee, tag.clone(), tag_range));
                    }
                }
            }
        }
        for (terminal, (stake, _, range)) in outcome_stakes {
            if !entries.explicit_stakes.contains(&terminal) {
                entries.stakes.insert(terminal.clone(), stake);
                self.spans.stakes.insert(terminal, self.s(range));
            }
        }
        for (terminal, (fee, _, _)) in outcome_fees {
            if !entries.explicit_fee_shifts.contains(&terminal) {
                entries.fee_shifts.insert(terminal, fee);
            }
        }
    }

    fn outcome_rows(block: &SyntaxNode) -> Vec<OutcomeRow> {
        child(block, K::RecordExpr)
            .map(|record| {
                children(&record, K::RecordField)
                    .filter_map(|field| {
                        let mut tokens = field
                            .children_with_tokens()
                            .filter_map(rowan::NodeOrToken::into_token);
                        tokens.find(|token| token.kind() == K::KwOutcome)?;
                        let tag = tokens.find(|token| token.kind() == K::Ident)?;
                        let value = field_value(&field);
                        Some((
                            tag.text().to_owned(),
                            crate::db::parse::range(tag.text_range()),
                            field,
                            value,
                        ))
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Current terminal nodes tagged with the declared outcome.
    fn terminals_with_outcome(
        &self,
        outcome: &str,
        as_of: &str,
        known_at: Option<&str>,
    ) -> Vec<String> {
        let Some(graph) = &self.graph else {
            return Vec::new();
        };
        let mut terminals = std::collections::BTreeSet::new();
        for package in &graph.packages {
            for declaration in package
                .modules
                .iter()
                .flat_map(|module| &module.declarations)
            {
                let crate::hir::Declaration::Node(node) = declaration else {
                    continue;
                };
                let qualified = format!("{}::{}", package.manifest.pack_id, node.meta.id);
                if graph.node_at(&qualified, as_of, known_at).is_some_and(|current| {
                    matches!(
                        &current.kind,
                        NodeKind::Terminal { outcomes, .. } if outcomes.iter().any(|tag| tag == outcome)
                    )
                }) {
                    terminals.insert(qualified);
                }
            }
        }
        terminals.into_iter().collect()
    }
}
