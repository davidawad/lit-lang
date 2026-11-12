//! One command line (lg-9lrj): every engine op is a subcommand on a
//! compiled matter or package (`litigate solve CASE`, `litigate explain
//! lit/pkg`); the engine has no raw JSON interface (lg-dfv4.7).
// Test helpers below may unwrap: a panic is the failed assertion.
#![allow(clippy::unwrap_used)]
mod support;

use litigate_engine::model::{CompileOptions, Graph, LinkFile};
use serde_json::{json, Value};
use support::{example, litigate, litigate_in, scratch, Run};

const ACME: &str = "cases/examples/acme.case";
const RULING: &str = "frcp-rule12-example::mtd-ruling";

/// The engine response an op subcommand returned, after checking the
/// envelope names the op and both are ok.
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
fn an_op_runs_on_a_matter_with_set_params_composed_onto_it() {
    let base = response(&litigate(&["solve", ACME]), "solve");
    let run = litigate(&["run", ACME, "--op", "solve"]);
    assert_eq!(run.json["op"], "run");
    assert_eq!(run.json["result"]["result"], base["result"]);
    // The civil case bills hours at its `rate`, so the rate moves its value.
    let civil = "cases/frcp-civil-case.case";
    let billed = response(&litigate(&["solve", civil]), "solve");
    let cheap = response(&litigate(&["solve", civil, "--set", "rate=1"]), "solve");
    assert_eq!(billed["result"]["start"], cheap["result"]["start"]);
    assert_ne!(billed["result"]["value"], cheap["result"]["value"]);
}

#[test]
fn frcp_civil_case_solve_uses_party_relative_judgment_stakes() {
    // The three-state judgment split preserves $0 for paths that never
    // establish a winner; the Rule 60 ruling is court-controlled, not a
    // plaintiff choice. This value includes both merits outcomes.
    let solved = response(&litigate(&["solve", "cases/frcp-civil-case.case"]), "solve");
    assert_eq!(solved["result"]["value"], 75_713.461_431);
    let labels: Vec<&str> = solved["result"]["best_line"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|edge| edge["label"].as_str())
        .collect();
    assert!(
        labels.contains(&"Request waiver of service instead of formal process"),
        "{labels:?}"
    );
    let line = solved["result"]["best_line"].as_array().unwrap();
    let terminal = line.last().unwrap()["to"].as_str().unwrap();
    let win_terminal = [
        "default-judgment-entered",
        "discovery-sanctions-default",
        "summary-judgment-for-plaintiff",
        "appeal-window-blown-plaintiff-judgment",
        "judgment-final-no-appeal-plaintiff",
        "judgment-affirmed-plaintiff",
    ]
    .iter()
    .any(|id| terminal.contains(id));
    if win_terminal {
        let merits_edges = [
            "frcp-civil-procedure::verdict-for-plaintiff",
            "frcp-civil-procedure::edge-sj-ruling-summary-judgment-for-plaintiff",
            "frcp-civil-procedure::edge-default-judgment-enters",
            "frcp-civil-procedure::edge-discovery-sanctions-default",
        ];
        assert!(
            line.iter()
                .filter_map(|edge| edge["id"].as_str())
                .any(|id| { merits_edges.iter().any(|merits| id.starts_with(merits)) }),
            "[win] terminal {terminal} has no preceding merits edge: {line:?}"
        );
    }
}

#[test]
fn frap_blended_terminals_follow_the_panel_result_flags() {
    let catalog = litgraph_packs::catalog().unwrap();
    let pack = catalog
        .packs
        .iter()
        .find(|(_, pack, _)| pack.id == "frap-appellate-procedure")
        .unwrap()
        .1
        .clone();
    assert_eq!(
        pack.nodes
            .iter()
            .find(|node| node.id == "rehearing-en-banc-both-denied")
            .unwrap()
            .payoff,
        Some(400_000.0)
    );
    let graph = Graph::compile(&[pack], &LinkFile::default(), &CompileOptions::default()).unwrap();

    let cases = [
        (
            "rehearing-en-banc-both-denied",
            "rehearing-en-banc-both-denied-after-affirmance",
            "rehearing-en-banc-both-denied-after-reversal",
        ),
        (
            "cert-not-sought",
            "cert-not-sought-after-affirmance",
            "cert-not-sought-after-reversal",
        ),
        (
            "cert-denied",
            "cert-denied-after-affirmance",
            "cert-denied-after-reversal",
        ),
    ];
    for (neutral_id, adverse_id, favourable_id) in cases {
        assert_frap_split(&graph, neutral_id, adverse_id, favourable_id);
    }
}

fn assert_frap_split(graph: &Graph, neutral_id: &str, adverse_id: &str, favourable_id: &str) {
    let reachable = reachable_nodes(graph);
    let neutral = graph
        .node(&format!("frap-appellate-procedure::{neutral_id}"))
        .unwrap();
    let adverse = graph
        .node(&format!(
            "frap-appellate-procedure::{adverse_id}{{panel-affirmed}}"
        ))
        .unwrap();
    let reversed = graph
        .node(&format!(
            "frap-appellate-procedure::{favourable_id}{{panel-reversed}}"
        ))
        .unwrap();
    let remanded = graph
        .node(&format!(
            "frap-appellate-procedure::{favourable_id}{{panel-reversed-and-remanded}}"
        ))
        .unwrap();
    for node in [neutral, adverse, reversed, remanded] {
        assert!(
            reachable[node],
            "split copy is unreachable: {}",
            graph.nodes[node].id
        );
    }
    assert_eq!(
        graph.nodes[neutral].id,
        format!("frap-appellate-procedure::{neutral_id}")
    );
    assert!(graph.nodes[neutral].flags.is_empty());
    let ends = [
        terminal_copy(graph, neutral),
        terminal_copy(graph, adverse),
        terminal_copy(graph, reversed),
        terminal_copy(graph, remanded),
    ];
    assert_eq!(graph.nodes[ends[0]].payoff, 400_000.0);
    assert!(graph.nodes[ends[0]].outcome.contains(&"final".to_owned()));
    assert_eq!(graph.nodes[ends[1]].payoff, 0.0);
    assert!(graph.nodes[ends[1]].outcome.contains(&"loss".to_owned()));
    assert_eq!(graph.nodes[ends[2]].payoff, 1_000_000.0);
    assert!(graph.nodes[ends[2]].outcome.contains(&"win".to_owned()));
    assert_eq!(graph.nodes[ends[3]].payoff, 600_000.0);
    assert!(graph.nodes[ends[3]].outcome.contains(&"win".to_owned()));
}

fn reachable_nodes(graph: &Graph) -> Vec<bool> {
    let mut seen = vec![false; graph.nodes.len()];
    let mut queue = std::collections::VecDeque::from([graph.start]);
    while let Some(node) = queue.pop_front() {
        if seen[node] {
            continue;
        }
        seen[node] = true;
        queue.extend(graph.out[node].iter().map(|edge| graph.edges[*edge].to));
    }
    seen
}

fn terminal_copy(graph: &Graph, node: usize) -> usize {
    graph.out[node]
        .iter()
        .find(|edge| graph.edges[**edge].synthetic)
        .map_or(node, |edge| graph.edges[*edge].to)
}

#[test]
fn an_unknown_set_param_returns_an_error_envelope_with_nearest_valid_param() {
    let run = litigate(&["solve", "cases/frcp-civil-case.case", "--set", "raet=900"]);
    assert_eq!(run.code, 2, "{}{}", run.stdout, run.stderr);
    assert_eq!(run.json["ok"], true);
    assert_eq!(run.json["result"]["ok"], false);
    assert_eq!(run.json["result"]["error"]["code"], "unknown_param");
    assert!(run.json["result"]["error"]["message"]
        .as_str()
        .unwrap()
        .contains("raet"));
    assert!(run.json["result"]["error"]["hint"]
        .as_str()
        .unwrap()
        .contains("rate"));
}

#[test]
fn an_op_runs_on_a_package_directory_or_one_of_its_files() {
    let dir = response(&litigate(&["chain", "lit/examples"]), "chain");
    let file = response(&litigate(&["chain", "lit/examples/rule12.lit"]), "chain");
    assert_eq!(dir["result"], file["result"]);
    let set = response(
        &litigate(&["chain", "lit/examples", "--set", "rate=1", "--no-links"]),
        "chain",
    );
    assert_eq!(set["result"]["start"], dir["result"]["start"]);
}

#[test]
fn a_package_that_fails_its_check_is_not_run() {
    let dir = example("ops-broken-package", "frobnicate", None);
    let run = litigate_in(&dir, &["solve", "lit/examples"]);
    assert_eq!(run.code, 2, "{}{}", run.stdout, run.stderr);
    assert_eq!(run.json["ok"], false);
    assert!(run.json.get("result").is_none(), "{}", run.stdout);
}

#[test]
fn malformed_case_op_returns_the_json_error_envelope() {
    let dir = scratch("ops-malformed-case");
    std::fs::write(dir.join("broken.case"), "case broken\n@\n").unwrap();
    let run = litigate_in(&dir, &["solve", "broken.case"]);
    assert_eq!(run.code, 2, "{}{}", run.stdout, run.stderr);
    let response = &run.json["result"];
    assert_eq!(response["ok"], false);
    assert_eq!(response["api_version"], 1);
    assert_eq!(response["op"], "solve");
    assert_eq!(response["error"]["code"], "invalid");
    assert!(response["error"]["hint"]
        .as_str()
        .unwrap()
        .contains("broken.case:"));
    assert!(response["error"]["message"].is_string());
    assert!(response["elapsed_ms"].is_number());
}

#[test]
fn explain_takes_a_path_and_op_arguments_or_a_code() {
    let node = response(
        &litigate(&["explain", ACME, "--arg", &format!("node={RULING}")]),
        "explain",
    );
    assert_eq!(node["result"]["node"]["id"], RULING);
    let code = litigate(&["explain", "L3004"]);
    assert_eq!(code.json["result"]["code"], "L3004");
}

#[test]
fn the_engine_manual_and_every_request_op_run_on_a_compiled_package() {
    // No raw JSON interface remains (lg-dfv4.7): the manual, the pack
    // list and a request's validation are ops on compiled source.
    let ran = |target: &str, op: &str| {
        let run = litigate(&["run", target, "--op", op]);
        assert_eq!(
            (run.code, &run.json["op"]),
            (0, &json!("run")),
            "{}",
            run.stdout
        );
        assert_eq!(run.json["result"]["op"], op);
        run.json["result"]["result"].clone()
    };
    let described = ran("lit/examples", "describe");
    assert_eq!(described["packs_source"], "litigate");
    let packs = ran(ACME, "packs");
    assert_eq!(packs[0]["id"], "frcp-rule12-example");
    let gone = litigate(&["engine", "describe"]);
    assert_eq!(gone.code, 1);
}
