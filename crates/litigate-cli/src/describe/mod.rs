//! `litigate describe`: the whole language and CLI contract as JSON, so one
//! call orients an agent (DESIGN §8). Grammar, types, and obligations are
//! read from the compiled-in SPEC; commands from the CLI definition itself.
mod constructs;
pub mod spec;

use crate::cli::Cli;
use litigate_verify::Obligation;

use clap::CommandFactory;
use litigate_base::DiagnosticCode;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// One EBNF production.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Production {
    /// Nonterminal.
    pub name: String,
    /// Right-hand side, whitespace-normalized.
    pub rule: String,
    /// The SPEC's `(* ... *)` remark, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

/// One language construct.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Construct {
    /// Keyword or block name.
    pub name: String,
    /// `lit` or `case`.
    pub file: String,
    /// What it means.
    pub summary: String,
    /// Source that parses.
    pub example: String,
    /// What it lowers to.
    pub lowers_to: String,
}

/// One built-in type (SPEC §6).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TypeInfo {
    /// Type syntax.
    pub name: String,
    /// Literal forms.
    pub literal: String,
    /// Permitted operations.
    pub operations: String,
}

/// One builtin function.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Builtin {
    /// Name.
    pub name: String,
    /// Signature.
    pub signature: String,
}

/// One diagnostic code.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CodeInfo {
    /// `L0001`.
    pub code: String,
    /// Short explanation.
    pub explain: String,
    /// Compiler phase.
    pub phase: String,
    /// Severity under `dev`.
    pub dev: String,
    /// Severity under `strict`.
    pub strict: String,
}

/// One verification obligation (SPEC §7).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ObligationInfo {
    /// Name.
    pub obligation: String,
    /// Definition.
    pub definition: String,
    /// Codes a failure emits.
    pub codes: String,
}

/// One declaration of the stdlib index.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct StdlibDecl {
    /// Local id.
    pub id: String,
    /// Kind.
    pub kind: String,
    /// In force from.
    pub from: Option<String>,
    /// In force until (exclusive).
    pub to: Option<String>,
}

/// One module of the stdlib index.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct StdlibModule {
    /// Dotted module name.
    pub name: String,
    /// Declarations.
    pub declarations: Vec<StdlibDecl>,
}

/// One package of the stdlib index.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct StdlibPackage {
    /// Name.
    pub name: String,
    /// Version.
    pub version: String,
    /// Directory.
    pub root: String,
    /// litgraph pack id (empty for a links package).
    pub pack_id: String,
    /// Error diagnostics (modules are listed only when zero).
    pub errors: usize,
    /// Modules.
    pub modules: Vec<StdlibModule>,
}

/// One CLI argument.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ArgInfo {
    /// Name (`--long` for options).
    pub name: String,
    /// Required.
    pub required: bool,
    /// Takes several values.
    pub multiple: bool,
    /// Help text.
    pub help: String,
    /// Default value.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default: Option<String>,
    /// Allowed values.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub values: Vec<String>,
}

/// One subcommand.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CommandInfo {
    /// Name.
    pub name: String,
    /// What it does.
    pub about: String,
    /// Arguments.
    pub args: Vec<ArgInfo>,
    /// Shape of `result`.
    pub result: String,
}

/// The envelope and exit-code contract.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EnvelopeInfo {
    /// Envelope fields.
    pub fields: Vec<String>,
    /// Diagnostic shape.
    pub diagnostic: String,
    /// Exit code meanings.
    pub exit_codes: Vec<String>,
    /// `--format` behaviour.
    pub formats: String,
}

/// The `describe` result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Describe {
    /// Language version.
    pub language_version: u32,
    /// Compiler version.
    pub compiler_version: String,
    /// EBNF (SPEC §2-§3).
    pub grammar: Vec<Production>,
    /// Constructs with examples.
    pub constructs: Vec<Construct>,
    /// Built-in types (SPEC §6).
    pub types: Vec<TypeInfo>,
    /// Builtin functions.
    pub builtins: Vec<Builtin>,
    /// Every diagnostic code.
    pub diagnostic_codes: Vec<CodeInfo>,
    /// Verification obligations.
    pub obligations: Vec<ObligationInfo>,
    /// A complete `.case` matter showing every block.
    pub example_case: String,
    /// Packages under the library directory (`lit`).
    pub stdlib: Vec<StdlibPackage>,
    /// Subcommands.
    pub commands: Vec<CommandInfo>,
    /// Envelope contract.
    pub envelope: EnvelopeInfo,
}

const BUILTINS: [(&str, &str); 10] = [
    (
        "deadline",
        "deadline(d) -> Date: due date of deadline d from the latest matching `given` event",
    ),
    (
        "has_event",
        "has_event(Pattern) -> Bool: some event matches the pattern",
    ),
    (
        "date_of",
        "date_of(Pattern) -> Date: date of the latest event matching the pattern; unknown before one (fact guards)",
    ),
    ("prob", "prob(Dist<E>, E) -> Probability"),
    (
        "given",
        "given(Dist<E>, List<E>) -> Dist<E>: condition on a nonzero-mass subset",
    ),
    ("in", "in(Date, Interval<Date>) -> Bool"),
    (
        "intersect",
        "intersect(Interval<Date>, Interval<Date>) -> Interval<Date>",
    ),
    (
        "disjoint",
        "disjoint(Interval<Date>, Interval<Date>) -> Bool",
    ),
    ("len", "len(List<T>) -> Number"),
    ("contains", "contains(List<T>, T) -> Bool"),
];

const RESULTS: [(&str, &str); 16] = [
    ("describe", "this document"),
    (
        "lock",
        "{lockfile, changed, packages:[{name, version, digest}]}; --merge: {lockfile, changed, unresolved:[name]}",
    ),
    ("check", "{profile, files, obligations:[{obligation, passed, detail}], packages, cases:[{path, id, start, replay, deadlines}]}"),
    ("build", "{outputs:{packs, links, clock, scenarios, source_maps, litigatables}, litigatable_hash, litigatables:[{package, path, content_hash}], obligations}"),
    ("test", "{passed, failed, tests:[{package, name, cite, span, passed, expectations:[{text, passed, actual, expected, steps}]}]}"),
    ("fmt", "{files:[{path, changed}]}"),
    ("run", "litgraph's response verbatim: {ok, api_version, op, result, warnings, provenance, elapsed_ms}; exit 2 when it is ok:false"),
    ("calendar", "{case, as_of, written, events:[{uid, deadline, due, status, trigger, trigger_date, authority, counting, counting_cite}], unscheduled:[{deadline, trigger, trigger_date, status, reason}], ics}; open deadlines only unless --all; a due date that is unknown is never given one"),
    ("lint-filing", "{case, filing:{date, event, mode, party, line}, as_of, accepted, replay:[step], discharges:[{deadline, due, before, status}], triggers:[{deadline, trigger, trigger_date, due, unknown, status}], open_before}; diagnostics are the ones the filing adds (L4001 not a legal move, L4002 late); the matter must check clean first"),
    ("where", "{id, kind, package, declaration_span, versions, edges, overrides, source, lowered:[json_pointer]} (or {kind: cite, cited_by} / {kind: diagnostic, ...})"),
    ("explain", "{code, explanation, phase, severity:{dev, strict}}"),
    ("explain-value", "{value, type, id, steps:[{rule, cite, detail}], versions, overrides}"),
    ("diff", "{equal, files:[{a, b, kind, equal, changes}], changes:[{kind, pointer, id, before, after, document}], unmatched}"),
    ("query", "{matches:[{id, kind, label, span, cite, in_force, detail}], total, next_cursor}"),
    ("import", "{files:[{path, from}], warnings}"),
    ("fix", "{fixes:[{code, message?, edits:[{span, replacement}], applicability}], changed_files, skipped_overlapping, dry_run}; a fix's edits apply together or not at all"),
];

fn commands() -> Vec<CommandInfo> {
    Cli::command()
        .get_subcommands()
        .map(|sub| CommandInfo {
            name: sub.get_name().to_owned(),
            about: sub.get_about().map(ToString::to_string).unwrap_or_default(),
            args: sub
                .get_arguments()
                .filter(|a| a.get_id() != "help")
                .map(|a| ArgInfo {
                    name: a
                        .get_long()
                        .map_or_else(|| a.get_id().to_string(), |l| format!("--{l}")),
                    required: a.is_required_set(),
                    multiple: matches!(a.get_action(), clap::ArgAction::Append),
                    help: a.get_help().map(ToString::to_string).unwrap_or_default(),
                    default: a
                        .get_default_values()
                        .first()
                        .map(|v| v.to_string_lossy().into_owned()),
                    values: a
                        .get_possible_values()
                        .iter()
                        .map(|v| v.get_name().to_owned())
                        .collect(),
                })
                .collect(),
            result: RESULTS
                .iter()
                .find(|(n, _)| *n == sub.get_name())
                .map(|(_, r)| (*r).to_owned())
                .unwrap_or_default(),
        })
        .collect()
}

fn stdlib() -> Vec<StdlibPackage> {
    let dirs: Vec<String> = crate::ws::library_dirs(&[])
        .iter()
        .map(|p| p.to_string_lossy().into_owned())
        .collect();
    let Ok(targets) = crate::ws::discover(&dirs) else {
        return Vec::new();
    };
    let targets = crate::ws::Targets {
        cases: Vec::new(),
        ..targets
    };
    let Ok(session) = crate::ws::Session::load(&targets, &[]) else {
        return Vec::new();
    };
    session
        .packages
        .iter()
        .map(|p| StdlibPackage {
            name: p.name.clone(),
            version: p.version.clone(),
            root: p.root.clone(),
            pack_id: p
                .check
                .package
                .as_ref()
                .map(|h| h.manifest.pack_id.clone())
                .unwrap_or_default(),
            errors: p
                .check
                .diagnostics
                .iter()
                .filter(|d| d.severity == litigate_base::Severity::Error)
                .count(),
            modules: p
                .check
                .package
                .iter()
                .flat_map(|h| h.modules.iter())
                .map(|m| StdlibModule {
                    name: m.name.clone(),
                    declarations: m
                        .declarations
                        .iter()
                        .filter_map(|d| {
                            let meta = litigate_sema::temporal::world::meta_of(d);
                            Some(StdlibDecl {
                                id: crate::hirx::decl_id(d)?,
                                kind: crate::hirx::decl_kind(d).to_owned(),
                                from: meta.map(|m| m.effective.from.clone()),
                                to: meta.and_then(|m| m.effective.to.clone()),
                            })
                        })
                        .collect(),
                })
                .collect(),
        })
        .collect()
}

/// The typed `describe` document.
#[must_use]
pub fn document() -> Describe {
    let table = spec::obligations();
    Describe {
        language_version: crate::prov::LANGUAGE_VERSION,
        compiler_version: env!("CARGO_PKG_VERSION").to_owned(),
        grammar: spec::grammar(),
        constructs: constructs::all(),
        types: spec::types(),
        builtins: BUILTINS.iter().map(|(n, s)| Builtin { name: (*n).to_owned(), signature: (*s).to_owned() }).collect(),
        diagnostic_codes: DiagnosticCode::ALL
            .iter()
            .map(|c| CodeInfo {
                code: c.as_str().to_owned(),
                explain: c.explain().to_owned(),
                phase: crate::cmd::phase(*c).to_owned(),
                dev: crate::render::severity_word(c.severity(litigate_base::Profile::Dev)).to_owned(),
                strict: crate::render::severity_word(c.severity(litigate_base::Profile::Strict)).to_owned(),
            })
            .collect(),
        obligations: Obligation::ALL
            .iter()
            .map(|o| {
                let name = serde_json::to_value(o).ok().and_then(|v| v.as_str().map(str::to_owned)).unwrap_or_default();
                let row = table
                    .iter()
                    .find(|(n, _, _)| *n == name)
                    .map(|(_, d, c)| (d.clone(), c.clone()));
                let (definition, codes) = row.unwrap_or_default();
                ObligationInfo {
                    definition,
                    codes,
                    obligation: name,
                }
            })
            .collect(),
        example_case: constructs::CASE_EXAMPLE.to_owned(),
        stdlib: stdlib(),
        commands: commands(),
        envelope: EnvelopeInfo {
            fields: ["schema_version", "ok", "op", "result", "diagnostics", "provenance", "elapsed_ms"].map(str::to_owned).to_vec(),
            diagnostic: "schema_version 2: {code, severity, span:{file, line, col, end_line, end_col}, message, labels:[{span, message}], notes:[{message}], fixes:[{message?, edits:[{span, replacement}], applicability}], derivation?:{rule, fact, span?, premises}, fingerprint, profile?, world:{name, as_of?, known_at?}}; sorted by (file, line, col, code); a well-formed code (L and four digits) outside the registry decodes; every L3xxx error carries a derivation; ok is false iff an error diagnostic exists, and result is kept alongside errors".to_owned(),
            exit_codes: ["0: ok", "2: the input ran and failed (error diagnostics, or a litgraph ok:false from run)", "1: usage or I/O failure (message on stderr, no envelope)"].map(str::to_owned).to_vec(),
            formats: "--format json|human; default human on a terminal, json otherwise".to_owned(),
        },
    }
}

/// `describe` as JSON.
#[must_use]
pub fn describe() -> Value {
    serde_json::to_value(document()).unwrap_or_default()
}

/// JSON Schema of the `describe` result.
#[must_use]
pub fn schema() -> Value {
    serde_json::to_value(schemars::schema_for!(Describe)).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builtins_match_the_checker() {
        let ours: Vec<&str> = BUILTINS.iter().map(|b| b.0).collect();
        assert_eq!(ours, litigate_sema::types::builtins::NAMES);
    }

    #[test]
    fn every_command_has_a_result_shape_and_every_obligation_a_row() {
        let d = document();
        let shapeless: Vec<&str> = d
            .commands
            .iter()
            .filter_map(|c| c.result.is_empty().then_some(c.name.as_str()))
            .collect();
        assert_eq!(shapeless, Vec::<&str>::new());
        assert_eq!(d.commands.len(), RESULTS.len());
        assert!(
            d.obligations.iter().all(|o| !o.definition.is_empty()),
            "{:?}",
            d.obligations
        );
        assert_eq!(d.diagnostic_codes.len(), DiagnosticCode::ALL.len());
    }
}
