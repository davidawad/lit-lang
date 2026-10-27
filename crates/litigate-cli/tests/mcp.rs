//! End to end: the `litigate` binary's `mcp` subcommand driven over stdio
//! by a real MCP client (`rmcp`): `initialize`, `tools/list`, an op on a
//! compiled matter, the authoring tools and the resources. There is no
//! raw-request tool: every op runs on compiled source (lg-dfv4.7).
#![allow(clippy::expect_used)]
use rmcp::model::{
    CallToolRequestParams, CallToolResult, ReadResourceRequestParams, ResourceContents,
};
use rmcp::service::RunningService;
use rmcp::transport::{ConfigureCommandExt, TokioChildProcess};
use rmcp::{RoleClient, ServiceExt};
use serde_json::{json, Map, Value};
use std::path::{Path, PathBuf};
use tokio::process::Command;

type Client = RunningService<RoleClient, ()>;

const FIXTURE_LIB: &str = "tests/fixtures/cov-cmds/lit";
const FIXTURE_CASES: &str = "tests/fixtures/cov-cmds/cases";
const FIXTURE_CASE: &str = "tests/fixtures/cov-cmds/cases/cov.case";

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// `litigate mcp` in this crate's directory over the coverage fixture,
/// `configure`d before it starts.
async fn connect(configure: impl FnOnce(&mut Command)) -> Client {
    let command = Command::new(env!("CARGO_BIN_EXE_litigate")).configure(|cmd| {
        cmd.current_dir(env!("CARGO_MANIFEST_DIR")).args([
            "mcp",
            "--lib",
            FIXTURE_LIB,
            "--cases",
            FIXTURE_CASES,
        ]);
        configure(cmd);
    });
    let transport = TokioChildProcess::new(command).expect("spawn litigate mcp");
    ().serve(transport).await.expect("initialize litigate mcp")
}

async fn call(client: &Client, name: &str, args: Value) -> CallToolResult {
    let args: Map<String, Value> = args.as_object().cloned().unwrap_or_default();
    let params = CallToolRequestParams::new(name.to_owned()).with_arguments(args);
    client.call_tool(params).await.expect("tools/call")
}

fn structured(result: &CallToolResult) -> Value {
    result.structured_content.clone().unwrap_or_default()
}

async fn read(client: &Client, uri: &str) -> Option<Value> {
    let read = client
        .read_resource(ReadResourceRequestParams::new(uri))
        .await
        .ok()?;
    match &read.contents[0] {
        ResourceContents::TextResourceContents { text, .. } => serde_json::from_str(text).ok(),
        _ => None,
    }
}

#[tokio::test]
async fn initialize_advertises_tools_and_resources() {
    let client = connect(|_| {}).await;
    let info = client.peer_info().expect("peer info");
    assert!(info.capabilities.tools.is_some());
    assert!(info.capabilities.resources.is_some());
    assert!(info.instructions.is_some());
    client.cancel().await.expect("cancel");
}

#[tokio::test]
async fn every_op_and_authoring_tool_is_listed() {
    let client = connect(|_| {}).await;
    let tools = client.list_all_tools().await.expect("tools/list");
    let names: Vec<&str> = tools.iter().map(|t| t.name.as_ref()).collect();
    let ops = [
        "describe",
        "packs",
        "lint",
        "validate",
        "graph",
        "metric",
        "explain",
        "solve",
        "chain",
        "simulate",
        "path",
        "pareto",
        "sweep",
        "tornado",
        "calibration",
        "posterior",
        "voi",
        "structure",
        "deadlines",
        "settle",
        "compare",
        "batch",
    ];
    let authoring = [
        "litigate_describe",
        "litigate_check",
        "litigate_build",
        "litigate_where",
        "litigate_timeline",
        "litigate_explain",
    ];
    for name in ops.iter().chain(&authoring) {
        assert!(names.contains(name), "missing `{name}` in {names:?}");
    }
    assert!(!names.contains(&"litigate_engine"), "{names:?}");
    let solve = tools.iter().find(|t| t.name == "solve").expect("solve");
    let props = solve.input_schema["properties"]
        .as_object()
        .cloned()
        .unwrap_or_default();
    assert!(
        props.contains_key("target") && props.contains_key("max_steps"),
        "{props:?}"
    );
    client.cancel().await.expect("cancel");
}

#[tokio::test]
async fn every_op_tool_runs_on_a_compiled_matter() {
    let client = connect(|_| {}).await;
    let args = |extra: Value| {
        let mut args = json!({"target": FIXTURE_CASE});
        args.as_object_mut().into_iter().for_each(|a| {
            a.extend(extra.as_object().cloned().unwrap_or_default());
        });
        args
    };
    let calls = [
        ("describe", json!({})),
        ("packs", json!({})),
        ("lint", json!({})),
        ("validate", json!({})),
        ("graph", json!({})),
        ("metric", json!({"spec": "dollars"})),
        ("explain", json!({})),
        ("solve", json!({})),
        ("chain", json!({})),
        ("simulate", json!({"runs": 50, "seed": 1})),
        ("path", json!({})),
        ("pareto", json!({})),
        (
            "sweep",
            json!({"param": "rate", "lo": 100, "hi": 200, "steps": 2}),
        ),
        ("tornado", json!({})),
        ("calibration", json!({})),
        ("posterior", json!({"samples": 20, "seed": 1})),
        ("voi", json!({"samples": 20, "seed": 1})),
        ("structure", json!({})),
        ("deadlines", json!({"trigger": "2025-03-05"})),
        ("settle", json!({"runs": 20, "seed": 1})),
        ("compare", json!({"variant": {"params": {"rate": 900}}})),
        ("batch", json!({"ops": [{"op": "solve"}]})),
    ];
    for (op, extra) in calls {
        let result = call(&client, op, args(extra)).await;
        let envelope = structured(&result);
        // The op ran: the engine answered as `result`, ok or not.
        assert_eq!(envelope["result"]["op"], json!(op), "{op}: {envelope}");
        assert_eq!(
            result.is_error,
            Some(envelope["result"]["ok"] != json!(true)),
            "{op}"
        );
    }
    let solved = structured(&call(&client, "solve", args(json!({"set": {"rate": 900}}))).await);
    assert!(solved["result"]["result"]["value"].is_number(), "{solved}");
    let bad = call(&client, "solve", args(json!({"not_a_real_field": 1}))).await;
    assert_eq!(bad.is_error, Some(true));
    assert_eq!(structured(&bad)["ok"], json!(false));
    client.cancel().await.expect("cancel");
}

#[tokio::test]
async fn resources_serve_the_compiled_library() {
    let client = connect(|_| {}).await;
    let resources = client.list_all_resources().await.expect("resources/list");
    let uris: Vec<&str> = resources.iter().map(|r| r.uri.as_str()).collect();
    assert_eq!(
        uris,
        [
            "litigate://describe",
            "litigate://links",
            "litigate://scenarios",
            "litigate://packs/cov-pack",
            "litigate://scenarios/cov-matter"
        ]
    );
    let index = read(&client, "litigate://scenarios")
        .await
        .unwrap_or_default();
    assert_eq!(index["scenarios"][0]["id"], json!("cov-matter"));
    let matter = read(&client, "litigate://scenarios/cov-matter")
        .await
        .unwrap_or_default();
    assert!(matter["scenario"].is_object(), "{matter}");
    assert!(read(&client, "litigate://links").await.is_some());
    assert!(read(&client, "litigate://scenarios/no-such-matter")
        .await
        .is_none());
    client.cancel().await.expect("cancel");
}

#[tokio::test]
async fn a_library_that_does_not_load_is_a_protocol_error_not_a_crash() {
    let client = connect(|cmd| {
        cmd.args(["--cases", "/no/such/litigate-cases-dir"]);
    })
    .await;
    assert!(client.list_all_resources().await.is_err());
    // The server is still up: an op on a matter compiles it afresh.
    let solved = call(&client, "solve", json!({"target": FIXTURE_CASE})).await;
    assert_eq!(
        solved.is_error,
        Some(false),
        "{:?}",
        solved.structured_content
    );
    client.cancel().await.expect("cancel");
}

#[tokio::test]
async fn the_authoring_tools_run_over_the_repository() {
    let client = connect(|cmd| {
        cmd.current_dir(root());
    })
    .await;
    let envelope = structured(&call(&client, "litigate_describe", json!({})).await);
    assert!(envelope["result"]["diagnostic_codes"].is_array());
    let checked = call(
        &client,
        "litigate_check",
        json!({"paths": ["lit/examples"]}),
    )
    .await;
    assert_eq!(
        structured(&checked)["ok"],
        json!(true),
        "{:?}",
        checked.structured_content
    );
    let found = structured(
        &call(
            &client,
            "litigate_where",
            json!({"id": "answer-duty", "paths": ["lit/examples"]}),
        )
        .await,
    );
    assert_eq!(
        found["result"]["declaration_span"]["file"],
        json!("lit/examples/rule12.lit"),
        "{found}"
    );
    let timeline = structured(
        &call(
            &client,
            "litigate_timeline",
            json!({"file": "cases/examples/acme.case"}),
        )
        .await,
    );
    assert_eq!(
        timeline["result"]["schema"],
        json!("litigate.timeline/1"),
        "{timeline}"
    );
    let explained = structured(&call(&client, "litigate_explain", json!({"code": "L3004"})).await);
    assert_eq!(explained["result"]["code"], json!("L3004"));
    let out = std::env::temp_dir().join(format!("litigate-mcp-build-{}", std::process::id()));
    let built = json!({"paths": ["lit/examples"], "out": out.display().to_string(), "pack_files": ["frcp-rule12-example=rule12"]});
    let built = structured(&call(&client, "litigate_build", built).await);
    assert_eq!(built["ok"], json!(true), "{built}");
    assert!(out.join("packs/rule12.json").is_file());
    let _ = std::fs::remove_dir_all(&out);
    let bad = call(
        &client,
        "litigate_check",
        json!({"paths": ["lit"], "bogus": 1}),
    )
    .await;
    assert_eq!(bad.is_error, Some(true));
    let message = structured(&bad)["error"]["message"].clone();
    assert!(
        message
            .as_str()
            .unwrap_or_default()
            .contains("unknown field `bogus`"),
        "{message}"
    );
    client.cancel().await.expect("cancel");
}
