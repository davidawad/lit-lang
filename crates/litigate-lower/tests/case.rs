//! `.case` lowering (SPEC §8.3). Sema does not yet produce case HIR (bead
//! "sema case"), so these tests build `hir::Case` directly: the SPEC §12
//! matter as a golden, and every named scenario in `scenarios/` read back
//! into case HIR and lowered to a semantically equal scenario.
mod support;

use litgraph::scenario::NamedScenario;
use litigate_base::DiagnosticCode;
use litigate_lower::{render, render_scenario, semantic_diff_scenarios, Lowerer};
use litigate_sema::hir::{
    Analysis, Case, CaseSource, Money, Observation, PackagePin, Party, Perspective, Stake,
};
use serde_json::{json, Map, Value};
use std::collections::BTreeMap;
use support::{load_hir, repo_root};

fn usd(minor_units: i64) -> Money {
    Money {
        minor_units,
        currency: "usd".into(),
    }
}

fn pin(name: &str) -> PackagePin {
    PackagePin {
        name: name.into(),
        pack_id: name.into(),
        version: "0.1.0".into(),
        content_hash: String::new(),
    }
}

/// `cases/examples/acme.case` as sema will check it: replay (SPEC §12)
/// lands on `mtd-ruling`. Events are not lowered, so none are listed.
fn acme() -> Case {
    let q = |id: &str| format!("frcp-rule12-example::{id}");
    Case {
        id: "acme-v-widget".into(),
        summary: "Acme sues Widget; Widget decides whether to answer or move to dismiss.".into(),
        notes: Some("Worked example for SPEC §12. Stakes are illustrative.".into()),
        sources: vec![CaseSource {
            title: "Federal Rules of Civil Procedure".into(),
            url: Some("https://www.law.cornell.edu/rules/frcp".into()),
            as_of: Some("2024-12-01".into()),
        }],
        uses: vec![pin("frcp-rule12-example")],
        as_of: "2025-06-01".into(),
        known_at: None,
        start: Some(q("mtd-ruling")),
        perspective: Some(Perspective::Party("widget".into())),
        parties: vec![
            Party {
                id: "acme".into(),
                actor: "Plaintiff".into(),
                entity_type: "Corporation".into(),
            },
            Party {
                id: "widget".into(),
                actor: "Defendant".into(),
                entity_type: "Corporation".into(),
            },
        ],
        events: Vec::new(),
        facts: BTreeMap::from([(q("time-barred"), q("not-barred"))]),
        stakes: BTreeMap::from([
            (q("judgment-entered"), Stake::Value(usd(-200_000_000))),
            (q("dismissed"), Stake::Value(usd(0))),
        ]),
        observations: vec![Observation {
            node: q("mtd-ruling"),
            outcomes: vec![(q("granted"), 3), (q("denied"), 1)],
        }],
        analysis: Analysis {
            fields: BTreeMap::from([("params".into(), json!({"rate": 650, "opp_rate": 550}))]),
        },
    }
}

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
    insta::assert_snapshot!("acme_map", render(&lowered.source_map).unwrap_or_default());
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
    }
}

#[test]
fn every_named_scenario_round_trips() {
    let dir = repo_root().join("scenarios");
    let mut paths: Vec<_> = std::fs::read_dir(&dir)
        .map(|r| r.filter_map(Result::ok).map(|e| e.path()).collect())
        .unwrap_or_default();
    paths.sort();
    assert!(paths.len() >= 5, "{paths:?}");
    for path in paths {
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
