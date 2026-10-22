//! Expressions (SPEC §3.3): precedence climbing over
//! `|| < && < comparison (non-associative) < + - < * / % < unary < postfix`.
use super::values::{cite, hole, keyword_literal, reference, token_literal};
use super::{delimited, delimited_node, Sep};
use crate::error::SyntaxError;
use crate::parser::Parser;
use crate::syntax_kind::SyntaxKind::{self, *};
use litigate_base::DiagnosticCode;

/// Tokens that can start an expression (for messages).
pub(crate) const EXPR_START: &[SyntaxKind] = &[Ident, Number, String, Date, LParen];

/// `Expr = MatchExpr | OrExpr`. Returns false when nothing can start one.
pub(crate) fn expr(p: &mut Parser<'_>) -> bool {
    if p.at_kw(KwMatch) && p.nth(1) != LParen {
        match_expr(p);
        return true;
    }
    binary(p, 0)
}

/// Requires an expression, reporting L0001 when none starts here.
pub(crate) fn require_expr(p: &mut Parser<'_>) -> bool {
    expr(p) || {
        p.error_expected(EXPR_START);
        false
    }
}

const LEVELS: [&[SyntaxKind]; 5] = [
    &[OrOr],
    &[AndAnd],
    &[EqEq, NotEq, Less, LessEq, Greater, GreaterEq],
    &[Plus, Minus],
    &[Star, Slash, Percent],
];
const COMPARISON: usize = 2;

fn binary(p: &mut Parser<'_>, level: usize) -> bool {
    let Some(ops) = LEVELS.get(level) else {
        return unary(p);
    };
    let checkpoint = p.checkpoint();
    if !binary(p, level + 1) {
        return false;
    }
    let mut seen = 0usize;
    while ops.contains(&p.nth(0)) {
        if level == COMPARISON && seen > 0 {
            let at = p.current_range();
            p.push_error(SyntaxError::new(
                DiagnosticCode::UnexpectedToken,
                at,
                "comparison operators are non-associative; parenthesize one side",
            ));
        }
        p.start_at(checkpoint, BinaryExpr);
        p.bump();
        let rhs = p.nested(|p| {
            if !binary(p, level + 1) {
                p.error_expected(EXPR_START);
            }
        });
        p.finish();
        seen += 1;
        if !rhs {
            break;
        }
    }
    true
}

fn unary(p: &mut Parser<'_>) -> bool {
    if matches!(p.nth(0), Bang | Minus) {
        p.start(UnaryExpr);
        p.bump();
        p.nested(|p| {
            if !unary(p) {
                p.error_expected(EXPR_START);
            }
        });
        p.finish();
        return true;
    }
    postfix(p)
}

fn postfix(p: &mut Parser<'_>) -> bool {
    let checkpoint = p.checkpoint();
    let was_ref = p.at(Ident) && p.nth(1) != LParen;
    if !primary(p) {
        return false;
    }
    if was_ref {
        return true;
    }
    while p.at(Dot) && p.nth(1) == Ident {
        p.start_at(checkpoint, FieldExpr);
        p.bump();
        p.bump();
        p.finish();
    }
    true
}

/// `Primary = Literal | Hole | Cite | Call | Ref | Record | List | "(" Expr ")"`.
pub(crate) fn primary(p: &mut Parser<'_>) -> bool {
    match p.nth(0) {
        Number | Money | Duration | String | Date => token_literal(p, p.nth(0)),
        Question => hole(p),
        KwCite => cite(p),
        LParen => paren(p),
        LBrace => record_expr(p),
        LBracket => list_expr(p),
        Ident if p.at_any_kw(&[KwTrue, KwFalse, KwUnit]).is_some() => {
            keyword_literal(p, &[KwTrue, KwFalse, KwUnit])
        }
        Ident => ref_or_call(p),
        _ => false,
    }
}

fn paren(p: &mut Parser<'_>) -> bool {
    p.start(ParenExpr);
    let opener = p.current_range();
    p.bump();
    p.nested(|p| {
        require_expr(p);
    });
    if !p.eat(RParen) {
        p.unclosed(opener, ")");
    }
    p.finish();
    true
}

fn ref_or_call(p: &mut Parser<'_>) -> bool {
    let checkpoint = p.checkpoint();
    reference(p);
    if p.at(LParen) {
        p.start_at(checkpoint, CallExpr);
        arg_list(p);
        p.finish();
    } else {
        p.start_at(checkpoint, RefExpr);
        p.finish();
    }
    true
}

/// `"(" [Expr {"," Expr} [","]] ")"` as an `ArgList`.
pub(crate) fn arg_list(p: &mut Parser<'_>) {
    p.nested(|p| {
        delimited_node(
            p,
            ArgList,
            (LParen, RParen),
            Sep::Required,
            EXPR_START,
            &mut expr,
        );
    });
}

fn record_expr(p: &mut Parser<'_>) -> bool {
    p.nested(|p| {
        super::values::record(p, super::values::Key::Ident, expr);
    });
    true
}

fn list_expr(p: &mut Parser<'_>) -> bool {
    p.start(ListExpr);
    p.nested(|p| {
        delimited(
            p,
            (LBracket, RBracket),
            Sep::Required,
            EXPR_START,
            &mut expr,
        );
    });
    p.finish();
    true
}

/// `"match" OrExpr "{" {Pattern "=>" Expr Sep} "}"`.
fn match_expr(p: &mut Parser<'_>) {
    p.start(MatchExpr);
    p.bump_as(KwMatch);
    if !binary(p, 0) {
        p.error_expected(EXPR_START);
    }
    p.nested(|p| {
        delimited(
            p,
            (LBrace, RBrace),
            Sep::Optional,
            &[Ident, Underscore],
            &mut match_arm,
        );
    });
    p.finish();
}

fn match_arm(p: &mut Parser<'_>) -> bool {
    if !(p.at(Ident) || p.at(Underscore)) {
        return false;
    }
    p.start(MatchArm);
    if !p.eat(Underscore) {
        reference(p);
    }
    if p.expect(FatArrow) {
        p.nested(|p| {
            require_expr(p);
        });
    }
    p.finish();
    true
}
