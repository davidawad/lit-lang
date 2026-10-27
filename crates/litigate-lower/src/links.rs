//! A links package to a litgraph `LinkFile` (SPEC §8.4).
use crate::diag::Sink;
use crate::edge::{self, EdgeCx};
use crate::source_map::{Citations, Entry, GuardOpTrace, GuardTrace, SourceMap};
use crate::Build;
use litigate_base::{DiagnosticCode as C, Span};
use litigate_engine::model::{
    local_edge_ids, pointer_token, CiteSyntax, Hole, Instance, LinkFile, Pack, RawEdge,
    LINKS_SCHEMA_VERSION,
};
use litigate_sema::guard;
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
        counting: None,
    };
    let mut links: Vec<(RawEdge, &hir::Edge)> = Vec::new();
    let mut instances = BTreeMap::new();
    let mut spans: BTreeMap<String, Option<Span>> = BTreeMap::new();
    let mut bases: BTreeMap<String, Option<Pack>> = BTreeMap::new();
    let (mut instance_holes, mut guards): (Vec<Hole>, Vec<GuardTrace>) = (Vec::new(), Vec::new());
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
                    instance_holes.extend(holes_of(i));
                    guards.extend(guard_traces(build, &pkg.manifest.pack_id, i, &instance));
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
    let basis = crate::basis::labels("/links", links.iter().map(|(_, e)| e.basis.as_ref()));
    let lowered = build.lowered(link_holes.chain(instance_holes).collect());
    let file = LinkFile {
        schema_version: Some(LINKS_SCHEMA_VERSION),
        instances,
        links: links.iter().map(|(e, _)| e.clone()).collect(),
        lowered: Some(lowered.with_basis(basis)),
    };
    let citations = Citations::new(CiteSyntax::default(), &[], build.corpus);
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
        guards,
        ..build.map_header()
    };
    (file, map)
}

/// The guard provenance of instance `i`'s patches that lowered (SPEC
/// §3.6): each patched edge's base guard, its operations in the order
/// they apply with their authority and whether each is in force at the
/// build dates, and the guard they compose then.
fn guard_traces(
    build: &Build<'_, '_>,
    from: &str,
    i: &hir::Instance,
    lowered: &Instance,
) -> Vec<GuardTrace> {
    let base = i.pack.as_str();
    let selection = &build.selection;
    let (as_of, known_at) = (selection.as_of.as_str(), selection.known_at.as_deref());
    i.patch_edges
        .iter()
        .filter(|(edge, p)| !p.guard_ops.is_empty() && lowered.patch_edges.contains_key(*edge))
        .map(|(edge, p)| {
            let guard = selection
                .node_with_edge(base, edge)
                .and_then(|n| n.edges.iter().find(|e| e.id.as_deref() == Some(edge)))
                .and_then(|e| e.guard.as_ref());
            let ops: Vec<hir::GuardOp> = p
                .guard_ops
                .iter()
                .map(|op| guard::respelled(op, from, base))
                .collect();
            let refs: Vec<&hir::GuardOp> = ops.iter().collect();
            let composed = guard::compose(selection.lattice(), guard, &refs, as_of, known_at);
            GuardTrace {
                pointer: format!(
                    "/instances/{}/patch_edges/{}",
                    pointer_token(&i.name),
                    pointer_token(edge)
                ),
                base: guard.map(guard::text),
                ops: guard::order(selection.lattice(), &refs)
                    .into_iter()
                    .map(|k| {
                        let a = guard::Applied::of(refs[k]);
                        GuardOpTrace {
                            op: a.op.name().to_owned(),
                            condition: a.condition,
                            by: a.by,
                            cite: a.cite,
                            from: a.from,
                            to: a.to,
                            known_at: a.known_at,
                            applied: composed.applied.contains(&k),
                            span: refs[k].span.clone(),
                        }
                    })
                    .collect(),
                composed: composed.guard.as_ref().map(guard::text),
            }
        })
        .collect()
}

/// The holes of instance `i`: a `stake` hole and each patch's.
fn holes_of(i: &hir::Instance) -> Vec<Hole> {
    let at = format!("/instances/{}", pointer_token(&i.name));
    let stake = match &i.stake {
        Some(hir::Authored::Hole(reason)) => Some(Hole {
            pointer: format!("{at}/stake"),
            reason: reason.clone(),
        }),
        _ => None,
    };
    let patches = i.patch_edges.iter().flat_map(|(edge, p)| {
        let base = format!("{at}/patch_edges/{}", pointer_token(edge));
        crate::holes::at(&base, &p.holes)
    });
    stake.into_iter().chain(patches).collect()
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
        spawn: Vec::new(),
        joins: Vec::new(),
        anytime: false,
        guard: None,
        basis: None,
        formerly: Vec::new(),
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
