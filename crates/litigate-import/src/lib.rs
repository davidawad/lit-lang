//! `litigate import`: legacy litgraph JSON (packs, `links.json`, named
//! scenarios) to idiomatic `.lit`/`.case` source (SPEC §8 read in reverse,
//! §9.3 `import`).
//!
//! Every output file is canonical `litigate fmt` output and parses with no
//! diagnostics. The import is lossless by construction: a value with no
//! exact spelling is an L6002 error naming its JSON pointer rather than an
//! approximation, so lowering the result gives back JSON that is
//! semantically equal to the input (SPEC §8.6).
//!
//! What the porter fills in afterwards is explicit:
//! - `in_force` and `authority` in `Lit.toml` are placeholders (legacy JSON
//!   records neither);
//! - a citation that doesn't parse under the pack's citation syntax, or has
//!   no `cite` spelling, is a cite hole `cite ?("original text")`;
//! - every node and link carries the `/// imported` marker (SPEC §7);
//! - a `.case` asserts its `start` rather than replaying `events`; a v2
//!   scenario's docket is kept as a commented `events` block.
//!
//! ```
//! use litigate_import::{import, Input};
//!
//! let pack = r#"{"schemaVersion": 2, "id": "demo", "title": "Demo", "forum": "demo",
//!   "startNodeId": "a", "roles": {"applicant": "self", "either": "nature"},
//!   "nodes": [{"id": "a", "label": "Start"}, {"id": "b", "kind": "terminal", "label": "End"}],
//!   "edges": [{"id": "go", "from": "a", "to": "b", "label": "Go", "actor": "applicant"}]}"#;
//! let imported = import(&[Input::new("packs/demo.json", pack)]).map_err(|d| format!("{d:?}"))?;
//! let paths: Vec<&str> = imported.files.iter().map(|f| f.path.as_str()).collect();
//! assert_eq!(paths, ["lit/demo/01-ungrouped.lit", "lit/demo/Lit.toml", "lit/demo/demo.lit"]);
//! assert!(imported.files[0].text.contains("go \"Go\" -> b"));
//! # Ok::<(), String>(())
//! ```
mod case;
mod filing;
mod issues;
mod json;
mod links;
mod manifest;
mod modules;
mod node;
mod pack;
pub mod text;
mod transform;

use issues::Issues;
use litigate_base::{Diagnostic, DiagnosticCode, FileId, Label, LineIndex};
use litigate_engine::model::{LinkFile, Pack};
use litigate_engine::scenario::NamedScenario;
use litigate_syntax::FileKind;
use serde::de::DeserializeOwned;
use serde_json::{Map, Value};
use std::collections::BTreeMap;

/// One legacy JSON document to import.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Input {
    /// Repository-relative path, recorded in output comments and
    /// diagnostics.
    pub path: String,
    /// The JSON text.
    pub text: String,
}

impl Input {
    /// An input named `path` with contents `text`.
    pub fn new(path: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            text: text.into(),
        }
    }
}

/// One generated source file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportedFile {
    /// Output path relative to the output root (`lit/<pkg>/...` or
    /// `cases/<id>.case`).
    pub path: String,
    /// The input it was generated from.
    pub from: String,
    /// File contents: canonical `litigate fmt` output for `.lit`/`.case`.
    pub text: String,
}

/// The result of an import.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Imported {
    /// Generated files, sorted by path.
    pub files: Vec<ImportedFile>,
}

/// What kind of file an emitted text is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    Lit,
    Case,
    Toml,
}

/// A generated file before canonical formatting.
#[derive(Debug, Clone)]
pub(crate) struct Emitted {
    path: String,
    kind: Kind,
    text: String,
}

impl Emitted {
    fn new(path: String, kind: Kind, text: String) -> Self {
        Self { path, kind, text }
    }
    pub(crate) fn lit(path: String, text: String) -> Self {
        Self::new(path, Kind::Lit, text)
    }
    pub(crate) fn case(path: String, text: String) -> Self {
        Self::new(path, Kind::Case, text)
    }
    pub(crate) fn toml(path: String, text: String) -> Self {
        Self::new(path, Kind::Toml, text)
    }
    fn under(mut self, root: &str) -> Self {
        self.path = format!("{root}/{}", self.path);
        self
    }
}

/// Imports packs, `links.json` files and named scenarios together, so links
/// and cases see the packs they reference (citation syntax, package
/// directories, `known_at`). The document kind is detected from its keys.
///
/// # Errors
/// Every diagnostic, sorted, when any input is invalid or holds a value
/// with no lossless `.lit`/`.case` spelling (L6002).
pub fn import(inputs: &[Input]) -> Result<Imported, Vec<Diagnostic>> {
    let mut diagnostics = Vec::new();
    let docs = read(inputs, &mut diagnostics);
    let emitted = docs.emit(&mut diagnostics);
    // Output of an input that already failed is discarded unformatted, so
    // its errors aren't repeated as parse failures.
    let failed: Vec<String> = diagnostics.iter().map(|d| d.span.file.0.clone()).collect();
    let mut files = Vec::new();
    for (from, file) in emitted {
        if failed.contains(&from) {
            continue;
        }
        match finish(&from, file) {
            Ok(file) => files.push(file),
            Err(diagnostic) => diagnostics.push(diagnostic),
        }
    }
    files.sort_by(|a, b| a.path.cmp(&b.path));
    for pair in files.windows(2) {
        if pair[0].path == pair[1].path {
            let mut issues = Issues::new(&pair[1].from);
            let message = format!("two inputs both import to `{}`", pair[1].path);
            issues.invalid(1, 1, message);
            diagnostics.extend(issues.into_vec());
        }
    }
    if diagnostics.is_empty() {
        return Ok(Imported { files });
    }
    diagnostics.sort_by(|a, b| {
        let key = |d: &Diagnostic| {
            (
                d.span.file.0.clone(),
                d.span.line,
                d.span.col,
                d.code.as_str(),
            )
        };
        key(a).cmp(&key(b))
    });
    Err(diagnostics)
}

/// The valid inputs, by kind.
#[derive(Default)]
struct Docs<'a> {
    packs: Vec<(&'a Input, Pack)>,
    links: Vec<(&'a Input, LinkFile)>,
    scenarios: Vec<(&'a Input, NamedScenario, Map<String, Value>)>,
}

/// Detects and strictly parses every input.
fn read<'a>(inputs: &'a [Input], diagnostics: &mut Vec<Diagnostic>) -> Docs<'a> {
    let mut docs = Docs::default();
    for input in inputs {
        let mut issues = Issues::new(&input.path);
        let value = match serde_json::from_str::<Value>(&input.text) {
            Ok(value) => value,
            Err(error) => {
                issues.invalid(
                    error.line(),
                    error.column(),
                    format!("invalid JSON: {error}"),
                );
                diagnostics.extend(issues.into_vec());
                continue;
            }
        };
        match detect(&value) {
            Some(Doc::Pack) => {
                let validate = |text: &str| Pack::from_json(text).map(|_| ());
                if let Some(pack) = parse::<Pack>(input, &mut issues, validate) {
                    docs.packs.push((input, pack));
                }
            }
            Some(Doc::Links) => {
                if let Some(file) = parse::<LinkFile>(input, &mut issues, |_| Ok(())) {
                    docs.links.push((input, file));
                }
            }
            Some(Doc::Scenario) => {
                let validate = |text: &str| NamedScenario::from_json(text).map(|_| ());
                if let Some(named) = parse::<NamedScenario>(input, &mut issues, validate) {
                    let raw = value.get("scenario").and_then(Value::as_object).cloned();
                    docs.scenarios.push((input, named, raw.unwrap_or_default()));
                }
            }
            None => issues.invalid(1, 1, "not a pack, links file, or named scenario"),
        }
        diagnostics.extend(issues.into_vec());
    }
    docs
}

impl Docs<'_> {
    /// Every generated file, paired with the input path it came from.
    fn emit(&self, diagnostics: &mut Vec<Diagnostic>) -> Vec<(String, Emitted)> {
        let dirs = package_dirs(&self.packs);
        let dir_of = |id: &str| dirs.get(id).map_or(id.to_owned(), Clone::clone);
        let mut emitted = Vec::new();
        let mut run =
            |input: &Input,
             under: Option<&str>,
             generate: &mut dyn FnMut(&str, &mut Issues) -> Vec<Emitted>| {
                let mut issues = Issues::new(&input.path);
                let files = generate(&shown(&input.path), &mut issues);
                emitted.extend(files.into_iter().map(|f| {
                    let f = match under {
                        Some(root) => f.under(root),
                        None => f,
                    };
                    (input.path.clone(), f)
                }));
                diagnostics.extend(issues.into_vec());
            };
        for (input, pack) in &self.packs {
            let dir = dir_of(&pack.id);
            run(input, Some("lit"), &mut |from, issues| {
                pack::import_pack(pack, from, &dir, issues)
            });
        }
        let known: links::Known<'_> = self
            .packs
            .iter()
            .map(|(_, pack)| {
                let dir = dirs
                    .get(pack.id.as_str())
                    .map_or(pack.id.as_str(), String::as_str);
                (
                    pack.id.as_str(),
                    (dir, &pack.cite_syntax, manifest::known_at(pack)),
                )
            })
            .collect();
        for (input, file) in &self.links {
            run(input, Some("lit"), &mut |from, issues| {
                links::import_links(file, from, &known, issues)
            });
        }
        let by_id: BTreeMap<&str, &Pack> =
            self.packs.iter().map(|(_, p)| (p.id.as_str(), p)).collect();
        for (input, named, raw) in &self.scenarios {
            run(input, None, &mut |from, issues| {
                case::import_case(named, raw, from, &by_id, issues)
            });
        }
        emitted
    }
}

/// Document kinds `import` accepts.
enum Doc {
    Pack,
    Links,
    Scenario,
}

/// The kind of a parsed JSON document, from its keys.
fn detect(value: &Value) -> Option<Doc> {
    let object = value.as_object()?;
    if object.contains_key("nodes") {
        Some(Doc::Pack)
    } else if object.contains_key("links") || object.contains_key("instances") {
        Some(Doc::Links)
    } else if object.contains_key("scenario") || object.contains_key("summary") {
        Some(Doc::Scenario)
    } else {
        None
    }
}

/// Deserializes `input` strictly, then runs litgraph's own validation.
fn parse<T: DeserializeOwned>(
    input: &Input,
    issues: &mut Issues,
    validate: impl Fn(&str) -> litigate_engine::Result<()>,
) -> Option<T> {
    match serde_json::from_str::<T>(&input.text) {
        Ok(value) => match validate(&input.text) {
            Ok(()) => Some(value),
            Err(error) => {
                issues.invalid(1, 1, error.to_string());
                None
            }
        },
        Err(error) => {
            issues.invalid(error.line(), error.column(), error.to_string());
            None
        }
    }
}

/// Package directory per pack id: the pack's `forum` key when no other
/// imported pack shares it and it is an identifier, else the pack id.
fn package_dirs(packs: &[(&Input, Pack)]) -> BTreeMap<String, String> {
    let forum_count = |forum: &str| {
        packs
            .iter()
            .filter(|(_, p)| p.forum.as_deref() == Some(forum))
            .count()
    };
    packs
        .iter()
        .map(|(_, pack)| {
            let dir = pack
                .forum
                .as_deref()
                .filter(|f| text::is_ident(f) && forum_count(f) == 1)
                .unwrap_or(&pack.id);
            (pack.id.clone(), dir.to_owned())
        })
        .collect()
}

/// The input path as shown in output: repository-relative paths verbatim,
/// an absolute path reduced to its file name (outputs carry no absolute
/// paths, DESIGN §8).
fn shown(path: &str) -> String {
    let path = path.replace('\\', "/");
    if path.starts_with('/') || path.get(1..3) == Some(":/") {
        path.rsplit('/').next().unwrap_or_default().to_owned()
    } else {
        path
    }
}

/// Canonical formatting for source files; a file the formatter rejects is
/// an importer bug, reported with the parser's diagnostics as notes.
fn finish(from: &str, file: Emitted) -> Result<ImportedFile, Diagnostic> {
    let kind = match file.kind {
        Kind::Toml => {
            return Ok(ImportedFile {
                path: file.path,
                from: from.to_owned(),
                text: file.text,
            })
        }
        Kind::Lit => FileKind::Lit,
        Kind::Case => FileKind::Case,
    };
    let text = format!("{}\n", file.text);
    if let Ok(text) = litigate_syntax::format(&text, kind) {
        return Ok(ImportedFile {
            path: file.path,
            from: from.to_owned(),
            text,
        });
    }
    let parsed = litigate_syntax::parse(&text, kind);
    let (id, index) = (FileId(file.path.clone()), LineIndex::new(&text));
    let issues = Issues::new(from);
    let mut diagnostic = Diagnostic::error(
        DiagnosticCode::SemanticDiff,
        issues.span(1, 1),
        format!("generated `{}` does not parse", file.path),
    );
    diagnostic.labels = parsed
        .errors()
        .iter()
        .map(|e| {
            let d = e.to_diagnostic(&id, &index);
            Label {
                span: d.span,
                message: d.message,
            }
        })
        .collect();
    Err(diagnostic)
}
