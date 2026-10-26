//! Declaration clauses: `by`, `discretion`, `info`, `outcome`, version clauses,
//! relations, and citations, plus the missing-`cite` recovery.
use super::values::{cite, date, id_list, interval, quote_rest, reference, require, skip_until};
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

/// Which clauses a declaration accepts besides `cite`, which every
/// declaration with clauses accepts (a set of flags).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Clauses(u8);

impl Clauses {
    const BY: u8 = 1;
    const NODE: u8 = 2;
    const VERSION: u8 = 4;
    const RELATION: u8 = 8;

    /// Node declarations: every clause.
    pub(crate) const NODE_DECL: Self = Self(15);
    /// Rules, deadlines, counting rules, defaults: version, relation, cite.
    pub(crate) const RULE: Self = Self(Self::VERSION | Self::RELATION);
    /// Calendars: version and cite.
    pub(crate) const CALENDAR: Self = Self(Self::VERSION);
    /// Transitions and exceptions: cite only.
    pub(crate) const CITE: Self = Self(0);

    const fn has(self, flag: u8) -> bool {
        self.0 & flag != 0
    }

    fn keywords(self) -> Vec<SyntaxKind> {
        let mut out = Vec::new();
        if self.has(Self::BY) {
            out.push(KwBy);
        }
        if self.has(Self::NODE) {
            out.extend([KwDiscretion, KwInfo, KwOutcome]);
        }
        if self.has(Self::VERSION) {
            out.extend([
                KwEffective,
                KwKnownAt,
                KwRetracted,
                KwUnknownBefore,
                KwCommenced,
            ]);
        }
        if self.has(Self::RELATION) {
            out.extend(RELATIONS);
            out.push(KwDelegable);
        }
        out.push(KwCite);
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
    if !near_miss {
        missing_cite(p);
    }
    keywords
}

fn clause(p: &mut Parser<'_>, allowed: Clauses) -> bool {
    if cite(p) {
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
    if allowed.has(Clauses::NODE) && p.at_kw(KwInfo) && p.nth(1) == Ident {
        keyword_field(p, KwInfo, Some(reference));
        return true;
    }
    if allowed.has(Clauses::NODE) && p.at_kw(KwOutcome) {
        keyword_field(p, KwOutcome, Some(id_list));
        return true;
    }
    (allowed.has(Clauses::VERSION) && version_clause(p))
        || (allowed.has(Clauses::RELATION) && relation(p))
}

/// `IndexClause = "Rule", "[", Index, { ",", Index }, "]"` with `Index =
/// "_" | Interval | Ref`: a rule's indices (SPEC §3.1), `_` leaving one to
/// its default. Sema checks there are five.
pub(crate) fn index_clause(p: &mut Parser<'_>) {
    p.start(IndexClause);
    p.bump_as(KwRuleIndex);
    super::delimited(
        p,
        (LBracket, RBracket),
        super::Sep::Required,
        &[Underscore, LBracket, Ident],
        &mut |p| p.eat(Underscore) || interval(p) || reference(p),
    );
    p.finish();
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

/// `VersionClause = ("effective" | "commenced") Interval | ("known_at" |
/// "retracted" | "unknown_before") Date`.
pub(crate) fn version_clause(p: &mut Parser<'_>) -> bool {
    let (kw, value): (SyntaxKind, super::values::Value) =
        if let Some(kw) = p.at_any_kw(&[KwEffective, KwCommenced]) {
            (kw, interval)
        } else if let Some(kw) = p.at_any_kw(&[KwKnownAt, KwRetracted, KwUnknownBefore]) {
            (kw, date)
        } else {
            return false;
        };
    p.start(VersionClause);
    p.bump_as(kw);
    let expected = if matches!(kw, KwEffective | KwCommenced) {
        LBracket
    } else {
        Date
    };
    require(p, value, &[expected]);
    p.finish();
    true
}

/// `Relation = (subject_to | notwithstanding | supersedes | revives |
/// override) Ref | delegable_to IdList | delegable Delegation`. The legacy
/// `delegable_to` still parses so that sema can name its replacement.
fn relation(p: &mut Parser<'_>) -> bool {
    if p.at_kw(KwDelegable) && p.nth(1) == LBrace {
        p.start(Relation);
        p.bump_as(KwDelegable);
        super::fields::delegation(p);
        p.finish();
        return true;
    }
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
/// offers a machine-applicable fix quoting it after `cite` (up to the
/// first `{`, `}`, `,`, or `//`). That text is wrapped in an `Error` node.
/// (Item keywords are keywords, so this never starts at a recovery anchor.)
fn missing_cite(p: &mut Parser<'_>) {
    let keyword = SyntaxKind::from_keyword(p.nth_text(0)).is_some();
    if !p.at(Ident) || keyword || p.newline_before() {
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
    let (end, quoted) = quote_rest(p, start);
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
            range(start, end),
            format!("cite {quoted}"),
            Applicability::MachineApplicable,
        ),
    );
    skip_until(p, end);
}

fn newline_between(p: &Parser<'_>, n: usize) -> bool {
    let prev = p.range_of(n - 1).end();
    let next = p.range_of(n).start();
    prev < next && p.text_between(prev, next).contains('\n')
}
