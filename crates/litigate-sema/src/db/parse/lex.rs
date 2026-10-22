//! Hand-written lexer for `.lit` sources (SPEC §2), including the `cite`
//! citation mode. Trivia is skipped; doc comments are kept as tokens.
use crate::db::span::{Report, TextRange};
use litigate_base::DiagnosticCode as C;

/// Token class.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tok {
    /// Identifier or contextual keyword.
    Ident,
    /// A lone `_`.
    Underscore,
    /// `21`, `0.35`.
    Number,
    /// `2_000.usd`.
    Money,
    /// `21.days`, `8.hours`.
    Duration,
    /// Quoted string (raw text, quotes included).
    Str,
    /// `2025-03-03`.
    Date,
    /// Normalized citation text after `cite`.
    CiteText,
    /// `/// doc` line.
    DocComment,
    /// Punctuation.
    Punct,
    /// Unrecognized input (already reported).
    Error,
    /// End of input.
    Eof,
}

/// One significant token.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    /// Class.
    pub kind: Tok,
    /// Byte range.
    pub range: TextRange,
    /// Text (normalized for cite text; raw otherwise).
    pub text: String,
    /// First significant token on its line.
    pub line_start: bool,
}

const PUNCT: [&str; 31] = [
    "::", "->", "=>", "==", "!=", "<=", ">=", "&&", "||", "..", "{", "}", "(", ")", "[", "]", ":",
    ",", ".", "=", "<", ">", "+", "-", "*", "/", "%", "!", "?", "@", "^",
];
const DURATION_SUFFIXES: [&str; 4] = ["days", "court_days", "business_days", "hours"];

fn is_start(c: char) -> bool {
    c.is_alphabetic() || c == '_'
}

fn is_continue(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

struct Lexer<'a> {
    src: &'a str,
    pos: usize,
    tokens: Vec<Token>,
    reports: Vec<Report>,
    line_start: bool,
}

/// Tokenizes `src`; the last token is always [`Tok::Eof`].
#[must_use]
pub fn lex(src: &str) -> (Vec<Token>, Vec<Report>) {
    let mut lexer = Lexer {
        src,
        pos: 0,
        tokens: Vec::new(),
        reports: Vec::new(),
        line_start: true,
    };
    if src.starts_with('\u{feff}') {
        lexer.report(C::InvalidToken, 0, 3, "a byte-order mark is not allowed");
        lexer.pos = 3;
    }
    while lexer.step() {}
    let end = lexer.pos;
    lexer.push(Tok::Eof, end, end, String::new());
    (lexer.tokens, lexer.reports)
}

fn offset(n: usize) -> u32 {
    u32::try_from(n).unwrap_or(u32::MAX)
}

impl Lexer<'_> {
    fn rest(&self) -> &str {
        &self.src[self.pos..]
    }

    fn peek(&self, n: usize) -> Option<char> {
        self.rest().chars().nth(n)
    }

    fn report(&mut self, code: C, start: usize, end: usize, message: &str) {
        self.reports.push(Report::new(
            code,
            TextRange::new(offset(start), offset(end)),
            message,
        ));
    }

    fn push(&mut self, kind: Tok, start: usize, end: usize, text: String) {
        self.tokens.push(Token {
            kind,
            range: TextRange::new(offset(start), offset(end)),
            text,
            line_start: self.line_start,
        });
        self.line_start = false;
    }

    fn take_while(&mut self, pred: impl Fn(char) -> bool) -> usize {
        let len = self.rest().find(|c| !pred(c)).unwrap_or(self.rest().len());
        self.pos += len;
        len
    }

    /// Lexes one token or trivia run; false at end of input.
    fn step(&mut self) -> bool {
        let Some(c) = self.peek(0) else {
            return false;
        };
        let start = self.pos;
        if c.is_whitespace() {
            let len = self
                .rest()
                .find(|c: char| !c.is_whitespace())
                .unwrap_or(self.rest().len());
            self.line_start |= self.rest()[..len].contains('\n');
            self.pos += len;
        } else if self.rest().starts_with("///") {
            let len = self.rest().find('\n').unwrap_or(self.rest().len());
            let body = &self.rest()[3..len];
            let text = body.strip_prefix(' ').unwrap_or(body).trim_end().to_owned();
            self.pos += len;
            self.push(Tok::DocComment, start, self.pos, text);
        } else if self.rest().starts_with("//") {
            self.pos += self.rest().find('\n').unwrap_or(self.rest().len());
        } else if self.rest().starts_with("/*") {
            match self.rest()[2..].find("*/") {
                Some(end) => {
                    self.line_start |= self.rest()[..end + 2].contains('\n');
                    self.pos += end + 4;
                }
                None => {
                    self.report(
                        C::UnterminatedLiteral,
                        start,
                        start + 2,
                        "unterminated block comment",
                    );
                    self.pos = self.src.len();
                }
            }
        } else if c == '"' {
            self.string(start);
        } else if c.is_ascii_digit() {
            self.number(start);
        } else if is_start(c) {
            self.word(start);
        } else if let Some(p) = PUNCT.iter().find(|p| self.rest().starts_with(**p)) {
            self.pos += p.len();
            self.push(Tok::Punct, start, self.pos, (*p).to_owned());
        } else {
            self.pos += c.len_utf8();
            self.report(
                C::InvalidToken,
                start,
                self.pos,
                &format!("unexpected character `{c}`"),
            );
            self.push(Tok::Error, start, self.pos, c.to_string());
        }
        true
    }

    fn string(&mut self, start: usize) {
        let mut chars = self.rest().char_indices().skip(1);
        let mut value = String::new();
        let mut end = None;
        let mut bad_escapes = Vec::new();
        while let Some((i, c)) = chars.next() {
            match c {
                '"' => {
                    end = Some(i + 1);
                    break;
                }
                '\n' => break,
                '\\' => match chars.next() {
                    Some((_, e @ ('\\' | '"'))) => value.push(e),
                    Some((_, 'n')) => value.push('\n'),
                    Some((_, 'r')) => value.push('\r'),
                    Some((_, 't')) => value.push('\t'),
                    Some((j, other)) => {
                        bad_escapes.push((start + j - 1, start + j + other.len_utf8()))
                    }
                    None => break,
                },
                other => value.push(other),
            }
        }
        for (from, to) in bad_escapes {
            self.report(C::InvalidToken, from, to, "invalid escape");
        }
        if let Some(len) = end {
            self.pos += len;
            self.push(Tok::Str, start, self.pos, value);
        } else {
            let len = self.rest().find('\n').unwrap_or(self.rest().len());
            self.pos += len;
            self.report(
                C::UnterminatedLiteral,
                start,
                self.pos,
                "unterminated string",
            );
            self.push(Tok::Error, start, self.pos, String::new());
        }
    }

    fn digits(&mut self) -> bool {
        let text = &self.rest()[..self
            .rest()
            .find(|c: char| !(c.is_ascii_digit() || c == '_'))
            .unwrap_or(self.rest().len())];
        let ok = !text.starts_with('_') && !text.ends_with('_') && !text.contains("__");
        self.pos += text.len();
        ok
    }

    fn number(&mut self, start: usize) {
        let date_shape = self.rest().len() >= 10
            && self.rest().as_bytes()[..10]
                .iter()
                .enumerate()
                .all(|(i, b)| {
                    if i == 4 || i == 7 {
                        *b == b'-'
                    } else {
                        b.is_ascii_digit()
                    }
                })
            && !self.rest()[10..].starts_with(is_continue);
        if date_shape {
            self.pos += 10;
            let text = self.src[start..self.pos].to_owned();
            if !crate::types::date::is_valid(&text) {
                self.report(
                    C::InvalidDate,
                    start,
                    self.pos,
                    &format!("`{text}` is not a valid date"),
                );
            }
            self.push(Tok::Date, start, self.pos, text);
            return;
        }
        let mut ok = self.digits();
        let mut decimal = false;
        if self.peek(0) == Some('.') && self.peek(1).is_some_and(|c| c.is_ascii_digit()) {
            self.pos += 1;
            ok &= self.digits();
            decimal = true;
        }
        let mut kind = Tok::Number;
        if self.peek(0) == Some('.') && self.peek(1).is_some_and(is_start) {
            let dot = self.pos;
            self.pos += 1;
            self.take_while(is_continue);
            let suffix = &self.src[dot + 1..self.pos];
            if suffix == "usd" {
                kind = Tok::Money;
            } else if DURATION_SUFFIXES.contains(&suffix) {
                kind = Tok::Duration;
                if decimal && suffix != "hours" {
                    self.report(
                        C::InvalidNumber,
                        start,
                        self.pos,
                        "day amounts must be integers",
                    );
                }
            } else {
                self.report(
                    C::InvalidNumber,
                    dot,
                    self.pos,
                    &format!("unknown literal suffix `{suffix}`"),
                );
                kind = Tok::Error;
            }
        }
        if !ok {
            self.report(
                C::InvalidNumber,
                start,
                self.pos,
                "`_` may only separate digits",
            );
        }
        let text = self.src[start..self.pos].to_owned();
        self.push(kind, start, self.pos, text);
    }

    fn word(&mut self, start: usize) {
        loop {
            self.take_while(is_continue);
            if self.peek(0) == Some('-') && self.peek(1).is_some_and(is_continue) {
                self.pos += 1;
            } else {
                break;
            }
        }
        let text = self.src[start..self.pos].to_owned();
        if text == "_" {
            self.push(Tok::Underscore, start, self.pos, text);
            return;
        }
        let cite = text == "cite";
        self.push(Tok::Ident, start, self.pos, text);
        if cite {
            self.take_while(|c| c == ' ' || c == '\t');
            let begin = self.pos;
            let rest = self.rest();
            let stop = rest
                .char_indices()
                .find(|(i, c)| matches!(c, '\n' | '{' | '}' | ',') || rest[*i..].starts_with("//"))
                .map_or(rest.len(), |(i, _)| i);
            let text = rest[..stop]
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ");
            let trimmed = rest[..stop].trim_end().len();
            self.pos += stop;
            self.push(Tok::CiteText, begin, begin + trimmed, text);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kinds(src: &str) -> Vec<(Tok, String)> {
        lex(src).0.into_iter().map(|t| (t.kind, t.text)).collect()
    }

    #[test]
    fn lexes_typed_literals_and_kebab_idents() {
        let toks = kinds("answer-due 21.days 2_000.usd 1.5.hours 2025-03-03 a - b _ x");
        let k: Vec<Tok> = toks.iter().map(|t| t.0).collect();
        assert_eq!(
            k,
            [
                Tok::Ident,
                Tok::Duration,
                Tok::Money,
                Tok::Duration,
                Tok::Date,
                Tok::Ident,
                Tok::Punct,
                Tok::Ident,
                Tok::Underscore,
                Tok::Ident,
                Tok::Eof
            ]
        );
    }

    #[test]
    fn cite_mode_stops_at_brace_and_collapses_space() {
        let toks = kinds("cite FRCP  6(a)   {");
        assert_eq!(toks[1], (Tok::CiteText, "FRCP 6(a)".to_owned()));
        assert_eq!(toks[2].1, "{");
    }

    #[test]
    fn reports_lexical_errors() {
        let codes = |s: &str| lex(s).1.into_iter().map(|r| r.code).collect::<Vec<_>>();
        assert_eq!(codes("2025-02-30"), [C::InvalidDate]);
        assert_eq!(codes("1.5.days"), [C::InvalidNumber]);
        assert_eq!(codes("3.weeks"), [C::InvalidNumber]);
        assert_eq!(codes("1__0"), [C::InvalidNumber]);
        assert_eq!(codes("\"abc"), [C::UnterminatedLiteral]);
        assert_eq!(codes("/* x"), [C::UnterminatedLiteral]);
        assert_eq!(codes("$"), [C::InvalidToken]);
    }

    #[test]
    fn doc_comments_and_line_starts() {
        let (toks, _) = lex("/// Hello\nstate x // c\n  y");
        assert_eq!(toks[0].text, "Hello");
        assert!(toks[1].line_start && !toks[2].line_start && toks[3].line_start);
    }
}
