// SPDX-License-Identifier: GPL-3.0-or-later
//! The vendored L0 corpus: plain-text extracts of primary law under
//! `sources/*.txt` (see `sources/PROVENANCE.md`), embedded into the binary
//! the same way `packs/*.json` is (`api::catalog`), or loaded from a
//! directory for `LITGRAPH_SOURCES` overrides / tests.

use std::borrow::Cow;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use crate::error::{Error, Result};

include!(concat!(env!("OUT_DIR"), "/embedded_sources.rs"));

/// One `## <ref>` block from a vendored source file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Section {
    /// The canonical heading, e.g. `"Rule 12"`, `"28 U.S.C. § 1498"`.
    pub heading: String,
    /// Everything between this `## ` line and the next (or EOF).
    pub body: String,
}

/// A loaded, parsed set of vendored source files, indexed by canonical
/// heading so [`crate::cite::normalize::CiteRef::heading`] resolves in one
/// lookup regardless of which file it came from.
#[derive(Debug, Clone, Default)]
pub struct SourceCorpus {
    /// `repo-relative path -> raw file text`, as loaded (matches
    /// `Pack.sources[].path`).
    pub files: HashMap<String, String>,
    /// heading -> `(file path, section)`, built once at load time. A
    /// heading present in more than one file keeps only the first loaded
    /// (files are loaded in a stable, sorted order, so this is
    /// deterministic); [`SourceCorpus::lookup_all`] returns every match.
    index: HashMap<String, Vec<(String, Section)>>,
}

impl SourceCorpus {
    /// Build a corpus from `(path, text)` pairs, parsing each into
    /// sections.
    #[must_use]
    pub fn from_files(files: impl IntoIterator<Item = (String, String)>) -> SourceCorpus {
        let mut corpus = SourceCorpus::default();
        let mut files: Vec<(String, String)> = files.into_iter().collect();
        files.sort_by(|a, b| a.0.cmp(&b.0));
        for (path, text) in files {
            for section in parse_sections(&text) {
                corpus
                    .index
                    .entry(section.heading.clone())
                    .or_default()
                    .push((path.clone(), section));
            }
            corpus.files.insert(path, text);
        }
        corpus
    }

    /// The corpus baked into this build (`sources/*.txt` at compile time;
    /// empty without the `embed-sources` feature).
    #[must_use]
    pub fn embedded() -> SourceCorpus {
        SourceCorpus::embedded_ref().clone()
    }

    /// The embedded corpus, parsed once per process and shared.
    #[must_use]
    pub fn embedded_ref() -> &'static SourceCorpus {
        static CORPUS: OnceLock<SourceCorpus> = OnceLock::new();
        CORPUS.get_or_init(|| {
            SourceCorpus::from_files(
                SOURCES_EMBEDDED
                    .iter()
                    .map(|(p, t)| ((*p).to_string(), (*t).to_string())),
            )
        })
    }

    /// True when no source files are loaded (for example a build without
    /// `embed-sources` and no `LITGRAPH_SOURCES`).
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.files.is_empty()
    }

    /// Every `*.txt` in `dir`, keyed by `sources/<relative path>` (matching how
    /// `Pack.sources[].path` names embedded files).
    ///
    /// # Errors
    /// Unreadable directory.
    pub fn load(dir: &Path) -> Result<SourceCorpus> {
        let mut files = vec![];
        load_dir(dir, dir, &mut files)?;
        Ok(SourceCorpus::from_files(files))
    }

    /// `$LITGRAPH_SOURCES` if set, else the embedded corpus.
    ///
    /// # Errors
    /// As [`SourceCorpus::load`].
    pub fn default_source() -> Result<Cow<'static, SourceCorpus>> {
        SourceCorpus::from_override(std::env::var_os("LITGRAPH_SOURCES"))
    }

    /// The directory `dir` if given, else the embedded corpus: the body of
    /// [`SourceCorpus::default_source`] with the env var already read, so
    /// tests can exercise it without mutating the process environment
    /// (which races with same-process readers under `cargo test`).
    fn from_override(dir: Option<std::ffi::OsString>) -> Result<Cow<'static, SourceCorpus>> {
        match dir {
            Some(d) => SourceCorpus::load(&PathBuf::from(d)).map(Cow::Owned),
            None => Ok(Cow::Borrowed(SourceCorpus::embedded_ref())),
        }
    }

    /// Every `(file path, section)` whose heading exactly equals `heading`
    /// (there can be more than one if two vendored files both define, say,
    /// `"RCFC 56"` — that shouldn't happen with a clean corpus, but a
    /// consumer should not silently pick one over the other without
    /// knowing).
    #[must_use]
    pub fn lookup_all(&self, heading: &str) -> &[(String, Section)] {
        self.index.get(heading).map_or(&[], Vec::as_slice)
    }

    /// True if `path` (as it would appear in `Pack.sources[].path`) is
    /// present in this corpus.
    #[must_use]
    pub fn has_file(&self, path: &str) -> bool {
        self.files.contains_key(path)
    }
}

/// Split `text` into `## <heading>` blocks. A heading line's own text
/// continues on the same output as the section body's first paragraph if
/// the next line is not itself a heading — vendored files put an optional
/// human-readable heading line right after `## <ref>`, which is included
/// in `body` (a fuzzy match on the body should still find it).
fn parse_sections(text: &str) -> Vec<Section> {
    let mut sections = vec![];
    let mut current: Option<(String, String)> = None;
    for line in text.lines() {
        let rule_heading = line.trim().strip_prefix("RULE ").and_then(|line| {
            line.split_once(':')
                .map(|(number, _)| format!("Local Rule {}", number.trim()))
        });
        if let Some(heading) = line
            .strip_prefix("## ")
            .map(str::to_string)
            .or(rule_heading)
        {
            if let Some((h, body)) = current.take() {
                sections.push(Section {
                    heading: h,
                    body: body.trim().to_string(),
                });
            }
            current = Some((heading.trim().to_string(), String::new()));
        } else if let Some((_, body)) = current.as_mut() {
            body.push_str(line);
            body.push('\n');
        }
    }
    if let Some((h, body)) = current {
        sections.push(Section {
            heading: h,
            body: body.trim().to_string(),
        });
    }
    sections
}

fn load_dir(root: &Path, dir: &Path, files: &mut Vec<(String, String)>) -> Result<()> {
    let read = std::fs::read_dir(dir).map_err(|e| Error::Io(format!("{}: {e}", dir.display())))?;
    for entry in read.filter_map(std::result::Result::ok) {
        let path = entry.path();
        if path.is_dir() {
            load_dir(root, &path, files)?;
        } else if path.extension().is_some_and(|x| x == "txt") {
            let text = std::fs::read_to_string(&path)
                .map_err(|e| Error::Io(format!("{}: {e}", path.display())))?;
            let relative = path.strip_prefix(root).unwrap_or(&path).to_string_lossy();
            files.push((
                format!(
                    "sources/{}",
                    relative.replace(std::path::MAIN_SEPARATOR, "/")
                ),
                text,
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURE: &str = "\
## Rule 12
Rule 12 — Defenses and Objections
(a) Time to serve.
(b) How to present defenses. Every defense... including (6) failure to
state a claim upon which relief can be granted.

## Rule 13
Rule 13 — Counterclaim and Crossclaim
(a) Compulsory counterclaim.
";

    #[test]
    fn parses_sections_by_heading() {
        let sections = parse_sections(FIXTURE);
        assert_eq!(sections.len(), 2);
        assert_eq!(sections[0].heading, "Rule 12");
        assert!(sections[0].body.contains("failure to"));
        assert_eq!(sections[1].heading, "Rule 13");
        assert!(sections[1].body.contains("Compulsory"));
    }

    #[test]
    fn from_files_indexes_by_heading() {
        let corpus = SourceCorpus::from_files([(
            "sources/frcp-fixture.txt".to_string(),
            FIXTURE.to_string(),
        )]);
        let hits = corpus.lookup_all("Rule 12");
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].0, "sources/frcp-fixture.txt");
        assert!(corpus.has_file("sources/frcp-fixture.txt"));
        assert!(corpus.lookup_all("Rule 99").is_empty());
    }

    #[test]
    fn duplicate_headings_across_files_both_appear() {
        let corpus = SourceCorpus::from_files([
            (
                "sources/a.txt".to_string(),
                "## Rule 1\nfrom a\n".to_string(),
            ),
            (
                "sources/b.txt".to_string(),
                "## Rule 1\nfrom b\n".to_string(),
            ),
        ]);
        let hits = corpus.lookup_all("Rule 1");
        assert_eq!(hits.len(), 2);
    }

    #[test]
    fn empty_and_embedded_corpora_report_emptiness() {
        assert!(SourceCorpus::default().is_empty());
        assert!(!SourceCorpus::embedded_ref().is_empty());
        assert!(std::ptr::eq(
            SourceCorpus::embedded_ref(),
            SourceCorpus::embedded_ref()
        ));
    }

    #[test]
    fn embedded_corpus_loads_and_has_known_sections() {
        let corpus = SourceCorpus::embedded();
        assert!(corpus.has_file("sources/frcp.txt"));
        assert!(corpus.has_file("sources/ca8/ca8-local-rules.txt"));
        assert!(!corpus.lookup_all("Rule 12").is_empty());
        assert!(!corpus.lookup_all("FRAP 31").is_empty());
        assert!(!corpus.lookup_all("Local Rule 34A").is_empty());
        assert!(!corpus.lookup_all("37 C.F.R. § 42.108").is_empty());
        assert!(!corpus.lookup_all("19 U.S.C. § 1337").is_empty());
    }

    #[test]
    fn default_source_honors_litgraph_sources_override() {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../sources");
        assert!(dir.is_dir(), "expected a sources/ dir at {}", dir.display());
        let corpus = SourceCorpus::from_override(Some(dir.into_os_string())).unwrap();
        assert!(matches!(corpus, Cow::Owned(_)));
        assert!(corpus.has_file("sources/frcp.txt"));
        let embedded = SourceCorpus::from_override(None).unwrap();
        assert!(matches!(embedded, Cow::Borrowed(_)));
    }

    #[test]
    fn load_reports_missing_directory() {
        assert!(SourceCorpus::load(Path::new("/does/not/exist/at/all")).is_err());
    }
}
