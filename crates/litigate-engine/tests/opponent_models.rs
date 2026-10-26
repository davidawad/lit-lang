// SPDX-License-Identifier: GPL-3.0-or-later
//! Per-decision opponent models (`scenario.opponent_models`, lg-hgrj): at a
//! stipulation the opponent may accept, the response can be adversarial
//! (the default), a calibrated chance, or the opponent's own best payoff.
//! The fixture is synthetic: stipulating is worth 100 if accepted and 60 if
//! refused, moving instead is worth 70, so an always-refusing opponent
//! makes moving the better course and any real chance of acceptance can
//! reverse it.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use litigate_engine::algo::{equilibrium, mdp};
use litigate_engine::api::{handle_json, Catalog};
use litigate_engine::model::{CompileOptions, Graph, LinkFile, Pack};
use litigate_engine::scenario::{
    Control, ModelKind, OpponentMode, OpponentModel, Scenario, View, Warning,
};
use serde_json::{json, Value};
use std::collections::BTreeMap;

fn pack_json() -> Value {
    json!({
        "schemaVersion": 2, "id": "stip", "title": "stip", "startNodeId": "start",
        "roles": { "plaintiff": "self", "defendant": "opponent", "court": "nature" },
        "nodes": [
            { "id": "start", "kind": "decision", "label": "start" },
            { "id": "response", "kind": "state", "label": "stipulation response" },
            { "id": "stipulated", "kind": "terminal", "label": "stipulated", "payoff": 100.0,
              "attrs": { "opp_payoff": 10.0 } },
            { "id": "refused", "kind": "terminal", "label": "refused", "payoff": 60.0,
              "attrs": { "opp_payoff": 0.0 } },
            { "id": "moved", "kind": "terminal", "label": "moved", "payoff": 70.0 },
            { "id": "mixed", "kind": "state", "label": "opponent and court" },
            { "id": "ruled", "kind": "terminal", "label": "ruled", "payoff": 0.0 }
        ],
        "edges": [
            { "id": "stipulate", "from": "start", "to": "response", "label": "stipulate", "actor": "plaintiff" },
            { "id": "move", "from": "start", "to": "moved", "label": "move", "actor": "plaintiff" },
            { "id": "accept", "from": "response", "to": "stipulated", "label": "accept", "actor": "defendant" },
            { "id": "refuse", "from": "response", "to": "refused", "label": "refuse", "actor": "defendant" },
            { "id": "concede", "from": "mixed", "to": "stipulated", "label": "concede", "actor": "defendant" },
            { "id": "rule", "from": "mixed", "to": "ruled", "label": "rule", "actor": "court", "probability": 0.5 }
        ]
    })
}

fn graph() -> Graph {
    let p: Pack = serde_json::from_value(pack_json()).unwrap();
    Graph::compile(&[p], &LinkFile::default(), &CompileOptions::default()).unwrap()
}

fn models(entries: Value) -> BTreeMap<String, OpponentModel> {
    serde_json::from_value(entries).unwrap()
}

fn with(entries: Value) -> Scenario {
    Scenario {
        opponent_models: models(entries),
        ..Scenario::default()
    }
}

/// `(value at start, our choice there)`.
fn solve(g: &Graph, sc: &Scenario) -> (f64, String) {
    let v = View::new(g, sc).unwrap();
    let s = mdp::solve(&v, &mdp::SolveOptions::default()).unwrap();
    (s.value[v.start], g.edges[s.choice[&v.start]].id.clone())
}

fn coded<'a>(ws: &'a [Warning], code: &str) -> Vec<&'a Warning> {
    ws.iter().filter(|w| w.code == code).collect()
}

#[test]
fn the_default_opponent_refuses_so_stipulating_never_beats_moving() {
    let g = graph();
    let (value, choice) = solve(&g, &Scenario::default());
    assert_eq!(choice, "stip::move");
    assert!((value - 70.0).abs() < 1e-9);
    let (value, choice) = solve(
        &g,
        &with(json!({ "stip::response": { "model": "adversarial" } })),
    );
    assert_eq!(choice, "stip::move");
    assert!((value - 70.0).abs() < 1e-9);
}

#[test]
fn a_calibrated_acceptance_makes_stipulating_worth_it() {
    let g = graph();
    let sc = with(json!({
        "stip::response": { "model": "chance", "probabilities": { "accept": 0.4 } }
    }));
    let v = View::new(&g, &sc).unwrap();
    let response = g.node("stip::response").unwrap();
    assert_eq!(v.plan[response].control, Control::Chance);
    assert_eq!(v.plan[response].model, Some(ModelKind::Chance));
    // The named edge takes its number; the unnamed refusal takes the rest.
    assert_eq!(v.prob[g.edge("stip::accept").unwrap()], Some(0.4));
    assert!((v.prob[g.edge("stip::refuse").unwrap()].unwrap() - 0.6).abs() < 1e-12);
    let (value, choice) = solve(&g, &sc);
    assert_eq!(choice, "stip::stipulate");
    assert!((value - 76.0).abs() < 1e-9);
    // Below 25% acceptance moving is still better.
    let low = with(json!({
        "stip::response": { "model": "chance", "probabilities": { "accept": 0.2, "refuse": 0.8 } }
    }));
    assert_eq!(solve(&g, &low).1, "stip::move");
}

#[test]
fn a_chance_model_naming_no_edge_keeps_the_authored_probabilities() {
    let mut j = pack_json();
    j["edges"][2]["probability"] = json!(0.3);
    j["edges"][3]["probability"] = json!(0.7);
    let p: Pack = serde_json::from_value(j).unwrap();
    let g = Graph::compile(&[p], &LinkFile::default(), &CompileOptions::default()).unwrap();
    // Authored but adversarial: the opponent still refuses.
    let adversarial = Scenario {
        opponent: OpponentMode::Adversarial,
        ..Scenario::default()
    };
    assert_eq!(solve(&g, &adversarial).1, "stip::move");
    // The per-node chance model overrides the scenario-wide mode.
    let sc = Scenario {
        opponent: OpponentMode::Adversarial,
        ..with(json!({ "stip::response": { "model": "chance" } }))
    };
    let (value, choice) = solve(&g, &sc);
    assert_eq!(choice, "stip::stipulate");
    assert!((value - 72.0).abs() < 1e-9);
    // Unauthored, the fill splits the draw evenly and says so.
    let g = graph();
    let sc = with(json!({ "stip::response": { "model": "chance" } }));
    let v = View::new(&g, &sc).unwrap();
    assert_eq!(coded(&v.warnings, "probability-fill").len(), 1);
    assert!((solve(&g, &sc).0 - 80.0).abs() < 1e-9);
}

#[test]
fn an_own_objective_opponent_takes_its_best_payoff() {
    let g = graph();
    let accepts = with(json!({
        "stip::response": { "model": "own-objective", "payoffs": { "accept": 5000.0, "refuse": -2000.0 } }
    }));
    let v = View::new(&g, &accepts).unwrap();
    let response = g.node("stip::response").unwrap();
    assert_eq!(v.plan[response].control, Control::Opponent);
    assert_eq!(v.plan[response].model, Some(ModelKind::OwnObjective));
    assert_eq!(
        v.plan[response].choices,
        vec![g.edge("stip::accept").unwrap()]
    );
    let (value, choice) = solve(&g, &accepts);
    assert_eq!(choice, "stip::stipulate");
    assert!((value - 100.0).abs() < 1e-9);
    // Refusing pays them more: they refuse, and we move.
    let refuses = with(json!({
        "stip::response": { "model": "own-objective", "payoffs": { "accept": 0.0, "refuse": 1.0 } }
    }));
    assert_eq!(solve(&g, &refuses).1, "stip::move");
    // A tie goes against us; an unlisted edge is never taken.
    let tie = with(json!({
        "stip::response": { "model": "own-objective", "payoffs": { "accept": 1.0, "refuse": 1.0 } }
    }));
    assert_eq!(solve(&g, &tie).1, "stip::move");
    let only = with(json!({
        "stip::response": { "model": "own-objective", "payoffs": { "accept": -1.0 } }
    }));
    assert_eq!(solve(&g, &only).1, "stip::stipulate");
}

#[test]
fn an_own_objective_model_with_no_active_edge_falls_back_and_warns() {
    let g = graph();
    let sc = Scenario {
        remove_edges: vec!["stip::accept".into()],
        ..with(json!({
            "stip::response": { "model": "own-objective", "payoffs": { "accept": 1.0 } }
        }))
    };
    let v = View::new(&g, &sc).unwrap();
    let unused = coded(&v.warnings, "opponent-model-unused");
    assert_eq!(unused.len(), 1);
    assert_eq!(unused[0].at.as_deref(), Some("stip::response"));
    assert!(unused[0].message.contains("active opponent edge"));
    let response = g.node("stip::response").unwrap();
    assert_eq!(v.plan[response].model, None);
    assert_eq!(v.plan[response].control, Control::Opponent);
}

#[test]
fn a_model_where_the_opponent_does_not_choose_alone_is_unused() {
    let g = graph();
    for node in ["stip::start", "stip::stipulated"] {
        let sc = with(json!({ node: { "model": "adversarial" } }));
        let v = View::new(&g, &sc).unwrap();
        let unused = coded(&v.warnings, "opponent-model-unused");
        if node == "stip::start" {
            assert_eq!(unused.len(), 1, "{node}");
            assert!(unused[0].message.contains("only the opponent chooses"));
        } else {
            // A terminal has no plan to follow a model; nothing to warn.
            assert!(unused.is_empty());
        }
    }
}

#[test]
fn a_chance_model_naming_a_court_edge_sets_it_too() {
    let g = graph();
    let sc = with(json!({
        "stip::mixed": { "model": "chance", "probabilities": { "concede": 0.25, "rule": 0.75 } }
    }));
    let v = View::new(&g, &sc).unwrap();
    let mixed = g.node("stip::mixed").unwrap();
    assert_eq!(v.plan[mixed].control, Control::Chance);
    assert_eq!(v.prob[g.edge("stip::concede").unwrap()], Some(0.25));
    assert_eq!(v.prob[g.edge("stip::rule").unwrap()], Some(0.75));
    // Naming only the opponent's edge leaves the court's authored 0.5.
    let opp_only = with(json!({
        "stip::mixed": { "model": "chance", "probabilities": { "concede": 0.25 } }
    }));
    let v = View::new(&g, &opp_only).unwrap();
    assert!((v.prob[g.edge("stip::concede").unwrap()].unwrap() - 1.0 / 3.0).abs() < 1e-12);
    assert!((v.prob[g.edge("stip::rule").unwrap()].unwrap() - 2.0 / 3.0).abs() < 1e-12);
    // An own-objective model keeps the court's draw as an interrupt.
    let own = with(json!({
        "stip::mixed": { "model": "own-objective", "payoffs": { "concede": 1.0 } }
    }));
    let v = View::new(&g, &own).unwrap();
    assert_eq!(v.plan[mixed].control, Control::Opponent);
    assert_eq!(
        v.plan[mixed].draws,
        vec![(g.edge("stip::rule").unwrap(), 0.5)]
    );
}

#[test]
fn an_adversarial_model_stays_zero_sum_under_a_general_sum_opponent() {
    let g = graph();
    let general = Scenario {
        opponent_objective: Some("node.opp_payoff".into()),
        ..Scenario::default()
    };
    // Accepting pays the opponent 10 against 0: a self-interested opponent accepts.
    let v = View::new(&g, &general).unwrap();
    let eq = equilibrium::resolve(&v, &mdp::SolveOptions::default()).unwrap();
    let response = g.node("stip::response").unwrap();
    assert_eq!(
        eq.solution.choice[&response],
        g.edge("stip::accept").unwrap()
    );
    assert_eq!(
        eq.solution.choice[&v.start],
        g.edge("stip::stipulate").unwrap()
    );
    // The adversarial model at the response refuses all the same.
    let sc = Scenario {
        opponent_models: models(json!({ "stip::response": { "model": "adversarial" } })),
        ..general
    };
    let v = View::new(&g, &sc).unwrap();
    let eq = equilibrium::resolve(&v, &mdp::SolveOptions::default()).unwrap();
    assert_eq!(
        eq.solution.choice[&response],
        g.edge("stip::refuse").unwrap()
    );
    assert_eq!(eq.solution.choice[&v.start], g.edge("stip::move").unwrap());
    assert!((eq.opponent_value[response] - 0.0).abs() < 1e-12);
    // A chance model is a draw under general-sum too.
    let sc = Scenario {
        opponent_models: models(json!({
            "stip::response": { "model": "chance", "probabilities": { "accept": 0.5 } }
        })),
        ..sc
    };
    let v = View::new(&g, &sc).unwrap();
    let eq = equilibrium::resolve(&v, &mdp::SolveOptions::default()).unwrap();
    assert!((eq.solution.value[response] - 80.0).abs() < 1e-9);
    assert!((eq.opponent_value[response] - 5.0).abs() < 1e-9);
}

#[test]
fn bad_entries_are_rejected() {
    let g = graph();
    let err = |sc: Scenario| View::new(&g, &sc).err().unwrap();
    let e = err(with(json!({
        "stip::response": { "model": "chance", "probabilities": { "accept": 1.5 } }
    })));
    assert_eq!(e.code(), "invalid");
    assert!(e
        .to_string()
        .contains("opponent_models.stip::response.accept must be a probability in [0,1]"));
    let mut payoffs = BTreeMap::new();
    payoffs.insert("accept".to_string(), f64::NAN);
    let nan = Scenario {
        opponent_models: [(
            "stip::response".to_string(),
            OpponentModel::OwnObjective { payoffs },
        )]
        .into_iter()
        .collect(),
        ..Scenario::default()
    };
    assert!(err(nan).to_string().contains("must be a finite payoff"));
    let e = err(with(json!({ "stip::nowhere": { "model": "adversarial" } })));
    assert_eq!(e.code(), "not-found");
    let e = err(with(json!({
        "stip::response": { "model": "own-objective", "payoffs": { "move": 1.0 } }
    })));
    assert_eq!(e.code(), "not-found");
    // The shape is strict: an unknown model or field does not parse.
    for bad in [
        json!({ "n": { "model": "cooperative" } }),
        json!({ "n": { "model": "chance", "p": {} } }),
        json!({ "n": { "model": "own-objective" } }),
    ] {
        assert!(serde_json::from_value::<BTreeMap<String, OpponentModel>>(bad).is_err());
    }
}

fn request(scenario: &Value, op: &Value) -> Value {
    let catalog =
        Catalog::from_files("t".into(), vec![("t.json".into(), pack_json().to_string())]).unwrap();
    let req = json!({ "packs": ["stip"], "scenario": scenario, "op": op });
    let resp = handle_json(&req.to_string(), &catalog);
    assert!(resp.ok, "{resp:?}");
    serde_json::to_value(resp).unwrap()
}

#[test]
fn the_request_carries_the_models_into_explain_and_provenance() {
    let explain = json!({ "op": "explain", "node": "response" });
    let cases = [
        (
            json!({ "model": "chance", "probabilities": { "accept": 0.4 } }),
            "opponent (chance: calibrated response)",
        ),
        (
            json!({ "model": "own-objective", "payoffs": { "accept": 1.0 } }),
            "opponent (own payoffs at this decision)",
        ),
        (json!({ "model": "adversarial" }), "opponent (adversarial)"),
    ];
    for (model, who) in cases {
        let sc = json!({ "opponent_models": { "stip::response": model } });
        let out = request(&sc, &explain);
        assert_eq!(out["result"]["who_decides"], json!(who));
        assert_eq!(
            out["provenance"]["modes"]["opponent_models"]["stip::response"],
            model
        );
    }
    // No models, no provenance entry.
    let out = request(&json!({}), &explain);
    assert!(out["provenance"]["modes"].get("opponent_models").is_none());
}

#[test]
fn explain_regret_is_ours_at_an_adversarial_node_under_general_sum() {
    let sc = json!({
        "opponent_objective": "node.opp_payoff",
        "opponent_models": { "stip::response": { "model": "adversarial" } }
    });
    let out = request(&sc, &json!({ "op": "explain", "node": "response" }));
    let options = out["result"]["options"].as_array().unwrap();
    let accept = options
        .iter()
        .find(|o| o["id"] == json!("stip::accept"))
        .unwrap();
    let refuse = options
        .iter()
        .find(|o| o["id"] == json!("stip::refuse"))
        .unwrap();
    assert_eq!(refuse["chosen"], json!(true));
    // Accepting would cost the adversary 40 of our value it means to deny us.
    assert_eq!(accept["regret"], json!(40.0));
    assert_eq!(
        out["result"]["who_decides"],
        json!("opponent (adversarial)")
    );
}
