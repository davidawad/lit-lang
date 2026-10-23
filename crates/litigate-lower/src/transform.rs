//! A typed `payoff_transform` as the litgraph expression the engine
//! evaluates (SPEC §8.4).
use crate::diag::Sink;
use litigate_base::DiagnosticCode as C;
use litigate_sema::hir::{BinaryOp, Expr, ExprKind, UnaryOp, Value};

/// Precedence levels of litgraph's grammar: a sum, a product, a negation,
/// an atom.
const SUM: u8 = 1;
const PRODUCT: u8 = 2;
const NEGATION: u8 = 3;
const ATOM: u8 = 4;

/// `e` in litgraph syntax: money in major units, numbers as binary64,
/// `payoff` and `stake` by name, and `+ - * /` and unary `-` parenthesized
/// by precedence (operators are left-associative). Anything else has no
/// engine spelling (L6003).
pub(crate) fn engine_text(sink: &mut Sink, e: &Expr) -> Option<String> {
    spelled(sink, e).map(|(text, _)| text)
}

fn spelled(sink: &mut Sink, e: &Expr) -> Option<(String, u8)> {
    let span = e.span.as_ref();
    let literal = |n: f64| {
        let level = if n < 0.0 { NEGATION } else { ATOM };
        (n.to_string(), level)
    };
    match &e.kind {
        ExprKind::Literal {
            value: Value::Money(m),
        } => sink.money(m, span).map(literal),
        ExprKind::Literal {
            value: Value::Number(r),
        } => sink.number(*r, span).map(literal),
        ExprKind::Ref { target } => Some((target.clone(), ATOM)),
        ExprKind::Unary {
            op: UnaryOp::Neg,
            operand,
        } => {
            let inner = spelled(sink, operand)?;
            Some((format!("-{}", wrap(inner, NEGATION)), NEGATION))
        }
        ExprKind::Binary { op, lhs, rhs } => {
            let (symbol, level) = match op {
                BinaryOp::Add => ("+", SUM),
                BinaryOp::Sub => ("-", SUM),
                BinaryOp::Mul => ("*", PRODUCT),
                BinaryOp::Div => ("/", PRODUCT),
                _ => return unsupported(sink, e),
            };
            let (l, r) = (spelled(sink, lhs)?, spelled(sink, rhs)?);
            // Left-associative: a right operand at the same level keeps
            // its parentheses.
            let text = format!("{} {symbol} {}", wrap(l, level), wrap(r, level + 1));
            Some((text, level))
        }
        _ => unsupported(sink, e),
    }
}

/// `text` parenthesized when its level binds looser than `least`.
fn wrap((text, level): (String, u8), least: u8) -> String {
    if level < least {
        format!("({text})")
    } else {
        text
    }
}

fn unsupported(sink: &mut Sink, e: &Expr) -> Option<(String, u8)> {
    sink.error(
        C::InvalidEngineExpression,
        e.span.as_ref(),
        "a `payoff_transform` lowers only `payoff`, `stake`, money and number literals, \
         `+ - * /` and unary `-`",
    );
    None
}
