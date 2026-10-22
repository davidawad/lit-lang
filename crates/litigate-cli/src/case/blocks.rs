//! `facts`, `stakes`, `beliefs`, and `analysis` blocks (SPEC §3.2, §5.6, §8.3).
use super::Cx;
use litigate_base::{Applicability, DiagnosticCode as C, Fix, Profile};
use litigate_sema::hir::{self, Money, NodeKind};
use litigate_sema::types::rational;
use litigate_syntax::ast::{self, AstNode, CaseFile, Expr, RecordExpr};
use litigate_syntax::SyntaxKind;
use serde_json::{Map, Value};
use std::collections::{BTreeMap, BTreeSet};

fn block(file: &CaseFile, kind: SyntaxKind) -> Option<ast::CaseBlock> {
    file.block(kind)
}

/// `(qualified key, value node)` entries of a record block.
fn entries(cx: &Cx<'_>, record: Option<RecordExpr>) -> Vec<(String, ast::RecordField)> {
    record
        .map(|r| r.fields())
        .unwrap_or_default()
        .into_iter()
        .filter_map(|f| Some((cx.qualify(&f.key_text()?), f)))
        .collect()
}

/// `pack::edge` when `edge` (local or qualified) is one of `node`'s edges.
fn own_edge(node: &hir::Node, pack: &str, edge: &str) -> Option<String> {
    let local = edge.rsplit("::").next().unwrap_or(edge);
    node.edges
        .iter()
        .any(|e| e.id.as_deref() == Some(local))
        .then(|| format!("{pack}::{local}"))
}

pub(super) fn facts(cx: &mut Cx<'_>, file: &CaseFile) -> BTreeMap<String, String> {
    let list = entries(
        cx,
        block(file, SyntaxKind::FactsBlock).and_then(|b| b.record()),
    );
    list.into_iter()
        .filter_map(|(node, f)| {
            let span = cx.node_span(f.syntax());
            let edge = f.value_ref().map(|r| r.text()).unwrap_or_default();
            let found = cx.node(&node).map(|(p, n)| {
                let fact =
                    matches!(n.kind, NodeKind::Chance { .. }) && n.tags.iter().any(|t| t == "fact");
                (fact, own_edge(n, &p.manifest.pack_id, &edge))
            });
            match found {
                Some((true, Some(edge))) => Some((node, edge)),
                Some((true, None)) => {
                    cx.error(
                        C::InvalidCaseFact,
                        span,
                        format!("`{edge}` is not an edge of `{node}`"),
                    );
                    None
                }
                Some((false, _)) => {
                    cx.error(
                        C::InvalidCaseFact,
                        span,
                        format!("`{node}` is not a `fact`-tagged chance node"),
                    );
                    None
                }
                None => {
                    cx.error(
                        C::InvalidCaseFact,
                        span,
                        format!("`{node}` is not a node in force on {}", cx.as_of),
                    );
                    None
                }
            }
        })
        .collect()
}

/// A signed money literal (`-2_000_000.usd`).
fn money(expr: &Expr) -> Option<Money> {
    let (negative, literal) = match expr.kind() {
        SyntaxKind::UnaryExpr => (true, expr.operands().into_iter().next()?),
        _ => (false, expr.clone()),
    };
    let token = ast::Literal::cast(literal.syntax().clone())?.token()?;
    if token.kind() != SyntaxKind::Money {
        return None;
    }
    let text = token.text().replace('_', "");
    let (amount, currency) = text.rsplit_once('.')?;
    let minor = rational::mul(
        rational::parse_decimal(amount).ok()?,
        hir::Rational::integer(100),
    )
    .ok()?;
    (minor.denominator == 1).then(|| Money {
        minor_units: if negative {
            -minor.numerator
        } else {
            minor.numerator
        },
        currency: currency.to_owned(),
    })
}

pub(super) fn stakes(cx: &mut Cx<'_>, file: &CaseFile) -> BTreeMap<String, hir::Stake> {
    let list = entries(
        cx,
        block(file, SyntaxKind::StakesBlock).and_then(|b| b.record()),
    );
    list.into_iter()
        .filter_map(|(terminal, f)| {
            let span = cx.node_span(f.syntax());
            if !cx
                .node(&terminal)
                .is_some_and(|(_, n)| matches!(n.kind, NodeKind::Terminal { .. }))
            {
                cx.error(
                    C::MissingStake,
                    span,
                    format!(
                        "`{terminal}` is not a terminal node in force on {}",
                        cx.as_of
                    ),
                );
                return None;
            }
            let value = f.value_expr()?;
            let unvalued = ast::Literal::cast(value.syntax().clone())
                .and_then(|l| l.token())
                .is_some_and(|t| t.kind() == SyntaxKind::KwUnvalued);
            if unvalued {
                if cx.profile == Profile::Strict {
                    cx.error(
                        C::MissingStake,
                        span,
                        format!("`{terminal}` is `unvalued`, which strict profile rejects"),
                    );
                }
                return Some((terminal, hir::Stake::Unvalued));
            }
            match money(&value) {
                Some(m) => Some((terminal, hir::Stake::Value(m))),
                None => {
                    cx.error(
                        C::TypeMismatch,
                        span,
                        "a stake is a money literal such as `-2_000.usd`, or `unvalued`",
                    );
                    None
                }
            }
        })
        .collect()
}

pub(super) fn beliefs(cx: &mut Cx<'_>, file: &CaseFile) -> Vec<hir::Observation> {
    let observations = block(file, SyntaxKind::BeliefsBlock)
        .map(|b| b.observations())
        .unwrap_or_default();
    observations
        .iter()
        .filter_map(|o| {
            let span = cx.node_span(o.syntax());
            let node = cx.qualify(&o.node()?.text());
            let Some((pkg, n)) = cx
                .node(&node)
                .filter(|(_, n)| matches!(n.kind, NodeKind::Chance { .. }))
            else {
                cx.error(
                    C::InvalidBelief,
                    span,
                    format!("`{node}` is not a chance node in force on {}", cx.as_of),
                );
                return None;
            };
            let pack = pkg.manifest.pack_id.clone();
            let n = n.clone();
            let outcomes = o
                .counts()
                .map(|r| r.fields())
                .unwrap_or_default()
                .iter()
                .filter_map(|f| {
                    let at = cx.node_span(f.syntax());
                    let edge = f.key_text()?;
                    let count = f
                        .value_expr()
                        .and_then(|e| ast::Literal::cast(e.syntax().clone()))
                        .and_then(|l| l.token());
                    let count = count.and_then(|t| t.text().replace('_', "").parse::<u64>().ok());
                    match (own_edge(&n, &pack, &edge), count) {
                        (Some(edge), Some(count)) => Some((edge, count)),
                        (None, _) => {
                            cx.error(
                                C::InvalidBelief,
                                at,
                                format!("`{edge}` is not an edge of `{node}`"),
                            );
                            None
                        }
                        (_, None) => {
                            cx.error(
                                C::InvalidBelief,
                                at,
                                "an observation count is a non-negative integer",
                            );
                            None
                        }
                    }
                })
                .collect();
            Some(hir::Observation { node, outcomes })
        })
        .collect()
}

/// A JSON value from an `analysis` expression; numbers exact to binary64.
fn json(cx: &mut Cx<'_>, expr: &Expr) -> Option<Value> {
    let span = cx.node_span(expr.syntax());
    match expr.kind() {
        SyntaxKind::RecordExpr => {
            let record = RecordExpr::cast(expr.syntax().clone())?;
            let mut map = Map::new();
            for f in record.fields() {
                let key = f.key_text()?;
                map.insert(key, json(cx, &f.value_expr()?)?);
            }
            Some(Value::Object(map))
        }
        SyntaxKind::ListExpr => {
            let items: Vec<Expr> = expr.syntax().children().filter_map(Expr::cast).collect();
            items
                .iter()
                .map(|e| json(cx, e))
                .collect::<Option<Vec<_>>>()
                .map(Value::Array)
        }
        SyntaxKind::UnaryExpr => match json(cx, &expr.operands().into_iter().next()?)? {
            Value::Number(n) => n
                .as_i64()
                .map(|i| Value::from(-i))
                .or_else(|| n.as_f64().map(|f| Value::from(-f))),
            _ => {
                cx.error(C::InvalidAnalysis, span, "only numbers can be negated");
                None
            }
        },
        SyntaxKind::Literal => {
            let token = ast::Literal::cast(expr.syntax().clone())?.token()?;
            let text = token.text().replace('_', "");
            match token.kind() {
                SyntaxKind::String => Some(Value::String(ast::unescape(token.text()))),
                SyntaxKind::KwTrue => Some(Value::Bool(true)),
                SyntaxKind::KwFalse => Some(Value::Bool(false)),
                SyntaxKind::KwUnset => Some(Value::Null),
                SyntaxKind::Number => text.parse::<i64>().map(Value::from).ok().or_else(|| {
                    text.parse::<f64>()
                        .ok()
                        .filter(|f| f.is_finite())
                        .map(Value::from)
                }),
                _ => {
                    cx.error(
                        C::InvalidAnalysis,
                        span,
                        format!("`{}` has no JSON value", token.text()),
                    );
                    None
                }
            }
        }
        _ => {
            cx.error(
                C::InvalidAnalysis,
                span,
                "analysis values are JSON: records, lists, strings, numbers, booleans, unset",
            );
            None
        }
    }
}

pub(super) fn analysis(cx: &mut Cx<'_>, file: &CaseFile) -> hir::Analysis {
    let fields = block(file, SyntaxKind::AnalysisBlock)
        .and_then(|b| b.record())
        .map(|r| r.fields())
        .unwrap_or_default()
        .iter()
        .filter_map(|f| Some((f.key_text()?, json(cx, &f.value_expr()?)?)))
        .collect();
    hir::Analysis { fields }
}

/// L4004: every terminal reachable from the replayed start needs a stake.
pub(super) fn stakes_cover(cx: &mut Cx<'_>, file: &CaseFile, case: &hir::Case) {
    let start = case.start.clone().or_else(|| {
        cx.primary()
            .map(|p| format!("{}::{}", p.manifest.pack_id, p.start))
    });
    let mut seen = BTreeSet::new();
    let mut stack: Vec<String> = start.into_iter().collect();
    let mut missing = Vec::new();
    while let Some(id) = stack.pop() {
        if !seen.insert(id.clone()) {
            continue;
        }
        let Some((pkg, node)) = cx.node(&id) else {
            continue;
        };
        let pack = pkg.manifest.pack_id.clone();
        // A terminal with an authored pack `payoff` already has a value.
        let valued = matches!(
            &node.kind,
            NodeKind::Terminal {
                payoff: Some(hir::Authored::Known(_)),
                ..
            }
        );
        if matches!(node.kind, NodeKind::Terminal { .. })
            && !valued
            && !case.stakes.contains_key(&id)
        {
            missing.push(id.clone());
        }
        stack.extend(node.edges.iter().map(|e| {
            if e.to.contains("::") {
                e.to.clone()
            } else {
                format!("{pack}::{}", e.to)
            }
        }));
    }
    missing.sort();
    let stakes = block(file, SyntaxKind::StakesBlock);
    for terminal in missing {
        let local = terminal.rsplit("::").next().unwrap_or(&terminal).to_owned();
        let span = stakes.as_ref().map_or_else(
            || cx.span(file.syntax().text_range()),
            |b| cx.node_span(b.syntax()),
        );
        let fix = stakes.as_ref().and_then(|b| {
            let close = b.record()?.syntax().last_token()?;
            let at = cx.span(close.text_range());
            Some(Fix {
                span: litigate_base::Span {
                    end_line: at.line,
                    end_col: at.col,
                    ..at
                },
                replacement: format!(", {local}: unvalued "),
                applicability: Applicability::HasPlaceholders,
            })
        });
        cx.diagnostics.push(litigate_base::Diagnostic {
            fixes: fix.into_iter().collect(),
            ..litigate_base::Diagnostic::error(
                C::MissingStake,
                span,
                format!("`{terminal}` is reachable from the replayed start but has no stake (a value or `unvalued`)"),
            )
        });
    }
}
