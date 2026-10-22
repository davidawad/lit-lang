//! Shared test support: the repository corpus imported by
//! `litigate-import`, checked by sema, as HIR packages.
#![allow(dead_code)]
use litigate_import::{import, ImportedFile, Input};
use litigate_sema::db::inputs::create_workspace;
use litigate_sema::hir;
use litigate_sema::pkg::load::{LoadedFile, LoadedPackage};
use litigate_sema::LitDatabase;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// The repository root.
pub fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// Every legacy pack, `links.json`, and named scenario in the repository.
pub fn corpus() -> Vec<Input> {
    let mut paths = Vec::new();
    for dir in ["packs", "scenarios"] {
        let entries = std::fs::read_dir(repo_root().join(dir))
            .map(|r| r.filter_map(Result::ok).collect::<Vec<_>>())
            .unwrap_or_default();
        for entry in entries {
            let name = entry.file_name().to_string_lossy().into_owned();
            if Path::new(&name).extension().is_some_and(|e| e == "json") {
                paths.push(format!("{dir}/{name}"));
            }
        }
    }
    paths.sort();
    paths
        .into_iter()
        .map(|path| {
            let text = std::fs::read_to_string(repo_root().join(&path)).unwrap_or_default();
            Input::new(path, text)
        })
        .collect()
}

/// `files` grouped into sema package inputs by `lit/<dir>/`.
pub fn packages(files: &[ImportedFile]) -> Vec<LoadedPackage> {
    let mut by_dir: BTreeMap<String, LoadedPackage> = BTreeMap::new();
    for file in files {
        let Some((dir, relative)) = file
            .path
            .strip_prefix("lit/")
            .and_then(|rest| rest.split_once('/'))
        else {
            continue;
        };
        let pkg = by_dir
            .entry(dir.to_owned())
            .or_insert_with(|| LoadedPackage {
                root: format!("lit/{dir}"),
                manifest: String::new(),
                lock: None,
                files: Vec::new(),
            });
        if relative == "Lit.toml" {
            pkg.manifest.clone_from(&file.text);
        } else {
            pkg.files.push(LoadedFile {
                relative: relative.to_owned(),
                text: file.text.clone(),
            });
        }
    }
    by_dir.into_values().collect()
}

/// Imports `inputs` and checks the result: `(import origin, HIR)` per
/// package, in package-directory order.
pub fn imported_hir(inputs: &[Input]) -> Vec<(String, hir::Package)> {
    let files = import(inputs)
        .unwrap_or_else(|d| panic!("{}", serde_json::to_string_pretty(&d).unwrap_or_default()))
        .files;
    let loaded = packages(&files);
    let mut db = LitDatabase::default();
    let ws = create_workspace(&mut db, &loaded);
    let checks = litigate_sema::check_workspace(&db, ws);
    loaded
        .iter()
        .zip(checks)
        .map(|(pkg, check)| {
            let origin = files
                .iter()
                .find(|f| f.path == format!("{}/Lit.toml", pkg.root))
                .map(|f| f.from.clone())
                .unwrap_or_default();
            let hir = check.package.clone().unwrap_or_else(|| {
                panic!(
                    "{} has no HIR: {}",
                    pkg.root,
                    serde_json::to_string_pretty(&check.diagnostics).unwrap_or_default()
                )
            });
            (origin, hir)
        })
        .collect()
}

/// Loads checked packages from `roots` (repository-relative directories).
pub fn load_hir(roots: &[&str]) -> Vec<hir::Package> {
    let root = repo_root();
    let dirs: Vec<PathBuf> = roots.iter().map(|r| root.join(r)).collect();
    let (db, ws) = litigate_sema::load(&root, &dirs, None).unwrap_or_else(|e| panic!("{e}"));
    litigate_sema::check_workspace(&db, ws)
        .iter()
        .map(|c| {
            c.package.clone().unwrap_or_else(|| {
                panic!(
                    "no HIR: {}",
                    serde_json::to_string_pretty(&c.diagnostics).unwrap_or_default()
                )
            })
        })
        .collect()
}
