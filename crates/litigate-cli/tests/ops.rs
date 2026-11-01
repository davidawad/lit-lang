//! One command line (lg-9lrj): every engine op is a subcommand on a
//! compiled matter or package (`litigate solve CASE`, `litigate explain
//! lit/pkg`); the engine has no raw JSON interface (lg-dfv4.7).
// Test helpers below may unwrap: a panic is the failed assertion.
#![allow(clippy::unwrap_used)]
mod support;

use serde_json::{json, Value};
use support::{example, litigate, litigate_in, Run};

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
