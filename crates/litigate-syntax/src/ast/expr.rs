//! Accessors for references, literals, and expressions.
use super::{
    child, child_after, children, token, tokens, unescape, AstNode, CallExpr, Expr, Hole, Literal,
    MatchArm, Ratio, RecordExpr, RecordField, Ref, Token,
};
use crate::syntax_kind::SyntaxKind;

/// The structural parts of a reference.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RefParts {
    /// First identifier (a pack id when qualified with `::`).
    pub head: String,
    /// Instance after `head@` (`base@origin::x` gives `origin`).
    pub instance: Option<String>,
    /// Remaining `(separator, identifier)` segments; separator is `.` or `::`.
    pub segments: Vec<(String, String)>,
    /// Version date after a trailing `@`.
    pub version: Option<String>,
}

impl Ref {
    /// The reference text without trivia (`base@origin::entry`).
    #[must_use]
    pub fn text(&self) -> String {
        tokens(self.syntax())
            .filter(|token| !crate::parser::is_trivia(token.kind()))
            .map(|token| token.text().to_owned())
            .collect()
    }

    /// Identifier tokens in order (head first).
    #[must_use]
    pub fn idents(&self) -> Vec<Token> {
        tokens(self.syntax())
            .filter(|token| token.kind() == SyntaxKind::Ident)
            .collect()
    }

    /// True when the reference uses `::` (pack- or instance-qualified).
    #[must_use]
    pub fn is_qualified(&self) -> bool {
        token(self.syntax(), SyntaxKind::ColonColon).is_some()
    }

    /// Splits the reference into head, instance, segments, and version.
    #[must_use]
    pub fn parts(&self) -> RefParts {
        let mut parts = RefParts {
            head: String::new(),
            instance: None,
            segments: Vec::new(),
            version: None,
        };
        let mut separator: Option<String> = None;
        let mut after_at = false;
        for token in tokens(self.syntax()) {
            let text = token.text().to_owned();
            match token.kind() {
                SyntaxKind::Ident if parts.head.is_empty() => parts.head = text,
                SyntaxKind::Ident if after_at => parts.instance = Some(text),
                SyntaxKind::Ident => {
                    parts
                        .segments
                        .push((separator.take().unwrap_or_default(), text));
                }
                SyntaxKind::Date => parts.version = Some(text),
                SyntaxKind::Dot | SyntaxKind::ColonColon => separator = Some(text),
                _ => {}
            }
            after_at = token.kind() == SyntaxKind::At;
        }
        parts
    }
}

impl Literal {
    /// The literal token (number, money, duration, string, date, or a
    /// keyword literal such as `true`).
    #[must_use]
    pub fn token(&self) -> Option<Token> {
        tokens(self.syntax()).find(|token| !crate::parser::is_trivia(token.kind()))
    }

    /// Decoded value of a string literal.
    #[must_use]
    pub fn string_value(&self) -> Option<String> {
        self.token()
            .filter(|token| token.kind() == SyntaxKind::String)
            .map(|token| unescape(token.text()))
    }
}

impl Hole {
    /// The decoded reason of `?("reason")`.
    #[must_use]
    pub fn reason(&self) -> Option<String> {
        child::<Literal>(self.syntax()).and_then(|literal| literal.string_value())
    }
}

impl Ratio {
    /// Numerator token.
    #[must_use]
    pub fn numerator(&self) -> Option<Token> {
        token(self.syntax(), SyntaxKind::Number)
    }

    /// Denominator token.
    #[must_use]
    pub fn denominator(&self) -> Option<Token> {
        tokens(self.syntax())
            .filter(|token| token.kind() == SyntaxKind::Number)
            .nth(1)
    }
}

impl CallExpr {
    /// The called function or event constructor.
    #[must_use]
    pub fn callee(&self) -> Option<Ref> {
        child(self.syntax())
    }

    /// Arguments in order.
    #[must_use]
    pub fn args(&self) -> Vec<Expr> {
        self.syntax()
            .children()
            .find(|node| node.kind() == SyntaxKind::ArgList)
            .map_or_else(Vec::new, |list| children(&list))
    }
}

impl RecordExpr {
    /// Fields in order.
    #[must_use]
    pub fn fields(&self) -> Vec<RecordField> {
        children(self.syntax())
    }
}

impl RecordField {
    /// The key token (identifier, keyword, or JSON string); `None` when the
    /// key is a qualified reference (see [`RecordField::key_ref`]).
    #[must_use]
    pub fn key(&self) -> Option<Token> {
        tokens(self.syntax()).find(|token| !crate::parser::is_trivia(token.kind()))
    }

    /// Decoded key text: identifier text, unescaped string, or ref text.
    #[must_use]
    pub fn key_text(&self) -> Option<String> {
        if let Some(reference) = self.key_ref() {
            return Some(reference.text());
        }
        self.key().map(|token| match token.kind() {
            SyntaxKind::String => unescape(token.text()),
            _ => token.text().to_owned(),
        })
    }

    /// The key when it is a (possibly qualified) reference (case blocks).
    #[must_use]
    pub fn key_ref(&self) -> Option<Ref> {
        self.syntax()
            .first_child_or_token()
            .and_then(rowan::NodeOrToken::into_node)
            .and_then(Ref::cast)
    }

    /// The value node after `:`.
    #[must_use]
    pub fn value(&self) -> Option<crate::SyntaxNode> {
        self.syntax()
            .children_with_tokens()
            .skip_while(|element| element.kind() != SyntaxKind::Colon)
            .find_map(rowan::NodeOrToken::into_node)
    }

    /// The value as an expression.
    #[must_use]
    pub fn value_expr(&self) -> Option<Expr> {
        self.value().and_then(Expr::cast)
    }

    /// The value as a reference.
    #[must_use]
    pub fn value_ref(&self) -> Option<Ref> {
        self.value().and_then(Ref::cast)
    }
}

impl MatchArm {
    /// The pattern reference; `None` for the `_` wildcard.
    #[must_use]
    pub fn pattern(&self) -> Option<Ref> {
        child(self.syntax())
    }

    /// The arm value.
    #[must_use]
    pub fn value(&self) -> Option<Expr> {
        child_after(self.syntax(), SyntaxKind::FatArrow)
    }
}

impl Expr {
    /// Kind of expression node.
    #[must_use]
    pub fn kind(&self) -> SyntaxKind {
        self.syntax().kind()
    }

    /// Sub-expressions (operands, arguments, elements, arm values).
    #[must_use]
    pub fn operands(&self) -> Vec<Expr> {
        match self.kind() {
            SyntaxKind::CallExpr => {
                CallExpr::cast(self.syntax().clone()).map_or_else(Vec::new, |call| call.args())
            }
            _ => children(self.syntax()),
        }
    }

    /// Operator token of a unary or binary expression.
    #[must_use]
    pub fn operator(&self) -> Option<Token> {
        match self.kind() {
            SyntaxKind::UnaryExpr | SyntaxKind::BinaryExpr => {
                tokens(self.syntax()).find(|token| !crate::parser::is_trivia(token.kind()))
            }
            _ => None,
        }
    }

    /// Reference of a `RefExpr`.
    #[must_use]
    pub fn reference(&self) -> Option<Ref> {
        (self.kind() == SyntaxKind::RefExpr)
            .then(|| child(self.syntax()))
            .flatten()
    }

    /// Projected field name of a `FieldExpr`.
    #[must_use]
    pub fn field_name(&self) -> Option<Token> {
        (self.kind() == SyntaxKind::FieldExpr)
            .then(|| token(self.syntax(), SyntaxKind::Ident))
            .flatten()
    }

    /// Arms of a `MatchExpr`.
    #[must_use]
    pub fn arms(&self) -> Vec<MatchArm> {
        children(self.syntax())
    }
}
