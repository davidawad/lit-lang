//! A rule package to a litgraph `Pack` (SPEC §8.1).
use crate::diag::Sink;
use crate::edge::{self, EdgeCx};
use crate::source_map::{Citations, Entry, SourceMap};
use crate::Build;
use litgraph::model::{
    local_edge_ids, CiteCode, CiteRuleForm, CiteShorthand, CiteSyntax, Group, NodeKind, Pack,
    RawEdge, RawNode, Role, Source,
};
use litigate_base::DiagnosticCode as C;
use litigate_sema::hir::{self, ActorRole, Authored, Declaration};
use std::collections::BTreeMap;

/// Modules in module order (SPEC §4.1): their files' package-relative
/// paths, which fix node order. A module without spanned declarations
/// sorts by name; it contributes no nodes.
pub(crate) fn module_order(pkg: &hir::Package) -> Vec<&hir::Module> {
    let file = |m: &hir::Module| {
        m.declarations
            .iter()
            .find_map(|d| litigate_sema::temporal::world::meta_of(d)?.span.clone())
            .map(|s| s.file.0)
    };
    let mut modules: Vec<(Option<String>, &hir::Module)> =
        pkg.modules.iter().map(|m| (file(m), m)).collect();
    modules.sort_by(|(a, x), (b, y)| a.cmp(b).then_with(|| x.name.cmp(&y.name)));
    modules.into_iter().map(|(_, m)| m).collect()
}

/// The declarations of `pkg` this build emits, in module then source order.
pub(crate) fn emitted<'p>(
    build: &Build<'_, '_>,
    pkg: &'p hir::Package,
    sink: &mut Sink,
) -> Vec<&'p Declaration> {
    module_order(pkg)
        .into_iter()
        .flat_map(|m| m.declarations.iter())
        .filter(|d| match build.selection.emits(&pkg.manifest.pack_id, d) {
            Ok(keep) => keep,
            Err(diagnostic) => {
                sink.out.push(diagnostic);
                false
            }
        })
        .collect()
}

/// litgraph's role for an actor role.
pub(crate) fn role(r: ActorRole) -> Role {
    match r {
        ActorRole::Protagonist => Role::Me,
        ActorRole::Opponent => Role::Opponent,
        ActorRole::Nature => Role::Nature,
    }
}

/// `[sources.*]` in declared order; `cite_prefixes` and `section_heading`
/// are compiler-only.
fn sources(m: &hir::Manifest) -> Vec<Source> {
    m.sources
        .iter()
        .map(|s| Source {
            id: s.id.clone(),
            title: s.title.clone(),
            url: s.url.clone(),
            path: s.path.clone(),
            sha256: s.sha256.clone(),
            as_of: s.as_of.clone(),
        })
        .collect()
}

/// `[[cite_syntax.*]]` as `Pack.citeSyntax`.
pub(crate) fn cite_syntax(sink: &mut Sink, m: &hir::Manifest) -> CiteSyntax {
    let rules = m
        .cite_syntax
        .rules
        .iter()
        .map(|r| CiteRuleForm {
            prefixes: r.prefixes.clone(),
            heading: r.heading.clone(),
            source: r.source.clone(),
        })
        .collect();
    let shorthands = m
        .cite_syntax
        .shorthands
        .iter()
        .filter_map(|s| {
            let code = match s.code.as_str() {
                "usc" => CiteCode::Usc,
                "cfr" => CiteCode::Cfr,
                other => {
                    sink.unsupported(None, format!("cite_syntax code `{other}` (usc or cfr)"));
                    return None;
                }
            };
            let (Ok(title), Ok(section_parts)) =
                (u16::try_from(s.title), u8::try_from(s.section_parts))
            else {
                sink.unsupported(
                    None,
                    format!("cite_syntax shorthand `{}` is out of range", s.prefix),
                );
                return None;
            };
            Some(CiteShorthand {
                prefix: s.prefix.clone(),
                code,
                title,
                section_parts,
                source: s.source.clone(),
            })
        })
        .collect();
    CiteSyntax { rules, shorthands }
}

/// A node, without its edges.
fn node(sink: &mut Sink, n: &hir::Node) -> RawNode {
    let span = n.meta.span.as_ref();
    let (kind, outcome, payoff, payoff_by_flag) = match &n.kind {
        hir::NodeKind::State | hir::NodeKind::Chance { .. } => {
            (NodeKind::State, Vec::new(), None, BTreeMap::new())
        }
        hir::NodeKind::Decision => (NodeKind::Decision, Vec::new(), None, BTreeMap::new()),
        hir::NodeKind::Terminal {
            outcomes,
            payoff,
            payoff_by_flag,
        } => (
            NodeKind::Terminal,
            outcomes.clone(),
            match payoff {
                Some(Authored::Known(m)) => sink.money(m, span),
                Some(Authored::Hole(_)) | None => None,
            },
            payoff_by_flag
                .iter()
                .filter_map(|(flag, m)| Some((flag.clone(), sink.money(m, span)?)))
                .collect(),
        ),
    };
    RawNode {
        id: n.meta.id.clone(),
        kind: Some(kind),
        label: n.meta.label.clone(),
        cite: n.meta.cite.clone(),
        note: n.note.clone(),
        group: n.group.clone(),
        valence: n.valence.map(edge::valence),
        court_listener_url: n.court_listener_url.clone(),
        payoff,
        outcome,
        tags: n.tags.clone(),
        attrs: edge::attrs(sink, &n.attrs, span),
        payoff_by_flag,
    }
}

/// `Pack.schemaVersion`, `Pack.counting`: version 3 exactly when the
/// package has a counting rule (SPEC §8.1).
fn counting(sink: &mut Sink, pkg: &hir::Package, decls: &[&Declaration]) -> Option<String> {
    let declared: Vec<&hir::Counting> = decls
        .iter()
        .filter_map(|d| match d {
            Declaration::Counting(c) => Some(c),
            _ => None,
        })
        .collect();
    if let [_, second, ..] = declared.as_slice() {
        sink.unsupported(
            second.meta.span.as_ref(),
            "a pack has one counting rule (`Pack.counting`); this package declares more than one",
        );
    }
    match (&pkg.manifest.counting, declared.first()) {
        (Some(id), Some(c)) if *id != c.meta.id => {
            sink.unsupported(
                c.meta.span.as_ref(),
                format!(
                    "Lit.toml names counting rule `{id}`, but the package declares `{}`",
                    c.meta.id
                ),
            );
            None
        }
        (Some(id), _) => Some(id.clone()),
        (None, Some(c)) => Some(c.meta.id.clone()),
        (None, None) => None,
    }
}

/// `pkg` as a `Pack` and its source map.
pub(crate) fn pack(
    build: &Build<'_, '_>,
    pkg: &hir::Package,
    sink: &mut Sink,
) -> (Pack, SourceMap) {
    let m = &pkg.manifest;
    let decls = emitted(build, pkg, sink);
    let cx = EdgeCx {
        selection: &build.selection,
        pack: &m.pack_id,
    };
    let mut nodes = Vec::new();
    let mut edges: Vec<(RawEdge, &hir::Edge)> = Vec::new();
    let mut origins = Vec::new();
    for d in &decls {
        match d {
            Declaration::Node(n) => {
                nodes.push(node(sink, n));
                origins.push(n);
                edges.extend(
                    n.edges
                        .iter()
                        .map(|e| (edge::edge(&cx, sink, n.meta.id.clone(), e), e)),
                );
            }
            Declaration::Link(_) | Declaration::Instance(_) => sink.error(
                C::InvalidField,
                None,
                "a package with a pack identity cannot hold links or instances",
            ),
            _ => {}
        }
    }
    let counting = counting(sink, pkg, &decls);
    let pack = Pack {
        schema_version: if counting.is_some() { 3 } else { 2 },
        id: m.pack_id.clone(),
        title: m.title.clone(),
        description: m.description.clone(),
        jurisdiction: m.jurisdiction.clone(),
        forum: Some(m.forum.clone()),
        counting,
        start_node_id: pkg.start.clone(),
        groups: pkg
            .groups
            .iter()
            .map(|g| Group {
                id: g.id.clone(),
                label: g.label.clone(),
            })
            .collect(),
        roles: m.roles.iter().map(|(a, r)| (a.clone(), role(*r))).collect(),
        sources: sources(m),
        cite_syntax: cite_syntax(sink, m),
        nodes,
        edges: edges.iter().map(|(e, _)| e.clone()).collect(),
    };
    let map = pack_map(build, pkg, &pack, &origins, &edges);
    (pack, map)
}

/// The source map of a lowered pack.
fn pack_map(
    build: &Build<'_, '_>,
    pkg: &hir::Package,
    pack: &Pack,
    nodes: &[&hir::Node],
    edges: &[(RawEdge, &hir::Edge)],
) -> SourceMap {
    let citations = Citations::new(pack.cite_syntax.clone(), build.corpus);
    let node_entries = pack
        .nodes
        .iter()
        .zip(nodes)
        .enumerate()
        .map(|(i, (raw, n))| Entry {
            pointer: format!("/nodes/{i}"),
            id: raw.id.clone(),
            span: n.meta.span.clone(),
            decl: Some(crate::select::Selection::versioned(&n.meta)),
            cite: citations.map(raw.cite.as_ref()),
        });
    let ids = local_edge_ids(&pack.edges);
    let edge_entries = edges
        .iter()
        .zip(ids)
        .enumerate()
        .map(|(i, ((raw, e), id))| Entry {
            pointer: format!("/edges/{i}"),
            id,
            span: e.span.clone(),
            decl: None,
            cite: citations.map(raw.authority.as_ref()),
        });
    SourceMap {
        kind: "pack".to_owned(),
        id: pack.id.clone(),
        package: Some(build.package_ref(pkg)),
        entries: node_entries.chain(edge_entries).collect(),
        ..build.map_header()
    }
}
