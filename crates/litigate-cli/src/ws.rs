//! Finding and loading inputs: package directories, `.case` matters, and
//! the library packages a matter `uses`. Paths are repository-relative to
//! the working directory, so outputs carry no absolute paths (DESIGN §8).
use crate::out::Failure;
use litigate_sema::db::inputs::{PackageInput, Workspace};
use litigate_sema::hir;
use litigate_sema::pkg::load::{relative_to, LoadedPackage};
use litigate_sema::pkg::lock::LockFile;
use litigate_sema::pkg::manifest::parse_manifest;
use litigate_sema::types::build::expected_lock;
use litigate_sema::{LitDatabase, PackageCheck};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// What the user's paths name.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Targets {
    /// Package directories (holding `Lit.toml`).
    pub packages: Vec<PathBuf>,
    /// `.case` files.
    pub cases: Vec<PathBuf>,
    /// Every `.lit` and `.case` file named or found.
    pub files: Vec<PathBuf>,
}

fn skipped(name: &str) -> bool {
    name.starts_with('.') || name == "target"
}

fn walk(dir: &Path, out: &mut Targets) -> std::io::Result<()> {
    if dir.join("Lit.toml").is_file() {
        out.packages.push(dir.to_path_buf());
    }
    let mut entries: Vec<PathBuf> = std::fs::read_dir(dir).and_then(|listing| {
        listing
            .map(|e| e.map(|e| e.path()))
            .collect::<std::io::Result<_>>()
    })?;
    entries.sort();
    for path in entries {
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        if path.is_dir() {
            if !skipped(&name) {
                walk(&path, out)?;
            }
        } else {
            match path.extension().and_then(|e| e.to_str()) {
                Some("case") => {
                    out.cases.push(path.clone());
                    out.files.push(path);
                }
                Some("lit") => out.files.push(path),
                _ => {}
            }
        }
    }
    Ok(())
}

/// The nearest directory at or above `file` holding `Lit.toml`.
fn package_of(file: &Path) -> Option<PathBuf> {
    file.ancestors()
        .skip(1)
        .find(|dir| dir.join("Lit.toml").is_file())
        .map(Path::to_path_buf)
}

/// Classifies `paths` (files or directories, searched recursively).
///
/// # Errors
/// A path that does not exist, or a file that is not `.lit`, `.case`, or
/// `Lit.toml`, or a `.lit` file outside any package.
pub fn discover(paths: &[String]) -> Result<Targets, Failure> {
    let mut out = Targets::default();
    for raw in paths {
        let path = PathBuf::from(raw);
        if path.is_dir() {
            walk(&path, &mut out)?;
            continue;
        }
        if !path.is_file() {
            return Err(Failure::new(format!("{raw}: no such file or directory")));
        }
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default();
        match path.extension().and_then(|e| e.to_str()) {
            Some("case") => {
                out.cases.push(path.clone());
                out.files.push(path);
            }
            Some("lit") => {
                let dir = package_of(&path).ok_or_else(|| {
                    Failure::new(format!(
                        "{raw}: not inside a package (no Lit.toml above it)"
                    ))
                })?;
                out.packages.push(dir);
                out.files.push(path);
            }
            _ if name == "Lit.toml" => {
                // An existing file always has a parent (`""` for a bare name).
                out.packages
                    .push(path.parent().map(Path::to_path_buf).unwrap_or_default());
            }
            _ => {
                return Err(Failure::new(format!(
                    "{raw}: not a .lit, .case, or Lit.toml file"
                )))
            }
        }
    }
    for list in [&mut out.packages, &mut out.cases, &mut out.files] {
        list.sort();
        list.dedup();
    }
    Ok(out)
}

/// The library directories: `--lib` values, else `lit` when it exists.
#[must_use]
pub fn library_dirs(lib: &[String]) -> Vec<PathBuf> {
    if lib.is_empty() {
        let default = PathBuf::from("lit");
        return if default.is_dir() {
            vec![default]
        } else {
            Vec::new()
        };
    }
    lib.iter().map(PathBuf::from).collect()
}

/// [`library_dirs`], plus (without `--lib`) the nearest `lit` directory
/// above each matter, so `cases/x.case` finds its sibling `lit/`.
#[must_use]
pub fn library_for(lib: &[String], cases: &[PathBuf]) -> Vec<PathBuf> {
    let mut dirs = library_dirs(lib);
    if lib.is_empty() {
        dirs.extend(cases.iter().filter_map(|case| {
            let absolute = case.canonicalize().ok()?;
            absolute
                .ancestors()
                .skip(1)
                .map(|a| a.join("lit"))
                .find(|l| l.is_dir())
        }));
    }
    let mut seen = BTreeSet::new();
    dirs.retain(|d| seen.insert(d.canonicalize().unwrap_or_else(|_| d.clone())));
    dirs
}

/// Every package directory under `dirs` with its manifest name.
#[must_use]
pub fn library(dirs: &[PathBuf]) -> Vec<(String, PathBuf)> {
    let mut found = Targets::default();
    for dir in dirs {
        // An unreadable library directory simply contributes no packages.
        let _ = walk(dir, &mut found);
    }
    found
        .packages
        .into_iter()
        .filter_map(|dir| {
            let text = std::fs::read_to_string(dir.join("Lit.toml")).ok()?;
            let name = parse_manifest(&text).ok()?.manifest.name.clone();
            Some((name, dir))
        })
        .collect()
}

/// Library packages that are links packages (SPEC §8.4).
#[must_use]
pub fn links_packages(lib: &[String], cases: &[PathBuf]) -> Vec<PathBuf> {
    library(&library_for(lib, cases))
        .into_iter()
        .filter(|(_, dir)| {
            std::fs::read_to_string(dir.join("Lit.toml"))
                .ok()
                .and_then(|t| parse_manifest(&t).ok())
                .is_some_and(|m| m.manifest.links)
        })
        .map(|(_, dir)| dir)
        .collect()
}

/// One loaded package and its check.
#[derive(Debug, Clone)]
pub struct Pkg {
    /// The salsa input.
    pub input: PackageInput,
    /// Repository-relative directory.
    pub root: String,
    /// Named by the user (rather than pulled in as a dependency or library package).
    pub requested: bool,
    /// Manifest name (empty when the manifest is invalid).
    pub name: String,
    /// Manifest version.
    pub version: String,
    /// `sha256:<hex>` content digest (SPEC §4.2).
    pub digest: String,
    /// Sema's result.
    pub check: PackageCheck,
    /// `(path, text)` of `Lit.toml` and every module.
    pub files: Vec<(String, String)>,
}

/// One `.case` file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaseText {
    /// Repository-relative path.
    pub path: String,
    /// Contents.
    pub text: String,
    /// The `Lit.lock` beside it, which pins what it `uses` (SPEC §5.6).
    pub lock: Option<String>,
}

/// Everything loaded for one command.
pub struct Session {
    /// The database.
    pub db: LitDatabase,
    /// The workspace input.
    pub ws: Workspace,
    /// Packages in load order.
    pub packages: Vec<Pkg>,
    /// Matters.
    pub cases: Vec<CaseText>,
}

/// Package names a `.case` text `uses`.
pub(crate) fn uses_of(text: &str) -> Vec<String> {
    litigate_syntax::parse_case(text)
        .case_file()
        .and_then(|f| f.uses())
        .map(|u| {
            u.items()
                .iter()
                .filter_map(|i| i.package().map(|t| t.text().to_owned()))
                .collect()
        })
        .unwrap_or_default()
}

/// Display path: repository-relative POSIX.
#[must_use]
pub fn shown(path: &Path) -> String {
    let root = std::env::current_dir().unwrap_or_default();
    let full = path.canonicalize().unwrap_or_else(|_| root.join(path));
    let root = root.canonicalize().unwrap_or(root);
    if full.starts_with(&root) {
        relative_to(&root, &full)
    } else {
        path.to_string_lossy().replace('\\', "/")
    }
}

impl Session {
    /// Loads `targets`, the library packages their matters use, and every
    /// path dependency, then checks every package.
    ///
    /// # Errors
    /// I/O errors reading inputs.
    pub fn load(targets: &Targets, lib: &[String]) -> Result<Self, Failure> {
        let root = std::env::current_dir()?;
        let cases = targets
            .cases
            .iter()
            .map(|p| {
                Ok(CaseText {
                    path: shown(p),
                    text: std::fs::read_to_string(p)
                        .map_err(|e| Failure::new(format!("{}: {e}", p.display())))?,
                    lock: std::fs::read_to_string(p.with_file_name("Lit.lock")).ok(),
                })
            })
            .collect::<Result<Vec<_>, Failure>>()?;
        let wanted: BTreeSet<String> = cases.iter().flat_map(|c| uses_of(&c.text)).collect();
        let mut roots: Vec<PathBuf> = targets.packages.iter().map(|p| root.join(p)).collect();
        if !wanted.is_empty() {
            roots.extend(
                library(&library_for(lib, &targets.cases))
                    .into_iter()
                    .filter(|(name, _)| wanted.contains(name))
                    .map(|(_, dir)| root.join(dir)),
            );
        }
        let requested: BTreeSet<String> = targets
            .packages
            .iter()
            .map(|p| relative_to(&root, &root.join(p)))
            .collect();
        let (db, ws) = litigate_sema::load(&root, &roots, None)?;
        let packages = ws
            .packages(&db)
            .iter()
            .map(|input| package(&db, *input, &requested))
            .collect();
        Ok(Self {
            db,
            ws,
            packages,
            cases,
        })
    }

    /// A session over in-memory packages (all requested) and matters.
    #[must_use]
    pub fn from_loaded(loaded: &[LoadedPackage], cases: Vec<CaseText>) -> Self {
        let mut db = LitDatabase::default();
        let ws = litigate_sema::db::inputs::create_workspace(&mut db, loaded);
        let requested: BTreeSet<String> = loaded.iter().map(|p| p.root.clone()).collect();
        let packages = ws
            .packages(&db)
            .iter()
            .map(|input| package(&db, *input, &requested))
            .collect();
        Self {
            db,
            ws,
            packages,
            cases,
        }
    }

    /// The file a diagnostic `FileId` names. Sema keeps paths outside the
    /// working directory root-relative (`tmp/x/a.lit` for `/tmp/x/a.lit`).
    #[must_use]
    pub fn real_path(file: &str) -> PathBuf {
        let here = PathBuf::from(file);
        let rooted = Path::new("/").join(file);
        if here.exists() || !rooted.exists() {
            here
        } else {
            rooted
        }
    }

    /// HIR of every package that checked without errors.
    #[must_use]
    pub fn hir(&self) -> Vec<&hir::Package> {
        self.packages
            .iter()
            .filter_map(|p| p.check.package.as_ref())
            .collect()
    }

    /// Packages the user named.
    pub fn requested(&self) -> impl Iterator<Item = &Pkg> {
        self.packages.iter().filter(|p| p.requested)
    }

    /// The `Lit.lock` a directory of `matters` needs: every loaded package
    /// they `use`, with its resolved dependencies (SPEC §4.2, §5.6).
    #[must_use]
    pub fn matter_lock<'a>(&self, matters: impl IntoIterator<Item = &'a CaseText>) -> LockFile {
        let wanted: BTreeSet<String> = matters.into_iter().flat_map(|c| uses_of(&c.text)).collect();
        let mut lock = LockFile::new(
            self.packages
                .iter()
                .filter(|p| wanted.contains(&p.name))
                .flat_map(|p| expected_lock(&self.db, self.ws, p.input).packages)
                .collect(),
        );
        lock.packages.dedup();
        lock
    }

    /// Every source text, for rendering and fixes.
    #[must_use]
    pub fn sources(&self) -> Vec<(String, String)> {
        self.packages
            .iter()
            .flat_map(|p| p.files.iter().cloned())
            .chain(self.cases.iter().map(|c| (c.path.clone(), c.text.clone())))
            .collect()
    }
}

/// One package input as loaded: its sema check, files, and digest.
#[must_use]
pub fn package(db: &LitDatabase, input: PackageInput, requested: &BTreeSet<String>) -> Pkg {
    let root = input.root(db).clone();
    let check = litigate_sema::db::queries::check_package(db, input).clone();
    let manifest = parse_manifest(input.manifest_text(db)).ok();
    let files: Vec<(String, String)> =
        std::iter::once((input.manifest_path(db), input.manifest_text(db).clone()))
            .chain(
                input
                    .files(db)
                    .iter()
                    .map(|f| (f.path(db), f.text(db).clone())),
            )
            .collect();
    let digest = check.package.as_ref().map_or_else(
        || {
            litigate_sema::pkg::lock::digest(
                input.manifest_text(db),
                input
                    .files(db)
                    .iter()
                    .map(|f| (f.relative(db).as_str(), f.text(db).as_str())),
            )
        },
        |p| p.digest.clone(),
    );
    Pkg {
        input,
        requested: requested.contains(&root),
        name: manifest
            .as_ref()
            .map(|m| m.manifest.name.clone())
            .unwrap_or_default(),
        version: manifest
            .as_ref()
            .map(|m| m.manifest.version.clone())
            .unwrap_or_default(),
        digest: format!("sha256:{}", digest.trim_start_matches("sha256:")),
        root,
        check,
        files,
    }
}
