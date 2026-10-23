// SPDX-License-Identifier: GPL-3.0-or-later
//! The engine visits nodes in canonical id order (lg-flu.5, SPEC §4.1): a
//! pack's node array order, the order its nodes' edge runs appear in, the
//! links file's link order and the order a request lists packs in are all
//! unobservable. Only a node's own edge order (one declaration) is kept.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use litgraph::api::{handle_json, Catalog};
use serde_json::{json, Value};

const ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");
const COFC: &str = "cofc";
const CAFC: &str = "cafc";

fn read(name: &str) -> Value {
    serde_json::from_str(&std::fs::read_to_string(format!("{ROOT}/packs/{name}.json")).unwrap())
        .unwrap()
}

/// `pack` with its nodes reversed, and its edges regrouped so each node's
/// edges follow the reversed node order while keeping their own order.
fn permuted(mut pack: Value) -> Value {
    let nodes = pack["nodes"].as_array_mut().unwrap();
    nodes.reverse();
    let order: Vec<Value> = nodes.iter().map(|n| n["id"].clone()).collect();
    let edges = pack["edges"].as_array().unwrap().clone();
    let regrouped: Vec<Value> = order
        .iter()
        .flat_map(|id| edges.iter().filter(move |e| e["from"] == *id).cloned())
        .collect();
    assert_eq!(regrouped.len(), edges.len(), "every edge leaves a node");
    pack["edges"] = Value::Array(regrouped);
    pack
}

fn catalog(permute: bool) -> Catalog {
    let mut links = read("links");
    let (mut cofc, mut cafc) = (
        read("cofc-court-of-federal-claims"),
        read("cafc-federal-circuit"),
    );
    if permute {
        links["links"].as_array_mut().unwrap().reverse();
        (cofc, cafc) = (permuted(cofc), permuted(cafc));
    }
    let files = [
        (format!("{COFC}.json"), cofc),
        (format!("{CAFC}.json"), cafc),
        ("links.json".to_owned(), links),
    ];
    Catalog::from_files("t".into(), files.map(|(n, v)| (n, v.to_string()))).unwrap()
}

fn result(catalog: &Catalog, packs: &[&str], op: &Value) -> Value {
    let req = json!({
        "packs": packs,
        "scenario": {"start": format!("{COFC}::complaint-filed")},
        "op": op,
    });
    let r = handle_json(&req.to_string(), catalog);
    assert!(r.ok, "{:?}", r.error);
    r.result.unwrap()
}

#[test]
fn node_link_and_pack_order_leave_every_analysis_bit_identical() {
    let (base, other) = (catalog(false), catalog(true));
    for op in [
        json!({"op": "solve"}),
        json!({"op": "chain"}),
        json!({"op": "simulate", "runs": 200, "seed": 7}),
        json!({"op": "path", "k": 3}),
        json!({"op": "structure"}),
    ] {
        let want = result(&base, &[COFC, CAFC], &op);
        assert_eq!(result(&other, &[COFC, CAFC], &op), want, "{op}");
        assert_eq!(result(&base, &[CAFC, COFC], &op), want, "{op}");
    }
}
