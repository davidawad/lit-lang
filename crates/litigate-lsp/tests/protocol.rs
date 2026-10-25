//! LSP protocol tests over the SPEC §12 worked example (`lit/examples`,
//! `cases/examples/acme.case`), driving the server in-process through the
//! wire protocol at the repository root.
#![allow(clippy::unwrap_used, clippy::expect_used)]

mod support;

use serde_json::{json, Value};
use support::{at, codes, place, read, uri, Client};

const LIT: &str = "lit/examples/rule12.lit";
const CASE: &str = "cases/examples/acme.case";

fn shape(ds: &[Value]) -> Vec<Value> {
    let mut out: Vec<Value> = ds
        .iter()
        .map(|d| json!({"code": d["code"], "severity": d["severity"], "range": d["range"], "message": d["message"]}))
        .collect();
    out.sort_by_key(|d| (d["range"]["start"]["line"].as_u64(), d["code"].to_string()));
    out
}

#[tokio::test]
async fn advertises_every_feature() {
    let (_, init) = Client::start(&support::repo(), json!({})).await;
    let caps = &init["capabilities"];
    for key in [
        "hoverProvider",
        "definitionProvider",
        "referencesProvider",
        "documentSymbolProvider",
        "workspaceSymbolProvider",
        "documentFormattingProvider",
        "codeActionProvider",
        "inlayHintProvider",
    ] {
        assert_eq!(caps[key], json!(true), "{key}");
    }
    assert_eq!(caps["textDocumentSync"]["change"], json!(2));
    assert_eq!(init["serverInfo"]["name"], "litigate-lsp");
}

#[tokio::test]
async fn publishes_the_check_diagnostics_of_the_worked_example() {
    let mut c = Client::repo().await;
    // SPEC §12: under `dev`, L9002 for the three holes; both edges are named.
    let ds = c.diagnostics(&uri(LIT)).await;
    assert_eq!(codes(&ds), ["L9002", "L9002", "L9002"]);
    insta::assert_json_snapshot!("worked_example_dev", shape(&ds));
    let status = c.status().await;
    assert_eq!(
        status["files"],
        json!(sources(&["lit", "cases"])),
        "the analysis is `litigate check lit cases`"
    );
}

/// Every `Lit.toml`, `.lit` and `.case` file under `roots`, repo-relative
/// and sorted: what `litigate check` analyses (`lit/core` included).
fn sources(roots: &[&str]) -> Vec<String> {
    fn walk(dir: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
        for entry in std::fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                walk(&path, out);
            } else {
                out.push(path);
            }
        }
    }
    let repo = support::repo();
    let mut files = Vec::new();
    for root in roots {
        walk(&repo.join(root), &mut files);
    }
    let mut out: Vec<String> = files
        .iter()
        .filter(|p| {
            p.file_name().is_some_and(|n| n == "Lit.toml")
                || p.extension().is_some_and(|e| e == "lit" || e == "case")
        })
        .map(|p| {
            p.strip_prefix(&repo)
                .unwrap()
                .to_string_lossy()
                .into_owned()
        })
        .collect();
    out.sort();
    out
}

#[tokio::test]
async fn strict_profile_fails_each_hole() {
    let (mut c, _) = Client::start(&support::repo(), json!({"profile": "strict"})).await;
    let ds = c.diagnostics(&uri(LIT)).await;
    let errors: Vec<&Value> = ds.iter().filter(|d| d["severity"] == 1).collect();
    assert_eq!(codes(&ds).iter().filter(|c| *c == "L5002").count(), 3);
    assert!(errors.len() >= 3);
}

#[tokio::test]
async fn edits_recheck_incrementally_over_one_database() {
    let mut c = Client::repo().await;
    let text = read(LIT);
    assert_eq!(c.diagnostics(&uri(LIT)).await.len(), 3);
    c.open(LIT, &text).await;
    // `p 1/10` -> `p 2/10`: the fact's distribution no longer sums to one.
    let range = json!({"start": at(&text, "1/10", 0, 0), "end": at(&text, "1/10", 0, 1)});
    c.edit(LIT, 2, range.clone(), "2").await;
    let broken = c.diagnostics_where(&uri(LIT), |ds| ds.len() != 3).await;
    assert!(codes(&broken).contains(&"L2003".to_owned()), "{broken:?}");
    c.edit(LIT, 3, range, "1").await;
    let fixed = c.diagnostics_where(&uri(LIT), |ds| ds.len() == 3).await;
    assert_eq!(shape(&fixed).len(), 3);
    assert_eq!(
        c.status().await["rebuilds"],
        1,
        "text edits reuse the salsa inputs"
    );
    // A change to the matter replays it under the edited package.
    let case = read(CASE);
    c.open(CASE, &case).await;
    let range =
        json!({"start": at(&case, "not-barred", 0, 0), "end": at(&case, "not-barred", 0, 10)});
    c.edit(CASE, 2, range, "nope").await;
    // (While the package was broken the matter could not resolve it.)
    let ds = c
        .diagnostics_where(&uri(CASE), |ds| codes(ds).contains(&"L4003".to_owned()))
        .await;
    // Advice (severity 3: L9006 on each event without an id) aside.
    let ds: Vec<Value> = ds.into_iter().filter(|d| d["severity"] != 3).collect();
    assert_eq!(codes(&ds), ["L4003"], "{ds:?}");
}

#[tokio::test]
async fn hover_shows_kind_type_versions_and_cite() {
    let mut c = Client::repo().await;
    let text = read(LIT);
    let value = |h: &Value| {
        h["contents"]["value"]
            .as_str()
            .unwrap_or_default()
            .to_owned()
    };
    let respond = value(&c.at("hover", LIT, at(&text, "respond:", 0, 1)).await);
    assert!(
        respond.contains("decision `frcp-rule12-example::respond`"),
        "{respond}"
    );
    assert!(
        respond.contains("[2009-12-01, ..) known_at 2025-01-01 · FederalRule"),
        "{respond}"
    );
    assert!(
        respond.contains("**cite** FRCP 12(a)(1)(A)(i)"),
        "{respond}"
    );
    let period = value(
        &c.at("hover", LIT, at(&text, "within answer-period", 0, 8))
            .await,
    );
    assert!(
        period.contains("`Days<CalendarDays>`: 21 after Served(complaint) counted_by `frcp6`"),
        "{period}"
    );
    let served = value(&c.at("hover", LIT, at(&text, "event Served", 0, 7)).await);
    assert!(
        served.contains("Service of a pleading on a party."),
        "{served}"
    );
    assert!(
        served.contains("`Served(document: Document, method: ServiceMethod)`"),
        "{served}"
    );
    insta::assert_snapshot!("hover_respond", respond);
}

/// `(from, needle, shift into it, to, needle on the target line)`.
const GOTO: [(&str, &str, u32, &str, &str); 12] = [
    (CASE, "time-barred: ", 1, LIT, "chance time-barred"),
    (CASE, "not-barred", 1, LIT, "not-barred \"Timely\""),
    (CASE, "perspective widget", 13, CASE, "widget: Defendant"),
    (CASE, "Served(", 1, LIT, "event Served"),
    (
        CASE,
        "Plaintiff",
        1,
        "lit/examples/Lit.toml",
        "Plaintiff = ",
    ),
    (CASE, "granted", 1, LIT, "granted \"Motion"),
    (
        CASE,
        "judgment-entered:",
        1,
        LIT,
        "terminal judgment-entered",
    ),
    (
        LIT,
        "within answer-period",
        9,
        LIT,
        "deadline answer-period",
    ),
    (LIT, "counted_by frcp6", 12, LIT, "counting frcp6"),
    (LIT, "group pleadings\n  answer", 7, LIT, "group pleadings:"),
    (LIT, "Served(complaint, Mail)", 0, LIT, "event Served"),
    (LIT, "(complaint, Mail)", 1, LIT, "enum Document"),
];

#[tokio::test]
async fn goto_definition_crosses_modules_and_matters() {
    let mut c = Client::repo().await;
    c.open(CASE, &read(CASE)).await;
    for (from, needle, shift, to, target) in GOTO {
        let text = read(to);
        let line = u64::try_from(text[..text.find(target).unwrap()].matches('\n').count()).unwrap();
        let position = at(&read(from), needle, 0, shift);
        let found = c.at("definition", from, position).await;
        assert_eq!(place(&found), (to.to_owned(), line), "{from}: {needle}");
    }
}

/// An outcome inside a `prior` resolves to its node's edge, as one inside
/// `observe` does.
#[tokio::test]
async fn goto_definition_reads_a_prior_outcome_as_its_nodes_edge() {
    let mut c = Client::repo().await;
    let text = read(CASE).replacen("observe mtd-ruling", "prior mtd-ruling", 1);
    c.open(CASE, &text).await;
    let lit = read(LIT);
    let line = u64::try_from(
        lit[..lit.find("granted \"Motion").unwrap()]
            .matches('\n')
            .count(),
    )
    .unwrap();
    let found = c.at("definition", CASE, at(&text, "granted", 0, 1)).await;
    assert_eq!(place(&found), (LIT.to_owned(), line));
}

#[tokio::test]
async fn references_span_modules_and_matters() {
    let mut c = Client::repo().await;
    let lit = read(LIT);
    let refs = c
        .at("references", LIT, at(&lit, "deadline answer-period", 0, 10))
        .await;
    let lines: Vec<(String, u64)> = refs.as_array().unwrap().iter().map(place).collect();
    // The declaration, two `within`s, and the two inline tests.
    assert_eq!(lines.len(), 5, "{lines:?}");
    let ruling = c
        .at("references", LIT, at(&lit, "chance mtd-ruling", 0, 8))
        .await;
    let files: Vec<String> = ruling
        .as_array()
        .unwrap()
        .iter()
        .map(|l| place(l).0)
        .collect();
    assert!(
        files.contains(&CASE.to_owned()),
        "the matter's `observe mtd-ruling`: {files:?}"
    );
    assert!(files.iter().filter(|f| *f == LIT).count() >= 2, "{files:?}");
}

#[tokio::test]
async fn symbols_nest_edges_under_nodes() {
    let mut c = Client::repo().await;
    let syms = c
        .request(
            "textDocument/documentSymbol",
            json!({"textDocument": {"uri": uri(LIT)}}),
        )
        .await;
    let tree: Vec<Value> = syms
        .as_array()
        .unwrap()
        .iter()
        .map(|s| {
            let kids: Vec<&Value> = s["children"]
                .as_array()
                .map(|k| k.iter().map(|k| &k["name"]).collect())
                .unwrap_or_default();
            json!({"name": s["name"], "kind": s["kind"], "children": kids})
        })
        .collect();
    insta::assert_json_snapshot!("rule12_symbols", tree);
    let found = c
        .request("workspace/symbol", json!({"query": "answer"}))
        .await;
    let names: Vec<&str> = found
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|s| s["name"].as_str())
        .collect();
    assert!(
        names.contains(&"answer-period")
            && names.contains(&"answer-duty")
            && names.contains(&"answer"),
        "{names:?}"
    );
}

#[tokio::test]
async fn formatting_returns_the_canonical_text() {
    let mut c = Client::repo().await;
    let canonical = read(LIT);
    let messy = canonical
        .replace("start time-barred", "start    time-barred")
        .replace("\n\ngroup motions", "\ngroup motions");
    c.open(LIT, &messy).await;
    let edits = c
        .request("textDocument/formatting", json!({"textDocument": {"uri": uri(LIT)}, "options": {"tabSize": 2, "insertSpaces": true}}))
        .await;
    let edits = edits.as_array().unwrap();
    assert_eq!(edits.len(), 1);
    assert_eq!(edits[0]["newText"], canonical);
    assert_eq!(
        edits[0]["range"]["start"],
        json!({"line": 0, "character": 0})
    );
}

#[tokio::test]
async fn code_actions_apply_the_diagnostic_fix() {
    // SPEC §9.2's example: `expected `->``, fixed by inserting `-> `.
    let mut c = Client::repo().await;
    let text = read(LIT);
    c.open(LIT, &text).await;
    let arrow = at(&text, "-> answered {", 0, 0);
    let end = at(&text, "-> answered {", 0, 3);
    c.edit(LIT, 2, json!({"start": arrow, "end": end}), "")
        .await;
    let has = |ds: &[Value]| ds.iter().any(|d| d["code"] == "L0001");
    let ds = c.diagnostics_where(&uri(LIT), has).await;
    let l0001 = ds
        .iter()
        .find(|d| d["code"] == "L0001")
        .expect("a missing arrow is L0001");
    let actions = c
        .request(
            "textDocument/codeAction",
            json!({"textDocument": {"uri": uri(LIT)}, "range": l0001["range"], "context": {"diagnostics": [l0001]}}),
        )
        .await;
    let action = &actions.as_array().unwrap()[0];
    assert_eq!(action["kind"], "quickfix");
    assert_eq!(action["isPreferred"], true);
    let edit = &action["edit"]["changes"][uri(LIT)][0];
    assert_eq!(edit["newText"], "-> ");
    assert_eq!(edit["range"]["start"], arrow);
}

#[tokio::test]
async fn inlay_hints_show_engine_computed_deadlines() {
    let mut c = Client::repo().await;
    let case = read(CASE);
    c.open(CASE, &case).await;
    let all = json!({"start": {"line": 0, "character": 0}, "end": {"line": 999, "character": 0}});
    let hints = c
        .request(
            "textDocument/inlayHint",
            json!({"textDocument": {"uri": uri(CASE)}, "range": all}),
        )
        .await;
    let hints = hints.as_array().unwrap();
    assert_eq!(hints.len(), 1);
    // SPEC §12: `Served` is timing-only, due 2025-03-26.
    assert_eq!(
        hints[0]["label"],
        "// answer-period due 2025-03-26 (FRCP 6(a))"
    );
    assert_eq!(
        hints[0]["position"]["line"],
        at(&case, "2025-03-05 Served", 0, 0)["line"]
    );
    // Mail service: 21 days lands on Wed 2025-03-26, +3 (FRCP 6(d)) on Sat
    // 2025-03-29, which rolls to Mon 2025-03-31.
    let range = json!({"start": at(&case, "Personal", 0, 0), "end": at(&case, "Personal", 0, 8)});
    c.edit(CASE, 2, range, "Mail").await;
    let hints = c
        .request(
            "textDocument/inlayHint",
            json!({"textDocument": {"uri": uri(CASE)}, "range": all}),
        )
        .await;
    assert_eq!(
        hints[0]["label"],
        "// answer-period due 2025-03-31 (FRCP 6(a) rolled +Mail service)"
    );
}
