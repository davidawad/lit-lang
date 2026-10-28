//! The `*.map.json` sidecar (SPEC §8.6): every generated litgraph id back
//! to its `.lit` span, declaration version, and the L0 source sections its
//! citation resolves to, plus the `generated` marker. Sidecars are excluded
//! from semantic equality.
//!
//! A resolved citation is structured (SPEC §4.3): the work (the source
//! binding), its expression (the binding's `as_of`, whose text the
//! binding's `path_sha256` pins) and the pinpoint, with the SHA-256 of the
//! cited section's text once that text is verified against the pin.
use litigate_base::Span;
use litigate_engine::cite::{parse_cite_string, CiteGrammar, Family, SourceCorpus};
use litigate_engine::model::{CiteCode, CiteRuleForm, CiteSyntax};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fmt::Write as _;

pub mod history;
pub use history::{Amendment, History, Proof};

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
    /// Date of the cited expression: the binding's `as_of`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub as_of: Option<String>,
    /// The pinpoint below the section (`["b", "6"]` for `12(b)(6)`).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub pinpoint: Vec<String>,
    /// SHA-256 (lowercase hex) of the section's text, the lines after its
    /// `## <heading>` line up to the next `## ` line, trimmed. Present
    /// only when `path`'s text is the one the binding's `path_sha256` pins.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sha256: Option<String>,
    /// The amendment history proving the text, dated after the citing
    /// version took effect, was its text then (see [`history`]).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub history: Option<History>,
}

/// How a binding's vendored file checks against its `path_sha256` pin.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Pin {
    /// The binding has no `path`: nothing to verify.
    NoPath,
    /// `path` is not in the corpus.
    Absent,
    /// The binding pins no `path_sha256`; `actual` is the file's hash.
    Unpinned {
        /// SHA-256 of the corpus file.
        actual: String,
    },
    /// The corpus file is not the pinned text.
    Mismatch {
        /// The binding's `path_sha256`.
        pinned: String,
        /// SHA-256 of the corpus file.
        actual: String,
    },
    /// The corpus file is the pinned text.
    Verified,
}

/// Lowercase hex SHA-256 of `text`.
#[must_use]
pub fn sha256_hex(text: &str) -> String {
    Sha256::digest(text.as_bytes())
        .iter()
        .fold(String::with_capacity(64), |mut s, b| {
            let _ = write!(s, "{b:02x}");
            s
        })
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
    /// A links document's patched guards (SPEC §3.6), by instance then
    /// edge: the provenance of each guard operation.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub guards: Vec<GuardTrace>,
}

/// One patched edge's guard and the typed operations on it (SPEC §3.6),
/// so an alternate timeline can diff guards between change-file sets.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct GuardTrace {
    /// The patch's JSON pointer (`/instances/<name>/patch_edges/<edge>`).
    pub pointer: String,
    /// The base edge's own guard, in the base pack's view; absent when it
    /// has none.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base: Option<String>,
    /// Every operation, in the order they apply (authority, then
    /// effective date; never source order).
    pub ops: Vec<GuardOpTrace>,
    /// The guard in force at the build dates; absent when there is none.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub composed: Option<String>,
}

/// One guard operation with its authority.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct GuardOpTrace {
    /// `narrow`, `widen` or `replace`.
    pub op: String,
    /// The condition, in the base pack's view.
    pub condition: String,
    /// The instrument's authority level.
    pub by: String,
    /// The instrument; a hole's reason when unverified (it then never
    /// applies).
    pub cite: String,
    /// First day in force.
    pub from: String,
    /// First day no longer in force; absent when open.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub to: Option<String>,
    /// When the instrument became known.
    pub known_at: String,
    /// Whether it is in force at the build dates.
    pub applied: bool,
    /// Declaring source span.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub span: Option<Span>,
}

/// A heading template or shorthand family, the source id bound to it, its
/// vendored path and the prefixes it is cited with.
type Binding = (Family, String, Option<String>, Vec<String>);

/// Resolves citations under one pack's citation syntax and source
/// bindings against an optional vendored corpus.
pub struct Citations<'c> {
    syntax: CiteSyntax,
    /// Heading template or shorthand code/title to bound source id, its
    /// vendored path and the prefixes it is cited with. Several bindings
    /// may share one template (19 C.F.R. parts 201 and 210 in separate
    /// files; two rule bodies both headed `Rule {n}`); a citation goes to
    /// those written with its prefix, and of them the one whose file
    /// holds the heading wins.
    bound: Vec<Binding>,
    /// Source id to its binding's expression date, vendored path, and how
    /// that file checks against its pin.
    pins: BTreeMap<String, Bound>,
    corpus: Option<&'c SourceCorpus>,
}

impl<'c> Citations<'c> {
    /// The citations of `pkg`: its `[[cite_syntax.*]]` and source
    /// bindings, located in `corpus` when given. The pack's own lowering
    /// reports a bad citation syntax.
    #[must_use]
    pub fn of_package(pkg: &litigate_sema::hir::Package, corpus: Option<&'c SourceCorpus>) -> Self {
        let syntax = crate::pack::cite_syntax(&mut crate::diag::Sink::new(""), &pkg.manifest);
        Self::new(syntax, &pkg.manifest.sources, corpus)
    }

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
        // A pack rule's own prefixes, before any source's are merged in.
        let mut bound: Vec<Binding> = syntax
            .rules
            .iter()
            .filter_map(|r| {
                let id = r.source.clone()?;
                let p = path(&id);
                Some((Family::Rules(r.heading.clone()), id, p, r.prefixes.clone()))
            })
            .collect();
        for s in sources.iter().filter(|s| !s.cite_prefixes.is_empty()) {
            let heading = s
                .section_heading
                .as_deref()
                .map_or_else(|| "{}".to_owned(), |h| h.replace("{n}", "{}"));
            // A heading another rule already reads also takes this
            // source's prefixes, so its citations parse and are checked.
            if let Some(rule) = syntax.rules.iter_mut().find(|r| r.heading == heading) {
                for p in &s.cite_prefixes {
                    if !rule.prefixes.iter().any(|q| q.eq_ignore_ascii_case(p)) {
                        rule.prefixes.push(p.clone());
                    }
                }
            } else {
                syntax.rules.push(CiteRuleForm {
                    prefixes: s.cite_prefixes.clone(),
                    heading: heading.clone(),
                    source: Some(s.id.clone()),
                });
            }
            bound.push((
                Family::Rules(heading),
                s.id.clone(),
                s.path.clone(),
                s.cite_prefixes.clone(),
            ));
        }
        let shorthands = syntax.shorthands.iter().filter_map(|s| {
            let family = match s.code {
                CiteCode::Usc => Family::Usc(s.title),
                CiteCode::Cfr => Family::Cfr(s.title),
            };
            let id = s.source.clone()?;
            let p = path(&id);
            Some((family, id, p, Vec::new()))
        });
        bound.extend(shorthands);
        let mut hashes = BTreeMap::new();
        let mut hash = |path: &String| -> Option<String> {
            let text = corpus?.files.get(path)?;
            Some(
                hashes
                    .entry(path.clone())
                    .or_insert_with(|| sha256_hex(text))
                    .clone(),
            )
        };
        let pins = sources
            .iter()
            .map(|s| {
                let actual = s.path.as_ref().and_then(&mut hash);
                let bound = Bound {
                    as_of: s.as_of.clone(),
                    path: s.path.clone(),
                    pin: pin(s.path.is_some(), s.path_sha256.as_ref(), actual),
                };
                (s.id.clone(), bound)
            })
            .collect();
        Self {
            syntax,
            bound,
            pins,
            corpus,
        }
    }

    /// How source `id`'s vendored file checks against its pin (an
    /// undeclared id has no path).
    #[must_use]
    pub fn pin(&self, id: &str) -> &Pin {
        self.pins.get(id).map_or(&Pin::NoPath, |b| &b.pin)
    }

    /// The citation syntax in effect: the pack's own plus one rule per
    /// source binding with `cite_prefixes`.
    #[must_use]
    pub fn syntax(&self) -> &CiteSyntax {
        &self.syntax
    }

    /// The grammar of [`Self::syntax`], reading hyphenated sections against
    /// the corpus when there is one.
    #[must_use]
    pub fn grammar(&self) -> CiteGrammar<'_> {
        CiteGrammar::new(&self.syntax).with_corpus(self.corpus)
    }

    /// The map of `text`: litgraph's own citation parser, then the bound
    /// source and a corpus lookup of each heading.
    #[must_use]
    pub fn map(&self, text: Option<&String>) -> Option<CiteMap> {
        let text = text?;
        let grammar = self.grammar();
        // The prefix the last prefixed citation was written with, which a
        // bare trailing number (`MLBRA 1, 2`) carries forward.
        let mut written: Option<&str> = None;
        let sections = parse_cite_string(text, &grammar)
            .iter()
            .filter_map(|c| {
                let heading = c.heading()?;
                if let Some(prefix) = grammar.rule_prefix(&c.raw) {
                    written = Some(prefix);
                }
                let of_family = || self.bound.iter().filter(|(f, ..)| *f == c.family);
                let by_prefix = |(_, _, _, ps): &&Binding| {
                    written.is_some_and(|w| ps.iter().any(|p| p.eq_ignore_ascii_case(w)))
                };
                let mut candidates: Vec<_> = of_family().filter(by_prefix).collect();
                if candidates.is_empty() {
                    candidates = of_family().collect();
                }
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
                    .find(|(_, _, p, _)| holds(p))
                    .or_else(|| candidates.first())
                    .map(|(_, s, ..)| s.clone());
                Some(self.locate(heading, c.subsections.clone(), source))
            })
            .collect();
        Some(CiteMap {
            text: text.clone(),
            sections,
        })
    }

    /// `heading` in the file of the binding `source` resolves through.
    fn locate(&self, heading: String, pinpoint: Vec<String>, source: Option<String>) -> L0Section {
        let bound = source.as_ref().and_then(|id| self.pins.get(id));
        let path = bound.and_then(|b| b.path.as_ref());
        let section = self.corpus.zip(path).and_then(|(c, p)| {
            let (_, section) = c.lookup_all(&heading).iter().find(|(f, _)| f == p)?;
            // The corpus indexed the heading from this file, so its line is there.
            let text = c.files.get(p).map_or("", String::as_str);
            let line = text
                .lines()
                .position(|l| l.strip_prefix("## ").is_some_and(|h| h.trim() == heading))
                .map_or(0, |i| i + 1);
            Some((u32::try_from(line).unwrap_or(u32::MAX), section))
        });
        let verified = bound.is_some_and(|b| b.pin == Pin::Verified);
        L0Section {
            path: section.and(path.cloned()),
            line: section.map(|(line, _)| line),
            as_of: bound.and_then(|b| b.as_of.clone()),
            sha256: section
                .filter(|_| verified)
                .map(|(_, s)| sha256_hex(&s.body)),
            heading,
            source,
            pinpoint,
            history: None,
        }
    }

    /// [`Citations::map`] for a cite of a version in force from
    /// `effective`: each section a history proves carries that history.
    #[must_use]
    pub fn map_at(&self, text: Option<&String>, effective: Option<&str>) -> Option<CiteMap> {
        let mut map = self.map(text)?;
        for s in &mut map.sections {
            if let Some(Proof::Holds(h)) = effective.map(|e| self.prove(s, e)) {
                s.history = Some(h);
            }
        }
        Some(map)
    }
}

/// A source binding as citations see it.
struct Bound {
    as_of: Option<String>,
    path: Option<String>,
    pin: Pin,
}

/// A binding's file (`actual` its hash, when in the corpus) against its
/// `path_sha256`.
fn pin(has_path: bool, pinned: Option<&String>, actual: Option<String>) -> Pin {
    match (has_path, actual, pinned) {
        (false, _, _) => Pin::NoPath,
        (true, None, _) => Pin::Absent,
        (true, Some(actual), None) => Pin::Unpinned { actual },
        (true, Some(actual), Some(pinned)) if *pinned != actual => Pin::Mismatch {
            pinned: pinned.clone(),
            actual,
        },
        (true, Some(_), Some(_)) => Pin::Verified,
    }
}
