// SPDX-License-Identifier: GPL-3.0-or-later
//! `explain` at a node the state-flag product copied (lg-55om): a base
//! node id resolves to the copy in the replayed context (the start's
//! flags, else the copy the start reaches first, else the base node when
//! the start reaches none), a flagged id is explained as named, and the
//! other copies are summarized under `other_contexts`. A node with no
//! flagged copy keeps the plain `explain` shape. The fixture is a tiny
//! synthetic pack, so every value is traceable by hand.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use litigate_engine::api::{handle, Catalog, Op, Request};
use serde_json::{json, Value};

/// `start` reaches `mid` directly with no flag, or through `late` having
/// set `f` (and maybe `g`); under `f` the win pays -5, so `mid{f}` stops
/// instead. `hold`
/// is act-or-wait: waiting lets the office drift it to the win.
fn catalog() -> Catalog {
    let pack = json!({
        "schemaVersion": 2, "id": "ctx", "title": "ctx", "startNodeId": "start",
        "roles": { "applicant": "self", "office": "nature" },
        "nodes": [
            {"id": "start", "kind": "decision", "label": "Start"},
            {"id": "mid", "kind": "decision", "label": "Mid"},
            {"id": "late", "kind": "decision", "label": "Late"},
            {"id": "hold", "kind": "decision", "label": "Hold"},
            {"id": "win", "kind": "terminal", "label": "Win", "payoff": 10, "payoffByFlag": {"f": -5}},
            {"id": "lose", "kind": "terminal", "label": "Lose", "payoff": 0},
            {"id": "island", "kind": "state", "label": "Island"}
        ],
        "edges": [
            {"id": "plain", "from": "start", "to": "mid", "label": "plain", "actor": "applicant"},
            {"id": "flagged", "from": "start", "to": "late", "label": "flagged", "actor": "applicant", "sets": ["f"]},
            {"id": "direct", "from": "start", "to": "hold", "label": "direct", "actor": "applicant"},
            {"id": "late-on", "from": "late", "to": "mid", "label": "late on", "actor": "applicant"},
            {"id": "late-on-g", "from": "late", "to": "mid", "label": "late on g", "actor": "applicant", "sets": ["g"]},
            {"id": "late-hold", "from": "late", "to": "hold", "label": "late hold", "actor": "applicant"},
            {"id": "go", "from": "mid", "to": "win", "label": "go", "actor": "applicant"},
            {"id": "stop", "from": "mid", "to": "lose", "label": "stop", "actor": "applicant"},
            {"id": "act", "from": "hold", "to": "lose", "label": "act", "actor": "applicant"},
            {"id": "drift", "from": "hold", "to": "win", "label": "drift", "actor": "office", "probability": 1.0},
            {"id": "island-stop", "from": "island", "to": "lose", "label": "island stop", "actor": "office", "probability": 1.0}
        ]
    });
    Catalog::from_files("t".into(), vec![("t.json".into(), pack.to_string())])
        .expect("ctx catalog compiles")
}

fn explain(node: Option<&str>, from: Option<&str>) -> Value {
    let resp = handle(
        &Request {
            packs: vec!["ctx".into()],
            op: Op::Explain {
                node: node.map(Into::into),
                from: from.map(Into::into),
            },
            ..Request::default()
        },
        &catalog(),
    );
    assert!(resp.ok, "{resp:?}");
    resp.result.unwrap()
}

/// `(id, steps, chosen id, same_choice)` of every reachable other context.
fn reachable(x: &Value) -> Vec<(String, u64, Value, bool)> {
    x["other_contexts"]["reachable"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| {
            (
                c["id"].as_str().unwrap().to_owned(),
                c["steps"].as_u64().unwrap(),
                c["chosen"]["id"].clone(),
                c["same_choice"].as_bool().unwrap(),
            )
        })
        .collect()
}

#[test]
fn a_base_id_is_explained_in_the_replayed_context_and_the_rest_summarized() {
    let x = explain(Some("mid"), None);
    assert_eq!(x["node"]["id"], "ctx::mid");
    assert_eq!(
        x["context"],
        json!({"base_id": "ctx::mid", "flags": [], "basis": "replayed", "reachable": true})
    );
    // `mid{g}` hangs off the base `late`, which the start never reaches.
    assert_eq!(x["other_contexts"]["count"], 3);
    assert_eq!(x["other_contexts"]["unreached"], 1);
    assert_eq!(
        reachable(&x),
        [
            (
                "ctx::mid{f,g}".to_owned(),
                2,
                json!("ctx::stop{f,g}"),
                false
            ),
            ("ctx::mid{f}".to_owned(), 2, json!("ctx::stop{f}"), false),
        ]
    );
    // Equally near copies list by id.
    assert_eq!(x["other_contexts"]["reachable"][1]["value"], 0.0);
    assert_eq!(x["other_contexts"]["reachable"][1]["flags"], json!(["f"]));
}

#[test]
fn a_base_id_the_start_reaches_only_flagged_resolves_to_the_nearest_copy() {
    let x = explain(Some("late"), None);
    assert_eq!(x["node"]["id"], "ctx::late{f}");
    assert_eq!(x["context"]["basis"], "reached");
    assert_eq!(x["context"]["flags"], json!(["f"]));
    assert_eq!(x["other_contexts"]["unreached"], 1);
    assert!(reachable(&x).is_empty());
}

#[test]
fn a_base_id_the_start_never_reaches_is_explained_as_itself() {
    let x = explain(Some("mid"), Some("island"));
    assert_eq!(x["node"]["id"], "ctx::mid");
    assert_eq!(x["context"]["basis"], "unreached");
    assert_eq!(x["context"]["reachable"], false);
    assert_eq!(x["other_contexts"]["unreached"], 3);
}

#[test]
fn a_flagged_id_is_explained_as_named() {
    let x = explain(Some("ctx::mid{f}"), None);
    assert_eq!(x["node"]["id"], "ctx::mid{f}");
    assert_eq!(x["context"]["basis"], "named");
    assert_eq!(
        reachable(&x),
        [
            ("ctx::mid".to_owned(), 1, json!("ctx::go"), false),
            ("ctx::mid{f,g}".to_owned(), 2, json!("ctx::stop{f,g}"), true),
        ]
    );
}

#[test]
fn a_flagged_start_is_the_replayed_context() {
    let x = explain(None, Some("ctx::mid{f}"));
    assert_eq!(x["node"]["id"], "ctx::mid{f}");
    assert_eq!(x["context"]["basis"], "replayed");
    assert_eq!(x["other_contexts"]["unreached"], 3);

    let win = explain(Some("win"), Some("ctx::mid{f}"));
    assert_eq!(win["node"]["id"], "ctx::win{f}");
    assert_eq!(win["context"]["basis"], "replayed");
}

#[test]
fn contexts_nobody_chooses_at_carry_no_choice_and_wait_compares_as_wait() {
    let win = explain(Some("win"), None);
    assert_eq!(
        reachable(&win),
        [
            ("ctx::win{f,g}".to_owned(), 3, Value::Null, true),
            ("ctx::win{f}".to_owned(), 3, Value::Null, true),
        ]
    );
    assert_eq!(win["other_contexts"]["reachable"][0]["chosen"], Value::Null);

    let hold = explain(Some("hold"), None);
    assert_eq!(
        reachable(&hold),
        [("ctx::hold{f}".to_owned(), 2, json!("ctx::act{f}"), false)]
    );
    let chosen = hold["options"]
        .as_array()
        .unwrap()
        .iter()
        .find(|o| o["chosen"] == true)
        .unwrap();
    assert_eq!(chosen["id"], "WAIT");
}

#[test]
fn a_node_without_flagged_copies_keeps_the_plain_shape() {
    for x in [explain(None, None), explain(Some("island"), None)] {
        assert!(x.get("context").is_none(), "{x}");
        assert!(x.get("other_contexts").is_none(), "{x}");
    }
}
