//! Lossless syntax for `.lit` and `.case` (SPEC §2-§3, §10): a logos lexer,
//! a hand-written recursive-descent parser with error recovery producing a
//! rowan CST, typed AST views, and the canonical formatter.
//!
//! Parsing is total: [`parse`] never panics and always returns a tree whose
//! text is exactly the input, with every problem reported as an L0xxx
//! [`SyntaxError`]. Convert errors with [`Parse::diagnostics`].
//!
//! ```
//! use litigate_syntax::{format, parse, FileKind};
//!
//! let parsed = parse("module m\nstart s\n", FileKind::Lit);
//! assert!(parsed.is_ok());
//! assert_eq!(parsed.syntax().to_string(), "module m\nstart s\n");
//! assert_eq!(format("module   m\nstart s", FileKind::Lit).as_deref(), Ok("module m\n\nstart s\n"));
//! ```
//!
//! Contract changes made by W1 are additive (SPEC §11): node kinds
//! `ImportList`, `Param`, `FieldExpr`, `Ratio`; keyword `KwUrl`; the
//! lookahead sentinel `Eof`; new typed views; `Expr` also casts `FieldExpr`
//! and `Ratio`.
pub mod ast;
mod error;
#[allow(clippy::enum_glob_use)]
pub mod format;
pub mod keyword_table;
// Grammar rules read best with the token/node vocabulary in scope.
#[allow(clippy::enum_glob_use)]
mod grammar;
pub mod lexer;
mod parse;
#[allow(clippy::enum_glob_use)]
mod parser;
mod suggest;
pub mod syntax_kind;
#[allow(clippy::enum_glob_use)]
mod vocab;

pub use error::{span_of, SyntaxError, TextFix};
pub use format::{check_format, format, FormatError};
pub use lexer::{
    FILING_WORDS, GUARD_OP_WORDS, REFINEMENT_WORDS, RESERVED_FILING, RESERVED_GUARD_OP,
    RESERVED_REFINEMENT, RESERVED_WORDS,
};
pub use parse::{parse, parse_case, parse_lit, Edition, FileKind, Parse, SyntaxNode, SyntaxToken};
pub use rowan::{TextRange, TextSize};
pub use syntax_kind::{LitLanguage, SyntaxKind};
pub use vocab::{CASE_ITEM_KEYWORDS, LIT_ITEM_KEYWORDS};
