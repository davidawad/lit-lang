//! The per-package salsa query running every temporal and precedence check,
//! and the item-tree [`Locate`] that gives them clause-precise spans.
use crate::db::inputs::{PackageInput, SourceFile};
use crate::db::parse::tree::{Item, ItemKind};
use crate::db::queries::{check_module, item_tree, manifest};
use crate::db::span::{to_span, TextRange};
use crate::resolve::sig::{clauses_of, flatten};
use crate::temporal::world::{Decl, Locate, PackInfo, Rank, World};
use crate::types::build::item_name;
use litigate_base::{Diagnostic, FileId, LineIndex, Span};
use std::collections::BTreeMap;

/// `pkg` followed by its transitive dependencies, each once, and the
/// stdlib package `core`, which every package may import undeclared (SPEC
/// §4.3) and whose authority lattice every package shares (§5.5).
fn closure(db: &dyn salsa::Database, pkg: PackageInput) -> Vec<PackageInput> {
    let Some(ws) = pkg.files(db).first().map(|f| f.workspace(db)) else {
        return vec![pkg];
    };
    let by_name: BTreeMap<String, PackageInput> = ws
        .packages(db)
        .iter()
        .filter_map(|p| {
            manifest(db, *p)
                .as_ref()
                .ok()
                .map(|m| (m.manifest.name.clone(), *p))
        })
        .collect();
    let mut out = vec![pkg];
    let mut i = 0;
    while let Some(&p) = out.get(i) {
        i += 1;
        if let Ok(m) = manifest(db, p) {
            let deps = m.manifest.dependencies.keys().map(String::as_str);
            for dep in deps.chain(["core"]) {
                if let Some(&d) = by_name.get(dep) {
                    if !out.contains(&d) {
                        out.push(d);
                    }
                }
            }
        }
    }
    out
}

/// Temporal and precedence diagnostics for `pkg` (L2001, L2007, L3001-L3009
/// as SPEC §5.4-§5.5 assign them), sorted.
#[salsa::tracked(returns(ref))]
pub fn legal_checks(db: &dyn salsa::Database, pkg: PackageInput) -> Vec<Diagnostic> {
    let packages = closure(db, pkg);
    let mut world = World::default();
    let mut files: BTreeMap<String, SourceFile> = BTreeMap::new();
    let mut by_pack = Vec::new();
    for p in &packages {
        let Ok(m) = manifest(db, *p) else {
            continue;
        };
        for f in p.files(db) {
            files.insert(f.path(db), *f);
        }
        by_pack.push(p.files(db).clone());
        let modules = p
            .files(db)
            .iter()
            .filter_map(|f| check_module(db, *f).module.as_ref());
        world.add(PackInfo::of(&m.manifest), modules);
    }
    if world.packs.is_empty() || manifest(db, pkg).is_err() {
        return Vec::new();
    }
    let loc = Source { db, files, by_pack };
    let mut out = super::check(&world, &loc, 0);
    crate::db::span::sort_diagnostics(&mut out);
    out
}

/// Spans from the salsa item trees.
struct Source<'db> {
    db: &'db dyn salsa::Database,
    files: BTreeMap<String, SourceFile>,
    by_pack: Vec<Vec<SourceFile>>,
}

impl Source<'_> {
    /// The item `d` was built from, with its file's span converter.
    fn item<R>(
        &self,
        d: &Decl<'_>,
        f: impl FnOnce(&Item, &dyn Fn(TextRange) -> Span) -> R,
    ) -> Option<R> {
        let span = d.meta.span.as_ref()?;
        let file = *self.files.get(&span.file.0)?;
        let index = LineIndex::new(file.text(self.db));
        let id = FileId(span.file.0.clone());
        let conv = |r: TextRange| to_span(&id, &index, r);
        let tree = &item_tree(self.db, file).tree;
        let item = flatten(tree).into_iter().map(|(i, _)| i).find(|i| {
            item_name(i).is_some_and(|n| n.text == d.meta.id) && {
                let s = conv(i.range);
                s.line == span.line && s.col == span.col
            }
        })?;
        Some(f(item, &conv))
    }
}

impl Locate for Source<'_> {
    fn clause(&self, d: &Decl<'_>, key: &str) -> Option<(Span, Option<Span>)> {
        self.item(d, |item, conv| {
            let special = match (&item.kind, key) {
                (ItemKind::Transition(_, amends, _, _), "amends") => Some(amends.range),
                (ItemKind::Deadline { counted_by, .. }, "counted_by") => Some(counted_by.range),
                (ItemKind::Exception { when, .. } | ItemKind::Default(_, _, when, _), "when") => {
                    Some(when.range)
                }
                (
                    ItemKind::Exception {
                        priority: Some(p), ..
                    },
                    "priority",
                ) => {
                    return Some((conv(p.range), Some(conv(p.range))));
                }
                (_, "") => item_name(item).map(|n| n.range),
                _ => None,
            };
            if let Some(r) = special {
                return Some((conv(r), None));
            }
            clauses_of(item)
                .iter()
                .find(|f| f.key.text == key)
                .map(|f| (conv(f.range), f.values.first().map(|(_, r)| conv(*r))))
        })
        .flatten()
    }

    fn end(&self, d: &Decl<'_>) -> Option<Span> {
        self.item(d, |item, conv| {
            conv(TextRange::new(item.range.end, item.range.end))
        })
    }

    fn rank(&self, rank: &Rank) -> Option<Span> {
        self.by_pack.get(rank.pack)?.iter().find_map(|file| {
            let tree = &item_tree(self.db, *file).tree;
            let range = flatten(tree).into_iter().find_map(|(i, _)| match &i.kind {
                ItemKind::Authority(chain) => chain
                    .windows(2)
                    .find(|w| w[0].text == rank.above && w[1].text == rank.below)
                    .map(|w| w[0].range.cover(w[1].range)),
                _ => None,
            })?;
            let id = FileId(file.path(self.db));
            Some(to_span(&id, &LineIndex::new(file.text(self.db)), range))
        })
    }
}
