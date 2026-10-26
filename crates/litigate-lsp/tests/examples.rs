//! Every SPEC example through the protocol: the §12 worked example
//! (checked as a workspace) and the syntax fixtures that use every other
//! construct (open documents outside any package, parsed only). Each
//! parses cleanly, has symbols, formats idempotently, and every declared
//! name hovers and resolves to itself.
#![allow(clippy::unwrap_used, clippy::expect_used)]

mod support;

use serde_json::{json, Value};
use support::{read, Client};

/// SPEC §12, then the construct fixtures (`links`, instances,
/// transitions, defaults, exceptions: what §12 leaves out).
const EXAMPLES: [&str; 6] = [
    "lit/examples/rule12.lit",
    "cases/examples/acme.case",
    "lit/examples/Lit.toml",
    "crates/litigate-syntax/tests/fixtures/constructs.lit",
    "crates/litigate-syntax/tests/fixtures/links.lit",
    "crates/litigate-syntax/tests/fixtures/matter.case",
];

fn flatten(symbols: &Value, out: &mut Vec<Value>) {
    for s in symbols.as_array().into_iter().flatten() {
        out.push(s.clone());
        flatten(&s["children"], out);
    }
}

async fn formatting(c: &mut Client, path: &str) -> Value {
    c.request(
        "textDocument/formatting",
        json!({"textDocument": {"uri": c.uri(path)}, "options": {"tabSize": 2, "insertSpaces": true}}),
    )
    .await
}

#[tokio::test]
async fn every_example_is_served() {
    let mut c = Client::examples("examples-served", &EXAMPLES[3..]).await;
    for path in EXAMPLES {
        let text = read(path);
        c.open(path, &text).await;
        let symbols = c
            .request(
                "textDocument/documentSymbol",
                json!({"textDocument": {"uri": c.uri(path)}}),
            )
            .await;
        let mut all = Vec::new();
        flatten(&symbols, &mut all);
        assert!(!all.is_empty(), "{path}: no symbols");
        for s in &all {
            let start = s["selectionRange"]["start"].clone();
            let hover = c.at("hover", path, start.clone()).await;
            assert!(
                hover["contents"]["value"].is_string(),
                "{path}: no hover for {}",
                s["name"]
            );
            let found = c.at("definition", path, start.clone()).await;
            assert_eq!(found["uri"], c.uri(path), "{path}: {}", s["name"]);
            assert_eq!(found["range"]["start"], start, "{path}: {}", s["name"]);
        }
        if path.ends_with("Lit.toml") {
            continue;
        }
        let edits = formatting(&mut c, path).await;
        if let Some(edit) = edits.as_array().and_then(|e| e.first()) {
            let formatted = edit["newText"].as_str().unwrap().to_owned();
            c.notify(
                "textDocument/didChange",
                json!({"textDocument": {"uri": c.uri(path), "version": 2},
                       "contentChanges": [{"text": formatted}]}),
            )
            .await;
            assert_eq!(
                formatting(&mut c, path).await,
                json!([]),
                "{path}: fmt is idempotent"
            );
        }
    }
    let _ = c.status().await;
    // The worked example checks without errors (SPEC §12); the syntax
    // fixtures parse cleanly (`matter.case` is a matter, so it is checked,
    // and the packages it names are not in this workspace).
    let worked: Vec<String> = EXAMPLES[..3].iter().map(|p| c.uri(p)).collect();
    let errors: Vec<&Value> = c
        .notes
        .iter()
        .filter(|m| m["method"] == "textDocument/publishDiagnostics")
        .flat_map(|m| {
            let worked = worked.iter().any(|u| m["params"]["uri"] == *u);
            m["params"]["diagnostics"]
                .as_array()
                .into_iter()
                .flatten()
                .filter(move |d| {
                    let parse = d["code"].as_str().is_some_and(|c| c.starts_with("L0"));
                    d["severity"] == 1 && (worked || parse)
                })
        })
        .collect();
    assert!(errors.is_empty(), "{errors:?}");
}

#[tokio::test]
async fn a_loose_document_gets_its_parse_diagnostics() {
    let mut c = Client::examples("examples-loose", &EXAMPLES[3..4]).await;
    let path = "crates/litigate-syntax/tests/fixtures/constructs.lit";
    let broken = read(path).replacen("->", "", 1);
    c.open(path, &broken).await;
    let ds = c.diagnostics(&c.uri(path)).await;
    assert!(ds.iter().any(|d| d["code"] == "L0001"), "{ds:?}");
    c.notify(
        "textDocument/didClose",
        json!({"textDocument": {"uri": c.uri(path)}}),
    )
    .await;
    assert_eq!(
        c.diagnostics(&c.uri(path)).await,
        Vec::<Value>::new(),
        "closing clears them"
    );
}
