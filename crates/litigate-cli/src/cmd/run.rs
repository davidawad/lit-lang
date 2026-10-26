//! `litigate run TARGET --op OP` and the op subcommands (`litigate solve
//! TARGET`, ...): compile the matter or package and the packages it uses,
//! then hand the engine a catalog of exactly that output and return its
//! response verbatim as the result.
use super::build::text;
use super::check::checked_targets;
use crate::cli::OpFlags;
use crate::out::{Failure, Outcome, Report};
use litigate_base::Profile;
use litigate_engine::api::{fingerprint, handle, Catalog, Request};
use litigate_engine::clock::ClockLibrary;
use litigate_engine::scenario::NamedScenario;
use litigate_lower::{Document, Options};
use litigate_sema::pkg::source::Source;
use serde_json::{json, Map, Value};
use std::collections::BTreeSet;

/// The op object for `--op`: a JSON object, or an op name.
///
/// # Errors
/// Text that is neither.
pub fn op_value(op: &str) -> Result<Map<String, Value>, Failure> {
    if op.trim_start().starts_with('{') {
        serde_json::from_str(op).map_err(|e| Failure::new(format!("--op is not JSON: {e}")))
    } else {
        Ok(Map::from_iter([("op".to_owned(), json!(op))]))
    }
}

/// A `--arg` value: a number, `true`/`false`, JSON (`{...}`/`[...]`), or
/// else the text itself.
#[must_use]
pub fn scalar(v: &str) -> Value {
    if let Ok(x) = v.parse::<f64>() {
        return json!(x);
    }
    match v {
        "true" => json!(true),
        "false" => json!(false),
        _ if v.starts_with('{') || v.starts_with('[') => {
            serde_json::from_str(v).unwrap_or_else(|_| json!(v))
        }
        _ => json!(v),
    }
}

/// `K=V` split at the first `=`.
///
/// # Errors
/// No `=`, naming `flag`.
pub fn kv<'a>(s: &'a str, flag: &str) -> Result<(&'a str, &'a str), Failure> {
    s.split_once('=')
        .ok_or_else(|| Failure::new(format!("{flag} expects K=V, got `{s}`")))
}

/// The scenario `params` of `--set K=V` flags.
///
/// # Errors
/// A flag that is not `K=V`, or a value that is not a number.
pub fn params(set: &[String]) -> Result<Map<String, Value>, Failure> {
    set.iter()
        .map(|s| {
            let (k, v) = kv(s, "--set")?;
            let x: f64 = v.parse().map_err(|_| {
                Failure::new(format!(
                    "--set {k}: `{v}` is not a number (params are numeric)"
                ))
            })?;
            Ok((k.to_owned(), json!(x)))
        })
        .collect()
}

/// The embedded clock library with `fragment`'s rules and calendars replacing same-id entries.
fn merged(mut lib: ClockLibrary, fragment: &ClockLibrary) -> ClockLibrary {
    lib.calendars
        .retain(|c| fragment.calendars.iter().all(|f| f.id != c.id));
    lib.counting_rules
        .retain(|r| fragment.counting_rules.iter().all(|f| f.id != r.id));
    lib.calendars.extend(fragment.calendars.iter().cloned());
    lib.counting_rules
        .extend(fragment.counting_rules.iter().cloned());
    lib
}

/// Whether `p` is a links package (SPEC §8.4).
fn is_links(p: &crate::ws::Pkg) -> bool {
    p.check.package.as_ref().is_some_and(|h| h.manifest.links)
}

/// What [`compile`] made of its paths.
pub(crate) struct Compiled {
    /// Diagnostics and provenance of the compiled scope; no result yet.
    pub report: Report,
    /// The lowered packs, links, clock fragments and matter scenarios.
    pub catalog: Catalog,
    /// The scenario id of each matter that lowered, in path order.
    pub scenarios: Vec<String>,
    /// The pack id of each named package that lowered to a pack, and of
    /// each package a named links package composes (without a matter).
    pub packs: Vec<String>,
}

/// Check `paths` (matters and packages) in `src` and lower what they name
/// into one catalog; `links` also loads the links packages composing two or more
/// packages the matters use. The scope is the closure of what the matters
/// `uses` and the named packages: no other package is loaded into the
/// catalog, and a dependency of a links package outside that scope is
/// checked (the links package needs it) but never reported, so it cannot
/// set `ok: false`. A matter with errors lowers no scenario.
///
/// # Errors
/// Usage or I/O failures.
pub(crate) fn compile(
    src: &dyn Source,
    paths: &[String],
    links: bool,
    lib: &[String],
) -> Result<Compiled, Failure> {
    let mut targets = crate::ws::discover_in(src, paths)?;
    let used: BTreeSet<String> = targets
        .cases
        .iter()
        .filter_map(|p| src.read_to_string(p).ok())
        .flat_map(|t| crate::ws::uses_of(&t))
        .collect();
    if links {
        // Links compose between whichever packs are loaded, as `links.json` does.
        targets.packages.extend(crate::ws::links_packages_in(
            src,
            lib,
            &targets.cases,
            &used,
        ));
    }
    let c = checked_targets(src, &targets, Profile::Dev, &Options::default(), lib)?;
    let session = &c.session;
    let mut scope = session.closure(used);
    scope.extend(session.requested().map(|p| p.name.clone()));
    let report = Report {
        result: Value::Null,
        provenance: crate::prov::session(
            session.packages.iter().filter(|p| scope.contains(&p.name)),
            None,
            None,
        ),
        sources: session.sources(),
        diagnostics: session.scoped(&scope, c.diagnostics.clone()),
        check_render: None,
        profile: Some(Profile::Dev),
        failed: false,
    };
    let matter_packages: BTreeSet<&String> = c.cases.iter().flat_map(|m| &m.packages).collect();
    // A links package named without a matter stands for the graph it
    // composes: the packages it links.
    let linked: BTreeSet<&String> = session
        .requested()
        .filter(|p| targets.cases.is_empty() && is_links(p))
        .flat_map(|p| &p.dependencies)
        .collect();
    let mut files = Vec::new();
    let mut packs = Vec::new();
    let mut clock = ClockLibrary::embedded().0.clone();
    // Verification lowered every package at the default dates already.
    let lowered = session
        .packages
        .iter()
        .zip(&c.verdicts)
        .filter(|(p, _)| {
            p.requested || matter_packages.contains(&p.name) || linked.contains(&p.name)
        })
        .filter_map(|(p, (_, v))| Some((p, v.lowered.as_ref()?)));
    for (p, out) in lowered {
        files.push(match &out.document {
            Document::Pack(pack) => {
                if p.requested || linked.contains(&p.name) {
                    packs.push(pack.id.clone());
                }
                (format!("{}.json", pack.id), text(pack))
            }
            Document::Links(links) => ("links.json".to_owned(), text(links)),
        });
        if let Some((fragment, _)) = &out.clock {
            clock = merged(clock, fragment);
        }
    }
    // `from_files` records unreadable files in the catalog rather than failing.
    Catalog::from_files("litigate".into(), files)
        .map(|mut catalog| {
            catalog.clock = clock;
            catalog.scenarios_origin = "litigate".into();
            catalog.scenarios = c
                .cases
                .iter()
                .filter_map(|m| m.lowered.as_ref())
                .map(|lowered| named_scenario(&lowered.scenario))
                .collect();
            Compiled {
                report,
                scenarios: catalog.scenarios.iter().map(|s| s.1.id.clone()).collect(),
                catalog,
                packs,
            }
        })
        .map_err(Failure::from)
}

/// A matter's lowered scenario as a catalog entry: `(file name,
/// scenario, content fingerprint)`.
fn named_scenario(scenario: &NamedScenario) -> (String, NamedScenario, String) {
    let text = serde_json::to_string(scenario).unwrap_or_default();
    (
        format!("{}.json", scenario.id),
        scenario.clone(),
        fingerprint(text.as_bytes()),
    )
}

/// `litigate run` (and each op subcommand): `op` on `target` in `src`, a matter
/// (its scenario, with `--set` params composed onto it) or a package (its
/// pack under the default scenario with the `--set` params). A target
/// that fails its check is not run.
pub fn run(src: &dyn Source, target: &str, op: &str, flags: &OpFlags) -> Outcome {
    let mut op = op_value(op)?;
    for a in &flags.args {
        let (k, v) = kv(a, "--arg")?;
        op.insert(k.to_owned(), scalar(v));
    }
    let params = params(&flags.set)?;
    let links = !flags.no_links;
    let Compiled {
        mut report,
        catalog,
        scenarios,
        packs,
    } = compile(src, &[target.to_owned()], links, &flags.library.lib)?;
    if report.has_errors() {
        return Ok(report);
    }
    // A matter's scenario names its own packs.
    let (packs, scenario) = match (scenarios.first(), params.is_empty()) {
        (Some(id), true) => (Vec::new(), json!(id)),
        (Some(id), false) => (Vec::new(), json!({ "extends": id, "params": params })),
        (None, _) if packs.is_empty() => {
            return Err(Failure::new(format!(
                "{target}: neither a matter nor a package that lowers to a pack"
            )))
        }
        (None, _) => (packs, json!({ "params": params })),
    };
    let request = serde_json::from_value::<Request>(
        json!({ "packs": packs, "scenario": scenario, "op": op, "links": links }),
    )
    .map_err(|e| Failure::new(format!("invalid engine op: {e}")))?;
    let response = handle(&request, &catalog);
    report.failed = !response.ok;
    report.result = serde_json::to_value(&response).unwrap_or_default();
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn op_names_and_objects() {
        assert_eq!(
            op_value("chain").ok().map(Value::Object),
            Some(json!({"op": "chain"}))
        );
        assert_eq!(
            op_value(r#"{"op":"solve"}"#).ok().map(Value::Object),
            Some(json!({"op": "solve"}))
        );
        assert!(op_value("{bad").is_err());
        assert!(op_value("[1]").is_ok());
    }

    #[test]
    fn scalars_are_typed_and_fall_back_to_text() {
        assert_eq!(scalar("900"), json!(900.0));
        assert_eq!(scalar("true"), json!(true));
        assert_eq!(scalar("false"), json!(false));
        assert_eq!(scalar("[1,2]"), json!([1, 2]));
        assert_eq!(scalar("{not json"), json!("{not json"));
        assert_eq!(scalar("answer-due"), json!("answer-due"));
    }

    #[test]
    fn params_are_numeric_key_values() {
        assert_eq!(
            params(&["rate=900".into()]).ok().map(Value::Object),
            Some(json!({"rate": 900.0}))
        );
        let no_eq = params(&["rate".into()]).err().map(|f| f.0);
        assert_eq!(no_eq.as_deref(), Some("--set expects K=V, got `rate`"));
        let text = params(&["rate=fast".into()]).err().map(|f| f.0);
        assert_eq!(
            text.as_deref(),
            Some("--set rate: `fast` is not a number (params are numeric)")
        );
    }
}
