//! The `analysis` block of a matter (SPEC §8.3), typed field by field: the
//! closed set of litgraph `Scenario` fields a matter may set beyond its
//! dedicated blocks, each with its own type. Nothing passes through
//! unchecked: an engine option the engine's own schema types is checked
//! against it when the matter lowers, at its span.
use super::{child, children, field_value, R};
use crate::case::event;
use crate::db::parse::tree::{Expr, ExprKind};
use crate::db::span::{Report, TextRange};
use crate::hir::{Analysis, EngineExpr, EngineOption, Rational, UnaryOp};
use crate::resolve::suggest::nearest;
use crate::types::rational;
use litigate_base::{Applicability, DiagnosticCode as C};
use litigate_syntax::{SyntaxKind as K, SyntaxNode};
use std::collections::BTreeSet;

/// Fields holding one engine expression.
const EXPRESSIONS: [&str; 5] = [
    "cost",
    "utility",
    "mask",
    "probability_fn",
    "opponent_objective",
];

/// Engine options: values the engine's own schema types.
const OPTIONS: [&str; 6] = [
    "mixed",
    "opponent",
    "prob_fill",
    "objective",
    "fee_shift",
    "uncertainty",
];

/// Every `analysis` field. With [`DEDICATED`] and [`UNTYPED`], these are
/// exactly the fields of litgraph's `Scenario`.
pub const FIELDS: [&str; 17] = [
    "params",
    "metrics",
    "utilities",
    "cost",
    "utility",
    "mask",
    "probability_fn",
    "opponent_objective",
    "calibration",
    "discount_annual",
    "remove_edges",
    "mixed",
    "opponent",
    "prob_fill",
    "objective",
    "fee_shift",
    "uncertainty",
];

/// `Scenario` fields with a dedicated `.case` block.
pub const DEDICATED: [&str; 5] = ["start", "perspective", "facts", "payoffs", "observe"];

/// `Scenario` fields that key engine nodes or edges by string, which a
/// matter does not write.
pub const UNTYPED: [&str; 2] = ["probabilities", "policy"];

impl R<'_, '_> {
    pub(super) fn analysis(&mut self, block: &SyntaxNode) -> Analysis {
        let mut out = Analysis::default();
        let mut seen = BTreeSet::new();
        for f in child(block, K::RecordExpr)
            .iter()
            .flat_map(|r| children(r, K::RecordField))
        {
            let key = crate::db::parse::head(&f);
            let Some(value) = field_value(&f).and_then(|v| self.lower.expr(&v)) else {
                continue;
            };
            if !seen.insert(key.text.clone()) {
                self.invalid(key.range, format!("`{}` is given more than once", key.text));
                continue;
            }
            self.field(&mut out, &key.text, key.range, &value);
        }
        out
    }

    fn invalid(&mut self, range: TextRange, message: String) {
        self.out
            .push(Report::new(C::InvalidAnalysis, range, message));
    }

    /// Types one field `key: value` into `out`.
    fn field(&mut self, out: &mut Analysis, key: &str, at: TextRange, value: &Expr) {
        match key {
            "params" => {
                for (name, v) in self.record(key, value) {
                    if let Some(n) = self.number(&format!("params.{}", name.text), v) {
                        out.params.insert(name.text.clone(), n);
                    }
                }
            }
            "metrics" | "utilities" => {
                for (name, v) in self.record(key, value) {
                    let path = format!("{key}.{}", name.text);
                    if let Some(e) = self.engine_expr(&path, v) {
                        let map = if key == "metrics" {
                            &mut out.metrics
                        } else {
                            &mut out.utilities
                        };
                        map.insert(name.text.clone(), e);
                    }
                }
            }
            "calibration" => {
                for item in self.list(key, value) {
                    if let ExprKind::Str(s) = &item.kind {
                        out.calibration.push(s.clone());
                    } else {
                        self.invalid(item.range, "a calibration set is named by a string".into());
                    }
                }
            }
            "discount_annual" => out.discount_annual = self.number(key, value),
            "remove_edges" => out.remove_edges = self.removed_edges(value),
            _ if EXPRESSIONS.contains(&key) => {
                if let Some(e) = self.engine_expr(key, value) {
                    out.expressions.insert(key.to_owned(), e);
                }
            }
            _ if OPTIONS.contains(&key) => match json(value) {
                Some(v) => {
                    out.options.insert(
                        key.to_owned(),
                        EngineOption {
                            value: v,
                            span: Some(self.s(value.range)),
                        },
                    );
                }
                None => self.invalid(
                    value.range,
                    format!(
                        "`{key}` is written with numbers, strings, booleans, records and lists"
                    ),
                ),
            },
            _ => self.not_a_field(key, at),
        }
    }

    fn not_a_field(&mut self, key: &str, at: TextRange) {
        if DEDICATED.contains(&key) {
            return self.invalid(
                at,
                format!("`{key}` has its own `.case` block; it cannot appear in `analysis`"),
            );
        }
        if UNTYPED.contains(&key) {
            return self.invalid(
                at,
                format!(
                    "`analysis` has no `{key}`: it would name engine nodes or edges by string; \
                     a matter states what it knows with `facts` and `beliefs`"
                ),
            );
        }
        let mut r = Report::new(
            C::InvalidAnalysis,
            at,
            format!("`{key}` is not an `analysis` field"),
        )
        .note(format!("fields: {}", FIELDS.join(", ")));
        if let Some(best) = nearest(key, FIELDS) {
            r = r.fix(at, best, Applicability::MaybeIncorrect);
        }
        self.out.push(r);
    }

    /// The entries of record field `key` (reported when not a record).
    fn record<'e>(
        &mut self,
        key: &str,
        value: &'e Expr,
    ) -> &'e [(crate::db::parse::tree::Name, Expr)] {
        if let ExprKind::Record(entries) = &value.kind {
            return entries;
        }
        self.invalid(
            value.range,
            format!("`{key}` is a record `{{ name: value }}`"),
        );
        &[]
    }

    /// The items of list field `key` (reported when not a list).
    fn list<'e>(&mut self, key: &str, value: &'e Expr) -> &'e [Expr] {
        if let ExprKind::List(items) = &value.kind {
            return items;
        }
        self.invalid(value.range, format!("`{key}` is a list `[a, b]`"));
        &[]
    }

    /// An exact number, optionally negated (reported otherwise).
    fn number(&mut self, path: &str, value: &Expr) -> Option<Rational> {
        let parsed = match &value.kind {
            ExprKind::Number(text) => rational::parse_decimal(text).ok(),
            ExprKind::Unary(UnaryOp::Neg, inner) => match &inner.kind {
                ExprKind::Number(text) => rational::parse_decimal(text)
                    .ok()
                    .and_then(|r| rational::neg(r).ok()),
                _ => None,
            },
            _ => None,
        };
        if parsed.is_none() {
            self.invalid(value.range, format!("`{path}` is an exact number"));
        }
        parsed
    }

    /// An engine expression, written as a string (reported otherwise).
    fn engine_expr(&mut self, path: &str, value: &Expr) -> Option<EngineExpr> {
        if let ExprKind::Str(text) = &value.kind {
            return Some(EngineExpr {
                text: text.clone(),
                span: Some(self.s(value.range)),
            });
        }
        self.invalid(
            value.range,
            format!("`{path}` is an engine expression, written as a string"),
        );
        None
    }

    /// `remove_edges`: each item an `EdgeRef`, resolved (L1002) and
    /// qualified.
    fn removed_edges(&mut self, value: &Expr) -> Vec<String> {
        let items = self.list("remove_edges", value);
        let Some(graph) = self.graph.as_ref() else {
            return Vec::new();
        };
        let mut out = Vec::new();
        for item in items {
            match &item.kind {
                ExprKind::Ref(path) => {
                    out.extend(event::edge_ref(
                        graph,
                        &path.display(),
                        item.range,
                        &mut self.out,
                    ));
                }
                _ => self.out.push(Report::new(
                    C::TypeMismatch,
                    item.range,
                    "expected an `EdgeRef` here: an edge id, unquoted (`edge` or `pack::edge`)",
                )),
            }
        }
        out
    }
}

/// An engine option's value as JSON: numbers (optionally negated),
/// strings, booleans, records and lists. `None` for anything else.
fn json(e: &Expr) -> Option<serde_json::Value> {
    Some(match &e.kind {
        ExprKind::Number(text) => serde_json::from_str(&text.replace('_', "")).ok()?,
        ExprKind::Unary(UnaryOp::Neg, inner) => match &inner.kind {
            ExprKind::Number(text) => {
                serde_json::from_str(&format!("-{}", text.replace('_', ""))).ok()?
            }
            _ => return None,
        },
        ExprKind::Str(s) => serde_json::Value::String(s.clone()),
        ExprKind::Bool(b) => serde_json::Value::Bool(*b),
        ExprKind::Record(entries) => serde_json::Value::Object(
            entries
                .iter()
                .map(|(k, v)| Some((k.text.clone(), json(v)?)))
                .collect::<Option<_>>()?,
        ),
        ExprKind::List(items) => {
            serde_json::Value::Array(items.iter().map(json).collect::<Option<_>>()?)
        }
        _ => return None,
    })
}
