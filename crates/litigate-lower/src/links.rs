//! A links package to a litgraph `LinkFile` (SPEC §8.4).
use crate::diag::Sink;
use crate::edge::{self, EdgeCx};
use crate::source_map::{Citations, Entry, SourceMap};
use crate::Build;
use litgraph::model::{local_edge_ids, merge_patch, Instance, LinkFile, Pack, RawEdge};
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
    let file = LinkFile {
        instances,
        links: links.iter().map(|(e, _)| e.clone()).collect(),
    };
    let citations = Citations::new(litgraph::model::CiteSyntax::default(), build.corpus);
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

/// One instance. Each `patch` is applied to its base edge and must still
/// be a valid `RawEdge` (SPEC §8.4); `payoff_transform` must parse as a
/// litgraph expression (L6003).
fn instance(
    sink: &mut Sink,
    i: &hir::Instance,
    base: Option<&Pack>,
    span: Option<&Span>,
) -> Option<Instance> {
    if let Some(expr) = &i.payoff_transform {
        if let Err(e) = litgraph::expr::parse(expr) {
            sink.error_with(
                C::InvalidEngineExpression,
                span,
                format!("instance `{}`: `payoff_transform` does not parse", i.name),
                e.to_string(),
            );
        }
    }
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
        patch_edges: i.patch_edges.clone(),
    })
}
