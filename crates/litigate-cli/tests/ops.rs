//! One command line (lg-9lrj): every engine op is a subcommand on a
//! compiled matter or package (`litigate solve CASE`, `litigate explain
//! lit/pkg`), and `litigate engine` is the engine's raw JSON interface
//! over packs and named scenarios, compiled from the library unless a
//! directory is named.
// Test helpers below may unwrap: a panic is the failed assertion.
#![allow(clippy::unwrap_used)]
mod support;

use litigate_cli::engine::Builtin;
use serde_json::{json, Value};
use support::{example, litigate, litigate_in, repo, Run};

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

/// `litigate engine ARGS` at the repository root.
fn engine(args: &[&str]) -> Run {
    let mut all = vec!["engine"];
    all.extend_from_slice(args);
    litigate(&all)
}

/// The engine over the worked example only, which compiles quickly.
const EXAMPLE: [&str; 5] = [
    "--lib",
    "lit/examples",
    "--cases",
    "cases/examples",
    "--compact",
];

#[test]
fn the_engine_compiles_its_catalog_from_the_library() {
    let mut args = vec!["describe"];
    args.extend(EXAMPLE);
    let described = engine(&args);
    assert_eq!(described.code, 0, "{}", described.stderr);
    let result = &described.json["result"];
    assert_eq!(result["packs_source"], "litigate");
    assert_eq!(result["scenarios_source"], "litigate");
    let mut args = vec!["solve", "--scenario", "acme-v-widget", "--set", "rate=1"];
    args.extend(EXAMPLE);
    let solved = engine(&args);
    assert_eq!(
        (solved.code, &solved.json["ok"]),
        (0, &json!(true)),
        "{}",
        solved.stdout
    );
}

#[test]
fn the_engine_reads_matters_from_cases_by_default() {
    let dir = example("ops-engine-cases", "", None);
    let run = litigate_in(
        &dir,
        &["engine", "describe", "--lib", "lit/examples", "--compact"],
    );
    assert_eq!(run.code, 0, "{}", run.stderr);
    let library = &run.json["result"]["scenario_library"];
    assert!(library.to_string().contains("acme-v-widget"), "{library}");
    let missing = engine(&[
        "describe",
        "--lib",
        "lit/examples",
        "--cases",
        "no/such/cases",
    ]);
    assert_eq!(missing.code, 1);
    assert_eq!(
        missing.stderr,
        "litigate engine: no/such/cases: no such file or directory\n"
    );
}

#[test]
fn the_litgraph_shim_serves_the_packs_and_scenarios_built_into_it() {
    let printed = litigate_cli::engine::execute(
        ["litgraph", "describe", "--compact"],
        Builtin::Embedded(litgraph_packs::PACKS, litgraph_packs::SCENARIOS),
    );
    assert_eq!(printed.code, 0, "{}", printed.stderr);
    let v: Value = serde_json::from_str(&printed.stdout).unwrap();
    assert_eq!(v["result"]["packs_source"], "embedded", "{v}");
    assert_eq!(v["result"]["scenarios_source"], "embedded");
    let library = &v["result"]["scenario_library"];
    assert!(
        library.to_string().contains("cofc-1498-patent-case"),
        "{library}"
    );
}

#[test]
fn named_directories_replace_the_compiled_halves() {
    let packs = litgraph_packs::PACKS_DIR;
    let scenarios = litgraph_packs::SCENARIOS_DIR;
    let both = engine(&[
        "describe",
        "--packs-dir",
        packs,
        "--scenarios-dir",
        scenarios,
    ]);
    assert_eq!(both.json["result"]["packs_source"], packs);
    assert_eq!(both.json["result"]["scenarios_source"], scenarios);
    let mut args = vec!["describe", "--packs-dir", packs];
    args.extend(EXAMPLE);
    let half = engine(&args);
    assert_eq!(half.json["result"]["packs_source"], packs);
    assert_eq!(half.json["result"]["scenarios_source"], "litigate");
    let mut args = vec!["describe", "--scenarios-dir", scenarios];
    args.extend(EXAMPLE);
    let other = engine(&args);
    assert_eq!(other.json["result"]["packs_source"], "litigate");
    assert_eq!(other.json["result"]["scenarios_source"], scenarios);
    let missing = engine(&["describe", "--packs-dir", "/no/such/packs"]);
    assert_eq!(missing.code, 1);
    assert!(
        missing
            .stderr
            .starts_with("litigate engine: loading packs and scenarios: ")
            && missing.stderr.contains("--packs-dir: /no/such/packs"),
        "{}",
        missing.stderr
    );
}

#[test]
fn variables_name_directories_too() {
    let out = assert_cmd::Command::cargo_bin("litigate")
        .unwrap()
        .current_dir(repo())
        .env("LITGRAPH_PACKS", litgraph_packs::PACKS_DIR)
        .env("LITGRAPH_SCENARIOS", litgraph_packs::SCENARIOS_DIR)
        .args(["engine", "q", "-", "--compact"])
        .write_stdin(r#"{"scenario":"cofc-1498-patent-case","op":{"op":"chain"}}"#)
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    let v: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v["op"], "chain");
}

#[test]
fn standard_input_that_is_not_text_is_an_error() {
    let out = assert_cmd::Command::cargo_bin("litigate")
        .unwrap()
        .current_dir(repo())
        .args(["engine", "q", "-", "--packs-dir", litgraph_packs::PACKS_DIR])
        .args(["--scenarios-dir", litgraph_packs::SCENARIOS_DIR])
        .write_stdin(vec![0xff, 0xfe])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1));
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.starts_with("litigate engine: "), "{err}");
}

#[test]
fn engine_requests_documents_and_schemas() {
    let dirs = [
        "--packs-dir",
        litgraph_packs::PACKS_DIR,
        "--scenarios-dir",
        litgraph_packs::SCENARIOS_DIR,
        "--calibration-dir",
        "calibration",
    ];
    let with = |args: &[&str]| {
        let mut all = args.to_vec();
        all.extend(dirs);
        engine(&all)
    };
    let ok = with(&["q", r#"{"packs":["cofc"],"op":{"op":"solve"}}"#]);
    assert_eq!((ok.code, &ok.json["ok"]), (0, &json!(true)));
    let failed = with(&[
        "run",
        r#"{"scenario":"no-such-scenario","op":{"op":"solve"}}"#,
    ]);
    assert_eq!((failed.code, &failed.json["ok"]), (2, &json!(false)));
    assert_eq!(with(&["validate", r#"{"params":{"rate":1}}"#]).code, 0);
    assert_eq!(with(&["validate", r#"{"not_a_real_field":1}"#]).code, 2);
    let not_json = with(&["validate", "{not json"]);
    assert!(not_json
        .stderr
        .starts_with("litigate engine: document is not JSON"));
    let missing = with(&["run", "no/such/request.json"]);
    assert!(missing
        .stderr
        .starts_with("litigate engine: reading no/such/request.json"));
    let unread = with(&["validate", "no/such/document.json"]);
    assert_eq!(unread.code, 1);
    assert!(unread
        .stderr
        .starts_with("litigate engine: reading no/such/document.json"));
    let schema = engine(&["schema", "--compact"]);
    assert_eq!(schema.code, 0);
    assert!(schema.json.is_object());
    assert_eq!(engine(&["schema", "no-such-kind"]).code, 1);
    let shorthand = with(&["solve", "stray"]);
    assert_eq!(shorthand.code, 1);
    assert!(shorthand.stderr.contains("unexpected argument `stray`"));
}

#[test]
fn engine_help_and_usage_errors() {
    let help = engine(&["--help"]);
    assert_eq!((help.code, help.stderr.as_str()), (0, ""));
    assert!(
        help.stdout.contains("Usage: litigate engine"),
        "{}",
        help.stdout
    );
    let bad = engine(&["describe", "--frobnicate"]);
    assert_eq!((bad.code, bad.stdout.as_str()), (1, ""));
    assert!(bad.stderr.contains("--frobnicate"), "{}", bad.stderr);
}
