// SPDX-License-Identifier: GPL-3.0-or-later
//! The `litigate_*` tools: the authoring toolchain (`.lit` rule modules and
//! `.case` matters, `docs/litigate/SPEC.md` §9.3) over MCP, so an agent can
//! check, build and navigate the source that `packs/` and `scenarios/` are
//! generated from without a shell.
//!
//! Each tool is one `litigate` subcommand run through
//! [`litigate_cli::execute`], the library the `litigate` binary itself
//! calls, with `--format json`: the result is that command's own envelope
//! `{ok, op, result, diagnostics, provenance, elapsed_ms}`, unchanged. A
//! tool's arguments are declared once in [`TOOLS`]; both its input schema
//! and the command line are derived from that declaration, and arguments
//! are checked against it before anything runs (an unknown field or a
//! wrong type is an error naming the valid fields, never ignored). Paths
//! resolve against the server's working directory, as they would for the
//! CLI started there.

use rmcp::model::{JsonObject, Tool};
use serde_json::{json, Map, Value};

/// How one argument is spelled on the `litigate` command line.
#[derive(Debug, Clone, Copy)]
enum Spell {
    /// Positional, in declaration order (after `--`).
    Positional,
    /// `--flag VALUE` (repeated for each element of a list).
    Flag(&'static str),
}

/// The JSON type an argument takes.
#[derive(Debug, Clone, Copy)]
enum Kind {
    /// A string.
    Text,
    /// An array of strings.
    List,
    /// One of these strings.
    OneOf(&'static [&'static str]),
}

/// One tool argument.
#[derive(Debug, Clone, Copy)]
struct Arg {
    name: &'static str,
    kind: Kind,
    spell: Spell,
    required: bool,
    doc: &'static str,
}

/// One `litigate_*` tool: the subcommand it runs and its arguments.
#[derive(Debug, Clone, Copy)]
struct Spec {
    name: &'static str,
    command: &'static str,
    doc: &'static str,
    args: &'static [Arg],
}

const fn arg(
    name: &'static str,
    kind: Kind,
    spell: Spell,
    required: bool,
    doc: &'static str,
) -> Arg {
    Arg {
        name,
        kind,
        spell,
        required,
        doc,
    }
}

const PROFILE: Arg = arg(
    "profile",
    Kind::OneOf(&["dev", "strict"]),
    Spell::Flag("--profile"),
    false,
    "Checking profile (default `dev`; `strict` makes holes, missing tests and unresolved cites errors).",
);
const AS_OF: Arg = arg(
    "as_of",
    Kind::Text,
    Spell::Flag("--as-of"),
    false,
    "YYYY-MM-DD: select the law in force on this date (default: each package's `known_at`).",
);
const KNOWN_AT: Arg = arg(
    "known_at",
    Kind::Text,
    Spell::Flag("--known-at"),
    false,
    "YYYY-MM-DD: select the versions recorded by this date (default: each package's `known_at`).",
);
const LIB: Arg = arg(
    "lib",
    Kind::List,
    Spell::Flag("--lib"),
    false,
    "Library directories searched for packages a matter `uses` (default `[\"lit\"]`).",
);
const PATHS: Arg = arg(
    "paths",
    Kind::List,
    Spell::Positional,
    true,
    "Package directories (holding `Lit.toml`), `.lit`/`.case` files, or directories to search, e.g. `[\"lit\", \"cases\"]`.",
);

/// Every `litigate_*` tool.
const TOOLS: &[Spec] = &[
    Spec {
        name: "litigate_describe",
        command: "describe",
        doc: "The whole litigate language and CLI contract as JSON: grammar, types, builtins, every diagnostic code, commands. Start here before writing `.lit`/`.case` source.",
        args: &[],
    },
    Spec {
        name: "litigate_check",
        command: "check",
        doc: "Check `.lit` packages and `.case` matters, discharging their verification obligations. Returns diagnostics (stable code, span, fixes) and `{profile, files, obligations, packages}`; `ok: false` when any error diagnostic exists.",
        args: &[PATHS, PROFILE, AS_OF, KNOWN_AT, LIB],
    },
    Spec {
        name: "litigate_build",
        command: "build",
        doc: "Check, then write litigatables and lowered pack/links/scenario JSON under `out`. Writes files; the repository's own packs and scenarios are built by the `litgraph-packs` crate (`just lit-sync` writes a copy to target/lit-build/).",
        args: &[
            PATHS,
            arg(
                "out",
                Kind::Text,
                Spell::Flag("--out"),
                true,
                "Output directory (created if missing).",
            ),
            arg(
                "pack_files",
                Kind::List,
                Spell::Flag("--pack-file"),
                false,
                "`ID=STEM` entries: write pack ID as `packs/STEM.json`.",
            ),
            PROFILE,
            AS_OF,
            KNOWN_AT,
            LIB,
        ],
    },
    Spec {
        name: "litigate_where",
        command: "where",
        doc: "Resolve an id at any layer: `pack::node`, `node`, `decl@YYYY-MM-DD`, a diagnostic code such as `L0412`, or cite text. Returns its declaration span, versions and lowered JSON pointers.",
        args: &[
            arg(
                "id",
                Kind::Text,
                Spell::Positional,
                true,
                "The id, code or cite text to resolve.",
            ),
            arg(
                "paths",
                Kind::List,
                Spell::Flag("--path"),
                false,
                "Packages to search (default: the library).",
            ),
            LIB,
        ],
    },
    Spec {
        name: "litigate_timeline",
        command: "timeline",
        doc: "The timeline of a `.case` or `.lit` file as a `litigate.timeline/1` document (docs/litigate/TIMELINE.md, schema docs/litigate/timeline.v1.schema.json), the envelope's `result`: rows (events, deadlines with due date and status+reason, fork points) per world, alternate worlds branching at the matter's current posture, edges, and each row's layer provenance. A deadline the engine cannot compute is a row with `due: null` and status `unknown` with the reason.",
        args: &[
            arg(
                "file",
                Kind::Text,
                Spell::Positional,
                true,
                "The `.case` or `.lit` file.",
            ),
            arg(
                "as_of",
                Kind::Text,
                Spell::Flag("--as-of"),
                false,
                "YYYY-MM-DD: judge statuses at this date (default: the matter's own `as_of`; for a `.lit` file, its package's `known_at`).",
            ),
            KNOWN_AT,
            arg(
                "layers",
                Kind::List,
                Spell::Flag("--layers"),
                false,
                "Layer ids to enable (`ID` or `+ID`: only those named) or disable (`-ID`).",
            ),
            arg(
                "world",
                Kind::Text,
                Spell::Flag("--world"),
                false,
                "Only this world (e.g. `w1`) and its ancestors up to its fork row.",
            ),
            LIB,
        ],
    },
    Spec {
        name: "litigate_explain",
        command: "explain",
        doc: "The long explanation of a litigate diagnostic code (e.g. `L3004`), with its severity under each profile.",
        args: &[arg(
            "code",
            Kind::Text,
            Spell::Positional,
            true,
            "A diagnostic code such as `L3004`.",
        )],
    },
];

fn spec(name: &str) -> Option<&'static Spec> {
    TOOLS.iter().find(|s| s.name == name)
}

/// True when `name` is one of the `litigate_*` tools.
#[must_use]
pub fn is_tool(name: &str) -> bool {
    spec(name).is_some()
}

fn arg_schema(a: &Arg) -> Value {
    let mut schema = match a.kind {
        Kind::Text => json!({"type": "string"}),
        Kind::List => json!({"type": "array", "items": {"type": "string"}}),
        Kind::OneOf(values) => json!({"type": "string", "enum": values}),
    };
    schema["description"] = json!(a.doc);
    schema
}

fn input_schema(s: &Spec) -> JsonObject {
    let properties: Map<String, Value> = s
        .args
        .iter()
        .map(|a| (a.name.to_owned(), arg_schema(a)))
        .collect();
    let required: Vec<&str> = s
        .args
        .iter()
        .filter(|a| a.required)
        .map(|a| a.name)
        .collect();
    let mut schema = Map::new();
    schema.insert(
        "$schema".into(),
        json!("https://json-schema.org/draft/2020-12/schema"),
    );
    schema.insert("type".into(), json!("object"));
    schema.insert("properties".into(), Value::Object(properties));
    if !required.is_empty() {
        schema.insert("required".into(), json!(required));
    }
    schema.insert("additionalProperties".into(), json!(false));
    schema
}

/// Every `litigate_*` tool, in a fixed order.
#[must_use]
pub fn tools() -> Vec<Tool> {
    TOOLS
        .iter()
        .map(|s| Tool::new(s.name, s.doc, input_schema(s)))
        .collect()
}

/// The strings an argument value contributes, or why it is ill-typed.
fn values(a: &Arg, value: &Value) -> Result<Vec<String>, String> {
    let text = |v: &Value| v.as_str().map(str::to_owned);
    match (a.kind, value) {
        (Kind::List, Value::Array(items)) => items
            .iter()
            .map(|v| text(v).ok_or_else(|| format!("`{}` must be an array of strings", a.name)))
            .collect(),
        (Kind::List, _) => Err(format!("`{}` must be an array of strings", a.name)),
        (Kind::Text, v) => text(v)
            .map(|t| vec![t])
            .ok_or_else(|| format!("`{}` must be a string", a.name)),
        (Kind::OneOf(allowed), v) => text(v)
            .filter(|t| allowed.contains(&t.as_str()))
            .map(|t| vec![t])
            .ok_or_else(|| format!("`{}` must be one of {allowed:?}", a.name)),
    }
}

/// The `litigate` command line for tool `s` called with `args`: global
/// flags, the subcommand, its flags, then `--` and the positionals (so a
/// positional starting with `-` is never read as a flag).
fn argv(s: &Spec, args: &JsonObject) -> Result<Vec<String>, String> {
    let valid = || {
        s.args
            .iter()
            .map(|a| format!("`{}`", a.name))
            .collect::<Vec<_>>()
            .join(", ")
    };
    if let Some(unknown) = args.keys().find(|k| !s.args.iter().any(|a| a.name == *k)) {
        return Err(match s.args {
            [] => format!("unknown field `{unknown}`: `{}` takes no arguments", s.name),
            _ => format!("unknown field `{unknown}`; valid fields: {}", valid()),
        });
    }
    let mut flags = Vec::new();
    let mut positionals = Vec::new();
    for a in s.args {
        let given = match args.get(a.name) {
            Some(v) => values(a, v)?,
            None if a.required => return Err(format!("missing required field `{}`", a.name)),
            None => Vec::new(),
        };
        if a.required && given.is_empty() {
            return Err(format!("`{}` must not be empty", a.name));
        }
        match a.spell {
            Spell::Positional => positionals.extend(given),
            Spell::Flag(flag) => {
                flags.extend(given.into_iter().flat_map(|v| [flag.to_owned(), v]));
            }
        }
    }
    let head = ["litigate", "--format", "json", s.command].map(str::to_owned);
    let tail = if positionals.is_empty() {
        Vec::new()
    } else {
        std::iter::once("--".to_owned())
            .chain(positionals)
            .collect()
    };
    Ok(head.into_iter().chain(flags).chain(tail).collect())
}

/// A litigate-style failure envelope for a call that never reached a
/// subcommand (bad arguments, or a usage/I-O failure with no envelope).
fn failure(op: &str, message: &str) -> Value {
    json!({
        "ok": false,
        "op": op,
        "error": {"code": "usage", "message": message.trim()},
    })
}

/// Runs tool `name` with `arguments`: `(envelope, is_error)`, where
/// `is_error` is a non-zero `litigate` exit (an error diagnostic, or a
/// usage/I-O failure). `None` for a name that is not a `litigate_*` tool.
/// Never panics.
#[must_use]
pub fn call(name: &str, arguments: Option<JsonObject>) -> Option<(Value, bool)> {
    let s = spec(name)?;
    let args = arguments.unwrap_or_default();
    let line = match argv(s, &args) {
        Ok(line) => line,
        Err(message) => return Some((failure(s.command, &message), true)),
    };
    let printed = litigate_cli::execute(line);
    let envelope = serde_json::from_str::<Value>(&printed.stdout)
        .unwrap_or_else(|_| failure(s.command, &printed.stderr));
    Some((envelope, printed.code != 0))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn obj(v: Value) -> JsonObject {
        match v {
            Value::Object(m) => m,
            _ => Map::new(),
        }
    }

    fn line(name: &str, v: Value) -> Result<Vec<String>, String> {
        argv(spec(name).ok_or("no such tool")?, &obj(v))
    }

    #[test]
    fn every_tool_is_listed_with_a_closed_schema() {
        let names: Vec<String> = tools().iter().map(|t| t.name.to_string()).collect();
        assert_eq!(
            names,
            [
                "litigate_describe",
                "litigate_check",
                "litigate_build",
                "litigate_where",
                "litigate_timeline",
                "litigate_explain"
            ]
        );
        for tool in tools() {
            assert_eq!(tool.input_schema["additionalProperties"], json!(false));
            assert!(is_tool(&tool.name));
        }
        assert!(!is_tool("describe"));
    }

    #[test]
    fn required_fields_are_in_the_schema() {
        let build = tools()
            .into_iter()
            .find(|t| t.name == "litigate_build")
            .map(|t| t.input_schema["required"].clone());
        assert_eq!(build, Some(json!(["paths", "out"])));
    }

    #[test]
    fn arguments_become_flags_then_positionals_after_a_separator() {
        assert_eq!(
            line(
                "litigate_check",
                json!({"paths": ["lit", "-odd"], "profile": "strict", "lib": ["a", "b"]})
            ),
            Ok([
                "litigate",
                "--format",
                "json",
                "check",
                "--profile",
                "strict",
                "--lib",
                "a",
                "--lib",
                "b",
                "--",
                "lit",
                "-odd"
            ]
            .map(str::to_owned)
            .to_vec())
        );
        assert_eq!(
            line("litigate_describe", json!({})),
            Ok(["litigate", "--format", "json", "describe"]
                .map(str::to_owned)
                .to_vec())
        );
    }

    #[test]
    fn bad_arguments_are_rejected_before_running() {
        let cases = [
            (
                "litigate_check",
                json!({}),
                "missing required field `paths`",
            ),
            ("litigate_check", json!({"paths": []}), "must not be empty"),
            (
                "litigate_check",
                json!({"paths": "lit"}),
                "array of strings",
            ),
            (
                "litigate_check",
                json!({"paths": ["lit"], "profile": "loose"}),
                "one of",
            ),
            (
                "litigate_check",
                json!({"paths": ["lit"], "pofile": "dev"}),
                "valid fields: `paths`",
            ),
            ("litigate_explain", json!({"code": 3}), "must be a string"),
            ("litigate_describe", json!({"x": 1}), "takes no arguments"),
        ];
        for (name, args, expected) in cases {
            let error = line(name, args.clone()).err().unwrap_or_default();
            assert!(error.contains(expected), "{name} {args}: {error}");
        }
    }

    #[test]
    fn explain_returns_the_litigate_envelope() {
        let (envelope, is_error) =
            call("litigate_explain", Some(obj(json!({"code": "L3004"})))).unwrap_or_default();
        assert!(!is_error, "{envelope}");
        assert_eq!(envelope["ok"], json!(true));
        assert_eq!(envelope["op"], json!("explain"));
        assert_eq!(envelope["result"]["code"], json!("L3004"));
    }

    #[test]
    fn timeline_returns_the_timeline_document() {
        assert_eq!(
            line(
                "litigate_timeline",
                json!({"file": "a.case", "layers": ["-x"], "world": "w1"})
            ),
            Ok([
                "litigate", "--format", "json", "timeline", "--layers", "-x", "--world", "w1",
                "--", "a.case"
            ]
            .map(str::to_owned)
            .to_vec())
        );
        let (envelope, is_error) = call(
            "litigate_timeline",
            Some(obj(json!({"file": "/no/such/matter.case"}))),
        )
        .unwrap_or_default();
        assert!(is_error, "{envelope}");
        assert_eq!(envelope["ok"], json!(false));
    }

    #[test]
    fn bad_arguments_come_back_as_an_error_envelope() {
        let (envelope, is_error) =
            call("litigate_explain", Some(obj(json!({})))).unwrap_or_default();
        assert!(is_error);
        assert_eq!(envelope["ok"], json!(false));
        assert_eq!(envelope["error"]["code"], json!("usage"));
        assert!(call("nope", None).is_none());
    }

    #[test]
    fn a_missing_path_is_a_failure_not_a_panic() {
        let (envelope, is_error) = call(
            "litigate_check",
            Some(obj(json!({"paths": ["/no/such/litigate-dir"]}))),
        )
        .unwrap_or_default();
        assert!(is_error, "{envelope}");
        assert_eq!(envelope["ok"], json!(false));
    }
}
