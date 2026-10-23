//! Atoms shared by declarations, fields, and case blocks: refs, id lists,
//! literals, holes, citations, intervals, records, JSON values, and types.
use super::{delimited_node, expr, Sep};
use crate::error::SyntaxError;
use crate::parser::Parser;
use crate::syntax_kind::SyntaxKind::{self, *};
use litigate_base::DiagnosticCode;

/// A value parser: parses one construct or returns false untouched.
pub(crate) type Value = fn(&mut Parser<'_>) -> bool;

/// Runs `value`, reporting L0001 with `expected` when it can't start.
pub(crate) fn require(p: &mut Parser<'_>, value: Value, expected: &[SyntaxKind]) -> bool {
    value(p) || {
        p.error_expected(expected);
        false
    }
}

/// `Ref = Ident ["@" Ident] {("::" | ".") Ident} ["@" Date]`.
pub(crate) fn reference(p: &mut Parser<'_>) -> bool {
    if !p.at(Ident) {
        return false;
    }
    p.start(Ref);
    p.bump();
    if p.at(At) && p.nth(1) == Ident {
        p.bump();
        p.bump();
    }
    while matches!(p.nth(0), ColonColon | Dot) && p.nth(1) == Ident {
        p.bump();
        p.bump();
    }
    if p.at(At) && p.nth(1) == Date {
        p.bump();
        p.bump();
    }
    p.finish();
    true
}

/// `IdList = "[" [Ref {"," Ref} [","]] "]"`.
pub(crate) fn id_list(p: &mut Parser<'_>) -> bool {
    if !p.at(LBracket) {
        return false;
    }
    delimited_node(
        p,
        IdList,
        (LBracket, RBracket),
        Sep::Required,
        &[Ident],
        &mut reference,
    );
    true
}

/// `replaces [Ref | String, ...]`: a string names an anonymous edge by its
/// derived id (`"pack::from->to#n"`), which has no `Ref` spelling.
pub(crate) fn replaces_list(p: &mut Parser<'_>) -> bool {
    if !p.at(LBracket) {
        return false;
    }
    delimited_node(
        p,
        IdList,
        (LBracket, RBracket),
        Sep::Required,
        &[Ident, String],
        &mut |p| reference(p) || string(p),
    );
    true
}

/// `Date | Hole`: a date, or a hole whose reason keeps text that is not a
/// whole date (`?("2026-06")`).
pub(crate) fn date_or_hole(p: &mut Parser<'_>) -> bool {
    date(p) || hole(p)
}

/// A `Literal` node holding one token of `kind`.
pub(crate) fn token_literal(p: &mut Parser<'_>, kind: SyntaxKind) -> bool {
    if !p.at(kind) {
        return false;
    }
    p.start(Literal);
    p.bump();
    p.finish();
    true
}

/// A `Literal` node holding one keyword literal (`true`, `unset`, ...).
pub(crate) fn keyword_literal(p: &mut Parser<'_>, keywords: &[SyntaxKind]) -> bool {
    let Some(kw) = p.at_any_kw(keywords) else {
        return false;
    };
    p.start(Literal);
    p.bump_as(kw);
    p.finish();
    true
}

pub(crate) fn string(p: &mut Parser<'_>) -> bool {
    token_literal(p, String)
}

pub(crate) fn number(p: &mut Parser<'_>) -> bool {
    token_literal(p, Number)
}

/// A duration whose day amount must be whole (L0005 otherwise).
pub(crate) fn duration(p: &mut Parser<'_>) -> bool {
    whole_days(p);
    token_literal(p, Duration)
}

/// A `duration` field amount: an elapsed-time estimate, so decimal day
/// amounts are allowed.
pub(crate) fn duration_estimate(p: &mut Parser<'_>) -> bool {
    token_literal(p, Duration)
}

/// Reports L0005 when the current token is a day-unit duration with a
/// decimal amount (`1.5.days`).
pub(crate) fn whole_days(p: &mut Parser<'_>) {
    if !p.at(Duration) {
        return;
    }
    let text = p.nth_text(0);
    let (number, unit) = text.rsplit_once('.').unwrap_or((text, ""));
    if unit != "hours" && number.contains('.') {
        let error = SyntaxError::new(
            DiagnosticCode::InvalidNumber,
            p.current_range(),
            format!("day amounts must be integers, found `{number}`"),
        )
        .with_note("only `duration` estimates may use decimal days");
        p.push_error(error);
    }
}

pub(crate) fn date(p: &mut Parser<'_>) -> bool {
    token_literal(p, Date)
}

pub(crate) fn boolean(p: &mut Parser<'_>) -> bool {
    keyword_literal(p, &[KwTrue, KwFalse])
}

/// `Hole = "?" ["(" String ")"]`.
pub(crate) fn hole(p: &mut Parser<'_>) -> bool {
    if !p.at(Question) {
        return false;
    }
    p.start(HoleExpr);
    p.bump();
    if p.at(LParen) {
        let opener = p.current_range();
        p.bump();
        if !string(p) {
            p.error_expected(&[String]);
        }
        if !p.eat(RParen) {
            p.unclosed(opener, ")");
        }
    }
    p.finish();
    true
}

/// `Cite = "cite" (CiteText | Hole)`. The hole form keeps citation text
/// that has no `CiteText` spelling (a `,`, `{`, `}`, or `//` inside it) as
/// the hole's reason.
pub(crate) fn cite(p: &mut Parser<'_>) -> bool {
    if !p.at(KwCite) {
        return false;
    }
    p.start(CiteExpr);
    p.bump();
    if !p.eat(CiteText) {
        hole(p);
    }
    p.finish();
    true
}

/// `["-"] literal(kind)`: a `UnaryExpr` when negated.
pub(crate) fn signed(p: &mut Parser<'_>, kind: SyntaxKind) -> bool {
    if p.at(Minus) && p.nth(1) == kind {
        p.start(UnaryExpr);
        p.bump();
        token_literal(p, kind);
        p.finish();
        return true;
    }
    token_literal(p, kind)
}

pub(crate) fn signed_number(p: &mut Parser<'_>) -> bool {
    signed(p, Number)
}

pub(crate) fn signed_money(p: &mut Parser<'_>) -> bool {
    signed(p, Money)
}

pub(crate) fn money_or_hole(p: &mut Parser<'_>) -> bool {
    signed_money(p) || hole(p)
}

/// `ProbValue = Number ["/" Number] | Hole | "(" Expr ")"`.
pub(crate) fn prob_value(p: &mut Parser<'_>) -> bool {
    if p.at(Number) && p.nth(1) == Slash {
        p.start(Ratio);
        p.bump();
        p.bump();
        if !p.eat(Number) {
            p.error_expected(&[Number]);
        }
        p.finish();
        return true;
    }
    number(p) || hole(p) || (p.at(LParen) && expr::primary(p))
}

/// `Interval = "[" Date "," (Date | "..") ")"`.
pub(crate) fn interval(p: &mut Parser<'_>) -> bool {
    if !p.at(LBracket) {
        return false;
    }
    p.start(Interval);
    let opener = p.current_range();
    p.bump();
    let complete = p.expect(Date) && p.expect(Comma) && (p.eat(DotDot) || p.expect(Date));
    if complete && p.at(RBracket) {
        let at = p.current_range();
        p.push_error(
            crate::error::SyntaxError::new(
                litigate_base::DiagnosticCode::UnexpectedToken,
                at,
                "intervals are half-open: close with `)`, not `]`",
            )
            .with_fix(at, ")", litigate_base::Applicability::MachineApplicable),
        );
        p.bump_error();
    } else if complete && !p.eat(RParen) {
        p.unclosed(opener, ")");
    }
    p.finish();
    true
}

/// How a record field's key is written.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Key {
    /// A plain identifier.
    Ident,
    /// A (possibly qualified) `Ref` node.
    Ref,
}

/// `{ key ":" value Sep }` as a `RecordExpr`.
pub(crate) fn record(p: &mut Parser<'_>, key: Key, value: Value) -> bool {
    if !p.at(LBrace) {
        return false;
    }
    let mut field = |p: &mut Parser<'_>| record_field(p, key, value);
    delimited_node(
        p,
        RecordExpr,
        (LBrace, RBrace),
        Sep::Optional,
        &[Ident],
        &mut field,
    );
    true
}

fn record_field(p: &mut Parser<'_>, key: Key, value: Value) -> bool {
    if !p.at(Ident) {
        return false;
    }
    p.start(RecordField);
    if key == Key::Ref {
        reference(p);
    } else {
        p.bump();
    }
    if p.expect(Colon) && !value(p) {
        p.error_expected(&[Ident]);
    }
    p.finish();
    true
}

/// `Type = Ident ["<" Type {"," Type} ">"] | RefinedType`.
pub(crate) fn type_expr(p: &mut Parser<'_>) -> bool {
    if p.at(LBrace) {
        refined_type(p);
        return true;
    }
    if !p.at(Ident) {
        return false;
    }
    p.start(TypeExpr);
    p.bump();
    if p.at(Less) {
        p.nested(|p| {
            delimited_node(
                p,
                TypeArgs,
                (Less, Greater),
                Sep::Required,
                &[Ident],
                &mut type_expr,
            );
        });
    }
    p.finish();
    true
}

/// `RefinedType = "{" Ident ":" Type "|" Expr "}"` (SPEC §3.5): parsed in
/// full so the rest of the file parses, then rejected as L0009 because
/// refinements belong to the SMT tier.
fn refined_type(p: &mut Parser<'_>) {
    let opener = p.current_range();
    p.start(RefinedType);
    p.bump();
    p.nested(|p| {
        let complete = p.expect(Ident)
            && p.expect(Colon)
            && require(p, type_expr, &[Ident, LBrace])
            && p.expect(Pipe)
            && expr::require_expr(p);
        if complete && !p.at(RBrace) && !p.at_end() && !p.at_item_anchor() {
            p.error_expected(&[RBrace]);
        }
        p.skip_to(&[RBrace]);
        if !p.eat(RBrace) {
            p.unclosed(opener, "}");
        }
    });
    p.finish();
    let end = p.last_end().max(usize::from(opener.end()));
    p.push_error(SyntaxError::new(
        DiagnosticCode::ReservedRefinement,
        crate::error::range(usize::from(opener.start()), end),
        format!(
            "{}: a refinement type `{{x: T | p}}` is not yet part of the language; \
             write the base type",
            crate::lexer::RESERVED_REFINEMENT
        ),
    ));
}

/// `WhereClause = "where" Expr` (SPEC §3.5) after a parameter's or a type
/// declaration's type. The lexer already reports `where` as L0009; the
/// slot parses the predicate so the rest of the file parses.
pub(crate) fn where_clause(p: &mut Parser<'_>) {
    if !p.at_kw(KwWhere) {
        return;
    }
    p.start(WhereClause);
    p.bump_as(KwWhere);
    expr::require_expr(p);
    p.finish();
}

/// `CallExpr = Ref "(" args ")"` where the parenthesis is required.
pub(crate) fn call(p: &mut Parser<'_>) -> bool {
    if !p.at(Ident) {
        return false;
    }
    let checkpoint = p.checkpoint();
    reference(p);
    p.start_at(checkpoint, CallExpr);
    if p.at(LParen) {
        expr::arg_list(p);
    } else {
        p.error_expected(&[LParen]);
    }
    p.finish();
    true
}
