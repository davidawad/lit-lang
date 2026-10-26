// SPDX-License-Identifier: GPL-3.0-or-later
//! Statutory fee shifting (lg-vwwr): a terminal's `feeShift` offsets our
//! own hours, the fee motion's included, so a fee motion is worth the
//! award rather than the award less its own cost; a scenario's
//! `fee_shift` replaces the packs' fractions and `fee_shifts` sets one
//! terminal's.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use litgraph::algo::{mdp, sim};
use litgraph::model::{CompileOptions, Graph, LinkFile, Pack};
use litgraph::scenario::{FeeShift, Scenario, View};
use serde_json::{json, Value};

/// We won (litigation cost 300), then choose: move for fees (100 of our
/// hours) for a gross award of 50, or stop. `fees` carries `fee_shift`.
fn pack(fee_shift: &Value) -> Value {
    json!({
        "schemaVersion": 2, "id": "fees", "title": "fees", "startNodeId": "start",
        "roles": { "applicant": "self", "office": "nature" },
        "nodes": [
            { "id": "start", "kind": "decision", "label": "start" },
            { "id": "won", "kind": "decision", "label": "Judgment for plaintiff" },
            { "id": "fees", "kind": "terminal", "label": "Fees awarded", "payoff": 50,
              "cite": "42 U.S.C. 1988(b)", "feeShift": fee_shift },
            { "id": "stop", "kind": "terminal", "label": "No fee motion", "payoff": 0 }
        ],
        "edges": [
            { "id": "litigate", "from": "start", "to": "won", "label": "litigate", "actor": "applicant", "cost": 300 },
            { "id": "move", "from": "won", "to": "fees", "label": "move for fees", "actor": "applicant", "cost": 100 },
            { "id": "decline", "from": "won", "to": "stop", "label": "stop", "actor": "applicant" }
        ]
    })
}

fn graph(fee_shift: &Value) -> litgraph::Result<Graph> {
    let p: Pack = serde_json::from_value(pack(fee_shift)).expect("pack parses");
    Graph::compile(&[p], &LinkFile::default(), &CompileOptions::default())
}

/// The start's value and whether we move for fees.
fn solve(g: &Graph, sc: &Scenario) -> (f64, bool) {
    let v = View::new(g, sc).unwrap();
    let sol = mdp::solve(&v, &mdp::SolveOptions::default()).unwrap();
    let won = g.node("fees::won").unwrap();
    let moved = g.edges[sol.choice[&won]].id == "fees::move";
    (sol.value[v.start], moved)
}

#[test]
fn the_statutes_default_offsets_the_fee_motion_and_the_litigation() {
    // No fee shifting: the motion nets 50 - 100 < 0, so we stop.
    let (value, moved) = solve(&graph(&Value::Null).unwrap(), &Scenario::default());
    assert!(!moved);
    assert!((value + 300.0).abs() < 1e-9, "{value}");
    // Full fee shifting: every hour on the way to the award is recovered.
    let (value, moved) = solve(&graph(&json!(1.0)).unwrap(), &Scenario::default());
    assert!(moved);
    assert!((value - 50.0).abs() < 1e-9, "{value}");
    // Half: (300 + 100) / 2 recovered against the 50 award.
    let (value, moved) = solve(&graph(&json!(0.5)).unwrap(), &Scenario::default());
    assert!(moved);
    assert!((value + 150.0).abs() < 1e-9, "{value}");
}

#[test]
fn the_award_recovers_the_litigation_already_spent_so_the_motion_is_worth_filing() {
    // A 40 award at half: the motion alone nets 40 - 50 < 0, but it also
    // recovers 150 of the 300 already spent: 40 - 400 + 200 = -160 > -300.
    let g = graph(&json!(0.5)).unwrap();
    let sc = Scenario {
        payoffs: [("fees::fees".to_owned(), 40.0)].into(),
        ..Scenario::default()
    };
    let (value, moved) = solve(&g, &sc);
    assert!(moved);
    assert!((value + 160.0).abs() < 1e-9, "{value}");
}

#[test]
fn the_scenario_replaces_the_packs_fraction_and_a_terminal_override_wins() {
    let g = graph(&json!(1.0)).unwrap();
    // `fee_shift` with no eligible terminal turns the statute's default off.
    let off = Scenario {
        fee_shift: Some(FeeShift {
            fraction: 1.0,
            eligible: None,
        }),
        ..Scenario::default()
    };
    assert_eq!(solve(&g, &off), (-300.0, false));
    // A matter's own fraction at the terminal beats both.
    let matter = Scenario {
        fee_shifts: [("fees::fees".to_owned(), 0.5)].into(),
        ..off
    };
    let (value, moved) = solve(&g, &matter);
    assert!(moved);
    assert!((value + 150.0).abs() < 1e-9, "{value}");
}

#[test]
fn simulate_credits_the_terminals_fraction_of_our_spend() {
    let g = graph(&json!(0.5)).unwrap();
    let v = View::new(&g, &Scenario::default()).unwrap();
    let sol = mdp::solve(&v, &mdp::SolveOptions::default()).unwrap();
    let o = sim::SimOptions {
        runs: 10,
        seed: 7,
        alpha: 0.1,
        max_steps: 10,
        sample_durations: false,
        keep_samples: 0,
    };
    let r = sim::simulate(&v, &sol, v.start, &[], &o).unwrap();
    // 50 - 400 + 200.
    assert!((r.net.mean + 150.0).abs() < 1e-9, "{:?}", r.net);
}

#[test]
fn fractions_outside_zero_one_and_off_terminals_are_errors() {
    for bad in [json!(1.5), json!(-0.1)] {
        let err = graph(&bad).expect_err("rejected").to_string();
        assert!(err.contains("feeShift"), "{err}");
    }
    let mut off_terminal = pack(&Value::Null);
    off_terminal["nodes"][1]["feeShift"] = json!(0.5);
    let p: Pack = serde_json::from_value(off_terminal).unwrap();
    let err = Graph::compile(&[p], &LinkFile::default(), &CompileOptions::default())
        .expect_err("rejected")
        .to_string();
    assert!(err.contains("on a terminal"), "{err}");

    let g = graph(&Value::Null).unwrap();
    let cases = [
        (
            Scenario {
                fee_shift: Some(FeeShift {
                    fraction: 2.0,
                    eligible: None,
                }),
                ..Scenario::default()
            },
            "fee_shift.fraction",
        ),
        (
            Scenario {
                fee_shifts: [("fees::fees".to_owned(), -1.0)].into(),
                ..Scenario::default()
            },
            "not a fraction",
        ),
        (
            Scenario {
                fee_shifts: [("fees::won".to_owned(), 0.5)].into(),
                ..Scenario::default()
            },
            "not a terminal",
        ),
        (
            Scenario {
                fee_shifts: [("fees::nowhere".to_owned(), 0.5)].into(),
                ..Scenario::default()
            },
            "nowhere",
        ),
    ];
    for (sc, want) in cases {
        let err = View::new(&g, &sc).err().expect("rejected").to_string();
        assert!(err.contains(want), "{want}: {err}");
    }
}

#[test]
fn cvar_and_general_sum_warn_when_only_a_pack_shifts_fees() {
    let g = graph(&json!(1.0)).unwrap();
    let sc: Scenario = serde_json::from_value(json!({
        "objective": { "type": "cvar", "alpha": 0.1 },
        "opponent_objective": "0"
    }))
    .unwrap();
    let v = View::new(&g, &sc).unwrap();
    let codes: Vec<&str> = v.warnings.iter().map(|w| w.code).collect();
    assert!(codes.contains(&"cvar-ignores-fee-shift"), "{codes:?}");
    assert!(
        codes.contains(&"opponent-objective-ignores-fee-shift"),
        "{codes:?}"
    );
}
