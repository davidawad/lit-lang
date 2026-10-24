//! `litigate where ID` (DESIGN §8): resolve an id at any layer to its
//! declaration span, versions, L0 sections, what overrides it, and the
//! JSON pointers it lowers to.
use super::check::checked;
use crate::hirx::{self, local};
use crate::out::{Failure, Outcome, Report};
use litigate_base::{DiagnosticCode, Profile};
use litigate_lower::{Document, Lowerer, Options, SourceMap};
use litigate_sema::hir::{self, DeadlineWindow, Declaration};
use litigate_sema::temporal::world::{meta_of, relation_target};
use serde_json::{json, Value};

/// The search roots: `--path` values, else the library directories.
pub fn roots(paths: &[String], lib: &[String]) -> Result<Vec<String>, Failure> {
    let roots: Vec<String> = if paths.is_empty() {
        crate::ws::library_dirs(lib)
            .iter()
            .map(|p| p.to_string_lossy().into_owned())
            .collect()
    } else {
        paths.to_vec()
    };
    if roots.is_empty() {
        return Err(Failure::new(
            "nothing to search: pass --path or --lib (no `lit` directory here)",
        ));
    }
    Ok(roots)
}

/// `(package, pack id or name, local id, version date)` from an id.
fn split(id: &str) -> (Option<&str>, &str, Option<&str>) {
    let (id, version) = id
        .rsplit_once('@')
        .filter(|(_, v)| v.len() == 10 && v.as_bytes()[4] == b'-')
        .map_or((id, None), |(i, v)| (i, Some(v)));
    match id.split_once("::") {
        Some((pack, rest)) => (Some(pack.split('@').next().unwrap_or(pack)), rest, version),
        None => (None, id, version),
    }
}

fn version_json(meta: &hir::Metadata) -> Value {
    json!({
        "in_force": {"from": meta.effective.from, "to": meta.effective.to},
        "known_at": meta.effective.known_at,
        "span": meta.span,
        "cite": meta.cite,
        "authority": meta.authority,
    })
}

/// Declarations in `packages` relating to `target` (override, supersedes, exceptions, ...).
pub fn overrides(packages: &[&hir::Package], target: &str) -> Vec<Value> {
    packages
        .iter()
        .flat_map(|p| hirx::declarations(p).map(move |d| (*p, d)))
        .flat_map(|(p, d)| {
            let mut found: Vec<Value> = meta_of(d)
                .into_iter()
                .flat_map(|m| m.relations.iter().map(move |r| (m, r)))
                .filter(|(_, r)| relation_target(r).is_some_and(|t| local(t) == target))
                .map(|(m, r)| {
                    let relation = serde_json::to_value(r).ok().and_then(|v| v.get("relation").cloned());
                    json!({"package": p.manifest.name, "id": m.id, "relation": relation, "span": m.span, "cite": m.cite})
                })
                .collect();
            if let Declaration::Exception(e) = d {
                if local(&e.target) == target {
                    found.push(json!({"package": p.manifest.name, "id": e.meta.id, "relation": "exception", "kind": e.kind, "span": e.meta.span, "cite": e.meta.cite}));
                }
            }
            found
        })
        .collect()
}

/// The L0 sections the lowered source maps of `pkg` record for cite `text`
/// (none when `pkg` does not lower or no entry carries that cite).
pub fn cite_sections(all: &[&hir::Package], pkg: &hir::Package, text: &str) -> Option<Value> {
    let out = Lowerer::new(all)
        .with_corpus(crate::verdict::corpus())
        .package(pkg, &Options::default());
    out.iter()
        .flat_map(|out| std::iter::once(&out.source_map).chain(out.clock.as_ref().map(|(_, m)| m)))
        .flat_map(|m| m.entries.iter())
        .filter_map(|e| e.cite.as_ref())
        .find(|c| c.text == text)
        .map(|c| serde_json::to_value(&c.sections).unwrap_or_default())
}

/// JSON pointers `id` lowers to in `pkg`, with the L0 sections of its cite.
fn lowered(all: &[&hir::Package], pkg: &hir::Package, id: &str) -> (Vec<String>, Value) {
    let Ok(out) = Lowerer::new(all)
        .with_corpus(crate::verdict::corpus())
        .package(pkg, &Options::default())
    else {
        return (Vec::new(), Value::Null);
    };
    let maps: Vec<&SourceMap> = std::iter::once(&out.source_map)
        .chain(out.clock.as_ref().map(|(_, m)| m))
        .collect();
    let mut pointers = Vec::new();
    let mut l0 = Value::Null;
    for (n, map) in maps.iter().enumerate() {
        let prefix = if n == 0 { "" } else { "clock:" };
        for e in &map.entries {
            // A versioned entry's `decl` is `id@from` of this same id.
            if e.id == id {
                pointers.push(format!("{prefix}{}", e.pointer));
                if let Some(cite) = &e.cite {
                    l0 = serde_json::to_value(cite).unwrap_or_default();
                }
            }
        }
    }
    if let Document::Pack(pack) = &out.document {
        let corpus = &pkg.manifest.in_force.known_at;
        let uses: Vec<String> = hirx::nodes_at(all, pkg, corpus, Some(corpus))
            .iter()
            .flat_map(|n| n.edges.iter())
            .filter(|e| matches!(e.deadline.as_ref().map(|d| &d.window), Some(DeadlineWindow::Declared { deadline }) if local(deadline) == id))
            .filter_map(|e| e.id.clone())
            .collect();
        for (i, edge) in pack.edges.iter().enumerate() {
            if edge.id.as_ref().is_some_and(|eid| uses.contains(eid)) {
                pointers.push(format!("/edges/{i}/deadline"));
            }
        }
    }
    (pointers, l0)
}

/// `litigate where`.
pub fn where_(id: &str, paths: &[String], lib: &[String]) -> Outcome {
    if let Some(code) = DiagnosticCode::parse(&id.to_ascii_uppercase()) {
        let mut entry = super::explain::entry(code);
        entry["id"] = Value::String(id.to_owned());
        entry["kind"] = Value::String("diagnostic".into());
        return Ok(Report::of(entry));
    }
    let c = checked(&roots(paths, lib)?, Profile::Dev, &Options::default(), lib)?;
    let all = c.session.hir();
    let (pack, name, version) = split(id);
    let in_pack =
        |p: &&hir::Package| pack.is_none_or(|k| p.manifest.pack_id == k || p.manifest.name == k);
    let mut matches = Vec::new();
    for pkg in all.iter().copied().filter(|p| in_pack(p)) {
        let decls: Vec<&Declaration> = hirx::declarations(pkg)
            .filter(|d| hirx::decl_id(d).as_deref() == Some(name))
            .collect();
        let edges: Vec<(&hir::Node, &hir::Edge)> = hirx::nodes(pkg)
            .into_iter()
            .flat_map(|n| n.edges.iter().map(move |e| (n, e)))
            .filter(|(_, e)| e.id.as_deref() == Some(name))
            .collect();
        if decls.is_empty() && edges.is_empty() {
            continue;
        }
        let versions: Vec<&hir::Metadata> = decls
            .iter()
            .filter_map(|d| meta_of(d))
            .filter(|m| version.is_none_or(|v| hirx::in_force(m, v)))
            .collect();
        let kind = decls.first().map_or("edge", |d| hirx::decl_kind(d));
        let span = versions
            .first()
            .and_then(|m| m.span.clone())
            .or_else(|| edges.first().and_then(|(_, e)| e.span.clone()));
        let (pointers, l0) = lowered(&all, pkg, name);
        matches.push(json!({
            "id": format!("{}::{name}", pkg.manifest.pack_id),
            "kind": kind,
            "package": pkg.manifest.name,
            "declaration_span": span,
            "versions": versions.iter().map(|m| version_json(m)).collect::<Vec<_>>(),
            "edges": edges.iter().map(|(n, e)| json!({"from": n.meta.id, "to": e.to, "label": e.label, "actor": e.actor, "span": e.span})).collect::<Vec<_>>(),
            "overrides": overrides(&all, name),
            "source": l0,
            "lowered": pointers,
        }));
    }
    if matches.is_empty() {
        let needle = id.to_lowercase();
        let cited: Vec<Value> = all
            .iter()
            .flat_map(|p| hirx::declarations(p).map(move |d| (*p, d)))
            .filter_map(|(p, d)| {
                let m = meta_of(d)?;
                m.cite.as_ref().filter(|c| c.to_lowercase().contains(&needle))?;
                Some(json!({"id": format!("{}::{}", p.manifest.pack_id, m.id), "kind": hirx::decl_kind(d), "cite": m.cite, "span": m.span}))
            })
            .collect();
        if cited.is_empty() {
            return Err(Failure::new(format!("`{id}` names no declaration, edge, diagnostic code, or cite in the searched packages")));
        }
        return Ok(Report {
            result: json!({"id": id, "kind": "cite", "cited_by": cited}),
            provenance: c.provenance(None, None),
            ..Report::default()
        });
    }
    let result = if matches.len() == 1 {
        matches.remove(0)
    } else {
        json!({"id": id, "kind": "ambiguous", "matches": matches})
    };
    Ok(Report {
        result,
        provenance: c.provenance(None, None),
        ..Report::default()
    })
}

#[cfg(test)]
mod tests {
    use super::split;
    use proptest::prelude::*;

    #[test]
    fn ids_split_into_pack_name_version() {
        assert_eq!(
            split("p::answer-period@2009-12-01"),
            (Some("p"), "answer-period", Some("2009-12-01"))
        );
        assert_eq!(
            split("cafc@ptab::affirmed"),
            (Some("cafc"), "affirmed", None)
        );
        assert_eq!(split("respond"), (None, "respond", None));
        // Ten characters after `@` that are not a date stay in the id.
        assert_eq!(
            split("respond@abcdefghij"),
            (None, "respond@abcdefghij", None)
        );
        assert_eq!(split("respond@2009"), (None, "respond@2009", None));
    }

    proptest! {
        /// `pack::name@date` always splits back into its three parts.
        #[test]
        fn qualified_versioned_ids_round_trip(
            pack in "[a-z][a-z0-9-]{0,12}",
            name in "[a-z][a-z0-9-]{0,12}",
            date in "[0-9]{4}-[0-9]{2}-[0-9]{2}",
        ) {
            let id = format!("{pack}::{name}@{date}");
            prop_assert_eq!(
                split(&id),
                (Some(pack.as_str()), name.as_str(), Some(date.as_str()))
            );
        }
    }
}
