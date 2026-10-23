//! `semantic_diff` (SPEC §8.6): defaults, number spelling and edge
//! grouping are invisible; every real change is reported once, against
//! the id it belongs to.
use litgraph::model::{LinkFile, Pack};
use litigate_lower::{
    semantic_diff, semantic_diff_json, semantic_diff_links, ChangeKind, DocumentKind,
};
use proptest::prelude::*;
use serde_json::{json, Value};

fn pack(v: &Value) -> Pack {
    Pack::from_json(&v.to_string()).unwrap_or_else(|e| panic!("{e}"))
}

fn base() -> Value {
    json!({
        "schemaVersion": 2, "id": "p", "title": "P", "startNodeId": "a",
        "groups": [{"id": "g", "label": "G"}],
        "nodes": [
            {"id": "a", "kind": "decision", "label": "A", "group": "g"},
            {"id": "b", "label": "B"},
            {"id": "c", "kind": "terminal", "label": "C", "payoff": 45}
        ],
        "edges": [
            {"from": "a", "to": "b", "label": "go", "actor": "applicant", "deadline": {"length": 30}},
            {"id": "skip", "from": "a", "to": "c", "label": "skip", "probability": 0.5},
            {"from": "b", "to": "c", "label": "end"}
        ]
    })
}

#[test]
fn defaults_and_number_spelling_are_invisible() {
    let explicit = json!({
        "schemaVersion": 2, "id": "p", "title": "P", "startNodeId": "a",
        "groups": [{"id": "g", "label": "G"}],
        "roles": {},
        "nodes": [
            {"id": "a", "kind": "decision", "label": "A", "group": "g", "tags": []},
            {"id": "b", "kind": "state", "label": "B"},
            {"id": "c", "kind": "terminal", "label": "C", "payoff": 45.0}
        ],
        "edges": [
            {"id": "b->c#0", "from": "b", "to": "c", "label": "end", "actor": "either"},
            {"id": "a->b#0", "from": "a", "to": "b", "label": "go", "actor": "applicant",
             "deadline": {"length": 30.0, "unit": "calendar"}},
            {"id": "skip", "from": "a", "to": "c", "label": "skip", "probability": 0.5}
        ]
    });
    assert_eq!(semantic_diff(&pack(&base()), &pack(&explicit)), []);
}

#[test]
fn changes_are_reported_by_id() {
    let mut changed = base();
    changed["nodes"][1]["label"] = json!("B2");
    changed["nodes"][2]["payoff"] = json!(46);
    changed["edges"][1]["probability"] = json!(0.25);
    if let Some(nodes) = changed["nodes"].as_array_mut() {
        nodes.push(json!({"id": "d", "label": "D"}));
    }
    if let Some(edges) = changed["edges"].as_array_mut() {
        edges.remove(2);
    }
    let changes = semantic_diff(&pack(&base()), &pack(&changed));
    insta::assert_json_snapshot!("changes_by_id", changes);
}

#[test]
fn node_order_and_edge_order_within_a_node_are_significant() {
    let mut reordered = base();
    if let Some(nodes) = reordered["nodes"].as_array_mut() {
        nodes.swap(1, 2);
    }
    if let Some(edges) = reordered["edges"].as_array_mut() {
        edges.swap(0, 1);
    }
    let changes = semantic_diff(&pack(&base()), &pack(&reordered));
    let kinds: Vec<(ChangeKind, &str)> = changes
        .iter()
        .map(|c| (c.kind, c.pointer.as_str()))
        .collect();
    assert_eq!(
        kinds,
        [
            (ChangeKind::Order, "/edges/order/a"),
            (ChangeKind::Order, "/nodes/order")
        ]
    );
}

#[test]
fn links_compare_instances_and_derived_link_ids() {
    let a: LinkFile = serde_json::from_value(json!({
        "instances": {"x@y": {"pack": "x", "probabilities": {"e": 1}}},
        "links": [{"from": "y::a", "to": "x@y::b", "label": "up"}]
    }))
    .unwrap_or_default();
    let mut b = a.clone();
    assert_eq!(semantic_diff_links(&a, &b), []);
    b.links[0].id = Some("y::a->x@y::b#0".into());
    assert_eq!(semantic_diff_links(&a, &b), []);
    b.instances
        .get_mut("x@y")
        .map(|i| i.probabilities.insert("e".into(), 0.5));
    let changes = semantic_diff_links(&a, &b);
    assert_eq!(changes.len(), 1);
    assert_eq!(changes[0].pointer, "/instances/x@y/probabilities/e");
    assert_eq!(changes[0].id.as_deref(), Some("x@y"));
}

/// Links have no node list to regroup under: moving a link past one from
/// another source node is an `order` change (tuq.37).
#[test]
fn link_order_across_source_nodes_is_significant() {
    let a: LinkFile = serde_json::from_value(json!({
        "links": [
            {"from": "p::a", "to": "q::b", "label": "one"},
            {"from": "r::c", "to": "q::b", "label": "two"},
            {"from": "p::a", "to": "q::d", "label": "three"}
        ]
    }))
    .unwrap_or_default();
    let mut b = a.clone();
    b.links.swap(0, 1);
    let changes = semantic_diff_links(&a, &b);
    assert_eq!(changes.len(), 1, "{changes:?}");
    assert_eq!(changes[0].kind, ChangeKind::Order);
    assert_eq!(changes[0].pointer, "/links/order");
    assert_eq!(changes[0].id, None);
    assert_eq!(
        changes[0].after,
        Some(json!(["r::c->q::b#0", "p::a->q::b#0", "p::a->q::d#0"]))
    );
}

#[test]
fn json_entry_point_detects_the_document_kind() {
    let text = base().to_string();
    assert_eq!(
        semantic_diff_json(&text, &text),
        Ok((DocumentKind::Pack, Vec::new()))
    );
    let scenario =
        json!({"id": "s", "summary": "S", "scenario": {"params": {"rate": 1}}}).to_string();
    let other = json!({"id": "s", "summary": "S", "scenario": {"params": {"rate": 2}}}).to_string();
    let (kind, changes) = semantic_diff_json(&scenario, &other).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(kind, DocumentKind::Scenario);
    assert_eq!(changes.len(), 1);
    assert_eq!(changes[0].pointer, "/scenario/params/rate");
    assert!(semantic_diff_json(&text, &scenario).is_err());
    assert!(semantic_diff_json("{}", "{}").is_err());
}

/// A random small pack: `n` chain nodes with optional labels/probabilities.
fn arbitrary_pack() -> impl Strategy<Value = Value> {
    prop::collection::vec((any::<bool>(), 0u32..100), 2..8).prop_map(|spec| {
        let nodes: Vec<Value> = spec
            .iter()
            .enumerate()
            .map(|(i, (decision, _))| {
                json!({"id": format!("n{i}"), "kind": if *decision { "decision" } else { "state" }, "label": format!("N{i}")})
            })
            .collect();
        let edges: Vec<Value> = spec
            .iter()
            .enumerate()
            .skip(1)
            .flat_map(|(i, (_, p))| {
                [
                    json!({"from": format!("n{}", i - 1), "to": format!("n{i}"), "label": "next", "probability": f64::from(*p) / 100.0}),
                    json!({"from": format!("n{}", i - 1), "to": "n0", "label": "back"}),
                ]
            })
            .collect();
        json!({"schemaVersion": 2, "id": "r", "title": "R", "startNodeId": "n0", "nodes": nodes, "edges": edges})
    })
}

proptest! {
    #[test]
    fn a_pack_equals_itself_and_its_regrouping(v in arbitrary_pack()) {
        let a = pack(&v);
        prop_assert!(semantic_diff(&a, &a).is_empty());
        // Source groups edges under their node: reversing the groups while
        // keeping each group's order is invisible.
        let mut b = a.clone();
        let mut groups: Vec<Vec<litgraph::model::RawEdge>> = Vec::new();
        for e in b.edges.drain(..) {
            match groups.last_mut() {
                Some(g) if g[0].from == e.from => g.push(e),
                _ => groups.push(vec![e]),
            }
        }
        b.edges = groups.into_iter().rev().flatten().collect();
        prop_assert!(semantic_diff(&a, &b).is_empty(), "{:?}", semantic_diff(&a, &b));
    }

    #[test]
    fn every_label_change_is_one_change(v in arbitrary_pack(), at in any::<prop::sample::Index>()) {
        let a = pack(&v);
        let mut b = a.clone();
        let i = at.index(b.nodes.len());
        b.nodes[i].label.push('!');
        let changes = semantic_diff(&a, &b);
        prop_assert_eq!(changes.len(), 1);
        prop_assert_eq!(changes[0].kind, ChangeKind::Changed);
        prop_assert_eq!(changes[0].id.clone(), Some(b.nodes[i].id.clone()));
    }

    #[test]
    fn any_link_permutation_is_reported(n in 2usize..8, seed in any::<prop::sample::Index>()) {
        let links: Vec<Value> = (0..n)
            .map(|i| json!({"from": format!("p::n{}", i % 3), "to": format!("q::m{i}"), "label": "l"}))
            .collect();
        let a: LinkFile = serde_json::from_value(json!({"links": links})).unwrap_or_default();
        let mut b = a.clone();
        b.links.rotate_left(seed.index(n - 1) + 1);
        let changes = semantic_diff_links(&a, &b);
        prop_assert_eq!(changes.len(), 1);
        prop_assert_eq!(changes[0].kind, ChangeKind::Order);
        prop_assert!(semantic_diff_links(&a, &a).is_empty());
    }
}
