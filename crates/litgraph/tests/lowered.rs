// SPDX-License-Identifier: GPL-3.0-or-later
//! The `lowered` header (pack v4, links and scenario v2, clock v4): where
//! it is allowed, that its world digest and holes are checked on read, and
//! that a pack's numeric holes reach every response as `authored-hole`.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use litgraph::api::{handle, validate, Catalog, Request};
use litgraph::clock::ClockLibrary;
use litgraph::model::{world_digest, Hole, LinkFile, Lowered, Pin};
use litgraph::scenario::NamedScenario;
use litgraph::Pack;
use serde_json::{json, Value};

fn header(holes: &[(&str, Option<&str>)]) -> Value {
    let lowered = Lowered::new(
        vec![Pin {
            name: "demo".into(),
            version: "1.0.0".into(),
            digest: "sha256:00".into(),
        }],
        "2025-01-01".into(),
        Some("2024-06-01".into()),
        holes
            .iter()
            .map(|(p, r)| Hole {
                pointer: (*p).into(),
                reason: r.map(str::to_owned),
            })
            .collect(),
    );
    serde_json::to_value(lowered).unwrap()
}

fn pack(version: u32, lowered: Option<Value>) -> Value {
    let mut p = json!({
        "schemaVersion": version, "id": "demo", "title": "Demo", "startNodeId": "start",
        "nodes": [
            {"id": "start", "label": "Start"},
            {"id": "win", "label": "Win", "kind": "terminal", "payoff": 10.0},
            {"id": "lose", "label": "Lose", "kind": "terminal", "payoff": 0.0}
        ],
        "edges": [
            {"from": "start", "to": "win", "label": "win", "actor": "examiner"},
            {"from": "start", "to": "lose", "label": "lose", "actor": "examiner", "probability": 0.5}
        ]
    });
    if let Some(l) = lowered {
        p["lowered"] = l;
    }
    p
}

#[test]
fn a_pack_header_needs_v4_and_a_matching_world() {
    let ok = pack(
        4,
        Some(header(&[("/edges/0/probability", Some("no rate"))])),
    );
    let read = Pack::from_json(&ok.to_string()).unwrap();
    let lowered = read.lowered.unwrap();
    assert_eq!(
        lowered.world_digest,
        world_digest(&lowered.pins, "2025-01-01", Some("2024-06-01"))
    );
    let early = pack(3, Some(header(&[])));
    let err = Pack::from_json(&early.to_string()).unwrap_err().to_string();
    assert!(err.contains("`lowered` needs schemaVersion 4"), "{err}");
    let mut stale = ok.clone();
    stale["lowered"]["asOf"] = json!("2026-01-01");
    let err = Pack::from_json(&stale.to_string()).unwrap_err().to_string();
    assert!(err.contains("pack demo: lowered.worldDigest"), "{err}");
    let stray = pack(4, Some(header(&[("/edges/9/probability", None)])));
    assert!(Pack::from_json(&stray.to_string()).is_err());
    assert!(Pack::from_json(&pack(4, None).to_string()).is_ok());
}

#[test]
fn numeric_holes_warn_and_cite_holes_do_not() {
    let numeric = pack(
        4,
        Some(header(&[("/edges/0/probability", Some("no rate"))])),
    );
    let cite = pack(4, Some(header(&[("/nodes/0/cite", Some("FRCP 99"))])));
    for (doc, warned) in [(numeric, true), (cite, false)] {
        let catalog =
            Catalog::from_files("t".into(), [("demo.json".into(), doc.to_string())]).unwrap();
        let req: Request =
            serde_json::from_value(json!({"packs": ["demo"], "op": {"op": "chain"}})).unwrap();
        let resp = serde_json::to_value(handle(&req, &catalog)).unwrap();
        let codes: Vec<&str> = resp["warnings"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|w| w["code"].as_str())
            .collect();
        assert_eq!(codes.contains(&"authored-hole"), warned, "{resp}");
        if warned {
            let w = &resp["warnings"][codes.iter().position(|c| *c == "authored-hole").unwrap()];
            let text = w["example"].as_str().unwrap();
            assert!(text.contains("/edges/0/probability (no rate)"), "{text}");
        }
    }
}

#[test]
fn link_files_read_v1_and_v2() {
    let v1 = LinkFile::from_json(r#"{"links": []}"#).unwrap();
    assert_eq!(v1.schema_version, None);
    let v2 = json!({"schemaVersion": 2, "links": [], "lowered": header(&[])});
    assert!(LinkFile::from_json(&v2.to_string())
        .unwrap()
        .lowered
        .is_some());
    let early = json!({"links": [], "lowered": header(&[])});
    assert!(LinkFile::from_json(&early.to_string())
        .unwrap_err()
        .to_string()
        .contains("links.json: `lowered` needs schemaVersion 2"));
    let future = json!({"schemaVersion": 3, "links": []});
    assert!(LinkFile::from_json(&future.to_string())
        .unwrap_err()
        .to_string()
        .contains("unsupported schemaVersion 3"));
    assert!(LinkFile::from_json("{").is_err());
    let catalog = Catalog::embedded().unwrap();
    let checked = validate(&v2, None, &catalog);
    assert_eq!((checked.kind, checked.valid), ("links", true));
    assert!(!validate(&future, None, &catalog).valid);
}

#[test]
fn a_broken_links_file_is_a_load_error() {
    let catalog = Catalog::from_files(
        "t".into(),
        [
            ("demo.json".into(), pack(4, None).to_string()),
            ("links.json".into(), r#"{"schemaVersion": 9}"#.into()),
        ],
    )
    .unwrap();
    assert!(catalog
        .load_errors
        .iter()
        .any(|e| e.file == "links.json" && e.message.contains("schemaVersion 9")));
}

fn scenario(extra: &Value) -> Value {
    let mut s = json!({"id": "m", "summary": "A matter", "packs": ["demo"], "scenario": {}});
    for (k, v) in extra.as_object().unwrap() {
        s[k] = v.clone();
    }
    s
}

#[test]
fn named_scenarios_read_v1_and_v2() {
    let event = json!([{"date": "2025-01-02", "action": "Served(complaint)", "relation": "on", "party": "d"}]);
    let party = json!([{"id": "d", "actor": "opponent", "entity_type": "Corporation"}]);
    let v2 = scenario(
        &json!({"schema_version": 2, "events": event, "parties": party,
        "lowered": header(&[("/scenario/payoffs/demo::win", None)])}),
    );
    let read = NamedScenario::from_json(&v2.to_string()).unwrap();
    assert_eq!(read.events[0].action, "Served(complaint)");
    assert_eq!(read.parties[0].entity_type, "Corporation");
    for (bad, why) in [
        (scenario(&json!({"events": event})), "need schema_version 2"),
        (
            scenario(&json!({"parties": party})),
            "need schema_version 2",
        ),
        (
            scenario(&json!({"schema_version": 3})),
            "unsupported schema_version 3",
        ),
        (
            scenario(&json!({"lowered": header(&[])})),
            "`lowered` needs schemaVersion 2",
        ),
        (
            scenario(&json!({"summary": " "})),
            "`summary` must not be empty",
        ),
    ] {
        let err = NamedScenario::from_json(&bad.to_string())
            .unwrap_err()
            .to_string();
        assert!(err.contains(why), "{err}");
    }
    let catalog = Catalog::embedded().unwrap();
    let bad = scenario(&json!({"schema_version": 3}));
    let checked = validate(&bad, None, &catalog);
    assert_eq!((checked.kind, checked.valid), ("named-scenario", false));
}

#[test]
fn clock_fragments_read_v3_and_v4() {
    let v4 = json!({"schemaVersion": 4, "lowered": header(&[])});
    assert!(ClockLibrary::from_json(&v4.to_string()).is_ok());
    let early = json!({"schemaVersion": 3, "lowered": header(&[])});
    assert!(ClockLibrary::from_json(&early.to_string())
        .unwrap_err()
        .to_string()
        .contains("clock: `lowered` needs schemaVersion 4"));
    assert!(ClockLibrary::from_json(r#"{"schemaVersion": 5}"#).is_err());
}

#[test]
fn validating_a_pack_checks_its_header() {
    let catalog = Catalog::embedded().unwrap();
    let mut stale = pack(4, Some(header(&[])));
    stale["lowered"]["worldDigest"] = json!("sha256:00");
    let checked = validate(&stale, None, &catalog);
    assert_eq!(checked.kind, "pack");
    assert!(
        checked.errors.iter().any(|e| e.contains("worldDigest")),
        "{:?}",
        checked.errors
    );
}
