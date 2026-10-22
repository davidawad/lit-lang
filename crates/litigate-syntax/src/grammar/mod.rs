//! Grammar rules (SPEC §3). Each rule either parses its construct or
//! returns `false` without consuming or reporting, so callers own recovery.
mod case;
mod clauses;
mod decls;
mod expr;
mod fields;
mod items;
mod values;

pub(crate) use case::case_file;
pub(crate) use items::lit_file;

use crate::parser::Parser;
use crate::syntax_kind::SyntaxKind::{self, *};

/// Separator discipline of a delimited list.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Sep {
    /// Commas between elements (a trailing comma is allowed).
    Required,
    /// Elements may be juxtaposed or comma-separated.
    Optional,
    /// Elements are juxtaposed; commas are an error.
    Forbidden,
}

/// Parses `open { elem sep } close` after the caller started the node.
/// `elem` returns false (consuming nothing) when it can't start an element;
/// `expected` describes what could.
pub(crate) fn delimited(
    p: &mut Parser<'_>,
    (open, close): (SyntaxKind, SyntaxKind),
    sep: Sep,
    expected: &[SyntaxKind],
    elem: &mut dyn FnMut(&mut Parser<'_>) -> bool,
) {
    let opener = p.current_range();
    if !p.expect(open) {
        return;
    }
    let closer = close.text().unwrap_or("}");
    p.nested(|p| loop {
        let before = p.current_range();
        if p.eat(close) {
            return;
        }
        if p.at_end() {
            p.unclosed(opener, closer);
            return;
        }
        let errors = p.error_count();
        if !elem(p) {
            if p.at_item_anchor() {
                p.unclosed(opener, closer);
                return;
            }
            let mut want = expected.to_vec();
            want.push(close);
            p.recover(&want, &[Comma, close]);
        } else if p.error_count() > errors && p.nth(0) != close {
            p.skip_line(&[Comma, close]);
            p.eat(Comma);
        } else if !separator(p, open, close, sep) {
            p.unclosed(opener, closer);
            return;
        }
        if p.current_range() == before {
            if p.at_item_anchor() {
                p.unclosed(opener, closer);
                return;
            }
            p.bump_error();
        }
    });
}

/// Handles what follows an element. Returns false when a bracket or paren
/// list is missing its separator (treated as an unclosed delimiter).
fn separator(p: &mut Parser<'_>, open: SyntaxKind, close: SyntaxKind, sep: Sep) -> bool {
    if p.at(Comma) {
        if sep == Sep::Forbidden {
            p.error_expected(&[close]);
            p.bump_error();
        } else {
            p.bump();
        }
        return true;
    }
    if sep != Sep::Required || p.at(close) || p.at_end() {
        return true;
    }
    if open != LBrace {
        return false;
    }
    let at = p.last_end();
    let error = crate::error::SyntaxError::new(
        litigate_base::DiagnosticCode::UnexpectedToken,
        p.current_range(),
        format!(
            "expected `,` or `{}`, found {}",
            close.text().unwrap_or("}"),
            p.found()
        ),
    )
    .with_fix(
        crate::error::range(at, at),
        ",",
        litigate_base::Applicability::MachineApplicable,
    );
    p.push_error(error);
    true
}

/// Parses a node of `kind` holding a delimited list.
pub(crate) fn delimited_node(
    p: &mut Parser<'_>,
    kind: SyntaxKind,
    delims: (SyntaxKind, SyntaxKind),
    sep: Sep,
    expected: &[SyntaxKind],
    elem: &mut dyn FnMut(&mut Parser<'_>) -> bool,
) {
    p.start(kind);
    delimited(p, delims, sep, expected, elem);
    p.finish();
}

/// Parses `{ entries }` as a `FieldList`.
pub(crate) fn brace_list(
    p: &mut Parser<'_>,
    sep: Sep,
    expected: &[SyntaxKind],
    elem: &mut dyn FnMut(&mut Parser<'_>) -> bool,
) {
    delimited_node(p, FieldList, (LBrace, RBrace), sep, expected, elem);
}
