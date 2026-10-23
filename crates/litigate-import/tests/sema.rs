//! Every imported pack and the links package pass sema's core checks in
//! the `dev` profile (holes are warnings) and lower to HIR: the importer
//! emits SPEC-conforming source, and sema accepts every legacy shape SPEC
//! admits (pack-schema v3 manifest keys, links manifests, decimal duration
//! estimates).
mod support;

use litigate_base::Severity;
use litigate_sema::db::inputs::create_workspace;
use litigate_sema::pkg::load::{LoadedFile, LoadedPackage};
use litigate_sema::LitDatabase;
use std::collections::BTreeMap;
use support::{corpus, import_ok};

/// The imported `lit/<dir>/` packages as sema inputs.
fn packages() -> Vec<LoadedPackage> {
    let mut by_dir: BTreeMap<String, LoadedPackage> = BTreeMap::new();
    for file in import_ok(&corpus()) {
        let Some(rest) = file.path.strip_prefix("lit/") else {
            continue;
        };
        let Some((dir, relative)) = rest.split_once('/') else {
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
            pkg.manifest = file.text;
        } else {
            pkg.files.push(LoadedFile {
                relative: relative.to_owned(),
                text: file.text,
            });
        }
    }
    let mut out: Vec<LoadedPackage> = by_dir.into_values().collect();
    for p in &mut out {
        p.files.sort_by(|a, b| a.relative.cmp(&b.relative));
    }
    out
}

#[test]
fn every_imported_package_passes_core_checks_and_lowers_to_hir() {
    let packages = packages();
    let dirs: Vec<&str> = packages.iter().map(|p| p.root.as_str()).collect();
    assert_eq!(
        dirs,
        [
            "lit/cafc",
            "lit/cofc",
            "lit/frap",
            "lit/frcp",
            "lit/frcrimp",
            "lit/itc",
            "lit/links",
            "lit/ptab",
            "lit/ttab",
            "lit/uspto",
        ]
    );
    let mut db = LitDatabase::default();
    let ws = create_workspace(&mut db, &packages);
    let checks = litigate_sema::check_workspace(&db, ws);
    let errors: Vec<String> = checks
        .iter()
        .flat_map(|c| c.diagnostics.iter())
        .filter(|d| d.severity == Severity::Error)
        .map(|d| {
            format!(
                "{}:{}:{} {} {}",
                d.span.file.0,
                d.span.line,
                d.span.col,
                d.code.as_str(),
                d.message
            )
        })
        .collect();
    assert!(errors.is_empty(), "{errors:#?}");
    for (pkg, check) in packages.iter().zip(&checks) {
        let hir = check
            .package
            .as_ref()
            .unwrap_or_else(|| panic!("{} produced no HIR", pkg.root));
        assert_eq!(hir.manifest.links, pkg.root == "lit/links", "{}", pkg.root);
        if !hir.manifest.links {
            assert!(!hir.start.is_empty(), "{} has no start", pkg.root);
        }
        // The `/// imported` marker reaches HIR on every node (SPEC §7).
        let unmarked: Vec<&str> = hir
            .modules
            .iter()
            .flat_map(|m| &m.declarations)
            .filter_map(|d| match d {
                litigate_sema::hir::Declaration::Node(n) if !n.meta.imported => {
                    Some(n.meta.id.as_str())
                }
                _ => None,
            })
            .collect();
        assert!(unmarked.is_empty(), "{}: {unmarked:?}", pkg.root);
    }
}
