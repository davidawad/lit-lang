//! Bootstrap reader: `.lit` text to [`ItemTree`] (SPEC §2-§3.1, §3.3).
//!
//! This stands in for `litigate-syntax`'s CST until that parser lands; once
//! it does, the `item_tree` query lowers the CST instead and this module is
//! deleted. It follows the same grammar, LL decisions, and recovery rules.
mod expr;
mod fields;
mod items;
pub mod lex;
pub mod tree;

use crate::db::span::{Report, TextRange};
use lex::{Tok, Token};
use litigate_base::{Applicability, DiagnosticCode as C};
use tree::{ItemTree, Name};

/// Parse output: the tree and every syntax report.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Parsed {
    /// Item tree (recovered on error).
    pub tree: ItemTree,
    /// L0xxx reports and field-shape reports.
    pub reports: Vec<Report>,
}

/// Parses one `.lit` source.
#[must_use]
pub fn parse(src: &str) -> Parsed {
    let (tokens, mut reports) = lex::lex(src);
    let mut parser = Parser {
        tokens,
        pos: 0,
        reports: Vec::new(),
    };
    let tree = parser.file();
    reports.append(&mut parser.reports);
    Parsed { tree, reports }
}

/// Words that begin a top-level item (recovery points).
pub(crate) const ITEM_KEYWORDS: [&str; 22] = [
    "module",
    "import",
    "package",
    "start",
    "group",
    "type",
    "enum",
    "event",
    "state",
    "decision",
    "chance",
    "terminal",
    "link",
    "instance",
    "rule",
    "deadline",
    "counting",
    "calendar",
    "transition",
    "authority",
    "default",
    "exception",
];

/// Marker error: a report was already emitted; unwind to recovery.
pub(crate) struct Fail;

/// Result of a parse step.
pub(crate) type PResult<T> = Result<T, Fail>;

pub(crate) struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    reports: Vec<Report>,
}

impl Parser {
    pub(crate) fn peek(&self) -> &Token {
        self.nth(0)
    }

    pub(crate) fn nth(&self, n: usize) -> &Token {
        let last = self.tokens.len().saturating_sub(1);
        &self.tokens[(self.pos + n).min(last)]
    }

    pub(crate) fn bump(&mut self) -> Token {
        let token = self.peek().clone();
        if token.kind != Tok::Eof {
            self.pos += 1;
        }
        token
    }

    pub(crate) fn prev_end(&self) -> u32 {
        self.pos
            .checked_sub(1)
            .and_then(|i| self.tokens.get(i))
            .map_or(0, |t| t.range.end)
    }

    pub(crate) fn at_punct(&self, p: &str) -> bool {
        self.nth_punct(0, p)
    }

    pub(crate) fn nth_punct(&self, n: usize, p: &str) -> bool {
        let t = self.nth(n);
        t.kind == Tok::Punct && t.text == p
    }

    pub(crate) fn at_kw(&self, kw: &str) -> bool {
        let t = self.peek();
        t.kind == Tok::Ident && t.text == kw
    }

    pub(crate) fn eat_punct(&mut self, p: &str) -> bool {
        let hit = self.at_punct(p);
        if hit {
            self.bump();
        }
        hit
    }

    pub(crate) fn eat_kw(&mut self, kw: &str) -> bool {
        let hit = self.at_kw(kw);
        if hit {
            self.bump();
        }
        hit
    }

    pub(crate) fn describe(token: &Token) -> String {
        match token.kind {
            Tok::Eof => "end of file".to_owned(),
            Tok::Str => "a string".to_owned(),
            Tok::DocComment => "a doc comment".to_owned(),
            Tok::CiteText => "citation text".to_owned(),
            _ => format!("`{}`", token.text),
        }
    }

    /// Reports L0001 at the current token and fails.
    pub(crate) fn unexpected<T>(&mut self, expected: &str) -> PResult<T> {
        let token = self.peek().clone();
        let found = Self::describe(&token);
        self.reports.push(Report::new(
            C::UnexpectedToken,
            token.range,
            format!("expected {expected}, found {found}"),
        ));
        Err(Fail)
    }

    pub(crate) fn expect_punct(&mut self, p: &str) -> PResult<Token> {
        if self.at_punct(p) {
            Ok(self.bump())
        } else {
            self.unexpected(&format!("`{p}`"))
        }
    }

    pub(crate) fn expect_kw(&mut self, kw: &str) -> PResult<Token> {
        if self.at_kw(kw) {
            Ok(self.bump())
        } else {
            self.unexpected(&format!("`{kw}`"))
        }
    }

    pub(crate) fn ident(&mut self) -> PResult<Name> {
        if self.peek().kind == Tok::Ident && self.peek().text != "cite" {
            let t = self.bump();
            Ok(Name {
                text: t.text,
                range: t.range,
            })
        } else {
            self.unexpected("an identifier")
        }
    }

    pub(crate) fn token_name(&mut self, kind: Tok, what: &str) -> PResult<Name> {
        if self.peek().kind == kind {
            let t = self.bump();
            Ok(Name {
                text: t.text,
                range: t.range,
            })
        } else {
            self.unexpected(what)
        }
    }

    pub(crate) fn string(&mut self) -> PResult<Name> {
        self.token_name(Tok::Str, "a string")
    }

    pub(crate) fn date(&mut self) -> PResult<Name> {
        self.token_name(Tok::Date, "a date")
    }

    /// `cite` + citation text; the `cite` token is current.
    pub(crate) fn cite_text(&mut self) -> PResult<Name> {
        let cite = self.bump();
        if self.peek().kind == Tok::CiteText && !self.peek().text.is_empty() {
            let t = self.bump();
            Ok(Name {
                text: t.text,
                range: t.range,
            })
        } else {
            self.reports.push(Report::new(
                C::UnexpectedToken,
                cite.range,
                "`cite` needs citation text on the same line",
            ));
            if self.peek().kind == Tok::CiteText {
                self.bump();
            }
            Err(Fail)
        }
    }

    pub(crate) fn report(&mut self, report: Report) {
        self.reports.push(report);
    }

    /// Opening-delimiter bookkeeping: on a missing closer, L0002 at the
    /// opener with a fix inserting the closer.
    pub(crate) fn close(&mut self, open: TextRange, closer: &str) -> PResult<()> {
        if self.eat_punct(closer) {
            return Ok(());
        }
        let at = TextRange::new(self.prev_end(), self.prev_end());
        self.reports.push(
            Report::new(
                C::UnterminatedLiteral,
                open,
                format!("unclosed delimiter; expected `{closer}`"),
            )
            .fix(at, closer, Applicability::MachineApplicable),
        );
        Err(Fail)
    }

    /// Skips to the next `,`/`}` at depth 0 or the next line start.
    pub(crate) fn recover_in_body(&mut self) {
        let mut depth = 0_i32;
        let start = self.pos;
        loop {
            let t = self.peek();
            if t.kind == Tok::Eof {
                return;
            }
            if self.pos > start && depth == 0 && t.line_start {
                return;
            }
            if t.kind == Tok::Punct {
                match t.text.as_str() {
                    "{" | "[" | "(" => depth += 1,
                    "}" | "]" | ")" if depth == 0 => return,
                    "}" | "]" | ")" => depth -= 1,
                    "," if depth == 0 => {
                        self.bump();
                        return;
                    }
                    _ => {}
                }
            }
            self.bump();
        }
    }

    /// Skips to the next top-level item keyword at the start of a line.
    pub(crate) fn recover_top(&mut self) {
        let start = self.pos;
        loop {
            let t = self.peek();
            let at_item = t.line_start
                && (t.kind == Tok::DocComment
                    || (t.kind == Tok::Ident && ITEM_KEYWORDS.contains(&t.text.as_str()))
                    || (t.kind == Tok::Ident && t.text == "test"));
            if t.kind == Tok::Eof || (self.pos > start && at_item) {
                return;
            }
            self.bump();
        }
    }

    /// Optional `,` separator.
    pub(crate) fn sep(&mut self) {
        self.eat_punct(",");
    }
}

#[cfg(test)]
mod tests;
