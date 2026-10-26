// SPDX-License-Identifier: GPL-3.0-or-later
//! The summary-judgment reply-evidence branch of the embedded FRCP pack
//! (`lit/frcp/02-pleading-to-trial.lit`, lg-y28h; tests lg-jjeu): a
//! movant that submits new evidence with its reply reaches
//! `reply-evidence-filed`, where the nonmovant may seek leave to file a
//! sur-reply, move to strike, or not respond (FRCP 56(c)(2), 56(c)(4));
//! the court's `reply-evidence-ruling` then allows the sur-reply, excludes
//! or disregards the evidence, or denies the request, and every outcome
//! goes on to `sj-ruling` (FRCP 56(a)). The branch sets no deadline: Rule
//! 56 gives no sur-reply period, and N.D. Cal. Civil L.R. 7-3(d), whose
//! time for objecting to reply evidence would set one, is not vendored, so
//! the package does not state it (a hole, not a ruling that none exists).

#![allow(clippy::unwrap_used, clippy::expect_used)]

use litigate_engine::api::{handle, Request};
use serde_json::{json, Value};

const P: &str = "frcp-civil-procedure";

/// `id` qualified by the FRCP pack.
fn q(id: &str) -> String {
    format!("{P}::{id}")
}

/// The successful result of `op` on the embedded FRCP pack.
fn run(op: &Value) -> Value {
    let catalog = litgraph_packs::catalog().expect("embedded packs");
    let req: Request = serde_json::from_value(json!({ "packs": [P], "op": op })).unwrap();
    let resp = handle(&req, &catalog);
    assert!(resp.ok, "{:?}", resp.error);
    resp.result.expect("result")
}

/// `(edge, role, to, authority)` of every edge out of `node`, sorted.
fn out_edges(node: &str) -> Vec<(String, String, String, String)> {
    let graph = run(&json!({ "op": "graph", "node": q(node) }));
    let mut edges: Vec<_> = graph["edges"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["from"] == q(node))
        .map(|e| {
            let s = |k: &str| e[k].as_str().unwrap_or("-").to_owned();
            (s("id"), s("role"), s("to"), s("authority"))
        })
        .collect();
    edges.sort();
    edges
}

/// The edge sequences of the paths from `from` to `to`.
fn paths(from: &str, to: &str) -> Vec<Vec<String>> {
    let result = run(&json!({ "op": "path", "from": q(from), "to": q(to), "k": 50 }));
    result["paths"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| {
            p["edges"]
                .as_array()
                .unwrap()
                .iter()
                .map(|e| e.as_str().unwrap().to_owned())
                .collect()
        })
        .collect()
}

/// Whether some path from `summary-judgment-motion` to `sj-ruling` takes
/// exactly the edges `via` after submitting new reply evidence.
fn reply_evidence_path(via: &[&str]) -> bool {
    let want: Vec<String> = std::iter::once("edge-summary-judgment-motion-reply-evidence-filed")
        .chain(via.iter().copied())
        .map(q)
        .collect();
    paths("summary-judgment-motion", "sj-ruling").contains(&want)
}

#[test]
fn reply_evidence_filed_and_its_ruling_are_reachable_from_the_start_node() {
    for node in ["reply-evidence-filed", "reply-evidence-ruling"] {
        let result = run(&json!({ "op": "path", "to": q(node) }));
        let paths = result["paths"].as_array().unwrap();
        assert!(!paths.is_empty(), "no path from the start node to {node}");
        assert_eq!(paths[0]["end"]["id"], q(node));
    }
}

#[test]
fn summary_judgment_motion_offers_the_movant_new_evidence_on_reply() {
    let edges = out_edges("summary-judgment-motion");
    assert!(
        edges.contains(&(
            q("edge-summary-judgment-motion-reply-evidence-filed"),
            "self".to_owned(),
            q("reply-evidence-filed"),
            "FRCP 56(c)(1); 56(c)(4)".to_owned(),
        )),
        "{edges:?}"
    );
}

#[test]
fn reply_evidence_filed_offers_exactly_sur_reply_strike_or_no_response() {
    let rule56c = "FRCP 56(c)(2); 56(c)(4)".to_owned();
    assert_eq!(
        out_edges("reply-evidence-filed"),
        vec![
            (
                q("edge-reply-evidence-filed-motion-to-strike-filed"),
                "self".to_owned(),
                q("reply-evidence-ruling"),
                rule56c.clone(),
            ),
            (
                q("edge-reply-evidence-filed-no-response"),
                "self".to_owned(),
                q("sj-ruling"),
                "FRCP 56(a); 56(c)".to_owned(),
            ),
            (
                q("edge-reply-evidence-filed-sur-reply-requested"),
                "self".to_owned(),
                q("reply-evidence-ruling"),
                rule56c,
            ),
        ]
    );
}

#[test]
fn reply_evidence_ruling_outcomes_are_sur_reply_allowed_excluded_or_denied_each_into_sj_ruling() {
    let to = |e: &str| (q(e), "nature".to_owned(), q("sj-ruling"), "-".to_owned());
    assert_eq!(
        out_edges("reply-evidence-ruling"),
        vec![
            to("edge-reply-evidence-ruling-evidence-excluded"),
            to("edge-reply-evidence-ruling-request-denied"),
            to("edge-reply-evidence-ruling-sur-reply-allowed"),
        ]
    );
}

#[test]
fn a_sur_reply_request_the_court_grants_leads_to_sj_ruling() {
    assert!(reply_evidence_path(&[
        "edge-reply-evidence-filed-sur-reply-requested",
        "edge-reply-evidence-ruling-sur-reply-allowed",
    ]));
}

#[test]
fn a_motion_to_strike_on_which_the_court_disregards_the_evidence_leads_to_sj_ruling() {
    assert!(reply_evidence_path(&[
        "edge-reply-evidence-filed-motion-to-strike-filed",
        "edge-reply-evidence-ruling-evidence-excluded",
    ]));
}

#[test]
fn a_denied_request_leads_to_sj_ruling_on_the_submitted_record() {
    assert!(reply_evidence_path(&[
        "edge-reply-evidence-filed-sur-reply-requested",
        "edge-reply-evidence-ruling-request-denied",
    ]));
}

#[test]
fn no_response_to_reply_evidence_goes_straight_to_sj_ruling() {
    assert!(reply_evidence_path(&[
        "edge-reply-evidence-filed-no-response"
    ]));
}

#[test]
fn neither_reply_evidence_node_sets_a_deadline() {
    for node in ["reply-evidence-filed", "reply-evidence-ruling"] {
        let due = run(&json!({ "op": "deadlines", "trigger": "2025-03-04", "node": q(node) }));
        assert_eq!(due["count"], 0, "{node}: {due}");
    }
}
