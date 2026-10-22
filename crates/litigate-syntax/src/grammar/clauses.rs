//! Declaration clauses: `by`, `discretion`, `outcome`, version clauses,
//! relations, and citations, plus the missing-`cite` recovery.
use super::values::{cite, date, id_list, interval, reference, require};
use crate::error::{range, SyntaxError};
use crate::parser::Parser;
use crate::syntax_kind::SyntaxKind::{self, *};
use litigate_base::{Applicability, DiagnosticCode};

/// After a declaration header: a word on the same line that is not an item
/// keyword is a misplaced or misspelled clause (L0001 with a keyword fix).
pub(crate) fn expect_header_end(p: &mut Parser<'_>, keywords: &[SyntaxKind]) {
    if p.at(Ident) && !p.newline_before() && p.at_any_kw(p.item_keywords()).is_none() {
        let mut expected = keywords.to_vec();
        expected.push(LBrace);
        p.error_expected(&expected);
    }
}

/// Which clauses a declaration accepts (a set of flags).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Clauses(u8);

impl Clauses {
    const BY: u8 = 1;
    const NODE: u8 = 2;
    const VERSION: u8 = 4;
    const RELATION: u8 = 8;
    const CITATION: u8 = 16;

    /// Node declarations: every clause.
    pub(crate) const NODE_DECL: Self = Self(31);
    /// Rules, deadlines, counting rules, defaults: version, relation, cite.
    pub(crate) const RULE: Self = Self(Self::VERSION | Self::RELATION | Self::CITATION);
    /// Calendars: version and cite.
    pub(crate) const CALENDAR: Self = Self(Self::VERSION | Self::CITATION);
    /// Transitions and exceptions: cite only.
    pub(crate) const CITE: Self = Self(Self::CITATION);

    const fn has(self, flag: u8) -> bool {
        self.0 & flag != 0
    }

    fn keywords(self) -> Vec<SyntaxKind> {
        let mut out = Vec::new();
        if self.has(Self::BY) {
            out.push(KwBy);
        }
        if self.has(Self::NODE) {
            out.extend([KwDiscretion, KwOutcome]);
        }
        if self.has(Self::VERSION) {
            out.extend([KwEffective, KwKnownAt]);
        }
        if self.has(Self::RELATION) {
            out.extend(RELATIONS);
            out.push(KwDelegableTo);
        }
        if self.has(Self::CITATION) {
            out.push(KwCite);
        }
        out
    }
}

const RELATIONS: [SyntaxKind; 5] = [
    KwSubjectTo,
    KwNotwithstanding,
    KwSupersedes,
    KwRevives,
    KwOverride,
];

/// Parses clauses in any order until none applies. Returns the clause
/// keywords this declaration accepts (for "expected" messages).
pub(crate) fn clauses(p: &mut Parser<'_>, allowed: Clauses) -> Vec<SyntaxKind> {
    while clause(p, allowed) {}
    let keywords = allowed.keywords();
    let words: Vec<&str> = keywords.iter().filter_map(|kw| kw.text()).collect();
    let near_miss = crate::suggest::nearest(p.nth_text(0), &words).is_some();
    if allowed.has(Clauses::CITATION) && !near_miss {
        missing_cite(p);
    }
    keywords
}

fn clause(p: &mut Parser<'_>, allowed: Clauses) -> bool {
    if allowed.has(Clauses::CITATION) && cite(p) {
        return true;
    }
    if allowed.has(Clauses::BY) && p.at_kw(KwBy) && p.nth(1) == Ident {
        keyword_field(p, KwBy, Some(reference));
        return true;
    }
    if allowed.has(Clauses::NODE) && p.at_kw(KwDiscretion) {
        keyword_field(p, KwDiscretion, None);
        return true;
    }
    if allowed.has(Clauses::NODE) && p.at_kw(KwOutcome) {
        keyword_field(p, KwOutcome, Some(id_list));
        return true;
    }
    (allowed.has(Clauses::VERSION) && version_clause(p))
        || (allowed.has(Clauses::RELATION) && relation(p))
}

/// A `Field` node: `keyword [value]`.
pub(crate) fn keyword_field(
    p: &mut Parser<'_>,
    kw: SyntaxKind,
    value: Option<super::values::Value>,
) {
    p.start(Field);
    p.bump_as(kw);
    if let Some(value) = value {
        require(p, value, &[Ident]);
    }
    p.finish();
}

/// `VersionClause = "effective" Interval | "known_at" Date`.
pub(crate) fn version_clause(p: &mut Parser<'_>) -> bool {
    let (kw, value): (SyntaxKind, super::values::Value) = if p.at_kw(KwEffective) {
        (KwEffective, interval)
    } else if p.at_kw(KwKnownAt) {
        (KwKnownAt, date)
    } else {
        return false;
    };
    p.start(VersionClause);
    p.bump_as(kw);
    let expected = if kw == KwEffective { LBracket } else { Date };
    require(p, value, &[expected]);
    p.finish();
    true
}

/// `Relation = (subject_to | notwithstanding | supersedes | revives |
/// override) Ref | delegable_to IdList`.
fn relation(p: &mut Parser<'_>) -> bool {
    if p.at_kw(KwDelegableTo) {
        p.start(Relation);
        p.bump_as(KwDelegableTo);
        require(p, id_list, &[LBracket]);
        p.finish();
        return true;
    }
    let Some(kw) = p.at_any_kw(&RELATIONS) else {
        return false;
    };
    if p.nth(1) != Ident {
        return false;
    }
    p.start(Relation);
    p.bump_as(kw);
    reference(p);
    p.finish();
    true
}

/// Recognizes citation text written without `cite` (an identifier on the
/// same line, followed later on that line by a digit-bearing token) and
/// offers a machine-applicable `cite ` insertion. The rest of the line is
/// wrapped in an `Error` node.
fn missing_cite(p: &mut Parser<'_>) {
    let keyword = SyntaxKind::from_keyword(p.nth_text(0)).is_some();
    if !p.at(Ident) || keyword || p.newline_before() || p.at_item_anchor() {
        return;
    }
    let looks_cited = (0..16)
        .take_while(|n| !matches!(p.nth(*n), LBrace | Eof) && (*n == 0 || !newline_between(p, *n)))
        .any(|n| matches!(p.nth(n), Number | Money | Duration | Date));
    if !looks_cited {
        return;
    }
    let at = p.current_range();
    let start = usize::from(at.start());
    p.push_error(
        SyntaxError::new(
            DiagnosticCode::UnexpectedToken,
            at,
            format!(
                "expected a clause, found {}; citations start with `cite`",
                p.found()
            ),
        )
        .with_fix(
            range(start, start),
            "cite ",
            Applicability::MachineApplicable,
        ),
    );
    p.start(Error);
    while !p.at_end() && !p.at(LBrace) && !p.newline_before() {
        p.bump();
    }
    p.finish();
}

fn newline_between(p: &Parser<'_>, n: usize) -> bool {
    let prev = p.range_of(n - 1).end();
    let next = p.range_of(n).start();
    prev < next && p.text_between(prev, next).contains('\n')
}
