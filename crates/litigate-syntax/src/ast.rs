//! Typed views over lossless Rowan syntax nodes.
//!
//! The set of wrappers and the kinds each one casts from follow SPEC §3;
//! W1 added accessors and wrappers for the additive node kinds (`Param`,
//! `ImportList`, `FieldExpr`, `Ratio`) and for nodes that previously had
//! no view. Accessors never panic: absent parts (in a recovered tree)
//! are `None` or empty.
mod case;
mod decls;
mod expr;
mod items;

pub use case::VersionText;
pub use rowan::ast::AstNode;

use crate::syntax_kind::{LitLanguage, SyntaxKind};
use rowan::{NodeOrToken, SyntaxNode};

type Node = rowan::SyntaxNode<LitLanguage>;
type Token = rowan::SyntaxToken<LitLanguage>;

macro_rules! ast_node {
    ($name:ident, $($kind:ident)|+) => {
        #[doc = concat!("Typed view of a ", $("`", stringify!($kind), "` ",)+ "syntax node.")]
        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        pub struct $name {
            syntax: Node,
        }
        impl AstNode for $name {
            type Language = LitLanguage;
            fn can_cast(kind: SyntaxKind) -> bool {
                matches!(kind, $(SyntaxKind::$kind)|+)
            }
            fn cast(syntax: Node) -> Option<Self> {
                Self::can_cast(syntax.kind()).then_some(Self { syntax })
            }
            fn syntax(&self) -> &Node {
                &self.syntax
            }
        }
        impl $name {
            /// Returns this wrapper's Rowan node.
            pub fn syntax(&self) -> &Node {
                &self.syntax
            }

            /// The first identifier token directly under this node (its
            /// declared name, for declarations).
            #[must_use]
            pub fn name(&self) -> Option<rowan::SyntaxToken<LitLanguage>> {
                token(&self.syntax, SyntaxKind::Ident)
            }

            /// Leading `///` doc comment lines joined with `\n` (prefix and
            /// one optional space removed), if any.
            #[must_use]
            pub fn docs(&self) -> Option<String> {
                docs(&self.syntax)
            }
        }
    };
}

ast_node!(LitFile, SourceFile);
ast_node!(CaseFile, CaseFile);
ast_node!(ModuleDecl, ModuleDecl);
ast_node!(ImportDecl, ImportDecl);
ast_node!(PackageDecl, PackageDecl);
ast_node!(StartDecl, StartDecl);
ast_node!(GroupDecl, GroupDecl);
ast_node!(TypeDecl, TypeDecl);
ast_node!(EnumDecl, EnumDecl);
ast_node!(EventDecl, EventDecl);
ast_node!(
    NodeDecl,
    StateDecl | DecisionDecl | ChanceDecl | TerminalDecl
);
ast_node!(FieldList, FieldList);
ast_node!(Field, Field);
ast_node!(EdgeBlock, EdgeBlock);
ast_node!(EdgeDecl, EdgeDecl);
ast_node!(LinkDecl, LinkDecl);
ast_node!(InstanceDecl, InstanceDecl);
ast_node!(PatchDecl, PatchDecl);
ast_node!(RuleDecl, RuleDecl);
ast_node!(DeadlineDecl, DeadlineDecl);
ast_node!(CountingDecl, CountingDecl);
ast_node!(CalendarDecl, CalendarDecl);
ast_node!(HolidayDecl, HolidayDecl);
ast_node!(TransitionDecl, TransitionDecl);
ast_node!(TransitionArm, TransitionArm);
ast_node!(AuthorityDecl, AuthorityDecl);
ast_node!(DefaultDecl, DefaultDecl);
ast_node!(ExceptionDecl, ExceptionDecl);
ast_node!(TestDecl, TestDecl);
ast_node!(GrantDecl, GrantDecl);
ast_node!(RecharacterizeDecl, RecharacterizeDecl);
ast_node!(InvalidateDecl, InvalidateDecl);
ast_node!(CalibrationDecl, CalibrationDecl);
ast_node!(VersionClause, VersionClause);
ast_node!(Relation, Relation);
ast_node!(Ref, Ref);
ast_node!(TypeExpr, TypeExpr);
ast_node!(
    Expr,
    Literal
        | HoleExpr
        | CiteExpr
        | RefExpr
        | CallExpr
        | UnaryExpr
        | BinaryExpr
        | ParenExpr
        | RecordExpr
        | ListExpr
        | MatchExpr
        | FieldExpr
        | Ratio
);
ast_node!(UsesDecl, UsesDecl);
ast_node!(PerspectiveDecl, PerspectiveDecl);
ast_node!(PartyDecl, PartyDecl);
ast_node!(CaseEvent, CaseEvent);
ast_node!(
    CaseBlock,
    FactsBlock | StakesBlock | BeliefsBlock | AnalysisBlock
);
// Additive W1 wrappers.
ast_node!(ImportList, ImportList);
ast_node!(Param, Param);
ast_node!(IdList, IdList);
ast_node!(Interval, Interval);
ast_node!(Cite, CiteExpr);
ast_node!(Literal, Literal);
ast_node!(Hole, HoleExpr);
ast_node!(Ratio, Ratio);
ast_node!(RecordExpr, RecordExpr);
ast_node!(RecordField, RecordField);
ast_node!(CallExpr, CallExpr);
ast_node!(MatchArm, MatchArm);
ast_node!(GivenStmt, GivenStmt);
ast_node!(ExpectStmt, ExpectStmt);
ast_node!(UseItem, UseItem);
ast_node!(ForumDecl, ForumDecl);
ast_node!(GoverningLawDecl, GoverningLawDecl);
ast_node!(VersionReq, VersionReq);
ast_node!(SourceDecl, SourceDecl);
ast_node!(ObserveDecl, ObserveDecl);
ast_node!(PriorDecl, PriorDecl);
ast_node!(ErrorNode, Error);

/// Attempts to view a syntax root as a `.lit` source file.
#[must_use]
pub fn lit_file(root: SyntaxNode<LitLanguage>) -> Option<LitFile> {
    LitFile::cast(root)
}

/// Attempts to view a syntax root as a `.case` source file.
#[must_use]
pub fn case_file(root: SyntaxNode<LitLanguage>) -> Option<CaseFile> {
    CaseFile::cast(root)
}

// ---- shared helpers -------------------------------------------------------

/// First direct child of type `N`.
pub(crate) fn child<N: AstNode<Language = LitLanguage>>(node: &Node) -> Option<N> {
    node.children().find_map(N::cast)
}

/// Direct children of type `N`.
pub(crate) fn children<N: AstNode<Language = LitLanguage>>(node: &Node) -> Vec<N> {
    node.children().filter_map(N::cast).collect()
}

/// First direct token of `kind`.
pub(crate) fn token(node: &Node, kind: SyntaxKind) -> Option<Token> {
    tokens(node).find(|token| token.kind() == kind)
}

/// Direct tokens (trivia included).
pub(crate) fn tokens(node: &Node) -> impl Iterator<Item = Token> {
    node.children_with_tokens()
        .filter_map(NodeOrToken::into_token)
}

/// The first direct token that is a keyword.
pub(crate) fn keyword(node: &Node) -> Option<Token> {
    tokens(node).find(|token| token.kind().is_keyword())
}

/// The direct child of type `N` that follows the token `after`.
pub(crate) fn child_after<N: AstNode<Language = LitLanguage>>(
    node: &Node,
    after: SyntaxKind,
) -> Option<N> {
    node.children_with_tokens()
        .skip_while(|element| element.kind() != after)
        .filter_map(NodeOrToken::into_node)
        .find_map(N::cast)
}

/// `Field` children of a `FieldList` child (or of the node itself).
pub(crate) fn fields_of(node: &Node) -> Vec<Field> {
    child::<FieldList>(node).map_or_else(|| children(node), |list| children(list.syntax()))
}

/// `Cite` children (citation clauses) of a declaration.
pub(crate) fn cites_of(node: &Node) -> Vec<Cite> {
    children(node)
}

fn docs(node: &Node) -> Option<String> {
    let lines: Vec<String> = tokens(node)
        .take_while(|token| crate::parser::is_trivia(token.kind()))
        .filter(|token| token.kind() == SyntaxKind::DocComment)
        .map(|token| {
            let text = token.text().trim_start_matches("///");
            text.strip_prefix(' ').unwrap_or(text).trim_end().to_owned()
        })
        .collect();
    (!lines.is_empty()).then(|| lines.join("\n"))
}

/// Decodes a string literal token's text (quotes and escapes removed).
#[must_use]
pub fn unescape(literal: &str) -> String {
    let inner = literal.strip_prefix('"').unwrap_or(literal);
    let inner = inner.strip_suffix('"').unwrap_or(inner);
    let mut out = String::with_capacity(inner.len());
    let mut chars = inner.chars();
    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        match chars.next() {
            Some('n') => out.push('\n'),
            Some('r') => out.push('\r'),
            Some('t') => out.push('\t'),
            Some(other) => out.push(other),
            None => {}
        }
    }
    out
}

#[cfg(test)]
mod tests;
