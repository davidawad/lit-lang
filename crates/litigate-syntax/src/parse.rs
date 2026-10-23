//! Public parse entry points and the [`Parse`] result.
use crate::ast::{self, AstNode};
use crate::error::SyntaxError;
use crate::grammar::{case_file, lit_file};
use crate::lexer::lex;
use crate::parser::{Parser, CASE_ITEM_KEYWORDS, LIT_ITEM_KEYWORDS};
use crate::syntax_kind::LitLanguage;
use litigate_base::{Diagnostic, DiagnosticCode, FileId, LineIndex};
use rowan::GreenNode;
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
    errors.sort_by_key(|error| error.range.start());
    errors.dedup_by_key(|error| error.range.start());
    merge_closer_fixes(&mut errors);
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

/// Unclosed delimiters whose closers are inserted at the same offset get
/// one combined fix on the innermost diagnostic (closers innermost first),
/// so applying every machine-applicable fix yields well-nested text.
fn merge_closer_fixes(errors: &mut [SyntaxError]) {
    let insertion = |error: &SyntaxError| {
        (error.code == DiagnosticCode::UnterminatedLiteral)
            .then(|| error.fixes.first())
            .flatten()
            .filter(|fix| fix.range.is_empty())
            .map(|fix| fix.range.start())
    };
    let mut offsets: Vec<_> = errors.iter().filter_map(insertion).collect();
    offsets.sort_unstable();
    offsets.dedup();
    for offset in offsets {
        let mut group: Vec<usize> = (0..errors.len())
            .filter(|index| insertion(&errors[*index]) == Some(offset))
            .collect();
        if group.len() < 2 {
            continue;
        }
        group.sort_by_key(|index| std::cmp::Reverse(errors[*index].range.start()));
        let closers: String = group
            .iter()
            .filter_map(|index| errors[*index].fixes.first())
            .map(|fix| fix.replacement.clone())
            .collect();
        for (position, index) in group.iter().enumerate() {
            let error = &mut errors[*index];
            if position == 0 {
                error.fixes[0].replacement.clone_from(&closers);
            } else {
                error.fixes.clear();
                error
                    .notes
                    .push("closed by the fix on the innermost unclosed delimiter".to_owned());
            }
        }
    }
}

#[cfg(test)]
mod tests;
