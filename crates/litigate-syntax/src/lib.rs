//! Lossless syntax for `.lit` and `.case` (SPEC §2-§3, §10): a logos lexer,
//! a hand-written recursive-descent parser with error recovery producing a
//! rowan CST, typed AST views, and the canonical formatter.
//!
//! Parsing is total: [`parse`] never panics and always returns a tree whose
//! text is exactly the input, with every problem reported as an L0xxx
//! [`SyntaxError`].
pub mod ast;
mod error;
#[allow(clippy::enum_glob_use)]
pub mod format;
// Grammar rules read best with the token/node vocabulary in scope.
#[allow(clippy::enum_glob_use)]
mod grammar;
pub mod lexer;
mod parse;
#[allow(clippy::enum_glob_use)]
mod parser;
mod suggest;
pub mod syntax_kind;

pub use error::{span_of, SyntaxError, TextFix};
pub use format::{check_format, format, FormatError};
pub use parse::{parse, parse_case, parse_lit, FileKind, Parse, SyntaxNode, SyntaxToken};
pub use rowan::{TextRange, TextSize};
pub use syntax_kind::{LitLanguage, SyntaxKind};
