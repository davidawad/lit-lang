//! `litigate vendor [DIR]`: rewrite the stdlib packages a matter vendors
//! (`DIR/core`, `DIR/frcp`) to the ones this binary was built with, and the
//! L0 texts they bind, so a matter that copied them to survive checkout
//! moves checks again after an install (lg-70c6). Each package written is
//! stamped with `.litigate-vendor`, the binary's commit; `check` and `test`
//! report a vendored copy that differs and fails to check as L1015, naming
//! both commits, instead of the raw schema errors, or whose L0 text pins
//! differ and so fail their cites (lg-bhb0), instead of the raw L1010s.
use crate::out::{Failure, Outcome, Report};
use crate::ws::Pkg;
use litigate_base::{Diagnostic, DiagnosticCode, FileId, Span};
use litigate_sema::pkg::load::is_l0;
use litigate_sema::pkg::lock::digest;
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

include!(concat!(env!("OUT_DIR"), "/stdlib.rs"));

/// The stamp `litigate vendor` writes in each package directory.
pub const STAMP: &str = ".litigate-vendor";

/// The commit the binary (and so its embedded stdlib) was built from.
const COMMIT: &str = env!("LITIGATE_GIT_COMMIT");

/// What the stamp says: the commit, and `dirty` when the tree was.
fn stamp_text() -> String {
    format!("{COMMIT} {}\n", env!("LITIGATE_GIT_TREE"))
}

/// One embedded stdlib package.
struct Embedded {
    /// Its `lit/` directory name (`core`, `frcp`).
    dir: &'static str,
    /// Its manifest name.
    name: String,
    /// Its manifest version.
    version: String,
    /// Its source digest ([`digest`]).
    digest: String,
    /// Its L0 text pins ([`pins`]).
    pins: BTreeMap<String, String>,
    /// `(package-relative path, contents)`.
    files: Vec<(&'static str, &'static str)>,
}

fn embedded() -> &'static [Embedded] {
    static ALL: OnceLock<Vec<Embedded>> = OnceLock::new();
    ALL.get_or_init(|| {
        let dirs: BTreeSet<&str> = STDLIB_FILES
            .iter()
            .filter_map(|(k, _)| k.split_once('/').map(|(d, _)| d))
            .collect();
        dirs.into_iter()
            .filter_map(|dir| {
                let files: Vec<(&str, &str)> = STDLIB_FILES
                    .iter()
                    .filter_map(|(k, t)| {
                        k.strip_prefix(dir)
                            .and_then(|r| r.strip_prefix('/'))
                            .map(|r| (r, *t))
                    })
                    .collect();
                let manifest = files.iter().find(|(r, _)| *r == "Lit.toml")?.1;
                let (name, version) = name_version(manifest)?;
                Some(Embedded {
                    dir,
                    name,
                    version,
                    digest: source_digest(manifest, files.iter().copied()),
                    pins: pins(manifest),
                    files,
                })
            })
            .collect()
    })
}

/// `[package] name` and `version` of a manifest that is valid TOML, even
/// when it is not a valid manifest (a newer or older schema).
fn name_version(manifest: &str) -> Option<(String, String)> {
    let table: toml::Table = manifest.parse().ok()?;
    let package = table.get("package")?.as_table()?;
    let field = |k: &str| {
        package
            .get(k)
            .and_then(toml::Value::as_str)
            .map(str::to_owned)
    };
    Some((field("name")?, field("version").unwrap_or_default()))
}

/// `path -> path_sha256` of each `[sources.*]` binding of a manifest that
/// is valid TOML and pins its text.
fn pins(manifest: &str) -> BTreeMap<String, String> {
    let Ok(table) = manifest.parse::<toml::Table>() else {
        return BTreeMap::new();
    };
    table
        .get("sources")
        .and_then(toml::Value::as_table)
        .into_iter()
        .flat_map(|t| t.values())
        .filter_map(|s| {
            let field = |k: &str| s.get(k).and_then(toml::Value::as_str).map(str::to_owned);
            Some((field("path")?, field("path_sha256")?))
        })
        .collect()
}

/// The source digest over the manifest and `.lit` files of `files`
/// (`Lit.lock` and other files left out, as a relock changes them).
fn source_digest<'a>(manifest: &str, files: impl Iterator<Item = (&'a str, &'a str)>) -> String {
    digest(
        manifest,
        files.filter(|(r, _)| Path::new(r).extension().is_some_and(|e| e == "lit")),
    )
}

/// A loaded package that is a vendored stdlib package unlike the binary's.
struct Stale<'a> {
    pkg: &'a Pkg,
    name: String,
    /// The commit its stamp names, if `litigate vendor` wrote it.
    from: Option<String>,
    /// The L0 texts it pins differently from the binary's copy (or that
    /// only one of them pins), whose cites then fail as L1010.
    texts: Vec<String>,
}

fn stale(pkg: &Pkg) -> Option<Stale<'_>> {
    let (manifest_path, manifest) = pkg.files.first()?;
    if !manifest_path.ends_with("Lit.toml") {
        return None;
    }
    let (name, _) = name_version(manifest)?;
    let ours = embedded().iter().find(|e| e.name == name)?;
    let prefix = format!("{}/", pkg.root);
    let files = pkg
        .files
        .iter()
        .skip(1)
        .map(|(p, t)| (p.strip_prefix(&prefix).unwrap_or(p.as_str()), t.as_str()));
    if source_digest(manifest, files) == ours.digest {
        return None;
    }
    let from = std::fs::read_to_string(Path::new(&pkg.root).join(STAMP))
        .ok()
        .and_then(|s| s.split_whitespace().next().map(str::to_owned));
    let theirs = pins(manifest);
    let texts = theirs
        .iter()
        .filter(|(path, sha)| ours.pins.get(*path) != Some(*sha))
        .map(|(path, _)| path)
        .chain(ours.pins.keys().filter(|path| !theirs.contains_key(*path)))
        .cloned()
        .collect();
    Some(Stale {
        pkg,
        name,
        from,
        texts,
    })
}

/// `root` with `.` and `dir/..` steps folded (a path dependency's root,
/// `lit/frcp/../core`, shown as `lit/core`).
fn tidy(root: &str) -> String {
    let mut parts: Vec<&str> = Vec::new();
    for part in root.split('/') {
        match part {
            "." | "" => {}
            ".." if parts.last().is_some_and(|p| *p != "..") => {
                parts.pop();
            }
            _ => parts.push(part),
        }
    }
    if parts.is_empty() {
        ".".to_owned()
    } else {
        parts.join("/")
    }
}

/// The vendor directory to name in the advice: the package's parent.
fn parent(root: &str) -> String {
    tidy(&format!("{root}/.."))
}

/// Reports each loaded stdlib package that differs from the binary's (a
/// vendored copy from another commit) and fails to check as one L1015
/// naming both commits and the `litigate vendor` that fixes it, in place
/// of its raw L1004 errors and, when its L0 text pins differ from the
/// binary's, the L1010s on its own files those pins explain (all kept as
/// notes). A differing package that checks clean is left alone: an older
/// copy that still loads is not wrong.
pub fn retarget(packages: &[Pkg], diagnostics: &mut Vec<Diagnostic>) {
    let error = |d: &Diagnostic| d.severity == litigate_base::Severity::Error;
    for s in packages.iter().filter_map(stale) {
        let own = &s.pkg.check.diagnostics;
        let in_pkg = |d: &Diagnostic| s.pkg.files.iter().any(|(f, _)| *f == d.span.file.0);
        let explained = |d: &Diagnostic| {
            (d.code == DiagnosticCode::InvalidManifest && own.contains(d))
                || (d.code == DiagnosticCode::UnboundCite && !s.texts.is_empty() && in_pkg(d))
        };
        if !own.iter().any(error) && !diagnostics.iter().any(|d| error(d) && explained(d)) {
            continue;
        }
        let mut raw: Vec<Diagnostic> = Vec::new();
        diagnostics.retain(|d| {
            if !explained(d) {
                return true;
            }
            if !raw.contains(d) {
                raw.push(d.clone());
            }
            false
        });
        let from = s
            .from
            .as_deref()
            .map_or_else(|| "an unknown commit".to_owned(), |c| format!("commit {c}"));
        let span = raw.first().map_or_else(
            || Span {
                file: FileId(s.pkg.files[0].0.clone()),
                line: 1,
                col: 1,
                end_line: 1,
                end_col: 1,
            },
            |d| d.span.clone(),
        );
        let message = format!(
            "vendored `{}` ({}) is from {from}, binary is {COMMIT}; run `litigate vendor {}` \
             then `litigate lock` beside the matter",
            s.name,
            tidy(&s.pkg.root),
            parent(&s.pkg.root),
        );
        let mut d = Diagnostic::error(DiagnosticCode::StaleVendoredStdlib, span, message).note(
            "or install a litigate built from the commit the copy came from; \
             the package's errors follow from the mismatch",
        );
        if !s.texts.is_empty() {
            d = d.note(format!(
                "its text pins (`path_sha256`) differ from the binary's for {}",
                s.texts.join(", ")
            ));
        }
        for r in raw {
            d = d.note(format!("{}: {}", r.code.as_str(), r.message));
        }
        diagnostics.insert(0, d);
    }
}

/// Removes the files the package loader reads (`Lit.toml`, `Lit.lock`,
/// `.lit`) under `dir`, outside hidden directories and nested packages,
/// so no stale module outlives the rewrite.
fn clear(dir: &Path) -> std::io::Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let path = entry?.path();
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        if path.is_dir() {
            if !name.starts_with('.') && !path.join("Lit.toml").is_file() {
                clear(&path)?;
            }
        } else if name == "Lit.toml"
            || name == "Lit.lock"
            || path.extension().is_some_and(|e| e == "lit")
        {
            std::fs::remove_file(&path)?;
        }
    }
    Ok(())
}

fn io(path: &Path) -> impl Fn(std::io::Error) -> Failure + '_ {
    move |e| Failure::new(format!("{}: {e}", path.display()))
}

/// Refuses to rewrite `target` when it holds something other than the
/// package `e`: a non-empty directory with no `Lit.toml`, or another package.
fn writable(e: &Embedded, target: &Path) -> Result<(), Failure> {
    let manifest = target.join("Lit.toml");
    let occupied = target.is_dir()
        && std::fs::read_dir(target)
            .map_err(io(target))?
            .next()
            .is_some();
    if occupied && !manifest.is_file() {
        return Err(Failure::new(format!(
            "{}: exists and is not a package (no Lit.toml); move it before vendoring",
            target.display()
        )));
    }
    let name = std::fs::read_to_string(&manifest)
        .ok()
        .and_then(|t| name_version(&t))
        .map(|(n, _)| n);
    match name.filter(|n| *n != e.name) {
        Some(name) => Err(Failure::new(format!(
            "{}: holds package `{name}`, not `{}`; move it before vendoring",
            target.display(),
            e.name
        ))),
        None => Ok(()),
    }
}

/// Writes the L0 texts `e`'s `[sources.*]` bindings name that are missing
/// or differ, at the working-directory-relative paths the loader reads
/// them from, recording each written in `texts` and each the binary does
/// not embed in `missing`.
fn write_texts(
    e: &Embedded,
    texts: &mut BTreeSet<String>,
    missing: &mut BTreeSet<String>,
) -> Result<(), Failure> {
    let corpus = &litigate_engine::cite::corpus::SourceCorpus::embedded_ref().files;
    let manifest = e
        .files
        .iter()
        .find(|(r, _)| *r == "Lit.toml")
        .map_or("", |f| f.1);
    let paths = litigate_sema::pkg::manifest::parse_manifest(manifest)
        .map(|p| p.manifest.sources)
        .unwrap_or_default()
        .into_iter()
        .filter_map(|s| s.path.filter(|p| is_l0(p)));
    for path in paths {
        match corpus.get(&path) {
            Some(text) => {
                if std::fs::read_to_string(&path).ok().as_deref() != Some(text.as_str()) {
                    crate::cmd::build::write_atomic(Path::new(&path), text)?;
                    texts.insert(path);
                }
            }
            None => {
                missing.insert(path);
            }
        }
    }
    Ok(())
}

/// `litigate vendor [DIR]` (default `lit`).
pub fn vendor(dir: Option<&str>) -> Outcome {
    let dir = PathBuf::from(dir.unwrap_or("lit"));
    if embedded().is_empty() {
        return Err(Failure::new(
            "this litigate was built without lit/, so it embeds no stdlib to vendor",
        ));
    }
    for e in embedded() {
        writable(e, &dir.join(e.dir))?;
    }
    let mut packages = Vec::new();
    let mut texts: BTreeSet<String> = BTreeSet::new();
    let mut missing: BTreeSet<String> = BTreeSet::new();
    for e in embedded() {
        let target = dir.join(e.dir);
        let from = std::fs::read_to_string(target.join(STAMP))
            .ok()
            .and_then(|s| s.split_whitespace().next().map(str::to_owned));
        if target.is_dir() {
            clear(&target).map_err(io(&target))?;
        }
        for (rel, text) in &e.files {
            crate::cmd::build::write_atomic(&target.join(rel), text)?;
        }
        crate::cmd::build::write_atomic(&target.join(STAMP), &stamp_text())?;
        write_texts(e, &mut texts, &mut missing)?;
        packages.push(json!({
            "name": e.name,
            "version": e.version,
            "root": crate::ws::shown(&target),
            "from": from,
            "files": e.files.len(),
        }));
    }
    let mut report = Report::of(json!({
        "dir": crate::ws::shown(&dir),
        "commit": COMMIT,
        "packages": packages,
        "texts": texts,
    }));
    if !missing.is_empty() {
        report.result["missing_texts"] = Value::from(missing.into_iter().collect::<Vec<_>>());
    }
    Ok(report)
}
