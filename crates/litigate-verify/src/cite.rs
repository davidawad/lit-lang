//! `cite-resolves` (SPEC §7): every cite resolves to a source binding and,
//! when the binding has a vendored `path`, to its pinpoint's `## heading`
//! span in that file, whose text is the one the binding's `path_sha256`
//! pins. A cited expression (the binding's `as_of`, D) dated after the
//! citing version took effect (E) is that version's text only when the
//! section's vendored amendment history shows no amendment in (E, D];
//! otherwise the cite is a hole whose reason names the amendment (or why
//! there is no history to read). `historical` cites are exempt.
//!
//! Citations are parsed exactly as the lowering's source maps parse them
//! (`litigate_lower::source_map::Citations`, litgraph's own parser under
//! the pack's citation syntax plus the bindings' `cite_prefixes`), so the
//! verdict here and the sidecar can never disagree. Case citations and
//! other text the parser treats as out of scope are not rule cites and are
//! not checked (the same policy as the engine's `lint` op), except text
//! that names a rule body a vendored source heads its sections with, or a
//! division of it, which is a rule cite the parser could not bind.
//!
//! A cite of a declaration `litigate import` marked `/// imported` is
//! exempt in `dev`: its failures are warnings, so a freshly imported pack
//! checks while its citations are ported. `strict` exempts nothing.
use crate::context::{spelled, Cite, GAP, GAP_WORDS};
use crate::diag::{emit, note};
use crate::Context;
use litigate_base::{Diagnostic, DiagnosticCode as C, Profile, Severity, Span};
use litigate_engine::cite::{parse_cite_string, CiteGrammar, Family, SourceCorpus};
use litigate_engine::model::Pack;
use litigate_lower::source_map::{subdivided, Citations, L0Section, Pin};
use litigate_sema::hir::DayUnit;
use litigate_sema::Hole;
use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::LazyLock;

/// L1010 for every citation that does not resolve. `holes` are the
/// package's: a cite hole's reason is resolved like a cite, but a hole
/// cites no expression, so it is never held to its version's window.
pub(crate) fn unresolved(
    cx: &Context<'_>,
    pack: &Pack,
    corpus: &SourceCorpus,
    holes: &[Hole],
) -> Vec<Diagnostic> {
    let sources = &cx.package.manifest.sources;
    let citations = Citations::new(pack.cite_syntax.clone(), sources, Some(corpus));
    cx.cites()
        .flat_map(|cite| {
            let text = cite.text;
            let span = cx.span_or_manifest(cite.span);
            let sections = citations
                .map(Some(&text.to_owned()))
                .map(|m| m.sections)
                .unwrap_or_default();
            let unkeyed = unkeyed(&citations, corpus, text, &span, cx);
            let hole = written(holes, text, &span);
            sections
                .iter()
                .filter_map(|s| {
                    section(cx, &citations, corpus, text, s, &span)
                        .or_else(|| expression(cx, &cite, s, &span).filter(|_| !hole))
                })
                .chain(unkeyed)
                .chain(unquoted(cx, corpus, text, &cite.quotes, &sections, &span))
                .chain(miscounted(
                    cx,
                    corpus,
                    text,
                    cite.period.filter(|_| !cite.historical && !hole),
                    &sections,
                    &span,
                ))
                .map(|d| exempt(d, cite.imported, cx.profile))
                .collect::<Vec<_>>()
        })
        .collect()
}

/// L1010 when no vendored section `text` resolves to holds any of
/// `quotes`, the words it is cited for, compared with whitespace runs
/// collapsed and case folded (the rules capitalize a part or count that
/// opens a sentence or list item: `A Table of Contents`) and digit
/// grouping dropped (D. Mont. L.R. 7.1(d)(2) writes `6500 words`), a
/// [`GAP`] standing for a few words: a quote the corpus cannot show is
/// not the rule's text. No quotes, no check.
fn unquoted(
    cx: &Context<'_>,
    corpus: &SourceCorpus,
    text: &str,
    quotes: &[Cow<'_, str>],
    sections: &[L0Section],
    span: &Span,
) -> Option<Diagnostic> {
    let words = |s: &str| {
        let s = s
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
            .to_lowercase();
        let c: Vec<char> = s.chars().collect();
        c.iter()
            .enumerate()
            .filter(|&(i, &ch)| {
                !(ch == ','
                    && i > 0
                    && c[i - 1].is_ascii_digit()
                    && c.get(i + 1).is_some_and(char::is_ascii_digit))
            })
            .map(|(_, &ch)| ch)
            .collect::<String>()
    };
    let folded: Vec<String> = quotes.iter().map(|q| words(q)).collect();
    let found = sections.iter().any(|s| {
        corpus.lookup_all(&s.heading).iter().any(|(file, section)| {
            let body = words(&unpaged(&section.body));
            s.path.as_ref() == Some(file) && folded.iter().any(|q| holds(&body, q))
        })
    });
    if found || quotes.is_empty() {
        return None;
    }
    let quote = quotes
        .iter()
        .filter(|q| !q.contains(GAP))
        .map(|q| format!("\"{}\"", q.split_whitespace().collect::<Vec<_>>().join(" ")))
        .collect::<Vec<_>>()
        .join(" or ");
    let d = emit(
        C::UnboundCite,
        cx.profile,
        span.clone(),
        format!("`{text}` is cited for {quote}, which its vendored text does not hold"),
    );
    Some(note(
        d,
        None,
        "quote the cited rule's words exactly as the vendored source gives them",
    ))
}

/// L1010 when a deadline's period (`2.days ... cite "X"`) is not a count
/// its vendored sections state while they state counts of that measure:
/// `X` sets a period of so many days (or hours, weeks), months or years,
/// and none of them is this one. A count is digits or words (`fourteen
/// (14) days`, `14-day`), up to [`COUNT_WORDS`] words before its measure;
/// a day count also matches its hours (`48 hours`) and whole weeks, a
/// month count its whole years, and a period matches the sum of two
/// different counts of one measure (37 C.F.R. § 2.102(c)(3): extensions
/// `totaling ninety days` and then `an additional sixty days`, counted
/// from the thirty-day period's end), or of one count stated by two cited
/// sections (a `3-month shortened statutory period`, MPEP § 710.01(a),
/// extended by the fee `For reply within third month`, 37 C.F.R. §
/// 1.17(a)(3)). A pinpoint the corpus sections apart
/// (`MPEP § 710.01(a)` under `MPEP § 710.01`) is read with its section.
/// A `historical` cite or a hole states no period. Sections that state no count of the
/// measure (a period written `the next day`, or set in another section)
/// are not checked.
fn miscounted(
    cx: &Context<'_>,
    corpus: &SourceCorpus,
    text: &str,
    period: Option<(u64, DayUnit)>,
    sections: &[L0Section],
    span: &Span,
) -> Option<Diagnostic> {
    let (amount, unit) = period.filter(|&(n, _)| n > 0)?;
    let family = Measure::of(unit);
    let counts: Vec<(usize, u64, Measure)> = sections
        .iter()
        .enumerate()
        .flat_map(|(at, s)| {
            let pinned = (1..=s.pinpoint.len()).map(|n| {
                s.pinpoint[..n]
                    .iter()
                    .fold(s.heading.clone(), |h, p| format!("{h}({p})"))
            });
            std::iter::once(s.heading.clone())
                .chain(pinned)
                .flat_map(|heading| {
                    corpus
                        .lookup_all(&heading)
                        .iter()
                        .filter(|(file, _)| s.path.as_ref() == Some(file))
                        .flat_map(|(_, section)| counts(&unpaged(&section.body)))
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>()
                .into_iter()
                .map(move |(n, m)| (at, n, m))
        })
        .filter(|(_, _, m)| m.monthly() == family.monthly())
        .collect();
    let wanted = family.equivalents(amount);
    let summed = counts.iter().any(|&(s, a, m)| {
        counts.iter().any(|&(t, b, n)| {
            m == n && (a < b || (a == b && s < t)) && wanted.contains(&(a + b, m))
        })
    });
    if counts.is_empty() || summed || counts.iter().any(|&(_, n, m)| wanted.contains(&(n, m))) {
        return None;
    }
    let mut stated: Vec<String> = counts.iter().map(|&(_, n, m)| m.prose(n)).collect();
    stated.sort();
    stated.dedup();
    let d = emit(
        C::UnboundCite,
        cx.profile,
        span.clone(),
        format!(
            "`{text}` is cited for a period of {}, but its vendored text states {}",
            family.prose(amount),
            stated.join(", "),
        ),
    );
    Some(note(
        d,
        None,
        "set the period the cited rule states, or cite the rule that sets this one",
    ))
}

/// The most words between a count and its measure: `fourteen (14)
/// calendar days`.
const COUNT_WORDS: usize = 3;

/// What a rule counts a period in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Measure {
    Hour,
    Day,
    Week,
    Month,
    Year,
}

impl Measure {
    fn of(unit: DayUnit) -> Self {
        match unit {
            DayUnit::Calendar | DayUnit::Court | DayUnit::Business => Self::Day,
            DayUnit::Week => Self::Week,
            DayUnit::Month => Self::Month,
            DayUnit::Year => Self::Year,
        }
    }

    /// The measure a word names (`days`, `day,`, `hours`), if any.
    fn named(word: &str) -> Option<Self> {
        let word = word.trim_end_matches(|c: char| !c.is_ascii_alphabetic());
        Some(match word.strip_suffix('s').unwrap_or(word) {
            "hour" => Self::Hour,
            "day" => Self::Day,
            "week" => Self::Week,
            "month" => Self::Month,
            "year" => Self::Year,
            _ => return None,
        })
    }

    const fn monthly(self) -> bool {
        matches!(self, Self::Month | Self::Year)
    }

    /// `n` of this measure and the same period in the others it converts
    /// to exactly.
    fn equivalents(self, n: u64) -> Vec<(u64, Self)> {
        let mut out = vec![(n, self)];
        match self {
            Self::Day => {
                out.push((n * 24, Self::Hour));
                if n.is_multiple_of(7) {
                    out.push((n / 7, Self::Week));
                }
            }
            Self::Week => out.extend([(n * 7, Self::Day), (n * 168, Self::Hour)]),
            Self::Month if n.is_multiple_of(12) => out.push((n / 12, Self::Year)),
            Self::Year => out.push((n * 12, Self::Month)),
            Self::Hour | Self::Month => {}
        }
        out
    }

    fn prose(self, n: u64) -> String {
        let noun = match self {
            Self::Hour => "hour",
            Self::Day => "day",
            Self::Week => "week",
            Self::Month => "month",
            Self::Year => "year",
        };
        format!("{n} {noun}{}", if n == 1 { "" } else { "s" })
    }
}

/// Every count `body` states with its measure: a count in digits or words
/// up to [`COUNT_WORDS`] words before a measure (`14 days`, `fourteen (14)
/// calendar days`, `one hundred eighty days`, `35days`), hyphened to it
/// (`14-day`, `seven-day`), or an ordinal day (`the 21st day`, `the next
/// day`); `the prior Monday` is a week. A page opening on a measure, under
/// any running head, takes a count ending one of the previous page's last
/// lines (`within 3`, `seven (7)`): the footer and running head a
/// `pdftotext -layout` extract sets between them (`within 3` / `Page 38` /
/// `days`, Bankr. S.D. Tex. BLR 9017-1(c); `seven (7)` / `[13]` / `days`,
/// TNWB LBR 2083-1(b)(2)) are not the rule's words.
fn counts(body: &str) -> Vec<(u64, Measure)> {
    let text: String = body
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || matches!(c, '-' | ',') {
                c
            } else {
                ' '
            }
        })
        .collect();
    let mut words: Vec<String> = Vec::new();
    for w in text.split_whitespace() {
        let w = w
            .trim_matches(|c: char| matches!(c, '-' | ','))
            .to_lowercase();
        let at = w.find(|c: char| !c.is_ascii_digit() && c != ',');
        match at.map(|i| w.split_at(i)) {
            Some((n, m)) if !n.is_empty() && Measure::named(m).is_some() => {
                words.extend([n.to_owned(), m.to_owned()]);
            }
            _ => words.push(w),
        }
    }
    let mut out = Vec::new();
    for (i, word) in words.iter().enumerate() {
        if let Some(m) = Measure::named(word) {
            let from = i.saturating_sub(COUNT_WORDS + 3);
            for j in from..i {
                for k in (j + 1)..=i {
                    if k - j > 3 || i - k > COUNT_WORDS || within_count(&words, j, k, i) {
                        continue;
                    }
                    if let Some(n) = number(&words[j..k].join(" ")) {
                        out.push((n, m));
                    }
                }
            }
        } else if let Some((count, measure)) = word.rsplit_once('-') {
            if let (Some(n), Some(m)) = (number(count), Measure::named(measure)) {
                out.push((n, m));
            }
        } else if WEEKDAYS.contains(&word.as_str())
            && i > 0
            && matches!(
                words[i - 1].as_str(),
                "prior" | "preceding" | "following" | "next"
            )
        {
            out.push((1, Measure::Week));
        }
    }
    let lines: Vec<&str> = body.lines().collect();
    for (i, line) in lines.iter().enumerate() {
        let Some(page) = line.strip_prefix('\u{c}') else {
            continue;
        };
        let head = lines[i + 1..lines.len().min(i + PAGE_FOOT)].iter();
        let Some(m) = std::iter::once(&page)
            .chain(head)
            .find_map(|l| Measure::named(&l.split_whitespace().next()?.to_lowercase()))
        else {
            continue;
        };
        out.extend(
            lines[i.saturating_sub(PAGE_FOOT)..i]
                .iter()
                .filter_map(|l| {
                    let mut w = l.split_whitespace().rev();
                    let (count, before) = (w.next()?, w.next()?);
                    let count = count.trim_matches(['(', ')']);
                    (INTRODUCING.contains(&before.to_lowercase().as_str())
                        || number(&before.to_lowercase()) == number(count))
                    .then(|| number(count))?
                })
                .map(|n| (n, m)),
        );
    }
    out
}

/// Whether `words[j..k]` is only part of a longer count before the
/// measure at `i`: `one`, `one hundred` and `fifty` in `one hundred fifty
/// (150) days` are not counts of their own.
fn within_count(words: &[String], j: usize, k: usize, i: usize) -> bool {
    (j.saturating_sub(3)..=j).any(|a| {
        (k..=i.min(a + 3)).any(|b| (a, b) != (j, k) && number(&words[a..b].join(" ")).is_some())
    })
}

/// The words that introduce a period's count (`within 3`, `at least 7`).
const INTRODUCING: [&str; 5] = ["within", "least", "than", "additional", "of"];

/// The most lines a page's footer and running head take below its text.
const PAGE_FOOT: usize = 12;

const WEEKDAYS: [&str; 7] = [
    "monday",
    "tuesday",
    "wednesday",
    "thursday",
    "friday",
    "saturday",
    "sunday",
];

/// The count `phrase` writes, in digits (`14`, `1,000`), words
/// (`fourteen`, `twenty-one`, `one hundred eighty`) up to a thousand, or
/// as an ordinal (`21st`, `twenty-first`, `next`).
fn number(phrase: &str) -> Option<u64> {
    static SPELLED: LazyLock<HashMap<String, u64>> = LazyLock::new(|| {
        (1..=1000)
            .flat_map(|n| [(spelled(n), n), (ordinal_of(&spelled(n)), n)])
            .chain([("next".to_owned(), 1), ("following".to_owned(), 1)])
            .collect()
    });
    let digits = phrase.replace(',', "");
    let ordinal = ["st", "nd", "rd", "th"]
        .iter()
        .find_map(|x| digits.strip_suffix(x))
        .unwrap_or(&digits);
    if !ordinal.is_empty() && ordinal.len() <= 6 && ordinal.chars().all(|c| c.is_ascii_digit()) {
        return ordinal.parse().ok();
    }
    SPELLED.get(&phrase.replace(" and ", " ")).copied()
}

/// The ordinal of a spelled count: `twenty-one` → `twenty-first`.
fn ordinal_of(spelled: &str) -> String {
    let at = spelled.rfind([' ', '-']).map_or(0, |i| i + 1);
    let (head, last) = spelled.split_at(at);
    let last = match last {
        "one" => "first".to_owned(),
        "two" => "second".to_owned(),
        "three" => "third".to_owned(),
        "five" => "fifth".to_owned(),
        "eight" => "eighth".to_owned(),
        "nine" => "ninth".to_owned(),
        "twelve" => "twelfth".to_owned(),
        l if l.ends_with('y') => format!("{}ieth", &l[..l.len() - 1]),
        l => format!("{l}th"),
    };
    format!("{head}{last}")
}

/// Whether `body` holds `quote`, a [`GAP`] in it matching one to
/// [`GAP_WORDS`] whitespace-free words.
fn holds(body: &str, quote: &str) -> bool {
    let Some((before, after)) = quote.split_once(&format!(" {GAP} ")) else {
        return body.contains(quote);
    };
    body.match_indices(before).any(|(i, _)| {
        let mut rest = &body[i + before.len()..];
        (0..GAP_WORDS).any(|_| {
            let Some(r) = rest.strip_prefix(' ') else {
                return false;
            };
            let word = r.find(' ').unwrap_or(r.len());
            rest = &r[word..];
            word > 0 && rest.strip_prefix(' ').is_some_and(|r| r.starts_with(after))
        })
    })
}

/// `body` without the page number a `pdftotext -layout` extract sets on
/// its own line before each page break (form feed): a quote running across
/// the break (`3,500` / `89` / `words`, LR 72.2(c)(1)(A), D. Minn.) is
/// the rule's words with the page's folio between them.
fn unpaged(body: &str) -> Cow<'_, str> {
    if !body.contains('\u{c}') {
        return Cow::Borrowed(body);
    }
    let lines: Vec<&str> = body.lines().collect();
    let folio = |l: &str| {
        let l = l.trim();
        !l.is_empty()
            && l.len() <= 8
            && (l.chars().all(|c| c.is_ascii_digit())
                || l.chars().all(|c| matches!(c, 'i' | 'v' | 'x' | 'l' | 'c')))
    };
    let kept: Vec<&str> = lines
        .iter()
        .enumerate()
        .filter(|&(i, l)| {
            let breaks = lines.get(i + 1).is_some_and(|n| n.starts_with('\u{c}'));
            !(breaks && folio(l))
        })
        .map(|(_, l)| *l)
        .collect();
    Cow::Owned(kept.join("\n"))
}

/// A hole for every cite whose expression (the binding's `as_of`, D)
/// postdates the day its version took effect (E) and that the section's
/// vendored amendment history does not prove unamended in (E, D]: the
/// text cited may not have been the law while that version was. Test
/// cites (no window), `historical` cites (a later edition reporting the
/// earlier text, say) and cites already written as holes (`holes`, the
/// package's) are exempt: a hole cites no expression.
pub(crate) fn unproven(
    cx: &Context<'_>,
    pack: &Pack,
    corpus: &SourceCorpus,
    holes: &[Hole],
) -> Vec<Hole> {
    let sources = &cx.package.manifest.sources;
    let citations = Citations::new(pack.cite_syntax.clone(), sources, Some(corpus));
    cx.cites()
        .filter(|cite| !cite.historical)
        .filter_map(|cite| {
            let from = &cite.window?.from;
            let span = cx.span_or_manifest(cite.span);
            let written = written(holes, cite.text, &span);
            let reason = citations.unproven(cite.text, from).filter(|_| !written)?;
            Some(Hole {
                span,
                reason: Some(reason),
            })
        })
        .collect()
}

/// `d` as a warning when its declaration is imported and `profile` is
/// `dev` (SPEC §7); unchanged otherwise.
fn exempt(d: Diagnostic, imported: bool, profile: Profile) -> Diagnostic {
    if !imported || profile != Profile::Dev {
        return d;
    }
    note(
        Diagnostic {
            severity: Severity::Warning,
            ..d
        },
        None,
        "a warning because the declaration is `/// imported`; `strict` rejects it",
    )
}

/// `text` is written as one of `holes` inside `span`.
fn written(holes: &[Hole], text: &str, span: &Span) -> bool {
    holes
        .iter()
        .any(|h| h.reason.as_deref() == Some(text) && within(&h.span, span))
}

/// `inner` lies inside `outer`.
fn within(inner: &Span, outer: &Span) -> bool {
    inner.file == outer.file
        && (inner.line, inner.col) >= (outer.line, outer.col)
        && (inner.end_line, inner.end_col) <= (outer.end_line, outer.end_col)
}

/// Rule or statute citations that do not normalize to a heading at all;
/// citations that continue a declared rule prefix with a section or
/// pinpoint that does not parse (`SDCL 15-6-4(z)` under the prefix
/// `SDCL 15-6-4(`) or with a division the prefix's vendored text numbers
/// but has no heading for (`Neb. R. Bankr. P. Appendix Z`, the text
/// heading `Appendix A` through `Appendix H`); and citations of a body a
/// vendored source heads its sections with but whose prefix no binding
/// declares (`FRBP 9999` beside `[sources.frbp]`, headed `## FRBP 1001`,
/// with no `cite_prefixes`). The parser leaves those out of scope, so
/// nothing else would check them.
fn unkeyed(
    citations: &Citations<'_>,
    corpus: &SourceCorpus,
    text: &str,
    span: &Span,
    cx: &Context<'_>,
) -> Vec<Diagnostic> {
    let grammar = citations.grammar();
    parse_cite_string(text, &grammar)
        .into_iter()
        .filter_map(|r| {
            let (why, fix) = if r.is_out_of_scope() {
                if let Some((prefix, rest)) = grammar.glued_rule_prefix(&r.raw) {
                    let why = format!(
                        "`{}` in `{text}` is written with the rule prefix `{prefix}`, but its \
                         section and pinpoint `{rest}` do not parse",
                        r.raw
                    );
                    (why, None)
                } else if r.family == Family::Other {
                    undivided(cx, citations, corpus, &grammar, &r.raw, text)
                        .or_else(|| unprefixed(cx, corpus, &r.raw, text))?
                } else {
                    return None;
                }
            } else if r.heading().is_none() {
                let why = format!(
                    "`{}` in `{text}` does not normalize to a source heading",
                    r.raw
                );
                (why, None)
            } else {
                return None;
            };
            let d = emit(C::UnboundCite, cx.profile, span.clone(), why);
            Some(match fix {
                Some(fix) => note(d, None, fix),
                None => d,
            })
        })
        .collect()
}

/// The `## ` headings of the corpus file at `path`.
fn headings<'a>(corpus: &'a SourceCorpus, path: &str) -> impl Iterator<Item = &'a str> {
    corpus
        .files
        .get(path)
        .map_or("", String::as_str)
        .lines()
        .filter_map(|l| l.strip_prefix("## ").map(str::trim))
}

/// Why `raw`, written with one of `grammar`'s rule prefixes and then text
/// that is no section number (`Neb. R. Bankr. P. Appendix Z`), does not
/// resolve: the prefix's vendored file heads divisions with the text's
/// first word (`## Appendix A`) but none the text starts with. A word the
/// file heads nothing with (`RCFC Appendix A` against the rules alone)
/// names a division that is not vendored, and stays out of scope.
fn undivided(
    cx: &Context<'_>,
    citations: &Citations<'_>,
    corpus: &SourceCorpus,
    grammar: &CiteGrammar<'_>,
    raw: &str,
    text: &str,
) -> Option<(String, Option<String>)> {
    let prefix = grammar.rule_prefix(raw)?;
    let rest = raw.get(prefix.len()..)?.trim();
    let word = rest.split_whitespace().next()?;
    let id = citations
        .syntax()
        .rules
        .iter()
        .find(|r| r.prefixes.iter().any(|p| p.eq_ignore_ascii_case(prefix)))?
        .source
        .as_ref()?;
    let path = cx
        .package
        .manifest
        .sources
        .iter()
        .find(|s| &s.id == id)?
        .path
        .as_deref()?;
    let mut division = headings(corpus, path)
        .filter(|h| h.split_whitespace().next() == Some(word))
        .peekable();
    division.peek()?;
    let found = division.any(|h| {
        rest.strip_prefix(h)
            .is_some_and(|after| !after.starts_with(char::is_alphanumeric))
    });
    if found {
        return None;
    }
    let why = format!(
        "`{raw}` in `{text}` cites `{rest}` under the rule prefix `{prefix}`, but none of \
         the `## {word} ...` headings of `{path}` is `{rest}`"
    );
    let fix = "cite a division that exists in the vendored text, or mark the cite \
               `?(\"why\")` until the source is vendored";
    Some((why, Some(fix.to_owned())))
}

/// Why `raw`, a body's name then one section token (`FRBP 9999`), does
/// not resolve: one of the package's vendored sources heads its sections
/// `<name> <number>` (`## FRBP 1001`), but no binding declares the name a
/// cite prefix, so the cite binds to nothing and the parser leaves it out
/// of scope.
fn unprefixed(
    cx: &Context<'_>,
    corpus: &SourceCorpus,
    raw: &str,
    text: &str,
) -> Option<(String, Option<String>)> {
    let at = raw
        .match_indices(' ')
        .map(|(i, _)| i + 1)
        .find(|&i| raw[i..].starts_with(|c: char| c.is_ascii_digit()))?;
    let (name, section) = (raw[..at].trim_end(), &raw[at..]);
    if name.is_empty() || section.contains(char::is_whitespace) {
        return None;
    }
    let source = cx.package.manifest.sources.iter().find(|s| {
        s.path.as_deref().is_some_and(|path| {
            headings(corpus, path).any(|h| {
                h.strip_prefix(name)
                    .and_then(|r| r.strip_prefix(' '))
                    .is_some_and(|r| r.starts_with(|c: char| c.is_ascii_digit()))
            })
        })
    })?;
    let id = &source.id;
    let path = source.path.as_deref().unwrap_or_default();
    let why = format!(
        "`{raw}` in `{text}` cites `{name}`, which `{path}` heads its sections with, but no \
         source binding declares the cite prefix `{name}`, so it resolves through none"
    );
    let fix = format!(
        "bind the prefix: add `cite_prefixes = [\"{name}\"]` and \
         `section_heading = \"{name} {{n}}\"` to `[sources.{id}]`"
    );
    Some((why, Some(fix)))
}

fn section(
    cx: &Context<'_>,
    citations: &Citations<'_>,
    corpus: &SourceCorpus,
    text: &str,
    s: &L0Section,
    span: &Span,
) -> Option<Diagnostic> {
    let sources = &cx.package.manifest.sources;
    let Some(binding) = s
        .source
        .as_ref()
        .and_then(|id| sources.iter().find(|b| &b.id == id))
    else {
        let d = emit(
            C::UnboundCite,
            cx.profile,
            span.clone(),
            format!(
                "`{}` (cited as `{text}`) does not resolve through a declared source binding",
                s.heading
            ),
        );
        return Some(note(
            d,
            None,
            "add its prefix to a `[sources.<id>]` `cite_prefixes`, or bind a \
             `[[cite_syntax.rules]]` entry to a source",
        ));
    };
    let path = binding.path.as_deref()?;
    let id = &binding.id;
    let unverified = match citations.pin(id) {
        Pin::Unpinned { actual } => Some((
            format!("source `{id}` pins no `path_sha256`, so the text of `{path}` is unverified"),
            format!(
                "check `{path}` against `{}` and pin it: `path_sha256 = \"{actual}\"`",
                binding.url.as_deref().unwrap_or("its source"),
            ),
        )),
        Pin::Mismatch { pinned, actual } => Some((
            format!(
                "`{path}` is not the text source `{id}` pins (`path_sha256` {pinned}, the \
                 corpus file hashes to {actual})"
            ),
            "cite the expression the corpus holds, or vendor the pinned text; a `litigate` \
             built before the pin changed embeds the older text, so rebuild it or set \
             `LITGRAPH_SOURCES` to the vendored `sources/`"
                .to_owned(),
        )),
        Pin::NoPath | Pin::Absent | Pin::Verified => None,
    };
    let found = corpus
        .lookup_all(&s.heading)
        .iter()
        .any(|(file, _)| file == path);
    if found
        && unverified.is_none()
        && subdivided(corpus, path, &s.heading, &s.pinpoint, text)
    {
        return None;
    }
    let (why, fix) = unverified.unwrap_or_else(|| {
        let why = if found {
            format!("`{text}` has a pinpoint the section does not write")
        } else if corpus.has_file(path) {
            format!("`{path}` has no `## {}` heading", s.heading)
        } else {
            format!("`{path}` is not in the vendored source corpus")
        };
        let fix = "cite a pinpoint that exists in the vendored text, or mark the cite \
                   `?(\"why\")` until the source is vendored";
        (why, fix.to_owned())
    });
    let d = emit(
        C::UnboundCite,
        cx.profile,
        span.clone(),
        format!("`{text}` binds to source `{id}`, but {why}"),
    );
    Some(note(d, None, fix))
}

/// L1010 when an unvendored cited expression (the binding's `as_of`) is
/// dated on or after the end of the citing version's window: the text
/// cited was not yet the law while that version was. A vendored text is
/// held to its amendment history instead ([`unproven`]). `historical` cites (a later edition
/// reporting the earlier text, say) and expressions without a date are
/// exempt; an expression older than the window is the text as amended to
/// that date, presumed still current when the window opens.
fn expression(cx: &Context<'_>, cite: &Cite<'_>, s: &L0Section, span: &Span) -> Option<Diagnostic> {
    let as_of = s.as_of.as_deref().filter(|_| s.path.is_none())?;
    let until = cite.window?.to.as_deref()?;
    if cite.historical || as_of < until {
        return None;
    }
    let d = emit(
        C::UnboundCite,
        cx.profile,
        span.clone(),
        format!(
            "`{}` cites source `{}` as of {as_of}, but this version left force on {until}",
            cite.text,
            s.source.as_deref().unwrap_or_default(),
        ),
    );
    Some(note(
        d,
        None,
        "cite an expression in force during the version's window, or mark the cite \
         `historical` when a later text is cited on purpose",
    ))
}
