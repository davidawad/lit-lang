//! The workspace under analysis: what is on disk, overlaid with the open
//! documents, checked by the same pipeline as `litigate check`
//! ([`litigate_cli::check_session`]: sema, lowering, verification, case
//! replay).
//!
//! The salsa database lives across edits. When an edit keeps the set of
//! packages, files, manifests, and locks, only the changed source texts are
//! set on the existing inputs, so sema re-derives just what depends on them
//! (DESIGN §3.8), and verification and replay re-run only for the packages
//! and matters the edit reaches ([`litigate_cli::queries`]); otherwise the
//! session is rebuilt. A check a newer edit supersedes is cancelled between
//! packages and matters ([`Host::run`]), and the last complete analysis
//! stays current until the next one completes.
use crate::analysis::Analysis;
use litigate_base::Profile;
use litigate_cli::check_session_with;
use litigate_cli::ws::{self, CaseText, Session};
use litigate_lower::Options;
use litigate_sema::pkg::load::{load_tree, relative_to, LoadedFile, LoadedPackage};
use litigate_syntax::FileKind;
use salsa::Setter;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

/// How long an edit waits for the next before the workspace is checked,
/// by default.
pub const DEBOUNCE_MS: u64 = 200;

/// Server-wide settings (`initializationOptions`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Settings {
    /// Check profile (`"profile": "dev" | "strict"`), default `dev`.
    pub profile: Profile,
    /// Law-in-force date for package checks (`"asOf"`), as `--as-of`: the
    /// world's valid time (`litigate/selectWorld` changes it).
    pub as_of: Option<String>,
    /// Corpus date for package checks (`"knownAt"`), as `--known-at`: the
    /// world's transaction time.
    pub known_at: Option<String>,
    /// Milliseconds an edit waits for the next before the workspace is
    /// checked (`"debounceMs"`), default [`DEBOUNCE_MS`].
    pub debounce_ms: u64,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            profile: Profile::Dev,
            as_of: None,
            known_at: None,
            debounce_ms: DEBOUNCE_MS,
        }
    }
}

impl Settings {
    /// Reads `{"profile": "strict", "asOf": "2025-01-01", "knownAt":
    /// "2025-01-01", "debounceMs": 50}`; anything else keeps the default.
    #[must_use]
    pub fn from_options(options: Option<&serde_json::Value>) -> Self {
        let value = |key: &str| options.and_then(|o| o.get(key));
        let get = |key: &str| value(key).and_then(|v| v.as_str());
        Self {
            profile: match get("profile") {
                Some("strict") => Profile::Strict,
                _ => Profile::Dev,
            },
            as_of: get("asOf").map(str::to_owned),
            known_at: get("knownAt").map(str::to_owned),
            debounce_ms: value("debounceMs")
                .and_then(serde_json::Value::as_u64)
                .unwrap_or(DEBOUNCE_MS),
        }
    }
}

/// One open document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Open {
    /// Current text.
    pub text: String,
    /// Client version.
    pub version: i32,
}

/// What a session was built from; equal keys allow reusing its inputs.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Key {
    root: PathBuf,
    packages: Vec<(String, String, Option<String>, Vec<String>)>,
}

impl Key {
    fn of(root: &Path, loaded: &[LoadedPackage]) -> Self {
        Self {
            root: root.to_path_buf(),
            packages: loaded
                .iter()
                .map(|p| {
                    let files = p.files.iter().map(|f| f.relative.clone()).collect();
                    (p.root.clone(), p.manifest.clone(), p.lock.clone(), files)
                })
                .collect(),
        }
    }
}

/// Open documents, settings, and the live session.
#[derive(Default)]
pub struct Host {
    /// Workspace root from `initialize`.
    pub root: Option<PathBuf>,
    /// Settings.
    pub settings: Settings,
    /// Open documents by absolute path.
    pub open: BTreeMap<PathBuf, Open>,
    analysis: Option<(Key, Analysis)>,
    /// How many times the database was built from scratch (tests observe
    /// that edits reuse it).
    pub rebuilds: usize,
    /// How many checks ran, complete or cancelled (tests observe that
    /// debounced edits share one).
    pub checks: usize,
    /// How many checks were cancelled.
    pub cancelled: usize,
}

/// The nearest directory at or above `file` holding `Lit.toml`.
fn package_of(file: &Path) -> Option<PathBuf> {
    file.ancestors()
        .skip(1)
        .find(|dir| dir.join("Lit.toml").is_file())
        .map(Path::to_path_buf)
}

/// The `lit` directory nearest above `file`.
fn library_of(file: &Path) -> Option<PathBuf> {
    file.ancestors()
        .skip(1)
        .map(|dir| dir.join("lit"))
        .find(|dir| dir.is_dir())
}

/// Package names a `.case` text `uses`.
fn uses_of(text: &str) -> BTreeSet<String> {
    litigate_syntax::parse_case(text)
        .case_file()
        .map(|f| {
            f.use_items()
                .iter()
                .filter_map(|i| i.package().map(|t| t.text().to_owned()))
                .collect()
        })
        .unwrap_or_default()
}

impl Host {
    /// The analysis root: the client's workspace root, else the directory
    /// above the first open document holding `lit/`, `Lit.toml`, or `.git`.
    fn root(&self) -> PathBuf {
        self.root.clone().unwrap_or_else(|| {
            let first = self.open.keys().next().cloned().unwrap_or_default();
            first
                .ancestors()
                .skip(1)
                .find(|d| d.join("lit").is_dir() || d.join(".git").exists())
                .or_else(|| first.parent())
                .map_or_else(PathBuf::new, Path::to_path_buf)
        })
    }

    /// The text of `path`: the open document, else the file on disk.
    fn read(&self, path: &Path) -> Option<String> {
        self.open
            .get(path)
            .map(|o| o.text.clone())
            .or_else(|| std::fs::read_to_string(path).ok())
    }

    /// What to check: `lit/` and `cases/` under the root when present (the
    /// `just lit-check` layout), else the whole client root; plus the package or
    /// matter of every open document, and the library packages matters use.
    fn targets(&self, root: &Path) -> (Vec<PathBuf>, Vec<PathBuf>) {
        let conventional: Vec<PathBuf> = ["lit", "cases"]
            .iter()
            .map(|d| root.join(d))
            .filter(|d| d.is_dir())
            .collect();
        // Without a client root the guessed root is not walked whole.
        let scan = match (conventional.is_empty(), &self.root) {
            (false, _) => conventional,
            (true, Some(_)) => vec![root.to_path_buf()],
            (true, None) => Vec::new(),
        };
        let names: Vec<String> = scan
            .iter()
            .map(|p| p.to_string_lossy().into_owned())
            .collect();
        let found = ws::discover(&names).unwrap_or_default();
        let mut packages: BTreeSet<PathBuf> = found.packages.into_iter().collect();
        let mut cases: BTreeSet<PathBuf> = found.cases.into_iter().collect();
        for path in self.open.keys() {
            match FileKind::from_path(path) {
                Some(FileKind::Case) => {
                    cases.insert(path.clone());
                }
                Some(FileKind::Lit) => packages.extend(package_of(path)),
                // Any other file: the package of the directory it sits in.
                _ => packages.extend(
                    path.parent()
                        .filter(|d| d.join("Lit.toml").is_file())
                        .map(Path::to_path_buf),
                ),
            }
        }
        let wanted: BTreeSet<String> = cases
            .iter()
            .filter_map(|c| self.read(c))
            .flat_map(|t| uses_of(&t))
            .collect();
        let libraries: Vec<PathBuf> = cases.iter().filter_map(|c| library_of(c)).collect();
        packages.extend(
            ws::library(&libraries)
                .into_iter()
                .filter(|(name, _)| wanted.contains(name))
                .map(|(_, dir)| dir),
        );
        (packages.into_iter().collect(), cases.into_iter().collect())
    }

    /// Packages as loaded from disk, with open documents laid over them.
    fn packages(&self, root: &Path, dirs: &[PathBuf]) -> Vec<LoadedPackage> {
        let mut loaded = load_tree(root, dirs, None).unwrap_or_default();
        for pkg in &mut loaded {
            let dir = real_path(root, &pkg.root);
            for (path, open) in &self.open {
                if path == &dir.join("Lit.toml") {
                    pkg.manifest.clone_from(&open.text);
                } else if FileKind::from_path(path) == Some(FileKind::Lit)
                    && package_of(path).as_deref() == Some(dir.as_path())
                {
                    let relative = relative_to(&dir, path);
                    match pkg.files.iter_mut().find(|f| f.relative == relative) {
                        Some(file) => file.text.clone_from(&open.text),
                        None => pkg.files.push(LoadedFile {
                            relative,
                            text: open.text.clone(),
                        }),
                    }
                }
            }
            pkg.files.sort_by(|a, b| a.relative.cmp(&b.relative));
        }
        loaded
    }

    /// Re-checks the workspace and returns the analysis.
    pub fn analyze(&mut self) -> &Analysis {
        self.run(&|| false).1
    }

    /// Re-checks the workspace, stopping between packages and matters once
    /// `cancelled` holds. Returns whether the check completed, and the
    /// current analysis: the new one, or after a cancelled check the last
    /// complete one (the first check's partial one when there is none).
    /// What a cancelled check computed stays memoized in the database.
    pub fn run(&mut self, cancelled: &dyn Fn() -> bool) -> (bool, &Analysis) {
        self.checks += 1;
        let root = self.root();
        let (dirs, case_paths) = self.targets(&root);
        let loaded = self.packages(&root, &dirs);
        let cases: Vec<CaseText> = case_paths
            .iter()
            .filter_map(|p| {
                Some(CaseText {
                    path: relative_to(&root, p),
                    text: self.read(p)?,
                    lock: std::fs::read_to_string(p.with_file_name("Lit.lock")).ok(),
                })
            })
            .collect();
        let key = Key::of(&root, &loaded);
        let mut previous = self.analysis.take();
        // Reusing the last session lends it to this check; the packages and
        // matters it showed are kept to hand back if the check is cancelled.
        let mut shown = None;
        let session = match previous.as_mut() {
            Some((old, analysis)) if *old == key => {
                let mut session = std::mem::replace(
                    &mut analysis.checked.session,
                    Session::from_loaded(&[], Vec::new()),
                );
                shown = Some((
                    std::mem::take(&mut session.packages),
                    std::mem::take(&mut session.cases),
                ));
                reuse(session, &loaded, cases)
            }
            _ => {
                self.rebuilds += 1;
                Session::from_loaded(&loaded, cases)
            }
        };
        let options = Options {
            as_of: self.settings.as_of.clone(),
            known_at: self.settings.known_at.clone(),
        };
        let checked = check_session_with(session, self.settings.profile, &options, cancelled);
        let complete = checked.complete;
        self.cancelled += usize::from(!complete);
        let current = match previous {
            Some((old, mut analysis)) if !complete => {
                if let Some((packages, cases)) = shown {
                    let mut session = checked.session;
                    session.packages = packages;
                    session.cases = cases;
                    analysis.checked.session = session;
                }
                (old, analysis)
            }
            _ => {
                let loose = self
                    .open
                    .iter()
                    .map(|(path, open)| (path.clone(), open.text.clone()))
                    .collect();
                let world = self
                    .settings
                    .as_of
                    .clone()
                    .map(|a| (a, self.settings.known_at.clone()));
                (key, Analysis::new(root, checked, loose).in_world(world))
            }
        };
        (complete, &self.analysis.insert(current).1)
    }

    /// The last analysis, if any.
    #[must_use]
    pub fn current(&self) -> Option<&Analysis> {
        self.analysis.as_ref().map(|(_, a)| a)
    }
}

/// `session` with its source texts set to `loaded`'s (only where they
/// changed, so salsa keeps everything else) and its matters replaced.
fn reuse(mut session: Session, loaded: &[LoadedPackage], cases: Vec<CaseText>) -> Session {
    let inputs = session.ws.packages(&session.db).clone();
    for (input, pkg) in inputs.iter().zip(loaded) {
        let files = input.files(&session.db).clone();
        for (file, new) in files.iter().zip(&pkg.files) {
            if file.text(&session.db) != &new.text {
                file.set_text(&mut session.db).to(new.text.clone());
            }
        }
    }
    let requested: BTreeSet<String> = loaded.iter().map(|p| p.root.clone()).collect();
    session.packages = inputs
        .iter()
        .map(|input| ws::package(&session.db, *input, &requested))
        .collect();
    session.cases = cases;
    session
}

/// The file a repository-relative sema path names (sema keeps paths outside
/// the root as root-relative component lists, `tmp/x` for `/tmp/x`).
#[must_use]
pub fn real_path(root: &Path, relative: &str) -> PathBuf {
    let here = root.join(relative);
    let rooted = Path::new("/").join(relative);
    if here.exists() || !rooted.exists() {
        here
    } else {
        rooted
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn settings_read_the_initialization_options() {
        let strict = Settings::from_options(Some(
            &json!({"profile": "strict", "asOf": "2025-01-01", "knownAt": "2024-06-01"}),
        ));
        assert_eq!(strict.profile, Profile::Strict);
        assert_eq!(strict.as_of.as_deref(), Some("2025-01-01"));
        assert_eq!(strict.known_at.as_deref(), Some("2024-06-01"));
        assert_eq!(
            Settings::from_options(Some(&json!({"profile": 7}))),
            Settings::default()
        );
        assert_eq!(Settings::from_options(None).profile, Profile::Dev);
    }

    #[test]
    fn a_matter_names_the_packages_it_uses() {
        let uses = uses_of("case m\nuses a ^1.0.0, b =2.0.0\nas_of 2025-01-01\n");
        assert_eq!(uses.into_iter().collect::<Vec<_>>(), ["a", "b"]);
        assert!(uses_of("not a case").is_empty());
    }

    #[test]
    fn paths_outside_the_root_stay_absolute() {
        let root = std::env::temp_dir().join("litigate-lsp-no-such-root");
        assert_eq!(real_path(&root, "lit/x.lit"), root.join("lit/x.lit"));
        let outside = std::env::current_dir().unwrap_or_default();
        let rel = relative_to(&root, &outside);
        assert_eq!(real_path(&root, &rel), outside);
        // A file under the root is the root's, whatever lies under `/`.
        let manifest = real_path(&outside, "Cargo.toml");
        assert_eq!(manifest, outside.join("Cargo.toml"));
        assert!(manifest.is_file());
    }

    #[test]
    fn without_a_root_only_open_documents_are_analyzed() {
        let dir = std::env::temp_dir().join(format!("litigate-lsp-loose-{}", std::process::id()));
        let file = dir.join("x.lit");
        let mut host = Host::default();
        host.open.insert(
            file.clone(),
            Open {
                text: "module x\nstart s\n".into(),
                version: 1,
            },
        );
        let analysis = host.analyze();
        let rel = relative_to(&analysis.root, &file);
        assert_eq!(analysis.files.keys().collect::<Vec<_>>(), [&rel]);
        assert_eq!(host.rebuilds, 1);
    }
}
