//! A links package to a litgraph `LinkFile` (SPEC §8.4).
use crate::diag::Sink;
use crate::edge::{self, EdgeCx};
use crate::source_map::{Citations, Entry, SourceMap};
use crate::Build;
use litgraph::model::{
    local_edge_ids, merge_patch, pointer_token, Hole, Instance, LinkFile, Pack, RawEdge,
    LINKS_SCHEMA_VERSION,
};
use litigate_base::{DiagnosticCode as C, Span};
use litigate_sema::hir::{self, Declaration};
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
    let mut stake_holes: Vec<Hole> = Vec::new();
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
                if let Some(instance) = instance(sink, i, base.as_ref(), span.as_ref()) {
                    if let Some(hir::Authored::Hole(reason)) = &i.stake {
                        stake_holes.push(Hole {
                            pointer: format!("/instances/{}/stake", pointer_token(&i.name)),
                            reason: reason.clone(),
                        });
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
        lowered: Some(build.lowered(link_holes.chain(stake_holes).collect())),
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

/// `payoff_transform` must parse as a litgraph expression over `payoff`
/// and a declared `stake` (L6003), and a `stake` must be used by it
/// (L2007).
fn transform(sink: &mut Sink, i: &hir::Instance, span: Option<&Span>) {
    let vars = match i.payoff_transform.as_deref().map(litgraph::expr::parse) {
        Some(Ok(expr)) => expr.vars(),
        Some(Err(e)) => {
            sink.error_with(
                C::InvalidEngineExpression,
                span,
                format!("instance `{}`: `payoff_transform` does not parse", i.name),
                e.to_string(),
            );
            Vec::new()
        }
        None => Vec::new(),
    };
    let bound = |v: &str| v == "payoff" || (v == "stake" && i.stake.is_some());
    for v in vars.iter().filter(|v| !bound(v)) {
        sink.error(
            C::InvalidEngineExpression,
            span,
            format!(
                "instance `{}`: `payoff_transform` uses `{v}`, which is neither `payoff` nor \
                 a declared `stake` (write `stake ?(\"why\")` when only the matter knows it)",
                i.name
            ),
        );
    }
    if i.stake.is_some() && !vars.iter().any(|v| v == "stake") {
        sink.error(
            C::InvalidField,
            span,
            format!(
                "instance `{}` declares a `stake` its `payoff_transform` does not use",
                i.name
            ),
        );
    }
}

/// One instance. Each `patch` is applied to its base edge and must still
/// be a valid `RawEdge` (SPEC §8.4); see [`transform`] for the payoffs.
fn instance(
    sink: &mut Sink,
    i: &hir::Instance,
    base: Option<&Pack>,
    span: Option<&Span>,
) -> Option<Instance> {
    transform(sink, i, span);
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
        for (edge_id, patch) in &i.patch_edges {
            let Some(edge) = ids
                .iter()
                .position(|id| id == edge_id)
                .and_then(|at| base.edges.get(at))
            else {
                sink.error(
                    C::UnresolvedName,
                    span,
                    format!(
                        "instance `{}` patches `{edge_id}`, which `{}` does not have",
                        i.name, i.pack
                    ),
                );
                continue;
            };
            let mut patched = serde_json::to_value(edge).unwrap_or_default();
            merge_patch(&mut patched, patch);
            if let Err(e) = serde_json::from_value::<RawEdge>(patched) {
                sink.error_with(
                    C::LoweringUnsupported,
                    span,
                    format!(
                        "instance `{}`: patching `{edge_id}` does not give a valid edge",
                        i.name
                    ),
                    e.to_string(),
                );
            }
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
        payoff_transform: i.payoff_transform.clone(),
        stake: match &i.stake {
            Some(hir::Authored::Known(m)) => sink.money(m, span),
            Some(hir::Authored::Hole(_)) | None => None,
        },
        patch_edges: i.patch_edges.clone(),
    })
}
