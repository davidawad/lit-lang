// SPDX-License-Identifier: GPL-3.0-or-later
//! Concurrent tracks (lg-ww93): an edge's `spawn` and `joins` compile into
//! a product graph over markings, so a pack with two tracks solves to the
//! value of the same procedure built by hand as one walk, and a marking the
//! engine cannot expand is a clear error.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use litigate_engine::algo::mdp;
use litigate_engine::model::{CompileOptions, Graph, LinkFile, Pack};
use litigate_engine::scenario::{Scenario, View};
use serde_json::{json, Value};

const ROLES: &str = r#"{ "party": "self", "court": "nature" }"#;

fn pack(id: &str, nodes: &Value, edges: &Value) -> Pack {
    serde_json::from_value(json!({
        "schemaVersion": 2, "id": id, "title": id, "startNodeId": "s",
        "roles": serde_json::from_str::<Value>(ROLES).unwrap(),
        "nodes": nodes, "edges": edges
    }))
    .expect("pack parses")
}

fn compile(packs: &[Pack], opts: &CompileOptions) -> litigate_engine::Result<Graph> {
    Graph::compile(packs, &LinkFile::default(), opts)
}

fn value(g: &Graph) -> f64 {
    let v = View::new(g, &Scenario::default()).unwrap();
    mdp::solve(&v, &mdp::SolveOptions::default()).unwrap().value[v.start]
}

/// Filing (100) starts the motion track at `a`, where we choose between
/// two terminals, and spawns the discovery track at `b`, where the court
/// draws a sanction (4,000 a quarter of the time). The value is
/// -100 + max(1,000 - 10, 1,200 - 300) + (0.25 × 4,000 - 50) = 1,840.
fn tracks() -> Pack {
    pack(
        "tracks",
        &json!([
            { "id": "s", "kind": "decision", "label": "Start" },
            { "id": "a", "kind": "decision", "label": "Motion" },
            { "id": "a1", "kind": "terminal", "label": "Settled", "payoff": 1000 },
            { "id": "a2", "kind": "terminal", "label": "Tried", "payoff": 1200 },
            { "id": "b", "label": "Discovery" },
            { "id": "b1", "kind": "terminal", "label": "Sanctioned", "payoff": 4000 },
            { "id": "b2", "kind": "terminal", "label": "Closed", "payoff": 0 }
        ]),
        &json!([
            { "id": "file", "from": "s", "to": "a", "label": "file", "actor": "party", "cost": 100, "spawn": ["b"] },
            { "id": "settle", "from": "a", "to": "a1", "label": "settle", "actor": "party", "cost": 10 },
            { "id": "try", "from": "a", "to": "a2", "label": "try", "actor": "party", "cost": 300 },
            { "id": "sanction", "from": "b", "to": "b1", "label": "sanction", "actor": "court", "probability": 0.25, "cost": 50 },
            { "id": "close", "from": "b", "to": "b2", "label": "close", "actor": "court", "probability": 0.75, "cost": 50 }
        ]),
    )
}

/// `tracks` built by hand as one walk: the motion, then discovery after
/// either outcome, each terminal paying both tracks' payoffs.
fn product() -> Pack {
    pack(
        "product",
        &json!([
            { "id": "s", "kind": "decision", "label": "Start" },
            { "id": "a", "kind": "decision", "label": "Motion" },
            { "id": "x1", "label": "Settled, discovery" },
            { "id": "x2", "label": "Tried, discovery" },
            { "id": "x1b1", "kind": "terminal", "label": "Settled, sanctioned", "payoff": 5000 },
            { "id": "x1b2", "kind": "terminal", "label": "Settled, closed", "payoff": 1000 },
            { "id": "x2b1", "kind": "terminal", "label": "Tried, sanctioned", "payoff": 5200 },
            { "id": "x2b2", "kind": "terminal", "label": "Tried, closed", "payoff": 1200 }
        ]),
        &json!([
            { "from": "s", "to": "a", "label": "file", "actor": "party", "cost": 100 },
            { "from": "a", "to": "x1", "label": "settle", "actor": "party", "cost": 10 },
            { "from": "a", "to": "x2", "label": "try", "actor": "party", "cost": 300 },
            { "from": "x1", "to": "x1b1", "label": "sanction", "actor": "court", "probability": 0.25, "cost": 50 },
            { "from": "x1", "to": "x1b2", "label": "close", "actor": "court", "probability": 0.75, "cost": 50 },
            { "from": "x2", "to": "x2b1", "label": "sanction", "actor": "court", "probability": 0.25, "cost": 50 },
            { "from": "x2", "to": "x2b2", "label": "close", "actor": "court", "probability": 0.75, "cost": 50 }
        ]),
    )
}

#[test]
fn two_tracks_solve_to_the_value_of_their_hand_built_product() {
    let opts = CompileOptions::default();
    let by_hand = value(&compile(&[product()], &opts).unwrap());
    assert!((by_hand - 1840.0).abs() < 1e-9, "{by_hand}");
    let g = compile(&[tracks()], &opts).unwrap();
    let spawned = value(&g);
    assert!((spawned - by_hand).abs() < 1e-9, "{spawned} vs {by_hand}");
    // The markings of several tokens are new nodes; the pack's keep their ids.
    let ids: Vec<&str> = g.nodes.iter().map(|n| n.id.as_str()).collect();
    for id in [
        "tracks::s",
        "tracks::a",
        "tracks::b",
        "tracks::a+b",
        "tracks::a1+b1",
    ] {
        assert!(ids.contains(&id), "{id} missing from {ids:?}");
    }
    // A marking where the motion moves is a copy of the motion's node, and
    // its edges copies of the motion's.
    let ab = &g.nodes[g.node("tracks::a+b").unwrap()];
    assert_eq!(ab.base_id, "tracks::a");
    let settle: Vec<&str> = g
        .edges
        .iter()
        .filter(|e| e.base_id == "tracks::settle")
        .map(|e| e.id.as_str())
        .collect();
    assert_eq!(settle, ["tracks::settle", "tracks::settle@a+b"]);
    let both = &g.nodes[g.node("tracks::a1+b1").unwrap()];
    assert!(both.is_terminal());
    assert!((both.payoff - 5000.0).abs() < 1e-9);
}

#[test]
fn without_spawn_the_second_track_is_never_walked() {
    let mut one = tracks();
    one.edges[0].spawn.clear();
    let v = value(&compile(&[one], &CompileOptions::default()).unwrap());
    assert!((v - 890.0).abs() < 1e-9, "{v}");
}

/// The motion's ruling joins discovery: it is enabled only once discovery
/// has closed, and then one track remains.
fn joined() -> Pack {
    pack(
        "joined",
        &json!([
            { "id": "s", "kind": "decision", "label": "Start" },
            { "id": "a", "label": "Motion" },
            { "id": "b", "label": "Discovery" },
            { "id": "bc", "label": "Discovery closed" },
            { "id": "r", "kind": "terminal", "label": "Ruled", "payoff": 700 }
        ]),
        &json!([
            { "id": "file", "from": "s", "to": "a", "label": "file", "actor": "party", "cost": 100, "spawn": ["b"] },
            { "id": "close", "from": "b", "to": "bc", "label": "close", "actor": "court", "cost": 40 },
            { "id": "rule", "from": "a", "to": "r", "label": "rule", "actor": "court", "cost": 60, "joins": ["bc"] }
        ]),
    )
}

#[test]
fn a_join_waits_for_its_track_and_consumes_it() {
    let g = compile(&[joined()], &CompileOptions::default()).unwrap();
    let v = value(&g);
    assert!((v - 500.0).abs() < 1e-9, "{v}");
    // Alone at the motion the ruling is not enabled.
    let a = g.node("joined::a").unwrap();
    assert!(g.out[a].is_empty());
    // At {motion, discovery} discovery moves; at {motion, closed} the
    // ruling joins and leaves the one track at the ruling.
    let ab = &g.nodes[g.node("joined::a+b").unwrap()];
    assert_eq!(ab.base_id, "joined::b");
    let rule = g.edge("joined::rule@a+bc").unwrap();
    assert_eq!(g.nodes[g.edges[rule].to].id, "joined::r");
}

#[test]
fn markings_past_the_cap_are_a_clear_error() {
    // Each filing spawns another track at the start: unbounded markings.
    let p = pack(
        "unbounded",
        &json!([
            { "id": "s", "kind": "decision", "label": "Start" },
            { "id": "t", "kind": "terminal", "label": "Done" }
        ]),
        &json!([
            { "id": "again", "from": "s", "to": "s", "label": "again", "actor": "party", "spawn": ["s"] },
            { "id": "stop", "from": "s", "to": "t", "label": "stop", "actor": "party" }
        ]),
    );
    let opts = CompileOptions {
        max_product_nodes: 50,
        ..CompileOptions::default()
    };
    let err = compile(&[p], &opts).unwrap_err().to_string();
    assert!(
        err.contains("concurrent tracks exceed max_product_nodes (50)"),
        "{err}"
    );
}

#[test]
fn a_join_on_a_track_that_ended_elsewhere_is_a_deadlock() {
    // Discovery ends at a terminal of its own, so the ruling never joins.
    let mut p = joined();
    p.nodes.push(
        serde_json::from_value(json!({ "id": "x", "kind": "terminal", "label": "Ended" })).unwrap(),
    );
    p.edges[1].to = "x".into();
    let err = compile(&[p], &CompileOptions::default())
        .unwrap_err()
        .to_string();
    assert!(
        err.contains("concurrent tracks deadlock at marking joined::a+x"),
        "{err}"
    );
}

#[test]
fn a_link_cannot_spawn_a_track() {
    let lf = LinkFile {
        links: vec![serde_json::from_value(json!({
            "from": "tracks::a1", "to": "product::s", "label": "on",
            "spawn": ["b"]
        }))
        .unwrap()],
        ..LinkFile::default()
    };
    let err = Graph::compile(&[tracks(), product()], &lf, &CompileOptions::default())
        .unwrap_err()
        .to_string();
    assert!(err.contains("cannot spawn or join"), "{err}");
}

#[test]
fn a_track_naming_no_node_is_an_error() {
    let mut p = tracks();
    p.edges[0].spawn = vec!["nowhere".into()];
    let err = compile(&[p], &CompileOptions::default())
        .unwrap_err()
        .to_string();
    assert!(
        err.contains("spawn names unknown node tracks::nowhere"),
        "{err}"
    );
}
