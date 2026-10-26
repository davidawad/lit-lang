//! `litigate engine`: the engine's raw JSON-in/JSON-out interface over
//! packs and named scenarios, for debugging, tools, MCP and wasm. It was
//! the `litgraph` binary, which is now a deprecated shim over [`execute`].
//!
//! ```text
//! litigate engine describe                         capabilities, metrics, variables
//! litigate engine schema <request|response|scenario|pack|links>   JSON Schema of a document
//! litigate engine validate <file|->                check a pack, links, scenario or request
//! litigate engine q '<request json>' | -           run any request
//! litigate engine run <request.json>               run a request file
//! litigate engine <op> [--packs a,b] [--scenario f.json] [--set k=v]... [--arg k=v]...
//! ```
//!
//! It prints the engine response itself, never a `litigate` envelope.
//! Exit status: 0 ok, 2 the request ran and failed (`ok: false` /
//! invalid), 1 usage or I/O error.
use crate::cmd::run::{compile, kv, params, scalar};
use crate::out::{Failure, Printed};
use clap::{CommandFactory, FromArgMatches, Parser};
use litigate_engine::api::{self, Catalog, EmbeddedFiles};
use litigate_sema::pkg::source::{Disk, Source};
use serde_json::{json, Map, Value};
use std::ffi::OsString;
use std::io::Read as _;
use std::path::Path;

#[derive(Debug, Parser)]
#[command(
    name = "litigate engine",
    version = crate::VERSION,
    about = "The engine's raw interface: litigation procedure graphs as stochastic games over packs and named scenarios. JSON in, JSON out.",
    after_help = "Ops: describe packs lint validate graph metric explain solve chain simulate path pareto sweep tornado calibration posterior voi structure deadlines settle compare batch.\nRun `litigate engine describe` for the full machine-readable manual; `litigate <op> CASE` runs an op on a compiled matter."
)]
struct Cli {
    /// `q`, `run`, `validate`, `schema`, or an op name.
    command: String,
    /// Request JSON / file for `q` and `run`, file for `validate`, kind for `schema` (`-` = stdin).
    input: Option<String>,
    /// Comma-separated pack ids (default: all, or the named scenario's packs).
    #[arg(long)]
    packs: Option<String>,
    /// Scenario: a name from the library (`describe`), a JSON file, or inline JSON.
    #[arg(long)]
    scenario: Option<String>,
    /// Scenario parameter, e.g. `--set rate=900` (repeatable). Composes with
    /// a named `--scenario` via `extends`.
    #[arg(long = "set", value_name = "K=V")]
    set: Vec<String>,
    /// Op argument, e.g. `--arg node=answer-due --arg metrics=dollars,elapsed` (repeatable).
    #[arg(long = "arg", value_name = "K=V")]
    args: Vec<String>,
    /// Document kind for `validate` (default: detected).
    #[arg(long)]
    kind: Option<String>,
    /// Packs directory. Precedence: this flag, else `$LITGRAPH_PACKS` (empty
    /// counts as unset), else the packs compiled from `--lib`. A directory
    /// that does not exist, or holds no pack, is an error naming its source.
    #[arg(long)]
    packs_dir: Option<String>,
    /// Scenario library directory. Precedence: this flag, else
    /// `$LITGRAPH_SCENARIOS`, else the matters compiled from `--cases`.
    #[arg(long)]
    scenarios_dir: Option<String>,
    /// Calibration directory (default: `$LITGRAPH_CALIBRATION`, else the sets built in);
    /// sets `$LITGRAPH_CALIBRATION` for this process so `scenario.calibration` picks it up.
    #[arg(long)]
    calibration_dir: Option<String>,
    /// Library directory compiled for the packs (repeatable; default `lit`).
    #[arg(long = "lib", value_name = "DIR")]
    lib: Vec<String>,
    /// Directory of `.case` matters compiled for the scenario library
    /// (repeatable; default `cases`). A matter with errors is left out.
    #[arg(long = "cases", value_name = "DIR")]
    cases: Vec<String>,
    /// Do not apply links.
    #[arg(long)]
    no_links: bool,
    /// Compact (single-line) output.
    #[arg(long)]
    compact: bool,
}

/// Where packs and named scenarios come from when neither a flag nor a
/// variable names their directory.
#[derive(Debug, Clone, Copy)]
pub enum Builtin {
    /// Compiled from the `--lib` packages and `--cases` matters (`litigate engine`).
    Compiled,
    /// Built in at compile time: `(packs, scenarios)` (the `litgraph` shim).
    Embedded(EmbeddedFiles, EmbeddedFiles),
}

/// Parses `args` (including the program name) as the engine's command
/// line and runs it; `builtin` supplies whichever of the packs and the
/// scenario library no flag or variable names. Never panics on bad input.
pub fn execute<I, T>(args: I, builtin: Builtin) -> Printed
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    execute_in(&Disk, args, builtin)
}

/// [`execute`] over `src`: request, document and scenario files are read
/// from it, and [`Builtin::Compiled`] compiles its `--lib` packages and
/// `--cases` matters. Off the disk (WebAssembly) the packs and scenarios
/// are always the compiled ones: a directory flag is a usage failure.
pub fn execute_in<I, T>(src: &dyn Source, args: I, builtin: Builtin) -> Printed
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    let args: Vec<OsString> = args.into_iter().map(Into::into).collect();
    // Usage, help and `--version` name the program as invoked.
    let name = args
        .first()
        .map_or_else(String::new, |a| a.to_string_lossy().into_owned());
    let parsed = Cli::command()
        .display_name(name.clone())
        .bin_name(name.clone())
        .try_get_matches_from(args)
        .and_then(|m| Cli::from_arg_matches(&m));
    let cli = match parsed {
        Ok(cli) => cli,
        Err(error) => {
            let informational = !error.use_stderr();
            let rendered = error.render().to_string();
            let (stdout, stderr) = if informational {
                (rendered, String::new())
            } else {
                (String::new(), rendered)
            };
            return Printed {
                stdout,
                stderr,
                code: u8::from(!informational),
            };
        }
    };
    match run(src, &cli, builtin) {
        Ok((stdout, ok)) => Printed {
            stdout,
            stderr: String::new(),
            code: if ok { 0 } else { 2 },
        },
        Err(Failure(message)) => Printed {
            stdout: String::new(),
            stderr: format!("{name}: {message}\n"),
            code: 1,
        },
    }
}

fn read_input(src: &dyn Source, input: Option<&str>) -> Result<String, Failure> {
    match input {
        None | Some("-") => {
            let mut s = String::new();
            std::io::stdin().read_to_string(&mut s)?;
            Ok(s)
        }
        Some(s) if s.trim_start().starts_with('{') => Ok(s.to_owned()),
        Some(path) => src
            .read_to_string(Path::new(path))
            .map_err(|e| Failure::new(format!("reading {path}: {e}"))),
    }
}

/// A `--scenario` value: inline JSON (`{...}`/`[...]`), an existing file
/// (read and parsed as JSON), or otherwise a bare name from the library.
fn scenario_arg(src: &dyn Source, s: &str) -> Result<Value, Failure> {
    let t = s.trim_start();
    let text = if t.starts_with('{') || t.starts_with('[') {
        s.to_owned()
    } else if src.is_file(Path::new(s)) {
        read_input(src, Some(s))?
    } else {
        return Ok(json!(s));
    };
    serde_json::from_str(&text).map_err(|e| Failure::new(format!("scenario JSON: {e}")))
}

/// Build a request from shorthand flags.
fn shorthand(src: &dyn Source, cli: &Cli) -> Result<Value, Failure> {
    let mut scenario: Value = match &cli.scenario {
        Some(s) => scenario_arg(src, s)?,
        None => json!({}),
    };
    if !cli.set.is_empty() {
        // A bare named scenario (`--scenario <name>`) becomes
        // `{"extends": "<name>"}` so `--set` composes onto it instead of
        // indexing into a JSON string.
        if let Some(name) = scenario.as_str() {
            scenario = json!({ "extends": name });
        }
        for (k, v) in params(&cli.set)? {
            scenario["params"][k] = v;
        }
    }
    if let Some(i) = &cli.input {
        return Err(Failure::new(format!(
            "unexpected argument `{i}` for op `{}`; pass op arguments as --arg k=v",
            cli.command
        )));
    }
    let mut op = Map::new();
    op.insert("op".into(), json!(cli.command));
    for a in &cli.args {
        let (k, v) = kv(a, "--arg")?;
        op.insert(k.to_owned(), scalar(v));
    }
    let packs: Vec<&str> = cli
        .packs
        .as_deref()
        .map(|p| p.split(',').collect())
        .unwrap_or_default();
    Ok(json!({ "packs": packs, "links": !cli.no_links, "scenario": scenario, "op": op }))
}

fn emit(v: &impl serde::Serialize, compact: bool) -> String {
    let s = if compact {
        serde_json::to_string(v)
    } else {
        serde_json::to_string_pretty(v)
    };
    // The engine's documents are derive-`Serialize` trees with string keys.
    format!("{}\n", s.unwrap_or_default())
}

/// `flag`, else a nonempty `$var`, names a directory.
fn named(flag: Option<&str>, var: &str) -> bool {
    flag.is_some() || std::env::var_os(var).is_some_and(|v| !v.is_empty())
}

/// Resolve packs and the scenario library independently, each from its
/// flag, else its variable, else `builtin`.
fn load_catalog(src: &dyn Source, cli: &Cli, builtin: Builtin) -> Result<Catalog, Failure> {
    // No directory or variable is read off the disk: the compiled source is all there is.
    if !src.on_disk() && matches!(builtin, Builtin::Compiled) {
        return compiled(src, &cli.lib, &cli.cases);
    }
    let packs_dir = cli.packs_dir.as_deref();
    let scenarios_dir = cli.scenarios_dir.as_deref();
    let (packs, scenarios) = match builtin {
        Builtin::Embedded(packs, scenarios) => (packs, scenarios),
        Builtin::Compiled => (&[][..], &[][..]),
    };
    let mut catalog = Catalog::resolve(
        packs_dir.map(Path::new),
        scenarios_dir.map(Path::new),
        packs,
        scenarios,
    )
    .map_err(|e| Failure::new(format!("loading packs and scenarios: {e}")))?;
    let own_packs = !named(packs_dir, "LITGRAPH_PACKS");
    let own_scenarios = !named(scenarios_dir, "LITGRAPH_SCENARIOS");
    if matches!(builtin, Builtin::Compiled) && (own_packs || own_scenarios) {
        let compiled = compiled(src, &cli.lib, &cli.cases)?;
        if own_packs {
            catalog.origin = compiled.origin;
            catalog.packs = compiled.packs;
            catalog.links = compiled.links;
            catalog.clock = compiled.clock;
            catalog.load_errors = compiled.load_errors;
        }
        if own_scenarios {
            catalog.scenarios_origin = compiled.scenarios_origin;
            catalog.scenarios = compiled.scenarios;
        }
    }
    Ok(catalog)
}

/// The packs and named scenarios `litigate engine` serves when no flag
/// names them: `$LITGRAPH_PACKS`/`$LITGRAPH_SCENARIOS`, else compiled from
/// `lib` (default `lit`) and the matters under `cases` (default `cases`).
///
/// # Errors
/// A named directory that does not load, or a usage or I/O failure
/// compiling the library.
pub fn library_catalog(lib: Vec<String>, cases: Vec<String>) -> Result<Catalog, Failure> {
    let cli = Cli {
        command: "describe".into(),
        input: None,
        packs: None,
        scenario: None,
        set: Vec::new(),
        args: Vec::new(),
        kind: None,
        packs_dir: None,
        scenarios_dir: None,
        calibration_dir: None,
        lib,
        cases,
        no_links: false,
        compact: true,
    };
    load_catalog(&Disk, &cli, Builtin::Compiled)
}

/// The catalog of the `lib` packages (default `lit`) and the `cases`
/// matters (default `cases`, when it exists) in `src`, compiled: what
/// `litigate engine` serves when no flag or variable names a directory.
/// A matter with errors is left out.
///
/// # Errors
/// Usage or I/O failures.
pub fn compiled(src: &dyn Source, lib: &[String], cases: &[String]) -> Result<Catalog, Failure> {
    let mut paths: Vec<String> = crate::ws::library_dirs_in(src, lib)
        .iter()
        .map(|d| d.to_string_lossy().into_owned())
        .collect();
    if cases.is_empty() {
        paths.extend(src.is_dir(Path::new("cases")).then(|| "cases".to_owned()));
    } else {
        paths.extend(cases.iter().cloned());
    }
    Ok(compile(src, &paths, true, lib)?.catalog)
}

/// The response text and whether it is `ok`.
fn run(src: &dyn Source, cli: &Cli, builtin: Builtin) -> Result<(String, bool), Failure> {
    if !src.on_disk() {
        let flags = [
            ("--packs-dir", &cli.packs_dir),
            ("--scenarios-dir", &cli.scenarios_dir),
            ("--calibration-dir", &cli.calibration_dir),
        ];
        if let Some((flag, _)) = flags.iter().find(|(_, dir)| dir.is_some()) {
            return Err(Failure::new(format!(
                "{flag} names a directory on disk; over an in-memory source the packs and scenarios are the compiled `--lib` and `--cases`"
            )));
        }
    }
    if let Some(d) = &cli.calibration_dir {
        std::env::set_var("LITGRAPH_CALIBRATION", d);
    }
    if cli.command == "schema" {
        let kind = cli.input.as_deref().unwrap_or("request");
        return Ok((emit(&api::schema(kind)?, cli.compact), true));
    }
    let catalog = load_catalog(src, cli, builtin)?;
    if cli.command == "validate" {
        let doc: Value = serde_json::from_str(&read_input(src, cli.input.as_deref())?)
            .map_err(|e| Failure::new(format!("document is not JSON: {e}")))?;
        let v = api::validate(&doc, cli.kind.as_deref(), &catalog);
        return Ok((emit(&v, cli.compact), v.valid));
    }
    let request = if matches!(cli.command.as_str(), "q" | "run") {
        read_input(src, cli.input.as_deref())?
    } else {
        shorthand(src, cli)?.to_string()
    };
    let resp = api::handle_json(&request, &catalog);
    Ok((emit(&resp, cli.compact), resp.ok))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A `Cli` with `command` and every flag at its default; override
    /// fields with struct-update syntax.
    fn cli(command: &str) -> Cli {
        Cli {
            command: command.into(),
            input: None,
            packs: None,
            scenario: None,
            set: vec![],
            args: vec![],
            kind: None,
            packs_dir: None,
            scenarios_dir: None,
            calibration_dir: None,
            lib: vec![],
            cases: vec![],
            no_links: false,
            compact: false,
        }
    }

    fn message<T: std::fmt::Debug>(r: Result<T, Failure>) -> String {
        r.err().map(|f| f.0).unwrap_or_default()
    }

    #[test]
    fn scenario_arg_reads_inline_json_a_file_or_falls_back_to_a_bare_name() {
        assert_eq!(
            scenario_arg(&Disk, r#"{"params":{"rate":9}}"#).ok(),
            Some(json!({"params": {"rate": 9}}))
        );
        assert_eq!(scenario_arg(&Disk, "[1]").ok(), Some(json!([1])));
        assert!(message(scenario_arg(&Disk, "{not json")).starts_with("scenario JSON: "));
        let file = std::env::temp_dir().join(format!(
            "litigate-engine-scenario-arg-{}.json",
            std::process::id()
        ));
        let _ = std::fs::write(&file, r#"{"params":{"rate":7}}"#);
        assert_eq!(
            scenario_arg(&Disk, &file.to_string_lossy()).ok(),
            Some(json!({"params": {"rate": 7}}))
        );
        let _ = std::fs::remove_file(&file);
        assert_eq!(
            scenario_arg(&Disk, "cofc-1498-patent-case").ok(),
            Some(json!("cofc-1498-patent-case"))
        );
    }

    #[test]
    fn a_scenario_file_that_is_not_text_is_an_error() {
        let file = std::env::temp_dir().join(format!(
            "litigate-engine-scenario-binary-{}.json",
            std::process::id()
        ));
        let _ = std::fs::write(&file, [0xff, 0xfe]);
        let path = file.to_string_lossy().into_owned();
        assert!(message(scenario_arg(&Disk, &path)).starts_with(&format!("reading {path}: ")));
        let _ = std::fs::remove_file(&file);
    }

    #[test]
    fn read_input_names_a_missing_file() {
        assert!(message(read_input(&Disk, Some("no/such/request.json")))
            .starts_with("reading no/such/request.json: "));
    }

    #[test]
    fn shorthand_defaults_to_every_pack_and_an_empty_scenario() {
        let req = shorthand(&Disk, &cli("solve")).unwrap_or_default();
        assert_eq!(
            req,
            json!({"packs": [], "links": true, "scenario": {}, "op": {"op": "solve"}})
        );
    }

    #[test]
    fn shorthand_splits_packs_and_applies_arg_and_no_links() {
        let req = shorthand(
            &Disk,
            &Cli {
                packs: Some("cofc,cafc".into()),
                args: vec!["node=answer-due".into(), "top=5".into()],
                no_links: true,
                ..cli("explain")
            },
        )
        .unwrap_or_default();
        assert_eq!(req["packs"], json!(["cofc", "cafc"]));
        assert_eq!(req["links"], json!(false));
        assert_eq!(
            req["op"],
            json!({"op": "explain", "node": "answer-due", "top": 5.0})
        );
        let bad = shorthand(
            &Disk,
            &Cli {
                args: vec!["node".into()],
                ..cli("explain")
            },
        );
        assert_eq!(message(bad), "--arg expects K=V, got `node`");
    }

    #[test]
    fn shorthand_composes_set_onto_a_named_or_inline_scenario() {
        let named = shorthand(
            &Disk,
            &Cli {
                scenario: Some("cofc-1498-patent-case".into()),
                set: vec!["rate=999".into()],
                ..cli("chain")
            },
        )
        .unwrap_or_default();
        assert_eq!(
            named["scenario"],
            json!({"extends": "cofc-1498-patent-case", "params": {"rate": 999.0}})
        );
        let inline = shorthand(
            &Disk,
            &Cli {
                scenario: Some(r#"{"payoffs":{"cofc::x":1}}"#.into()),
                set: vec!["rate=100".into()],
                ..cli("chain")
            },
        )
        .unwrap_or_default();
        assert_eq!(
            inline["scenario"],
            json!({"payoffs": {"cofc::x": 1}, "params": {"rate": 100.0}})
        );
        let bad = shorthand(
            &Disk,
            &Cli {
                scenario: Some("{bad".into()),
                ..cli("chain")
            },
        );
        assert!(message(bad).starts_with("scenario JSON"));
        let text = shorthand(
            &Disk,
            &Cli {
                set: vec!["rate=fast".into()],
                ..cli("chain")
            },
        );
        assert!(message(text).contains("not a number"));
    }

    #[test]
    fn shorthand_rejects_a_stray_positional_argument() {
        let e = shorthand(
            &Disk,
            &Cli {
                input: Some("stray.json".into()),
                ..cli("chain")
            },
        );
        assert!(message(e).contains("unexpected argument"));
    }

    #[test]
    fn emit_prints_compact_and_pretty() {
        assert_eq!(emit(&json!({"a": 1}), true), "{\"a\":1}\n");
        assert_eq!(emit(&json!({"a": 1}), false), "{\n  \"a\": 1\n}\n");
    }

    #[test]
    fn embedded_files_serve_what_nothing_names() {
        let printed = execute(
            ["litgraph", "describe", "--compact"],
            Builtin::Embedded(&[], &[]),
        );
        // No pack at all: the engine answers, refusing the empty catalog.
        assert_eq!(printed.code, 2, "{}", printed.stderr);
        let v: Value = serde_json::from_str(&printed.stdout).unwrap_or_default();
        assert_eq!((&v["ok"], &v["op"]), (&json!(false), &json!("describe")));
    }

    #[test]
    fn embedded_files_serve_an_in_memory_source_too() {
        let empty =
            litigate_sema::pkg::source::MemoryTree::new(std::iter::empty::<(String, String)>());
        let printed = execute_in(
            &empty,
            ["litgraph", "describe", "--compact"],
            Builtin::Embedded(&[], &[]),
        );
        assert_eq!(printed.code, 2, "{}", printed.stderr);
    }

    #[test]
    fn a_named_directory_needs_a_flag_or_a_nonempty_variable() {
        assert!(named(Some("x"), "LITIGATE_ENGINE_TEST_UNSET"));
        assert!(!named(None, "LITIGATE_ENGINE_TEST_UNSET"));
    }
}
