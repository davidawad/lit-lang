//! The server's edges over the wire: requests before the first analysis,
//! documents that are not files, files that are not there, and the
//! notifications that only re-check.
#![allow(clippy::unwrap_used, clippy::expect_used)]

mod support;

use serde_json::{json, Value};
use std::path::Path;
use support::Client;
use tower_lsp::lsp_types::Url;

const UNTITLED: &str = "untitled:Untitled-1";

fn file_uri(path: &Path) -> String {
    Url::from_file_path(path).unwrap().to_string()
}

fn position(uri: &str, line: u32, character: u32) -> Value {
    json!({"textDocument": {"uri": uri}, "position": {"line": line, "character": character},
           "context": {"includeDeclaration": false}})
}

/// Every per-document request at `uri`, `(method, result)`.
async fn ask_everything(c: &mut Client, uri: &str) -> Vec<(String, Value)> {
    let doc = json!({"textDocument": {"uri": uri}});
    let range = json!({"start": {"line": 0, "character": 0}, "end": {"line": 9, "character": 0}});
    let mut with_range = doc.clone();
    with_range["range"] = range;
    with_range["context"] = json!({"diagnostics": []});
    let mut formatting = doc.clone();
    formatting["options"] = json!({"tabSize": 2, "insertSpaces": true});
    let requests = [
        ("textDocument/hover", position(uri, 0, 0)),
        ("textDocument/definition", position(uri, 0, 0)),
        ("textDocument/references", position(uri, 0, 0)),
        ("textDocument/documentSymbol", doc.clone()),
        ("textDocument/codeAction", with_range.clone()),
        ("textDocument/inlayHint", with_range),
        ("textDocument/formatting", formatting),
    ];
    let mut out = Vec::new();
    for (method, params) in requests {
        out.push((method.to_owned(), c.request(method, params).await));
    }
    out
}

fn nulls(answers: &[(String, Value)]) -> Vec<&str> {
    answers
        .iter()
        .filter(|(_, v)| v.is_null())
        .map(|(m, _)| m.as_str())
        .collect()
}

#[tokio::test]
async fn before_the_first_analysis_there_is_nothing_to_answer() {
    let root = support::worked("server-early", &[]);
    let (mut c, _) = Client::initialize(
        json!({"processId": null, "rootUri": file_uri(&root), "capabilities": {}}),
    )
    .await;
    let status = c.status().await;
    assert_eq!(status, json!({"rebuilds": 0, "files": []}));
    let module = file_uri(&root.join("lit/ex/rule12.lit"));
    let answers = ask_everything(&mut c, &module).await;
    // Formatting reads the open documents, not the analysis.
    assert_eq!(nulls(&answers).len(), answers.len(), "{answers:?}");
    let found = c.request("workspace/symbol", json!({"query": ""})).await;
    assert_eq!(found, Value::Null);
}

#[tokio::test]
async fn the_first_workspace_folder_is_the_root() {
    let root = support::worked("server-folders", &[]);
    let folder = json!([{"uri": file_uri(&root), "name": "ws"}]);
    let (mut c, _) = Client::initialize(
        json!({"processId": null, "rootUri": null, "workspaceFolders": folder, "capabilities": {}}),
    )
    .await;
    c.notify("initialized", json!({})).await;
    let files = c.status().await["files"].clone();
    assert_eq!(
        files,
        json!(["cases/acme.case", "lit/ex/Lit.toml", "lit/ex/rule12.lit"])
    );
}

#[tokio::test]
async fn documents_that_are_not_files_are_ignored() {
    let root = support::worked("server-untitled", &[]);
    let (mut c, _) = Client::start(&root, json!({})).await;
    let before = c.status().await;
    c.notify(
        "textDocument/didOpen",
        json!({"textDocument": {"uri": UNTITLED, "languageId": "litigate", "version": 1, "text": "module x\n"}}),
    )
    .await;
    c.notify(
        "textDocument/didChange",
        json!({"textDocument": {"uri": UNTITLED, "version": 2}, "contentChanges": [{"text": "module y\n"}]}),
    )
    .await;
    c.notify(
        "textDocument/didClose",
        json!({"textDocument": {"uri": UNTITLED}}),
    )
    .await;
    let answers = ask_everything(&mut c, UNTITLED).await;
    assert_eq!(nulls(&answers).len(), answers.len(), "{answers:?}");
    assert_eq!(c.status().await["files"], before["files"]);
}

#[tokio::test]
async fn files_that_are_not_there_have_no_answers() {
    let root = support::worked("server-missing", &[]);
    let (mut c, _) = Client::start(&root, json!({})).await;
    let gone = file_uri(&root.join("lit/ex/gone.lit"));
    // A change to a document that was never opened is dropped.
    c.notify(
        "textDocument/didChange",
        json!({"textDocument": {"uri": gone, "version": 2}, "contentChanges": [{"text": "module y\n"}]}),
    )
    .await;
    let answers = ask_everything(&mut c, &gone).await;
    let empty: Vec<(String, Value)> = answers.into_iter().filter(|(_, v)| !v.is_null()).collect();
    // Symbols, actions, and hints of a file the analysis lacks are empty.
    assert_eq!(
        empty,
        [
            ("textDocument/documentSymbol".to_owned(), json!([])),
            ("textDocument/codeAction".to_owned(), json!([])),
            ("textDocument/inlayHint".to_owned(), json!([])),
        ]
    );
}

#[tokio::test]
async fn formatting_leaves_what_it_cannot_format() {
    let root = support::worked("server-format", &[("notes.txt", "free text\n")]);
    let (mut c, _) = Client::start(&root, json!({})).await;
    let format = |uri: &str| json!({"textDocument": {"uri": uri}, "options": {"tabSize": 2, "insertSpaces": true}});
    // Not open.
    let module = file_uri(&root.join("lit/ex/rule12.lit"));
    assert_eq!(
        c.request("textDocument/formatting", format(&module)).await,
        Value::Null
    );
    // Open, but not a litigate file.
    let notes = file_uri(&root.join("notes.txt"));
    c.notify(
        "textDocument/didOpen",
        json!({"textDocument": {"uri": notes, "languageId": "plaintext", "version": 1, "text": "free text\n"}}),
    )
    .await;
    assert_eq!(
        c.request("textDocument/formatting", format(&notes)).await,
        Value::Null
    );
    // Open, but it does not parse.
    c.notify(
        "textDocument/didOpen",
        json!({"textDocument": {"uri": module, "languageId": "litigate", "version": 1, "text": "module {{{\n"}}),
    )
    .await;
    assert_eq!(
        c.request("textDocument/formatting", format(&module)).await,
        Value::Null
    );
}

#[tokio::test]
async fn saving_rechecks_and_references_can_leave_out_the_declaration() {
    let root = support::worked("server-save", &[]);
    let (mut c, _) = Client::start(&root, json!({})).await;
    let module = file_uri(&root.join("lit/ex/rule12.lit"));
    let first = c.diagnostics(&module).await;
    std::fs::write(
        root.join("lit/ex/rule12.lit"),
        support::read("lit/examples/rule12.lit")
            .replace("-> dismissed p 1/10", "-> nowhere p 1/10"),
    )
    .unwrap();
    c.notify(
        "textDocument/didSave",
        json!({"textDocument": {"uri": module}}),
    )
    .await;
    let saved = c
        .diagnostics_where(&module, |ds| ds.len() != first.len())
        .await;
    assert!(
        support::codes(&saved).contains(&"L1002".to_owned()),
        "{saved:?}"
    );
    let text = support::read("lit/examples/rule12.lit");
    let at = support::at(&text, "deadline answer-period", 0, 10);
    let refs = c
        .request(
            "textDocument/references",
            json!({"textDocument": {"uri": module}, "position": at, "context": {"includeDeclaration": false}}),
        )
        .await;
    // Two `within`s and two inline tests, without the declaration.
    assert_eq!(refs.as_array().map(Vec::len), Some(4), "{refs}");
    // Nothing is named, declared, or referenced on a blank line.
    let blank = support::at(&text, "\n\nstart", 0, 1);
    let none = c
        .request(
            "textDocument/hover",
            json!({"textDocument": {"uri": module}, "position": blank}),
        )
        .await;
    assert_eq!(none, Value::Null);
    let none = c
        .request(
            "textDocument/definition",
            json!({"textDocument": {"uri": module}, "position": blank}),
        )
        .await;
    assert_eq!(none, Value::Null);
    let none = c
        .request(
            "textDocument/references",
            json!({"textDocument": {"uri": module}, "position": blank, "context": {"includeDeclaration": true}}),
        )
        .await;
    assert_eq!(none, Value::Null);
}
