//! `cite-resolves` (SPEC §7): every cite resolves to a source binding and,
//! when the binding has a vendored `path`, to its pinpoint's `## heading`
//! span in that file.
//!
//! Citations are parsed exactly as the lowering's source maps parse them
//! (`litigate_lower::source_map::Citations`, litgraph's own parser under
//! the pack's citation syntax plus the bindings' `cite_prefixes`), so the
//! verdict here and the sidecar can never disagree. Case citations and
//! other text the parser treats as out of scope are not rule cites and are
//! never checked (the same policy as `litgraph lint`).
use crate::diag::{emit, note};
use crate::Context;
use litgraph::cite::{parse_cite_string, CiteGrammar, SourceCorpus};
use litgraph::model::Pack;
use litigate_base::{Diagnostic, DiagnosticCode as C, Span};
use litigate_lower::source_map::{Citations, L0Section};

/// L1010 for every citation that does not resolve.
pub(crate) fn unresolved(cx: &Context<'_>, pack: &Pack, corpus: &SourceCorpus) -> Vec<Diagnostic> {
    let sources = &cx.package.manifest.sources;
    let citations = Citations::new(pack.cite_syntax.clone(), sources, Some(corpus));
    cx.cites()
        .flat_map(|(text, span)| {
            let span = cx.span_or_manifest(span);
            let sections = citations
                .map(Some(&text.to_owned()))
                .map(|m| m.sections)
                .unwrap_or_default();
            let unkeyed = unkeyed(&citations, text, &span, cx);
            sections
                .iter()
                .filter_map(|s| section(cx, corpus, text, s, &span))
                .chain(unkeyed)
                .collect::<Vec<_>>()
        })
        .collect()
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
    let found = corpus
        .lookup_all(&s.heading)
        .iter()
        .any(|(file, _)| file == path);
    if found {
        return None;
    }
    let why = if corpus.has_file(path) {
        format!("`{path}` has no `## {}` heading", s.heading)
    } else {
        format!("`{path}` is not in the vendored source corpus")
    };
    let d = emit(
        C::UnboundCite,
        cx.profile,
        span.clone(),
        format!("`{text}` binds to source `{}`, but {why}", binding.id),
    );
    Some(note(
        d,
        None,
        "cite a pinpoint that exists in the vendored text, or mark the cite `?(\"why\")` \
         until the source is vendored",
    ))
}
