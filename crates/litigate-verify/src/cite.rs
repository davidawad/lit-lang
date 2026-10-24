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
//! never checked (the same policy as `litgraph lint`).
//!
//! A cite of a declaration `litigate import` marked `/// imported` is
//! exempt in `dev`: its failures are warnings, so a freshly imported pack
//! checks while its citations are ported. `strict` exempts nothing.
use crate::diag::{emit, note};
use crate::Context;
use litgraph::cite::{parse_cite_string, CiteGrammar, SourceCorpus};
use litgraph::model::Pack;
use litigate_base::{Diagnostic, DiagnosticCode as C, Profile, Severity, Span};
use litigate_lower::source_map::{Citations, L0Section, Pin};
use litigate_sema::Hole;

/// L1010 for every citation that does not resolve. `holes` are the
/// package's: a cite hole's reason is resolved like a cite.
pub(crate) fn unresolved(cx: &Context<'_>, pack: &Pack, corpus: &SourceCorpus) -> Vec<Diagnostic> {
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
            let unkeyed = unkeyed(&citations, text, &span, cx);
            sections
                .iter()
                .filter_map(|s| section(cx, &citations, corpus, text, s, &span))
                .chain(unkeyed)
                .map(|d| exempt(d, cite.imported, cx.profile))
                .collect::<Vec<_>>()
        })
        .collect()
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
            let written = holes
                .iter()
                .any(|h| h.reason.as_deref() == Some(cite.text) && within(&h.span, &span));
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

/// `inner` lies inside `outer`.
fn within(inner: &Span, outer: &Span) -> bool {
    inner.file == outer.file
        && (inner.line, inner.col) >= (outer.line, outer.col)
        && (inner.end_line, inner.end_col) <= (outer.end_line, outer.end_col)
}

/// Rule or statute citations that do not normalize to a heading at all.
fn unkeyed(
    citations: &Citations<'_>,
    text: &str,
    span: &Span,
    cx: &Context<'_>,
) -> Vec<Diagnostic> {
    parse_cite_string(text, &CiteGrammar::new(citations.syntax()))
        .into_iter()
        .filter(|r| !r.is_out_of_scope() && r.heading().is_none())
        .map(|r| {
            emit(
                C::UnboundCite,
                cx.profile,
                span.clone(),
                format!(
                    "`{}` in `{text}` does not normalize to a source heading",
                    r.raw
                ),
            )
        })
        .collect()
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
            "cite the expression the corpus holds, or vendor the pinned text".to_owned(),
        )),
        Pin::NoPath | Pin::Absent | Pin::Verified => None,
    };
    let found = corpus
        .lookup_all(&s.heading)
        .iter()
        .any(|(file, _)| file == path);
    if found && unverified.is_none() {
        return None;
    }
    let (why, fix) = unverified.unwrap_or_else(|| {
        let why = if corpus.has_file(path) {
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
