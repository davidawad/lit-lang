//! Resolution per world (design review §4.1): which declaration governs
//! each of a package's logical declarations in a [`WorldId`], memoized per
//! (package, world) and read per (declaration, world). The world is an
//! interned key, so switching worlds re-runs only these queries: syntax,
//! item trees, signatures and typing are world-independent.
use super::query::closure;
use super::select::select;
use crate::db::ids::{DeclId, WorldId};
use crate::db::inputs::PackageInput;
use crate::db::queries::{check_module, decl_item, manifest};
use crate::temporal::world::{PackInfo, World};
use crate::types::build::item_name;
use litigate_base::Span;
use std::collections::BTreeMap;

/// What governs one logical declaration in a world.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Resolution {
    /// The governing declaration: the precedence winner among the
    /// versions in force, possibly another package's override.
    Governed {
        /// Its pack id.
        pack: String,
        /// Its local id.
        id: String,
        /// The `from` date of its version.
        from: String,
        /// Where it is declared.
        span: Option<Span>,
    },
    /// Why nothing governs it, as a sentence.
    Unselected(String),
}

/// The resolution of every logical declaration of `pkg` in `world`, by
/// local id; empty without a valid manifest.
#[salsa::tracked(returns(ref))]
pub fn world_selection<'db>(
    db: &'db dyn salsa::Database,
    pkg: PackageInput,
    world: WorldId<'db>,
) -> BTreeMap<String, Resolution> {
    let Ok(own) = manifest(db, pkg) else {
        return BTreeMap::new();
    };
    let mut w = World::default();
    for (p, m) in closure(db, pkg, &own.manifest) {
        let modules = p
            .files(db)
            .iter()
            .filter_map(|f| check_module(db, *f).module.as_ref());
        w.add(PackInfo::of(m), modules);
    }
    let (as_of, known_at) = (world.as_of(db), world.known_at(db).as_deref());
    w.sets()
        .filter(|(key, _)| key.pack == own.manifest.pack_id)
        .map(|(key, _)| {
            let resolution = match select(&w, key, as_of, known_at) {
                Ok(i) => Resolution::Governed {
                    pack: w.pack_of(i).to_owned(),
                    id: w.decls[i].meta.id.clone(),
                    from: w.decls[i].meta.effective.from.clone(),
                    span: w.decls[i].meta.span.clone(),
                },
                Err(why) => {
                    Resolution::Unselected(why.describe(&w, &format!("`{}`", key.name), as_of))
                }
            };
            (key.name.clone(), resolution)
        })
        .collect()
}

/// What governs top-level declaration `decl` in `world`; `None` for a
/// declaration that is not rule-bearing (types, events, groups, tests).
/// Backdated per declaration, so a world change that leaves this one's
/// winner alone invalidates nothing that reads it.
#[salsa::tracked(returns(ref))]
pub fn resolve_in<'db>(
    db: &'db dyn salsa::Database,
    decl: DeclId<'db>,
    world: WorldId<'db>,
) -> Option<Resolution> {
    let name = decl_item(db, decl).as_ref().and_then(item_name)?;
    world_selection(db, decl.file(db).package(db), world)
        .get(&name.text)
        .cloned()
}
