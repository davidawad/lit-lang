//! The one version-selection query: each package's dependencies, resolved
//! with `PubGrub` over every loaded package (SPEC §4.2). Imports, `::`
//! qualification, the lock, and the HIR's `resolved` versions all read it,
//! so no consumer picks a package by load order.
use super::inputs::{PackageInput, Workspace};
use super::queries::{manifest, resolutions};
use crate::pkg::resolve::{resolve, Candidate, Failure};
use crate::pkg::semver::{Requirement, Version};
use std::collections::BTreeMap;

/// One package's resolution.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Resolution {
    /// Package name to the selected package: the package itself and its
    /// transitive dependencies (after a failure, only what
    /// [`crate::pkg::resolve::Resolved::selected`] keeps). Empty for a
    /// package whose manifest is invalid.
    pub selected: BTreeMap<String, Selected>,
    /// Why resolution failed.
    pub failure: Option<Failure>,
}

/// One selected package.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Selected {
    /// The package.
    pub input: PackageInput,
    /// Its exact version.
    pub version: String,
}

impl Resolution {
    /// The selected package named `name`.
    #[must_use]
    pub fn get(&self, name: &str) -> Option<PackageInput> {
        self.selected.get(name).map(|s| s.input)
    }
}

/// [`resolutions`]: every package's resolution, in load order.
pub(crate) fn build(db: &dyn salsa::Database, ws: Workspace) -> Vec<Resolution> {
    let packages = ws.packages(db);
    let candidates: Vec<(usize, Candidate)> = packages
        .iter()
        .enumerate()
        .filter_map(|(i, p)| {
            let m = &manifest(db, *p).as_ref().ok()?.manifest;
            Version::parse(&m.version).map(|version| {
                (
                    i,
                    Candidate {
                        root: p.root(db).clone(),
                        name: m.name.clone(),
                        version,
                        dependencies: m
                            .dependencies
                            .iter()
                            .filter_map(|(n, r)| Requirement::parse(r).map(|r| (n.clone(), r)))
                            .collect(),
                    },
                )
            })
        })
        .collect();
    let universe: Vec<Candidate> = candidates.iter().map(|(_, c)| c.clone()).collect();
    let mut out = vec![Resolution::default(); packages.len()];
    for (root, (i, _)) in candidates.iter().enumerate() {
        let resolved = resolve(&universe, root);
        out[*i] = Resolution {
            selected: resolved
                .selected
                .into_iter()
                .map(|(name, c)| {
                    let selected = Selected {
                        input: packages[candidates[c].0],
                        version: universe[c].version.to_string(),
                    };
                    (name, selected)
                })
                .collect(),
            failure: resolved.failure,
        };
    }
    out
}

/// `pkg`'s resolution in `ws` (the default when `pkg` is not in `ws`).
#[must_use]
pub fn resolution(db: &dyn salsa::Database, ws: Workspace, pkg: PackageInput) -> &Resolution {
    static NONE: Resolution = Resolution {
        selected: BTreeMap::new(),
        failure: None,
    };
    ws.packages(db)
        .iter()
        .position(|p| *p == pkg)
        .map_or(&NONE, |i| &resolutions(db, ws)[i])
}

/// The `core` package `pkg` sees (SPEC §4.2): the one its resolution
/// selects (`pkg` itself for `core`), else the highest loaded version.
#[must_use]
pub fn core(db: &dyn salsa::Database, ws: Workspace, pkg: PackageInput) -> Option<PackageInput> {
    resolution(db, ws, pkg).get("core").or_else(|| {
        ws.packages(db)
            .iter()
            .filter_map(|p| {
                let m = &manifest(db, *p).as_ref().ok()?.manifest;
                (m.name == "core").then(|| (Version::parse(&m.version), *p))
            })
            .max_by(|a, b| a.0.cmp(&b.0))
            .map(|(_, p)| p)
    })
}
