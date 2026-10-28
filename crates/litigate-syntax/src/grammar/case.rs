//! `.case` files (SPEC §3.2): items in a fixed order.
use super::fields::{field_list, stake_value, SOURCE_FIELDS};
use super::values::{
    call, cite, date, number, record, reference, require, signed_money, string, Key,
};
use super::{delimited, Sep};
use crate::error::SyntaxError;
use crate::parser::{Parser, CASE_ITEM_KEYWORDS};
use crate::syntax_kind::SyntaxKind::{self, *};
use litigate_base::DiagnosticCode;

/// `(keyword, node kind, repeatable)` in the required §3.2 order.
const ORDER: &[(SyntaxKind, SyntaxKind, bool)] = &[
    (KwForum, ForumDecl, false),
    (KwGoverningLaw, GoverningLawDecl, false),
    (KwUses, UsesDecl, true),
    (KwAsOf, AsOfDecl, false),
    (KwKnownAt, KnownAtDecl, false),
    (KwSummary, SummaryDecl, false),
    (KwNotes, NotesDecl, false),
    (KwSource, SourceDecl, true),
    (KwStart, StartDecl, false),
    (KwPerspective, PerspectiveDecl, false),
    (KwParties, PartiesBlock, false),
    (KwEvents, EventsBlock, false),
    (KwFacts, FactsBlock, false),
    (KwStakes, StakesBlock, false),
    (KwBeliefs, BeliefsBlock, false),
    (KwOpponents, OpponentsBlock, false),
    (KwAnalysis, AnalysisBlock, false),
];

const REQUIRED: [SyntaxKind; 2] = [KwAsOf, KwSummary];

/// `CaseFile = "case" Ident ( ForumDecl [GoverningLawDecl] {UsesDecl}
/// | UsesDecl {UsesDecl} ) "as_of" Date ...`.
pub(crate) fn case_file(p: &mut Parser<'_>) {
    p.start_root(CaseFile);
    super::edition(p);
    let header = p.current_range();
    if p.eat_kw(KwCase) {
        p.expect(Ident);
    } else {
        p.error_expected(&[KwCase]);
    }
    let mut next = 0usize;
    let mut seen: Vec<SyntaxKind> = Vec::new();
    while !p.at_end() {
        let before = p.current_range();
        let found = ORDER
            .iter()
            .enumerate()
            .find(|(_, (kw, _, _))| p.at_kw(*kw));
        match found {
            Some((index, (kw, kind, repeatable))) => {
                let repeated = index + 1 == next && !repeatable;
                if index + 1 < next || repeated {
                    order_error(p, *kw, repeated);
                }
                next = next.max(index + 1);
                seen.push(*kw);
                case_item(p, *kw, *kind);
                if !(p.at_end() || p.at_any_kw(CASE_ITEM_KEYWORDS).is_some()) {
                    p.recover(CASE_ITEM_KEYWORDS, &[]);
                }
            }
            None => p.recover(CASE_ITEM_KEYWORDS, &[]),
        }
        if p.current_range() == before {
            p.bump_error();
        }
    }
    // A matter names its packages in `uses`, or names its forum.
    let packages =
        (!seen.contains(&KwUses) && !seen.contains(&KwForum)).then(|| "`uses`".to_owned());
    let missing: Vec<std::string::String> = packages
        .into_iter()
        .chain(
            REQUIRED
                .iter()
                .filter(|kw| !seen.contains(kw))
                .filter_map(|kw| kw.text().map(|text| format!("`{text}`"))),
        )
        .collect();
    if !missing.is_empty() {
        p.push_error(SyntaxError::new(
            DiagnosticCode::UnexpectedToken,
            header,
            format!("case is missing required {}", missing.join(", ")),
        ));
    }
}

fn order_error(p: &mut Parser<'_>, kw: SyntaxKind, repeated: bool) {
    let text = kw.text().unwrap_or_default();
    let message = if repeated {
        format!("`{text}` may appear only once")
    } else {
        format!(
            "`{text}` is out of order: case items go forum, governing_law, uses, as_of, known_at, summary, notes, \
             source, start, perspective, parties, events, facts, stakes, beliefs, opponents, analysis"
        )
    };
    let at = p.current_range();
    p.push_error(SyntaxError::new(
        DiagnosticCode::UnexpectedToken,
        at,
        message,
    ));
}

fn case_item(p: &mut Parser<'_>, kw: SyntaxKind, kind: SyntaxKind) {
    p.start(kind);
    p.bump_as(kw);
    match kw {
        KwUses => uses(p),
        KwForum | KwGoverningLaw => {
            if p.at(Ident) {
                use_item(p);
            } else {
                p.error_expected(&[Ident]);
            }
        }
        KwAsOf | KwKnownAt => {
            require(p, date, &[Date]);
        }
        KwSummary | KwNotes => {
            require(p, string, &[String]);
        }
        KwSource => {
            if require(p, string, &[String]) {
                field_list(p, SOURCE_FIELDS);
            }
        }
        KwStart => {
            require(p, reference, &[Ident]);
        }
        KwPerspective => {
            let roles = |p: &mut Parser<'_>| record(p, Key::Ident, reference);
            if !(reference(p) || roles(p)) {
                p.error_expected(&[Ident, LBrace]);
            }
        }
        KwParties => block(p, Sep::Optional, &mut party),
        KwEvents => block(p, Sep::Forbidden, &mut event),
        KwFacts => {
            require(p, |p| record(p, Key::Ref, reference), &[LBrace]);
        }
        KwStakes => {
            require(p, |p| record(p, Key::Ref, stake_value), &[LBrace]);
        }
        KwBeliefs => block(p, Sep::Optional, &mut belief),
        KwOpponents => block(p, Sep::Optional, &mut opponent),
        _ => {
            require(p, |p| record(p, Key::Ident, super::expr::expr), &[LBrace]);
        }
    }
    p.finish();
}

fn block(p: &mut Parser<'_>, sep: Sep, elem: &mut dyn FnMut(&mut Parser<'_>) -> bool) {
    if p.at(LBrace) {
        delimited(p, (LBrace, RBrace), sep, &[Ident], elem);
    } else {
        p.error_expected(&[LBrace]);
    }
}

/// `uses pkg ^1.0.0 {, pkg req}`.
fn uses(p: &mut Parser<'_>) {
    loop {
        if !p.at(Ident) {
            p.error_expected(&[Ident]);
            return;
        }
        use_item(p);
        if !p.eat(Comma) {
            return;
        }
    }
}

/// One `pkg version-req` pin, at its name.
fn use_item(p: &mut Parser<'_>) {
    p.start(UseItem);
    p.bump();
    version_req(p);
    p.finish();
}

/// `("^" | "~" | ">=" | "=") MAJOR.MINOR.PATCH[-pre]`, tokens adjacent.
fn version_req(p: &mut Parser<'_>) {
    p.start(VersionReq);
    if matches!(p.nth(0), Caret | Tilde | GreaterEq | Eq) {
        p.bump();
        version(p);
    } else {
        p.error_expected(&[Caret, Tilde, GreaterEq, Eq]);
    }
    p.finish();
}

fn version(p: &mut Parser<'_>) {
    p.start(Version);
    let start = p.current_range();
    let mut ok =
        adjacent_bump(p, Number) && is_major_minor(p.text_between(start.start(), start.end()));
    ok = ok && adjacent_bump(p, Dot) && adjacent_bump(p, Number);
    if ok && p.at(Minus) && p.adjacent() {
        p.bump();
        ok = adjacent_bump(p, Ident) || adjacent_bump(p, Number);
        while ok && p.at(Dot) && p.adjacent() {
            p.bump();
            ok = adjacent_bump(p, Ident) || adjacent_bump(p, Number);
        }
    }
    if !ok {
        let at = p.current_range();
        p.push_error(SyntaxError::new(
            DiagnosticCode::UnexpectedToken,
            at,
            format!(
                "expected a version `MAJOR.MINOR.PATCH[-pre]` with no spaces, found {}",
                p.found()
            ),
        ));
    }
    p.finish();
}

fn adjacent_bump(p: &mut Parser<'_>, kind: SyntaxKind) -> bool {
    let ok = p.at(kind) && p.adjacent();
    if ok {
        p.bump();
    }
    ok
}

/// `text` is a `Number` token (`[0-9][0-9_]*(.[0-9][0-9_]*)?`), so its
/// parts are never empty: `MAJOR.MINOR` when both are plain digits.
fn is_major_minor(text: &str) -> bool {
    let mut parts = text.split('.');
    let digits =
        |part: Option<&str>| part.is_some_and(|part| part.bytes().all(|b| b.is_ascii_digit()));
    digits(parts.next()) && digits(parts.next()) && parts.next().is_none()
}

/// `id ":" Actor EntityType`.
fn party(p: &mut Parser<'_>) -> bool {
    if !p.at(Ident) {
        return false;
    }
    p.start(PartyDecl);
    p.bump();
    if p.expect(Colon) && require(p, reference, &[Ident]) {
        require(p, reference, &[Ident]);
    }
    p.finish();
    true
}

/// `[Ident ":"] Date ["docket" Number] CallExpr ("by" | "on") Ident [Cite]`.
fn event(p: &mut Parser<'_>) -> bool {
    let id = p.at(Ident) && p.nth(1) == Colon;
    if !(id || p.at(Date)) {
        return false;
    }
    p.start(CaseEvent);
    if id {
        p.bump();
        p.bump();
    }
    if !require(p, date, &[Date]) {
        p.finish();
        return true;
    }
    if p.at_kw(KwDocket) && p.nth(1) == Number {
        p.bump_as(KwDocket);
        p.bump();
    }
    if require(p, call, &[Ident]) {
        match p.at_any_kw(&[KwBy, KwOn]) {
            Some(kw) => {
                p.bump_as(kw);
                require(p, reference, &[Ident]);
                cite(p);
            }
            None => p.error_expected(&[KwBy, KwOn]),
        }
    }
    p.finish();
    true
}

/// `(prior | observe) Ref ":" { Ref: Number Sep }`.
fn belief(p: &mut Parser<'_>) -> bool {
    let Some(kw) = p.at_any_kw(&[KwPrior, KwObserve]) else {
        return false;
    };
    p.start(if kw == KwPrior {
        PriorDecl
    } else {
        ObserveDecl
    });
    p.bump_as(kw);
    if require(p, reference, &[Ident]) && p.expect(Colon) {
        require(p, |p| record(p, Key::Ref, number), &[LBrace]);
    }
    p.finish();
    true
}

/// `adversarial Ref | chance Ref [":" { Ref: Number Sep }]
/// | own_objective Ref ":" { Ref: Money Sep }`.
fn opponent(p: &mut Parser<'_>) -> bool {
    let Some(kw) = p.at_any_kw(&[KwAdversarial, KwChance, KwOwnObjective]) else {
        return false;
    };
    p.start(OpponentModelDecl);
    p.bump_as(kw);
    if require(p, reference, &[Ident]) {
        match kw {
            KwAdversarial => {}
            KwChance => {
                if p.eat(Colon) {
                    require(p, |p| record(p, Key::Ref, number), &[LBrace]);
                }
            }
            _ => {
                if p.expect(Colon) {
                    require(p, |p| record(p, Key::Ref, signed_money), &[LBrace]);
                }
            }
        }
    }
    p.finish();
    true
}
