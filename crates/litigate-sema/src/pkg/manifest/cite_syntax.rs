//! `[[cite_syntax.rules]]` and `[[cite_syntax.shorthands]]` (SPEC §4.1),
//! mirroring litgraph's pack schema v3 `citeSyntax`.
use super::{range, Checker, Ordered, RawSource};
use crate::hir::{CiteRule, CiteShorthand, CiteSyntax};
use serde::Deserialize;
use toml::Spanned;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RawCiteSyntax {
    #[serde(default)]
    rules: Vec<Spanned<RawRule>>,
    #[serde(default)]
    shorthands: Vec<Spanned<RawShorthand>>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawRule {
    prefixes: Vec<String>,
    heading: Spanned<String>,
    source: Option<Spanned<String>>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawShorthand {
    prefix: String,
    code: Spanned<String>,
    title: u32,
    section_parts: u32,
    source: Option<Spanned<String>>,
}

/// Validates the tables: non-empty prefixes, exactly one `{}` per heading,
/// `usc`/`cfr` codes, and `source` keys naming a `[sources.*]` table.
pub(super) fn check(
    c: &mut Checker,
    raw: &RawCiteSyntax,
    sources: &Ordered<Spanned<RawSource>>,
) -> CiteSyntax {
    let bound = |c: &mut Checker, source: Option<&Spanned<String>>| {
        let source = source?;
        let id = source.get_ref();
        c.check(sources.0.iter().any(|(k, _)| k == id), range(source), || {
            format!("`source = \"{id}\"` names no `[sources.{id}]` table")
        });
        Some(id.clone())
    };
    let rules = raw
        .rules
        .iter()
        .map(|rule| {
            let r = rule.get_ref();
            c.check(
                !r.prefixes.is_empty() && r.prefixes.iter().all(|p| !p.trim().is_empty()),
                range(rule),
                || "a cite_syntax rule needs non-empty `prefixes`".to_owned(),
            );
            c.check(
                r.heading.get_ref().matches("{}").count() == 1,
                range(&r.heading),
                || "`heading` must contain exactly one `{}`".to_owned(),
            );
            CiteRule {
                prefixes: r.prefixes.clone(),
                heading: r.heading.get_ref().clone(),
                source: bound(c, r.source.as_ref()),
            }
        })
        .collect();
    let shorthands = raw
        .shorthands
        .iter()
        .map(|shorthand| {
            let s = shorthand.get_ref();
            c.check(!s.prefix.trim().is_empty(), range(shorthand), || {
                "a cite_syntax shorthand needs a non-empty `prefix`".to_owned()
            });
            let code = s.code.get_ref();
            c.check(code == "usc" || code == "cfr", range(&s.code), || {
                format!("`code` must be \"usc\" or \"cfr\", found `{code}`")
            });
            CiteShorthand {
                prefix: s.prefix.clone(),
                code: code.clone(),
                title: s.title,
                section_parts: s.section_parts,
                source: bound(c, s.source.as_ref()),
            }
        })
        .collect();
    CiteSyntax { rules, shorthands }
}
