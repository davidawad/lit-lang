//! Reserved filing requirements (SPEC §3.7): `limit paper-kind { ... }`
//! with its length bounds, certificates, exclusions and ECF event codes.
//! Parsed in full so the rest of the file parses, then rejected as L0011
//! until lg-962.10 implements them. `limit`, `certificate` and `excludes`
//! are reserved words the lexer already reports; nothing here reports
//! L0011 a second time.
use super::clauses::version_clause;
use super::values::{cite, hole, number, reference, require, string};
use super::{brace_list, delimited_node, Sep};
use crate::parser::Parser;
use crate::syntax_kind::SyntaxKind::*;

/// The length measures of a `limit` body.
const MEASURES: [crate::SyntaxKind; 3] = [KwWords, KwPages, KwLines];

/// `LimitDecl = "limit" Ref {VersionClause | Cite} LimitBody
/// {VersionClause | Cite}`.
pub(crate) fn limit(p: &mut Parser<'_>) {
    p.start(LimitDecl);
    p.bump_as(KwLimit);
    if require(p, reference, &[Ident]) {
        authority(p);
        if p.at(LBrace) {
            let mut expected = MEASURES.to_vec();
            expected.extend([KwRequires, KwExcludes, KwEcf]);
            brace_list(p, Sep::Optional, &expected, &mut entry);
            authority(p);
        } else {
            p.error_expected(&[KwEffective, KwKnownAt, KwCite, LBrace]);
        }
    }
    p.finish();
}

/// `{VersionClause | Cite}`: the declaration's versions and authority.
fn authority(p: &mut Parser<'_>) {
    while version_clause(p) || cite(p) {}
}

/// One `limit` body entry, each with its own `{Cite}`.
fn entry(p: &mut Parser<'_>) -> bool {
    if let Some(measure) = p.at_any_kw(&MEASURES) {
        p.start(LimitBound);
        p.bump_as(measure);
        if p.expect(LessEq) {
            require(p, |p| number(p) || hole(p), &[Number, Question]);
        }
    } else if p.at_kw(KwRequires) {
        p.start(CertificateReq);
        p.bump_as(KwRequires);
        if p.expect_kw(KwCertificate) {
            require(p, reference, &[Ident]);
        }
    } else if p.at_kw(KwExcludes) {
        p.start(Exclusions);
        p.bump_as(KwExcludes);
        if p.at(LBracket) {
            delimited_node(
                p,
                IdList,
                (LBracket, RBracket),
                Sep::Required,
                &[Ident, Question],
                &mut |p| reference(p) || hole(p),
            );
        } else {
            p.error_expected(&[LBracket]);
        }
    } else if p.at_kw(KwEcf) {
        p.start(EcfCode);
        p.bump_as(KwEcf);
        require(p, |p| string(p) || hole(p), &[String, Question]);
    } else {
        return false;
    }
    while cite(p) {}
    p.finish();
    true
}
