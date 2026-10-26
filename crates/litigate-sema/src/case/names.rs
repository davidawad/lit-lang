//! How a `.case` names an event type (SPEC §5.6). A matter sees the event
//! types of the packages it `uses` and of their dependencies, its
//! vocabulary. It names one bare when that is unambiguous: the primary
//! package declares it, or no other package of the vocabulary does
//! besides the one that does. Otherwise it writes the package
//! (`pack::Name`, by pack id or package name) or the declaring module's
//! dotted path (`core.time.Served`). [`spell`] gives the shortest of
//! these that [`resolve`] reads back, so whatever prints an event for a
//! matter (`lint-filing`'s moves) prints one the matter accepts.
use super::graph::declarations;
use crate::hir::{Declaration, EventType, Package};
use std::collections::BTreeSet;

/// The packages a matter using `used` (non-empty, `uses` order) sees
/// among `loaded`: `used`, then their transitive dependencies (and
/// `core`), each once.
#[must_use]
pub fn vocabulary<'a>(used: &[&'a Package], loaded: &[&'a Package]) -> Vec<&'a Package> {
    let mut out = used.to_vec();
    out.extend(super::graph::dependencies(used, loaded));
    out
}

/// What an event name written in a matter resolves to.
#[derive(Debug, Clone, PartialEq)]
pub enum Resolution<'a> {
    /// A declared event type, with its pack id.
    Event(&'a str, &'a EventType),
    /// `(pack id, name)`: no event type of that name where the matter
    /// looked; `pack` is where it looked.
    Missing(String, String),
    /// A bare name several packages of the vocabulary declare, none of
    /// them primary: each as `pack::Name`.
    Ambiguous(Vec<String>),
}

/// Event type `name` of package `p`'s declarations `decls`, with its
/// pack id.
fn event<'a>(
    p: &'a Package,
    mut decls: impl Iterator<Item = &'a Declaration>,
    name: &str,
) -> Option<(&'a str, &'a EventType)> {
    decls.find_map(|d| match d {
        Declaration::Event(e) if e.name == name => Some((p.manifest.pack_id.as_str(), e)),
        _ => None,
    })
}

/// The event types `name` of `vocabulary`, one per pack, in vocabulary
/// order.
fn declaring<'a>(vocabulary: &[&'a Package], name: &str) -> Vec<(&'a str, &'a EventType)> {
    let mut seen = BTreeSet::new();
    vocabulary
        .iter()
        .filter_map(|p| event(p, declarations(p), name))
        .filter(|(p, _)| seen.insert(*p))
        .collect()
}

/// The primary pack of `vocabulary` (its first package).
fn primary<'a>(vocabulary: &[&'a Package]) -> &'a str {
    vocabulary
        .first()
        .map_or("", |p| p.manifest.pack_id.as_str())
}

/// Resolves the event name `written` in a matter seeing `vocabulary`
/// (primary first): `Name`, `pack::Name` (pack id or package name), or
/// `module.path.Name`.
#[must_use]
pub fn resolve<'a>(vocabulary: &[&'a Package], written: &str) -> Resolution<'a> {
    let primary = primary(vocabulary);
    let missing = |pack: &str, name: &str| Resolution::Missing(pack.to_owned(), name.to_owned());
    if let Some((q, name)) = written.split_once("::") {
        let pack = vocabulary
            .iter()
            .find(|p| p.manifest.pack_id == q || p.manifest.name == q);
        return match pack {
            Some(p) => event(p, declarations(p), name).map_or_else(
                || missing(&p.manifest.pack_id, name),
                |(pack, ty)| Resolution::Event(pack, ty),
            ),
            None => missing(primary, written),
        };
    }
    if let Some((module, name)) = written.rsplit_once('.') {
        let found = vocabulary.iter().find_map(|p| {
            p.modules
                .iter()
                .filter(|m| m.name == module)
                .find_map(|m| event(p, m.declarations.iter(), name))
        });
        return found.map_or_else(
            || missing(primary, written),
            |(pack, ty)| Resolution::Event(pack, ty),
        );
    }
    let found = declaring(vocabulary, written);
    let own = found.iter().find(|(p, _)| *p == primary);
    match (own, found.as_slice()) {
        (Some((pack, ty)), _) | (None, [(pack, ty)]) => Resolution::Event(pack, ty),
        (None, []) => missing(primary, written),
        (None, many) => Resolution::Ambiguous(
            many.iter()
                .map(|(p, _)| format!("{p}::{written}"))
                .collect(),
        ),
    }
}

/// The shortest spelling of event `name` declared in `pack` that
/// [`resolve`] reads back in a matter seeing `vocabulary`: bare when
/// unambiguous, else `pack::name`.
#[must_use]
pub fn spell(vocabulary: &[&Package], pack: &str, name: &str) -> String {
    if matches!(resolve(vocabulary, name), Resolution::Event(p, _) if p == pack) {
        name.to_owned()
    } else {
        format!("{pack}::{name}")
    }
}
