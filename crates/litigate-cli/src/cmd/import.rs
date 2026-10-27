//! `litigate import JSON... --out DIR` (SPEC §8.7): legacy JSON to source.
//! Nothing is written unless the generated source checks and lowers back
//! to JSON semantically equal (§8.6) to every input; otherwise L6002. A
//! calibration set of a pack not among the inputs goes into that pack's
//! package under `DIR/lit`, and is checked there.
use super::build::{lower_all, write_atomic};
use super::check::{check_session, checked_in};
use crate::out::{Failure, Outcome, Report};
use crate::ws::{self, CaseText, Session};
use litigate_base::{Diagnostic, DiagnosticCode as C, Label, Note, Profile, Severity, Span};
use litigate_import::{Imported, Input};
use litigate_lower::Options;
use litigate_sema::pkg::load::{LoadedFile, LoadedPackage};
use litigate_sema::pkg::manifest::parse_manifest;
use litigate_sema::pkg::source::MemoryTree;
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::path::Path;

/// Pack id to package directory of every package under `out/lit`.
fn existing_packages(out: &str) -> BTreeMap<String, String> {
    let lit = Path::new(out).join("lit");
    let Ok(entries) = std::fs::read_dir(&lit) else {
        return BTreeMap::new();
    };
    entries
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let text = std::fs::read_to_string(entry.path().join("Lit.toml")).ok()?;
            let parsed = parse_manifest(&text).ok()?;
            let dir = entry.file_name().to_string_lossy().into_owned();
            Some((parsed.manifest.pack_id, dir))
        })
        .collect()
}

/// The directories of the packages `imported` generates (its `Lit.toml`s).
fn generated_packages(imported: &Imported) -> Vec<String> {
    imported
        .files
        .iter()
        .filter_map(|f| f.path.strip_prefix("lit/")?.strip_suffix("/Lit.toml"))
        .map(str::to_owned)
        .collect()
}

/// The package directory of a generated `lit/<dir>/...` file.
fn package_dir(path: &str) -> Option<&str> {
    path.strip_prefix("lit/")?
        .split_once('/')
        .map(|(dir, _)| dir)
}

/// Imported files grouped into in-memory packages (`lit/<dir>/`) and matters.
fn loaded(imported: &Imported) -> (Vec<LoadedPackage>, Vec<CaseText>) {
    let mut packages: BTreeMap<String, LoadedPackage> = BTreeMap::new();
    let mut cases = Vec::new();
    for file in &imported.files {
        if is_case(&file.path) {
            cases.push(CaseText {
                path: file.path.clone(),
                text: file.text.clone(),
                lock: None,
            });
            continue;
        }
        // Every other `Imported` path is `lit/<dir>/<file>` (its contract).
        let (dir, relative) = file
            .path
            .strip_prefix("lit/")
            .and_then(|r| r.split_once('/'))
            .unwrap_or_default();
        let pkg = packages
            .entry(dir.to_owned())
            .or_insert_with(|| LoadedPackage {
                texts: std::collections::BTreeMap::new(),
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

/// The generated document an input must equal. `import` accepted the
/// input, so it is JSON, and a pack, scenario or calibration set has an
/// `id`.
fn counterpart(text: &str) -> String {
    let value: Value = serde_json::from_str(text).unwrap_or_default();
    let id = value.get("id").and_then(Value::as_str).unwrap_or_default();
    if value.get("nodes").is_some() {
        format!("packs/{id}.json")
    } else if value.get("links").is_some() || value.get("instances").is_some() {
        "packs/links.json".to_owned()
    } else if value.get("entries").is_some() {
        format!("packs/calibration/{id}.json")
    } else {
        format!("scenarios/{id}.json")
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
            let own: Vec<Label> = errors
                .iter()
                .filter(|d| generated.iter().any(|g| owns(g, &d.span.file.0)))
                .map(|d| Label {
                    span: d.span.clone(),
                    message: format!("{}: {}", d.code.as_str(), d.message),
                })
                .collect();
            (!own.is_empty()).then(|| Diagnostic {
                labels: own,
                ..Diagnostic::error(
                    C::SemanticDiff,
                    start(&input.path),
                    "the imported source does not check",
                )
            })
        })
        .collect()
}

/// Round-trip diagnostics: generated source that does not check, or does
/// not lower back to its input. Files going into a package under
/// `out/lit` that the import does not generate are checked there
/// ([`into_existing`]); the rest on their own.
fn round_trip(imported: &Imported, inputs: &[Input], out: &str) -> Vec<Diagnostic> {
    let generated = generated_packages(imported);
    let (into, own): (Vec<_>, Vec<_>) = imported
        .files
        .iter()
        .cloned()
        .partition(|f| package_dir(&f.path).is_some_and(|d| !generated.iter().any(|g| g == d)));
    let (into_inputs, own_inputs): (Vec<Input>, Vec<Input>) = inputs
        .iter()
        .cloned()
        .partition(|i| into.iter().any(|f| f.from == i.path));
    let mut out_diagnostics = Vec::new();
    if !into.is_empty() {
        out_diagnostics.extend(into_existing(&Imported { files: into }, &into_inputs, out));
    }
    if !own_inputs.is_empty() {
        out_diagnostics.extend(standalone(&Imported { files: own }, &own_inputs));
    }
    out_diagnostics
}

/// [`round_trip`] of files added to packages under `out/lit`: those
/// packages, with every other package there, overlaid with the files.
fn into_existing(imported: &Imported, inputs: &[Input], out: &str) -> Vec<Diagnostic> {
    let mut files: BTreeMap<String, String> = litigate_engine::cite::SourceCorpus::embedded_ref()
        .files
        .iter()
        .map(|(p, t)| (p.clone(), t.clone()))
        .collect();
    collect(&Path::new(out).join("lit"), "lit", &mut files);
    files.extend(
        imported
            .files
            .iter()
            .map(|f| (f.path.clone(), f.text.clone())),
    );
    let tree = MemoryTree::new(files);
    let mut dirs: Vec<String> = imported
        .files
        .iter()
        .filter_map(|f| package_dir(&f.path).map(|d| format!("lit/{d}")))
        .collect();
    dirs.dedup();
    let checked = match checked_in(
        &tree,
        &dirs,
        Profile::Dev,
        &Options::default(),
        &["lit".to_owned()],
    ) {
        Ok(checked) => checked,
        Err(failure) => {
            return inputs
                .iter()
                .map(|i| Diagnostic::error(C::SemanticDiff, start(&i.path), failure.0.clone()))
                .collect()
        }
    };
    compare_lowered(imported, inputs, &checked)
}

/// Every file under disk directory `dir`, keyed under `prefix`.
fn collect(dir: &Path, prefix: &str, out: &mut BTreeMap<String, String>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.filter_map(Result::ok) {
        let name = format!("{prefix}/{}", entry.file_name().to_string_lossy());
        let path = entry.path();
        if path.is_dir() {
            collect(&path, &name, out);
        } else if let Ok(text) = std::fs::read_to_string(&path) {
            out.insert(name, text);
        }
    }
}

/// [`round_trip`] of generated packages and matters on their own.
fn standalone(imported: &Imported, inputs: &[Input]) -> Vec<Diagnostic> {
    let (packages, cases) = loaded(imported);
    let mut session = Session::from_loaded(&packages, cases);
    // Nothing is locked yet (`litigate lock` writes `Lit.lock` after the
    // import), so the round trip pins every matter to the packages as
    // generated.
    let lock = session.matter_lock(&session.cases).render();
    for case in &mut session.cases {
        case.lock = Some(lock.clone());
    }
    let checked = check_session(session, Profile::Dev, &Options::default());
    compare_lowered(imported, inputs, &checked)
}

/// The check errors of `imported`'s files, else each input against what
/// its generated source lowers to.
fn compare_lowered(
    imported: &Imported,
    inputs: &[Input],
    checked: &super::check::Checked,
) -> Vec<Diagnostic> {
    let mut out = unchecked(imported, inputs, checked);
    if !out.is_empty() {
        return out;
    }
    let built = lower_all(checked, &Options::default());
    out.extend(built.diagnostics);
    let by_name: BTreeMap<&str, &str> = built
        .files
        .iter()
        .map(|(n, t)| (n.as_str(), t.as_str()))
        .collect();
    for input in inputs {
        let name = counterpart(&input.text);
        out.extend(compared(input, &name, by_name.get(name.as_str()).copied()));
    }
    out
}

/// The round-trip diagnostic for `input` against `lowered`, the text its
/// generated source lowered to as `name` (`None` when it lowered to
/// nothing): none when they are semantically equal. The importer writes
/// every value losslessly and the typed source rejects what it cannot
/// read, so this is the guard that keeps both honest.
fn compared(input: &Input, name: &str, lowered: Option<&str>) -> Option<Diagnostic> {
    let at = start(&input.path);
    let Some(text) = lowered else {
        return Some(Diagnostic::error(
            C::SemanticDiff,
            at,
            format!("the imported source does not lower to `{name}`"),
        ));
    };
    match litigate_lower::semantic_diff_json(text, &input.text) {
        Ok((_, changes)) if changes.is_empty() => None,
        Ok((_, changes)) => Some(Diagnostic {
            notes: changes
                .iter()
                .map(|c| Note {
                    message: format!("{}: {:?} -> {:?}", c.pointer, c.before, c.after),
                })
                .collect(),
            ..Diagnostic::error(
                C::SemanticDiff,
                at,
                format!(
                    "the imported source lowers to JSON with {} difference(s)",
                    changes.len()
                ),
            )
        }),
        Err(message) => Some(Diagnostic::error(C::SemanticDiff, at, message)),
    }
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
    let imported = match litigate_import::import_into(&inputs, &existing_packages(out)) {
        Ok(imported) => imported,
        Err(diagnostics) => {
            report.diagnostics = diagnostics;
            return Ok(report);
        }
    };
    report.diagnostics = round_trip(&imported, &inputs, out);
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

#[cfg(test)]
mod tests {
    use super::{compared, counterpart, is_case, owns, round_trip};
    use litigate_base::DiagnosticCode as C;
    use litigate_import::Input;

    #[test]
    fn a_round_trip_that_loses_or_changes_its_input_is_reported() {
        let input = Input::new(
            "s.json".to_owned(),
            r#"{"id": "s", "summary": "S", "packs": [], "scenario": {"params": {"rate": 1}}}"#
                .to_owned(),
        );
        let lost = compared(&input, "scenarios/s.json", None).map(|d| d.message);
        assert_eq!(
            lost.as_deref(),
            Some("the imported source does not lower to `scenarios/s.json`")
        );
        let changed =
            r#"{"id": "s", "summary": "S", "packs": [], "scenario": {"params": {"rate": 2}}}"#;
        let diff = compared(&input, "scenarios/s.json", Some(changed));
        assert_eq!(
            diff.as_ref().map(|d| (d.message.as_str(), d.notes.len())),
            Some(("the imported source lowers to JSON with 1 difference(s)", 1))
        );
        assert!(compared(&input, "scenarios/s.json", Some(&input.text)).is_none());
        assert!(compared(&input, "scenarios/s.json", Some("not json")).is_some());
    }

    #[test]
    fn inputs_pair_with_the_document_they_lower_to() {
        assert_eq!(counterpart(r#"{"id": "p", "nodes": []}"#), "packs/p.json");
        assert_eq!(counterpart(r#"{"links": []}"#), "packs/links.json");
        assert_eq!(counterpart(r#"{"instances": {}}"#), "packs/links.json");
        assert_eq!(
            counterpart(r#"{"id": "s", "scenario": {}}"#),
            "scenarios/s.json"
        );
    }

    #[test]
    fn a_generated_file_owns_its_package_directory_or_itself() {
        assert!(owns("lit/frcp/frcp.lit", "lit/frcp/Lit.toml"));
        assert!(!owns("lit/frcp/frcp.lit", "lit/frcp-two/Lit.toml"));
        assert!(!owns("lit/frcp/frcp.lit", "cases/x.case"));
        assert!(owns("cases/x.case", "cases/x.case"));
        assert!(!owns("cases/x.case", "cases/y.case"));
        assert!(is_case("cases/X.CASE"));
        assert!(!is_case("lit/x/x.lit"));
    }

    fn codes(found: &[litigate_base::Diagnostic]) -> Vec<(C, String)> {
        found.iter().map(|d| (d.code, d.message.clone())).collect()
    }

    /// Every outcome of the round trip in one binary: a pack whose id is
    /// `links` lowers to `packs/links.json`, where the links document
    /// lowers too. The importer names both packages `links` (one
    /// resolution package, L1011); renamed apart, the pack's output is
    /// compared with the links document and is no document of its kind.
    #[test]
    fn every_round_trip_outcome() {
        let built = |dir: &str, file: &str| {
            std::fs::read_to_string(std::path::Path::new(dir).join(file)).unwrap_or_default()
        };
        let read = |file: &str| built(litgraph_packs::PACKS_DIR, file);
        // These packages use only edge-window shapes the importer can spell;
        // FRCP/FRAP grants are refused by design until the source language
        // has a lossless spelling for their deadline-level grants.
        let text = read("cofc-court-of-federal-claims.json");
        let mut pack: serde_json::Value = serde_json::from_str(&text).unwrap_or_default();
        let plain = [
            Input::new("pack.json".to_owned(), pack.to_string()),
            Input::new("mpep.json".to_owned(), read("mpep-prosecution.json")),
            Input::new(
                "matter.json".to_owned(),
                built(litgraph_packs::SCENARIOS_DIR, "cofc-1498-patent-case.json"),
            ),
        ];
        let imported = litigate_import::import(&plain).unwrap_or_default();
        let round = round_trip(&imported, &plain, ".");
        assert_eq!(codes(&round), [], "{round:#?}");
        let mut relabelled = pack.clone();
        relabelled["title"] = "Another title".into();
        let extra = [
            Input::new("pack.json".to_owned(), relabelled.to_string()),
            Input::new(
                "extra.json".to_owned(),
                r#"{"id": "nope", "nodes": []}"#.to_owned(),
            ),
        ];
        assert_eq!(
            codes(&round_trip(&imported, &extra, ".")),
            [
                (
                    C::SemanticDiff,
                    "the imported source lowers to JSON with 1 difference(s)".to_owned()
                ),
                (
                    C::SemanticDiff,
                    "the imported source does not lower to `packs/nope.json`".to_owned()
                ),
            ]
        );
        pack["id"] = "links".into();
        let inputs = [
            Input::new("pack.json".to_owned(), pack.to_string()),
            Input::new(
                "links.json".to_owned(),
                r#"{"instances": {"links@copy": {"pack": "links"}}}"#.to_owned(),
            ),
        ];
        let mut imported = litigate_import::import(&inputs).unwrap_or_default();
        assert_eq!(
            codes(&round_trip(&imported, &inputs, ".")),
            [(
                C::SemanticDiff,
                "the imported source does not check".to_owned()
            )]
        );
        for f in &mut imported.files {
            f.text = if f.path == "lit/links/Lit.toml" {
                f.text.replace("\nlinks = {", "\nlinks-pack = {")
            } else {
                f.text.replace("name = \"links\"", "name = \"links-pack\"")
            };
        }
        assert_eq!(
            codes(&round_trip(&imported, &inputs, ".")),
            [(C::SemanticDiff, "second: not a links document".to_owned())]
        );
    }
}
