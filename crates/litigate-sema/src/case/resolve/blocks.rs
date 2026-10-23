//! The `facts`, `stakes`, `beliefs`, and `analysis` blocks of a matter.
use super::{child, children, field_value, money, sig, token_text, R};
use crate::db::parse::ref_path;
use crate::db::parse::tree::Json;
use crate::db::span::{Report, TextRange};
use crate::hir::{Analysis, Observation, Stake};
use litigate_base::DiagnosticCode as C;
use litigate_syntax::{SyntaxKind as K, SyntaxNode};
use std::collections::BTreeMap;

fn json(j: &Json) -> Option<serde_json::Value> {
    Some(match j {
        Json::Number(n) => serde_json::from_str(&n.replace('_', "")).ok()?,
        Json::Str(s) => serde_json::Value::String(s.clone()),
        Json::Bool(b) => serde_json::Value::Bool(*b),
        Json::Unset => serde_json::Value::Null,
        Json::Object(fields) => serde_json::Value::Object(
            fields
                .iter()
                .map(|(k, v)| Some((k.text.clone(), json(v)?)))
                .collect::<Option<_>>()?,
        ),
        Json::Array(items) => {
            serde_json::Value::Array(items.iter().map(json).collect::<Option<_>>()?)
        }
    })
}

impl R<'_, '_> {
    /// Record entries keyed by a node reference: (node, value node).
    fn keyed(&mut self, block: &SyntaxNode) -> Vec<(String, TextRange, SyntaxNode)> {
        let Some(record) = child(block, K::RecordExpr) else {
            return Vec::new();
        };
        children(&record, K::RecordField)
            .filter_map(|f| {
                // A field starts at its key; only its value can be missing.
                let (Some(key), Some(value)) = (
                    child(&f, K::Ref).and_then(|r| ref_path(&r)),
                    field_value(&f),
                ) else {
                    return None;
                };
                Some((self.node(&key)?, key.range, value))
            })
            .collect()
    }

    fn edge_of(node: &str, value: &SyntaxNode) -> String {
        let pack = node.split_once("::").map_or("", |(p, _)| p);
        format!("{pack}::{}", value.text().to_string().trim())
    }

    pub(super) fn facts(&mut self, block: &SyntaxNode) -> BTreeMap<String, String> {
        let mut facts = BTreeMap::new();
        for (node, key, value) in self.keyed(block) {
            self.spans
                .facts
                .insert(node.clone(), (self.s(key), self.s(sig(&value))));
            facts.insert(node.clone(), Self::edge_of(&node, &value));
        }
        facts
    }

    pub(super) fn stakes(&mut self, block: &SyntaxNode, text: &str) -> BTreeMap<String, Stake> {
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
        for (node, _, value) in self.keyed(block) {
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
        stakes
    }

    pub(super) fn beliefs(&mut self, block: &SyntaxNode) -> Vec<Observation> {
        let mut observations = Vec::new();
        for obs in children(block, K::ObserveDecl) {
            let Some(node_ref) = child(&obs, K::Ref).and_then(|r| ref_path(&r)) else {
                continue;
            };
            let node_span = Some(self.s(node_ref.range));
            let Some(node) = self.node(&node_ref) else {
                continue;
            };
            let mut outcomes = Vec::new();
            let mut outcome_spans = Vec::new();
            let record = child(&obs, K::RecordExpr);
            for f in record.iter().flat_map(|r| children(r, K::RecordField)) {
                let (Some(edge), Some(value)) = (child(&f, K::Ref), field_value(&f)) else {
                    continue;
                };
                outcome_spans.push(self.s(sig(&edge)));
                match value
                    .text()
                    .to_string()
                    .trim()
                    .replace('_', "")
                    .parse::<u64>()
                {
                    Ok(n) => outcomes.push((Self::edge_of(&node, &edge), n)),
                    Err(_) => self.out.push(Report::new(
                        C::TypeMismatch,
                        sig(&value),
                        "an observation count is a non-negative integer",
                    )),
                }
            }
            self.spans.observations.push((node_span, outcome_spans));
            observations.push(Observation { node, outcomes });
        }
        observations
    }

    pub(super) fn analysis(&mut self, block: &SyntaxNode) -> Analysis {
        let mut fields = BTreeMap::new();
        for f in child(block, K::RecordExpr)
            .iter()
            .flat_map(|r| children(r, K::RecordField))
        {
            let (Some((key, _)), Some(value)) = (token_text(&f, K::Ident), field_value(&f)) else {
                continue;
            };
            match self.lower.json(&value).as_ref().and_then(json) {
                Some(v) => {
                    fields.insert(key, v);
                }
                None => self.out.push(Report::new(
                    C::InvalidAnalysis,
                    sig(&value),
                    format!("`{key}` is not a JSON value"),
                )),
            }
        }
        Analysis { fields }
    }
}
