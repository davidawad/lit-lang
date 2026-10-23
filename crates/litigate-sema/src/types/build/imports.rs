//! Imports of one module: import diagnostics (L1001-L1003, L1006) and the
//! signatures its scope sees (direct, transitive, and `::`-qualified).
use crate::db::inputs::SourceFile;
use crate::db::parse::tree::{dotted, ItemTree};
use crate::db::queries::{manifest, module_graph, module_header, signature};
use crate::db::span::Report;
use crate::resolve::graph::ImportTarget;
use crate::resolve::sig::Signature;
use litigate_base::{Applicability, DiagnosticCode as C};
use std::collections::BTreeMap;

/// Import-level diagnostics for `file`.
pub(super) fn imports(
    db: &dyn salsa::Database,
    file: SourceFile,
    tree: &ItemTree,
    own: &Signature,
    reports: &mut Vec<Report>,
) -> Vec<String> {
    // Every file of the workspace has a graph node.
    let node = module_graph(db, file.workspace(db)).node(file);
    if let Some(node) = node.filter(|n| n.duplicate) {
        if let (Some(first), Some(last)) = (tree.module.first(), tree.module.last()) {
            reports.push(Report::new(
                C::DuplicateName,
                first.range.cover(last.range),
                format!(
                    "module `{}` is declared by another file of this package",
                    node.module
                ),
            ));
        }
    }
    let mut resolved = Vec::new();
    let mut named: BTreeMap<String, String> = BTreeMap::new();
    let targets = node.map_or(&[][..], |n| n.imports.as_slice());
    for (import, target) in tree.imports.iter().zip(targets) {
        let path = dotted(&import.path);
        let range = import.path.first().map_or(import.range, |f| {
            f.range
                .cover(import.path.last().map_or(f.range, |l| l.range))
        });
        match target {
            ImportTarget::Module(f) => {
                resolved.push(module_header(db, *f).module.clone());
                let sig = signature(db, *f);
                for n in import.names.iter().flatten() {
                    let exists = sig.values.contains_key(&n.text) || sig.types.contains_key(&n.text) || sig.groups.contains(&n.text) || sig.authorities.contains(&n.text);
                    if !exists {
                        let keys = sig.values.keys().chain(sig.types.keys()).chain(sig.groups.iter()).map(String::as_str);
                        let mut report = Report::new(C::UnresolvedName, n.range, format!("module `{path}` declares no `{}`", n.text));
                        if let Some(best) = crate::resolve::suggest::nearest(&n.text, keys) {
                            report = report.fix(n.range, best, Applicability::MaybeIncorrect);
                        }
                        reports.push(report);
                    } else if own.values.contains_key(&n.text) || own.types.contains_key(&n.text) || own.groups.contains(&n.text) {
                        reports.push(Report::new(C::DuplicateName, n.range, format!("`{}` is imported from `{path}` but also declared in this module", n.text)).note("shadowing is not allowed"));
                    } else if let Some(other) = named.insert(n.text.clone(), path.clone()).filter(|o| *o != path) {
                        reports.push(Report::new(C::DuplicateName, n.range, format!("`{}` is already imported from `{other}`", n.text)));
                    }
                }
            }
            ImportTarget::Unknown(best) => {
                let mut report = Report::new(C::UnresolvedName, range, format!("cannot find module `{path}`"));
                if let Some(best) = best {
                    report = report.fix(range, best.clone(), Applicability::MaybeIncorrect);
                }
                reports.push(report);
            }
            ImportTarget::Undeclared(pkg) => reports.push(
                Report::new(C::UndeclaredDependency, range, format!("module `{path}` belongs to package `{pkg}`, which is not a dependency"))
                    .note(format!("add `{pkg} = {{ version = \"^x.y.z\", path = \"...\" }}` to `[dependencies]` in Lit.toml")),
            ),
            ImportTarget::Cycle(cycle) => reports.push(Report::new(C::ImportCycle, range, format!("importing `{path}` creates a cycle")).note(format!("cycle: {}", cycle.join(" -> ")))),
        }
    }
    resolved
}

/// Signatures reachable through `::`: every module of this package and of
/// its declared dependencies.
pub(super) fn pack_signatures(db: &dyn salsa::Database, file: SourceFile) -> Vec<&Signature> {
    let pkg = file.package(db);
    let deps: Vec<String> = manifest(db, pkg)
        .as_ref()
        .map(|m| m.manifest.dependencies.keys().cloned().collect())
        .unwrap_or_default();
    file.workspace(db)
        .packages(db)
        .iter()
        .filter(|p| {
            **p == pkg
                || manifest(db, **p)
                    .as_ref()
                    .is_ok_and(|m| deps.contains(&m.manifest.name))
        })
        .flat_map(|p| p.files(db).iter().map(|f| signature(db, *f)))
        .collect()
}

/// Signatures of `file`'s direct imports and of [`transitive_imports`].
pub(super) fn import_signatures(
    db: &dyn salsa::Database,
    file: SourceFile,
) -> (Vec<&Signature>, Vec<&Signature>) {
    let graph = module_graph(db, file.workspace(db));
    let sigs = |files: Vec<SourceFile>| files.into_iter().map(|f| signature(db, f)).collect();
    (
        sigs(graph.imports(file)),
        sigs(transitive_imports(graph, file)),
    )
}

/// Modules imported by `file`'s imports, transitively (not `file`'s own
/// direct imports), in discovery order. The graph's import edges are
/// acyclic (cycle-closing imports are removed), so `file` is never reached.
fn transitive_imports(
    graph: &crate::resolve::graph::ModuleGraph,
    file: SourceFile,
) -> Vec<SourceFile> {
    let direct = graph.imports(file);
    let mut seen: Vec<SourceFile> = direct.clone();
    let mut out = Vec::new();
    let mut queue: Vec<SourceFile> = direct;
    while let Some(next) = queue.pop() {
        for f in graph.imports(next) {
            if !seen.contains(&f) {
                seen.push(f);
                out.push(f);
                queue.push(f);
            }
        }
    }
    out
}
