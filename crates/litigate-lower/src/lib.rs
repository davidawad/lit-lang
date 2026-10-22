//! Lowering: checked HIR to litgraph's own serde types (SPEC §8).
//!
//! - a rule package becomes a [`Pack`] (§8.1) and, when it declares its
//!   counting rule, a clock-library fragment (§8.2);
//! - a links package becomes a [`LinkFile`] (§8.4);
//! - a `.case` matter becomes a [`NamedScenario`] (§8.3).
//!
//! Output is built from litgraph's types and serialized by serde, never
//! written by hand, so it cannot drift from what the engine accepts. Every
//! document comes with a [`SourceMap`] sidecar carrying the generated
//! marker. [`render`] is byte-deterministic, and [`semantic_diff`] is the
//! SPEC §8.6 equality every port checks against its legacy JSON.
//!
//! ```
//! use litigate_lower::{render, semantic_diff};
//! use litgraph::Pack;
//!
//! let pack = Pack::from_json(r#"{"schemaVersion": 2, "id": "demo", "title": "Demo",
//!   "startNodeId": "a", "nodes": [{"id": "a", "label": "A"}], "edges": []}"#)
//!     .map_err(|e| e.to_string())?;
//! let text = render(&pack).map_err(|d| d.message)?;
//! assert!(text.ends_with("}\n"));
//! let again = Pack::from_json(&text).map_err(|e| e.to_string())?;
//! assert!(semantic_diff(&pack, &again).is_empty());
//! # Ok::<(), String>(())
//! ```
mod case;
mod clock;
mod diag;
pub mod diff;
mod edge;
mod links;
pub mod num;
mod pack;
mod select;
pub mod source_map;

pub use diff::{
    semantic_diff, semantic_diff_clock, semantic_diff_json, semantic_diff_links,
    semantic_diff_scenarios, Change, ChangeKind, DocumentKind,
};
pub use litgraph::clock::ClockLibrary;
pub use litgraph::model::{LinkFile, Pack};
pub use litgraph::scenario::NamedScenario;
pub use source_map::{Generated, SourceMap};

use diag::Sink;
use litgraph::cite::SourceCorpus;
use litigate_base::{Diagnostic, DiagnosticCode};
use litigate_sema::hir;
use select::Selection;
use serde::Serialize;

/// Build dates (SPEC §8.1): the version of every declaration in force on
/// `as_of`, among versions recorded by `known_at`. Each defaults to the
/// package's `known_at`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Options {
    /// Law-in-force date (`--as-of`).
    pub as_of: Option<String>,
    /// Corpus date (`--known-at`).
    pub known_at: Option<String>,
}

/// What a package lowers to.
#[derive(Debug, Clone, PartialEq)]
pub enum Document {
    /// A rule package's pack.
    Pack(Box<Pack>),
    /// A links package's link file.
    Links(LinkFile),
}

/// A lowered package.
#[derive(Debug, Clone, PartialEq)]
pub struct LoweredPackage {
    /// The pack or link file.
    pub document: Document,
    /// Its sidecar.
    pub source_map: SourceMap,
    /// The clock-library fragment of the package's own counting rule and
    /// calendar, with its sidecar (absent when `Pack.counting` names a
    /// rule of the shared library, or there is none).
    pub clock: Option<(ClockLibrary, SourceMap)>,
}

/// A lowered matter.
#[derive(Debug, Clone, PartialEq)]
pub struct LoweredCase {
    /// The named scenario.
    pub scenario: NamedScenario,
    /// Its sidecar.
    pub source_map: SourceMap,
}

/// Lowers the packages of one workspace. Holds every loaded package so
/// cross-package references (`within q::d`, instance bases, a case's
/// primary package) resolve.
pub struct Lowerer<'a> {
    packages: Vec<&'a hir::Package>,
    corpus: Option<&'a SourceCorpus>,
}

/// One build: the packages, the version selection, and the corpus.
pub(crate) struct Build<'s, 'a> {
    pub packages: &'s [&'a hir::Package],
    pub selection: Selection<'a>,
    pub corpus: Option<&'a SourceCorpus>,
}

impl Build<'_, '_> {
    /// The sidecar fields every package document shares.
    pub fn map_header(&self) -> SourceMap {
        SourceMap {
            generated: Generated::default(),
            kind: String::new(),
            id: String::new(),
            package: None,
            as_of: self.selection.as_of.clone(),
            known_at: Some(self.selection.known_at.clone()),
            entries: Vec::new(),
        }
    }

    /// `pkg`'s identity in a sidecar.
    pub fn package_ref(&self, pkg: &hir::Package) -> source_map::PackageRef {
        let _ = self;
        source_map::PackageRef {
            name: pkg.manifest.name.clone(),
            version: pkg.manifest.version.clone(),
            digest: format!("sha256:{}", pkg.digest.trim_start_matches("sha256:")),
        }
    }
}

impl<'a> Lowerer<'a> {
    /// A lowerer over every loaded package (the one being lowered, its
    /// dependencies, and for a links package or case, the packs they name).
    #[must_use]
    pub fn new(packages: &[&'a hir::Package]) -> Self {
        Self {
            packages: packages.to_vec(),
            corpus: None,
        }
    }

    /// Locates citations in the vendored L0 `corpus` for the source maps
    /// (`SourceCorpus::embedded_ref()` for the shipped one).
    #[must_use]
    pub fn with_corpus(mut self, corpus: &'a SourceCorpus) -> Self {
        self.corpus = Some(corpus);
        self
    }

    fn build(&self, known_at: &str, options: &Options) -> Build<'_, 'a> {
        let known = options
            .known_at
            .clone()
            .unwrap_or_else(|| known_at.to_owned());
        let as_of = options.as_of.clone().unwrap_or_else(|| known_at.to_owned());
        Build {
            packages: &self.packages,
            selection: Selection::new(&self.packages, as_of, known),
            corpus: self.corpus,
        }
    }

    /// Lowers `pkg` (a rule package to a pack, a links package to a link
    /// file).
    ///
    /// # Errors
    /// Every diagnostic, sorted: L6001 for a construct with no lowering,
    /// L6003/L6004 for engine expressions and numbers, L3007 for an
    /// ambiguous version.
    pub fn package(
        &self,
        pkg: &hir::Package,
        options: &Options,
    ) -> Result<LoweredPackage, Vec<Diagnostic>> {
        let build = self.build(&pkg.manifest.in_force.known_at, options);
        let mut sink = Sink::new(&manifest_file(pkg));
        let lowered = if pkg.manifest.links {
            let (file, source_map) = links::links(&build, pkg, &mut sink);
            LoweredPackage {
                document: Document::Links(file),
                source_map,
                clock: None,
            }
        } else {
            let (pack, source_map) = pack::pack(&build, pkg, &mut sink);
            let decls = pack::emitted(&build, pkg, &mut Sink::new(""));
            let clock = clock::fragment(&build, pkg, &decls, pack.counting.as_deref(), &mut sink);
            LoweredPackage {
                document: Document::Pack(Box::new(pack)),
                source_map,
                clock,
            }
        };
        sink.finish(lowered)
    }

    /// Lowers a checked matter. `file` is its repository-relative path,
    /// where span-less findings are reported.
    ///
    /// # Errors
    /// Every diagnostic, sorted: L4007 for an invalid `analysis`, L6003 for
    /// an engine expression that does not parse.
    pub fn case(&self, case: &hir::Case, file: &str) -> Result<LoweredCase, Vec<Diagnostic>> {
        let build = self.build(&case.as_of, &Options::default());
        let mut sink = Sink::new(file);
        match case::case(&build, case, &mut sink) {
            Some((scenario, source_map)) => sink.finish(LoweredCase {
                scenario,
                source_map,
            }),
            None => Err(sink.finish(()).err().unwrap_or_default()),
        }
    }
}

/// The package's `Lit.toml`, beside its first module file.
fn manifest_file(pkg: &hir::Package) -> String {
    pkg.modules
        .iter()
        .flat_map(|m| m.declarations.iter())
        .find_map(|d| litigate_sema::temporal::world::meta_of(d)?.span.clone())
        .and_then(|s| {
            std::path::Path::new(&s.file.0)
                .parent()
                .map(|p| p.join("Lit.toml").to_string_lossy().replace('\\', "/"))
        })
        .unwrap_or_else(|| "Lit.toml".to_owned())
}

/// [`render`] of a named scenario without the `scenario` fields that
/// equal litgraph's own defaults, which serde fills back in on read: the
/// output is semantically equal (SPEC §8.6) and reads like an authored
/// scenario file.
///
/// # Errors
/// L6005 when the value cannot be serialized.
pub fn render_scenario(named: &NamedScenario) -> Result<String, Box<Diagnostic>> {
    let defaults = serde_json::to_value(litgraph::Scenario::default()).unwrap_or_default();
    let mut value = serde_json::to_value(named).unwrap_or_default();
    if let Some(fields) = value
        .get_mut("scenario")
        .and_then(serde_json::Value::as_object_mut)
    {
        fields.retain(|k, v| defaults.get(k) != Some(v));
    }
    render(&value)
}

/// Byte-deterministic JSON (SPEC §8.6): two-space indentation and a
/// trailing newline. Map keys come out sorted where litgraph's types use
/// ordered maps, and in field order otherwise.
///
/// # Errors
/// L6005 when the value cannot be serialized.
pub fn render<T: Serialize>(value: &T) -> Result<String, Box<Diagnostic>> {
    serde_json::to_string_pretty(value)
        .map(|mut text| {
            text.push('\n');
            text
        })
        .map_err(|e| {
            Box::new(Diagnostic::error(
                DiagnosticCode::OutputError,
                diag::start_of(""),
                format!("could not serialize output: {e}"),
            ))
        })
}
