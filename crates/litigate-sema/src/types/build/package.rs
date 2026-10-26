//! Package-level checks (manifest, dependencies, one `start`, package-wide
//! namespaces, links-package shape) and the assembled package HIR.
use super::{item_name, Hole, ModuleCheck};
use crate::db::inputs::{PackageInput, Workspace};
use crate::db::queries::{check_module, item_tree, manifest, signature, span_map};
use crate::db::resolution::{resolution, Selected};
use crate::db::span::{sort_diagnostics, to_span, Report, TextRange};
use crate::hir;
use crate::pkg::lock::{digest, semantic_digest, LockFile, LockedPackage, UNCHECKED};
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

/// L1006 for each direct dependency no loaded package is named; L1011
/// when no choice of versions satisfies every requirement, or when a
/// selected dependency was itself checked against other versions than the
/// ones this package's resolution selects.
fn dependency_reports(
    db: &dyn salsa::Database,
    ws: Workspace,
    pkg: PackageInput,
    text: &str,
) -> Vec<Report> {
    let resolved = resolution(db, ws, pkg);
    let at = find(text, "[dependencies]");
    if let Some(failure) = &resolved.failure {
        let missing = failure.missing.iter().map(|name| {
            Report::new(
                C::UndeclaredDependency,
                find(text, &format!("{name} =")),
                format!("dependency `{name}` was not found (check its `path`)"),
            )
        });
        let conflict = failure.conflict.iter().map(|why| {
            Report::new(
                C::UnresolvedDependencies,
                at,
                "the dependencies do not resolve",
            )
            .note(why.clone())
        });
        return missing.chain(conflict).collect();
    }
    // A selected dependency whose own resolution selects this package
    // closes a cycle: the Merkle digest of neither covers the other.
    let cycle = resolved.selected.iter().find(|(_, d)| {
        d.input != pkg
            && resolution(db, ws, d.input)
                .selected
                .values()
                .any(|s| s.input == pkg)
    });
    if let Some((dep, _)) = cycle {
        return vec![Report::new(
            C::UnresolvedDependencies,
            at,
            format!("the package depends on itself through `{dep}`"),
        )
        .note("package dependencies form no cycle: move what both share into a third package")];
    }
    resolved
        .selected
        .iter()
        .filter(|(_, d)| d.input != pkg)
        .flat_map(|(dep, d)| {
            resolution(db, ws, d.input)
                .selected
                .iter()
                .filter_map(move |(name, own)| {
                    let mine = resolved.selected.get(name).filter(|mine| *mine != own)?;
                    Some(
                        Report::new(
                            C::UnresolvedDependencies,
                            at,
                            format!(
                                "`{dep}` was checked against `{name}` {}, but this package's resolution selects {}",
                                own.version, mine.version
                            ),
                        )
                        .note("one version of each package serves the whole dependency graph: align the requirements"),
                    )
                })
        })
        .collect()
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
    if let (Some(_), Some(ws)) = (m, ws) {
        reports.extend(dependency_reports(db, ws, pkg, text));
    }
    reports.extend(crate::change::class_reports(db, pkg, text));
    let checks: Vec<&ModuleCheck> = pkg.files(db).iter().map(|f| check_module(db, *f)).collect();
    let mut diagnostics: Vec<Diagnostic> = reports
        .iter()
        .map(|r| r.to_diagnostic(&manifest_file, &index))
        .collect();
    diagnostics.extend(checks.iter().flat_map(|c| c.diagnostics.iter().cloned()));
    diagnostics.extend(namespace_duplicates(db, pkg));
    diagnostics.extend(former_ids(checks.iter().filter_map(|c| c.module.as_ref())));
    diagnostics.extend(
        crate::precedence::query::legal_checks(db, pkg)
            .iter()
            .cloned(),
    );
    let starts: Vec<_> = checks.iter().flat_map(|c| c.starts.iter()).collect();
    diagnostics.extend(shape(
        &checks,
        &starts,
        m.is_some_and(|m| m.links),
        to_span(&manifest_file, &index, find(text, "[package]")),
    ));
    sort_diagnostics(&mut diagnostics);
    let holes = checks
        .iter()
        .flat_map(|c| c.holes.iter().cloned())
        .collect();
    let error = diagnostics
        .iter()
        .any(|d| d.severity == litigate_base::Severity::Error);
    // An invalid manifest is itself an error, so `m` is present whenever
    // the package is error-free.
    let package = m.filter(|_| !error).map(|m| {
        let mut order: Vec<&ModuleCheck> = checks.clone();
        order.sort_by(|a, b| {
            a.module
                .as_ref()
                .map(|m| &m.name)
                .cmp(&b.module.as_ref().map(|m| &m.name))
        });
        hir::Package {
            manifest: m.clone(),
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
            resolved: ws
                .map(|ws| resolution(db, ws, pkg))
                .into_iter()
                .flat_map(|r| &r.selected)
                .filter(|(_, s)| s.input != pkg)
                .map(|(name, s)| (name.clone(), s.version.clone()))
                .collect(),
        }
    });
    PackageCheck {
        package,
        diagnostics,
        holes,
    }
}

/// One `start` (none in a links package), no nodes in a links package,
/// and a pack identity in the manifest of a package with nodes.
fn shape(
    checks: &[&ModuleCheck],
    starts: &[&(Option<String>, litigate_base::Span)],
    links_manifest: bool,
    manifest_span: litigate_base::Span,
) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    if links_manifest && checks.iter().any(|c| c.first_node.is_some()) {
        diagnostics.push(Diagnostic::error(
            C::InvalidManifest,
            manifest_span.clone(),
            "invalid Lit.toml: missing field `pack_id` (a package with nodes needs a pack identity)",
        ));
    }
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

/// A `formerly` id that is some node's current id, or that two nodes both
/// claim (L1001); the same for the edges of one node. Two versions of one
/// node may repeat its `formerly`.
fn former_ids<'m>(modules: impl Iterator<Item = &'m hir::Module>) -> Vec<Diagnostic> {
    let nodes: Vec<&hir::Node> = modules
        .flat_map(|m| &m.declarations)
        .filter_map(|d| match d {
            hir::Declaration::Node(n) => Some(n),
            _ => None,
        })
        .collect();
    let mut out = Vec::new();
    let mut check = |owner: &str,
                     kind: &str,
                     live: &[&str],
                     claims: Vec<(&str, &str, Option<&litigate_base::Span>)>| {
        let mut seen: BTreeMap<&str, &str> = BTreeMap::new();
        for (old, current, span) in claims {
            let clash = if live.contains(&old) {
                Some(format!("{kind} `{old}`{owner} is a current id, so it cannot also be a former id of `{current}`"))
            } else {
                seen.insert(old, current)
                    .filter(|other| *other != current)
                    .map(|other| format!("{kind} `{old}`{owner} is a former id of both `{other}` and `{current}`"))
            };
            // Sema gives every declaration a span.
            out.extend(
                clash
                    .zip(span.cloned())
                    .map(|(message, span)| Diagnostic::error(C::DuplicateName, span, message)),
            );
        }
    };
    let live: Vec<&str> = nodes.iter().map(|n| n.meta.id.as_str()).collect();
    check(
        "",
        "node",
        &live,
        nodes
            .iter()
            .flat_map(|n| {
                n.formerly
                    .iter()
                    .map(|f| (f.as_str(), n.meta.id.as_str(), n.meta.span.as_ref()))
            })
            .collect(),
    );
    let mut by_id: BTreeMap<&str, Vec<&hir::Edge>> = BTreeMap::new();
    for n in &nodes {
        by_id.entry(&n.meta.id).or_default().extend(&n.edges);
    }
    for (node, edges) in by_id {
        let live: Vec<&str> = edges.iter().filter_map(|e| e.id.as_deref()).collect();
        check(
            &format!(" of node `{node}`"),
            "edge",
            &live,
            edges
                .iter()
                .flat_map(|e| {
                    let current = e.id.as_deref().unwrap_or_default();
                    e.formerly
                        .iter()
                        .map(move |f| (f.as_str(), current, e.span.as_ref()))
                })
                .collect(),
        );
    }
    out
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
                    out.push(
                        Report::new(C::DuplicateName, range, format!("`{name}` is also declared in module `{first}`"))
                            .note("names are unique across a package: node ids lower without module prefixes")
                            .to_diagnostic(&FileId(file.path(db)), &span_map(db, *file)),
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

/// The `Lit.lock` that `root` and its transitive dependencies resolve to:
/// one row per package of `root`'s resolution.
#[must_use]
pub fn expected_lock(db: &dyn salsa::Database, ws: Workspace, root: PackageInput) -> LockFile {
    let solution = &resolution(db, ws, root).selected;
    LockFile::new(
        solution
            .values()
            .map(|s| locked(db, solution, s.input))
            .collect(),
    )
}

/// `pkg`'s lock row under `solution`: its own digests, and each direct
/// dependency pinned as `name version` (SPEC §4.2).
fn locked(
    db: &dyn salsa::Database,
    solution: &BTreeMap<String, Selected>,
    pkg: PackageInput,
) -> LockedPackage {
    let m = manifest(db, pkg).as_ref().ok().map(|p| &p.manifest);
    let name = m.map(|m| m.name.clone()).unwrap_or_default();
    let version = m.map(|m| m.version.clone()).unwrap_or_default();
    let language = m.map_or(1, |m| m.language);
    let source = format!("sha256:{}", package_digest(db, pkg));
    let semantic = crate::db::queries::check_package(db, pkg)
        .package
        .as_ref()
        .map_or_else(
            || UNCHECKED.to_owned(),
            |p| format!("sha256:{}", semantic_digest(p)),
        );
    let texts: Vec<String> = m
        .into_iter()
        .flat_map(|m| &m.sources)
        .filter_map(|s| {
            let at = s.path.as_ref().filter(|p| crate::pkg::load::is_l0(p))?;
            let digest = pkg.texts(db).get(at).cloned().flatten();
            Some(format!(
                "{} {at} {}",
                s.id,
                digest.map_or_else(|| "missing".to_owned(), |d| format!("sha256:{d}"))
            ))
        })
        .collect();
    let dependencies: Vec<String> = m
        .into_iter()
        .flat_map(|m| m.dependencies.keys())
        .filter_map(|dep| solution.get(dep).map(|d| format!("{dep} {}", d.version)))
        .collect();
    let mut sorted = (texts, dependencies);
    sorted.0.sort();
    sorted.1.sort();
    LockedPackage {
        name,
        version,
        language,
        source,
        semantic,
        texts: sorted.0,
        dependencies: sorted.1,
    }
}
