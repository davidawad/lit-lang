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
    // A pinpoint the corpus does not section apart (`LR Crim. 32A(d)`
    // under `LR Crim. 32A`, S.D. Ga.) that states no count of the measure
    // sets its period in words (`the day prior to`), not by a sibling
    // subdivision's count (`Within 14 days`, 32A(c)).
    let uncounted = sections.iter().any(|s| {
        let Some((k, body)) = (0..=s.pinpoint.len()).rev().find_map(|k| {
            let heading = s.pinpoint[..k]
                .iter()
                .fold(s.heading.clone(), |h, p| format!("{h}({p})"));
            corpus
                .lookup_all(&heading)
                .iter()
                .find(|(file, _)| s.path.as_ref() == Some(file))
                .map(|(_, section)| (k, unpaged(&section.body).into_owned()))
        }) else {
            return false;
        };
        k < s.pinpoint.len()
            && !counts(&subdivision(&body, &s.pinpoint[k..]))
                .iter()
                .any(|(_, m)| m.monthly() == family.monthly())
    });
    if uncounted {
        return None;
    }
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
