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
    if found && unverified.is_none() && subdivided(corpus, path, &s.heading, &s.pinpoint, text) {
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

#[cfg(test)]
mod measure_tests {
    use super::Measure;

    #[test]
    fn equivalent_periods_skip_week_and_year_conversions_that_overflow() {
        for unit in [Measure::Week, Measure::Year] {
            assert_eq!(unit.equivalents(u64::MAX), [(u64::MAX, unit)]);
        }
    }
}
