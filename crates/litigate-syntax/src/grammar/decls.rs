//! Rule-bearing declarations: `rule`, `deadline`, `counting`, `calendar`,
//! `transition`, `default`, `exception`, `test`, `link`, `instance`.
use super::clauses::{clauses, keyword_field, version_clause, Clauses};
use super::expr::{require_expr, EXPR_START};
use super::fields::{
    field, field_list, keywords, number_value, patch, COUNTING_ITEMS, INSTANCE_FIELDS,
};
use super::items::{arrow, edge_tail, item, item_end, BODY_ITEM_KEYWORDS};
use super::values::{call, cite, date, duration, reference, require};
use super::{delimited, delimited_node, Sep};
use crate::parser::Parser;
use crate::syntax_kind::SyntaxKind::{self, *};

/// Parses the declaration introduced by `kw`.
pub(crate) fn decl(p: &mut Parser<'_>, kw: SyntaxKind) {
    match kw {
        KwLink => link(p),
        KwInstance => instance(p),
        KwRule => rule(p),
        KwDeadline => deadline(p),
        KwCounting => counting(p),
        KwCalendar => calendar(p),
        KwTransition => transition(p),
        KwDefault => default(p),
        KwException => exception(p),
        _ => test(p),
    }
}

/// Reports a missing body: expected one of the clause keywords or `{`.
fn require_body(p: &mut Parser<'_>, clause_keywords: &[SyntaxKind]) -> bool {
    if p.at(LBrace) {
        return true;
    }
    let mut expected = clause_keywords.to_vec();
    expected.push(LBrace);
    p.error_expected(&expected);
    false
}

/// `link (Ident | "_") String ":" Ref "->" Ref {EdgeClause} [EdgeFields]`.
fn link(p: &mut Parser<'_>) {
    p.start(LinkDecl);
    p.bump_as(KwLink);
    if p.at(Ident) || p.at(Underscore) {
        p.bump();
    } else {
        p.error_expected(&[Ident, Underscore]);
    }
    if p.expect(String) && p.expect(Colon) && require(p, reference, &[Ident]) {
        arrow(p);
        require(p, reference, &[Ident]);
        edge_tail(p);
    }
    p.finish();
}

/// `instance base "@" origin { InstanceField Sep }`.
fn instance(p: &mut Parser<'_>) {
    p.start(InstanceDecl);
    p.bump_as(KwInstance);
    if p.expect(Ident) && p.expect(At) && p.expect(Ident) {
        let mut expected = keywords(INSTANCE_FIELDS);
        expected.push(KwPatch);
        let mut entry = |p: &mut Parser<'_>| patch(p) || field(p, INSTANCE_FIELDS);
        delimited_node(
            p,
            FieldList,
            (LBrace, RBrace),
            Sep::Optional,
            &expected,
            &mut entry,
        );
    }
    p.finish();
}

/// `rule (duty | power | liability) id {clauses} { {Item Sep} }`.
fn rule(p: &mut Parser<'_>) {
    p.start(RuleDecl);
    p.bump_as(KwRule);
    match p.at_any_kw(&[KwDuty, KwPower, KwLiability]) {
        Some(kind) => p.bump_as(kind),
        None => p.error_expected(&[KwDuty, KwPower, KwLiability]),
    }
    p.expect(Ident);
    let clause_keywords = clauses(p, Clauses::RULE);
    if require_body(p, &clause_keywords) {
        let mut entry = |p: &mut Parser<'_>| match p.at_any_kw(BODY_ITEM_KEYWORDS) {
            Some(kw) => {
                item(p, kw);
                item_end(p, &[Comma, RBrace]);
                true
            }
            None => false,
        };
        p.nested(|p| {
            delimited(
                p,
                (LBrace, RBrace),
                Sep::Optional,
                BODY_ITEM_KEYWORDS,
                &mut entry,
            );
        });
    }
    p.finish();
}

/// `deadline id: N.unit (from | after | before) Expr counted_by Ref
/// {clauses} [DeadlineOpts]`.
fn deadline(p: &mut Parser<'_>) {
    p.start(DeadlineDecl);
    p.bump_as(KwDeadline);
    if p.expect(Ident) && p.expect(Colon) && require(p, duration, &[Duration]) {
        match p.at_any_kw(&[KwFrom, KwAfter, KwBefore]) {
            Some(direction) => {
                p.bump_as(direction);
                if require_expr(p) && p.expect_kw(KwCountedBy) {
                    require(p, reference, &[Ident]);
                }
            }
            None => p.error_expected(&[KwFrom, KwAfter, KwBefore]),
        }
        clauses(p, Clauses::RULE);
        field_list(p, super::fields::DEADLINE_OPTS);
    }
    p.finish();
}

/// `counting id {clauses} { CountingItem Sep }`.
fn counting(p: &mut Parser<'_>) {
    p.start(CountingDecl);
    p.bump_as(KwCounting);
    p.expect(Ident);
    let clause_keywords = clauses(p, Clauses::RULE);
    if require_body(p, &clause_keywords) {
        field_list(p, COUNTING_ITEMS);
    }
    p.finish();
}

/// `calendar id {Version | Cite} { HolidayDecl Sep }`.
fn calendar(p: &mut Parser<'_>) {
    p.start(CalendarDecl);
    p.bump_as(KwCalendar);
    p.expect(Ident);
    let clause_keywords = clauses(p, Clauses::CALENDAR);
    if require_body(p, &clause_keywords) {
        let mut entry = |p: &mut Parser<'_>| holiday(p);
        delimited(p, (LBrace, RBrace), Sep::Optional, &[Ident], &mut entry);
    }
    p.finish();
}

/// `id ":" CallExpr "effective" Interval ["observed" Ident]`.
fn holiday(p: &mut Parser<'_>) -> bool {
    if !p.at(Ident) {
        return false;
    }
    p.start(HolidayDecl);
    p.bump();
    if p.expect(Colon) && require(p, call, &[Ident]) {
        if !(p.at_kw(KwEffective) && version_clause(p)) {
            p.error_expected(&[KwEffective]);
        }
        if p.at_kw(KwObserved) {
            keyword_field(p, KwObserved, Some(reference));
        }
    }
    p.finish();
    true
}

/// `transition id amends Ref {Cite} { TransitionArm Sep }`.
fn transition(p: &mut Parser<'_>) {
    p.start(TransitionDecl);
    p.bump_as(KwTransition);
    if p.expect(Ident) && p.expect_kw(KwAmends) {
        require(p, reference, &[Ident]);
    }
    let clause_keywords = clauses(p, Clauses::CITE);
    if require_body(p, &clause_keywords) {
        let expected = [KwUnless, KwOtherwise];
        delimited(
            p,
            (LBrace, RBrace),
            Sep::Optional,
            &expected,
            &mut transition_arm,
        );
    }
    p.finish();
}

fn transition_arm(p: &mut Parser<'_>) -> bool {
    let Some(kw) = p.at_any_kw(&[KwUnless, KwOtherwise]) else {
        return false;
    };
    p.start(TransitionArm);
    p.bump_as(kw);
    if (kw == KwOtherwise || require_expr(p)) && p.expect(Arrow) {
        require(p, reference, &[Ident]);
    }
    p.finish();
    true
}

/// `default id {clauses} when Expr => Expr`.
fn default(p: &mut Parser<'_>) {
    p.start(DefaultDecl);
    p.bump_as(KwDefault);
    p.expect(Ident);
    let mut expected = clauses(p, Clauses::RULE);
    expected.push(KwWhen);
    if p.eat_kw(KwWhen) {
        if require_expr(p) && p.expect(FatArrow) {
            require_expr(p);
        }
    } else {
        p.error_expected(&expected);
    }
    p.finish();
}

/// `exception id (rebuts | undercuts) Ref {Cite} when Expr [=> Expr]
/// [priority Number]`.
fn exception(p: &mut Parser<'_>) {
    p.start(ExceptionDecl);
    p.bump_as(KwException);
    p.expect(Ident);
    match p.at_any_kw(&[KwRebuts, KwUndercuts]) {
        Some(kind) => {
            p.bump_as(kind);
            require(p, reference, &[Ident]);
        }
        None => p.error_expected(&[KwRebuts, KwUndercuts]),
    }
    let mut expected = clauses(p, Clauses::CITE);
    expected.push(KwWhen);
    if p.eat_kw(KwWhen) {
        if require_expr(p) && p.eat(FatArrow) {
            require_expr(p);
        }
        if p.at_kw(KwPriority) {
            keyword_field(p, KwPriority, Some(number_value));
        }
    } else {
        p.error_expected(&expected);
    }
    p.finish();
}

/// `test "name" [Cite] { {given Expr on Date | expect Expr} }`.
fn test(p: &mut Parser<'_>) {
    p.start(TestDecl);
    p.bump_as(KwTest);
    p.expect(String);
    cite(p);
    if require_body(p, &[KwCite]) {
        let expected = [KwGiven, KwExpect];
        delimited(
            p,
            (LBrace, RBrace),
            Sep::Forbidden,
            &expected,
            &mut test_stmt,
        );
    }
    p.finish();
}

fn test_stmt(p: &mut Parser<'_>) -> bool {
    if p.at_kw(KwGiven) {
        p.start(GivenStmt);
        p.bump_as(KwGiven);
        if require_expr(p) && p.expect_kw(KwOn) {
            require(p, date, &[Date]);
        }
        p.finish();
        return true;
    }
    if p.at_kw(KwExpect) {
        p.start(ExpectStmt);
        p.bump_as(KwExpect);
        if !super::expr::expr(p) {
            p.error_expected(EXPR_START);
        }
        p.finish();
        return true;
    }
    false
}
