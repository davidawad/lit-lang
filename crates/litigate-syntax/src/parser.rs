//! Recursive-descent parser core: token cursor, trivia attachment, error
//! reporting, and recovery (SPEC §3.4). Grammar rules live in `grammar/`.
use crate::error::{range, SyntaxError};
use crate::lexer::Token;
use crate::suggest::nearest;
use crate::syntax_kind::SyntaxKind::{self, *};
use crate::vocab::describe_set;
pub(crate) use crate::vocab::{is_decl, is_trivia, CASE_ITEM_KEYWORDS, LIT_ITEM_KEYWORDS};
use litigate_base::{Applicability, DiagnosticCode};
use rowan::{Checkpoint, GreenNode, GreenNodeBuilder, TextRange};

/// Maximum nesting of expressions, lists, and blocks before the parser
/// stops descending (keeps the parser total on adversarial input).
const MAX_DEPTH: usize = 128;

pub(crate) struct Parser<'t> {
    text: &'t str,
    tokens: Vec<Token>,
    sig: Vec<usize>,
    pos: usize,
    emitted: usize,
    builder: GreenNodeBuilder<'static>,
    pub(crate) errors: Vec<SyntaxError>,
    pub(crate) depth: usize,
    /// True between starting a declaration and its first token: doc
    /// comments emitted now lead it; any others are dangling (L0006).
    leading_docs: bool,
    item_keywords: &'static [SyntaxKind],
}

impl<'t> Parser<'t> {
    pub(crate) fn new(text: &'t str, tokens: Vec<Token>, items: &'static [SyntaxKind]) -> Self {
        let sig = tokens
            .iter()
            .enumerate()
            .filter(|(_, token)| !is_trivia(token.kind))
            .map(|(index, _)| index)
            .collect();
        Self {
            text,
            tokens,
            sig,
            pos: 0,
            emitted: 0,
            builder: GreenNodeBuilder::new(),
            errors: Vec::new(),
            depth: 0,
            leading_docs: false,
            item_keywords: items,
        }
    }

    // ---- lookahead -------------------------------------------------------

    pub(crate) fn nth(&self, n: usize) -> SyntaxKind {
        self.sig
            .get(self.pos + n)
            .map_or(Eof, |index| self.tokens[*index].kind)
    }

    pub(crate) fn nth_text(&self, n: usize) -> &'t str {
        self.sig
            .get(self.pos + n)
            .map_or("", |index| &self.text[self.tokens[*index].range])
    }

    pub(crate) fn at(&self, kind: SyntaxKind) -> bool {
        self.nth(0) == kind
    }

    pub(crate) fn nth_kw(&self, n: usize, kw: SyntaxKind) -> bool {
        self.nth(n) == Ident && kw.text() == Some(self.nth_text(n))
    }

    pub(crate) fn at_kw(&self, kw: SyntaxKind) -> bool {
        self.nth_kw(0, kw)
    }

    /// The keyword (from `set`) spelled by the current identifier, if any.
    pub(crate) fn at_any_kw(&self, set: &[SyntaxKind]) -> Option<SyntaxKind> {
        set.iter().copied().find(|kw| self.at_kw(*kw))
    }

    pub(crate) fn range_of(&self, n: usize) -> TextRange {
        self.sig.get(self.pos + n).map_or_else(
            || range(self.text.len(), self.text.len()),
            |index| self.tokens[*index].range,
        )
    }

    pub(crate) fn current_range(&self) -> TextRange {
        self.range_of(0)
    }

    pub(crate) fn text_between(&self, start: rowan::TextSize, end: rowan::TextSize) -> &'t str {
        &self.text[TextRange::new(start, end.max(start))]
    }

    /// End offset of the previous significant token.
    pub(crate) fn last_end(&self) -> usize {
        self.pos
            .checked_sub(1)
            .and_then(|prev| self.sig.get(prev))
            .map_or(0, |index| usize::from(self.tokens[*index].range.end()))
    }

    /// True when the current token is the first thing on its line.
    pub(crate) fn at_line_start(&self) -> bool {
        let start = usize::from(self.current_range().start());
        start == 0 || self.text.as_bytes().get(start - 1) == Some(&b'\n')
    }

    /// True when trivia between the previous and current token has a newline.
    pub(crate) fn newline_before(&self) -> bool {
        let start = usize::from(self.current_range().start());
        self.text
            .get(self.last_end()..start)
            .is_some_and(|gap| gap.contains('\n'))
    }

    /// True when no trivia separates the previous and current token.
    pub(crate) fn adjacent(&self) -> bool {
        self.last_end() == usize::from(self.current_range().start())
    }

    /// A top-level item keyword starting a line: a recovery anchor.
    pub(crate) fn at_item_anchor(&self) -> bool {
        self.at_line_start() && self.at_any_kw(self.item_keywords).is_some()
    }

    pub(crate) fn item_keywords(&self) -> &'static [SyntaxKind] {
        self.item_keywords
    }

    pub(crate) fn at_end(&self) -> bool {
        self.at(Eof)
    }

    // ---- tree building ---------------------------------------------------

    fn flush_trivia(&mut self, keep_docs: bool) {
        let target = self.sig.get(self.pos).copied().unwrap_or(self.tokens.len());
        let mut stop = target;
        if keep_docs {
            let mut index = target;
            while index > self.emitted {
                let kind = self.tokens[index - 1].kind;
                if kind == DocComment {
                    stop = index - 1;
                } else if kind != Whitespace {
                    break;
                }
                index -= 1;
            }
        }
        while self.emitted < stop {
            let token = self.tokens[self.emitted];
            if token.kind == DocComment && !self.leading_docs {
                self.errors.push(SyntaxError::new(
                    DiagnosticCode::DanglingDocComment,
                    token.range,
                    "doc comment is not followed by a declaration; use `//` for a plain comment",
                ));
            }
            self.builder
                .token(token.kind.into(), &self.text[token.range]);
            self.emitted += 1;
        }
    }

    pub(crate) fn start_root(&mut self, kind: SyntaxKind) {
        self.builder.start_node(kind.into());
    }

    pub(crate) fn start(&mut self, kind: SyntaxKind) {
        self.flush_trivia(is_decl(kind));
        self.builder.start_node(kind.into());
        self.leading_docs |= is_decl(kind);
    }

    pub(crate) fn finish(&mut self) {
        self.builder.finish_node();
    }

    pub(crate) fn checkpoint(&mut self) -> Checkpoint {
        self.flush_trivia(false);
        self.builder.checkpoint()
    }

    pub(crate) fn start_at(&mut self, checkpoint: Checkpoint, kind: SyntaxKind) {
        self.builder.start_node_at(checkpoint, kind.into());
    }

    /// Emits the current token as `kind` (keywords are remapped here).
    pub(crate) fn bump_as(&mut self, kind: SyntaxKind) {
        if let Some(index) = self.sig.get(self.pos).copied() {
            self.flush_trivia(false);
            let token = self.tokens[index];
            self.builder.token(kind.into(), &self.text[token.range]);
            self.emitted = index + 1;
            self.pos += 1;
            self.leading_docs = false;
        }
    }

    pub(crate) fn bump(&mut self) {
        let kind = self.nth(0);
        self.bump_as(kind);
    }

    pub(crate) fn eat(&mut self, kind: SyntaxKind) -> bool {
        let found = self.at(kind);
        if found {
            self.bump();
        }
        found
    }

    pub(crate) fn eat_kw(&mut self, kw: SyntaxKind) -> bool {
        let found = self.at_kw(kw);
        if found {
            self.bump_as(kw);
        }
        found
    }

    /// Consumes `kind` or reports L0001 (without consuming).
    pub(crate) fn expect(&mut self, kind: SyntaxKind) -> bool {
        self.eat(kind) || {
            self.error_expected(&[kind]);
            false
        }
    }

    pub(crate) fn expect_kw(&mut self, kw: SyntaxKind) -> bool {
        self.eat_kw(kw) || {
            self.error_expected(&[kw]);
            false
        }
    }

    pub(crate) fn finish_file(mut self) -> (GreenNode, Vec<SyntaxError>) {
        self.pos = self.sig.len();
        self.leading_docs = false;
        self.flush_trivia(false);
        self.builder.finish_node();
        (self.builder.finish(), self.errors)
    }

    // ---- errors and recovery ---------------------------------------------

    pub(crate) fn push_error(&mut self, error: SyntaxError) {
        self.errors.push(error);
    }

    /// Reports L0001 at the current token: "expected X, found Y", with a
    /// machine-applicable keyword fix when the found word is a near miss.
    pub(crate) fn error_expected(&mut self, expected: &[SyntaxKind]) {
        let at = self.current_range();
        let mut error = SyntaxError::new(
            DiagnosticCode::UnexpectedToken,
            at,
            format!(
                "expected {}, found {}",
                describe_set(expected),
                self.found()
            ),
        );
        if self.at(Ident) {
            let words: Vec<&str> = expected
                .iter()
                .filter(|kind| kind.is_keyword())
                .filter_map(|kind| kind.text())
                .collect();
            if let Some(best) = nearest(self.nth_text(0), &words) {
                error = error.with_note(format!("did you mean `{best}`?")).with_fix(
                    at,
                    best,
                    Applicability::MachineApplicable,
                );
            }
        }
        self.push_error(error);
    }

    pub(crate) fn found(&self) -> std::string::String {
        match self.nth(0) {
            Eof => "end of file".to_owned(),
            _ => format!("`{}`", self.nth_text(0)),
        }
    }

    /// Reports L0001 and wraps tokens up to the follow set (`follow`, a
    /// top-level item keyword at column 1, or EOF) in an `Error` node.
    /// Nested delimiters are skipped as a unit.
    pub(crate) fn recover(&mut self, expected: &[SyntaxKind], follow: &[SyntaxKind]) {
        self.error_expected(expected);
        self.skip_to(follow);
    }

    /// Wraps tokens up to the follow set in an `Error` node, always
    /// consuming at least one token unless already at a stop. Callers only
    /// recover before end of file.
    pub(crate) fn skip_to(&mut self, follow: &[SyntaxKind]) {
        if follow.contains(&self.nth(0)) || self.at_item_anchor() {
            return;
        }
        self.start(Error);
        let mut depth = 0usize;
        loop {
            let kind = self.nth(0);
            if kind == Eof || (depth == 0 && follow.contains(&kind)) || self.at_item_anchor() {
                break;
            }
            match kind {
                LBrace | LParen | LBracket => depth += 1,
                RBrace | RParen | RBracket if depth == 0 => {}
                RBrace | RParen | RBracket => depth -= 1,
                _ => {}
            }
            self.bump();
        }
        self.finish();
    }

    /// Wraps the rest of the current line (up to a `follow` token) in an
    /// `Error` node.
    pub(crate) fn skip_line(&mut self, follow: &[SyntaxKind]) {
        if self.at_end() || self.newline_before() || follow.contains(&self.nth(0)) {
            return;
        }
        self.start(Error);
        while !self.at_end() && !self.newline_before() && !follow.contains(&self.nth(0)) {
            self.bump();
        }
        self.finish();
    }

    pub(crate) fn error_count(&self) -> usize {
        self.errors.len()
    }

    /// Wraps the current token in an `Error` node (guarantees progress).
    pub(crate) fn bump_error(&mut self) {
        if !self.at_end() {
            self.start(Error);
            self.bump();
            self.finish();
        }
    }

    /// Reports an unclosed delimiter (L0002) at `opener`, with a
    /// machine-applicable fix inserting `closer` at the end of the line
    /// holding the last consumed token.
    pub(crate) fn unclosed(&mut self, opener: TextRange, closer: &str) {
        let end = self.last_end();
        let line_end = self.text[end..]
            .find('\n')
            .map_or(self.text.len(), |at| end + at);
        let line_end = if self.text[..line_end].ends_with('\r') {
            line_end - 1
        } else {
            line_end
        };
        let open = &self.text[opener];
        self.push_error(
            SyntaxError::new(
                DiagnosticCode::UnterminatedLiteral,
                opener,
                format!("unclosed `{open}`: expected `{closer}`"),
            )
            .with_fix(
                range(line_end, line_end),
                closer,
                Applicability::MachineApplicable,
            ),
        );
    }

    /// Runs `body` one level deeper, or reports and skips when the
    /// nesting limit is reached. Returns false when it refused.
    pub(crate) fn nested(&mut self, body: impl FnOnce(&mut Self)) -> bool {
        if self.depth >= MAX_DEPTH {
            let at = self.current_range();
            self.push_error(SyntaxError::new(
                DiagnosticCode::UnexpectedToken,
                at,
                format!("nesting deeper than {MAX_DEPTH} levels"),
            ));
            self.bump_error();
            return false;
        }
        self.depth += 1;
        body(self);
        self.depth -= 1;
        true
    }
}
