use super::*;

fn parse(s: &str) -> Value {
    serde_json::from_str(s).unwrap_or(Value::Null)
}

/// `v` with its `elapsed_ms` zeroed.
fn untimed(mut v: Value) -> Value {
    v["elapsed_ms"] = json!(0);
    v
}

fn run(args: &[&str], files: &Value, with_library: bool) -> LitigateRun {
    let args = serde_json::to_string(args).unwrap_or_default();
    serde_json::from_str(&litigate(&args, &files.to_string(), with_library)).unwrap_or_default()
}

/// One matter (with its lock) from the embedded `cases/`.
fn matter(path: &str) -> Value {
    let cases = parse(&litigate_cases());
    json!({ path: cases[path], "cases/Lit.lock": cases["cases/Lit.lock"] })
}

// One test, so the embedded sources compile once.
#[test]
fn engine_requests_run_over_the_compiled_embedded_sources() {
    let request = r#"{"scenario":"cofc-1498-patent-case","op":{"op":"solve"}}"#;
    let r = parse(&handle(request));
    assert_eq!(r["ok"], json!(true), "{r}");
    assert!(r["result"]["value"].is_number());
    // The same op on the matter compiled alone.
    let path = "cases/cofc-1498-patent-case.case";
    let solved = run(&["solve", path], &matter(path), true);
    assert_eq!(solved.code, 0, "{}", solved.stderr);
    assert_eq!(untimed(parse(&solved.stdout)["result"].clone()), untimed(r));
    let manual = parse(&describe());
    assert_eq!(manual["op"], json!("describe"));
    assert_eq!(manual["result"]["packs_source"], json!("litigate"));
    let library = &manual["result"]["scenario_library"];
    assert!(library.as_array().is_some_and(|s| !s.is_empty()));
    assert_eq!(parse(&handle("not json"))["ok"], json!(false));
    assert_eq!(
        parse(&validate(r#"{"params":{"rate":1}}"#, None))["valid"],
        json!(true)
    );
    let pack = validate(r#"{"params":{"rate":1}}"#, Some("pack".into()));
    assert_eq!(parse(&pack)["valid"], json!(false));
    assert_eq!(parse(&validate("{", None))["ok"], json!(false));
}

#[test]
fn a_compile_failure_is_every_response() {
    let broken = Failure::new("lit: broken");
    let failed = handle_in(Err(&broken), "{}");
    assert_eq!(parse(&failed)["error"]["message"], json!("lit: broken"));
    let failed = validate_in(Err(&broken), "{}", None);
    assert_eq!(parse(&failed)["error"]["code"], json!("parse"));
    // An empty tree has no `lit/` and no `cases/`: nothing to serve.
    let empty = compile([]).map(|c| (c.packs.len(), c.scenarios.len()));
    assert_eq!(empty, Ok((0, 0)));
}

#[test]
fn schemas() {
    assert!(parse(&schema("scenario"))["title"].is_string());
    assert_eq!(parse(&schema("nope"))["ok"], json!(false));
}

#[test]
fn litigate_checks_builds_and_runs_over_the_embedded_library() {
    let path = "cases/frcp-civil-case.case";
    let files = matter(path);
    let checked = run(&["check", "cases"], &files, true);
    assert_eq!((checked.code, checked.stderr.as_str()), (0, ""));
    assert_eq!(parse(&checked.stdout)["ok"], json!(true));
    let built = run(
        &["build", "--out", "out", "lit/frcp", "cases"],
        &files,
        true,
    );
    assert_eq!(built.code, 0, "{}", built.stderr);
    assert!(built
        .files
        .contains_key("out/packs/frcp-civil-procedure.json"));
    assert!(built
        .files
        .contains_key("out/scenarios/frcp-civil-case.json"));
    let chain = run(&["chain", path, "--set", "rate=900"], &files, true);
    assert_eq!(chain.code, 0, "{}", chain.stderr);
    let envelope = parse(&chain.stdout);
    assert_eq!(
        (&envelope["op"], &envelope["result"]["ok"]),
        (&json!("chain"), &json!(true))
    );
    let engine = run(
        &[
            "engine",
            "q",
            r#"{"scenario":"frcp-civil-case","op":{"op":"solve"}}"#,
        ],
        &files,
        true,
    );
    assert_eq!(engine.code, 0, "{}", engine.stderr);
    assert_eq!(parse(&engine.stdout)["op"], json!("solve"));
    let library = parse(&litigate_library());
    assert_eq!(
        library
            .as_object()
            .map(|m| m.keys().all(|k| k.starts_with("lit/"))),
        Some(true)
    );
    assert!(library["lit/frcp/Lit.toml"].is_string());
    let cases = parse(&litigate_cases());
    assert_eq!(
        cases
            .as_object()
            .map(|m| m.keys().all(|k| k.starts_with("cases/"))),
        Some(true)
    );
    assert!(cases[path].is_string());
}

#[test]
fn litigate_files_override_the_library_and_failures_exit_1() {
    let broken = json!({ "lit/examples/rule12.lit": "state" });
    assert_eq!(run(&["check", "lit/examples"], &broken, true).code, 2);
    let alone = run(&["check", "lit"], &json!({}), false);
    assert_eq!(alone.code, 1);
    assert_eq!(
        alone.stderr,
        "litigate check: lit: no such file or directory\n"
    );
    let bad = |args: &str, files: &str| -> LitigateRun {
        serde_json::from_str(&litigate(args, files, true)).unwrap_or_default()
    };
    assert!(bad("{}", "{}").stderr.starts_with("litigate: args must be"));
    assert!(bad("[]", "[]")
        .stderr
        .starts_with("litigate: files must be"));
    assert_eq!(bad(r#"["explain", "L3004"]"#, "{}").code, 1);
}

#[test]
fn inline_packs() {
    let pack = json!({"schemaVersion": 2, "id": "t", "title": "t", "startNodeId": "a",
        "nodes": [{"id": "a", "label": "a"}, {"id": "b", "kind": "terminal", "label": "b", "payoff": 5}],
        "edges": [{"from": "a", "to": "b", "label": "go", "actor": "applicant", "hours": 0}]});
    let packs = json!({ "t.json": pack }).to_string();
    let r = parse(&handle_with_packs(r#"{"op":{"op":"solve"}}"#, &packs));
    assert_eq!(r["result"]["value"], json!(5.0));
    assert_eq!(parse(&handle_with_packs("{}", "[]"))["ok"], json!(false));
    assert_eq!(parse(&handle_with_packs("{}", "{"))["ok"], json!(false));
}
