// SPDX-License-Identifier: GPL-3.0-or-later
//! Normalize the many surface forms of a legal citation ("28 U.S.C. §
//! 1498(a)", "35 USC 315(e)", "37 C.F.R. § 42.108", and every rule body a
//! pack declares in its `citeSyntax`) into a canonical [`CiteRef`] whose
//! [`CiteRef::heading`] matches the `## <ref>` section headings in the
//! vendored corpus (`sources/*.txt`, see `cite::corpus`).
//!
//! The engine knows only the forum-neutral shapes (U.S.C., C.F.R., case
//! names). Rule bodies and pack-private shorthands are data
//! ([`CiteSyntax`]), applied through a [`CiteGrammar`].
//!
//! Pack `cite`/`authority` strings are often compound ("28 U.S.C. 1291;
//! 2106", "35 U.S.C. 314(a); 314(d)", "18 U.S.C. § 3142(b), (c)") and
//! sometimes aren't rule/statute citations at all (case names, "Sup. Ct.
//! R. 10", doctrinal shorthand like "Fintiv factor 4"). [`parse_cite_string`]
//! splits a raw string into one [`CiteRef`] per citation, carrying the
//! family/title of the previous citation forward onto a bare trailing
//! number the way legal writing does ("§§ 1292(a)(1), 1292(b)").

use crate::cite::corpus::SourceCorpus;
use crate::model::{CiteCode, CiteShorthand, CiteSyntax};

/// Which body of law a citation belongs to. The title number on
/// [`Family::Usc`]/[`Family::Cfr`] is what makes `28 U.S.C. § 1291` and
/// `35 U.S.C. § 1291` (hypothetically) distinct citations.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Family {
    /// A rule/manual body declared in a pack's `citeSyntax.rules`,
    /// identified by its heading template (`"Rule {}"`).
    Rules(String),
    /// United States Code, title `.0`.
    Usc(u16),
    /// Code of Federal Regulations, title `.0`.
    Cfr(u16),
    /// A case citation ("*Bowles v. Russell*, 551 U.S. 205 (2007)") — not
    /// checkable against an L0 rule/statute source; recognized so it is
    /// skipped rather than misreported as a malformed rule cite.
    Case,
    /// Recognized text that isn't a rule/statute/case citation (a Supreme
    /// Court Rule, a sentencing guideline, doctrinal shorthand like "Fintiv
    /// factor 4") or text this parser doesn't understand at all.
    Other,
}

/// One normalized citation: a family plus a section/rule number and any
/// subsection chain (`"12(b)(6)"` → section `"12"`, subsections
/// `["b", "6"]`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CiteRef {
    /// Body of law.
    pub family: Family,
    /// Rule/section number, as written (may contain a dot: `"1.53"`,
    /// `"42.108"`; may contain a range dash: `"302–303"`).
    pub section: String,
    /// Chained parenthetical/subsection markers, in order (`["d", "2", "B"]`).
    pub subsections: Vec<String>,
    /// The original text this was parsed from (for diagnostics).
    pub raw: String,
}

impl CiteRef {
    /// The canonical `## <heading>` key this citation resolves to in the
    /// vendored corpus, e.g. `"Rule 12"`, `"28 U.S.C. § 1291"`,
    /// `"37 C.F.R. § 42.108"`. `None` for [`Family::Case`]/[`Family::Other`],
    /// which have no corpus to resolve against.
    #[must_use]
    pub fn heading(&self) -> Option<String> {
        match &self.family {
            Family::Rules(heading) => Some(heading.replacen("{}", &self.section, 1)),
            Family::Usc(t) if *t > 0 => Some(format!("{t} U.S.C. § {}", self.section)),
            Family::Cfr(t) if *t > 0 => Some(format!("{t} C.F.R. § {}", self.section)),
            Family::Usc(_) | Family::Cfr(_) | Family::Case | Family::Other => None,
        }
    }

    /// Whether this citation is a kind [`parse_cite_string`] recognized but
    /// deliberately never verifies against the L0 corpus (a case citation,
    /// or text with no recognized rule/statute shape).
    #[must_use]
    pub fn is_out_of_scope(&self) -> bool {
        matches!(self.family, Family::Case | Family::Other)
    }
}

/// A pack's [`CiteSyntax`], ready to apply: every rule prefix paired with
/// its body's heading template, plus the pack's shorthands, and the corpus
/// its headings resolve in when the caller has one.
#[derive(Debug, Clone, Default)]
pub struct CiteGrammar<'a> {
    rules: Vec<(&'a str, &'a str)>,
    shorthands: &'a [CiteShorthand],
    corpus: Option<&'a SourceCorpus>,
}

impl<'a> CiteGrammar<'a> {
    /// The grammar of one pack's declared citation syntax. Prefixes are
    /// tried longest first; `Pack::from_json` has already rejected
    /// overlapping prefixes, so the order never changes a result.
    #[must_use]
    pub fn new(syntax: &'a CiteSyntax) -> Self {
        let mut rules: Vec<(&str, &str)> = syntax
            .rules
            .iter()
            .flat_map(|f| f.prefixes.iter().map(|p| (p.as_str(), f.heading.as_str())))
            .collect();
        rules.sort_by(|a, b| b.0.len().cmp(&a.0.len()).then(a.0.cmp(b.0)));
        CiteGrammar {
            rules,
            shorthands: &syntax.shorthands,
            corpus: None,
        }
    }

    /// This grammar, reading a hyphenated section as one section rather
    /// than a range wherever `corpus` has that section's own heading
    /// (`N.D. Cal. Civil L.R. 7-11` is a rule, not Rules 7 through 11).
    #[must_use]
    pub fn with_corpus(mut self, corpus: Option<&'a SourceCorpus>) -> Self {
        self.corpus = corpus;
        self
    }

    /// The rule prefix `piece` is written with and the text after it,
    /// when `piece` continues one of this grammar's rule prefixes without
    /// a word break (`SDCL 15-6-4(` then `z)`): text there is the
    /// citation's own section or pinpoint, never prose that merely starts
    /// with the prefix (`RCFC Appendix A`).
    #[must_use]
    pub fn glued_rule_prefix<'p>(&self, piece: &'p str) -> Option<(&'a str, &'p str)> {
        self.rules.iter().find_map(|&(prefix, _)| {
            let rest = &piece[prefix_byte_len(piece, prefix)?..];
            let glued = !prefix.ends_with(char::is_whitespace)
                && rest.starts_with(|c: char| !c.is_whitespace());
            glued.then_some((prefix, rest))
        })
    }
}

/// Split `raw` into one [`CiteRef`] per citation and normalize each under
/// `grammar` (the citing pack's declared rule bodies and shorthands).
#[must_use]
pub fn parse_cite_string(raw: &str, grammar: &CiteGrammar<'_>) -> Vec<CiteRef> {
    let mut out = vec![];
    let mut last_family: Option<Family> = None;
    for segment in split_outside_prefixes(raw, ';', false, grammar) {
        let segment = segment.trim();
        if segment.is_empty() {
            continue;
        }
        // A case citation's own reporter cite ("551 U.S. 205 (2007)") is
        // comma-separated from the case name — never split it up.
        if looks_like_case_citation(segment) {
            out.extend(expand_range(
                parse_one(segment, last_family.as_ref(), grammar),
                grammar,
            ));
            continue;
        }
        for piece in split_outside_prefixes(segment, ',', true, grammar) {
            let piece = piece.trim();
            if piece.is_empty() {
                continue;
            }
            if let Some(rest) = piece.strip_prefix('(') {
                // A bare parenthetical continuation ("(c)" in "§ 3142(b), (c)")
                // extends the previous CiteRef's subsection chain.
                if let Some(prev) = out.last_mut() {
                    let inner = rest.trim_end_matches(')').trim();
                    if !inner.is_empty() {
                        let cref: &mut CiteRef = prev;
                        cref.subsections.push(inner.to_string());
                    }
                    continue;
                }
            }
            let cref = parse_one(piece, last_family.as_ref(), grammar);
            if !cref.is_out_of_scope() {
                last_family = Some(cref.family.clone());
            } else if bare_number(piece).is_none() {
                // This piece had its own (unrecognized-family or case-style)
                // text, not just a bare trailing number hoping to inherit
                // context ("Sup. Ct. R. 13.1" in "28 U.S.C. § 2101(c); Sup.
                // Ct. R. 13.1, 13.3") — it breaks the carry-forward chain, so
                // the NEXT bare number ("13.3") isn't wrongly attributed to
                // an unrelated family from an earlier segment.
                last_family = None;
            }
            out.extend(expand_range(cref, grammar));
        }
    }
    out
}

/// Decompose a member-range citation ("FRAP 28-31", "37 C.F.R. §§
/// 42.120-42.121", "18 U.S.C. §§ 3161-3162") into one [`CiteRef`] per member
/// section, so each resolves against the corpus independently instead of the
/// range being treated as one literal, unmatched heading. A citation with its
/// own subsections (`"12(b)(6)-(7)"`, not a form any pack in this repo
/// actually uses) is left alone — which subsection each range member binds to
/// is ambiguous, so guessing would be worse than not expanding. So is a
/// hyphenated section the grammar's corpus has a heading for (`Civil L.R.
/// 7-11` in a body numbered `<chapter>-<rule>`), and, against a corpus, one
/// whose ends aren't both headings there (`MLBR App. 1 Rule 13-99` is one
/// unknown rule, not Rules 13 through 99). Anything that isn't a plain
/// member-member range (not a range at all, spans more than 99 members, or
/// the two sides don't share a family/prefix) is returned unchanged.
fn expand_range(cref: CiteRef, grammar: &CiteGrammar<'_>) -> Vec<CiteRef> {
    if cref.is_out_of_scope() || !cref.subsections.is_empty() {
        return vec![cref];
    }
    if let (Some(corpus), Some(heading)) = (grammar.corpus, cref.heading()) {
        if !corpus.lookup_all(&heading).is_empty() {
            return vec![cref];
        }
    }
    let Some((sep, seplen)) = find_range_separator(&cref.section) else {
        return vec![cref];
    };
    let lo = cref.section[..sep].trim();
    let hi = cref.section[sep + seplen..].trim();
    let Some(members) = range_members(lo, hi) else {
        return vec![cref];
    };
    if let Some(corpus) = grammar.corpus {
        let missing = |section: Option<&String>| {
            section
                .and_then(|section| {
                    CiteRef {
                        section: section.clone(),
                        ..cref.clone()
                    }
                    .heading()
                })
                .is_none_or(|heading| corpus.lookup_all(&heading).is_empty())
        };
        if missing(members.first()) || missing(members.last()) {
            return vec![cref];
        }
    }
    members
        .into_iter()
        .map(|section| CiteRef {
            section,
            ..cref.clone()
        })
        .collect()
}

/// The byte offset and length of a `-`/`–`/`—` separator in `s`, if any.
fn find_range_separator(s: &str) -> Option<(usize, usize)> {
    s.char_indices()
        .find(|(_, c)| matches!(c, '-' | '\u{2013}' | '\u{2014}'))
        .map(|(i, c)| (i, c.len_utf8()))
}

/// Inclusive member list from `lo` to `hi`, capped at 100 members. Handles a
/// plain integer range (`"28".."31"`) and a dotted range sharing a prefix
/// (`"42.120".."42.121"`, or `"42.120".."121"` — legal writing often drops
/// the repeated prefix on the high side).
fn range_members(lo: &str, hi: &str) -> Option<Vec<String>> {
    if let (Ok(a), Ok(b)) = (lo.parse::<u32>(), hi.parse::<u32>()) {
        return (b >= a && b - a < 100).then(|| (a..=b).map(|n| n.to_string()).collect());
    }
    let (prefix, lo_last) = lo.rsplit_once('.')?;
    let a: u32 = lo_last.parse().ok()?;
    let hi_full;
    let hi = if hi.contains('.') {
        hi
    } else {
        hi_full = format!("{prefix}.{hi}");
        &hi_full
    };
    let (hi_prefix, hi_last) = hi.rsplit_once('.')?;
    if hi_prefix != prefix {
        return None;
    }
    let b: u32 = hi_last.parse().ok()?;
    (b >= a && b - a < 100).then(|| (a..=b).map(|n| format!("{prefix}.{n}")).collect())
}

/// Split `s` on `sep`, never inside one of `grammar`'s rule prefixes
/// (`MLBR App. 1, Rule 13-1` is one citation, not `MLBR App. 1` and
/// `Rule 13-1`) and, when `parens`, never inside `(...)` (so `"§ 3142(b),
/// (c)"` splits into `["§ 3142(b)", "(c)"]`, not something that severs the
/// parenthetical).
fn split_outside_prefixes<'s>(
    s: &'s str,
    sep: char,
    parens: bool,
    grammar: &CiteGrammar<'_>,
) -> Vec<&'s str> {
    let mut out = vec![];
    let mut depth = 0i32;
    let mut start = 0usize;
    let mut skip_to = 0usize;
    let mut prev: Option<char> = None;
    for (i, c) in s.char_indices() {
        let at_word = !prev.is_some_and(char::is_alphanumeric);
        prev = Some(c);
        if i < skip_to {
            continue;
        }
        if at_word {
            let held = grammar
                .rules
                .iter()
                .filter(|(p, _)| p.contains(sep))
                .find_map(|&(p, _)| prefix_byte_len(&s[i..], p));
            if let Some(n) = held {
                skip_to = i + n;
                continue;
            }
        }
        match c {
            '(' if parens => depth += 1,
            ')' if parens => depth -= 1,
            c if c == sep && depth <= 0 => {
                out.push(&s[start..i]);
                start = i + c.len_utf8();
            }
            _ => {}
        }
    }
    out.push(&s[start..]);
    out
}

fn parse_one(piece: &str, last_family: Option<&Family>, grammar: &CiteGrammar<'_>) -> CiteRef {
    if let Some(cref) = try_rule_family(piece, grammar) {
        return cref;
    }
    if let Some(cref) = try_usc_or_cfr(piece) {
        return cref;
    }
    if let Some(cref) = grammar
        .shorthands
        .iter()
        .find_map(|sh| try_shorthand(piece, sh))
    {
        return cref;
    }
    if looks_like_case_citation(piece) {
        return CiteRef {
            family: Family::Case,
            section: piece.to_string(),
            subsections: vec![],
            raw: piece.to_string(),
        };
    }
    if let Some(family) = last_family.filter(|f| !f.is_carry_forward_blocked()) {
        if let Some((section, subsections)) = bare_number(piece) {
            return CiteRef {
                family: family.clone(),
                section,
                subsections,
                raw: piece.to_string(),
            };
        }
    }
    CiteRef {
        family: Family::Other,
        section: piece.to_string(),
        subsections: vec![],
        raw: piece.to_string(),
    }
}

impl Family {
    /// `Case`/`Other` never seed a "bare trailing number" carry-forward —
    /// a stray "10" after a case name isn't a page pointing back at the
    /// last real rule/statute family.
    fn is_carry_forward_blocked(&self) -> bool {
        matches!(self, Family::Case | Family::Other)
    }
}

/// A citation written with one of `grammar`'s rule prefixes, then an
/// optional `§` and a section number. A matching prefix with no number
/// after it is not a rule citation at all (`None`), whatever else follows.
fn try_rule_family(piece: &str, grammar: &CiteGrammar<'_>) -> Option<CiteRef> {
    let (rest, heading) = grammar
        .rules
        .iter()
        .find_map(|&(prefix, heading)| strip_prefix_ci(piece, &[prefix]).map(|r| (r, heading)))?;
    let rest = rest.trim_start_matches('§').trim();
    let (section, subsections) = bare_number(rest)?;
    Some(CiteRef {
        family: Family::Rules(heading.to_string()),
        section,
        subsections,
        raw: piece.to_string(),
    })
}

/// `"28 U.S.C. § 1291"`, `"35 USC 315(e)"`, `"37 C.F.R. § 42.108"`, `"37 CFR
/// 1.104"` — a leading title number, a U.S.C./C.F.R. marker, an optional
/// §/§§, then a section number.
fn try_usc_or_cfr(piece: &str) -> Option<CiteRef> {
    let digit_end = piece.find(|c: char| !c.is_ascii_digit())?;
    if digit_end == 0 {
        return None;
    }
    let title: u16 = piece[..digit_end].parse().ok()?;
    let rest = piece[digit_end..].trim_start();
    let (make_family, rest): (fn(u16) -> Family, &str) =
        if let Some(r) = strip_prefix_ci(rest, &["U.S.C.", "USC"]) {
            (Family::Usc, r)
        } else {
            let r = strip_prefix_ci(rest, &["C.F.R.", "CFR"])?;
            (Family::Cfr, r)
        };
    let rest = rest
        .trim_start()
        .trim_start_matches("§§")
        .trim_start_matches('§')
        .trim_start();
    let (section, subsections) = bare_number(rest)?;
    Some(CiteRef {
        family: make_family(title),
        section,
        subsections,
        raw: piece.to_string(),
    })
}

/// A pack-declared compact dotted form (`"cfr210.16.b.4"` with
/// `sectionParts: 2` = 19 C.F.R. § 210.16(b)(4) when the shorthand implies
/// title 19): the first `sectionParts` dotted parts are the section (the
/// first all digits, the rest non-empty), every later non-empty part a
/// subsection.
fn try_shorthand(piece: &str, sh: &CiteShorthand) -> Option<CiteRef> {
    let rest = strip_prefix_ci(piece, &[sh.prefix.as_str()])?;
    let mut parts = rest.split('.');
    let lead = parts.next()?;
    if lead.is_empty() || !lead.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    let mut section = lead.to_string();
    for _ in 1..sh.section_parts {
        let part = parts.next().unwrap_or("");
        if part.is_empty() {
            return None;
        }
        section.push('.');
        section.push_str(part);
    }
    let subsections: Vec<String> = parts
        .map(str::to_string)
        .filter(|s| !s.is_empty())
        .collect();
    let family = match sh.code {
        CiteCode::Usc => Family::Usc(sh.title),
        CiteCode::Cfr => Family::Cfr(sh.title),
    };
    Some(CiteRef {
        family,
        section,
        subsections,
        raw: piece.to_string(),
    })
}

/// Case-insensitive prefix match against any of `prefixes`, tolerant of runs
/// of whitespace in `s` where the prefix has a single space. Returns the
/// remainder. Byte-scans `s` directly (never slices by a prefix's byte
/// length), so it's safe on non-ASCII input like a following `§`.
fn strip_prefix_ci<'a>(s: &'a str, prefixes: &[&str]) -> Option<&'a str> {
    for &p in prefixes {
        if let Some(off) = prefix_byte_len(s, p) {
            return Some(s[off..].trim_start());
        }
    }
    None
}

/// Case-insensitive, whitespace-tolerant match of literal `prefix` at the
/// start of `s`; returns how many bytes of `s` it consumed.
fn prefix_byte_len(s: &str, prefix: &str) -> Option<usize> {
    let sb = s.as_bytes();
    let pb = prefix.as_bytes();
    let mut si = 0usize;
    let mut pi = 0usize;
    while pi < pb.len() {
        if pb[pi] == b' ' {
            // one-or-more spaces in `s` for one space in `prefix`
            let mut n = 0;
            while si < sb.len() && sb[si] == b' ' {
                si += 1;
                n += 1;
            }
            if n == 0 {
                return None;
            }
            pi += 1;
            continue;
        }
        if si >= sb.len() || !sb[si].eq_ignore_ascii_case(&pb[pi]) {
            return None;
        }
        si += 1;
        pi += 1;
    }
    Some(si)
}

/// Parse `"12(b)(6)"` → `("12", ["b", "6"])`, `"1441"` → `("1441", [])`.
fn bare_number(s: &str) -> Option<(String, Vec<String>)> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }
    let first = s.chars().next()?;
    if !(first.is_ascii_digit() || s.starts_with('(')) {
        return None;
    }
    if let Some(idx) = s.find('(') {
        let section = s[..idx].trim().to_string();
        if section.is_empty() {
            return None;
        }
        let rest = s[idx..].trim();
        let inner = rest
            .strip_prefix('(')
            .and_then(|r| r.strip_suffix(')'))
            .unwrap_or(rest.trim_start_matches('(').trim_end_matches(')'));
        let subs: Vec<String> = inner
            .split(")(")
            .map(str::trim)
            .filter(|x| !x.is_empty())
            .map(str::to_string)
            .collect();
        Some((section, subs))
    } else {
        Some((s.to_string(), vec![]))
    }
}

fn looks_like_case_citation(s: &str) -> bool {
    s.contains(" v. ") || s.contains(" v ")
}

#[cfg(test)]
#[path = "normalize_tests.rs"]
mod tests;
