//! Expressions, references, types, and JSON records (SPEC §3.1, §3.3).
use super::tree::{Expr, ExprKind, Json, Name, PathSep, Pattern, RefPath, TypeRef};
use super::{elements, name, node_of, range, significant_range, token_of, Lower, SyntaxElement};
use crate::db::span::Report;
use crate::hir::{BinaryOp, UnaryOp};
use litigate_base::DiagnosticCode as C;
use litigate_syntax::{SyntaxKind as K, SyntaxNode};

/// True for node kinds that are expressions (including a bare `Ref`).
pub(crate) fn is_expr(kind: K) -> bool {
    matches!(
        kind,
        K::Literal
            | K::HoleExpr
            | K::CiteExpr
            | K::RefExpr
            | K::Ref
            | K::CallExpr
            | K::UnaryExpr
            | K::BinaryExpr
            | K::ParenExpr
            | K::RecordExpr
            | K::ListExpr
            | K::MatchExpr
            | K::FieldExpr
            | K::Ratio
            | K::IdList
    )
}

/// `Ident ["@" Ident] {("::" | ".") Ident} ["@" Date]` from a `Ref` node.
pub(crate) fn ref_path(node: &SyntaxNode) -> Option<RefPath> {
    let mut tokens = elements(node)
        .filter_map(SyntaxElement::into_token)
        .peekable();
    let head = name(&tokens.next().filter(|t| t.kind() == K::Ident)?);
    let mut path = RefPath {
        range: significant_range(node),
        head,
        instance: None,
        rest: Vec::new(),
        version: None,
    };
    while let Some(t) = tokens.next() {
        let next = tokens.next()?;
        match (t.kind(), next.kind()) {
            (K::At, K::Ident) if path.rest.is_empty() && path.instance.is_none() => {
                path.instance = Some(name(&next));
            }
            (K::At, K::Date) => path.version = Some(name(&next)),
            (K::Dot, K::Ident) => path.rest.push((PathSep::Dot, name(&next))),
            (K::ColonColon, K::Ident) => path.rest.push((PathSep::Colons, name(&next))),
            _ => return None,
        }
    }
    Some(path)
}

/// `Name<Args>` from a `TypeExpr` node.
pub(crate) fn type_ref(node: &SyntaxNode) -> Option<TypeRef> {
    let args = node_of(node, K::TypeArgs).map_or(Some(Vec::new()), |a| {
        super::nodes_of(&a, K::TypeExpr)
            .map(|t| type_ref(&t))
            .collect()
    })?;
    Some(TypeRef {
        name: name(&token_of(node, K::Ident)?),
        args,
        range: significant_range(node),
    })
}

fn binary_op(kind: K) -> Option<BinaryOp> {
    Some(match kind {
        K::OrOr => BinaryOp::Or,
        K::AndAnd => BinaryOp::And,
        K::EqEq => BinaryOp::Eq,
        K::NotEq => BinaryOp::Ne,
        K::Less => BinaryOp::Lt,
        K::LessEq => BinaryOp::Le,
        K::Greater => BinaryOp::Gt,
        K::GreaterEq => BinaryOp::Ge,
        K::Plus => BinaryOp::Add,
        K::Minus => BinaryOp::Sub,
        K::Star => BinaryOp::Mul,
        K::Slash => BinaryOp::Div,
        K::Percent => BinaryOp::Rem,
        _ => return None,
    })
}

/// Expression children of `node`, in order.
pub(crate) fn expr_children(node: &SyntaxNode) -> impl Iterator<Item = SyntaxNode> {
    node.children().filter(|n| is_expr(n.kind()))
}

/// A literal token as an expression.
fn literal(token: &litigate_syntax::SyntaxToken) -> Option<ExprKind> {
    let text = token.text().to_owned();
    Some(match token.kind() {
        K::Number => ExprKind::Number(text),
        K::Money => ExprKind::Money(text),
        K::Duration => ExprKind::Duration(text),
        K::String => ExprKind::Str(name(token).text),
        K::Date => ExprKind::Date(text),
        K::KwTrue => ExprKind::Bool(true),
        K::KwFalse => ExprKind::Bool(false),
        K::KwUnit => ExprKind::Unit,
        _ => return None,
    })
}

/// Normalized citation text (whitespace runs collapsed, SPEC §2).
pub(crate) fn cite_text(token: &litigate_syntax::SyntaxToken) -> String {
    token
        .text()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

impl Lower {
    /// A `cite` expression. A cite hole `cite ?("text")` is a hole whose
    /// reason is the citation text (SPEC §2, §8.1).
    fn cite(&mut self, node: &SyntaxNode) -> Option<Expr> {
        let Some(text) = token_of(node, K::CiteText) else {
            return self.expr(&node_of(node, K::HoleExpr)?);
        };
        Some(Expr {
            kind: ExprKind::Cite(cite_text(&text)),
            range: range(text.text_range()),
        })
    }

    /// Lowers an expression node; `None` when the node is incomplete (the
    /// parser already reported it).
    pub(crate) fn expr(&mut self, node: &SyntaxNode) -> Option<Expr> {
        let kind = match node.kind() {
            K::Literal => literal(&elements(node).find_map(SyntaxElement::into_token)?)?,
            K::HoleExpr => ExprKind::Hole(
                node_of(node, K::Literal)
                    .and_then(|l| token_of(&l, K::String))
                    .map(|t| name(&t).text),
            ),
            K::CiteExpr => return self.cite(node),
            K::Ref => ExprKind::Ref(ref_path(node)?),
            K::RefExpr => ExprKind::Ref(ref_path(&node_of(node, K::Ref)?)?),
            K::CallExpr => {
                let callee = ref_path(&node_of(node, K::Ref)?)?;
                let args = node_of(node, K::ArgList)?;
                let args = expr_children(&args)
                    .map(|a| self.expr(&a))
                    .collect::<Option<_>>()?;
                ExprKind::Call(callee, args)
            }
            K::UnaryExpr => {
                let op = match elements(node).next()?.kind() {
                    K::Bang => UnaryOp::Not,
                    K::Minus => UnaryOp::Neg,
                    _ => return None,
                };
                ExprKind::Unary(op, Box::new(self.expr(&expr_children(node).next()?)?))
            }
            K::BinaryExpr => {
                let mut operands = expr_children(node);
                let lhs = self.expr(&operands.next()?)?;
                let rhs = self.expr(&operands.next()?)?;
                let op = elements(node)
                    .filter_map(SyntaxElement::into_token)
                    .find_map(|t| binary_op(t.kind()))?;
                ExprKind::Binary(op, Box::new(lhs), Box::new(rhs))
            }
            K::Ratio => {
                let mut numbers = elements(node)
                    .filter_map(SyntaxElement::into_token)
                    .filter(|t| t.kind() == K::Number)
                    .map(|t| Expr {
                        kind: ExprKind::Number(t.text().to_owned()),
                        range: range(t.text_range()),
                    });
                let (n, d) = (numbers.next()?, numbers.next()?);
                ExprKind::Binary(BinaryOp::Div, Box::new(n), Box::new(d))
            }
            K::ParenExpr => ExprKind::Paren(Box::new(self.expr(&expr_children(node).next()?)?)),
            K::RecordExpr => ExprKind::Record(self.record(node)?),
            K::ListExpr | K::IdList => ExprKind::List(
                expr_children(node)
                    .map(|e| self.expr(&e))
                    .collect::<Option<_>>()?,
            ),
            K::FieldExpr => {
                let base = self.expr(&expr_children(node).next()?)?;
                let field = elements(node)
                    .filter_map(SyntaxElement::into_token)
                    .filter(|t| t.kind() == K::Ident)
                    .last()?;
                ExprKind::Field(Box::new(base), name(&field))
            }
            K::MatchExpr => {
                let scrutinee = self.expr(&expr_children(node).next()?)?;
                let arms = super::nodes_of(node, K::MatchArm)
                    .map(|arm| self.match_arm(&arm))
                    .collect::<Option<_>>()?;
                ExprKind::Match(Box::new(scrutinee), arms)
            }
            _ => return None,
        };
        Some(Expr {
            kind,
            range: significant_range(node),
        })
    }

    fn match_arm(&mut self, arm: &SyntaxNode) -> Option<(Pattern, Expr)> {
        let pattern = match token_of(arm, K::Underscore) {
            Some(u) => Pattern::Wildcard(range(u.text_range())),
            None => Pattern::Ref(ref_path(&node_of(arm, K::Ref)?)?),
        };
        let value = super::nodes_after(arm, K::FatArrow).find(|n| is_expr(n.kind()))?;
        Some((pattern, self.expr(&value)?))
    }

    /// `{ key: value, ... }` entries (keys may be contextual keywords).
    pub(crate) fn record(&mut self, node: &SyntaxNode) -> Option<Vec<(Name, Expr)>> {
        super::nodes_of(node, K::RecordField)
            .map(|f| {
                let key = elements(&f).find_map(SyntaxElement::into_token)?;
                let value = expr_children(&f).next()?;
                Some((name(&key), self.expr(&value)?))
            })
            .collect()
    }

    /// A `patch` record as JSON (SPEC §3.1 `JsonRecord`).
    pub(crate) fn json(&mut self, node: &SyntaxNode) -> Option<Json> {
        let bad = |lower: &mut Self| {
            lower.reports.push(Report::new(
                C::UnexpectedToken,
                significant_range(node),
                "expected a JSON value (number, string, `true`, `false`, `unset`, record, or list)",
            ));
            None
        };
        match node.kind() {
            K::RecordExpr => super::nodes_of(node, K::RecordField)
                .map(|f| {
                    let key = elements(&f).find_map(SyntaxElement::into_token)?;
                    Some((name(&key), self.json(&expr_children(&f).next()?)?))
                })
                .collect::<Option<_>>()
                .map(Json::Object),
            K::ListExpr => expr_children(node)
                .map(|e| self.json(&e))
                .collect::<Option<_>>()
                .map(Json::Array),
            K::UnaryExpr => match self.expr(node).map(|e| e.kind) {
                Some(ExprKind::Unary(UnaryOp::Neg, inner)) => match inner.kind {
                    ExprKind::Number(n) => Some(Json::Number(format!("-{n}"))),
                    _ => bad(self),
                },
                _ => bad(self),
            },
            K::Literal => {
                let token = elements(node).find_map(SyntaxElement::into_token)?;
                match token.kind() {
                    K::Number => Some(Json::Number(token.text().to_owned())),
                    K::String => Some(Json::Str(name(&token).text)),
                    K::KwTrue => Some(Json::Bool(true)),
                    K::KwFalse => Some(Json::Bool(false)),
                    K::KwUnset => Some(Json::Unset),
                    _ => bad(self),
                }
            }
            _ => bad(self),
        }
    }
}
