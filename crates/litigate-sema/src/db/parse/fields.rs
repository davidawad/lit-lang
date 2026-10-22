//! Keyed fields and header clauses as [`Field`]s: `key value...`, where each
//! value is an [`Atom`]. The parser has already restricted keys to each
//! context's closed table (SPEC §3.1, §3.4); lowering only reshapes.
use super::expr::is_expr;
use super::tree::{Atom, Expr, ExprKind, Field, Name};
use super::{elements, name, range, significant_range, token_of, Lower, SyntaxElement};
use crate::db::span::TextRange;
use litigate_syntax::{SyntaxKind as K, SyntaxNode};

/// Node kinds that are header clauses of a declaration.
fn is_clause(kind: K) -> bool {
    matches!(
        kind,
        K::Field | K::VersionClause | K::Relation | K::CiteExpr
    )
}

impl Lower {
    /// Header clauses that are direct children of `node`, in source order.
    pub(crate) fn clauses(&mut self, node: &SyntaxNode) -> Vec<Field> {
        self.clauses_in(node.children())
    }

    /// Header clauses among `nodes`, in order.
    pub(crate) fn clauses_in(&mut self, nodes: impl Iterator<Item = SyntaxNode>) -> Vec<Field> {
        nodes
            .filter(|n| is_clause(n.kind()))
            .flat_map(|n| self.clause(&n))
            .collect()
    }

    /// One clause node as fields (a `VersionClause` may hold both
    /// `effective` and `known_at`).
    pub(crate) fn clause(&mut self, node: &SyntaxNode) -> Vec<Field> {
        match node.kind() {
            K::CiteExpr => self.cite_field(node).into_iter().collect(),
            K::VersionClause => self.version_clause(node),
            K::Field | K::Relation => self.field(node).into_iter().collect(),
            _ => Vec::new(),
        }
    }

    fn cite_field(&mut self, node: &SyntaxNode) -> Option<Field> {
        let key = name(&token_of(node, K::KwCite)?);
        let value = self.expr(node)?;
        Some(Field {
            range: key.range.cover(value.range),
            values: vec![(Atom::Expr(value.clone()), value.range)],
            key,
        })
    }

    fn version_clause(&mut self, node: &SyntaxNode) -> Vec<Field> {
        let mut out: Vec<Field> = Vec::new();
        for element in elements(node) {
            match element {
                SyntaxElement::Token(t) if t.kind().is_keyword() => out.push(Field {
                    key: name(&t),
                    values: Vec::new(),
                    range: range(t.text_range()),
                }),
                SyntaxElement::Node(n) => {
                    let atom = match n.kind() {
                        K::Interval => interval(&n).map(|(from, to)| Atom::Interval(from, to)),
                        _ => self.expr(&n).map(Atom::Expr),
                    };
                    if let (Some(atom), Some(field)) = (atom, out.last_mut()) {
                        let r = significant_range(&n);
                        field.values.push((atom, r));
                        field.range = field.range.cover(r);
                    }
                }
                SyntaxElement::Token(_) => {}
            }
        }
        out.retain(|f| !f.values.is_empty());
        out
    }

    /// A `Field`/`Relation`/`PatchDecl` node: its keyword, then one atom per
    /// value child.
    pub(crate) fn field(&mut self, node: &SyntaxNode) -> Option<Field> {
        let key = elements(node).find_map(SyntaxElement::into_token)?;
        let key = name(&key);
        let mut values = Vec::new();
        for child in node.children() {
            let r = significant_range(&child);
            let atom = match child.kind() {
                K::FieldList => Some(Atom::Fields(self.field_list(&child))),
                K::Interval => interval(&child).map(|(from, to)| Atom::Interval(from, to)),
                K::RecordExpr if node.kind() == K::PatchDecl => self.json(&child).map(Atom::Json),
                k if is_expr(k) => self.expr(&child).map(Atom::Expr),
                _ => None,
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

    /// Every field (and `patch`) of a braced `FieldList`.
    pub(crate) fn field_list(&mut self, node: &SyntaxNode) -> Vec<Field> {
        node.children()
            .filter(|n| matches!(n.kind(), K::Field | K::PatchDecl))
            .filter_map(|n| self.field(&n))
            .collect()
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
