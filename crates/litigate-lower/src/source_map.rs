//! The `*.map.json` sidecar (SPEC §8.6): every generated litgraph id back
//! to its `.lit` span, declaration version, and the L0 source sections its
//! citation resolves to, plus the `generated` marker. Sidecars are excluded
//! from semantic equality.
use litgraph::cite::{parse_cite_string, CiteGrammar, Family, SourceCorpus};
use litgraph::model::{CiteCode, CiteRuleForm, CiteSyntax};
use litigate_base::Span;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Marks a file as compiler output: who generated it, from which language
/// version. Deterministic (no timestamps, no absolute paths).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct Generated {
    /// Always `litigate-lower`.
    pub by: String,
    /// Compiler version.
    pub version: String,
    /// Language version (SPEC §4.1 `language`).
    pub language: u32,
}

impl Default for Generated {
    fn default() -> Self {
        Self {
            by: "litigate-lower".to_owned(),
            version: env!("CARGO_PKG_VERSION").to_owned(),
            language: 1,
        }
    }
}

/// The package a document was lowered from.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct PackageRef {
    /// Package name.
    pub name: String,
    /// Exact version.
    pub version: String,
    /// `sha256:<hex>` source digest (SPEC §4.2).
    pub digest: String,
}

/// One citation of a `cite`/`authority` string, located in the L0 corpus.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct L0Section {
    /// Canonical corpus heading (`Rule 12`, `28 U.S.C. § 1291`).
    pub heading: String,
    /// The `[sources.*]` binding it resolves through, when bound.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    /// Repository-relative corpus file holding it, when vendored.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    /// One-based line of its `## <heading>` line in `path`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line: Option<u32>,
}

/// A cite string and where each of its citations is in L0.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct CiteMap {
    /// The emitted text.
    pub text: String,
    /// Each recognized rule/statute citation, in order.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sections: Vec<L0Section>,
}

/// One generated value and its origin.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct Entry {
    /// JSON pointer into the generated document (`/nodes/3`).
    pub pointer: String,
    /// The litgraph id there (`answer-due`, `respond->answered#0`).
    pub id: String,
    /// Declaring source span.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub span: Option<Span>,
    /// The emitted version, `id@in-force-from`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub decl: Option<String>,
    /// The `cite` (nodes) or `authority` (edges), located in L0.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cite: Option<CiteMap>,
}

/// A generated document's sidecar.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct SourceMap {
    /// The generated-file marker.
    pub generated: Generated,
    /// Document kind: `pack`, `links`, `clock`, or `scenario`.
    pub kind: String,
    /// Pack or scenario id (empty for links).
    pub id: String,
    /// Source package (absent for a case).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub package: Option<PackageRef>,
    /// Law-in-force date the build selected versions at.
    pub as_of: String,
    /// Corpus date the build selected versions at.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub known_at: Option<String>,
    /// Entries in document order.
    pub entries: Vec<Entry>,
}

/// Resolves citations under one pack's citation syntax and source
/// bindings against an optional vendored corpus.
pub struct Citations<'c> {
    syntax: CiteSyntax,
    /// Heading template or shorthand code/title to bound source id and
    /// its vendored path. Several bindings may share one template (19
    /// C.F.R. parts 201 and 210 in separate files); the one whose file
    /// holds the heading wins.
    bound: Vec<(Family, String, Option<String>)>,
    corpus: Option<&'c SourceCorpus>,
}

impl<'c> Citations<'c> {
    /// Citations written in the pack's `syntax` or bound by a source's
    /// `cite_prefixes` (with its `section_heading`, `{n}` the section;
    /// SPEC §4.3), located in `corpus` when given.
    #[must_use]
    pub fn new(
        syntax: CiteSyntax,
        sources: &[litigate_sema::hir::SourceBinding],
        corpus: Option<&'c SourceCorpus>,
    ) -> Self {
        let mut syntax = syntax;
        let path = |id: &str| {
            sources
                .iter()
                .find(|s| s.id == id)
                .and_then(|s| s.path.clone())
        };
        let mut shared = Vec::new();
        for s in sources.iter().filter(|s| !s.cite_prefixes.is_empty()) {
            let heading = s
                .section_heading
                .as_deref()
                .map_or_else(|| "{}".to_owned(), |h| h.replace("{n}", "{}"));
            if syntax.rules.iter().any(|r| r.heading == heading) {
                shared.push((Family::Rules(heading), s.id.clone(), s.path.clone()));
            } else {
                syntax.rules.push(CiteRuleForm {
                    prefixes: s.cite_prefixes.clone(),
                    heading,
                    source: Some(s.id.clone()),
                });
            }
        }
        let rules = syntax.rules.iter().filter_map(|r| {
            let id = r.source.clone()?;
            let p = path(&id);
            Some((Family::Rules(r.heading.clone()), id, p))
        });
        let shorthands = syntax.shorthands.iter().filter_map(|s| {
            let family = match s.code {
                CiteCode::Usc => Family::Usc(s.title),
                CiteCode::Cfr => Family::Cfr(s.title),
            };
            let id = s.source.clone()?;
            let p = path(&id);
            Some((family, id, p))
        });
        let bound = rules.chain(shorthands).chain(shared).collect();
        Self {
            syntax,
            bound,
            corpus,
        }
    }

    /// The citation syntax in effect: the pack's own plus one rule per
    /// source binding with `cite_prefixes`.
    #[must_use]
    pub fn syntax(&self) -> &CiteSyntax {
        &self.syntax
    }

    /// The map of `text`: litgraph's own citation parser, then the bound
    /// source and a corpus lookup of each heading.
    #[must_use]
    pub fn map(&self, text: Option<&String>) -> Option<CiteMap> {
        let text = text?;
        let grammar = CiteGrammar::new(&self.syntax);
        let sections = parse_cite_string(text, &grammar)
            .iter()
            .filter_map(|c| {
                let heading = c.heading()?;
                let candidates: Vec<_> = self
                    .bound
                    .iter()
                    .filter(|(f, _, _)| *f == c.family)
                    .collect();
                let holds = |p: &Option<String>| {
                    self.corpus.is_some_and(|corpus| {
                        corpus
                            .lookup_all(&heading)
                            .iter()
                            .any(|(file, _)| Some(file) == p.as_ref())
                    })
                };
                let source = candidates
                    .iter()
                    .find(|(_, _, p)| holds(p))
                    .or_else(|| candidates.first())
                    .map(|(_, s, _)| s.clone());
                Some(self.locate(heading, source))
            })
            .collect();
        Some(CiteMap {
            text: text.clone(),
            sections,
        })
    }

    fn locate(&self, heading: String, source: Option<String>) -> L0Section {
        let found = self
            .corpus
            .and_then(|c| Some((c, c.lookup_all(&heading).first()?)));
        let (path, line) = found.map_or((None, None), |(corpus, (path, _))| {
            let marker = format!("## {heading}");
            let line = corpus.files.get(path).and_then(|text| {
                text.lines()
                    .position(|l| l.trim_end() == marker)
                    .and_then(|i| u32::try_from(i + 1).ok())
            });
            (Some(path.clone()), line)
        });
        L0Section {
            heading,
            source,
            path,
            line,
        }
    }
}
