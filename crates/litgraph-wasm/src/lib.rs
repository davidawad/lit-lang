// SPDX-License-Identifier: GPL-3.0-or-later
//! WebAssembly bindings for litgraph.
//!
//! The contract is identical to the CLI and the Rust API: JSON strings in,
//! JSON strings out, errors reported inside the response (`ok: false`), never
//! thrown. Packs are embedded; pass your own with [`handle_with_packs`].
//! [`litigate`] runs `litigate check`/`build` over an in-memory file tree,
//! with the repository's `lit/` library embedded.
//!
//! ```js
//! import init, { handle, describe } from "litgraph-wasm";
//! await init();
//! const packId = JSON.parse(describe()).result.packs[0].id; // any embedded pack
//! const resp = JSON.parse(handle(JSON.stringify({ packs: [packId], op: { op: "solve" } })));
//! ```

use std::collections::BTreeMap;
use std::sync::OnceLock;

use litgraph::api::{self, Catalog};
use litigate_cli::LitigateRun;
use serde_json::{json, Value};
use wasm_bindgen::prelude::wasm_bindgen;

include!(concat!(env!("OUT_DIR"), "/embedded_library.rs"));

fn catalog() -> Result<&'static Catalog, String> {
    static CATALOG: OnceLock<Result<Catalog, String>> = OnceLock::new();
    CATALOG
        .get_or_init(|| Catalog::embedded().map_err(|e| e.to_string()))
        .as_ref()
        .map_err(Clone::clone)
}

fn error(code: &str, message: &str) -> String {
    json!({ "ok": false, "api_version": api::API_VERSION, "op": "unknown", "warnings": [], "error": { "code": code, "message": message, "hint": "" }, "elapsed_ms": 0.0 }).to_string()
}

fn to_string(v: &impl serde::Serialize) -> String {
    serde_json::to_string(v).unwrap_or_else(|e| error("invalid", &e.to_string()))
}

/// Run a request (JSON string) against the embedded packs; returns the response JSON.
#[wasm_bindgen]
#[must_use]
pub fn handle(request_json: &str) -> String {
    match catalog() {
        Ok(c) => to_string(&api::handle_json(request_json, c)),
        Err(e) => error("parse", &e),
    }
}

/// Run a request against caller-supplied packs: `packs_json` is an object
/// mapping file names to pack documents, e.g. `{"my.json": {...}, "links.json": {...}}`.
#[wasm_bindgen]
#[must_use]
pub fn handle_with_packs(request_json: &str, packs_json: &str) -> String {
    let files: serde_json::Map<String, Value> = match serde_json::from_str(packs_json) {
        Ok(Value::Object(m)) => m,
        Ok(_) => return error("parse", "packs must be an object of {file name: pack}"),
        Err(e) => return error("parse", &e.to_string()),
    };
    match Catalog::from_files(
        "inline".into(),
        files.into_iter().map(|(k, v)| (k, v.to_string())),
    ) {
        Ok(c) => to_string(&api::handle_json(request_json, &c)),
        Err(e) => error("parse", &e.to_string()),
    }
}

/// Validate a document (pack, links, scenario or request); `kind` forces the kind.
// `wasm_bindgen`'s JS-interop ABI needs owned `Option<String>` here, not `Option<&str>`.
#[allow(clippy::needless_pass_by_value)]
#[wasm_bindgen]
#[must_use]
pub fn validate(doc_json: &str, kind: Option<String>) -> String {
    let c = match catalog() {
        Ok(c) => c,
        Err(e) => return error("parse", &e),
    };
    match serde_json::from_str::<Value>(doc_json) {
        Ok(doc) => to_string(&api::validate(&doc, kind.as_deref(), c)),
        Err(e) => error("parse", &e.to_string()),
    }
}

/// JSON Schema for a `litgraph::api::SCHEMA_KINDS` kind: `request`, `response`,
/// `scenario`, `named-scenario`, `pack`, `links`, `calibration` or `clock`.
#[wasm_bindgen]
#[must_use]
pub fn schema(kind: &str) -> String {
    match api::schema(kind) {
        Ok(s) => s.to_string(),
        Err(e) => error(e.code(), &e.to_string()),
    }
}

/// Runs the `litigate` CLI over an in-memory file tree and returns a
/// `LitigateRun` (`{code, stdout, stderr, files}`) as JSON. `args_json` is
/// the argument array without the program name (`["check", "lit", "cases"]`);
/// only `check` and `build` run. `files_json` maps repository-relative paths
/// to texts; with `with_library`, the embedded `lit/` library fills in every
/// `lit/` path the caller does not give. `build --out DIR` writes into
/// `files`, never to a disk. Output is JSON unless `--format human`.
#[wasm_bindgen]
#[must_use]
pub fn litigate(args_json: &str, files_json: &str, with_library: bool) -> String {
    let failed = |message: String| LitigateRun {
        code: 1,
        stderr: format!("litigate: {message}\n"),
        ..LitigateRun::default()
    };
    let run = match (
        serde_json::from_str::<Vec<String>>(args_json),
        serde_json::from_str::<BTreeMap<String, String>>(files_json),
    ) {
        (Err(e), _) => failed(format!("args must be an array of strings: {e}")),
        (_, Err(e)) => failed(format!("files must be an object of {{path: text}}: {e}")),
        (Ok(args), Ok(files)) => {
            let library = EMBEDDED_LIBRARY
                .iter()
                .filter(|_| with_library)
                .map(|(path, text)| ((*path).to_owned(), (*text).to_owned()));
            // The caller's files come last, so they win over the library's.
            litigate_cli::execute_in_memory(library.chain(files), &args)
        }
    };
    to_string(&run)
}

/// The embedded `lit/` library as a `{path: text}` JSON object.
#[wasm_bindgen]
#[must_use]
pub fn litigate_library() -> String {
    to_string(&EMBEDDED_LIBRARY.iter().copied().collect::<BTreeMap<_, _>>())
}

/// The machine-readable manual.
#[wasm_bindgen]
#[must_use]
pub fn describe() -> String {
    handle(r#"{"op":{"op":"describe"}}"#)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(s: &str) -> Value {
        serde_json::from_str(s).unwrap_or(Value::Null)
    }

    #[test]
    fn same_contract_as_the_cli() {
        let r = parse(&handle(r#"{"packs":["cofc"],"op":{"op":"solve"}}"#));
        assert_eq!(r["ok"], json!(true));
        assert!(r["result"]["value"].is_number());
        assert_eq!(parse(&handle("not json"))["ok"], json!(false));
        assert_eq!(parse(&describe())["op"], json!("describe"));
        assert!(parse(&schema("scenario"))["title"].is_string());
        assert_eq!(parse(&schema("nope"))["ok"], json!(false));
        assert_eq!(
            parse(&validate(r#"{"params":{"rate":1}}"#, None))["valid"],
            json!(true)
        );
        assert_eq!(parse(&validate("{", None))["ok"], json!(false));
    }

    fn run(args: &[&str], files: &Value, with_library: bool) -> LitigateRun {
        let args = serde_json::to_string(args).unwrap_or_default();
        serde_json::from_str(&litigate(&args, &files.to_string(), with_library)).unwrap_or_default()
    }

    #[test]
    fn litigate_checks_and_builds_over_the_embedded_library() {
        let case = include_str!("../../../cases/frcp-civil-case.case");
        let lock = include_str!("../../../cases/Lit.lock");
        let files = json!({ "cases/frcp-civil-case.case": case, "cases/Lit.lock": lock });
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
        let library = parse(&litigate_library());
        assert_eq!(
            library
                .as_object()
                .map(|m| m.keys().all(|k| k.starts_with("lit/"))),
            Some(true)
        );
        assert!(library["lit/frcp/Lit.toml"].is_string());
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
}
