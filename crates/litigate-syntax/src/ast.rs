//! Typed, intentionally minimal wrappers over lossless Rowan syntax nodes.
use crate::syntax_kind::{LitLanguage, SyntaxKind};
use rowan::{ast::AstNode, SyntaxNode};

type Node = rowan::SyntaxNode<LitLanguage>;

macro_rules! ast_node {
    ($name:ident, $kind:ident) => {
        /// Typed view of the corresponding syntax node.
        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        pub struct $name {
            syntax: Node,
        }
        impl AstNode for $name {
            type Language = LitLanguage;
            fn can_cast(kind: SyntaxKind) -> bool {
                kind == SyntaxKind::$kind
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
ast_node!(RuleDecl, RuleDecl);
ast_node!(Expr, Expr);
ast_node!(TypeDecl, TypeDecl);
ast_node!(EnumDecl, EnumDecl);
ast_node!(StateDecl, StateDecl);
ast_node!(DecisionDecl, DecisionDecl);
ast_node!(ChanceDecl, ChanceDecl);
ast_node!(TerminalDecl, TerminalDecl);
ast_node!(EdgeDecl, EdgeDecl);
ast_node!(DeadlineDecl, DeadlineDecl);
ast_node!(CountingDecl, CountingDecl);
ast_node!(CalendarDecl, CalendarDecl);
ast_node!(AuthorityDecl, AuthorityDecl);
ast_node!(TestDecl, TestDecl);
ast_node!(ExceptionDecl, ExceptionDecl);
ast_node!(DefaultDecl, DefaultDecl);
ast_node!(TransitionDecl, TransitionDecl);
ast_node!(PartyDecl, PartyDecl);
ast_node!(CaseEvent, CaseEvent);
ast_node!(CaseAssignment, CaseAssignment);

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
