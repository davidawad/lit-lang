//! A legacy `payoff_transform` string as the typed expression a links
//! package writes (SPEC §8.4 in reverse): `"stake - payoff"` becomes
//! `(stake - payoff)` and `"1000000 - payoff"` becomes
//! `(1_000_000.usd - payoff)`, a number added to or subtracted from money
//! being money.
use crate::issues::Issues;
use crate::text;
use litgraph::expr::{self, Expr};

/// Precedence levels shared by litgraph and `.lit` expressions.
const SUM: u8 = 1;
const PRODUCT: u8 = 2;
const NEGATION: u8 = 3;
const ATOM: u8 = 4;

/// The `payoff_transform` field value for `source`, or `None` after
/// reporting at `pointer` what has no spelling.
pub(crate) fn transform(source: &str, pointer: &str, issues: &mut Issues) -> Option<String> {
    let spelled = match expr::parse(source) {
        Ok(e) => spell(&e, true),
        Err(e) => {
            issues.report(pointer, format!("`payoff_transform` does not parse: {e}"));
            return None;
        }
    };
    if spelled.is_none() {
        issues.report(
            pointer,
            "a `payoff_transform` reads `payoff` and `stake`, with number literals, \
             `+ - * /` and unary `-`; this one has no `.lit` spelling",
        );
    }
    spelled.map(|(text, _)| format!("({text})"))
}

/// `e` spelled as money when `money`, else as a number.
fn spell(e: &Expr, money: bool) -> Option<(String, u8)> {
    match e {
        Expr::Num(n) if money => Some((text::money(*n)?, ATOM)),
        Expr::Num(n) => Some((text::number(*n)?, ATOM)),
        Expr::Var(v) if money && (v == "payoff" || v == "stake") => Some((v.clone(), ATOM)),
        Expr::Unary('-', x) => {
            let inner = spell(x, money)?;
            Some((format!("-{}", wrap(inner, NEGATION)), NEGATION))
        }
        Expr::Binary(op @ ("+" | "-"), l, r) => binary(op, (l, money), (r, money), SUM),
        // Money times a number, either way round; money over a number.
        Expr::Binary("*", l, r) if money && reads_money(l) => {
            binary("*", (l, true), (r, false), PRODUCT)
        }
        Expr::Binary("*", l, r) => binary("*", (l, false), (r, money), PRODUCT),
        Expr::Binary("/", l, r) => binary("/", (l, money), (r, false), PRODUCT),
        _ => None,
    }
}

fn binary(
    op: &str,
    (l, lm): (&Expr, bool),
    (r, rm): (&Expr, bool),
    level: u8,
) -> Option<(String, u8)> {
    let (l, r) = (spell(l, lm)?, spell(r, rm)?);
    Some((
        format!("{} {op} {}", wrap(l, level), wrap(r, level + 1)),
        level,
    ))
}

/// Whether `e` reads a variable (so it is the money operand of a product).
fn reads_money(e: &Expr) -> bool {
    !e.vars().is_empty()
}

/// `text` parenthesized when its level binds looser than `least`.
fn wrap((text, level): (String, u8), least: u8) -> String {
    if level < least {
        format!("({text})")
    } else {
        text
    }
}
