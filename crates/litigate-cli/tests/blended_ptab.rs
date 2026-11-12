//! PTAB terminal splits preserve the last fully modeled final written decision.
// Test helpers below may unwrap: a panic is the failed assertion.
#![allow(clippy::unwrap_used)]
mod support;

use serde_json::{json, Value};
use support::{litigate, Run};

/// The engine response an op subcommand returned, after checking its envelope.
fn response(run: &Run, op: &str) -> Value {
    assert_eq!(run.code, 0, "{}{}", run.stdout, run.stderr);
    assert_eq!(run.json["op"], op);
    assert_eq!(run.json["ok"], true);
    let response = run.json["result"].clone();
    assert_eq!(
        (&response["op"], &response["ok"]),
        (&json!(op), &json!(true))
    );
    response
}

#[test]
fn ptab_blended_terminals_follow_the_last_fwd_result() {
    let pkg = "ptab-patent-trial-appeal-board::";
    let contexts = [
        ("fwd-all-unpatentable,ipr-estopped", "win", 1_000_000.0),
        ("fwd-all-confirmed,ipr-estopped", "loss", 0.0),
        ("ipr-estopped", "neutral", 0.0),
    ];
    let split_terminals = [
        (
            "appeal-forfeited-unappealed",
            "appeal-forfeited-unappealed-win",
            "appeal-forfeited-unappealed-loss",
            400_000.0,
        ),
        (
            "cafc-affirms",
            "cafc-affirms-win",
            "cafc-affirms-loss",
            450_000.0,
        ),
    ];

    for (neutral_id, win_id, loss_id, neutral_payoff) in split_terminals {
        for (flags, direction, directional_payoff) in contexts {
            let node = if neutral_id == "cafc-affirms" {
                "federal-circuit-review"
            } else {
                "federal-circuit-window"
            };
            let query = format!("node={pkg}{node}{{{flags}}}");
            let graph = response(&litigate(&["graph", "lit/ptab", "--arg", &query]), "graph")
                ["result"]
                .clone();
            let target_id = match direction {
                "win" => win_id,
                "loss" => loss_id,
                _ => neutral_id,
            };
            let edge = graph["edges"]
                .as_array()
                .unwrap()
                .iter()
                .find(|edge| edge["to"].as_str().unwrap().contains(target_id))
                .unwrap_or_else(|| {
                    panic!("{node} with {flags} does not reach {target_id}: {graph}")
                });
            assert_eq!(edge["active"], true);
            let terminal = graph["nodes"]
                .as_array()
                .unwrap()
                .iter()
                .find(|node| node["id"].as_str().unwrap() == edge["to"])
                .unwrap();
            assert!(terminal["outcome"]
                .as_array()
                .unwrap()
                .iter()
                .any(|outcome| outcome
                    == if direction == "neutral" {
                        "final"
                    } else {
                        direction
                    }));
            assert_eq!(
                terminal["payoff"],
                if direction == "neutral" {
                    json!(neutral_payoff)
                } else {
                    json!(directional_payoff)
                }
            );
        }
    }
}
