//! `litigate import JSON... --out DIR` (SPEC §8.7): legacy JSON to source.
//! Nothing is written unless the generated source checks and lowers back
//! to JSON semantically equal (§8.6) to every input; otherwise L6002.
use super::build::{lower_all, write_atomic};
use super::check::check_session;
use crate::out::{Failure, Outcome, Report};
use crate::ws::{self, CaseText, Session};
use litigate_base::{Diagnostic, DiagnosticCode as C, Note, Profile, Severity, Span};
use litigate_import::{Imported, Input};
use litigate_lower::Options;
use litigate_sema::pkg::load::{LoadedFile, LoadedPackage};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::path::Path;

/// Imported files grouped into in-memory packages (`lit/<dir>/`) and matters.
fn loaded(imported: &Imported) -> (Vec<LoadedPackage>, Vec<CaseText>) {
    let mut packages: BTreeMap<String, LoadedPackage> = BTreeMap::new();
    let mut cases = Vec::new();
    for file in &imported.files {
        if is_case(&file.path) {
            cases.push(CaseText {
                path: file.path.clone(),
                text: file.text.clone(),
            });
            continue;
        }
        let Some((dir, relative)) = file
            .path
            .strip_prefix("lit/")
            .and_then(|r| r.split_once('/'))
        else {
            continue;
        };
        let pkg = packages
            .entry(dir.to_owned())
            .or_insert_with(|| LoadedPackage {
                root: format!("lit/{dir}"),
                manifest: String::new(),
                lock: None,
                files: Vec::new(),
            });
        if relative == "Lit.toml" {
            pkg.manifest.clone_from(&file.text);
        } else {
            pkg.files.push(LoadedFile {
                relative: relative.to_owned(),
                text: file.text.clone(),
            });
        }
    }
    (packages.into_values().collect(), cases)
}

/// True for a `.case` path.
fn is_case(path: &str) -> bool {
    Path::new(path)
        .extension()
        .is_some_and(|e| e.eq_ignore_ascii_case("case"))
}

/// The generated document an input must equal.
fn counterpart(text: &str) -> Option<String> {
    let value: Value = serde_json::from_str(text).ok()?;
    let id = value.get("id").and_then(Value::as_str);
    if value.get("nodes").is_some() {
        Some(format!("packs/{}.json", id?))
    } else if value.get("links").is_some() || value.get("instances").is_some() {
        Some("packs/links.json".to_owned())
    } else {
        Some(format!("scenarios/{}.json", id?))
    }
}

fn start(file: &str) -> Span {
    Span {
        file: file.into(),
        line: 1,
        col: 1,
        end_line: 1,
        end_col: 1,
    }
}

/// True when a diagnostic in `file` belongs to generated file `generated`:
/// the file itself, or any file of the package directory it lies in.
fn owns(generated: &str, file: &str) -> bool {
    if is_case(generated) {
        return generated == file;
    }
    generated
        .strip_prefix("lit/")
        .and_then(|r| r.split_once('/'))
        .is_some_and(|(dir, _)| file.starts_with(&format!("lit/{dir}/")))
}

/// Round-trip diagnostics: generated source that does not check, or does
/// not lower back to its input.
/// One L6002 per input whose generated source has check errors.
fn unchecked(
    imported: &Imported,
    inputs: &[Input],
    checked: &super::check::Checked,
) -> Vec<Diagnostic> {
    // `Lit.lock` is written by `litigate lock`, never by import (SPEC §4.2).
    let errors: Vec<&Diagnostic> = checked
        .diagnostics
        .iter()
        .filter(|d| d.severity == Severity::Error && d.code != C::StaleLock)
        .collect();
    inputs
        .iter()
        .filter_map(|input| {
            let generated: Vec<&str> = imported
                .files
                .iter()
                .filter(|f| f.from == input.path)
                .map(|f| f.path.as_str())
                .collect();
            let own: Vec<Note> = errors
                .iter()
                .filter(|d| generated.iter().any(|g| owns(g, &d.span.file.0)))
                .map(|d| Note {
                    span: Some(d.span.clone()),
                    message: format!("{}: {}", d.code.as_str(), d.message),
                })
                .collect();
            (!own.is_empty()).then(|| Diagnostic {
                notes: own,
                ..Diagnostic::error(
                    C::SemanticDiff,
                    start(&input.path),
                    "the imported source does not check",
                )
            })
        })
        .collect()
}

fn round_trip(imported: &Imported, inputs: &[Input]) -> Vec<Diagnostic> {
    let (packages, cases) = loaded(imported);
    let checked = check_session(Session::from_loaded(&packages, cases), Profile::Dev, None);
    let mut out = unchecked(imported, inputs, &checked);
    if !out.is_empty() {
        return out;
    }
    let built = lower_all(&checked, &Options::default());
    out.extend(built.diagnostics);
    let by_name: BTreeMap<&str, &str> = built
        .files
        .iter()
        .map(|(n, t)| (n.as_str(), t.as_str()))
        .collect();
    for input in inputs {
        let Some(name) = counterpart(&input.text) else {
            continue;
        };
        let Some(text) = by_name.get(name.as_str()) else {
            out.push(Diagnostic::error(
                C::SemanticDiff,
                start(&input.path),
                format!("the imported source does not lower to `{name}`"),
            ));
            continue;
        };
        match litigate_lower::semantic_diff_json(text, &input.text) {
            Ok((_, changes)) if changes.is_empty() => {}
            Ok((_, changes)) => out.push(Diagnostic {
                notes: changes
                    .iter()
                    .map(|c| Note {
                        span: None,
                        message: format!("{}: {:?} -> {:?}", c.pointer, c.before, c.after),
                    })
                    .collect(),
                ..Diagnostic::error(
                    C::SemanticDiff,
                    start(&input.path),
                    format!(
                        "the imported source lowers to JSON with {} difference(s)",
                        changes.len()
                    ),
                )
            }),
            Err(message) => out.push(Diagnostic::error(
                C::SemanticDiff,
                start(&input.path),
                message,
            )),
        }
    }
    out
}

/// `litigate import`.
pub fn import(paths: &[String], out: &str) -> Outcome {
    let inputs = paths
        .iter()
        .map(|p| {
            let text = std::fs::read_to_string(p).map_err(|e| Failure::new(format!("{p}: {e}")))?;
            Ok(Input::new(ws::shown(Path::new(p)), text))
        })
        .collect::<Result<Vec<_>, Failure>>()?;
    let mut report = Report::of(Value::Null);
    report.sources = inputs
        .iter()
        .map(|i| (i.path.clone(), i.text.clone()))
        .collect();
    let imported = match litigate_import::import(&inputs) {
        Ok(imported) => imported,
        Err(diagnostics) => {
            report.diagnostics = diagnostics;
            return Ok(report);
        }
    };
    report.diagnostics = round_trip(&imported, &inputs);
    if report.has_errors() {
        return Ok(report);
    }
    for file in &imported.files {
        write_atomic(&Path::new(out).join(&file.path), &file.text)?;
    }
    let holes = imported
        .files
        .iter()
        .map(|f| f.text.matches("?(").count())
        .sum::<usize>();
    report.result = json!({
        "files": imported.files.iter().map(|f| json!({"path": f.path, "from": f.from})).collect::<Vec<_>>(),
        "warnings": [
            "Lit.toml `in_force` and `authority` are placeholders (legacy JSON records neither)",
            format!("{holes} cite or value hole(s) carry legacy text for the porter to replace"),
            "imported matters assert `start`; replace it with `events` to derive posture",
        ],
    });
    Ok(report)
}
