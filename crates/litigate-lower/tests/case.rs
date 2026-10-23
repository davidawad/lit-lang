//! `.case` lowering (SPEC §8.3). These tests build `hir::Case` directly:
//! the SPEC §12 matter as a golden, every named scenario in `scenarios/`
//! read back into case HIR and lowered to a semantically equal scenario,
//! and the edge-removing ones again with their removals as derived
//! `Case::removals`.
mod support;

use litgraph::scenario::NamedScenario;
use litigate_base::DiagnosticCode;
use litigate_lower::{render_scenario, semantic_diff_scenarios, Lowerer};
use litigate_sema::hir::{Analysis, Case, Observation, Perspective, Removal, Stake};
use serde_json::{json, Map, Value};
use std::collections::BTreeMap;
use support::{acme, load_hir, pin, render_map, repo_root, usd};

#[test]
fn acme_golden() {
    let packages = load_hir(&["lit/examples"]);
    let refs: Vec<_> = packages.iter().collect();
    let lowered = Lowerer::new(&refs)
        .case(&acme(), "cases/examples/acme.case")
        .unwrap_or_else(|d| panic!("{d:#?}"));
    let text = render_scenario(&lowered.scenario).unwrap_or_default();
    insta::assert_snapshot!("acme_scenario", text);
    let reread = NamedScenario::from_json(&text).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(reread, lowered.scenario);
    insta::assert_snapshot!("acme_map", render_map(&lowered.source_map));
    // Widget is the Defendant: `perspective widget` makes Defendant `self`
    // and Plaintiff `opponent`, both differing from the package roles.
    let perspective = &lowered.scenario.scenario.perspective;
    assert_eq!(perspective.len(), 2);
}

fn codes(case: &Case) -> Vec<DiagnosticCode> {
    Lowerer::new(&[])
        .case(case, "cases/x.case")
        .err()
        .unwrap_or_default()
        .iter()
        .map(|d| d.code)
        .collect()
}

#[test]
fn analysis_is_judged_by_litgraph() {
    let mut case = acme();
    case.perspective = None;
    case.analysis.fields.insert("start".into(), json!("x"));
    assert_eq!(codes(&case), [DiagnosticCode::InvalidAnalysis]);
    let mut case = acme();
    case.perspective = None;
    case.analysis.fields.insert("strat".into(), json!("x"));
    assert_eq!(codes(&case), [DiagnosticCode::InvalidAnalysis]);
    let mut case = acme();
    case.perspective = None;
    case.analysis.fields.insert("mask".into(), json!("hours >"));
    assert_eq!(codes(&case), [DiagnosticCode::InvalidEngineExpression]);
    // `perspective <party>` needs the primary package's roles.
    assert_eq!(codes(&acme()), [DiagnosticCode::LoweringUnsupported]);
}

/// A legacy named scenario read back into case HIR (SPEC §8.7 import
/// conventions: explicit start, actor perspective, everything else under
/// `analysis`).
fn case_of(named: &Value) -> Case {
    let mut scenario: Map<String, Value> =
        named["scenario"].as_object().cloned().unwrap_or_default();
    let take = |s: &mut Map<String, Value>, k: &str| s.remove(k).unwrap_or(Value::Null);
    let start = take(&mut scenario, "start").as_str().map(str::to_owned);
    let perspective: BTreeMap<String, litigate_sema::hir::ActorRole> =
        serde_json::from_value(take(&mut scenario, "perspective")).unwrap_or_default();
    let facts: BTreeMap<String, String> =
        serde_json::from_value(take(&mut scenario, "facts")).unwrap_or_default();
    let payoffs: BTreeMap<String, f64> =
        serde_json::from_value(take(&mut scenario, "payoffs")).unwrap_or_default();
    let observe: BTreeMap<String, BTreeMap<String, u64>> =
        serde_json::from_value(take(&mut scenario, "observe")).unwrap_or_default();
    #[allow(clippy::cast_possible_truncation)]
    let stakes = payoffs
        .into_iter()
        .map(|(k, v)| {
            let minor = (v * 100.0).round();
            assert!((minor / 100.0 - v).abs() < f64::EPSILON, "{k}: {v}");
            (k, Stake::Value(usd(minor as i64)))
        })
        .collect();
    let text = |k: &str| named[k].as_str().map(str::to_owned);
    Case {
        id: text("id").unwrap_or_default(),
        summary: text("summary").unwrap_or_default(),
        notes: text("notes").filter(|n| !n.is_empty()),
        sources: serde_json::from_value(named["sources"].clone()).unwrap_or_default(),
        uses: named["packs"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .map(pin)
            .collect(),
        as_of: "2026-01-01".into(),
        known_at: None,
        start,
        perspective: (!perspective.is_empty()).then_some(Perspective::Actors(perspective)),
        parties: Vec::new(),
        events: Vec::new(),
        facts,
        stakes,
        observations: observe
            .into_iter()
            .map(|(node, counts)| Observation {
                node,
                outcomes: counts.into_iter().collect(),
            })
            .collect(),
        analysis: Analysis {
            fields: scenario.into_iter().collect(),
        },
        removals: Vec::new(),
    }
}

/// `scenarios/*.json`, sorted.
fn scenario_paths() -> Vec<std::path::PathBuf> {
    let dir = repo_root().join("scenarios");
    let mut paths: Vec<_> = std::fs::read_dir(&dir)
        .map(|r| r.filter_map(Result::ok).map(|e| e.path()).collect())
        .unwrap_or_default();
    paths.sort();
    assert!(paths.len() >= 5, "{paths:?}");
    paths
}

#[test]
fn every_named_scenario_round_trips() {
    for path in scenario_paths() {
        let text = std::fs::read_to_string(&path).unwrap_or_default();
        let original = NamedScenario::from_json(&text).unwrap_or_else(|e| panic!("{e}"));
        let value: Value = serde_json::from_str(&text).unwrap_or_default();
        let lowered = Lowerer::new(&[])
            .case(&case_of(&value), "cases/x.case")
            .unwrap_or_else(|d| panic!("{}: {d:#?}", path.display()));
        let changes = semantic_diff_scenarios(&original, &lowered.scenario);
        assert!(changes.is_empty(), "{}: {changes:#?}", path.display());
    }
}

/// `case_of`, with `remove_edges` read as removals derived from cited
/// events (one per edge) instead of an `analysis` field.
fn case_with_removals(named: &Value) -> Case {
    let mut case = case_of(named);
    let edges = case
        .analysis
        .fields
        .remove("remove_edges")
        .and_then(|v| serde_json::from_value::<Vec<String>>(v).ok())
        .unwrap_or_default();
    case.removals = edges
        .into_iter()
        .enumerate()
        .map(|(event, edge)| Removal {
            edge,
            date: "2026-01-01".into(),
            event,
            authority: "FRCP 12(f)".into(),
        })
        .collect();
    case
}

/// Every named scenario that removes edges round-trips with those edges
/// carried by `Case::removals` (SPEC §8.3), and each is mapped to its cite.
#[test]
fn masked_scenarios_round_trip_through_removals() {
    let mut masked = Vec::new();
    for path in scenario_paths() {
        let text = std::fs::read_to_string(&path).unwrap_or_default();
        let original = NamedScenario::from_json(&text).unwrap_or_else(|e| panic!("{e}"));
        if original.scenario.remove_edges.is_empty() {
            continue;
        }
        let value: Value = serde_json::from_str(&text).unwrap_or_default();
        let case = case_with_removals(&value);
        assert!(!case.analysis.fields.contains_key("remove_edges"));
        let lowered = Lowerer::new(&[])
            .case(&case, "cases/x.case")
            .unwrap_or_else(|d| panic!("{}: {d:#?}", path.display()));
        let changes = semantic_diff_scenarios(&original, &lowered.scenario);
        assert!(changes.is_empty(), "{}: {changes:#?}", path.display());
        let mapped: Vec<(&str, &str)> = lowered
            .source_map
            .entries
            .iter()
            .filter_map(|e| Some((e.pointer.as_str(), e.cite.as_ref()?.text.as_str())))
            .collect();
        assert_eq!(mapped.len(), original.scenario.remove_edges.len());
        masked.push(original.id);
    }
    assert_eq!(masked, ["cofc-1498-patent-case", "cofc-bid-protest"]);
}

/// Derived removals follow any `analysis.remove_edges`, each edge once; a
/// non-list `analysis.remove_edges` is still judged by litgraph.
#[test]
fn removals_merge_with_analysis() {
    let mut case = acme();
    case.perspective = None;
    case.analysis.fields.insert(
        "remove_edges".into(),
        json!(["frcp-rule12-example::answer"]),
    );
    case.removals = [
        "frcp-rule12-example::answer",
        "frcp-rule12-example::default",
    ]
    .iter()
    .enumerate()
    .map(|(event, edge)| Removal {
        edge: (*edge).into(),
        date: "2025-03-10".into(),
        event,
        authority: "FRCP 12(h)(1)".into(),
    })
    .collect();
    let lowered = Lowerer::new(&[])
        .case(&case, "cases/x.case")
        .unwrap_or_else(|d| panic!("{d:#?}"));
    insta::assert_snapshot!(
        "removals_scenario",
        render_scenario(&lowered.scenario).unwrap_or_default()
    );
    let pointers: Vec<&str> = lowered
        .source_map
        .entries
        .iter()
        .map(|e| e.pointer.as_str())
        .collect();
    assert_eq!(
        pointers,
        ["", "/scenario/remove_edges/0", "/scenario/remove_edges/1"]
    );
    case.analysis
        .fields
        .insert("remove_edges".into(), json!("frcp-rule12-example::answer"));
    assert_eq!(codes(&case), [DiagnosticCode::InvalidAnalysis]);
}

/// The engine resolves derived removals: compiled with the lowered pack,
/// the scenario deactivates exactly the removed edges.
#[test]
fn removals_deactivate_engine_edges() {
    let packages = load_hir(&["lit/examples"]);
    let refs: Vec<_> = packages.iter().collect();
    let lowerer = Lowerer::new(&refs);
    let pack = match lowerer
        .package(&packages[0], &litigate_lower::Options::default())
        .unwrap_or_else(|d| panic!("{d:#?}"))
        .document
    {
        litigate_lower::Document::Pack(p) => *p,
        litigate_lower::Document::Links(_) => panic!("a rule package"),
    };
    let graph = litgraph::Graph::compile(
        &[pack],
        &litgraph::model::LinkFile::default(),
        &litgraph::model::CompileOptions::default(),
    )
    .unwrap_or_else(|e| panic!("{e}"));
    let mut case = acme();
    case.removals = vec![Removal {
        edge: "frcp-rule12-example::move-dismiss".into(),
        date: "2025-03-10".into(),
        event: 0,
        authority: "FRCP 12(h)(1)".into(),
    }];
    let lowered = lowerer
        .case(&case, "cases/examples/acme.case")
        .unwrap_or_else(|d| panic!("{d:#?}"));
    let view = litgraph::scenario::View::new(&graph, &lowered.scenario.scenario)
        .unwrap_or_else(|e| panic!("{e}"));
    let removed = graph
        .edge("frcp-rule12-example::move-dismiss")
        .unwrap_or_else(|e| panic!("{e}"));
    let inactive: Vec<usize> = (0..graph.edges.len())
        .filter(|&e| !view.active[e])
        .collect();
    assert_eq!(inactive, [removed]);
}

proptest::proptest! {
    /// `remove_edges` is the authored list followed by the derived edges
    /// not already in it: every edge once, authored order kept.
    #[test]
    fn remove_edges_is_an_ordered_union(
        authored in proptest::collection::vec(0u8..6, 0..6),
        derived in proptest::collection::vec(0u8..6, 1..6),
    ) {
        let edge = |n: &u8| format!("frcp-rule12-example::e{n}");
        let mut case = acme();
        case.perspective = None;
        if !authored.is_empty() {
            let list: Vec<String> = authored.iter().map(edge).collect();
            case.analysis.fields.insert("remove_edges".into(), json!(list));
        }
        case.removals = derived
            .iter()
            .enumerate()
            .map(|(event, n)| Removal {
                edge: edge(n),
                date: "2025-03-10".into(),
                event,
                authority: "FRCP 12(f)".into(),
            })
            .collect();
        let lowered = Lowerer::new(&[])
            .case(&case, "cases/x.case")
            .unwrap_or_else(|d| panic!("{d:#?}"));
        let mut expected: Vec<String> = Vec::new();
        for e in authored.iter().map(edge) {
            expected.push(e);
        }
        for e in derived.iter().map(edge) {
            if !expected.contains(&e) {
                expected.push(e);
            }
        }
        proptest::prop_assert_eq!(&lowered.scenario.scenario.remove_edges, &expected);
    }
}
