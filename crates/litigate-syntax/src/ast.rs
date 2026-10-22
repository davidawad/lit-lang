//! Typed, intentionally minimal wrappers over lossless Rowan syntax nodes.
//!
//! W1 adds accessors; the set of wrappers and the kinds each one casts from
//! are frozen by SPEC §3.
use crate::syntax_kind::{LitLanguage, SyntaxKind};
pub use rowan::ast::AstNode;
use rowan::SyntaxNode;

type Node = rowan::SyntaxNode<LitLanguage>;

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
                self.syntax
                    .children_with_tokens()
                    .filter_map(rowan::NodeOrToken::into_token)
                    .find(|token| token.kind() == SyntaxKind::Ident)
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
);
ast_node!(UsesDecl, UsesDecl);
ast_node!(PerspectiveDecl, PerspectiveDecl);
ast_node!(PartyDecl, PartyDecl);
ast_node!(CaseEvent, CaseEvent);
ast_node!(
    CaseBlock,
    FactsBlock | StakesBlock | BeliefsBlock | AnalysisBlock
);

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

#[cfg(test)]
mod tests {
    use super::{Expr, NodeDecl};
    use crate::syntax_kind::SyntaxKind;
    use rowan::{ast::AstNode, GreenNodeBuilder, SyntaxNode};

    #[test]
    fn multi_kind_wrappers_cast_every_listed_kind() {
        assert!(NodeDecl::can_cast(SyntaxKind::ChanceDecl));
        assert!(NodeDecl::can_cast(SyntaxKind::TerminalDecl));
        assert!(!NodeDecl::can_cast(SyntaxKind::EdgeDecl));
        assert!(Expr::can_cast(SyntaxKind::MatchExpr) && !Expr::can_cast(SyntaxKind::Ident));
    }

    #[test]
    fn name_is_first_identifier() {
        let mut builder = GreenNodeBuilder::new();
        builder.start_node(SyntaxKind::StateDecl.into());
        builder.token(SyntaxKind::KwState.into(), "state");
        builder.token(SyntaxKind::Whitespace.into(), " ");
        builder.token(SyntaxKind::Ident.into(), "answer-due");
        builder.finish_node();
        let root = SyntaxNode::new_root(builder.finish());
        let decl = NodeDecl::cast(root).map(|d| d.name().map(|t| t.text().to_string()));
        assert_eq!(decl, Some(Some("answer-due".to_string())));
    }
}
