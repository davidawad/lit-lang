//! Package-level checks (manifest, dependencies, one `start`, package-wide
//! namespaces, links-package shape) and the assembled package HIR.
use super::{item_name, Hole, ModuleCheck};
use crate::db::inputs::{PackageInput, Workspace};
use crate::db::queries::{check_module, item_tree, manifest, signature};
use crate::db::span::{sort_diagnostics, to_span, Report, TextRange};
use crate::hir;
use crate::pkg::lock::{digest, LockFile, LockedPackage};
use crate::pkg::semver::{Requirement, Version};
use crate::resolve::sig::flatten;
use litigate_base::{Diagnostic, DiagnosticCode as C, FileId, LineIndex};
use std::collections::BTreeMap;

/// Result of checking a package.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PackageCheck {
    /// HIR, present only when the whole package is error-free.
    pub package: Option<hir::Package>,
    /// Every diagnostic of the manifest and all modules, sorted.
    pub diagnostics: Vec<Diagnostic>,
    /// Every hole in the package.
    pub holes: Vec<Hole>,
}

/// Locates `needle` in the manifest for a diagnostic span.
fn find(text: &str, needle: &str) -> TextRange {
    let at = |n: usize| u32::try_from(n).unwrap_or(0);
    text.find(needle).map_or(TextRange::default(), |s| {
        TextRange::new(at(s), at(s + needle.len()))
    })
}

/// The package satisfying dependency `name` with requirement `req`.
pub(crate) fn resolve_dependency(
    db: &dyn salsa::Database,
    ws: Workspace,
    name: &str,
    req: &str,
) -> Result<PackageInput, String> {
    let candidates: Vec<(PackageInput, String)> = ws
        .packages(db)
        .iter()
        .filter_map(|p| {
            manifest(db, *p)
                .as_ref()
                .ok()
                .filter(|m| m.manifest.name == name)
                .map(|m| (*p, m.manifest.version.clone()))
        })
        .collect();
    let req = Requirement::parse(req).ok_or_else(|| format!("invalid requirement `{req}`"))?;
    candidates
        .iter()
        .find(|(_, v)| Version::parse(v).is_some_and(|v| req.matches(&v)))
        .map(|(p, _)| *p)
        .ok_or_else(|| match candidates.first() {
            Some((_, v)) => format!(
                "dependency `{name}` is version {v}, which does not satisfy the requirement"
            ),
            None => format!("dependency `{name}` was not found (check its `path`)"),
        })
}

/// Checks a package and assembles its HIR.
pub fn check_package(db: &dyn salsa::Database, pkg: PackageInput) -> PackageCheck {
    let manifest_file = FileId(pkg.manifest_path(db));
    let text = pkg.manifest_text(db);
    let index = LineIndex::new(text);
    let mut reports: Vec<Report> = Vec::new();
    let parsed = manifest(db, pkg);
    let m = match parsed {
        Ok(p) => Some(&p.manifest),
        Err(errors) => {
            reports.extend(errors.iter().cloned());
            None
        }
    };
    let ws = pkg.files(db).first().map(|f| f.workspace(db));
    if let (Some(m), Some(ws)) = (m, ws) {
        for (name, req) in &m.dependencies {
            if let Err(message) = resolve_dependency(db, ws, name, req) {
                reports.push(Report::new(
                    C::UndeclaredDependency,
                    find(text, &format!("{name} =")),
                    message,
                ));
            }
        }
    }
    let checks: Vec<&ModuleCheck> = pkg.files(db).iter().map(|f| check_module(db, *f)).collect();
    let mut diagnostics: Vec<Diagnostic> = reports
        .iter()
        .map(|r| r.to_diagnostic(&manifest_file, &index))
        .collect();
    diagnostics.extend(checks.iter().flat_map(|c| c.diagnostics.iter().cloned()));
    diagnostics.extend(namespace_duplicates(db, pkg));
    let starts: Vec<_> = checks.iter().flat_map(|c| c.starts.iter()).collect();
    diagnostics.extend(shape(
        &checks,
        &starts,
        to_span(&manifest_file, &index, TextRange::default()),
    ));
    sort_diagnostics(&mut diagnostics);
    let holes = checks
        .iter()
        .flat_map(|c| c.holes.iter().cloned())
        .collect();
    let error = diagnostics
        .iter()
        .any(|d| d.severity == litigate_base::Severity::Error);
    let package = (!error)
        .then(|| {
            let mut order: Vec<&ModuleCheck> = checks.clone();
            order.sort_by(|a, b| {
                a.module
                    .as_ref()
                    .map(|m| &m.name)
                    .cmp(&b.module.as_ref().map(|m| &m.name))
            });
            Some(hir::Package {
                manifest: m?.clone(),
                digest: package_digest(db, pkg),
                modules: order.iter().filter_map(|c| c.module.clone()).collect(),
                start: starts
                    .first()
                    .and_then(|(s, _)| s.clone())
                    .unwrap_or_default(),
                groups: order
                    .iter()
                    .flat_map(|c| c.groups.iter().cloned())
                    .collect(),
            })
        })
        .flatten();
    PackageCheck {
        package,
        diagnostics,
        holes,
    }
}

/// One `start` (none in a links package) and no nodes in a links package.
fn shape(
    checks: &[&ModuleCheck],
    starts: &[&(Option<String>, litigate_base::Span)],
    manifest_span: litigate_base::Span,
) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let is_links = checks.iter().any(|c| c.first_link.is_some());
    if is_links {
        if let Some(node) = checks.iter().find_map(|c| c.first_node.clone()) {
            diagnostics.push(Diagnostic::error(
                C::InvalidField,
                node,
                "a links package (one with `link` or `instance`) must declare no nodes",
            ));
        }
    } else if starts.is_empty() {
        diagnostics.push(Diagnostic::error(
            C::InvalidStart,
            manifest_span,
            "the package declares no `start` node",
        ));
    }
    for (_, span) in starts.iter().skip(1) {
        diagnostics.push(Diagnostic::error(
            C::InvalidStart,
            span.clone(),
            "a package has exactly one `start`",
        ));
    }
    diagnostics
}

/// Names declared by more than one module of the package (L1001).
fn namespace_duplicates(db: &dyn salsa::Database, pkg: PackageInput) -> Vec<Diagnostic> {
    let mut owners: BTreeMap<(u8, String), String> = BTreeMap::new();
    let mut out = Vec::new();
    for file in pkg.files(db) {
        let sig = signature(db, *file);
        let names = sig
            .values
            .keys()
            .map(|k| (0, k))
            .chain(sig.types.keys().map(|k| (1, k)))
            .chain(sig.groups.iter().map(|k| (2, k)));
        for (ns, name) in names {
            let key = (ns, name.clone());
            match owners.get(&key) {
                Some(first) if *first != sig.module => {
                    let tree = &item_tree(db, *file).tree;
                    let range = flatten(tree)
                        .iter()
                        .filter_map(|(i, _)| item_name(i))
                        .find(|n| n.text == *name)
                        .map_or(TextRange::default(), |n| n.range);
                    let index = LineIndex::new(file.text(db));
                    out.push(
                        Report::new(C::DuplicateName, range, format!("`{name}` is also declared in module `{first}`"))
                            .note("names are unique across a package: node ids lower without module prefixes")
                            .to_diagnostic(&FileId(file.path(db)), &index),
                    );
                }
                Some(_) => {}
                None => {
                    owners.insert(key, sig.module.clone());
                }
            }
        }
    }
    out
}

/// Lowercase-hex SHA-256 package digest (SPEC §4.2).
pub fn package_digest(db: &dyn salsa::Database, pkg: PackageInput) -> String {
    digest(
        pkg.manifest_text(db),
        pkg.files(db)
            .iter()
            .map(|f| (f.relative(db).as_str(), f.text(db).as_str())),
    )
}

/// The `Lit.lock` that `root` and its transitive dependencies resolve to.
#[must_use]
pub fn expected_lock(db: &dyn salsa::Database, ws: Workspace, root: PackageInput) -> LockFile {
    let mut queue = vec![root];
    let mut done: Vec<PackageInput> = Vec::new();
    let mut packages = Vec::new();
    while let Some(p) = queue.pop() {
        if done.contains(&p) {
            continue;
        }
        done.push(p);
        let Ok(parsed) = manifest(db, p) else {
            continue;
        };
        let m = &parsed.manifest;
        let deps: Vec<PackageInput> = m
            .dependencies
            .iter()
            .filter_map(|(n, r)| resolve_dependency(db, ws, n, r).ok())
            .collect();
        let dependencies = deps
            .iter()
            .filter_map(|d| {
                manifest(db, *d)
                    .as_ref()
                    .ok()
                    .map(|dm| format!("{} {}", dm.manifest.name, dm.manifest.version))
            })
            .collect();
        packages.push(LockedPackage {
            name: m.name.clone(),
            version: m.version.clone(),
            digest: format!("sha256:{}", package_digest(db, p)),
            dependencies,
            language: m.language,
        });
        queue.extend(deps);
    }
    LockFile::new(packages)
}
