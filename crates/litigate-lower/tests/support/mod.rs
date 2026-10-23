//! Shared test support: the repository corpus imported by
//! `litigate-import`, checked by sema, as HIR packages.
#![allow(dead_code)]
use litigate_import::{import, ImportedFile, Input};
use litigate_sema::db::inputs::create_workspace;
use litigate_sema::hir::{
    self, Analysis, Case, CaseSource, Money, Observation, PackagePin, Party, Perspective, Stake,
};
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

/// The checked HIR of `lit/examples` (the SPEC §12 worked example),
/// loaded once and cloned for each mutation.
pub fn example() -> hir::Package {
    static EXAMPLE: std::sync::OnceLock<hir::Package> = std::sync::OnceLock::new();
    EXAMPLE
        .get_or_init(|| load_hir(&["lit/examples"]).remove(0))
        .clone()
}

/// Every declaration of `pkg`, mutably.
pub fn decls_mut(pkg: &mut hir::Package) -> impl Iterator<Item = &mut hir::Declaration> {
    pkg.modules
        .iter_mut()
        .flat_map(|m| m.declarations.iter_mut())
}

/// The package's first counting rule.
pub fn counting_mut(pkg: &mut hir::Package) -> &mut hir::Counting {
    decls_mut(pkg)
        .find_map(|d| match d {
            hir::Declaration::Counting(c) => Some(c),
            _ => None,
        })
        .unwrap_or_else(|| panic!("no counting"))
}

/// The package's first calendar.
pub fn calendar_mut(pkg: &mut hir::Package) -> &mut hir::Calendar {
    decls_mut(pkg)
        .find_map(|d| match d {
            hir::Declaration::Calendar(c) => Some(c),
            _ => None,
        })
        .unwrap_or_else(|| panic!("no calendar"))
}

/// Node `id` of the package.
pub fn node_mut<'p>(pkg: &'p mut hir::Package, id: &str) -> &'p mut hir::Node {
    decls_mut(pkg)
        .find_map(|d| match d {
            hir::Declaration::Node(n) if n.meta.id == id => Some(n),
            _ => None,
        })
        .unwrap_or_else(|| panic!("no node {id}"))
}

/// Diagnostics as stable snapshot text: code, location, message, notes.
pub fn render_diagnostics(diagnostics: &[litigate_base::Diagnostic]) -> String {
    use std::fmt::Write;
    let mut out = String::new();
    for d in diagnostics {
        let _ = write!(
            out,
            "{} {}:{}:{}: {}",
            d.code.as_str(),
            d.span.file.0,
            d.span.line,
            d.span.col,
            d.message
        );
        for n in &d.notes {
            let _ = write!(out, "\n  note: {}", n.message);
        }
        out.push('\n');
    }
    out
}

/// The diagnostics of lowering `pkg` alone at the default dates, as
/// snapshot text; panics when it lowers.
pub fn lowering_errors(pkg: &hir::Package) -> String {
    let found = litigate_lower::Lowerer::new(&[pkg])
        .package(pkg, &litigate_lower::Options::default())
        .err()
        .unwrap_or_else(|| panic!("expected lowering diagnostics"));
    render_diagnostics(&found)
}

/// `pkg` lowered alone at the default dates; panics on diagnostics.
pub fn lowered(pkg: &hir::Package) -> litigate_lower::LoweredPackage {
    litigate_lower::Lowerer::new(&[pkg])
        .package(pkg, &litigate_lower::Options::default())
        .unwrap_or_else(|d| panic!("{}", render_diagnostics(&d)))
}

/// The pack of a lowered rule package.
pub fn pack_of(lowered: &litigate_lower::LoweredPackage) -> &litgraph::Pack {
    match &lowered.document {
        litigate_lower::Document::Pack(p) => p,
        litigate_lower::Document::Links(_) => panic!("expected a pack"),
    }
}

/// The clock fragment of a lowered rule package.
pub fn clock_of(lowered: &litigate_lower::LoweredPackage) -> &litgraph::clock::ClockLibrary {
    lowered
        .clock
        .as_ref()
        .map_or_else(|| panic!("no clock fragment"), |(c, _)| c)
}

/// `minor_units` US cents.
pub fn usd(minor_units: i64) -> Money {
    Money {
        minor_units,
        currency: "usd".into(),
    }
}

/// A pin of package `name` (its pack id too), version 0.1.0.
pub fn pin(name: &str) -> PackagePin {
    PackagePin {
        name: name.into(),
        pack_id: name.into(),
        version: "0.1.0".into(),
        content_hash: String::new(),
    }
}

/// `cases/examples/acme.case` as sema will check it: replay (SPEC §12)
/// lands on `mtd-ruling`. Events are not lowered, so none are listed.
pub fn acme() -> Case {
    let q = |id: &str| format!("frcp-rule12-example::{id}");
    Case {
        id: "acme-v-widget".into(),
        summary: "Acme sues Widget; Widget decides whether to answer or move to dismiss.".into(),
        notes: Some("Worked example for SPEC §12. Stakes are illustrative.".into()),
        sources: vec![CaseSource {
            title: "Federal Rules of Civil Procedure".into(),
            url: Some("https://www.law.cornell.edu/rules/frcp".into()),
            as_of: Some("2024-12-01".into()),
        }],
        uses: vec![pin("frcp-rule12-example")],
        as_of: "2025-06-01".into(),
        known_at: None,
        start: Some(q("mtd-ruling")),
        perspective: Some(Perspective::Party("widget".into())),
        parties: vec![
            Party {
                id: "acme".into(),
                actor: "Plaintiff".into(),
                entity_type: "Corporation".into(),
            },
            Party {
                id: "widget".into(),
                actor: "Defendant".into(),
                entity_type: "Corporation".into(),
            },
        ],
        events: Vec::new(),
        facts: BTreeMap::from([(q("time-barred"), q("not-barred"))]),
        stakes: BTreeMap::from([
            (q("judgment-entered"), Stake::Value(usd(-200_000_000))),
            (q("dismissed"), Stake::Value(usd(0))),
        ]),
        observations: vec![Observation {
            node: q("mtd-ruling"),
            outcomes: vec![(q("granted"), 3), (q("denied"), 1)],
        }],
        analysis: Analysis {
            fields: BTreeMap::from([(
                "params".into(),
                serde_json::json!({"rate": 650, "opp_rate": 550}),
            )]),
        },
        removals: Vec::new(),
    }
}
