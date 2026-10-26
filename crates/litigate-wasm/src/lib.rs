// SPDX-License-Identifier: GPL-3.0-or-later
//! `litigate` as WebAssembly: the compiler and the engine in-process, from
//! `.lit`/`.case` source, with no filesystem.
//!
//! [`litigate`] runs the `litigate` command line over an in-memory file
//! tree: `check`, `build`, `worlds`, `run`, the op subcommands (`solve`,
//! `chain`, ...) and `engine`, each compiling the matters and packages it
//! names.
//! The repository's `lit/` library and `cases/` matters are embedded;
//! [`handle`], [`validate`] and [`describe`] answer engine requests over
//! them compiled (once per instance), as `litigate engine q` does at the
//! repository root. [`handle_with_packs`] runs a request against
//! caller-supplied pack documents. Errors come back inside the response
//! (`ok: false`, or a nonzero `code`), never thrown.
//!
//! ```js
//! import init, { describe, handle, litigate } from "litigate-wasm";
//! await init();
//! const name = JSON.parse(describe()).result.scenario_library[0].id; // any compiled matter
//! const resp = JSON.parse(handle(JSON.stringify({ scenario: name, op: { op: "solve" } })));
//! const run = JSON.parse(litigate(JSON.stringify(["solve", "cases/x.case"]), JSON.stringify({ "cases/x.case": text }), true));
//! ```

use std::collections::BTreeMap;
use std::sync::OnceLock;

use litigate_cli::out::Failure;
use litigate_cli::LitigateRun;
use litigate_engine::api::{self, Catalog};
use serde_json::{json, Value};
use wasm_bindgen::prelude::wasm_bindgen;

include!(concat!(env!("OUT_DIR"), "/embedded_sources.rs"));

/// The embedded `lit/` and `cases/` files.
fn embedded() -> impl Iterator<Item = (String, String)> {
    EMBEDDED_LIBRARY
        .iter()
        .chain(EMBEDDED_CASES)
        .map(|(path, text)| ((*path).to_owned(), (*text).to_owned()))
}

/// The packs and named scenarios `files` compile to, as `litigate engine`
/// serves them: its `lit/` packages and `cases/` matters.
fn compile(files: impl IntoIterator<Item = (String, String)>) -> Result<Catalog, Failure> {
    litigate_cli::engine::compiled(&litigate_cli::memory_tree(files), &[], &[])
}

/// The embedded sources compiled, once.
fn catalog() -> Result<&'static Catalog, &'static Failure> {
    static CATALOG: OnceLock<Result<Catalog, Failure>> = OnceLock::new();
    CATALOG.get_or_init(|| compile(embedded())).as_ref()
}

fn error(code: &str, message: &str) -> String {
    json!({ "ok": false, "api_version": api::API_VERSION, "op": "unknown", "warnings": [], "error": { "code": code, "message": message, "hint": "" }, "elapsed_ms": 0.0 }).to_string()
}

fn to_string(v: &impl serde::Serialize) -> String {
    // Responses and runs are derive-`Serialize` trees with string keys.
    serde_json::to_string(v).unwrap_or_default()
}

/// [`handle`] over `catalog`, or its failure as an error response.
fn handle_in(catalog: Result<&Catalog, &Failure>, request_json: &str) -> String {
    match catalog {
        Ok(c) => to_string(&api::handle_json(request_json, c)),
        Err(e) => error("parse", &e.0),
    }
}

/// [`validate`] over `catalog`, or its failure as an error response.
fn validate_in(catalog: Result<&Catalog, &Failure>, doc_json: &str, kind: Option<&str>) -> String {
    match (catalog, serde_json::from_str::<Value>(doc_json)) {
        (Err(e), _) => error("parse", &e.0),
        (Ok(c), Ok(doc)) => to_string(&api::validate(&doc, kind, c)),
        (Ok(_), Err(e)) => error("parse", &e.to_string()),
    }
}

/// Run an engine request (JSON string) against the embedded `lit/` and
/// `cases/` compiled; returns the response JSON, as `litigate engine q`.
#[wasm_bindgen]
#[must_use]
pub fn handle(request_json: &str) -> String {
    handle_in(catalog(), request_json)
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
    let files = files.into_iter().map(|(k, v)| (k, v.to_string()));
    // `from_files` records a pack that does not parse rather than failing.
    let catalog = Catalog::from_files("inline".into(), files).map_err(Failure::from);
    handle_in(catalog.as_ref(), request_json)
}

/// Validate a document (pack, links, scenario or request) against the
/// compiled embedded sources; `kind` forces the kind.
// `wasm_bindgen`'s JS-interop ABI needs owned `Option<String>` here, not `Option<&str>`.
#[allow(clippy::needless_pass_by_value)]
#[wasm_bindgen]
#[must_use]
pub fn validate(doc_json: &str, kind: Option<String>) -> String {
    validate_in(catalog(), doc_json, kind.as_deref())
}

/// JSON Schema for a `litigate_engine::api::SCHEMA_KINDS` kind: `request`, `response`,
/// `scenario`, `named-scenario`, `pack`, `links`, `calibration` or `clock`.
#[wasm_bindgen]
#[must_use]
pub fn schema(kind: &str) -> String {
    match api::schema(kind) {
        Ok(s) => s.to_string(),
        Err(e) => error(e.code(), &e.to_string()),
    }
}

/// The engine's machine-readable manual over the compiled embedded sources.
#[wasm_bindgen]
#[must_use]
pub fn describe() -> String {
    handle(r#"{"op":{"op":"describe"}}"#)
}

/// Runs the `litigate` CLI over an in-memory file tree and returns a
/// `LitigateRun` (`{code, stdout, stderr, files}`) as JSON. `args_json` is
/// the argument array without the program name (`["solve",
/// "cases/x.case"]`); `check`, `build`, `worlds` (alternate timelines),
/// `run`, the op subcommands and `engine` run. `files_json` maps repository-relative paths to texts;
/// with `with_library`, the embedded `lit/` library fills in every `lit/`
/// path the caller does not give. `build --out DIR` writes into `files`,
/// never to a disk. Output is JSON unless `--format human`.
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

/// The embedded `cases/` matters as a `{path: text}` JSON object.
#[wasm_bindgen]
#[must_use]
pub fn litigate_cases() -> String {
    to_string(&EMBEDDED_CASES.iter().copied().collect::<BTreeMap<_, _>>())
}

#[cfg(test)]
mod tests;
