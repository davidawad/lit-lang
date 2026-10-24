//! One checked snapshot of the workspace: every source text, the check's
//! diagnostics by file, and the symbol index over the same texts.
use crate::host::real_path;
use crate::index::Index;
use litigate_base::{Diagnostic, FileId};
use litigate_cli::matter::CaseCheck;
use litigate_cli::ws::Pkg;
use litigate_cli::Checked;
use litigate_sema::hir;
use litigate_sema::pkg::load::relative_to;
use litigate_syntax::{parse, FileKind};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use tower_lsp::lsp_types::Url;

/// A checked workspace snapshot.
pub struct Analysis {
    /// The root every path is relative to.
    pub root: PathBuf,
    /// The `litigate check` result.
    pub checked: Checked,
    /// Repository-relative path to text: every checked file, plus open
    /// documents outside any checked package.
    pub files: BTreeMap<String, String>,
    /// Diagnostics by repository-relative path (an entry, possibly empty,
    /// for every file in `files`).
    pub diagnostics: BTreeMap<String, Vec<Diagnostic>>,
    /// Definitions and references.
    pub index: Index,
    /// The selected world: `as_of` and `known_at` (`None`: each package's
    /// own dates).
    pub world: Option<(String, Option<String>)>,
}

impl Analysis {
    /// Indexes `checked`; `open` documents the session did not check get
    /// their parser diagnostics.
    #[must_use]
    pub fn new(root: PathBuf, checked: Checked, open: BTreeMap<PathBuf, String>) -> Self {
        let mut files: BTreeMap<String, String> = checked.session.sources().into_iter().collect();
        let mut diagnostics: BTreeMap<String, Vec<Diagnostic>> =
            files.keys().map(|f| (f.clone(), Vec::new())).collect();
        for (path, text) in open {
            let rel = relative_to(&root, &path);
            if files.contains_key(&rel) {
                continue;
            }
            let parsed = FileKind::from_path(&path).map(|kind| parse(&text, kind));
            diagnostics.insert(
                rel.clone(),
                parsed.map_or_else(Vec::new, |p| p.diagnostics(&FileId(rel.clone()))),
            );
            files.insert(rel, text);
        }
        for d in &checked.diagnostics {
            diagnostics
                .entry(d.span.file.0.clone())
                .or_default()
                .push(d.clone());
        }
        let index = Index::build(&files, &checked);
        Self {
            root,
            checked,
            files,
            diagnostics,
            index,
            world: None,
        }
    }

    /// The analysis, checked in `world`.
    #[must_use]
    pub fn in_world(mut self, world: Option<(String, Option<String>)>) -> Self {
        self.world = world;
        self
    }

    /// The repository-relative path of `uri`.
    #[must_use]
    pub fn path_of(&self, uri: &Url) -> Option<String> {
        let path = uri.to_file_path().ok()?;
        Some(relative_to(&self.root, &path))
    }

    /// The URI of repository-relative `path`.
    #[must_use]
    pub fn uri_of(&self, path: &str) -> Option<Url> {
        Url::from_file_path(real_path(&self.root, path)).ok()
    }

    /// The text of `path` (a checked or open file, else the file on disk,
    /// e.g. a `Lit.lock` a diagnostic points at).
    #[must_use]
    pub fn text(&self, path: &str) -> Option<String> {
        self.files
            .get(path)
            .cloned()
            .or_else(|| std::fs::read_to_string(real_path(&self.root, path)).ok())
    }

    /// The checked package holding source file `path`.
    #[must_use]
    pub fn package_of(&self, path: &str) -> Option<&Pkg> {
        self.checked
            .session
            .packages
            .iter()
            .find(|p| p.files.iter().any(|(f, _)| f == path))
    }

    /// The checked matter at `path`.
    #[must_use]
    pub fn case_of(&self, path: &str) -> Option<&CaseCheck> {
        self.checked.cases.iter().find(|c| c.path == path)
    }

    /// HIR of the package named `name`, when it checked.
    #[must_use]
    pub fn hir(&self, name: &str) -> Option<&hir::Package> {
        self.checked
            .session
            .packages
            .iter()
            .find(|p| p.name == name)
            .and_then(|p| p.check.package.as_ref())
    }

    /// The package with pack id or name `key`.
    #[must_use]
    pub fn hir_by_key(&self, key: &str) -> Option<&hir::Package> {
        self.checked
            .session
            .hir()
            .into_iter()
            .find(|p| p.manifest.pack_id == key || p.manifest.name == key)
    }

    /// The file kind of `path`.
    #[must_use]
    pub fn kind(path: &str) -> Option<FileKind> {
        FileKind::from_path(Path::new(path))
    }
}
