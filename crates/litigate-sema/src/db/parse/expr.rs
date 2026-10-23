//! Expressions, references, types, and JSON records (SPEC §3.1, §3.3).
use super::tree::{Expr, ExprKind, Name, PathSep, Pattern, RefPath, TypeRef};
use super::{
    elements, head, is_trivia, name, node_of, range, significant_range, token_of, Lower,
    SyntaxElement,
};
use crate::hir::{BinaryOp, UnaryOp};
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

/// `Ident ["@" Ident] {("::" | ".") Ident} ["@" Date]` from a `Ref` node
/// (or the `RefExpr` wrapping one). The grammar's `reference` rule is the
/// only producer of `Ref` nodes, so its tokens are always in this shape.
pub(crate) fn path_of(node: &SyntaxNode) -> RefPath {
    let range = significant_range(node);
    let mut path = RefPath {
        head: Name {
            text: String::new(),
            range,
        },
        instance: None,
        rest: Vec::new(),
        version: None,
        range,
    };
    let mut separator = None;
    let tokens = node
        .descendants_with_tokens()
        .filter_map(SyntaxElement::into_token)
        .filter(|t| !is_trivia(t.kind()));
    for t in tokens {
        match (separator.take(), t.kind()) {
            (None, K::Ident) => path.head = name(&t),
            (Some(K::At), K::Ident) => path.instance = Some(name(&t)),
            (Some(K::Dot), K::Ident) => path.rest.push((PathSep::Dot, name(&t))),
            (Some(_), K::Ident) => path.rest.push((PathSep::Colons, name(&t))),
            (_, K::Date) => path.version = Some(name(&t)),
            (_, kind) => separator = Some(kind),
        }
    }
    path
}

/// [`path_of`] for callers chaining over optional nodes.
#[allow(clippy::unnecessary_wraps)]
pub(crate) fn ref_path(node: &SyntaxNode) -> Option<RefPath> {
    Some(path_of(node))
}

/// `Name<Args>` from a `TypeExpr` node (the grammar's `type_expr` starts
/// every one with its name).
pub(crate) fn type_ref(node: &SyntaxNode) -> TypeRef {
    TypeRef {
        name: head(node),
        args: node_of(node, K::TypeArgs).map_or_else(Vec::new, |a| {
            super::nodes_of(&a, K::TypeExpr)
                .map(|t| type_ref(&t))
                .collect()
        }),
        range: significant_range(node),
    }
}

const BINARY_OPS: [(K, BinaryOp); 13] = [
    (K::OrOr, BinaryOp::Or),
    (K::AndAnd, BinaryOp::And),
    (K::EqEq, BinaryOp::Eq),
    (K::NotEq, BinaryOp::Ne),
    (K::Less, BinaryOp::Lt),
    (K::LessEq, BinaryOp::Le),
    (K::Greater, BinaryOp::Gt),
    (K::GreaterEq, BinaryOp::Ge),
    (K::Plus, BinaryOp::Add),
    (K::Minus, BinaryOp::Sub),
    (K::Star, BinaryOp::Mul),
    (K::Slash, BinaryOp::Div),
    (K::Percent, BinaryOp::Rem),
];

/// Builds an expression literal from its token text.
type MakeExpr = fn(&str) -> ExprKind;

/// Expression literals by token kind (`unvalued` is a literal only in
/// stakes).
const LITERALS: [(K, MakeExpr); 8] = [
    (K::Number, |t| ExprKind::Number(t.to_owned())),
    (K::Money, |t| ExprKind::Money(t.to_owned())),
    (K::Duration, |t| ExprKind::Duration(t.to_owned())),
    (K::String, |t| {
        ExprKind::Str(litigate_syntax::ast::unescape(t))
    }),
    (K::Date, |t| ExprKind::Date(t.to_owned())),
    (K::KwTrue, |_| ExprKind::Bool(true)),
    (K::KwFalse, |_| ExprKind::Bool(false)),
    (K::KwUnit, |_| ExprKind::Unit),
];

/// Expression children of `node`, in order.
pub(crate) fn expr_children(node: &SyntaxNode) -> impl Iterator<Item = SyntaxNode> {
    node.children().filter(|n| is_expr(n.kind()))
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
        match token_of(node, K::CiteText) {
            Some(text) => Some(Expr {
                kind: ExprKind::Cite(cite_text(&text)),
                range: range(text.text_range()),
            }),
            None => self.expr(&node_of(node, K::HoleExpr)?),
        }
    }

    /// Lowers an expression node; `None` when the node is incomplete (the
    /// parser already reported it).
    pub(crate) fn expr(&mut self, node: &SyntaxNode) -> Option<Expr> {
        let kind = match node.kind() {
            K::HoleExpr => Some(ExprKind::Hole(
                node_of(node, K::Literal)
                    .and_then(|l| token_of(&l, K::String))
                    .map(|t| name(&t).text),
            )),
            K::CiteExpr => return self.cite(node),
            K::Ref | K::RefExpr => Some(ExprKind::Ref(path_of(node))),
            K::CallExpr => self.call(node),
            K::UnaryExpr => self.unary(node),
            K::BinaryExpr => self.binary(node),
            K::Ratio => ratio(node),
            K::ParenExpr => self
                .first_operand(node)
                .map(|e| ExprKind::Paren(Box::new(e))),
            K::RecordExpr => self.record(node).map(ExprKind::Record),
            K::ListExpr | K::IdList => self.exprs(expr_children(node)).map(ExprKind::List),
            K::FieldExpr => self.field_expr(node),
            K::MatchExpr => self.match_expr(node),
            // A `Literal`; any other node is not an expression.
            _ => elements(node)
                .find_map(SyntaxElement::into_token)
                .filter(|_| node.kind() == K::Literal)
                .and_then(|t| {
                    LITERALS
                        .iter()
                        .find(|(k, _)| *k == t.kind())
                        .map(|(_, make)| make(t.text()))
                }),
        }?;
        Some(Expr {
            kind,
            range: significant_range(node),
        })
    }

    /// Every node of `nodes` lowered, or `None` if one is incomplete.
    fn exprs(&mut self, nodes: impl Iterator<Item = SyntaxNode>) -> Option<Vec<Expr>> {
        nodes.map(|n| self.expr(&n)).collect()
    }

    /// The first expression child of `node`, lowered.
    pub(crate) fn first_operand(&mut self, node: &SyntaxNode) -> Option<Expr> {
        self.expr(&expr_children(node).next()?)
    }

    /// `callee(args)`: `ref_or_call` always nests the callee `Ref` and the
    /// `ArgList`, so only an incomplete argument drops the call.
    fn call(&mut self, node: &SyntaxNode) -> Option<ExprKind> {
        let args = node_of(node, K::ArgList).and_then(|a| self.exprs(expr_children(&a)));
        node_of(node, K::Ref)
            .zip(args)
            .map(|(callee, args)| ExprKind::Call(path_of(&callee), args))
    }

    /// `!x` or `-x` (the only unary operators).
    fn unary(&mut self, node: &SyntaxNode) -> Option<ExprKind> {
        let op = if token_of(node, K::Bang).is_some() {
            UnaryOp::Not
        } else {
            UnaryOp::Neg
        };
        Some(ExprKind::Unary(op, Box::new(self.first_operand(node)?)))
    }

    /// `lhs op rhs`; the parser builds the node once `lhs` and `op` are
    /// read, so a missing right operand is the incomplete case.
    fn binary(&mut self, node: &SyntaxNode) -> Option<ExprKind> {
        let mut operands = expr_children(node);
        let op = elements(node)
            .filter_map(SyntaxElement::into_token)
            .find_map(|t| BINARY_OPS.iter().find(|(k, _)| *k == t.kind()));
        let (Some(lhs), Some(rhs), Some((_, op))) = (operands.next(), operands.next(), op) else {
            return None;
        };
        let lhs = self.expr(&lhs)?;
        Some(ExprKind::Binary(
            *op,
            Box::new(lhs),
            Box::new(self.expr(&rhs)?),
        ))
    }

    /// `base.field` (`postfix` adds the field name with the dot).
    fn field_expr(&mut self, node: &SyntaxNode) -> Option<ExprKind> {
        let base = self.first_operand(node)?;
        let field = elements(node)
            .filter_map(SyntaxElement::into_token)
            .filter(|t| t.kind() == K::Ident)
            .last();
        field.map(|f| ExprKind::Field(Box::new(base), name(&f)))
    }

    /// `match scrutinee { pattern => value ... }`.
    fn match_expr(&mut self, node: &SyntaxNode) -> Option<ExprKind> {
        let scrutinee = self.first_operand(node)?;
        let arms = super::nodes_of(node, K::MatchArm)
            .map(|arm| self.match_arm(&arm))
            .collect::<Option<_>>()?;
        Some(ExprKind::Match(Box::new(scrutinee), arms))
    }

    /// `(Ref | "_") "=>" Expr`.
    fn match_arm(&mut self, arm: &SyntaxNode) -> Option<(Pattern, Expr)> {
        let pattern = node_of(arm, K::Ref).map_or_else(
            || Pattern::Wildcard(head(arm).range),
            |r| Pattern::Ref(path_of(&r)),
        );
        let value = super::nodes_after(arm, K::FatArrow).find(|n| is_expr(n.kind()))?;
        Some((pattern, self.expr(&value)?))
    }

    /// `{ key: value, ... }` entries (keys may be contextual keywords).
    pub(crate) fn record(&mut self, node: &SyntaxNode) -> Option<Vec<(Name, Expr)>> {
        super::nodes_of(node, K::RecordField)
            .map(|f| Some((head(&f), self.expr(&expr_children(&f).next()?)?)))
            .collect()
    }
}

/// `n / d` (`prob_value` builds the node at the slash; the denominator may
/// be missing).
fn ratio(node: &SyntaxNode) -> Option<ExprKind> {
    let mut numbers = elements(node)
        .filter_map(SyntaxElement::into_token)
        .filter(|t| t.kind() == K::Number)
        .map(|t| Expr {
            kind: ExprKind::Number(t.text().to_owned()),
            range: range(t.text_range()),
        });
    let (Some(n), Some(d)) = (numbers.next(), numbers.next()) else {
        return None;
    };
    Some(ExprKind::Binary(BinaryOp::Div, Box::new(n), Box::new(d)))
}
