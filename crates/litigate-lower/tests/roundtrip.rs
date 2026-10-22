//! Bead acceptance: every legacy pack and `links.json` in the repository,
//! imported by `litigate-import`, checked by sema and lowered, is
//! semantically equal to the original (SPEC §8.6, §8.7).
mod support;

use litigate_lower::{semantic_diff, semantic_diff_links, Document, Lowerer, Options};
use support::{corpus, imported_hir, repo_root};

#[test]
fn import_then_lower_is_semantically_equal() {
    let inputs = corpus();
    let packages = imported_hir(&inputs);
    let refs: Vec<&litigate_sema::hir::Package> = packages.iter().map(|(_, p)| p).collect();
    let lowerer = Lowerer::new(&refs);
    let mut checked = Vec::new();
    for (origin, pkg) in &packages {
        let original = std::fs::read_to_string(repo_root().join(origin)).unwrap_or_default();
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
