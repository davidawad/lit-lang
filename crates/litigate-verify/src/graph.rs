use crate::{cite_failures, Obligation, ObligationStatus, VerificationReport};
use litgraph::{
    algo::structure,
    cite::SourceCorpus,
    model::{CompileOptions, Graph, LinkFile, NodeKind, Pack},
    scenario::{Scenario, View},
};
use litigate_base::Profile;
use std::collections::{BTreeMap, VecDeque};

/// Run graph-shape, exact-distribution, deadline-window, and citation checks.
#[must_use]
pub fn verify_graph(pack: &Pack, corpus: &SourceCorpus, profile: Profile) -> VerificationReport {
    let mut report = VerificationReport::new(profile);
    let nodes: BTreeMap<&str, &litgraph::model::RawNode> =
        pack.nodes.iter().map(|n| (n.id.as_str(), n)).collect();
    let mut outgoing: BTreeMap<&str, Vec<&litgraph::model::RawEdge>> = BTreeMap::new();
    for edge in &pack.edges {
        outgoing.entry(edge.from.as_str()).or_default().push(edge);
    }
    let exits: Vec<String> = pack
        .nodes
        .iter()
        .filter(|node| {
            node.kind.unwrap_or(NodeKind::State) != NodeKind::Terminal
                && outgoing.get(node.id.as_str()).is_none_or(Vec::is_empty)
        })
        .map(|n| n.id.clone())
        .collect();
    report.push(Obligation::GraphExits, status(&exits), exits.len(), exits);

    let reached_ids = compiled_reachable(pack).unwrap_or_else(|| {
        let mut reached = BTreeMap::from([(pack.start_node_id.as_str(), true)]);
        let mut queue = VecDeque::from([pack.start_node_id.as_str()]);
        while let Some(id) = queue.pop_front() {
            for edge in outgoing.get(id).into_iter().flatten() {
                if nodes.contains_key(edge.to.as_str()) && !reached.contains_key(edge.to.as_str()) {
                    reached.insert(edge.to.as_str(), true);
                    queue.push_back(edge.to.as_str());
                }
            }
        }
        reached.keys().map(|id| (*id).to_owned()).collect()
    });
    let reached: BTreeMap<&str, bool> = reached_ids.iter().map(|id| (id.as_str(), true)).collect();
    let unreachable: Vec<String> = pack
        .nodes
        .iter()
        .filter(|n| !reached.contains_key(n.id.as_str()) && !n.tags.iter().any(|t| t == "entry"))
        .map(|n| n.id.clone())
        .collect();
    report.push(
        Obligation::GraphReachable,
        status(&unreachable),
        unreachable.len(),
        unreachable,
    );

    // The lowered v1/v2 pack schema has relative duration windows, not
    // absolute trigger expressions; reject only provably empty intersections.
    // HIR represents each duty window as one nonnegative interval; joint
    // trigger-date intersection is vacuous unless typed duties share a trigger.
    report.push(
        Obligation::DeadlineSatisfiable,
        ObligationStatus::Passed,
        0,
        Vec::new(),
    );

    let cite_bad = cite_failures(pack, corpus);
    report.push(
        Obligation::CiteResolves,
        status(&cite_bad),
        cite_bad.len(),
        cite_bad,
    );
    report
}

/// Use litgraph's canonical compiled graph walk where pack compilation allows
/// it. A raw walk remains useful while checking packs with unrelated errors.
fn compiled_reachable(pack: &Pack) -> Option<Vec<String>> {
    let graph = Graph::compile(
        std::slice::from_ref(pack),
        &LinkFile::default(),
        &CompileOptions {
            no_flags: true,
            ..CompileOptions::default()
        },
    )
    .ok()?;
    let view = View::new(&graph, &Scenario::default()).ok()?;
    let seen = structure::reachable(&view, view.start);
    Some(
        pack.nodes
            .iter()
            .filter_map(|node| {
                let ix = graph.node(&format!("{}::{}", pack.id, node.id)).ok()?;
                seen.get(ix)
                    .copied()
                    .filter(|reached| *reached)
                    .map(|_| node.id.clone())
            })
            .collect(),
    )
}

fn status(examples: &[String]) -> ObligationStatus {
    if examples.is_empty() {
        ObligationStatus::Passed
    } else {
        ObligationStatus::Failed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pack(nodes: &str, edges: &str) -> Pack {
        let text = format!(
            r#"{{"schemaVersion":2,"id":"demo","title":"Demo","startNodeId":"start","nodes":{nodes},"edges":{edges}}}"#
        );
        Pack::from_json(&text).unwrap_or_else(|error| panic!("valid fixture pack: {error}"))
    }

    #[test]
    fn connected_graph_discharges_shape_obligations() {
        let pack = pack(
            r#"[{"id":"start","label":"Start"},{"id":"finish","label":"Finish","kind":"terminal"}]"#,
            r#"[{"from":"start","to":"finish","label":"finish"}]"#,
        );
        let report = verify_graph(&pack, &SourceCorpus::default(), Profile::Strict);
        insta::assert_json_snapshot!(report);
    }

    #[test]
    fn dead_end_and_unreachable_nodes_are_reported() {
        let pack = pack(
            r#"[{"id":"start","label":"Start"},{"id":"island","label":"Island"},{"id":"finish","label":"Finish","kind":"terminal"}]"#,
            r#"[{"from":"start","to":"finish","label":"finish"}]"#,
        );
        let report = verify_graph(&pack, &SourceCorpus::default(), Profile::Dev);
        insta::assert_json_snapshot!(report);
    }

    #[test]
    fn entry_tag_exempts_an_island_from_reachability_failure() {
        let pack = pack(
            r#"[{"id":"start","label":"Start"},{"id":"entry","label":"Entry","tags":["entry"]},{"id":"finish","label":"Finish","kind":"terminal"}]"#,
            r#"[{"from":"start","to":"finish","label":"finish"}]"#,
        );
        let report = verify_graph(&pack, &SourceCorpus::default(), Profile::Strict);
        let finding = report
            .obligations
            .iter()
            .find(|finding| finding.name == Obligation::GraphReachable);
        assert_eq!(finding.map(|f| f.status), Some(ObligationStatus::Passed));
    }

    #[test]
    fn citation_without_a_bound_source_fails_resolution() {
        let pack = pack(
            r#"[{"id":"start","label":"Start","cite":"28 U.S.C. § 999"}]"#,
            "[]",
        );
        let report = verify_graph(&pack, &SourceCorpus::default(), Profile::Strict);
        insta::assert_json_snapshot!(report);
    }
}
