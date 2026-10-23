//! The `cases/` ports of `scenarios/*.json` and the scenario-bearing
//! `examples/*.json` (DESIGN §11.2). Each named scenario's matter lowers to
//! JSON semantically equal to the legacy file, with its posture replayed
//! from events rather than asserted; and `litigate run` on each matter gives
//! litgraph's own answer on the legacy input.
// Test helpers below may unwrap: a panic is the failed assertion.
#![allow(clippy::unwrap_used)]
mod support;

use litgraph::api::{handle, Catalog, Request};
use serde_json::{json, Value};
use support::{litigate, repo};

/// The five named scenarios, each with the start its events replay to
/// (`None`: the matter stays at the primary package's start).
const SCENARIOS: [(&str, Option<&str>); 5] = [
    ("cofc-1498-patent-case", None),
    ("cofc-bid-protest", None),
    ("frcp-civil-case", None),
    (
        "ptab-ipr-defense",
        Some("ptab-patent-trial-appeal-board::popr-window"),
    ),
    (
        "ttab-opposition-applicant-defense",
        Some("ttab-trademark-trial-and-appeal-board::period-ends-day120"),
    ),
];

/// Examples whose matter resolves the Court of Federal Claims limitations and § 1500 fact
/// nodes through `facts` where the example removed the barred edges: the
/// answer is the same, and litgraph's `fact-unset` warning is gone.
const FACTS_FIXED: [&str; 9] = [
    "cofc-1498-chain",
    "cofc-1498-risk-averse-sim",
    "cofc-1498-tornado",
    "cofc-1500-trap-compare",
    "cofc-appeal-threshold-sweep",
    "cofc-cvar-tail-averse",
    "cofc-explain-dispositive-fork",
    "cofc-general-sum-settlement",
    "cofc-prejudgment-interest-sim",
];

fn read(path: &str) -> String {
    std::fs::read_to_string(repo().join(path)).unwrap()
}

/// A litgraph response with its wall-clock and byte-level provenance
/// dropped at every depth (generated pack JSON is semantically, not
/// byte-for-byte, equal to the embedded packs).
fn comparable(mut v: Value) -> Value {
    fn strip(v: &mut Value) {
        match v {
            Value::Object(map) => {
                map.remove("elapsed_ms");
                map.remove("provenance");
                map.values_mut().for_each(strip);
            }
            Value::Array(items) => items.iter_mut().for_each(strip),
            _ => {}
        }
    }
    strip(&mut v);
    v
}

/// litgraph's response to `request` over the embedded packs and scenarios.
fn engine(request: &Value) -> Value {
    let catalog = Catalog::embedded().unwrap();
    let request: Request = serde_json::from_value(request.clone()).unwrap();
    // Through the same JSON text the `litigate` binary prints, so both
    // sides parse numbers the same way.
    let text = serde_json::to_string(&handle(&request, &catalog)).unwrap();
    comparable(serde_json::from_str(&text).unwrap())
}

/// `litigate run CASE --op OP`'s litgraph response.
fn run(case: &str, op: &Value, links: bool) -> Value {
    let op = op.to_string();
    let mut args = vec!["run", case, "--op", op.as_str()];
    if !links {
        args.push("--no-links");
    }
    let out = litigate(&args);
    assert_eq!(out.code, 0, "{case}: {}", out.stdout);
    comparable(out.json["result"].clone())
}

/// `response` without `fact-unset` warnings at any depth (a `compare`
/// nests a response per side).
fn without_fact_unset(mut response: Value) -> Value {
    fn strip(v: &mut Value) {
        match v {
            Value::Object(map) => {
                if let Some(Value::Array(w)) = map.get_mut("warnings") {
                    w.retain(|w| w["code"] != "fact-unset");
                }
                map.values_mut().for_each(strip);
            }
            Value::Array(items) => items.iter_mut().for_each(strip),
            _ => {}
        }
    }
    strip(&mut response);
    response
}

/// Whether a `fact-unset` warning appears at any depth.
fn has_fact_unset(v: &Value) -> bool {
    match v {
        Value::Object(map) => {
            map.get("code") == Some(&json!("fact-unset")) || map.values().any(has_fact_unset)
        }
        Value::Array(items) => items.iter().any(has_fact_unset),
        _ => false,
    }
}

#[test]
fn every_named_scenario_lowers_equal_to_its_legacy_json() {
    for (id, _) in SCENARIOS {
        let case = format!("cases/{id}.case");
        let against = format!("scenarios/{id}.json");
        let out = litigate(&["diff", &case, "--against", &against, "--check"]);
        assert_eq!(out.code, 0, "{id}: {}", out.stdout);
        assert_eq!(out.json["result"]["equal"], true, "{id}");
    }
}

#[test]
fn postures_are_replayed_and_removals_derived() {
    for (id, start) in SCENARIOS {
        let case = format!("cases/{id}.case");
        let text = read(&case);
        assert!(!text.contains("\nstart "), "{id} asserts its start");
        assert!(!text.contains("remove_edges:"), "{id} asserts removals");
        let out = litigate(&["check", &case]);
        assert_eq!(out.code, 0, "{id}: {}", out.stdout);
        let summary = &out.json["result"]["cases"][0];
        assert_eq!(summary["start"].as_str(), start, "{id}");
        if start.is_some() {
            let steps = summary["replay"].as_array().unwrap();
            assert!(steps.iter().any(|s| s["effect"] == "took"), "{id}");
        }
    }
}

#[test]
fn matters_have_no_strict_findings_of_their_own() {
    // Strict fails on the used packages' own holes (lit/<forum>); nothing
    // in the matters themselves is a hole, unvalued, or unresolved.
    let out = litigate(&["check", "--profile", "strict", "cases"]);
    let own: Vec<&Value> = out.json["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|d| d["span"]["file"].as_str().unwrap().starts_with("cases/"))
        .collect();
    assert!(own.is_empty(), "{own:#?}");
}

#[test]
fn run_matches_litgraph_on_every_named_scenario() {
    for (id, _) in SCENARIOS {
        for op in [json!({"op": "solve"}), json!({"op": "chain"})] {
            let ours = run(&format!("cases/{id}.case"), &op, true);
            let theirs = engine(&json!({"scenario": id, "op": op}));
            assert_eq!(theirs["ok"], true, "{id}");
            assert_eq!(ours, theirs, "{id} {op}");
        }
    }
}

/// `(example, matter)` for every example that carries a scenario.
fn examples() -> Vec<(String, String)> {
    let mut out = Vec::new();
    for entry in std::fs::read_dir(repo().join("examples")).unwrap() {
        let path = entry.unwrap().path();
        let stem = path.file_stem().unwrap().to_string_lossy().into_owned();
        let request: Value = serde_json::from_str(&read(&format!("examples/{stem}.json"))).unwrap();
        let matter = match &request["scenario"] {
            Value::Null => continue,
            // A bare named scenario is that scenario's own matter.
            Value::String(id) => format!("cases/{id}.case"),
            _ => format!("cases/requests/{stem}.case"),
        };
        out.push((stem, matter));
    }
    out.sort();
    out
}

#[test]
fn every_scenario_bearing_example_has_a_matter() {
    let found = examples();
    assert_eq!(found.len(), 17);
    for (stem, matter) in &found {
        assert!(repo().join(matter).is_file(), "{stem}: no {matter}");
    }
}

#[test]
fn run_reproduces_every_scenario_bearing_example() {
    for (stem, matter) in examples() {
        let request: Value = serde_json::from_str(&read(&format!("examples/{stem}.json"))).unwrap();
        let links = request["links"].as_bool().unwrap_or(true);
        let ours = run(&matter, &request["op"], links);
        let mut theirs = engine(&request);
        assert_eq!(theirs["ok"], true, "{stem}");
        if FACTS_FIXED.contains(&stem.as_str()) {
            assert!(has_fact_unset(&theirs), "{stem}: no fact-unset to fix");
            assert!(!has_fact_unset(&ours), "{stem}: facts left unset");
            theirs = without_fact_unset(theirs);
        }
        assert_eq!(ours, theirs, "{stem}");
    }
}
