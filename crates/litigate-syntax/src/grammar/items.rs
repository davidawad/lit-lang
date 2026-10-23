//! `.lit` files (SPEC §3.1): header, item dispatch, and graph declarations.
use super::clauses::{clauses, expect_header_end, keyword_field, Clauses};
use super::fields::{field, field_list, fixed_fields, keywords, Spec, EDGE_FIELDS, NODE_FIELDS};
use super::values::{prob_value, reference, require, type_expr};
use super::{decls, delimited_node, Sep};
use crate::error::SyntaxError;
use crate::parser::{Parser, LIT_ITEM_KEYWORDS};
use crate::syntax_kind::SyntaxKind::{self, *};
use litigate_base::{Applicability, DiagnosticCode};

/// Item keywords allowed inside a `rule` body (no header items).
pub(crate) const BODY_ITEM_KEYWORDS: &[SyntaxKind] = &[
    KwStart,
    KwGroup,
    KwType,
    KwEnum,
    KwEvent,
    KwState,
    KwDecision,
    KwChance,
    KwTerminal,
    KwLink,
    KwInstance,
    KwRule,
    KwDeadline,
    KwCounting,
    KwCalendar,
    KwTransition,
    KwAuthority,
    KwDefault,
    KwException,
    KwTest,
];

/// `LitFile = [EditionDecl] ModuleDecl {ImportDecl} [PackageDecl] {Item}`.
pub(crate) fn lit_file(p: &mut Parser<'_>) {
    p.start_root(SourceFile);
    super::edition(p);
    if !p.at_kw(KwModule) {
        p.error_expected(&[KwModule]);
    }
    let mut rank = 0u8;
    // Every pass consumes a token: an item consumes its keyword, and
    // recovery stops only at an item keyword (the anchors of a `.lit` file).
    while !p.at_end() {
        match p.at_any_kw(LIT_ITEM_KEYWORDS) {
            Some(kw) => {
                rank = check_order(p, kw, rank);
                item(p, kw);
                item_end(p, &[]);
            }
            None => p.recover(LIT_ITEM_KEYWORDS, &[]),
        }
    }
}

fn check_order(p: &mut Parser<'_>, kw: SyntaxKind, rank: u8) -> u8 {
    let (this, once, rule) = match kw {
        KwModule => (
            1,
            true,
            "`module` must be the first declaration and appear once",
        ),
        KwImport => (
            2,
            false,
            "imports must follow `module` and precede `package` and items",
        ),
        KwPackage => (
            3,
            true,
            "`package` must follow the imports, precede items, and appear once",
        ),
        _ => (4, false, ""),
    };
    if this < rank || (once && this == rank) {
        let at = p.current_range();
        p.push_error(SyntaxError::new(DiagnosticCode::UnexpectedToken, at, rule));
    }
    rank.max(this)
}

/// After an item: the next token must start another item, close the
/// enclosing body, or end the file.
pub(crate) fn item_end(p: &mut Parser<'_>, follow: &[SyntaxKind]) {
    if p.at_end() || follow.contains(&p.nth(0)) || p.at_any_kw(LIT_ITEM_KEYWORDS).is_some() {
        return;
    }
    let mut expected = follow.to_vec();
    expected.extend(p.item_keywords());
    p.recover(&expected, follow);
}

/// Parses the item introduced by `kw` (the current token).
pub(crate) fn item(p: &mut Parser<'_>, kw: SyntaxKind) {
    match kw {
        KwModule | KwImport => header(p, kw),
        KwPackage => package(p),
        KwStart => simple(p, StartDecl, kw),
        KwGroup => group(p),
        KwType => type_decl(p),
        KwEnum => enum_decl(p),
        KwEvent => event_decl(p),
        KwState | KwDecision | KwChance | KwTerminal => node_decl(p, kw),
        KwAuthority => authority(p),
        _ => decls::decl(p, kw),
    }
}

fn simple(p: &mut Parser<'_>, kind: SyntaxKind, kw: SyntaxKind) {
    p.start(kind);
    p.bump_as(kw);
    require(p, reference, &[Ident]);
    p.finish();
}

/// `module a.b` / `import a.b [. {x, y}]`.
fn header(p: &mut Parser<'_>, kw: SyntaxKind) {
    let kind = if kw == KwModule {
        ModuleDecl
    } else {
        ImportDecl
    };
    p.start(kind);
    p.bump_as(kw);
    if require(p, reference, &[Ident]) && kw == KwImport && p.at(Dot) && p.nth(1) == LBrace {
        p.bump();
        let mut name = |p: &mut Parser<'_>| p.eat(Ident);
        delimited_node(
            p,
            ImportList,
            (LBrace, RBrace),
            Sep::Required,
            &[Ident],
            &mut name,
        );
    }
    p.finish();
}

/// `package name { forum f, authority A }`.
fn package(p: &mut Parser<'_>) {
    const FIELDS: &[Spec] = &[(KwForum, Some(reference)), (KwAuthority, Some(reference))];
    p.start(PackageDecl);
    p.bump_as(KwPackage);
    p.expect(Ident);
    if !fixed_fields(p, FIELDS, false) {
        p.error_expected(&[LBrace]);
    }
    p.finish();
}

/// `group id: "Label"`.
fn group(p: &mut Parser<'_>) {
    p.start(GroupDecl);
    p.bump_as(KwGroup);
    let _ = p.expect(Ident) && p.expect(Colon) && p.expect(String);
    p.finish();
}

/// `type Name = Type`.
fn type_decl(p: &mut Parser<'_>) {
    p.start(TypeDecl);
    p.bump_as(KwType);
    if p.expect(Ident) && p.expect(Eq) {
        require(p, type_expr, &[Ident]);
    }
    p.finish();
}

/// `enum Name { A, B }`.
fn enum_decl(p: &mut Parser<'_>) {
    p.start(EnumDecl);
    p.bump_as(KwEnum);
    if p.expect(Ident) {
        let mut variant = |p: &mut Parser<'_>| p.eat(Ident);
        super::delimited(p, (LBrace, RBrace), Sep::Required, &[Ident], &mut variant);
    }
    p.finish();
}

/// `event Name(param: Type, ...)`.
fn event_decl(p: &mut Parser<'_>) {
    p.start(EventDecl);
    p.bump_as(KwEvent);
    if p.expect(Ident) {
        delimited_node(
            p,
            ParamList,
            (LParen, RParen),
            Sep::Required,
            &[Ident],
            &mut param,
        );
    }
    p.finish();
}

fn param(p: &mut Parser<'_>) -> bool {
    if !p.at(Ident) {
        return false;
    }
    p.start(Param);
    p.bump();
    if p.expect(Colon) {
        require(p, type_expr, &[Ident]);
    }
    p.finish();
    true
}

/// `authority A > B > C`.
fn authority(p: &mut Parser<'_>) {
    p.start(AuthorityDecl);
    p.bump_as(KwAuthority);
    if p.expect(Ident) {
        while p.eat(Greater) {
            if !p.expect(Ident) {
                break;
            }
        }
    }
    p.finish();
}

/// `NodeKw Ident ":" String {NodeClause} [NodeBody]`.
fn node_decl(p: &mut Parser<'_>, kw: SyntaxKind) {
    let kind = match kw {
        KwState => StateDecl,
        KwDecision => DecisionDecl,
        KwChance => ChanceDecl,
        _ => TerminalDecl,
    };
    p.start(kind);
    p.bump_as(kw);
    let _ = p.expect(Ident) && p.expect(Colon) && p.expect(String);
    let clause_keywords = clauses(p, Clauses::NODE_DECL);
    expect_header_end(p, &clause_keywords);
    if p.at(LBrace) {
        let mut expected = keywords(NODE_FIELDS);
        expected.push(Ident);
        delimited_node(
            p,
            EdgeBlock,
            (LBrace, RBrace),
            Sep::Optional,
            &expected,
            &mut body_entry,
        );
    }
    p.finish();
}

fn body_entry(p: &mut Parser<'_>) -> bool {
    if at_edge(p) {
        edge(p);
        return true;
    }
    field(p, NODE_FIELDS)
}

/// LL(3): `(Ident | "_") String "->"` is an edge; so is `Ident String`
/// when the identifier is not a string-valued field keyword.
fn at_edge(p: &Parser<'_>) -> bool {
    let id = p.at(Underscore) || p.at(Ident);
    let string_field = p.at_kw(KwNote) || p.at_kw(KwCourtListenerUrl);
    id && p.nth(1) == String && (p.nth(2) == Arrow || !string_field)
}

/// `(Ident | "_") String "->" Ref {EdgeClause} [EdgeFields]`.
fn edge(p: &mut Parser<'_>) {
    p.start(EdgeDecl);
    p.bump();
    p.bump();
    arrow(p);
    require(p, reference, &[Ident]);
    edge_tail(p);
    p.finish();
}

/// `{EdgeClause} [EdgeFields]`, shared with `link`.
pub(crate) fn edge_tail(p: &mut Parser<'_>) {
    loop {
        if p.at_kw(KwBy) && p.nth(1) == Ident && p.nth(2) != String {
            keyword_field(p, KwBy, Some(reference));
        } else if p.at_kw(KwP) && p.nth(1) != String {
            keyword_field(p, KwP, Some(prob_value));
        } else {
            break;
        }
    }
    field_list(p, EDGE_FIELDS);
}

/// `->`, with the SPEC §9.2 diagnostic and fix when it is missing.
pub(crate) fn arrow(p: &mut Parser<'_>) {
    if p.eat(Arrow) {
        return;
    }
    let at = p.current_range();
    let insert = crate::error::range(usize::from(at.start()), usize::from(at.start()));
    p.push_error(
        SyntaxError::new(
            DiagnosticCode::UnexpectedToken,
            at,
            format!("expected `->`, found {}", p.found()),
        )
        .with_note("edges are written `id \"label\" -> target`")
        .with_fix(insert, "-> ", Applicability::MachineApplicable),
    );
}
