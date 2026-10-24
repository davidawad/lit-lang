//! `.case` lowering (SPEC §8.3). These tests build `hir::Case` directly:
//! the SPEC §12 matter as a golden, every named scenario in `scenarios/`
//! read back into case HIR and lowered to a semantically equal scenario,
//! and the edge-removing ones again with their removals as derived
//! `Case::removals`.
mod support;

use litgraph::model::world_digest;
use litgraph::scenario::{EventRelation, NamedScenario};
use litigate_base::DiagnosticCode;
use litigate_lower::{render_scenario, semantic_diff_scenarios, Lowerer};
use litigate_sema::hir::{
    Case, CaseSource, EngineExpr, EngineOption, Event, EventParty, Expr, ExprKind, Observation,
    Param, Perspective, Prior, Rate, RateUnit, Rational, Removal, Stake, Type,
};
use serde_json::{json, Map, Value};
use std::collections::BTreeMap;
use support::{acme, analysis_of, load_hir, pin, render_map, usd};

#[test]
fn acme_golden() {
    let packages = load_hir(&["lit/examples"]);
    let refs: Vec<_> = packages.iter().collect();
    // `rate: 650.usd / 1.hours` lowers to the engine's 650 USD per hour.
    let mut case = acme();
    case.analysis.params.insert(
        "rate".into(),
        Param::Rate(Rate {
            minor_units: Rational {
                numerator: 65_000,
                denominator: 1,
            },
            currency: "usd".into(),
            per: RateUnit::Hour,
        }),
    );
    let lowered = Lowerer::new(&refs)
        .case(&case, "cases/examples/acme.case")
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

/// A `prior` lowers to its mean as the outcomes' `probabilities` and its
/// total as the node's `uncertainty.concentration` (SPEC §8.3); one whose
/// pseudo-counts overflow exact arithmetic is L6004.
#[test]
fn priors_lower_to_probabilities_and_concentration() {
    let packages = load_hir(&["lit/examples"]);
    let refs: Vec<_> = packages.iter().collect();
    let q = |id: &str| format!("frcp-rule12-example::{id}");
    let r = |numerator, denominator| Rational {
        numerator,
        denominator,
    };
    let mut case = acme();
    case.priors = vec![Prior {
        node: q("mtd-ruling"),
        pseudo_counts: vec![(q("granted"), r(3, 2)), (q("denied"), r(9, 2))],
    }];
    let lowered = Lowerer::new(&refs)
        .case(&case, "cases/examples/acme.case")
        .unwrap_or_else(|d| panic!("{d:#?}"));
    let scenario = &lowered.scenario.scenario;
    assert_eq!(
        scenario.probabilities,
        BTreeMap::from([(q("granted"), 0.25), (q("denied"), 0.75)])
    );
    assert_eq!(
        scenario.uncertainty.concentration,
        BTreeMap::from([(q("mtd-ruling"), 6.0)])
    );
    case.priors[0].pseudo_counts[0].1 = r(i64::MAX, 1);
    case.perspective = None;
    assert_eq!(codes(&case), [DiagnosticCode::NonFiniteNumber]);
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

/// Engine options are judged by litgraph's own schema, each on its own;
/// engine expressions must parse.
#[test]
fn analysis_is_judged_by_litgraph() {
    let option = |key: &str, value: Value| {
        let mut case = acme();
        case.perspective = None;
        case.analysis
            .options
            .insert(key.into(), EngineOption { value, span: None });
        codes(&case)
    };
    assert_eq!(
        option("objective", json!({"type": "cvar", "alpah": 0.1})),
        [DiagnosticCode::InvalidAnalysis]
    );
    assert_eq!(
        option("mixed", json!("sideways")),
        [DiagnosticCode::InvalidAnalysis]
    );
    assert_eq!(option("mixed", json!("optimistic")), []);
    let mut case = acme();
    case.perspective = None;
    case.analysis.expressions.insert(
        "mask".into(),
        EngineExpr {
            text: "hours >".into(),
            span: None,
        },
    );
    assert_eq!(codes(&case), [DiagnosticCode::InvalidEngineExpression]);
    let mut case = acme();
    case.perspective = None;
    case.analysis.params.insert(
        "x".into(),
        Param::Number(Rational {
            numerator: 1,
            denominator: 0,
        }),
    );
    assert_eq!(codes(&case), [DiagnosticCode::NonFiniteNumber]);
    // A rate lowers in major units; one whose cents-to-dollars division
    // overflows exact arithmetic is L6004 too.
    let mut case = acme();
    case.perspective = None;
    case.analysis.params.insert(
        "rate".into(),
        Param::Rate(Rate {
            minor_units: Rational {
                numerator: 1,
                denominator: u64::MAX,
            },
            currency: "usd".into(),
            per: RateUnit::Hour,
        }),
    );
    assert_eq!(codes(&case), [DiagnosticCode::NonFiniteNumber]);
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
        priors: Vec::new(),
        analysis: analysis_of(scenario),
        removals: Vec::new(),
    }
}

/// `scenarios/*.json`, sorted.
fn scenario_paths() -> Vec<std::path::PathBuf> {
    let dir = std::path::Path::new(litgraph_packs::SCENARIOS_DIR);
    let mut paths: Vec<_> = std::fs::read_dir(dir)
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
    let edges = std::mem::take(&mut case.analysis.remove_edges);
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
        assert!(case.analysis.remove_edges.is_empty());
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

/// Derived removals follow any `analysis.remove_edges`, each edge once.
#[test]
fn removals_merge_with_analysis() {
    let mut case = acme();
    case.perspective = None;
    case.analysis.remove_edges = vec!["frcp-rule12-example::answer".into()];
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
            case.analysis.remove_edges = authored.iter().map(edge).collect();
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

/// A docket event `callee(args)` with `relation` to `party`.
fn event(date: &str, callee: &str, relation: EventParty, party: &str) -> Event {
    Event {
        id: None,
        date: date.into(),
        docket: None,
        action: Expr {
            ty: Type::Unit,
            kind: ExprKind::Call {
                callee: callee.into(),
                args: vec![Expr {
                    ty: Type::Unit,
                    kind: ExprKind::Ref {
                        target: "complaint".into(),
                    },
                    span: None,
                }],
            },
            span: None,
        },
        relation,
        party: party.into(),
        authority: Some("FRCP 4(c)".into()),
    }
}

/// SPEC §8.3: a v2 scenario records the world it was built in (pins,
/// dates, world digest), the docket, and every hole the matter left.
#[test]
fn a_matter_records_its_world_docket_and_holes() {
    let packages = load_hir(&["lit/examples"]);
    let refs: Vec<_> = packages.iter().collect();
    let mut case = acme();
    case.known_at = Some("2025-01-01".into());
    // Out of source order: the docket is recorded in docket order.
    case.events = vec![
        Event {
            docket: Some(2),
            ..event("2025-01-20", "Answered", EventParty::By, "widget")
        },
        Event {
            docket: Some(1),
            ..event("2025-01-02", "Served", EventParty::On, "widget")
        },
    ];
    case.stakes
        .insert("frcp-rule12-example::settled".into(), Stake::Unvalued);
    case.sources.push(CaseSource {
        title: "Local practice".into(),
        url: None,
        as_of: Some("2026-06".into()),
        as_of_hole: true,
    });
    case.sources.push(CaseSource {
        title: "Undated".into(),
        url: None,
        as_of: Some(String::new()),
        as_of_hole: true,
    });
    let named = Lowerer::new(&refs)
        .case(&case, "cases/examples/acme.case")
        .unwrap_or_else(|d| panic!("{d:#?}"))
        .scenario;
    assert_eq!(named.schema_version, Some(2));
    let lowered = named.lowered.clone().unwrap_or_else(|| panic!("no header"));
    let pins: Vec<&str> = lowered.pins.iter().map(|p| p.name.as_str()).collect();
    assert_eq!(pins, ["frcp-rule12-example"]);
    assert_eq!(
        (lowered.as_of.as_str(), lowered.known_at.as_deref()),
        ("2025-06-01", Some("2025-01-01"))
    );
    assert_eq!(
        lowered.world_digest,
        world_digest(&lowered.pins, "2025-06-01", Some("2025-01-01"))
    );
    let holes: Vec<(&str, Option<&str>)> = lowered
        .holes
        .iter()
        .map(|h| (h.pointer.as_str(), h.reason.as_deref()))
        .collect();
    assert_eq!(
        holes,
        [
            ("/scenario/payoffs/frcp-rule12-example::settled", None),
            ("/sources/1/as_of", Some("2026-06")),
            ("/sources/2/as_of", None),
        ]
    );
    let docket: Vec<(Option<u64>, &str, EventRelation, &str)> = named
        .events
        .iter()
        .map(|e| (e.docket, e.action.as_str(), e.relation, e.party.as_str()))
        .collect();
    assert_eq!(
        docket,
        [
            (Some(1), "Served(complaint)", EventRelation::On, "widget"),
            (Some(2), "Answered(complaint)", EventRelation::By, "widget"),
        ]
    );
    assert_eq!(named.parties.len(), 2);
    let text = render_scenario(&named).unwrap_or_default();
    assert_eq!(NamedScenario::from_json(&text).ok(), Some(named));
}

/// `analysis` types every litgraph `Scenario` field the matter writes:
/// its fields, the dedicated blocks and the untyped fields it refuses are
/// exactly the engine's.
#[test]
fn analysis_fields_are_the_engine_scenario_fields() {
    use litigate_sema::case::resolve::analysis::{DEDICATED, FIELDS, UNTYPED};
    let schema =
        serde_json::to_value(schemars::schema_for!(litgraph::Scenario)).unwrap_or_default();
    let mut engine: Vec<&str> = schema["properties"]
        .as_object()
        .into_iter()
        .flat_map(|p| p.keys().map(String::as_str))
        .collect();
    engine.sort_unstable();
    let mut ours: Vec<&str> = FIELDS
        .iter()
        .chain(&DEDICATED)
        .chain(&UNTYPED)
        .copied()
        .collect();
    ours.sort_unstable();
    assert_eq!(ours, engine);
}

/// Each typed `analysis` field lowers to its `Scenario` field: numbers to
/// binary64, set names in order, no `params` when none are written.
#[test]
fn typed_analysis_lowers_field_by_field() {
    let mut case = acme();
    case.perspective = None;
    case.analysis = analysis_of(
        json!({"calibration": ["a", "b"], "discount_annual": 0.08})
            .as_object()
            .cloned()
            .unwrap_or_default(),
    );
    let lowered = Lowerer::new(&[])
        .case(&case, "cases/x.case")
        .unwrap_or_else(|d| panic!("{d:#?}"));
    let s = &lowered.scenario.scenario;
    assert_eq!(s.calibration, ["a", "b"]);
    assert_eq!(s.discount_annual, Some(0.08));
    assert_eq!(s.params, BTreeMap::new());
}
