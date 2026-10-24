//! Bead acceptance: every legacy pack and `links.json` in the repository,
//! imported by `litigate-import`, checked by sema and lowered, is
//! semantically equal to the original (SPEC §8.6, §8.7).
mod support;

use litgraph::cite::SourceCorpus;
use litigate_lower::{render, semantic_diff, semantic_diff_links, Document, Lowerer, Options};
use support::{built_path, corpus, imported_hir, legacy, repo_root};

#[test]
fn import_then_lower_is_semantically_equal() {
    let inputs = corpus();
    let packages = imported_hir(&inputs);
    let refs: Vec<&litigate_sema::hir::Package> = packages.iter().map(|(_, p)| p).collect();
    let lowerer = Lowerer::new(&refs);
    let mut checked = Vec::new();
    for (origin, pkg) in &packages {
        let original = legacy(&std::fs::read_to_string(built_path(origin)).unwrap_or_default());
        let lowered = lowerer
            .package(pkg, &Options::default())
            .unwrap_or_else(|d| {
                panic!(
                    "{origin}: {}",
                    serde_json::to_string_pretty(&d).unwrap_or_default()
                )
            });
        let changes = match &lowered.document {
            Document::Pack(pack) => {
                let expected = litgraph::Pack::from_json(&original)
                    .unwrap_or_else(|e| panic!("{origin}: {e}"));
                semantic_diff(&expected, pack)
            }
            Document::Links(links) => {
                let expected: litgraph::model::LinkFile =
                    serde_json::from_str(&original).unwrap_or_else(|e| panic!("{origin}: {e}"));
                semantic_diff_links(&expected, links)
            }
        };
        assert!(
            changes.is_empty(),
            "{origin}: {}",
            serde_json::to_string_pretty(&changes).unwrap_or_default()
        );
        checked.push(origin.clone());
    }
    assert_eq!(checked.len(), 10, "{checked:?}");
}

/// The lowered documents, re-read from their rendered text, compile into
/// one engine graph: the output is exactly what litgraph accepts.
#[test]
fn lowered_output_compiles_in_the_engine() {
    let packages = imported_hir(&corpus());
    let refs: Vec<&litigate_sema::hir::Package> = packages.iter().map(|(_, p)| p).collect();
    let lowerer = Lowerer::new(&refs);
    let mut packs = Vec::new();
    let mut links = litgraph::model::LinkFile::default();
    for pkg in &refs {
        let lowered = lowerer
            .package(pkg, &Options::default())
            .unwrap_or_else(|d| panic!("{d:#?}"));
        match &lowered.document {
            Document::Pack(p) => {
                let text = render(p).unwrap_or_default();
                packs.push(litgraph::Pack::from_json(&text).unwrap_or_else(|e| panic!("{e}")));
            }
            Document::Links(l) => {
                let text = render(l).unwrap_or_default();
                links = serde_json::from_str(&text).unwrap_or_else(|e| panic!("{e}"));
            }
        }
    }
    let graph =
        litgraph::Graph::compile(&packs, &links, &litgraph::model::CompileOptions::default());
    assert!(graph.is_ok(), "{:?}", graph.err());
}

/// With the vendored corpus, a node's cite in the source map points at its
/// `## Rule N` section of `sources/frcp.txt`.
#[test]
fn source_maps_locate_cites_in_l0() {
    let packages = imported_hir(&corpus());
    let refs: Vec<&litigate_sema::hir::Package> = packages.iter().map(|(_, p)| p).collect();
    let frcp = refs
        .iter()
        .find(|p| p.manifest.pack_id == "frcp-civil-procedure")
        .unwrap_or_else(|| panic!("no frcp"));
    let lowered = Lowerer::new(&refs)
        .with_corpus(SourceCorpus::embedded_ref())
        .package(frcp, &Options::default())
        .unwrap_or_else(|d| panic!("{d:#?}"));
    let map = &lowered.source_map;
    assert_eq!(map.generated.by, "litigate-lower");
    let entry = map
        .entries
        .iter()
        .find(|e| e.id == "service-window")
        .unwrap_or_else(|| panic!("no service-window entry"));
    assert!(entry
        .span
        .as_ref()
        .is_some_and(|s| std::path::Path::new(&s.file.0)
            .extension()
            .is_some_and(|e| e == "lit")));
    let section = entry
        .cite
        .as_ref()
        .and_then(|c| c.sections.first())
        .unwrap_or_else(|| panic!("no cite section: {entry:?}"));
    assert_eq!(section.heading, "Rule 4");
    assert_eq!(section.source.as_deref(), Some("frcp-rules"));
    assert_eq!(section.path.as_deref(), Some("sources/frcp.txt"));
    let text = std::fs::read_to_string(repo_root().join("sources/frcp.txt")).unwrap_or_default();
    let line = section.line.unwrap_or_else(|| panic!("no line"));
    assert_eq!(text.lines().nth(line as usize - 1), Some("## Rule 4"));
    assert_eq!(map.entries.len(), {
        let Document::Pack(p) = &lowered.document else {
            panic!()
        };
        p.nodes.len() + p.edges.len()
    });
}
