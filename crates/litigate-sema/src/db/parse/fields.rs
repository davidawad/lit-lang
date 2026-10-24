//! Keyed fields and header clauses as [`Field`]s: `key value...`, where each
//! value is an [`Atom`]. The parser has already restricted keys to each
//! context's closed table (SPEC §3.1, §3.4); lowering only reshapes.
use super::tree::{Atom, Expr, ExprKind, Field, Name};
use super::{elements, head, name, significant_range, Lower, SyntaxElement};
use crate::db::span::TextRange;
use litigate_syntax::{SyntaxKind as K, SyntaxNode, SyntaxToken};

impl Lower {
    /// Header clauses that are direct children of `node`, in source order.
    pub(crate) fn clauses(&mut self, node: &SyntaxNode) -> Vec<Field> {
        self.clauses_in(node.children())
    }

    /// Header clauses among `nodes`, in order.
    pub(crate) fn clauses_in(&mut self, nodes: impl Iterator<Item = SyntaxNode>) -> Vec<Field> {
        nodes.filter_map(|n| self.clause(&n)).collect()
    }

    /// One clause node as a field; `None` for other nodes and for a version
    /// clause missing its value.
    fn clause(&mut self, node: &SyntaxNode) -> Option<Field> {
        match node.kind() {
            K::CiteExpr => self.cite_field(node),
            K::VersionClause => self.field(node).filter(|f| !f.values.is_empty()),
            K::Field | K::Relation => self.field(node),
            _ => None,
        }
    }

    fn cite_field(&mut self, node: &SyntaxNode) -> Option<Field> {
        let key = head(node);
        let value = self.expr(node)?;
        Some(Field {
            range: key.range.cover(value.range),
            values: vec![(Atom::Expr(value.clone()), value.range)],
            key,
        })
    }

    /// A `Field`/`Relation`/`VersionClause`/`PatchDecl` node: its keyword,
    /// then one atom per value child.
    pub(crate) fn field(&mut self, node: &SyntaxNode) -> Option<Field> {
        let key = head(node);
        let mut values = Vec::new();
        for child in node.children() {
            let r = significant_range(&child);
            let atom = match child.kind() {
                K::FieldList => Some(Atom::Fields(self.field_list(&child))),
                K::Interval => interval(&child).map(|(from, to)| Atom::Interval(from, to)),
                _ => self.expr(&child).map(Atom::Expr),
            };
            values.push((atom?, r));
        }
        if node.kind() == K::PatchDecl {
            let edge = super::ident(node)?;
            let r = edge.range;
            values.insert(0, (Atom::Expr(ref_expr(edge)), r));
        }
        let range = values
            .iter()
            .fold(key.range, |acc, (_, r): &(Atom, TextRange)| acc.cover(*r));
        Some(Field { key, values, range })
    }

    /// Every field (and `patch`, and guard operation) of a braced
    /// `FieldList`.
    pub(crate) fn field_list(&mut self, node: &SyntaxNode) -> Vec<Field> {
        node.children()
            .filter_map(|n| match n.kind() {
                K::Field | K::PatchDecl => self.field(&n),
                K::GuardOp => self.guard_op(&n),
                _ => None,
            })
            .collect()
    }

    /// A `GuardOp` (SPEC §3.6) as a field keyed by its operator words
    /// (`when and`, `when or`, `supersedes when`, which no other key can
    /// spell): its condition, then its `by`, version and `cite` clauses as
    /// nested fields. `None` without a condition (a parse error).
    fn guard_op(&mut self, node: &SyntaxNode) -> Option<Field> {
        let words: Vec<SyntaxToken> = elements(node)
            .map_while(rowan::NodeOrToken::into_token)
            .collect();
        let key = Name {
            text: words
                .iter()
                .map(SyntaxToken::text)
                .collect::<Vec<_>>()
                .join(" "),
            range: words
                .iter()
                .map(|t| super::range(t.text_range()))
                .reduce(TextRange::cover)?,
        };
        let condition = node
            .children()
            .find(|c| c.kind() != K::CiteExpr && super::is_expr(c.kind()))?;
        let condition = self.expr(&condition)?;
        let clauses = self.clauses(node);
        let mut values = vec![(Atom::Expr(condition.clone()), condition.range)];
        if let Some(r) = clauses.iter().map(|f| f.range).reduce(TextRange::cover) {
            values.push((Atom::Fields(clauses), r));
        }
        let range = values.iter().fold(key.range, |acc, (_, r)| acc.cover(*r));
        Some(Field { key, values, range })
    }
}

/// A plain identifier as a reference expression.
fn ref_expr(n: Name) -> Expr {
    let range = n.range;
    Expr {
        kind: ExprKind::Ref(super::tree::RefPath {
            head: n,
            instance: None,
            rest: Vec::new(),
            version: None,
            range,
        }),
        range,
    }
}

/// `[from, to)` with `None` for `..`.
pub(crate) fn interval(node: &SyntaxNode) -> Option<(Name, Option<Name>)> {
    let mut dates = elements(node)
        .filter_map(SyntaxElement::into_token)
        .filter(|t| matches!(t.kind(), K::Date | K::DotDot));
    let from = dates.next().filter(|t| t.kind() == K::Date)?;
    let to = dates.next()?;
    Some((name(&from), (to.kind() == K::Date).then(|| name(&to))))
}
