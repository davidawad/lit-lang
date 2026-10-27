// SPDX-License-Identifier: GPL-3.0-or-later
//! Unit tests for `view.rs`.

#![allow(clippy::unwrap_used)]
use super::*;
use crate::model::{CompileOptions, LinkFile, Pack};

/// A `fact`-tagged decision with two `office` (nature) out-edges and an
/// authored 90/10 prior, mirroring `cofc::limitations-check`.
fn fact_pack(json: &str) -> Graph {
    let pack = Pack::from_json(json).unwrap();
    Graph::compile(&[pack], &LinkFile::default(), &CompileOptions::default()).unwrap()
}

const FACT_JSON: &str = r#"{
    "schemaVersion": 2, "id": "demo", "title": "Demo", "startNodeId": "check",
    "nodes": [
        {"id": "check", "label": "Check", "kind": "decision", "tags": ["fact"]},
        {"id": "ok", "label": "OK", "kind": "terminal", "payoff": 1000.0},
        {"id": "barred", "label": "Barred", "kind": "terminal", "payoff": 0.0}
    ],
    "edges": [
        {"id": "e-ok", "from": "check", "to": "ok", "label": "clear", "actor": "office", "probability": 0.9},
        {"id": "e-barred", "from": "check", "to": "barred", "label": "barred", "actor": "office", "probability": 0.1}
    ]
}"#;

#[test]
fn unset_fact_uses_the_authored_prior_and_warns() {
    let g = fact_pack(FACT_JSON);
    let v = View::new(&g, &Scenario::default()).unwrap();
    let check = g.node("demo::check").unwrap();
    let ok = g.edge("demo::e-ok").unwrap();
    let barred = g.edge("demo::e-barred").unwrap();
    assert_eq!(v.prob[ok], Some(0.9));
    assert_eq!(v.prob[barred], Some(0.1));
    assert!(v
        .warnings
        .iter()
        .any(|w| w.code == "fact-unset" && w.at.as_deref() == Some(g.nodes[check].id.as_str())));
}

#[test]
fn a_fact_node_with_one_live_edge_is_decided_without_warning() {
    // A links instance that removes every route but one (`cafc@cofc`'s
    // `remand-destination`) leaves the fact nothing to decide.
    let json = FACT_JSON.replace(
        r#",
        {"id": "e-barred", "from": "check", "to": "barred", "label": "barred", "actor": "office", "probability": 0.1}"#,
        "",
    );
    let g = fact_pack(&json);
    let v = View::new(&g, &Scenario::default()).unwrap();
    assert!(!v.warnings.iter().any(|w| w.code == "fact-unset"));
}

#[test]
fn set_fact_forces_its_edge_and_zeroes_the_sibling_without_warning() {
    let g = fact_pack(FACT_JSON);
    let sc = Scenario {
        facts: BTreeMap::from([("demo::check".into(), "demo::e-barred".into())]),
        ..Scenario::default()
    };
    let v = View::new(&g, &sc).unwrap();
    let ok = g.edge("demo::e-ok").unwrap();
    let barred = g.edge("demo::e-barred").unwrap();
    assert_eq!(v.prob[barred], Some(1.0));
    assert_eq!(v.prob[ok], Some(0.0));
    assert!(!v.warnings.iter().any(|w| w.code == "fact-unset"));
    assert!(!v.warnings.iter().any(|w| w.code == "probability-fill"));
}

#[test]
fn a_non_fact_chance_node_is_unaffected() {
    // Same shape, no `fact` tag: plain chance-node warnings still apply
    // (regression guard: the fact path must not swallow ordinary nodes).
    const JSON: &str = r#"{
        "schemaVersion": 2, "id": "demo", "title": "Demo", "startNodeId": "check",
        "nodes": [
            {"id": "check", "label": "Check", "kind": "decision"},
            {"id": "a", "label": "A", "kind": "terminal", "payoff": 1.0},
            {"id": "b", "label": "B", "kind": "terminal", "payoff": 0.0}
        ],
        "edges": [
            {"from": "check", "to": "a", "label": "a", "actor": "office"},
            {"from": "check", "to": "b", "label": "b", "actor": "office"}
        ]
    }"#;
    let g = fact_pack(JSON);
    let v = View::new(&g, &Scenario::default()).unwrap();
    assert!(v.warnings.iter().any(|w| w.code == "probability-fill"));
    assert!(!v.warnings.iter().any(|w| w.code == "fact-unset"));
}

/// A fact is authored once for the pack node it describes, not once per
/// state-flag history that node can be reached under (same reasoning as
/// the calibration fix, docs/PACK_SCHEMA.md#state-flags): forcing
/// `demo::check`'s fact must also force `demo::check{remanded}`'s copy of
/// the same edge, not just the base node's.
#[test]
fn a_fact_set_on_the_base_node_also_forces_its_flagged_copies() {
    const FLAGGED_FACT_JSON: &str = r#"{
        "schemaVersion": 2, "id": "demo", "title": "Demo", "startNodeId": "check",
        "nodes": [
            {"id": "check", "label": "Check", "kind": "decision", "tags": ["fact"]},
            {"id": "ok", "label": "OK", "kind": "terminal", "payoff": 1000.0},
            {"id": "barred", "label": "Barred", "kind": "terminal", "payoff": 0.0},
            {"id": "remand", "label": "Remand", "kind": "state"}
        ],
        "edges": [
            {"id": "e-ok", "from": "check", "to": "ok", "label": "clear", "actor": "office", "probability": 0.9},
            {"id": "e-barred", "from": "check", "to": "barred", "label": "barred", "actor": "office", "probability": 0.1},
            {"id": "re-check", "from": "remand", "to": "check", "label": "re-check", "actor": "office", "sets": ["remanded"]}
        ]
    }"#;
    let g = fact_pack(FLAGGED_FACT_JSON);
    // The flagged copy must actually exist, or this test would pass for
    // the wrong reason.
    let check_flagged = g.node("demo::check{remanded}").expect(
        "check{remanded} should exist: remand->check sets the flag before re-entering check",
    );
    assert!(g.nodes[check_flagged].has_tag("fact"));

    let sc = Scenario {
        facts: BTreeMap::from([("demo::check".into(), "demo::e-barred".into())]),
        ..Scenario::default()
    };
    let v = View::new(&g, &sc).unwrap();

    let ok = g.edge("demo::e-ok").unwrap();
    let barred = g.edge("demo::e-barred").unwrap();
    assert_eq!(v.prob[barred], Some(1.0));
    assert_eq!(v.prob[ok], Some(0.0));

    let ok_flagged = g.edge("demo::e-ok{remanded}").unwrap();
    let barred_flagged = g.edge("demo::e-barred{remanded}").unwrap();
    assert_eq!(
        v.prob[barred_flagged],
        Some(1.0),
        "the flagged copy's edge must be forced too, not just the base node's"
    );
    assert_eq!(v.prob[ok_flagged], Some(0.0));

    assert!(!v.warnings.iter().any(|w| w.code == "fact-unset"));
}

/// Real-pack version of the synthetic test above: ptab's own
/// `petition-time-bar-check` (35 U.S.C. §315(b) time-bar, tagged `fact`)
/// grows a real flagged copy (`{prior-petition-denied}`) once a prior
/// petition on the same patent was denied institution (General Plastic, 35
/// U.S.C. §314(a) — see the sourced `sets` on
/// institution-denied-merits/-fintiv's follow-on edges). Setting the fact on
/// the base node must also force the flagged copy's edge, and neither copy
/// should warn `fact-unset`.
#[test]
fn a_real_pack_fact_node_forces_its_flagged_copy_too() {
    let pack = Pack::from_json(
        &std::fs::read_to_string(format!(
            "{}/ptab-patent-trial-appeal-board.json",
            litgraph_packs::PACKS_DIR
        ))
        .unwrap(),
    )
    .unwrap();
    let g = Graph::compile(&[pack], &LinkFile::default(), &CompileOptions::default())
        .expect("ptab pack compiles");

    let base = g
        .node("ptab-patent-trial-appeal-board::petition-time-bar-check")
        .unwrap();
    assert!(g.nodes[base].has_tag("fact"));
    let flagged = g
        .node("ptab-patent-trial-appeal-board::petition-time-bar-check{prior-petition-denied}")
        .expect(
            "the flagged copy should exist: institution-denied-merits/-fintiv's follow-on edges \
             set prior-petition-denied before a refiled petition re-enters the time-bar check",
        );
    assert!(g.nodes[flagged].has_tag("fact"));

    let sc = Scenario {
        facts: BTreeMap::from([(
            "ptab-patent-trial-appeal-board::petition-time-bar-check".into(),
            "ptab-patent-trial-appeal-board::e-time-bar-barred".into(),
        )]),
        ..Scenario::default()
    };
    let v = View::new(&g, &sc).unwrap();

    let barred = g
        .edge("ptab-patent-trial-appeal-board::e-time-bar-barred")
        .unwrap();
    let clear = g
        .edge("ptab-patent-trial-appeal-board::e-time-bar-clear")
        .unwrap();
    assert_eq!(v.prob[barred], Some(1.0));
    assert_eq!(v.prob[clear], Some(0.0));

    let barred_flagged = g
        .edge("ptab-patent-trial-appeal-board::e-time-bar-barred{prior-petition-denied}")
        .unwrap();
    let clear_flagged = g
        .edge("ptab-patent-trial-appeal-board::e-time-bar-clear{prior-petition-denied}")
        .unwrap();
    assert_eq!(
        v.prob[barred_flagged],
        Some(1.0),
        "the flagged copy's edge must be forced too"
    );
    assert_eq!(v.prob[clear_flagged], Some(0.0));

    // Neither copy should still be treated as an unset matter fact.
    assert!(!v.warnings.iter().any(|w| w.code == "fact-unset"));
}

/// One decision reaching one terminal either plainly or under flag `f`, so
/// the terminal compiles to two copies: `demo::end` and `demo::end{f}`.
/// Neither authors a payoff.
const FLAGGED_JSON: &str = r#"{
    "schemaVersion": 2, "id": "demo", "title": "Demo", "startNodeId": "go",
    "nodes": [
        {"id": "go", "label": "Go", "kind": "decision"},
        {"id": "end", "label": "End", "kind": "terminal", "feeShift": 0.0}
    ],
    "edges": [
        {"id": "plain", "from": "go", "to": "end", "label": "plain", "actor": "self"},
        {"id": "flag", "from": "go", "to": "end", "label": "flag", "actor": "self", "sets": ["f"]}
    ]
}"#;

#[test]
fn a_payoff_reaches_every_flagged_copy_of_its_terminal() {
    let g = fact_pack(FLAGGED_JSON);
    let copies = g.node_family(g.node("demo::end").unwrap());
    assert_eq!(copies.len(), 2);
    let sc = Scenario {
        payoffs: BTreeMap::from([("end".into(), 700.0)]),
        ..Scenario::default()
    };
    let v = View::new(&g, &sc).unwrap();
    for n in copies {
        assert_eq!(v.payoff[n], 700.0, "{}", g.nodes[n].id);
    }
    assert!(!v.warnings.iter().any(|w| w.code == "payoff-not-authored"));
}

#[test]
fn a_payoff_on_one_flagged_copy_wins_over_its_family() {
    let g = fact_pack(FLAGGED_JSON);
    let sc = Scenario {
        payoffs: BTreeMap::from([("demo::end".into(), 700.0), ("demo::end{f}".into(), 50.0)]),
        ..Scenario::default()
    };
    let v = View::new(&g, &sc).unwrap();
    assert_eq!(v.payoff[g.node("demo::end").unwrap()], 700.0);
    assert_eq!(v.payoff[g.node("demo::end{f}").unwrap()], 50.0);
}

#[test]
fn a_fee_shift_reaches_every_flagged_copy_of_its_terminal() {
    let g = fact_pack(FLAGGED_JSON);
    let sc = Scenario {
        fee_shifts: BTreeMap::from([("end".into(), 0.5)]),
        ..Scenario::default()
    };
    let v = View::new(&g, &sc).unwrap();
    for n in g.node_family(g.node("demo::end").unwrap()) {
        assert_eq!(v.fee[n], 0.5, "{}", g.nodes[n].id);
    }
}

/// Two chance edges with no package probability (holes) under one node.
const HOLES_JSON: &str = r#"{
    "schemaVersion": 2, "id": "demo", "title": "Demo", "startNodeId": "draw",
    "nodes": [
        {"id": "draw", "label": "Draw", "kind": "decision"},
        {"id": "a", "label": "A", "kind": "terminal", "payoff": 1.0},
        {"id": "b", "label": "B", "kind": "terminal", "payoff": 0.0}
    ],
    "edges": [
        {"id": "e-a", "from": "draw", "to": "a", "label": "a", "actor": "office"},
        {"id": "e-b", "from": "draw", "to": "b", "label": "b", "actor": "office"}
    ]
}"#;

/// lg-w0o0: overriding A=0.5 then B=0.5 on all-hole siblings must keep
/// A at 0.5, not rescale it to `0 + 0 - 0.5 → 0`.
#[test]
fn overriding_hole_siblings_keeps_each_override() {
    let g = fact_pack(HOLES_JSON);
    let sc = Scenario {
        probabilities: BTreeMap::from([("demo::e-a".into(), 0.5), ("demo::e-b".into(), 0.5)]),
        ..Scenario::default()
    };
    let v = View::new(&g, &sc).unwrap();
    assert_eq!(v.prob[g.edge("demo::e-a").unwrap()], Some(0.5));
    assert_eq!(v.prob[g.edge("demo::e-b").unwrap()], Some(0.5));
}

/// An override never rescales another override; only package-authored,
/// non-overridden siblings absorb the change in mass.
#[test]
fn a_later_override_does_not_rescale_an_earlier_one() {
    let json = r#"{
        "schemaVersion": 2, "id": "demo", "title": "Demo", "startNodeId": "draw",
        "nodes": [
            {"id": "draw", "label": "Draw", "kind": "decision"},
            {"id": "a", "label": "A", "kind": "terminal", "payoff": 1.0},
            {"id": "b", "label": "B", "kind": "terminal", "payoff": 0.5},
            {"id": "c", "label": "C", "kind": "terminal", "payoff": 0.0}
        ],
        "edges": [
            {"id": "e-a", "from": "draw", "to": "a", "label": "a", "actor": "office", "probability": 0.2},
            {"id": "e-b", "from": "draw", "to": "b", "label": "b", "actor": "office", "probability": 0.3},
            {"id": "e-c", "from": "draw", "to": "c", "label": "c", "actor": "office", "probability": 0.5}
        ]
    }"#;
    let g = fact_pack(json);
    let sc = Scenario {
        probabilities: BTreeMap::from([("demo::e-a".into(), 0.5), ("demo::e-b".into(), 0.3)]),
        ..Scenario::default()
    };
    let v = View::new(&g, &sc).unwrap();
    for (id, want) in [("demo::e-a", 0.5), ("demo::e-b", 0.3), ("demo::e-c", 0.2)] {
        let got = v.prob[g.edge(id).unwrap()].unwrap();
        assert!((got - want).abs() < 1e-12, "{id} = {got}, want {want}");
    }
}
