//! `semantic_diff_json` and `detect` (SPEC §8.6, `litigate diff`): every
//! document kind from either of its keys, every error of the JSON entry
//! point as a snapshot, and added/removed object keys and array items.
mod support;

use litigate_lower::diff::detect;
use litigate_lower::{render, semantic_diff_clock, semantic_diff_json, DocumentKind};
use serde_json::{json, Value};
use support::{clock_of, example, lowered};

#[test]
fn every_kind_is_detected_from_either_key() {
    let kinds = [
        (json!({"nodes": []}), Some(DocumentKind::Pack)),
        (json!({"startNodeId": "a"}), Some(DocumentKind::Pack)),
        (json!({"countingRules": []}), Some(DocumentKind::Clock)),
        (json!({"calendars": []}), Some(DocumentKind::Clock)),
        (json!({"summary": "s"}), Some(DocumentKind::Scenario)),
        (json!({"scenario": {}}), Some(DocumentKind::Scenario)),
        (json!({"links": []}), Some(DocumentKind::Links)),
        (json!({"instances": {}}), Some(DocumentKind::Links)),
        (json!({"title": "t"}), None),
    ];
    for (value, kind) in kinds {
        assert_eq!(detect(&value), kind, "{value}");
    }
}

/// The error of each malformed pair, one per line.
fn errors(pairs: &[(&str, &str)]) -> String {
    pairs
        .iter()
        .map(|(a, b)| semantic_diff_json(a, b).err().unwrap_or_default() + "\n")
        .collect()
}

#[test]
fn malformed_documents_are_errors() {
    let pack = r#"{"schemaVersion": 2, "id": "p", "title": "P", "startNodeId": "a",
        "nodes": [{"id": "a", "label": "A"}], "edges": []}"#;
    let scenario = r#"{"id": "s", "summary": "S"}"#;
    let links = r#"{"links": []}"#;
    let clock = render(clock_of(&lowered(&example()))).unwrap_or_default();
    insta::assert_snapshot!(errors(&[
        ("{", pack),
        (pack, "nodes"),
        (r#"{"nodes": 5}"#, pack),
        (pack, r#"{"nodes": 5}"#),
        (r#"{"summary": 5}"#, scenario),
        (scenario, r#"{"summary": 5}"#),
        (r#"{"links": 5}"#, links),
        (links, r#"{"links": 5}"#),
        (r#"{"calendars": 5}"#, &clock),
        (&clock, r#"{"calendars": 5}"#),
    ]));
}

#[test]
fn link_files_and_clocks_are_compared() {
    let links = json!({"links": [{"from": "a::x", "to": "b::y", "label": "go"}]}).to_string();
    let more = json!({"links": [
        {"from": "a::x", "to": "b::y", "label": "go"},
        {"from": "a::x", "to": "b::z", "label": "also"}
    ]})
    .to_string();
    let (kind, changes) = semantic_diff_json(&links, &more).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(kind, DocumentKind::Links);
    insta::assert_json_snapshot!("links_added", changes);

    let library = clock_of(&lowered(&example())).clone();
    let text = render(&library).unwrap_or_default();
    let mut retitled = library.clone();
    retitled.calendars[0].title = "Holidays".into();
    let other = render(&retitled).unwrap_or_default();
    let (kind, changes) = semantic_diff_json(&text, &other).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(kind, DocumentKind::Clock);
    assert_eq!(changes, semantic_diff_clock(&library, &retitled));
    insta::assert_json_snapshot!("clock_changed", changes);
}

/// Object keys and plain array items present on one side only are
/// `added` or `removed` at their own pointer.
#[test]
fn keys_and_items_on_one_side_only() {
    let scenario = |params: Value, packs: Value| {
        json!({"id": "s", "summary": "S", "packs": packs, "scenario": {"params": params}})
            .to_string()
    };
    let small = scenario(json!({"rate": 1}), json!(["a"]));
    let large = scenario(json!({"rate": 1, "opp_rate": 2}), json!(["a", "b"]));
    let diff = |a: &str, b: &str| semantic_diff_json(a, b).map(|(_, c)| c).unwrap_or_default();
    insta::assert_json_snapshot!("added", diff(&small, &large));
    insta::assert_json_snapshot!("removed", diff(&large, &small));
}

/// Ids that repeat on either side cannot key an array: it is compared by
/// index instead.
#[test]
fn repeated_ids_compare_by_index() {
    let pack = |second: &str| {
        json!({
            "schemaVersion": 2, "id": "p", "title": "P", "startNodeId": "a",
            "nodes": [{"id": "a", "label": "A"}, {"id": second, "label": "B"}],
            "edges": []
        })
        .to_string()
    };
    let (unique, repeated) = (pack("b"), pack("a"));
    let changes = |a: &str, b: &str| semantic_diff_json(a, b).map(|(_, c)| c).unwrap_or_default();
    let forward = changes(&unique, &repeated);
    let backward = changes(&repeated, &unique);
    let pointers: Vec<&str> = forward.iter().map(|c| c.pointer.as_str()).collect();
    assert_eq!(pointers, ["/nodes/1/id"]);
    assert_eq!(backward.len(), 1);
}
