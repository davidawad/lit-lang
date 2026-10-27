//! `litigate mcp`: the toolchain as a stdio [MCP](https://modelcontextprotocol.io)
//! server, newline-delimited JSON-RPC 2.0 on standard input and output.
//!
//! - Every engine op is a tool run on a compiled `.case` matter or
//!   package, as `litigate run TARGET --op ...` runs it ([`ops`]);
//!   `litigate_engine` takes a raw engine request, for debugging.
//! - The `litigate_*` tools are the authoring toolchain: `describe`,
//!   `check`, `build`, `where`, `timeline`, `actions` (a party's legal
//!   moves) and `explain` ([`authoring`]).
//! - The library's compiled packs, links, named scenarios and the engine
//!   manual are resources under `litigate://` URIs ([`resources`]).
//!
//! A tool returns its command's own envelope as structured content (and
//! as text); a call that fails (an error diagnostic, an engine `ok:
//! false`, bad arguments) is a tool error carrying that envelope, never a
//! protocol error. The server is synchronous: one request at a time, each
//! answered before the next is read.

pub mod authoring;
pub mod ops;
pub mod resources;

use crate::out::{Failure, Outcome};
use clap::Parser;
use litigate_engine::api::Catalog;
use serde_json::{json, Map, Value};
use std::ffi::OsString;
use std::io::{BufRead, Write};
use std::process::ExitCode;
use std::sync::OnceLock;

/// The protocol versions served, newest first; the newest answers a
/// client asking for any other.
pub const PROTOCOL_VERSIONS: [&str; 4] = ["2025-11-25", "2025-06-18", "2025-03-26", "2024-11-05"];

const PARSE_ERROR: i64 = -32700;
const INVALID_REQUEST: i64 = -32600;
const METHOD_NOT_FOUND: i64 = -32601;
const INVALID_PARAMS: i64 = -32602;
const INTERNAL_ERROR: i64 = -32603;
const RESOURCE_NOT_FOUND: i64 = -32002;

const INSTRUCTIONS: &str = "Litigation procedure as a checked language, over MCP. Start with `litigate_describe` (the language and CLI contract) and the `describe` op or the `litigate://describe` resource (the engine manual). Every engine op tool (`solve`, `chain`, `simulate`, `explain`, ...) runs on a compiled matter: `{target, set?, lib?, no_links?, ...op fields}`, `target` a `.case` matter (its scenario) or a package (`.lit` file or package directory); it returns the `litigate` envelope {schema_version, ok, op, result, diagnostics, provenance, elapsed_ms} with the engine's response {ok, api_version, op, result, warnings, provenance, elapsed_ms} as `result`. `litigate_check`, `litigate_build`, `litigate_where`, `litigate_timeline` and `litigate_explain` drive the authoring language (`lit/*.lit` rules, `cases/*.case` matters). `litigate_actions {case, party, date?}` is a party's action set: only the moves replay accepts from the matter's posture on the date, each with the event line to record. `litigate_engine` runs a raw engine request over the library's compiled packs and named scenarios, for debugging. A failed call (error diagnostics, an engine `ok: false`, bad arguments) is a tool error carrying the same envelope. Paths resolve against the server's working directory.";

/// `litigate mcp`: serve the toolchain over MCP on standard input and
/// output.
#[derive(Debug, Parser)]
#[command(
    name = "litigate mcp",
    version,
    about = "Serve the litigate toolchain over MCP (stdio)"
)]
struct Cli {
    /// Library directory the tools compile packages from (repeatable;
    /// default `lit`).
    #[arg(long = "lib", value_name = "DIR")]
    lib: Vec<String>,
    /// Directory of `.case` matters whose scenarios `litigate_engine` and
    /// the resources serve (repeatable; default `cases`).
    #[arg(long = "cases", value_name = "DIR")]
    cases: Vec<String>,
}

/// The server: its library, and the catalog compiled from it on first use.
#[derive(Debug, Default)]
pub struct Server {
    lib: Vec<String>,
    cases: Vec<String>,
    catalog: OnceLock<Result<Catalog, String>>,
}

/// A JSON-RPC error: code and message.
type RpcError = (i64, String);

/// An MCP tool description.
fn tool(name: &str, description: &str, input_schema: Value) -> Value {
    let mut tool = json!({"name": name, "description": description});
    tool["inputSchema"] = input_schema;
    tool
}

/// A `litigate`-style failure envelope for a call that never produced one.
fn failure(op: &str, message: &str) -> Value {
    json!({"ok": false, "op": op, "error": {"code": "usage", "message": message.trim()}})
}

/// A command outcome as `(envelope, is_error)`, `is_error` when the
/// command would exit non-zero.
fn reply(op: &str, outcome: Outcome, elapsed_ms: f64) -> (Value, bool) {
    match outcome {
        Ok(report) => {
            let envelope = crate::out::envelope(op, &report, elapsed_ms);
            let is_error = !envelope.ok || report.failed;
            (
                serde_json::to_value(&envelope).unwrap_or_default(),
                is_error,
            )
        }
        Err(Failure(message)) => (failure(op, &message), true),
    }
}

fn invalid(message: impl Into<String>) -> RpcError {
    (INVALID_PARAMS, message.into())
}

impl Server {
    /// A server compiling packages from `lib` and matters from `cases`
    /// (each defaulting as `litigate engine` does).
    #[must_use]
    pub fn new(lib: Vec<String>, cases: Vec<String>) -> Self {
        Self {
            lib,
            cases,
            catalog: OnceLock::new(),
        }
    }

    /// The library's packs and named scenarios, compiled once, or why
    /// they did not load.
    fn catalog(&self) -> Result<&Catalog, &str> {
        self.catalog
            .get_or_init(|| {
                crate::engine::library_catalog(self.lib.clone(), self.cases.clone())
                    .map_err(|f| f.0)
            })
            .as_ref()
            .map_err(String::as_str)
    }

    /// The reply to one message, `None` for a notification (or a
    /// response, which this server never asks for).
    #[must_use]
    pub fn handle(&self, message: &str) -> Option<Value> {
        let message: Value = match serde_json::from_str(message) {
            Ok(message) => message,
            Err(e) => return Some(error(&Value::Null, (PARSE_ERROR, format!("not JSON: {e}")))),
        };
        let id = message.get("id")?;
        let Some(method) = message.get("method").and_then(Value::as_str) else {
            return Some(error(
                id,
                (INVALID_REQUEST, "a request needs a `method`".into()),
            ));
        };
        let params = message.get("params").unwrap_or(&Value::Null);
        Some(match self.dispatch(method, params) {
            Ok(result) => json!({"jsonrpc": "2.0", "id": id, "result": result}),
            Err(e) => error(id, e),
        })
    }

    fn dispatch(&self, method: &str, params: &Value) -> Result<Value, RpcError> {
        match method {
            "initialize" => Ok(initialize(params)),
            "ping" => Ok(json!({})),
            "tools/list" => {
                let mut tools = ops::tools();
                tools.extend(authoring::tools());
                Ok(json!({"tools": tools}))
            }
            "tools/call" => self.call_tool(params),
            "resources/list" => {
                let catalog = self.catalog().map_err(internal)?;
                Ok(json!({"resources": resources::list(catalog)}))
            }
            "resources/templates/list" => Ok(json!({"resourceTemplates": []})),
            "resources/read" => self.read_resource(params),
            _ => Err((METHOD_NOT_FOUND, format!("no method `{method}`"))),
        }
    }

    fn call_tool(&self, params: &Value) -> Result<Value, RpcError> {
        let name = params["name"]
            .as_str()
            .ok_or_else(|| invalid("tools/call needs a tool `name`"))?;
        let args = match &params["arguments"] {
            Value::Null => Map::new(),
            Value::Object(args) => args.clone(),
            _ => return Err(invalid("tool `arguments` must be an object")),
        };
        let (envelope, is_error) = if let Some(called) = authoring::call(name, &args) {
            called
        } else if name == ops::RAW_TOOL {
            ops::call_raw(self.catalog(), args)
        } else if ops::op_names().iter().any(|op| op == name) {
            ops::call(name, args, &self.lib)
        } else {
            return Err(invalid(format!("no tool `{name}`")));
        };
        Ok(json!({
            "content": [{"type": "text", "text": envelope.to_string()}],
            "structuredContent": envelope,
            "isError": is_error,
        }))
    }

    fn read_resource(&self, params: &Value) -> Result<Value, RpcError> {
        let uri = params["uri"]
            .as_str()
            .ok_or_else(|| invalid("resources/read needs a `uri`"))?;
        let catalog = self.catalog().map_err(internal)?;
        let contents = resources::read(catalog, uri)
            .ok_or_else(|| (RESOURCE_NOT_FOUND, format!("no resource at `{uri}`")))?;
        Ok(json!({"contents": contents}))
    }
}

fn internal(message: &str) -> RpcError {
    (INTERNAL_ERROR, format!("loading the library: {message}"))
}

fn error(id: &Value, (code, message): RpcError) -> Value {
    json!({"jsonrpc": "2.0", "id": id, "error": {"code": code, "message": message}})
}

/// The `initialize` result: the client's protocol version when served,
/// else the newest.
fn initialize(params: &Value) -> Value {
    let asked = params["protocolVersion"].as_str();
    let version = asked
        .filter(|v| PROTOCOL_VERSIONS.contains(v))
        .unwrap_or(PROTOCOL_VERSIONS[0]);
    json!({
        "protocolVersion": version,
        "capabilities": {"tools": {}, "resources": {}},
        "serverInfo": {"name": "litigate", "version": env!("CARGO_PKG_VERSION")},
        "instructions": INSTRUCTIONS,
    })
}

/// Answers each line of `input` on `output` until `input` ends.
///
/// # Errors
/// Reading `input` or writing `output` failed.
pub fn serve(
    server: &Server,
    input: &mut dyn BufRead,
    output: &mut dyn Write,
) -> std::io::Result<()> {
    for line in input.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        if let Some(reply) = server.handle(&line) {
            writeln!(output, "{reply}")?;
            output.flush()?;
        }
    }
    Ok(())
}

/// `litigate mcp ARGS` (`args` without the program name): serve on
/// standard input and output; exit 1 on a usage or I/O failure.
#[must_use]
pub fn main(args: &[OsString]) -> ExitCode {
    let (stdin, stdout) = (std::io::stdin(), std::io::stdout());
    ExitCode::from(run(
        args,
        &mut stdin.lock(),
        &mut stdout.lock(),
        &mut std::io::stderr(),
    ))
}

/// [`main`] over `input` and `output`, messages to `errors`: the exit code.
fn run(
    args: &[OsString],
    input: &mut dyn BufRead,
    output: &mut dyn Write,
    errors: &mut dyn Write,
) -> u8 {
    let argv = std::iter::once(OsString::from("litigate mcp")).chain(args.iter().cloned());
    let cli = match Cli::try_parse_from(argv) {
        Ok(cli) => cli,
        Err(e) => {
            // Help and `--version` are answers, not failures.
            let sink: &mut dyn Write = if e.use_stderr() { errors } else { &mut *output };
            let _ = write!(sink, "{}", e.render());
            return u8::from(e.use_stderr());
        }
    };
    match serve(&Server::new(cli.lib, cli.cases), input, output) {
        Ok(()) => 0,
        Err(e) => {
            let _ = writeln!(errors, "litigate mcp: {e}");
            1
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> Server {
        Server::new(
            vec!["tests/fixtures/cov-cmds/lit".into()],
            vec!["tests/fixtures/cov-cmds/cases".into()],
        )
    }

    /// No library at all: no `lit` nor `cases` under this directory.
    fn missing() -> Server {
        Server::new(vec!["no/such/lit".into()], vec!["no/such/cases".into()])
    }

    fn request(server: &Server, method: &str, params: Value) -> Value {
        let mut message = json!({"jsonrpc": "2.0", "id": 7, "method": method});
        message["params"] = params;
        server.handle(&message.to_string()).unwrap_or_default()
    }

    #[test]
    fn initialize_negotiates_a_served_version() {
        let s = Server::default();
        let asked = request(&s, "initialize", json!({"protocolVersion": "2025-06-18"}));
        assert_eq!(asked["id"], json!(7));
        assert_eq!(asked["result"]["protocolVersion"], json!("2025-06-18"));
        assert!(asked["result"]["capabilities"]["tools"].is_object());
        assert!(asked["result"]["capabilities"]["resources"].is_object());
        let newer = request(&s, "initialize", json!({"protocolVersion": "2099-01-01"}));
        assert_eq!(
            newer["result"]["protocolVersion"],
            json!(PROTOCOL_VERSIONS[0])
        );
        assert_eq!(request(&s, "ping", Value::Null)["result"], json!({}));
        let none = request(&s, "resources/templates/list", Value::Null);
        assert_eq!(none["result"], json!({"resourceTemplates": []}));
    }

    #[test]
    fn malformed_messages_are_protocol_errors_and_notifications_unanswered() {
        let s = Server::default();
        let code = |reply: Option<Value>| reply.unwrap_or_default()["error"]["code"].clone();
        assert_eq!(code(s.handle("{")), json!(PARSE_ERROR));
        assert_eq!(code(s.handle(r#"{"id": 1}"#)), json!(INVALID_REQUEST));
        assert_eq!(
            code(s.handle(r#"{"id": 1, "method": "nope"}"#)),
            json!(METHOD_NOT_FOUND)
        );
        assert!(s
            .handle(r#"{"method": "notifications/initialized"}"#)
            .is_none());
    }

    #[test]
    fn every_tool_is_listed_once() {
        let listed = request(&Server::default(), "tools/list", Value::Null);
        let names: Vec<&str> = listed["result"]["tools"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|t| t["name"].as_str())
            .collect();
        let mut unique = names.clone();
        unique.sort_unstable();
        unique.dedup();
        assert_eq!(unique.len(), names.len());
        assert_eq!(names.len(), ops::op_names().len() + 1 + 7);
        for name in ["solve", "describe", "litigate_engine", "litigate_check"] {
            assert!(names.contains(&name), "{name}");
        }
    }

    #[test]
    fn a_tool_call_returns_its_envelope() {
        let s = fixture();
        let case = "tests/fixtures/cov-cmds/cases/cov.case";
        let solved = request(
            &s,
            "tools/call",
            json!({"name": "solve", "arguments": {"target": case}}),
        );
        assert_eq!(solved["result"]["isError"], json!(false), "{solved}");
        let envelope = &solved["result"]["structuredContent"];
        assert_eq!(envelope["result"]["op"], json!("solve"));
        let text = solved["result"]["content"][0]["text"]
            .as_str()
            .unwrap_or_default();
        assert_eq!(
            serde_json::from_str::<Value>(text).ok().as_ref(),
            Some(envelope)
        );
        let explained = request(
            &s,
            "tools/call",
            json!({"name": "litigate_explain", "arguments": {"code": "L3004"}}),
        );
        assert_eq!(explained["result"]["isError"], json!(false));
        let raw = json!({"name": "litigate_engine", "arguments": {"op": {"op": "packs"}}});
        let packs = request(&s, "tools/call", raw);
        assert_eq!(
            packs["result"]["structuredContent"]["result"][0]["id"],
            json!("cov-pack")
        );
        let failed = request(&s, "tools/call", json!({"name": "solve"}));
        assert_eq!(failed["result"]["isError"], json!(true));
        let broken =
            std::env::temp_dir().join(format!("litigate-mcp-broken-{}.case", std::process::id()));
        let _ = std::fs::write(&broken, "case broken\nuses nothing-at-all ^9.9.9\n");
        let target = broken.display().to_string();
        let unchecked = request(
            &s,
            "tools/call",
            json!({"name": "chain", "arguments": {"target": target}}),
        );
        let _ = std::fs::remove_file(&broken);
        assert_eq!(unchecked["result"]["isError"], json!(true), "{unchecked}");
        let envelope = &unchecked["result"]["structuredContent"];
        assert_eq!(envelope["ok"], json!(false), "{unchecked}");
        assert_eq!(envelope["diagnostics"][0]["severity"], json!("error"));
    }

    #[test]
    fn a_bad_tool_call_is_invalid_params() {
        let s = Server::default();
        for params in [
            json!({}),
            json!({"name": "solve", "arguments": [1]}),
            json!({"name": "nope"}),
        ] {
            let reply = request(&s, "tools/call", params.clone());
            assert_eq!(reply["error"]["code"], json!(INVALID_PARAMS), "{params}");
        }
    }

    #[test]
    fn resources_list_and_read_the_compiled_library() {
        let s = fixture();
        let listed = request(&s, "resources/list", Value::Null);
        assert_eq!(
            listed["result"]["resources"].as_array().map(Vec::len),
            Some(5)
        );
        let read = request(
            &s,
            "resources/read",
            json!({"uri": "litigate://packs/cov-pack"}),
        );
        assert_eq!(
            read["result"]["contents"][0]["uri"],
            json!("litigate://packs/cov-pack")
        );
        let unknown = request(&s, "resources/read", json!({"uri": "litigate://nope"}));
        assert_eq!(unknown["error"]["code"], json!(RESOURCE_NOT_FOUND));
        let no_uri = request(&s, "resources/read", json!({}));
        assert_eq!(no_uri["error"]["code"], json!(INVALID_PARAMS));
    }

    #[test]
    fn a_library_that_does_not_load_is_an_internal_error() {
        let s = missing();
        for (method, params) in [
            ("resources/list", Value::Null),
            ("resources/read", json!({"uri": "litigate://describe"})),
        ] {
            let reply = request(&s, method, params);
            assert_eq!(reply["error"]["code"], json!(INTERNAL_ERROR), "{reply}");
        }
        let raw = request(&s, "tools/call", json!({"name": "litigate_engine"}));
        assert_eq!(raw["result"]["isError"], json!(true), "{raw}");
    }

    /// A writer failing on `write` or, with `flush_only`, on `flush`.
    struct Broken {
        flush_only: bool,
    }

    impl Write for Broken {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            if self.flush_only {
                Ok(buf.len())
            } else {
                Err(std::io::ErrorKind::BrokenPipe.into())
            }
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Err(std::io::ErrorKind::BrokenPipe.into())
        }
    }

    fn session(args: &[&str], input: &[u8]) -> (u8, String, String) {
        let args: Vec<OsString> = args.iter().map(OsString::from).collect();
        let (mut output, mut errors) = (Vec::new(), Vec::new());
        let code = run(&args, &mut { input }, &mut output, &mut errors);
        let text = |b: Vec<u8>| String::from_utf8(b).unwrap_or_default();
        (code, text(output), text(errors))
    }

    #[test]
    fn a_session_answers_each_request_line() {
        let input = b"{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"ping\"}\n\n{\"method\":\"notifications/initialized\"}\n{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"ping\"}\n";
        let (code, output, errors) = session(&[], input);
        assert_eq!((code, errors.as_str()), (0, ""));
        let ids: Vec<Value> = output
            .lines()
            .map(|l| serde_json::from_str::<Value>(l).unwrap_or_default()["id"].clone())
            .collect();
        assert_eq!(ids, [json!(1), json!(2)]);
    }

    #[test]
    fn usage_help_and_io_failures() {
        let (code, output, _) = session(&["--help"], b"");
        assert_eq!(code, 0);
        assert!(output.contains("--lib"), "{output}");
        let (code, _, errors) = session(&["--bogus"], b"");
        assert_eq!(code, 1);
        assert!(errors.contains("--bogus"), "{errors}");
        let (code, _, errors) = session(&[], b"\xff\n");
        assert_eq!(code, 1);
        assert!(errors.starts_with("litigate mcp: "), "{errors}");
        let ping = b"{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"ping\"}\n".as_slice();
        for flush_only in [false, true] {
            let served = serve(
                &Server::default(),
                &mut { ping },
                &mut Broken { flush_only },
            );
            assert!(served.is_err(), "{flush_only}");
        }
        assert_eq!(main(&[OsString::from("--version")]), ExitCode::SUCCESS);
    }
}
