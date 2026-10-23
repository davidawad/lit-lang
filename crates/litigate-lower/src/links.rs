//! A links package to a litgraph `LinkFile` (SPEC §8.4).
use crate::diag::Sink;
use crate::edge::{self, EdgeCx};
use crate::source_map::{Citations, Entry, SourceMap};
use crate::Build;
use litgraph::model::{
    local_edge_ids, pointer_token, Hole, Instance, LinkFile, Pack, RawEdge, LINKS_SCHEMA_VERSION,
};
use litigate_base::{DiagnosticCode as C, Span};
use litigate_sema::hir::{self, Declaration};
use serde_json::{Map, Value};
use std::collections::BTreeMap;

/// `pkg` as a `LinkFile` and its source map.
pub(crate) fn links(
    build: &Build<'_, '_>,
    pkg: &hir::Package,
    sink: &mut Sink,
) -> (LinkFile, SourceMap) {
    let decls = crate::pack::emitted(build, pkg, sink);
    let cx = EdgeCx {
        selection: &build.selection,
        pack: &pkg.manifest.pack_id,
    };
    let mut links: Vec<(RawEdge, &hir::Edge)> = Vec::new();
    let mut instances = BTreeMap::new();
    let mut spans: BTreeMap<String, Option<Span>> = BTreeMap::new();
    let mut bases: BTreeMap<String, Option<Pack>> = BTreeMap::new();
    let mut instance_holes: Vec<Hole> = Vec::new();
    for d in decls {
        match d {
            Declaration::Link(l) => {
                let mut raw = edge::edge(&cx, sink, l.from.clone(), &l.edge);
                raw.replaces.clone_from(&l.replaces);
                links.push((raw, &l.edge));
            }
            Declaration::Instance(i) => {
                let span = instance_span(pkg, i);
                let base = bases
                    .entry(i.pack.clone())
                    .or_insert_with(|| base_pack(build, &i.pack));
                if let Some(instance) = instance(&cx, sink, i, base.as_ref(), span.as_ref()) {
                    let at = format!("/instances/{}", pointer_token(&i.name));
                    if let Some(hir::Authored::Hole(reason)) = &i.stake {
                        instance_holes.push(Hole {
                            pointer: format!("{at}/stake"),
                            reason: reason.clone(),
                        });
                    }
                    for (edge, p) in &i.patch_edges {
                        let base = format!("{at}/patch_edges/{}", pointer_token(edge));
                        instance_holes.extend(crate::holes::at(&base, &p.holes));
                    }
                    spans.insert(i.name.clone(), span);
                    instances.insert(i.name.clone(), instance);
                }
            }
            Declaration::Node(n) => sink.error(
                C::InvalidField,
                n.meta.span.as_ref(),
                "a links package must declare no nodes",
            ),
            _ => {}
        }
    }
    let link_holes = links
        .iter()
        .enumerate()
        .flat_map(|(i, (_, e))| crate::holes::at(&format!("/links/{i}"), &e.holes));
    let file = LinkFile {
        schema_version: Some(LINKS_SCHEMA_VERSION),
        instances,
        links: links.iter().map(|(e, _)| e.clone()).collect(),
        lowered: Some(build.lowered(link_holes.chain(instance_holes).collect())),
    };
    let citations = Citations::new(litgraph::model::CiteSyntax::default(), &[], build.corpus);
    let link_entries = links
        .iter()
        .zip(local_edge_ids(&file.links))
        .enumerate()
        .map(|(i, ((raw, e), id))| Entry {
            pointer: format!("/links/{i}"),
            id,
            span: e.span.clone(),
            decl: None,
            cite: citations.map(raw.authority.as_ref()),
        });
    let instance_entries = spans.into_iter().map(|(name, span)| Entry {
        pointer: format!("/instances/{}", name.replace('~', "~0").replace('/', "~1")),
        id: name,
        span,
        decl: None,
        cite: None,
    });
    let map = SourceMap {
        kind: "links".to_owned(),
        id: String::new(),
        package: Some(build.package_ref(pkg)),
        entries: link_entries.chain(instance_entries).collect(),
        ..build.map_header()
    };
    (file, map)
}

/// HIR instances carry no span; the links module's first link or the
/// package's first spanned declaration locates them.
fn instance_span(pkg: &hir::Package, _: &hir::Instance) -> Option<Span> {
    pkg.modules
        .iter()
        .flat_map(|m| m.declarations.iter())
        .find_map(|d| match d {
            Declaration::Link(l) => l.edge.span.clone(),
            _ => None,
        })
}

/// The loaded pack an instance copies, lowered at the same dates.
fn base_pack(build: &Build<'_, '_>, pack_id: &str) -> Option<Pack> {
    let pkg = build
        .packages
        .iter()
        .find(|p| !p.manifest.links && p.manifest.pack_id == pack_id)?;
    let mut scratch = Sink::new("");
    Some(crate::pack::pack(build, pkg, &mut scratch).0)
}

/// One instance. Each `patch` names an edge of the loaded base pack and
/// lowers to the merge patch [`patch`] builds (SPEC §8.4).
fn instance(
    cx: &EdgeCx<'_, '_>,
    sink: &mut Sink,
    i: &hir::Instance,
    base: Option<&Pack>,
    span: Option<&Span>,
) -> Option<Instance> {
    let mut patch_edges = BTreeMap::new();
    if !i.patch_edges.is_empty() {
        let Some(base) = base else {
            sink.unsupported(
                span,
                format!(
                    "instance `{}`: its base pack `{}` is not loaded, so its patches cannot be \
                     checked",
                    i.name, i.pack
                ),
            );
            return None;
        };
        let ids = local_edge_ids(&base.edges);
        for (edge_id, p) in &i.patch_edges {
            let Some(edge) = ids
                .iter()
                .position(|id| id == edge_id)
                .and_then(|at| base.edges.get(at))
            else {
                sink.error(
                    C::UnresolvedName,
                    p.span.as_ref().or(span),
                    format!(
                        "instance `{}` patches `{edge_id}`, which `{}` does not have",
                        i.name, i.pack
                    ),
                );
                continue;
            };
            patch_edges.insert(edge_id.clone(), patch(cx, sink, p, edge));
        }
    }
    let probabilities = i
        .probabilities
        .iter()
        .filter_map(|(edge, p)| Some((edge.clone(), sink.number(*p, span)?)))
        .collect();
    Some(Instance {
        pack: i.pack.clone(),
        note: i.note.clone(),
        remove_edges: i.remove_edges.clone(),
        probabilities,
        roles: i
            .roles
            .iter()
            .map(|(a, r)| (a.clone(), crate::pack::role(*r)))
            .collect(),
        payoff_transform: i
            .payoff_transform
            .as_ref()
            .and_then(|e| crate::transform::engine_text(sink, e)),
        stake: match &i.stake {
            Some(hir::Authored::Known(m)) => sink.money(m, span),
            Some(hir::Authored::Hole(_)) | None => None,
        },
        patch_edges,
    })
}

/// The JSON merge patch (RFC 7386) that applies typed patch `p` to `base`:
/// each field `p` writes, as an edge declaration lowers it (SPEC §8.1),
/// replacing the base edge's whole field. A written field that lowers to
/// nothing (a `cost` hole) is null, and a member of a base object field
/// (`deadline`, `duration`, `attrs`) that the written field lacks is null,
/// so no part of the base field survives a merge.
fn patch(cx: &EdgeCx<'_, '_>, sink: &mut Sink, p: &hir::EdgePatch, base: &RawEdge) -> Value {
    let written = [
        ("authority", p.authority.is_some()),
        ("hours", p.hours.is_some()),
        ("cost", p.cost.is_some()),
        ("deadline", p.deadline.is_some()),
        ("duration", p.duration.is_some()),
        ("valence", p.valence.is_some()),
        ("courtListenerUrl", p.court_listener_url.is_some()),
        ("actionId", p.action_id.is_some()),
        ("note", p.note.is_some()),
        ("tags", p.tags.is_some()),
        ("attrs", p.attrs.is_some()),
        ("sets", p.sets.is_some()),
        ("clears", p.clears.is_some()),
        ("requires", p.requires.is_some()),
        ("forbids", p.forbids.is_some()),
    ];
    let e = hir::Edge {
        id: None,
        to: String::new(),
        label: String::new(),
        actor: String::new(),
        authority: p.authority.clone(),
        probability: None,
        hours: p.hours,
        cost: p.cost.clone(),
        event: None,
        deadline: p.deadline.clone(),
        duration: p.duration.clone(),
        valence: p.valence,
        court_listener_url: p.court_listener_url.clone(),
        action_id: p.action_id.clone(),
        note: p.note.clone(),
        tags: p.tags.clone().unwrap_or_default(),
        attrs: p.attrs.clone().unwrap_or_default(),
        sets: p.sets.clone().unwrap_or_default(),
        clears: p.clears.clone().unwrap_or_default(),
        requires: p.requires.clone().unwrap_or_default(),
        forbids: p.forbids.clone().unwrap_or_default(),
        span: p.span.clone(),
        holes: BTreeMap::new(),
    };
    let lowered = json_object(&edge::edge(cx, sink, String::new(), &e));
    let base = json_object(base);
    let mut out = Map::new();
    for (key, _) in written.iter().filter(|(_, w)| *w) {
        let mut value = lowered.get(*key).cloned().unwrap_or_else(|| empty(key));
        if let (Value::Object(new), Some(Value::Object(old))) = (&mut value, base.get(*key)) {
            for member in old.keys() {
                new.entry(member.clone()).or_insert(Value::Null);
            }
        }
        out.insert((*key).to_owned(), value);
    }
    Value::Object(out)
}

/// A written field the lowered edge omits: an empty list, an empty
/// record, or null for a field with no value (a `cost` hole).
fn empty(key: &str) -> Value {
    match key {
        "tags" | "sets" | "clears" | "requires" | "forbids" => Value::Array(Vec::new()),
        "attrs" => Value::Object(Map::new()),
        _ => Value::Null,
    }
}

/// `edge` as a JSON object.
fn json_object(edge: &RawEdge) -> Map<String, Value> {
    serde_json::to_value(edge)
        .ok()
        .and_then(|v| v.as_object().cloned())
        .unwrap_or_default()
}
