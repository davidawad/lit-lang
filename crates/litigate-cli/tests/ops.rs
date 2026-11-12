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

fn itc_explain(node: &str) -> Value {
    let arg = format!("node={node}");
    response(&litigate(&["explain", "lit/itc", "--arg", &arg]), "explain")["result"].clone()
}

fn itc_outcome(terminal: &str) -> Vec<String> {
    let arg = format!("node={terminal}");
    let graph = response(&litigate(&["graph", "lit/itc", "--arg", &arg]), "graph");
    graph["result"]["nodes"]
        .as_array()
        .unwrap()
        .iter()
        .find(|node| node["id"].as_str().is_some_and(|id| id.ends_with(terminal)))
        .unwrap_or_else(|| panic!("{terminal} missing from {graph}"))["outcome"]
        .as_array()
        .unwrap()
        .iter()
        .map(|tag| tag.as_str().unwrap().to_owned())
        .collect()
}

fn has_terminal(option: &Value, terminal: &str) -> bool {
    option["then"].as_array().is_some_and(|xs| {
        xs.iter().any(|x| {
            x["terminal"]
                .as_str()
                .is_some_and(|id| id.starts_with(terminal))
        })
    })
}

#[test]
fn itc_violation_without_appeal_reaches_the_violation_win_terminal() {
    let explain = itc_explain("federal-circuit-appeal-window{violation-found}");
    let options = explain["options"].as_array().unwrap();
    assert!(options
        .iter()
        .any(|option| { has_terminal(option, "determination-final-unappealed-violation") }));
    assert_eq!(
        itc_outcome("determination-final-unappealed-violation"),
        ["final", "win"]
    );
}

#[test]
fn itc_no_violation_without_appeal_reaches_the_no_violation_loss_terminal() {
    let explain = itc_explain("federal-circuit-appeal-window{no-violation}");
    let options = explain["options"].as_array().unwrap();
    assert!(options
        .iter()
        .any(|option| { has_terminal(option, "determination-final-unappealed-no-violation") }));
    assert_eq!(
        itc_outcome("determination-final-unappealed-no-violation"),
        ["final", "loss"]
    );
}

#[test]
fn itc_violation_affirmance_reaches_the_violation_win_terminal() {
    let review = itc_explain("federal-circuit-review{violation-found}");
    let affirm = review["options"]
        .as_array()
        .unwrap()
        .iter()
        .find(|option| {
            option["to"]
                .as_str()
                .is_some_and(|id| id.contains("cafc-affirmed-result"))
        })
        .unwrap();
    assert!(has_terminal(affirm, "cafc-affirmed-violation"));
    assert_eq!(
        itc_outcome("cafc-affirmed-violation"),
        ["affirmed", "final", "win"]
    );
}

#[test]
fn itc_no_violation_affirmance_reaches_the_no_violation_loss_terminal() {
    let review = itc_explain("federal-circuit-review{no-violation}");
    let affirm = review["options"]
        .as_array()
        .unwrap()
        .iter()
        .find(|option| {
            option["to"]
                .as_str()
                .is_some_and(|id| id.contains("cafc-affirmed-result"))
        })
        .unwrap();
    assert!(has_terminal(affirm, "cafc-affirmed-no-violation"));
    assert_eq!(
        itc_outcome("cafc-affirmed-no-violation"),
        ["affirmed", "final", "loss"]
    );
}

#[test]
fn itc_appeal_and_affirmance_copies_follow_the_result_flag() {
    for (flag, allowed, forbidden) in [
        ("violation-found", "violation", "no-violation"),
        ("no-violation", "no-violation", "violation"),
    ] {
        let appeal = itc_explain(&format!("federal-circuit-appeal-window{{{flag}}}"));
        let appeal_options = appeal["options"].as_array().unwrap();
        assert!(appeal_options.iter().any(|option| {
            option["to"]
                .as_str()
                .is_some_and(|id| id.contains(&format!("determination-final-unappealed-{allowed}")))
        }));
        assert!(!appeal_options.iter().any(|option| {
            option["to"].as_str().is_some_and(|id| {
                id.contains(&format!("determination-final-unappealed-{forbidden}"))
            })
        }));

        let review = itc_explain(&format!("federal-circuit-review{{{flag}}}"));
        let affirm = review["options"]
            .as_array()
            .unwrap()
            .iter()
            .find(|option| {
                option["to"]
                    .as_str()
                    .is_some_and(|id| id.contains("cafc-affirmed-result"))
            })
            .unwrap();
        assert!(has_terminal(affirm, &format!("cafc-affirmed-{allowed}")));
        assert!(!has_terminal(affirm, &format!("cafc-affirmed-{forbidden}")));
    }
}

#[test]
fn itc_remand_clears_the_first_result_and_allows_a_different_second_result() {
    let review = itc_explain("federal-circuit-review{violation-found}");
    let remand = review["options"]
        .as_array()
        .unwrap()
        .iter()
        .find(|option| {
            option["to"]
                .as_str()
                .is_some_and(|id| id.contains("cafc-remanded"))
        })
        .unwrap();
    assert!(has_terminal(remand, "cafc-affirmed-no-violation"));
    assert!(has_terminal(
        remand,
        "determination-final-unappealed-violation"
    ));
    let return_to_merits = itc_explain("cafc-remanded");
    assert!(return_to_merits["options"]
        .as_array()
        .unwrap()
        .iter()
        .any(|option| {
            option["to"]
                .as_str()
                .is_some_and(|id| id.contains("commission-final-determination"))
        }));
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
fn cafc_cert_terminals_preserve_panel_direction_and_neutral_stakes() {
    let out = scratch("cafc-cert-terminal-split");
    let built = litigate(&["build", "lit/cafc", "--out", out.to_str().unwrap()]);
    assert_eq!(built.code, 0, "{}{}", built.stdout, built.stderr);
    let pack: Value =
        serde_json::from_slice(&std::fs::read(out.join("packs/cafc.json")).unwrap()).unwrap();

    let node = |id: &str| {
        pack["nodes"]
            .as_array()
            .unwrap()
            .iter()
            .find(|node| node["id"] == id)
            .unwrap_or_else(|| panic!("missing CAFC node {id}"))
    };
    let edge = |id: &str| {
        pack["edges"]
            .as_array()
            .unwrap()
            .iter()
            .find(|edge| edge["id"] == id)
            .unwrap_or_else(|| panic!("missing CAFC edge {id}"))
    };
    let has = |values: &Value, value: &str| {
        values
            .as_array()
            .is_some_and(|items| items.iter().any(|item| item == value))
    };

    let neutral_payoffs = json!({
        "panel-affirmed": 0,
        "panel-mixed": 250_000,
        "panel-reversed": 1_000_000,
    });
    for (neutral, loss, win) in [
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
    ] {
        assert_eq!(node(neutral)["outcome"], json!(["final"]));
        assert_eq!(node(neutral)["payoff"], json!(0));
        assert_eq!(node(neutral)["payoffByFlag"], neutral_payoffs);
        assert_eq!(node(loss)["outcome"], json!(["final", "loss"]));
        assert_eq!(node(loss)["payoff"], json!(0));
        assert_eq!(node(win)["outcome"], json!(["final", "win"]));
        assert_eq!(node(win)["payoff"], json!(1_000_000));
    }
    for id in ["cert-window-lapse-affirmed", "cert-review-deny-affirmed"] {
        assert!(has(&edge(id)["requires"], "panel-affirmed"), "{id}");
        assert!(has(&edge(id)["forbids"], "panel-reversed"), "{id}");
    }
    for id in ["cert-window-lapse-reversed", "cert-review-deny-reversed"] {
        assert!(has(&edge(id)["requires"], "panel-reversed"), "{id}");
    }
    for (id, target) in [
        (
            "cert-window-lapse-affirmed",
            "cert-not-sought-after-affirmance",
        ),
        (
            "cert-window-lapse-reversed",
            "cert-not-sought-after-reversal",
        ),
        ("cert-window-lapse-neutral", "cert-not-sought"),
        ("cert-review-deny-affirmed", "cert-denied-after-affirmance"),
        ("cert-review-deny-reversed", "cert-denied-after-reversal"),
        ("cert-review-deny-neutral", "cert-denied"),
    ] {
        assert_eq!(edge(id)["to"], target, "{id}");
    }
    for id in ["cert-window-lapse-neutral", "cert-review-deny-neutral"] {
        assert!(has(&edge(id)["forbids"], "panel-affirmed"), "{id}");
        assert!(has(&edge(id)["forbids"], "panel-reversed"), "{id}");
    }
    assert_eq!(edge("cert-window-lapse-affirmed")["actor"], "appellant");
    assert_eq!(edge("cert-window-lapse-reversed")["actor"], "appellee");
    assert_eq!(
        edge("cert-window-lapse-affirmed")["authority"],
        "Sup. Ct. R. 13.1"
    );
    assert_eq!(
        edge("cert-window-lapse-reversed")["authority"],
        "Sup. Ct. R. 13.1"
    );
    assert_eq!(
        edge("cert-review-deny-affirmed")["authority"],
        "Sup. Ct. R. 10"
    );
    assert_eq!(
        edge("cert-review-deny-reversed")["authority"],
        "Sup. Ct. R. 10"
    );
    assert_eq!(
        edge("cert-review-deny-neutral")["authority"],
        "Sup. Ct. R. 10"
    );
    for id in [
        "cert-review-deny-affirmed",
        "cert-review-deny-reversed",
        "cert-review-deny-neutral",
    ] {
        assert_eq!(edge(id)["probability"], json!(0.99), "{id}");
    }
    assert_eq!(edge("cert-review-grant")["probability"], json!(0.01));

    for (id, flag, cleared) in [
        (
            "panel-to-rule36",
            "panel-affirmed",
            ["panel-reversed", "panel-mixed"],
        ),
        (
            "panel-to-affirmed",
            "panel-affirmed",
            ["panel-reversed", "panel-mixed"],
        ),
        (
            "panel-to-reversed",
            "panel-reversed",
            ["panel-affirmed", "panel-mixed"],
        ),
        (
            "panel-to-mixed",
            "panel-mixed",
            ["panel-affirmed", "panel-reversed"],
        ),
    ] {
        assert!(has(&edge(id)["sets"], flag), "{id}");
        for other in cleared {
            assert!(has(&edge(id)["clears"], other), "{id} must clear {other}");
        }
    }
    for id in ["panel-to-remand", "rehearing-granted-to-decision"] {
        for flag in ["panel-affirmed", "panel-reversed", "panel-mixed"] {
            assert!(has(&edge(id)["clears"], flag), "{id} must clear {flag}");
        }
    }
    assert!(edge("rehearing-denied-edge")["clears"].is_null());
    assert!(edge("panel-to-dismissed")["sets"].is_null());
    assert!(edge("panel-to-dismissed")["clears"].is_null());
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
