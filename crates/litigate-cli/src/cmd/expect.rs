//! `litigate test tests/expect`: the expect tests (lg-wvlf.1; SPEC §5.8, expect tests).
//! Each is one directory, `tests/expect/<case>/`, holding a matter's
//! `.case` and its `Lit.lock`, one `.world` per circuit or forum it runs
//! under, and `expect.toml`: the package directories the matter needs
//! (`paths`, from the working directory) and one `[[run]]` per world file
//! (and, optionally, `as_of` and `known_at`), whose `expect` table records,
//! for each world the file spans, the values the matter gives there: its
//! verdict, the layers in force, each filing's outcome, and each deadline
//! counted with its due date, status, the cite of the rule that sets it
//! and the cite of the rule that counted it. A value that moved fails the test with one row of case, world,
//! field, expected and actual. `--bless` records the values found instead;
//! the commit says why they moved.
use super::worlds;
use crate::out::Failure;
use litigate_base::{Diagnostic, DiagnosticCode, Severity, Span};
use litigate_sema::pkg::source::Source;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

/// The file that makes a directory an expect test.
pub const FILE: &str = "expect.toml";

/// An `expect.toml`.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Expect {
    /// Package directories the matter and its worlds' layers need, besides
    /// the library.
    #[serde(default)]
    pub(crate) paths: Vec<String>,
    /// One per world file and dates.
    #[serde(rename = "run", default)]
    runs: Vec<Run>,
}

/// One world file the matter runs under.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Run {
    /// The `.world` file, in the test's directory.
    world: String,
    /// Replaces the matter's `as_of`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    as_of: Option<String>,
    /// Replaces the matter's `known_at`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    known_at: Option<String>,
    /// The expected values, by world (its id, then its choices).
    #[serde(default)]
    expect: BTreeMap<String, Values>,
}

/// What a matter gives in one world.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Values {
    /// `clean`, the codes of its errors (`L4002` for a late filing, say),
    /// or `undetermined` when the world's own law is in error.
    verdict: String,
    /// The world's layers in force at the run's dates.
    #[serde(default)]
    in_force: Vec<String>,
    /// Each filing's outcome: `timely`, `late`, `unjudged`, `untimed`,
    /// `rejected` or `undetermined`.
    #[serde(default)]
    outcome: BTreeMap<String, String>,
    /// Each deadline the matter counted.
    #[serde(default)]
    deadlines: Vec<Deadline>,
}

/// One computed deadline.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
#[allow(clippy::struct_field_names)] // `deadline` is the key `expect.toml` reads
struct Deadline {
    /// The qualified deadline id.
    deadline: String,
    /// Its trigger's date.
    trigger: String,
    /// Its due date (`unknown`: none could be counted).
    due: String,
    /// `open`, `met`, `late`, ...
    status: String,
    /// The cite of the deadline's version that governs it (`-`: none).
    cite: String,
    /// The cite of the counting rule's version that counted it (`-`: none).
    counted_by: String,
}

/// One value that differs from its expectation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Mismatch {
    /// The test's directory name.
    pub case: String,
    /// The world file (`@as_of` when the run sets one), then the world.
    pub world: String,
    /// `verdict`, `in_force`, `outcome KEY`, `deadline ID @DATE due`, ...
    pub field: String,
    /// The recorded value (`-`: none).
    pub expected: String,
    /// The value found (`-`: none).
    pub actual: String,
}

/// An expect test's result, its diagnostics, and its `expect.toml` as
/// `(path, text)`.
pub type Ran = (CaseResult, Vec<Diagnostic>, (String, String));

/// One expect test's result.
#[derive(Debug, Clone, Serialize)]
pub struct CaseResult {
    /// The test's directory name.
    pub case: String,
    /// Its directory.
    pub dir: String,
    /// No value moved.
    pub passed: bool,
    /// Whether `--bless` rewrote its `expect.toml`.
    pub blessed: bool,
    /// The values that moved.
    pub mismatches: Vec<Mismatch>,
}

/// The expect test directories at or under `path`, in path order: those
/// holding an [`FILE`].
#[must_use]
pub fn dirs(src: &dyn Source, path: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    if !src.is_dir(path) {
        return out;
    }
    if src.is_file(&path.join(FILE)) {
        out.push(path.to_path_buf());
        return out;
    }
    let mut entries = src.entries(path).unwrap_or_default();
    entries.sort();
    for entry in entries {
        out.extend(dirs(src, &entry));
    }
    out
}

/// The name of the test in `dir`: the directory's name.
#[must_use]
pub fn name_of(dir: &Path) -> String {
    dir.file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default()
}

/// `dir`'s `expect.toml`, read and parsed.
///
/// # Errors
/// It is missing, unreadable or not an expect test.
pub(crate) fn read(src: &dyn Source, dir: &Path) -> Result<(String, Expect), Failure> {
    let path = dir.join(FILE);
    let shown = crate::ws::shown_in(src, &path);
    let text = src
        .read_to_string(&path)
        .map_err(|e| Failure::new(format!("{shown}: {e}")))?;
    let expect = toml::from_str(&text).map_err(|e| Failure::new(format!("{shown}: {e}")))?;
    Ok((text, expect))
}

/// The one `.case` file in `dir`.
fn case_in(src: &dyn Source, dir: &Path) -> Result<PathBuf, Failure> {
    let mut cases: Vec<PathBuf> = src
        .entries(dir)
        .map_err(|e| Failure::new(format!("{}: {e}", dir.display())))?
        .into_iter()
        .filter(|p| p.extension().is_some_and(|e| e == "case") && src.is_file(p))
        .collect();
    cases.sort();
    match cases.as_slice() {
        [case] => Ok(case.clone()),
        _ => Err(Failure::new(format!(
            "{}: an expect test holds exactly one .case file, found {}",
            crate::ws::shown_in(src, dir),
            cases.len()
        ))),
    }
}

/// The world column of `run`'s rows.
fn world_of(run: &Run, world: &str) -> String {
    match &run.as_of {
        Some(date) => format!("{}@{date} {world}", run.world),
        None => format!("{} {world}", run.world),
    }
}

/// The values `run` of the matter `case` gives, by world.
///
/// # Errors
/// The world query cannot run, or the matter, a package or the world file
/// is in error.
fn observe(
    src: &dyn Source,
    dir: &Path,
    case: &Path,
    paths: &[String],
    run: &Run,
    lib: &[String],
) -> Result<BTreeMap<String, Values>, Failure> {
    let world = dir.join(&run.world);
    let report = worlds::worlds(
        src,
        &worlds::Args {
            case: case.to_string_lossy().into_owned(),
            world: world.to_string_lossy().into_owned(),
            paths: paths.to_vec(),
            as_of: run.as_of.clone(),
            known_at: run.known_at.clone(),
            lib: lib.to_vec(),
            cites: true,
            ..worlds::Args::default()
        },
    )?;
    let errors: BTreeSet<String> = report
        .diagnostics
        .iter()
        .filter(|d| d.severity == Severity::Error)
        .map(|d| format!("{} {}", d.code.as_str(), d.message))
        .collect();
    if !errors.is_empty() {
        return Err(Failure::new(
            errors.into_iter().collect::<Vec<_>>().join("; "),
        ));
    }
    let text = |v: &Value| match v {
        Value::String(s) => s.clone(),
        Value::Null => "-".to_owned(),
        other => other.to_string(),
    };
    let mut out = BTreeMap::new();
    for w in report.result["worlds"].as_array().into_iter().flatten() {
        let mut label = text(&w["id"]);
        for (choice, option) in w["choices"].as_object().into_iter().flatten() {
            let _ = write!(label, " {choice}={}", text(option));
        }
        let codes: BTreeSet<String> = w["diagnostics"]
            .as_array()
            .into_iter()
            .flatten()
            .filter(|d| d["severity"] == "error")
            .map(|d| text(&d["code"]))
            .collect();
        let verdict = if w["lawful"] != Value::Bool(true) {
            "undetermined".to_owned()
        } else if codes.is_empty() {
            "clean".to_owned()
        } else {
            codes.into_iter().collect::<Vec<_>>().join(" ")
        };
        let values = Values {
            verdict,
            in_force: w["layers"]
                .as_array()
                .into_iter()
                .flatten()
                .filter(|l| l["in_force"] == Value::Bool(true))
                .map(|l| text(&l["name"]))
                .collect(),
            outcome: w["filings"]
                .as_object()
                .into_iter()
                .flatten()
                .map(|(k, v)| (k.clone(), text(v)))
                .collect(),
            deadlines: w["deadlines"]
                .as_array()
                .into_iter()
                .flatten()
                .map(|d| Deadline {
                    deadline: text(&d["deadline"]),
                    trigger: text(&d["trigger_date"]),
                    due: d["due"].as_str().unwrap_or("unknown").to_owned(),
                    status: text(&d["status"]),
                    cite: text(&d["cite"]),
                    counted_by: text(&d["counted_by"]),
                })
                .collect(),
        };
        out.insert(label, values);
    }
    Ok(out)
}

/// The values of one world as `(field, value)` rows.
fn rows(values: &Values) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    out.insert("verdict".to_owned(), values.verdict.clone());
    let in_force = if values.in_force.is_empty() {
        "-".to_owned()
    } else {
        values.in_force.join(", ")
    };
    out.insert("in_force".to_owned(), in_force);
    for (key, outcome) in &values.outcome {
        out.insert(format!("outcome {key}"), outcome.clone());
    }
    let mut seen: BTreeMap<String, usize> = BTreeMap::new();
    for d in &values.deadlines {
        let mut field = format!("deadline {} @{}", d.deadline, d.trigger);
        let n = seen.entry(field.clone()).or_default();
        *n += 1;
        if *n > 1 {
            let _ = write!(field, " #{n}");
        }
        out.insert(format!("{field} due"), d.due.clone());
        out.insert(format!("{field} status"), d.status.clone());
        out.insert(format!("{field} cite"), d.cite.clone());
        out.insert(format!("{field} counted_by"), d.counted_by.clone());
    }
    out
}

/// The rows of the values of every world of a run, by `(world, field)`.
fn world_rows(run: &Run, values: &BTreeMap<String, Values>) -> BTreeMap<(String, String), String> {
    values
        .iter()
        .flat_map(|(world, v)| {
            let world = world_of(run, world);
            rows(v)
                .into_iter()
                .map(move |(field, value)| ((world.clone(), field), value))
        })
        .collect()
}

/// The `(world, field)` rows the expect test in `dir` gives now, for the
/// change impact report (`litigate test --impact`): a run that cannot be
/// made is one `error` row.
#[must_use]
pub fn observed(
    src: &dyn Source,
    dir: &Path,
    lib: &[String],
) -> BTreeMap<(String, String), String> {
    let mut out = BTreeMap::new();
    let error = |message: String| (("-".to_owned(), "error".to_owned()), message);
    let (expect, case) = match read(src, dir).and_then(|(_, e)| Ok((e, case_in(src, dir)?))) {
        Ok(found) => found,
        Err(Failure(message)) => {
            out.extend([error(message)]);
            return out;
        }
    };
    for run in &expect.runs {
        match observe(src, dir, &case, &expect.paths, run, lib) {
            Ok(values) => out.extend(world_rows(run, &values)),
            Err(Failure(message)) => {
                let world = world_of(run, "-");
                out.insert((world, "error".to_owned()), message);
            }
        }
    }
    out
}

/// Runs the expect test in `dir` (on disk when `bless`, which records the
/// values found in its `expect.toml` in place of those it had): its
/// result, one L5006 per value that moved, and its `expect.toml` as a
/// source the diagnostics cite.
///
/// # Errors
/// `dir` is not an expect test, or, when blessing, a run cannot be made
/// or the file cannot be written.
pub fn run(src: &dyn Source, dir: &Path, lib: &[String], bless: bool) -> Result<Ran, Failure> {
    let name = name_of(dir);
    let file = crate::ws::shown_in(src, &dir.join(FILE));
    let (text, mut expect) = read(src, dir)?;
    let case = case_in(src, dir)?;
    let mut mismatches = Vec::new();
    let none = String::from("-");
    for run in &mut expect.runs {
        let found = match observe(src, dir, &case, &expect.paths, run, lib) {
            Ok(found) => found,
            Err(Failure(message)) if !bless => {
                mismatches.push(Mismatch {
                    case: name.clone(),
                    world: world_of(run, "-"),
                    field: "error".to_owned(),
                    expected: none.clone(),
                    actual: message,
                });
                continue;
            }
            Err(Failure(message)) => {
                return Err(Failure::new(format!("{file}: {}: {message}", run.world)))
            }
        };
        let expected = world_rows(run, &run.expect);
        let actual = world_rows(run, &found);
        let keys: BTreeSet<&(String, String)> = expected.keys().chain(actual.keys()).collect();
        for key in keys {
            let (e, a) = (expected.get(key), actual.get(key));
            if e != a {
                mismatches.push(Mismatch {
                    case: name.clone(),
                    world: key.0.clone(),
                    field: key.1.clone(),
                    expected: e.unwrap_or(&none).clone(),
                    actual: a.unwrap_or(&none).clone(),
                });
            }
        }
        if bless {
            run.expect = found;
        }
    }
    let blessed = bless && !mismatches.is_empty();
    if blessed {
        let body = toml::to_string_pretty(&expect).map_err(|e| Failure::new(e.to_string()))?;
        let header: Vec<&str> = text
            .lines()
            .take_while(|l| l.starts_with('#') || l.trim().is_empty())
            .collect();
        let header = header.join("\n").trim_end().to_owned();
        let out = if header.is_empty() {
            body
        } else {
            format!("{header}\n\n{body}")
        };
        crate::cmd::build::write_atomic_in(src, &dir.join(FILE), &out)?;
    }
    let diagnostics = if bless {
        Vec::new()
    } else {
        let at = Span {
            file: file.as_str().into(),
            line: 1,
            col: 1,
            end_line: 1,
            end_col: 1,
        };
        mismatches
            .iter()
            .map(|m| {
                Diagnostic::error(
                    DiagnosticCode::TestFailed,
                    at.clone(),
                    format!(
                        "expect test `{}`: {} {}: expected {}, found {}",
                        m.case, m.world, m.field, m.expected, m.actual
                    ),
                )
            })
            .collect()
    };
    let result = CaseResult {
        case: name,
        dir: crate::ws::shown_in(src, dir),
        passed: bless || mismatches.is_empty(),
        blessed,
        mismatches,
    };
    Ok((result, diagnostics, (file, text)))
}
