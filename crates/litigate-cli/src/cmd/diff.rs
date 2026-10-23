//! `litigate diff`: the SPEC §8.6 semantic diff of two sides, where a side
//! is a litgraph JSON file, a package directory or `.case` (built at a
//! date), or a directory of JSON (files paired by relative path).
//! `--against JSON` and `--check` turn every change into an L6002 error.
use super::build::lower_all;
use super::check::checked;
use crate::out::{Failure, Outcome, Report};
use litigate_base::{Diagnostic, DiagnosticCode as C, LineIndex, Note, Profile, Span};
use litigate_lower::{Change, Options};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::path::Path;

/// `diff` arguments.
pub struct Args {
    /// First side.
    pub a: String,
    /// Second side.
    pub b: Option<String>,
    /// Expected JSON for the first side.
    pub against: Option<String>,
    /// Date for both sides (or the first).
    pub as_of: Option<String>,
    /// Date for the second side.
    pub to_as_of: Option<String>,
    /// Differences are errors.
    pub check: bool,
    /// Library directories.
    pub lib: Vec<String>,
}

/// Documents of one side, by relative name, plus diagnostics building it.
type Side = (
    BTreeMap<String, (String, String)>,
    Vec<Diagnostic>,
    Vec<(String, String)>,
);

fn json_files(
    dir: &Path,
    base: &Path,
    out: &mut BTreeMap<String, (String, String)>,
) -> std::io::Result<()> {
    let mut entries: Vec<_> = std::fs::read_dir(dir)?
        .filter_map(Result::ok)
        .map(|e| e.path())
        .collect();
    entries.sort();
    for path in entries {
        let name = path
            .strip_prefix(base)
            .unwrap_or(&path)
            .to_string_lossy()
            .replace('\\', "/");
        if path.is_dir() {
            json_files(&path, base, out)?;
        } else if has_ext(&name, "json")
            && !name.ends_with(".map.json")
            && !name.starts_with("litigatables/")
        {
            let text = std::fs::read_to_string(&path)?;
            out.insert(
                identity(&text).unwrap_or(name),
                (crate::ws::shown(&path), text),
            );
        }
    }
    Ok(())
}

/// How documents of two directories pair: `pack:<id>`, `links`, or
/// `scenario:<id>` (file names need not match a pack's id). Clock fragments pair by path.
fn identity(text: &str) -> Option<String> {
    let value: Value = serde_json::from_str(text).ok()?;
    let id = || value.get("id").and_then(Value::as_str).map(str::to_owned);
    match litigate_lower::diff::detect(&value)? {
        litigate_lower::DocumentKind::Pack => Some(format!("pack:{}", id()?)),
        litigate_lower::DocumentKind::Links => Some("links".to_owned()),
        litigate_lower::DocumentKind::Scenario => Some(format!("scenario:{}", id()?)),
        litigate_lower::DocumentKind::Clock => None,
    }
}

/// Reads or builds one side.
fn side(path: &str, as_of: Option<&String>, lib: &[String]) -> Result<Side, Failure> {
    let p = Path::new(path);
    let mut docs = BTreeMap::new();
    if p.is_file() && p.extension().is_some_and(|e| e == "json") {
        let text = std::fs::read_to_string(p).map_err(|e| Failure::new(format!("{path}: {e}")))?;
        docs.insert(String::new(), (crate::ws::shown(p), text));
        return Ok((docs, Vec::new(), Vec::new()));
    }
    if p.is_dir()
        && !p.join("Lit.toml").is_file()
        && !crate::ws::discover(&[path.to_owned()])
            .is_ok_and(|t| !t.packages.is_empty() || !t.cases.is_empty())
    {
        json_files(p, p, &mut docs)?;
        return Ok((docs, Vec::new(), Vec::new()));
    }
    let options = Options {
        as_of: as_of.cloned(),
        known_at: None,
    };
    let c = checked(&[path.to_owned()], Profile::Dev, &options, lib)?;
    let built = lower_all(&c, &options);
    let mut diagnostics = c.diagnostics.clone();
    diagnostics.extend(built.diagnostics);
    for (name, text) in built.files {
        if !name.ends_with(".map.json") && !name.starts_with("litigatables/") {
            let key = identity(&text).unwrap_or_else(|| name.clone());
            docs.insert(key, (format!("{path}#{name}"), text));
        }
    }
    Ok((docs, diagnostics, c.session.sources()))
}

/// When exactly one side is a single file (keyed `""`), re-keys the other
/// side's one primary document (not a clock fragment) to `""` and drops
/// the rest, which have no counterpart by construction.
fn align(a: &mut BTreeMap<String, (String, String)>, b: &mut BTreeMap<String, (String, String)>) {
    let single = |m: &BTreeMap<String, (String, String)>| m.len() == 1 && m.contains_key("");
    let rekey = |m: &mut BTreeMap<String, (String, String)>| {
        let primary: Vec<String> = m
            .keys()
            .filter(|k| !k.starts_with("packs/clock/"))
            .cloned()
            .collect();
        if let [one] = primary.as_slice() {
            let doc = m.remove(one);
            m.clear();
            m.extend(doc.map(|d| (String::new(), d)));
        }
    };
    if single(a) && !single(b) {
        rekey(b);
    } else if single(b) && !single(a) {
        rekey(a);
    }
}

/// Line/column of the first occurrence of `"id"` in `text`, for a change's span.
fn locate(file: &str, text: &str, id: Option<&str>) -> Span {
    let at = id
        .and_then(|id| text.find(&format!("\"{id}\"")))
        .unwrap_or(0);
    let (line, col) = LineIndex::new(text).line_col(at);
    Span {
        file: file.into(),
        line,
        col,
        end_line: line,
        end_col: col,
    }
}

fn change_diagnostic(file: &str, text: &str, change: &Change) -> Diagnostic {
    let show = |v: &Option<Value>| {
        v.as_ref()
            .map_or_else(|| "(absent)".to_owned(), ToString::to_string)
    };
    let mut d = Diagnostic::error(
        C::SemanticDiff,
        locate(file, text, change.id.as_deref()),
        format!(
            "{} at {}: {} -> {}",
            serde_json::to_value(change.kind)
                .ok()
                .and_then(|v| v.as_str().map(str::to_owned))
                .unwrap_or_default(),
            change.pointer,
            show(&change.before),
            show(&change.after)
        ),
    );
    if let Some(id) = &change.id {
        d.notes.push(Note {
            span: None,
            message: format!("belongs to `{id}`"),
        });
    }
    d
}

/// `litigate diff`.
pub fn diff(args: &Args) -> Outcome {
    let (b_path, b_date, strict) = match (&args.against, &args.b) {
        (Some(expected), _) => (expected.clone(), None, true),
        (None, Some(b)) => (
            b.clone(),
            args.to_as_of.as_ref().or(args.as_of.as_ref()),
            args.check,
        ),
        (None, None) if args.to_as_of.is_some() => {
            (args.a.clone(), args.to_as_of.as_ref(), args.check)
        }
        (None, None) => {
            return Err(Failure::new(
                "give a second side, `--against JSON`, or `--to-as-of DATE`",
            ))
        }
    };
    let (mut a_docs, mut diagnostics, mut sources) = side(&args.a, args.as_of.as_ref(), &args.lib)?;
    let (mut b_docs, b_diags, b_sources) = side(&b_path, b_date, &args.lib)?;
    align(&mut a_docs, &mut b_docs);
    diagnostics.extend(b_diags);
    sources.extend(b_sources);
    let mut report = Report {
        provenance: crate::prov::of(Vec::new(), args.as_of.as_deref(), None),
        ..Report::default()
    };
    if diagnostics
        .iter()
        .any(|d| d.severity == litigate_base::Severity::Error)
    {
        report.diagnostics = diagnostics;
        report.sources = sources;
        return Ok(report);
    }
    let compared = compare(&a_docs, &b_docs, strict);
    diagnostics.extend(compared.diagnostics);
    sources.extend(compared.sources);
    if compared.files.is_empty() && compared.unmatched.is_empty() {
        return Err(Failure::new(
            "nothing to compare: the two sides share no documents",
        ));
    }
    report.result = json!({
        "equal": compared.changes.is_empty() && compared.unmatched.is_empty(),
        "files": compared.files,
        "changes": compared.changes,
        "unmatched": compared.unmatched,
    });
    report.diagnostics = diagnostics;
    report.sources = sources;
    Ok(report)
}

/// `(shown path, text)` of one document.
type Doc = (String, String);

/// The outcome of pairing two sides.
#[derive(Default)]
struct Compared {
    files: Vec<Value>,
    changes: Vec<Value>,
    unmatched: Vec<String>,
    diagnostics: Vec<Diagnostic>,
    sources: Vec<Doc>,
}

/// Diffs every document present on both sides; `strict` makes each change L6002.
fn compare(
    a_docs: &BTreeMap<String, Doc>,
    b_docs: &BTreeMap<String, Doc>,
    strict: bool,
) -> Compared {
    let mut out = Compared {
        unmatched: a_docs
            .keys()
            .chain(b_docs.keys())
            .filter(|k| !(a_docs.contains_key(*k) && b_docs.contains_key(*k)))
            .cloned()
            .collect(),
        ..Compared::default()
    };
    for (name, (a_file, a_text)) in a_docs {
        let Some((b_file, b_text)) = b_docs.get(name) else {
            continue;
        };
        match litigate_lower::semantic_diff_json(a_text, b_text) {
            Ok((kind, found)) => {
                out.files.push(json!({"a": a_file, "b": b_file, "kind": kind, "equal": found.is_empty(), "changes": found.len()}));
                if strict {
                    out.diagnostics
                        .extend(found.iter().map(|c| change_diagnostic(b_file, b_text, c)));
                    out.sources.push((b_file.clone(), b_text.clone()));
                }
                out.changes.extend(found.into_iter().map(|c| {
                    let mut v = serde_json::to_value(c).unwrap_or_default();
                    if !name.is_empty() {
                        v["document"] = Value::String(name.clone());
                    }
                    v
                }));
            }
            Err(message) => out.diagnostics.push(Diagnostic::error(
                C::SemanticDiff,
                locate(b_file, "", None),
                format!("cannot compare `{a_file}` with `{b_file}`: {message}"),
            )),
        }
    }
    out
}

/// True when `name` ends in `.ext` (ASCII case-insensitive).
fn has_ext(name: &str, ext: &str) -> bool {
    Path::new(name)
        .extension()
        .is_some_and(|e| e.eq_ignore_ascii_case(ext))
}

#[cfg(test)]
mod tests {
    use super::{align, has_ext, identity, locate};
    use proptest::prelude::*;
    use std::collections::BTreeMap;

    fn docs(keys: &[&str]) -> BTreeMap<String, (String, String)> {
        keys.iter()
            .map(|k| ((*k).to_owned(), ((*k).to_owned(), String::new())))
            .collect()
    }

    #[test]
    fn documents_are_identified_by_kind_and_id() {
        assert_eq!(
            identity(r#"{"id": "p", "nodes": []}"#).as_deref(),
            Some("pack:p")
        );
        assert_eq!(identity(r#"{"links": []}"#).as_deref(), Some("links"));
        assert_eq!(
            identity(r#"{"id": "s", "summary": ""}"#).as_deref(),
            Some("scenario:s")
        );
        assert_eq!(identity(r#"{"countingRules": []}"#), None);
        assert_eq!(identity(r#"{"nodes": []}"#), None);
        assert_eq!(identity(r#"{"title": "x"}"#), None);
        assert_eq!(identity("not json"), None);
    }

    #[test]
    fn a_single_file_pairs_with_the_one_primary_document() {
        let (mut a, mut b) = (docs(&[""]), docs(&["pack:p", "packs/clock/p.json"]));
        align(&mut a, &mut b);
        assert_eq!(b.keys().collect::<Vec<_>>(), [""]);
        assert_eq!(b[""].0, "pack:p");
        let (mut a, mut b) = (docs(&["pack:p", "pack:q"]), docs(&[""]));
        align(&mut a, &mut b);
        assert_eq!(a.len(), 2, "two primaries: nothing to pair");
        let (mut a, mut b) = (docs(&[""]), docs(&[""]));
        align(&mut a, &mut b);
        assert_eq!((a.len(), b.len()), (1, 1));
    }

    #[test]
    fn extensions_match_case_insensitively() {
        assert!(has_ext("a/B.JSON", "json"));
        assert!(!has_ext("a/b.json5", "json"));
        assert!(!has_ext("json", "json"));
    }

    proptest! {
        /// A change's span is the first quoted occurrence of its id, else
        /// the start of the file.
        #[test]
        fn changes_point_at_their_id(
            before in "[a-z \n]{0,40}",
            id in "[a-z]{1,8}",
            after in "[a-z \n]{0,40}",
        ) {
            let text = format!("{before}\"{id}\"{after}");
            let span = locate("f.json", &text, Some(&id));
            let line = u32::try_from(before.matches('\n').count() + 1).unwrap();
            let col = u32::try_from(before.rsplit('\n').next().map_or(0, str::len) + 1).unwrap();
            prop_assert_eq!((span.line, span.col), (line, col));
            let none = locate("f.json", &text, None);
            prop_assert_eq!((none.line, none.col), (1, 1));
        }
    }
}
