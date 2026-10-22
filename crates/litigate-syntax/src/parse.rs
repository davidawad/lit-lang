//! Public parse entry points and the [`Parse`] result.
use crate::ast::{self, AstNode};
use crate::error::SyntaxError;
use crate::grammar::{case_file, lit_file};
use crate::lexer::lex;
use crate::parser::{is_decl, is_trivia, Parser, CASE_ITEM_KEYWORDS, LIT_ITEM_KEYWORDS};
use crate::syntax_kind::{LitLanguage, SyntaxKind};
use litigate_base::{Diagnostic, DiagnosticCode, FileId, LineIndex};
use rowan::{GreenNode, NodeOrToken};
use std::path::Path;

/// A syntax node of the litigate CST.
pub type SyntaxNode = rowan::SyntaxNode<LitLanguage>;
/// A syntax token of the litigate CST.
pub type SyntaxToken = rowan::SyntaxToken<LitLanguage>;

/// Which grammar a source file uses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FileKind {
    /// A `.lit` rule module.
    Lit,
    /// A `.case` matter.
    Case,
}

impl FileKind {
    /// Kind of a path by extension (`.lit` / `.case`).
    #[must_use]
    pub fn from_path(path: &Path) -> Option<Self> {
        match path.extension()?.to_str()? {
            "lit" => Some(Self::Lit),
            "case" => Some(Self::Case),
            _ => None,
        }
    }
}

/// A lossless syntax tree plus every L0xxx error found while building it.
/// Parsing never fails: errors are recovered into `Error` nodes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Parse {
    green: GreenNode,
    errors: Vec<SyntaxError>,
    kind: FileKind,
}

impl Parse {
    /// The immutable green tree.
    #[must_use]
    pub fn green(&self) -> &GreenNode {
        &self.green
    }

    /// A fresh red-tree root over the green tree.
    #[must_use]
    pub fn syntax(&self) -> SyntaxNode {
        SyntaxNode::new_root(self.green.clone())
    }

    /// Grammar the file was parsed with.
    #[must_use]
    pub fn kind(&self) -> FileKind {
        self.kind
    }

    /// Syntax errors, sorted by position, at most one per position.
    #[must_use]
    pub fn errors(&self) -> &[SyntaxError] {
        &self.errors
    }

    /// True when the file has no syntax errors.
    #[must_use]
    pub fn is_ok(&self) -> bool {
        self.errors.is_empty()
    }

    /// Errors as structured diagnostics for `file`.
    #[must_use]
    pub fn diagnostics(&self, file: &FileId) -> Vec<Diagnostic> {
        let index = LineIndex::new(&self.syntax().to_string());
        self.errors
            .iter()
            .map(|error| error.to_diagnostic(file, &index))
            .collect()
    }

    /// Typed root of a `.lit` file.
    #[must_use]
    pub fn lit_file(&self) -> Option<ast::LitFile> {
        ast::LitFile::cast(self.syntax())
    }

    /// Typed root of a `.case` file.
    #[must_use]
    pub fn case_file(&self) -> Option<ast::CaseFile> {
        ast::CaseFile::cast(self.syntax())
    }

    /// Indented dump of the CST (kinds, ranges, token text) for snapshots.
    #[must_use]
    pub fn debug_tree(&self) -> String {
        format!("{:#?}", self.syntax())
    }
}

/// Parses `text` with the grammar of `kind`.
#[must_use]
pub fn parse(text: &str, kind: FileKind) -> Parse {
    let (tokens, mut errors) = lex(text);
    let items = match kind {
        FileKind::Lit => LIT_ITEM_KEYWORDS,
        FileKind::Case => CASE_ITEM_KEYWORDS,
    };
    let mut parser = Parser::new(text, tokens, items);
    match kind {
        FileKind::Lit => lit_file(&mut parser),
        FileKind::Case => case_file(&mut parser),
    }
    let (green, parse_errors) = parser.finish_file();
    let lexical = errors.clone();
    errors.extend(parse_errors.into_iter().filter(|error| {
        !lexical.iter().any(|lex| {
            lex.range.start() == error.range.start()
                || lex
                    .range
                    .intersect(error.range)
                    .is_some_and(|r| !r.is_empty())
        })
    }));
    errors.extend(dangling_docs(&SyntaxNode::new_root(green.clone())));
    errors.sort_by_key(|error| error.range.start());
    errors.dedup_by_key(|error| error.range.start());
    Parse {
        green,
        errors,
        kind,
    }
}

/// Parses a `.lit` rule module.
#[must_use]
pub fn parse_lit(text: &str) -> Parse {
    parse(text, FileKind::Lit)
}

/// Parses a `.case` matter.
#[must_use]
pub fn parse_case(text: &str) -> Parse {
    parse(text, FileKind::Case)
}

/// L0006 for every doc comment that does not lead a declaration.
fn dangling_docs(root: &SyntaxNode) -> Vec<SyntaxError> {
    root.descendants_with_tokens()
        .filter_map(NodeOrToken::into_token)
        .filter(|token| token.kind() == SyntaxKind::DocComment && !leads_decl(token))
        .map(|token| {
            SyntaxError::new(
                DiagnosticCode::DanglingDocComment,
                token.text_range(),
                "doc comment is not followed by a declaration; use `//` for a plain comment",
            )
        })
        .collect()
}

fn leads_decl(token: &SyntaxToken) -> bool {
    let Some(parent) = token.parent() else {
        return false;
    };
    let mut before = std::iter::successors(token.prev_sibling_or_token(), |element| {
        element.prev_sibling_or_token()
    });
    is_decl(parent.kind()) && before.all(|element| is_trivia(element.kind()))
}
