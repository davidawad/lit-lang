//! One MCP tool per engine op, run on a compiled matter or package as
//! `litigate run TARGET --op ...` runs it, plus the raw-request
//! [`RAW_TOOL`] for debugging.
//!
//! An op tool's input schema is sliced out of the engine's own `schemars`
//! schema for [`Request`](api::Request): the `Op` variant whose `op` const
//! is the tool's name, less that tag, plus the matter fields of
//! [`matter_properties`]. A field added to an op becomes a tool argument
//! with no edit here, and the engine parses the op as strictly as it
//! parses every request (an unknown field is an error naming the valid
//! ones).

use super::{failure, reply};
use crate::cli::{Library, OpFlags};
use crate::out::Failure;
use litigate_engine::api::{self, Catalog};
use serde_json::{json, Map, Value};
use std::sync::OnceLock;

/// The raw-request tool: a [`Request`](api::Request) verbatim over the
/// library's packs and named scenarios, as `litigate engine q` runs it.
pub const RAW_TOOL: &str = "litigate_engine";

fn request_schema() -> &'static Value {
    static SCHEMA: OnceLock<Value> = OnceLock::new();
    SCHEMA.get_or_init(|| {
        serde_json::to_value(schemars::schema_for!(api::Request)).unwrap_or_default()
    })
}

fn variants() -> impl Iterator<Item = &'static Value> {
    request_schema()["$defs"]["Op"]["oneOf"]
        .as_array()
        .into_iter()
        .flatten()
}

/// An `Op` variant's wire name, its `op` const.
fn name_of(variant: &Value) -> &str {
    variant["properties"]["op"]["const"]
        .as_str()
        .unwrap_or_default()
}

/// Every engine op's wire name, in the order the engine's `Op` schema
/// lists them.
#[must_use]
pub fn op_names() -> &'static [String] {
    static NAMES: OnceLock<Vec<String>> = OnceLock::new();
    NAMES.get_or_init(|| variants().map(|v| name_of(v).to_owned()).collect())
}

/// The fields naming the matter an op runs on.
fn matter_properties() -> Map<String, Value> {
    let fields = json!({
        "target": {
            "type": "string",
            "description": "A `.case` matter (the op runs under its scenario), or a package (`.lit` file or package directory: its pack under the default scenario; a links package: the packs it composes). A target that fails its check is not run.",
        },
        "set": {
            "type": "object",
            "additionalProperties": {"type": "number"},
            "description": "Scenario params composed onto the matter's `analysis { params }`, e.g. `{\"rate\": 900}`.",
        },
        "lib": {
            "type": "array",
            "items": {"type": "string"},
            "description": "Library directories searched for the packages the target `uses` (default: the server's `--lib`, else `lit` and the nearest `lit` above the matter).",
        },
        "no_links": {
            "type": "boolean",
            "description": "Do not compose links between the used packages.",
        },
    });
    fields.as_object().cloned().unwrap_or_default()
}

/// The input schema of an `Op` variant's tool: the matter fields and the
/// op's own fields (less the `op` tag), with the request schema's `$defs`
/// so every `$ref` resolves.
fn op_input_schema(variant: &Value) -> Value {
    let mut properties = matter_properties();
    let own = variant["properties"].as_object().into_iter().flatten();
    properties.extend(
        own.filter(|(k, _)| *k != "op")
            .map(|(k, v)| (k.clone(), v.clone())),
    );
    let required: Vec<Value> = std::iter::once(json!("target"))
        .chain(
            variant["required"]
                .as_array()
                .into_iter()
                .flatten()
                .filter(|v| *v != "op")
                .cloned(),
        )
        .collect();
    json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "type": "object",
        "properties": properties,
        "required": required,
        "additionalProperties": false,
        "$defs": request_schema()["$defs"],
    })
}

/// Every op tool, then [`RAW_TOOL`]. An op tool's description is the
/// engine manual's own for that op.
pub fn tools() -> Vec<Value> {
    // The manual's op descriptions do not depend on the packs.
    let manual = Catalog::from_files(String::new(), [])
        .map(|empty| api::describe(&empty))
        .unwrap_or_default();
    let mut out: Vec<Value> = variants()
        .map(|variant| {
            let name = name_of(variant);
            let doc = manual["ops"][name].as_str().unwrap_or_default();
            let doc = format!(
                "{doc} Runs on a compiled `.case` matter or package (`target`); the result is the `litigate` envelope with the engine's response as `result`."
            );
            super::tool(name, &doc, op_input_schema(variant))
        })
        .collect();
    out.push(super::tool(
        RAW_TOOL,
        "Run a raw engine request `{packs?, links?, no_continuations?, scenario?, op}` verbatim over the packs and named scenarios compiled from the library (`lit/` and `cases/`, or `$LITGRAPH_PACKS`/`$LITGRAPH_SCENARIOS`), as `litigate engine q` does; for debugging. Returns the engine's response.",
        request_schema().clone(),
    ));
    out
}

/// A matter field of the wrong type.
fn ill_typed(field: &str, kind: &str) -> Failure {
    Failure::new(format!("`{field}` must be {kind}"))
}

/// The strings of a string-array field.
fn strings(field: &str, value: &Value) -> Result<Vec<String>, Failure> {
    let ill = || ill_typed(field, "an array of strings");
    let items = value.as_array().ok_or_else(ill)?;
    items
        .iter()
        .map(|v| v.as_str().map(str::to_owned).ok_or_else(ill))
        .collect()
}

/// Takes the matter fields out of `args`: the target and how to run on it.
fn matter(args: &mut Map<String, Value>, lib: &[String]) -> Result<(String, OpFlags), Failure> {
    let target = match args.remove("target") {
        Some(Value::String(t)) => t,
        Some(_) => return Err(ill_typed("target", "a string")),
        None => return Err(Failure::new("missing required field `target`")),
    };
    let set = match args.remove("set") {
        Some(Value::Object(params)) => params
            .into_iter()
            .map(|(k, v)| match v {
                Value::Number(x) => Ok(format!("{k}={x}")),
                _ => Err(ill_typed(&format!("set.{k}"), "a number")),
            })
            .collect::<Result<_, _>>()?,
        Some(_) => return Err(ill_typed("set", "an object of numbers")),
        None => Vec::new(),
    };
    let lib = match args.remove("lib") {
        Some(dirs) => strings("lib", &dirs)?,
        None => lib.to_vec(),
    };
    let no_links = match args.remove("no_links") {
        Some(Value::Bool(b)) => b,
        Some(_) => return Err(ill_typed("no_links", "a boolean")),
        None => false,
    };
    let flags = OpFlags {
        args: Vec::new(),
        set,
        no_links,
        library: Library { lib },
    };
    Ok((target, flags))
}

/// Runs op `name` on the matter `args` names, with the rest of `args` as
/// the op's fields: `(envelope, is_error)`, `is_error` when `litigate run`
/// would exit non-zero. `lib` is the library a call naming none uses.
#[must_use]
pub fn call(name: &str, mut args: Map<String, Value>, lib: &[String]) -> (Value, bool) {
    let started = crate::Stopwatch::start();
    let outcome = matter(&mut args, lib).and_then(|(target, flags)| {
        args.insert("op".to_owned(), json!(name));
        crate::cmd::run::run(&target, &Value::Object(args).to_string(), &flags)
    });
    reply(name, outcome, started.elapsed_ms())
}

/// Runs a raw request over `catalog` (the library's, or why it did not
/// load): the engine's response and whether it is not `ok`.
#[must_use]
pub fn call_raw(catalog: Result<&Catalog, &str>, args: Map<String, Value>) -> (Value, bool) {
    match catalog {
        Ok(catalog) => {
            let response = api::handle_json(&Value::Object(args).to_string(), catalog);
            (
                serde_json::to_value(&response).unwrap_or_default(),
                !response.ok,
            )
        }
        Err(message) => (failure("engine", message), true),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CASE: &str = "tests/fixtures/cov-cmds/cases/cov.case";
    const LIB: &str = "tests/fixtures/cov-cmds/lit";

    fn obj(v: Value) -> Map<String, Value> {
        v.as_object().cloned().unwrap_or_default()
    }

    #[test]
    fn every_engine_op_is_a_tool_and_the_raw_request_one_more() {
        let names: Vec<Value> = tools().iter().map(|t| t["name"].clone()).collect();
        for op in ["describe", "solve", "chain", "explain", "validate", "batch"] {
            assert!(names.contains(&json!(op)), "missing {op}");
        }
        assert_eq!(names.len(), op_names().len() + 1);
        assert_eq!(names.last(), Some(&json!(RAW_TOOL)));
    }

    #[test]
    fn an_op_schema_takes_the_matter_and_the_ops_own_fields() {
        let sweep = tools().into_iter().find(|t| t["name"] == "sweep");
        let schema = sweep.map(|t| t["inputSchema"].clone()).unwrap_or_default();
        let props = schema["properties"]
            .as_object()
            .cloned()
            .unwrap_or_default();
        for field in ["target", "set", "lib", "no_links", "param", "lo", "hi"] {
            assert!(props.contains_key(field), "missing {field}");
        }
        assert!(!props.contains_key("op"));
        assert_eq!(schema["required"], json!(["target", "param"]));
        assert_eq!(schema["additionalProperties"], json!(false));
        assert!(schema["$defs"]["Scenario"].is_object());
    }

    #[test]
    fn an_op_runs_on_a_compiled_matter() {
        let (envelope, is_error) =
            call("solve", obj(json!({"target": CASE, "max_steps": 50})), &[]);
        assert!(!is_error, "{envelope}");
        assert_eq!(envelope["op"], json!("solve"));
        assert_eq!(envelope["result"]["op"], json!("solve"));
        assert!(envelope["result"]["result"]["value"].is_number());
        let args = json!({"target": CASE, "set": {"rate": 900}, "lib": [LIB], "no_links": true});
        let (envelope, is_error) = call("chain", obj(args), &[]);
        assert!(!is_error, "{envelope}");
        assert_eq!(envelope["result"]["ok"], json!(true));
    }

    #[test]
    fn the_server_library_is_the_default() {
        let (envelope, is_error) = call("solve", obj(json!({"target": CASE})), &[LIB.to_owned()]);
        assert!(!is_error, "{envelope}");
        let (envelope, is_error) = call("solve", obj(json!({"target": "nope.case"})), &[]);
        assert!(is_error);
        assert_eq!(envelope["error"]["code"], json!("usage"));
    }

    #[test]
    fn an_engine_rejection_is_an_error_carrying_the_envelope() {
        let (envelope, is_error) = call("solve", obj(json!({"target": CASE, "bogus": 1})), &[]);
        assert!(is_error);
        assert!(envelope["error"]["message"]
            .as_str()
            .unwrap_or_default()
            .contains("bogus"));
        let (envelope, is_error) = call("graph", obj(json!({"target": CASE, "node": "nope"})), &[]);
        assert!(is_error, "{envelope}");
        assert_eq!(envelope["result"]["ok"], json!(false));
    }

    #[test]
    fn ill_typed_matter_fields_are_usage_failures() {
        let cases = [
            (json!({}), "missing required field `target`"),
            (json!({"target": 1}), "`target` must be a string"),
            (
                json!({"target": CASE, "set": {"rate": "x"}}),
                "`set.rate` must be a number",
            ),
            (
                json!({"target": CASE, "set": [1]}),
                "`set` must be an object of numbers",
            ),
            (
                json!({"target": CASE, "lib": "lit"}),
                "`lib` must be an array of strings",
            ),
            (
                json!({"target": CASE, "lib": [1]}),
                "`lib` must be an array of strings",
            ),
            (
                json!({"target": CASE, "no_links": 1}),
                "`no_links` must be a boolean",
            ),
        ];
        for (args, expected) in cases {
            let (envelope, is_error) = call("solve", obj(args.clone()), &[]);
            assert!(is_error, "{args}");
            assert_eq!(envelope["error"]["message"], json!(expected), "{args}");
        }
    }

    #[test]
    fn the_raw_tool_runs_a_request_or_reports_the_library() {
        let catalog = Catalog::from_files("empty".into(), []).unwrap();
        let (response, is_error) = call_raw(Ok(&catalog), obj(json!({"op": {"op": "describe"}})));
        assert!(is_error, "an empty catalog is refused: {response}");
        assert_eq!(response["op"], json!("describe"));
        let (envelope, is_error) = call_raw(Err("no packs"), Map::new());
        assert!(is_error);
        assert_eq!(envelope["error"]["message"], json!("no packs"));
    }
}
