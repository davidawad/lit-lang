//! `.lit` text to [`ItemTree`]: parse with `litigate-syntax` (SPEC §2-§3),
//! then lower its lossless CST into sema's syntax-independent item tree.
//!
//! Lowering is total over recovered trees: a declaration missing a required
//! part (already reported as L0xxx by the parser) is dropped, so later
//! queries only see well-formed items while the LSP keeps working.
mod expr;
mod fields;
mod items;
pub mod tree;

pub(crate) use expr::{is_expr, ref_path};

use crate::db::span::{Report, TextRange};
use litigate_syntax::{SyntaxKind as K, SyntaxNode, SyntaxToken};
use tree::{ItemTree, Name};

/// A CST node or token.
pub(crate) type SyntaxElement = rowan::NodeOrToken<SyntaxNode, SyntaxToken>;

/// Parse output: the tree and every syntax report.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Parsed {
    /// Item tree (recovered on error).
    pub tree: ItemTree,
    /// L0xxx reports from the parser (and any shape reports from lowering).
    pub reports: Vec<Report>,
}

/// Parses one `.lit` source.
#[must_use]
pub fn parse(src: &str) -> Parsed {
    lower(&litigate_syntax::parse_lit(src))
}

/// Lowers a parsed `.lit` file: its syntax errors plus the item tree.
#[must_use]
pub fn lower(parse: &litigate_syntax::Parse) -> Parsed {
    let reports: Vec<Report> = parse.errors().iter().map(syntax_report).collect();
    let mut cx = Lower;
    let tree = parse
        .lit_file()
        .map(|file| cx.file(file.syntax()))
        .unwrap_or_default();
    Parsed { tree, reports }
}

pub(crate) fn syntax_report(e: &litigate_syntax::SyntaxError) -> Report {
    let mut report = Report::new(e.code, range(e.range), e.message.clone());
    for note in &e.notes {
        report = report.note(note.clone());
    }
    for fix in &e.fixes {
        report = report.fix(range(fix.range), fix.replacement.clone(), fix.applicability);
    }
    report
}

/// Converts a rowan range.
pub(crate) fn range(r: litigate_syntax::TextRange) -> TextRange {
    TextRange::new(u32::from(r.start()), u32::from(r.end()))
}

/// Lowering from the syntax tree to the item tree. It is total: every shape
/// the grammar accepts has an item-tree form (`patch` and analysis values
/// are built only from JSON shapes), so lowering reports nothing.
pub(crate) struct Lower;

pub(crate) fn is_trivia(kind: K) -> bool {
    matches!(kind, K::Whitespace | K::Comment | K::DocComment)
}

/// Significant direct children (nodes and non-trivia tokens).
pub(crate) fn elements(node: &SyntaxNode) -> impl Iterator<Item = SyntaxElement> {
    node.children_with_tokens().filter(|e| !is_trivia(e.kind()))
}

/// Direct child nodes of `kind`.
pub(crate) fn nodes_of(node: &SyntaxNode, kind: K) -> impl Iterator<Item = SyntaxNode> {
    node.children().filter(move |n| n.kind() == kind)
}

/// First direct child node of `kind`.
pub(crate) fn node_of(node: &SyntaxNode, kind: K) -> Option<SyntaxNode> {
    nodes_of(node, kind).next()
}

/// First direct token of `kind`.
pub(crate) fn token_of(node: &SyntaxNode, kind: K) -> Option<SyntaxToken> {
    node.children_with_tokens()
        .filter_map(SyntaxElement::into_token)
        .find(|t| t.kind() == kind)
}

/// Direct child nodes after the first direct token of kind `after`.
pub(crate) fn nodes_after(node: &SyntaxNode, after: K) -> impl Iterator<Item = SyntaxNode> {
    node.children_with_tokens()
        .skip_while(move |e| e.kind() != after)
        .filter_map(SyntaxElement::into_node)
}

/// A token as a [`Name`]; string tokens are unescaped.
pub(crate) fn name(token: &SyntaxToken) -> Name {
    let text = if token.kind() == K::String {
        litigate_syntax::ast::unescape(token.text())
    } else {
        token.text().to_owned()
    };
    Name {
        text,
        range: range(token.text_range()),
    }
}

/// The node's range without leading doc comments or trailing trivia.
pub(crate) fn significant_range(node: &SyntaxNode) -> TextRange {
    let significant = node
        .descendants_with_tokens()
        .filter_map(SyntaxElement::into_token)
        .filter(|t| !is_trivia(t.kind()))
        .map(|t| t.text_range())
        .reduce(litigate_syntax::TextRange::cover);
    range(significant.unwrap_or(node.text_range()))
}

/// The first significant direct token of `node`: the keyword of a field or
/// declaration, the key of a record field, the `_` of a wildcard arm, or the
/// name of a type. The grammar starts every such node with that token; a
/// node without one (never built) gives an empty name at its start.
pub(crate) fn head(node: &SyntaxNode) -> Name {
    let start = u32::from(node.text_range().start());
    let empty = Name {
        text: String::new(),
        range: TextRange::new(start, start),
    };
    elements(node)
        .find_map(SyntaxElement::into_token)
        .map_or(empty, |t| name(&t))
}

/// The first direct `Ident` token (a declaration's name).
pub(crate) fn ident(node: &SyntaxNode) -> Option<Name> {
    token_of(node, K::Ident).map(|t| name(&t))
}

/// The direct `String` token (a declaration's label).
pub(crate) fn label(node: &SyntaxNode) -> Option<Name> {
    token_of(node, K::String).map(|t| name(&t))
}

/// Joined `///` lines leading a declaration.
pub(crate) fn docs(node: &SyntaxNode) -> Option<String> {
    let lines: Vec<String> = node
        .children_with_tokens()
        .filter_map(SyntaxElement::into_token)
        .take_while(|t| is_trivia(t.kind()))
        .filter(|t| t.kind() == K::DocComment)
        .map(|t| {
            let text = t.text().trim_start_matches("///");
            text.strip_prefix(' ').unwrap_or(text).trim_end().to_owned()
        })
        .collect();
    (!lines.is_empty()).then(|| lines.join("\n"))
}

#[cfg(test)]
mod tests;
