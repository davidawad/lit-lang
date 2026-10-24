//! Editing over the wire (design review §4.4, §6 item 25): debounced and
//! cancellable checks on the shared database, the world picker, rename
//! across files, and completion.
#![allow(clippy::unwrap_used, clippy::expect_used)]

mod support;

use serde_json::{json, Value};
use std::collections::BTreeMap;
use support::{at, codes, read, Client};
use tower_lsp::lsp_types::Url;

const MODULE: &str = "lit/ex/rule12.lit";
const CASE: &str = "cases/acme.case";

/// What a workspace edit changes, as `(file, [(line, character, new)])`
/// with files relative to `root`, sorted.
fn changes(edit: &Value, root: &std::path::Path) -> BTreeMap<String, Vec<(u64, u64, String)>> {
    edit["changes"]
        .as_object()
        .unwrap()
        .iter()
        .map(|(uri, edits)| {
            let path = Url::parse(uri).unwrap().to_file_path().unwrap();
            let rel = path
                .strip_prefix(root)
                .unwrap()
                .to_string_lossy()
                .replace('\\', "/");
            let mut list: Vec<(u64, u64, String)> = edits
                .as_array()
                .unwrap()
                .iter()
                .map(|e| {
                    (
                        e["range"]["start"]["line"].as_u64().unwrap(),
                        e["range"]["start"]["character"].as_u64().unwrap(),
                        e["newText"].as_str().unwrap().to_owned(),
                    )
                })
                .collect();
            list.sort();
            (rel, list)
        })
        .collect()
}

fn file_uri(root: &std::path::Path, path: &str) -> String {
    Url::from_file_path(root.join(path)).unwrap().to_string()
}

/// The hover text at `needle` (two characters in) of `text`, open at `uri`.
async fn hover(c: &mut Client, uri: &str, text: &str, needle: &str) -> String {
    let position = position(uri, at(text, needle, 0, 2));
    c.request("textDocument/hover", position).await["contents"]["value"]
        .as_str()
        .unwrap_or_default()
        .to_owned()
}

fn position(uri: &str, at: Value) -> Value {
    json!({"textDocument": {"uri": uri}, "position": at})
}

#[tokio::test]
async fn a_burst_of_edits_is_checked_once() {
    let root = support::worked("editing-debounce", &[]);
    let (mut c, _) = Client::start(&root, json!({"debounceMs": 300})).await;
    let module = read("lit/examples/rule12.lit");
    let uri = file_uri(&root, MODULE);
    c.notify(
        "textDocument/didOpen",
        json!({"textDocument": {"uri": uri, "languageId": "litigate", "version": 1, "text": module}}),
    )
    .await;
    let _ = c.diagnostics(&uri).await;
    let before = c.status().await["checks"].as_u64().unwrap();
    // Three edits within the debounce window, each at the same place: the
    // last names the unknown counting rule `frcp6zyx`.
    let end = at(&module, "counted_by frcp6", 0, 16);
    for (version, text) in [(2, "x"), (3, "y"), (4, "z")] {
        let range = json!({"start": end, "end": end});
        c.notify(
            "textDocument/didChange",
            json!({"textDocument": {"uri": uri, "version": version},
                   "contentChanges": [{"range": range, "text": text}]}),
        )
        .await;
    }
    let ds = c
        .diagnostics_where(&uri, |ds| codes(ds).contains(&"L1002".to_owned()))
        .await;
    assert!(ds
        .iter()
        .any(|d| d["message"].as_str().unwrap().contains("frcp6zyx")));
    assert_eq!(c.status().await["checks"].as_u64().unwrap(), before + 1);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_newer_change_cancels_the_running_check() {
    // The whole repository: the check `initialized` starts takes long
    // enough that the document opened next arrives while it runs.
    let mut c = Client::repo().await;
    let path = "lit/examples/rule12.lit";
    c.open(path, &read(path)).await;
    let ds = c.diagnostics(&support::uri(path)).await;
    assert_eq!(codes(&ds), ["L9002", "L9002", "L9002", "L9003", "L9003"]);
    let status = c.status().await;
    assert_eq!(status["cancelled"], json!(1), "{status}");
    assert_eq!(status["checks"], json!(2), "{status}");
    assert_eq!(
        status["rebuilds"],
        json!(1),
        "the cancelled check's database is reused"
    );
}

#[tokio::test]
async fn the_world_picker_selects_the_dates_packages_are_checked_at() {
    let extra = "deadline reply-period: 14.days from Served(complaint) counted_by frcp6 effective [2012-01-01, ..) notwithstanding answer-period\n";
    let module = format!("{}\n{extra}", read("lit/examples/rule12.lit"));
    let root = support::worked("editing-worlds", &[("lit/ex/rule12.lit", &module)]);
    support::pin(&root, &mut support::host(&root));
    let (mut c, _) = Client::start(&root, json!({})).await;
    let uri = file_uri(&root, MODULE);
    c.open_at(&uri, &module).await;
    let _ = c.diagnostics(&uri).await;
    let worlds = c.request("litigate/worlds", Value::Null).await;
    assert_eq!(
        worlds,
        json!({
            "current": {"label": "selected", "asOf": null, "knownAt": null},
            "worlds": [
                {"label": "package dates", "asOf": null, "knownAt": null},
                {"label": "cases/acme.case", "asOf": "2025-06-01", "knownAt": null},
            ],
        })
    );
    // No world selected: no world line.
    assert!(!hover(&mut c, &uri, &module, "deadline answer-period")
        .await
        .contains("**world**"));

    let selected = c
        .request(
            "litigate/selectWorld",
            json!({"asOf": "2020-01-01", "knownAt": null}),
        )
        .await;
    assert_eq!(selected["current"]["asOf"], "2020-01-01");
    let answer = hover(&mut c, &uri, &module, "answer-period:").await;
    assert!(
        answer.contains(
            "**world** on 2020-01-01, governed by `frcp-rule12-example::reply-period@2012-01-01`"
        ),
        "{answer}"
    );
    let reply = hover(&mut c, &uri, &module, "reply-period:").await;
    assert!(
        reply.contains("**world** in force on 2020-01-01: the version from 2012-01-01"),
        "{reply}"
    );
    // An enum has no governing version.
    assert!(!hover(&mut c, &uri, &module, "Document {")
        .await
        .contains("**world**"));

    let _ = c
        .request(
            "litigate/selectWorld",
            json!({"asOf": "2010-06-01", "knownAt": "2025-01-01"}),
        )
        .await;
    let early = hover(&mut c, &uri, &module, "reply-period:").await;
    assert!(
        early.contains("**world** on 2010-06-01 (known_at 2025-01-01): no version of `reply-period` is in force on 2010-06-01"),
        "{early}"
    );
    for bad in [json!({"asOf": "June"}), json!({"knownAt": 7})] {
        let error = c.request_error("litigate/selectWorld", bad).await;
        assert_eq!(error["code"], json!(-32602), "{error}");
    }
    // Back to the packages' own dates.
    let reset = c.request("litigate/selectWorld", json!({})).await;
    assert_eq!(reset["current"]["asOf"], Value::Null);
}

#[tokio::test]
async fn rename_edits_every_version_and_reference() {
    let extra = "deadline reply-period: 14.days from Served(complaint) counted_by frcp6 effective [2009-12-01, 2016-12-01)\ndeadline reply-period: 21.days from Served(complaint) counted_by frcp6 effective [2016-12-01, ..)\n";
    let module = format!("{}\n{extra}", read("lit/examples/rule12.lit"));
    let root = support::worked("editing-rename", &[("lit/ex/rule12.lit", &module)]);
    let (mut c, _) = Client::start(&root, json!({})).await;
    let module_uri = file_uri(&root, MODULE);
    let case_uri = file_uri(&root, CASE);
    let case = read("cases/examples/acme.case");
    c.open_at(&module_uri, &module).await;
    let _ = c.diagnostics(&module_uri).await;

    // A node, named in the matter too: one edit across both files.
    let declared = at(&module, "time-barred:", 0, 2);
    let prepared = c
        .request(
            "textDocument/prepareRename",
            position(&module_uri, declared.clone()),
        )
        .await;
    assert_eq!(prepared["placeholder"], "time-barred");
    let mut rename = position(&module_uri, declared);
    rename["newName"] = json!("limitations");
    let edit = c.request("textDocument/rename", rename).await;
    let found = changes(&edit, &root);
    let lines = |f: &str| found[f].iter().map(|(l, _, _)| *l).collect::<Vec<_>>();
    assert_eq!(found.keys().collect::<Vec<_>>(), [CASE, MODULE]);
    assert_eq!(lines(CASE).len(), 1);
    assert_eq!(lines(MODULE).len(), 2, "the start and the declaration");
    assert!(found.values().flatten().all(|(_, _, n)| n == "limitations"));

    // From a reference: the range is the reference's; both versions move.
    let used = at(&module, "within answer-period", 0, 9);
    let prepared = c
        .request(
            "textDocument/prepareRename",
            position(&module_uri, used.clone()),
        )
        .await;
    assert_eq!(
        prepared["range"]["start"],
        at(&module, "within answer-period", 0, 7)
    );
    let reply = at(&module, "reply-period", 1, 1);
    let mut rename = position(&module_uri, reply);
    rename["newName"] = json!("response-period");
    let edit = c.request("textDocument/rename", rename).await;
    assert_eq!(changes(&edit, &root)[MODULE].len(), 2, "both versions");

    // A party of the matter, in the matter only.
    let party = at(&case, "widget: Defendant", 0, 1);
    let mut rename = position(&case_uri, party);
    rename["newName"] = json!("widgetco");
    let edit = c.request("textDocument/rename", rename).await;
    assert_eq!(
        changes(&edit, &root)[CASE].len(),
        4,
        "declared, perspective, on, by"
    );

    // A module path is not renamed; nor is a name to something not an
    // identifier.
    let module_name = at(&module, "frcp.rule12", 0, 1);
    assert_eq!(
        c.request(
            "textDocument/prepareRename",
            position(&module_uri, module_name)
        )
        .await,
        Value::Null
    );
    let mut bad = position(&module_uri, at(&module, "time-barred:", 0, 2));
    bad["newName"] = json!("two words");
    let error = c.request_error("textDocument/rename", bad).await;
    assert_eq!(error["code"], json!(-32602));
}

#[tokio::test]
async fn completion_offers_what_a_reference_can_name() {
    let root = support::worked("editing-complete", &[]);
    let (mut c, _) = Client::start(&root, json!({})).await;
    let module = read("lit/examples/rule12.lit");
    let module_uri = file_uri(&root, MODULE);
    c.open_at(&module_uri, &module).await;
    let _ = c.diagnostics(&module_uri).await;
    let labels = |v: &Value| -> Vec<String> {
        v.as_array()
            .unwrap()
            .iter()
            .map(|i| i["label"].as_str().unwrap().to_owned())
            .collect()
    };
    let complete = |uri: &str, at: Value| json!({"textDocument": {"uri": uri}, "position": at, "context": {"triggerKind": 1}});

    // Typing in the open document: completion reads what is typed.
    let end = at(&module, "counted_by frcp6", 0, 11);
    let range = json!({"start": end, "end": at(&module, "counted_by frcp6", 0, 16)});
    c.notify(
        "textDocument/didChange",
        json!({"textDocument": {"uri": module_uri, "version": 2},
               "contentChanges": [{"range": range, "text": "fr"}]}),
    )
    .await;
    let _ = c.diagnostics(&module_uri).await;
    let mut typed = end.clone();
    typed["character"] = json!(end["character"].as_u64().unwrap() + 2);
    let found = c
        .request("textDocument/completion", complete(&module_uri, typed))
        .await;
    assert_eq!(labels(&found), ["frcp6"]);
    assert_eq!(found[0]["detail"], "counting · frcp-rule12-example");

    // A matter sees the packages it uses, its parties and its keywords.
    let case_uri = file_uri(&root, CASE);
    let case = read("cases/examples/acme.case");
    let start = at(&case, "perspective widget", 0, 12);
    let found = labels(
        &c.request("textDocument/completion", complete(&case_uri, start))
            .await,
    );
    for want in [
        "acme",
        "widget",
        "time-barred",
        "Served",
        "Document",
        "complaint",
        "Defendant",
        "events",
    ] {
        assert!(found.contains(&want.to_owned()), "{want}: {found:?}");
    }
    assert!(
        !found.contains(&"state".to_owned()),
        ".lit keywords stay out of a matter"
    );

    // After `pack::`, that package's declarations only.
    c.open_at(&case_uri, &format!("{case}\nfrcp-rule12-example::ans"))
        .await;
    let lines = u32::try_from(case.lines().count()).unwrap();
    let qualified = json!({"line": lines + 1, "character": 24});
    let found = labels(
        &c.request("textDocument/completion", complete(&case_uri, qualified))
            .await,
    );
    assert!(found.contains(&"answer-period".to_owned()), "{found:?}");
    assert!(found.iter().all(|l| l.starts_with("ans")), "{found:?}");
}
