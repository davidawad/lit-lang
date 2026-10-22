//! A legacy pack to a package: `Lit.toml`, a root module with the groups,
//! and one module per node section (see [`crate::modules`]).
use crate::issues::Issues;
use crate::manifest;
use crate::modules::{sections, Section};
use crate::node::Spelling;
use crate::text;
use crate::Emitted;
use litgraph::cite::normalize::CiteGrammar;
use litgraph::model::{Pack, RawEdge};
use std::collections::{BTreeMap, BTreeSet};

/// Imports `pack` into the package directory `dir` (also the root module
/// name). `from` is the input path recorded in the header comments.
pub(crate) fn import_pack(pack: &Pack, from: &str, dir: &str, issues: &mut Issues) -> Vec<Emitted> {
    if pack.schema_version != expected_schema(pack) {
        issues.report(
            "/schemaVersion",
            format!(
                "schemaVersion {} is not derivable: lowering emits 3 exactly when the pack names a counting rule, else 2 (SPEC §8.1)",
                pack.schema_version
            ),
        );
    }
    // The directory is also the root module's path, so it must be an
    // identifier (it is the forum key, or else the pack id).
    issues.ident(dir, "/id", false);
    let forum = pack.forum.clone().unwrap_or_else(|| {
        issues.report("/forum", "a package needs a `forum` key (SPEC §4.1)");
        String::new()
    });
    let mut files = vec![Emitted::toml(
        format!("{dir}/Lit.toml"),
        manifest::pack_manifest(pack, from, &forum),
    )];
    let mut root = vec![
        format!("// Imported by `litigate import` from {from}."),
        format!("module {dir}"),
    ];
    for (index, group) in pack.groups.iter().enumerate() {
        let id = issues.ident(&group.id, &format!("/groups/{index}/id"), false);
        root.push(format!("group {id}: {}", text::string(&group.label)));
    }
    files.push(Emitted::lit(format!("{dir}/{dir}.lit"), root.join("\n\n")));
    let parts = sections(pack);
    let width = parts.len().to_string().len().max(2);
    let section_of = section_index(pack, &parts);
    let mut out_edges: BTreeMap<&str, Vec<(usize, &RawEdge)>> = BTreeMap::new();
    for (index, edge) in pack.edges.iter().enumerate() {
        if section_of.contains_key(edge.from.as_str()) {
            out_edges
                .entry(edge.from.as_str())
                .or_default()
                .push((index, edge));
        } else {
            let message = format!("`{}` is not a node of this pack", edge.from);
            issues.report(&format!("/edges/{index}/from"), message);
        }
    }
    let grammar = CiteGrammar::new(&pack.cite_syntax);
    let mut spelling = Spelling { grammar, issues };
    let layout = Layout {
        pack,
        dir,
        from,
        parts: &parts,
        section_of: &section_of,
        out_edges: &out_edges,
    };
    for (number, part) in parts.iter().enumerate() {
        let path = format!("{dir}/{:0width$}-{}.lit", number + 1, part.name);
        files.push(Emitted::lit(path, layout.module(number, &mut spelling)));
    }
    if !pack.nodes.iter().any(|node| node.id == pack.start_node_id) {
        spelling
            .issues
            .report("/startNodeId", "the start node is not a node of this pack");
    }
    files
}

fn expected_schema(pack: &Pack) -> u32 {
    if pack.counting.is_some() {
        3
    } else {
        2
    }
}

/// Node id to the index of its section.
fn section_index<'a>(pack: &'a Pack, parts: &[Section]) -> BTreeMap<&'a str, usize> {
    pack.nodes
        .iter()
        .enumerate()
        .filter_map(|(index, node)| {
            let part = parts.iter().position(|s| s.nodes.contains(&index))?;
            Some((node.id.as_str(), part))
        })
        .collect()
}

/// Everything a section module needs to know about its package.
struct Layout<'a> {
    pack: &'a Pack,
    dir: &'a str,
    from: &'a str,
    parts: &'a [Section],
    section_of: &'a BTreeMap<&'a str, usize>,
    out_edges: &'a BTreeMap<&'a str, Vec<(usize, &'a RawEdge)>>,
}

impl Layout<'_> {
    /// The source of section module `number`.
    fn module(&self, number: usize, spelling: &mut Spelling<'_>) -> String {
        let (pack, dir) = (self.pack, self.dir);
        let part = &self.parts[number];
        let nodes = &pack.nodes[part.nodes.clone()];
        let mut groups = BTreeSet::new();
        let mut imported: BTreeMap<usize, BTreeSet<&str>> = BTreeMap::new();
        for node in nodes {
            if let Some(group) = &node.group {
                groups.insert(group.as_str());
            }
            for (index, edge) in self.out_edges.get(node.id.as_str()).into_iter().flatten() {
                match self.section_of.get(edge.to.as_str()) {
                    Some(&target) if target != number => {
                        imported.entry(target).or_default().insert(edge.to.as_str());
                    }
                    Some(_) => {}
                    None => spelling.issues.report(
                        &format!("/edges/{index}/to"),
                        format!("`{}` is not a node of this pack", edge.to),
                    ),
                }
            }
        }
        let mut out = vec![format!(
            "// Imported by `litigate import` from {}.\nmodule {dir}.{}",
            self.from, part.name
        )];
        if !groups.is_empty() {
            let groups: Vec<&str> = groups.into_iter().collect();
            out.push(format!("import {dir}.{{{}}}", groups.join(", ")));
        }
        for (target, names) in imported {
            let names: Vec<&str> = names.into_iter().collect();
            out.push(format!(
                "import {dir}.{}.{{{}}}",
                self.parts[target].name,
                names.join(", ")
            ));
        }
        if part.nodes.contains(&index_of(pack, &pack.start_node_id)) {
            out.push(format!("start {}", pack.start_node_id));
        }
        for (offset, node) in nodes.iter().enumerate() {
            let edges = self
                .out_edges
                .get(node.id.as_str())
                .map_or(&[][..], Vec::as_slice);
            out.push(spelling.node(part.nodes.start + offset, node, edges));
        }
        out.join("\n\n")
    }
}

fn index_of(pack: &Pack, id: &str) -> usize {
    pack.nodes
        .iter()
        .position(|node| node.id == id)
        .unwrap_or(usize::MAX)
}
