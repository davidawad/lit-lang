//! `litigate fix PATHS` (DESIGN §8 "an agent edits by fixes"): apply the
//! fixes `check` attaches to diagnostics. Only `machine_applicable` fixes
//! by default; `--include-maybe` adds `maybe_incorrect`. Overlapping edits
//! keep the one nearest the end of the file; run `fix` again for the rest.
//! A fix of several edits (possibly across files) applies whole or not at
//! all.
use super::build::write_atomic;
use super::check::checked;
use crate::out::{Failure, Outcome, Report};
use crate::span::byte_range;
use litigate_base::{Applicability, DiagnosticCode, Fix, Profile};
use litigate_lower::Options;
use serde_json::json;
use std::collections::BTreeMap;
use std::ops::Range;

/// `fix` arguments.
pub struct Args {
    /// Inputs.
    pub paths: Vec<String>,
    /// Only this code's fixes.
    pub code: Option<String>,
    /// Do not write.
    pub dry_run: bool,
    /// Also `maybe_incorrect` fixes.
    pub include_maybe: bool,
    /// Profile used to find fixes.
    pub profile: Profile,
    /// Library directories.
    pub lib: Vec<String>,
}

/// What [`apply`] did: the new text of every file a fix touched, the
/// applied fixes, and how many were skipped for overlapping.
#[derive(Debug, Default)]
pub struct Applied {
    /// `path → text` after the fixes, for every file an applied fix edits.
    pub texts: BTreeMap<String, String>,
    /// The applied fixes, earliest in the source first.
    pub fixes: Vec<(DiagnosticCode, Fix)>,
    /// Fixes skipped because an edit overlaps an applied one.
    pub skipped: usize,
}

/// Applies `fixes` to `sources` (`path → text`). A fix is all of its edits
/// or none: one with an edit into a file `sources` lacks is dropped, and
/// one with an edit overlapping an applied fix's (or its own) is skipped.
/// Fixes reaching furthest into the files go first.
#[must_use]
pub fn apply(sources: &BTreeMap<String, String>, fixes: Vec<(DiagnosticCode, Fix)>) -> Applied {
    type Located = Vec<(String, Range<usize>, String)>;
    let mut located: Vec<(Located, DiagnosticCode, Fix)> = fixes
        .into_iter()
        .filter_map(|(code, fix)| {
            let edits = fix
                .edits
                .iter()
                .map(|e| {
                    let text = sources.get(&e.span.file.0)?;
                    let range = byte_range(text, &e.span);
                    Some((e.span.file.0.clone(), range, e.replacement.clone()))
                })
                .collect::<Option<Located>>()?;
            Some((edits, code, fix))
        })
        .collect();
    let reach = |edits: &Located| {
        edits
            .iter()
            .map(|(file, range, _)| (file.clone(), range.start, range.end))
            .max()
    };
    located.sort_by_key(|(edits, ..)| std::cmp::Reverse(reach(edits)));
    let overlaps = |a: &(String, Range<usize>, String), b: &(String, Range<usize>, String)| {
        a.0 == b.0 && a.1.start < b.1.end && b.1.start < a.1.end
    };
    let mut taken: Located = Vec::new();
    let mut out = Applied::default();
    for (edits, code, fix) in located {
        let clashes = edits
            .iter()
            .enumerate()
            .any(|(i, e)| taken.iter().chain(&edits[..i]).any(|t| overlaps(e, t)));
        if clashes || out.fixes.iter().any(|(_, f)| *f == fix) {
            out.skipped += 1;
            continue;
        }
        taken.extend(edits);
        out.fixes.push((code, fix));
    }
    out.fixes.reverse();
    taken.sort_by(|a, b| (&b.0, b.1.start, b.1.end).cmp(&(&a.0, a.1.start, a.1.end)));
    for (file, range, replacement) in taken {
        out.texts
            .entry(file.clone())
            .or_insert_with(|| sources[&file].clone())
            .replace_range(range, &replacement);
    }
    out
}

/// `litigate fix`.
pub fn fix(args: &Args) -> Outcome {
    let wanted = match &args.code {
        Some(code) => Some(
            DiagnosticCode::parse(&code.to_ascii_uppercase())
                .ok_or_else(|| Failure::new(format!("`{code}` is not a diagnostic code")))?,
        ),
        None => None,
    };
    let c = checked(&args.paths, args.profile, &Options::default(), &args.lib)?;
    let allowed = |a: Applicability| {
        a == Applicability::MachineApplicable
            || (args.include_maybe && a == Applicability::MaybeIncorrect)
    };
    let candidates: Vec<(DiagnosticCode, Fix)> = c
        .diagnostics
        .iter()
        .filter(|d| wanted.is_none_or(|w| d.code == w))
        .flat_map(|d| {
            d.fixes
                .iter()
                .filter(|f| allowed(f.applicability))
                .map(|f| (d.code, f.clone()))
        })
        .collect();
    // Fixes into files the session has no source for are dropped.
    let sources: BTreeMap<String, String> = c.session.sources().into_iter().collect();
    let applied = apply(&sources, candidates);
    let changed: Vec<(&String, &String)> = applied
        .texts
        .iter()
        .filter(|(file, out)| sources[*file] != **out)
        .collect();
    if !args.dry_run {
        for (file, out) in &changed {
            write_atomic(&crate::ws::Session::real_path(file), out)?;
        }
    }
    let changed: Vec<&String> = changed.into_iter().map(|(file, _)| file).collect();
    let fixes: Vec<serde_json::Value> = applied
        .fixes
        .iter()
        .map(|(code, f)| {
            let mut fix = json!(f);
            fix["code"] = json!(code);
            fix
        })
        .collect();
    Ok(Report {
        result: json!({"fixes": fixes, "changed_files": changed, "skipped_overlapping": applied.skipped, "dry_run": args.dry_run}),
        provenance: c.provenance(None, None),
        ..Report::default()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use litigate_base::{Edit, Span};
    use proptest::prelude::*;

    fn at(file: &str, line: u32, col: u32, end_col: u32) -> Span {
        Span {
            file: file.into(),
            line,
            col,
            end_line: line,
            end_col,
        }
    }

    fn fix(line: u32, col: u32, end_col: u32, replacement: &str) -> (DiagnosticCode, Fix) {
        (
            DiagnosticCode::UnexpectedToken,
            Fix::single(
                at("f", line, col, end_col),
                replacement,
                Applicability::MachineApplicable,
            ),
        )
    }

    fn one(text: &str, fixes: Vec<(DiagnosticCode, Fix)>) -> (String, usize, usize) {
        let sources = BTreeMap::from([("f".to_owned(), text.to_owned())]);
        let applied = apply(&sources, fixes);
        let out = applied.texts.get("f").cloned().unwrap_or(text.to_owned());
        (out, applied.fixes.len(), applied.skipped)
    }

    #[test]
    fn applies_from_the_end_and_skips_overlaps() {
        let (text, applied, skipped) = one(
            "ab cd\nef",
            vec![fix(1, 1, 3, "X"), fix(2, 1, 2, "Y"), fix(1, 2, 4, "Z")],
        );
        assert_eq!(text, "aZcd\nYf");
        assert_eq!((applied, skipped), (2, 1));
    }

    #[test]
    fn a_repeated_fix_applies_once() {
        let (text, applied, skipped) = one("ab", vec![fix(1, 3, 3, "c"), fix(1, 3, 3, "c")]);
        assert_eq!((text.as_str(), applied, skipped), ("abc", 1, 1));
    }

    #[test]
    fn a_multi_edit_fix_applies_whole_across_files_or_not_at_all() {
        let sources = BTreeMap::from([
            ("a".to_owned(), "let old = 1".to_owned()),
            ("b".to_owned(), "use old".to_owned()),
        ]);
        let rename = |edits: Vec<(&str, u32, u32)>| {
            (
                DiagnosticCode::UnresolvedName,
                Fix {
                    message: Some("rename `old` to `new`".into()),
                    edits: edits
                        .into_iter()
                        .map(|(file, col, end)| Edit {
                            span: at(file, 1, col, end),
                            replacement: "new".into(),
                        })
                        .collect(),
                    applicability: Applicability::MachineApplicable,
                },
            )
        };
        let applied = apply(
            &sources,
            vec![rename(vec![("a", 5, 8), ("b", 5, 8)]), fix(1, 1, 2, "x")],
        );
        assert_eq!(applied.texts["a"], "let new = 1");
        assert_eq!(applied.texts["b"], "use new");
        assert_eq!((applied.fixes.len(), applied.skipped), (1, 0));
        // A clash on one edit drops the whole fix, edits in other files too.
        let clash = (
            DiagnosticCode::UnexpectedToken,
            Fix::single(at("b", 1, 7, 8), "e", Applicability::MachineApplicable),
        );
        let applied = apply(
            &sources,
            vec![rename(vec![("a", 5, 8), ("b", 5, 8)]), clash],
        );
        assert_eq!(applied.texts.get("a"), None);
        assert_eq!(applied.texts["b"], "use ole");
        assert_eq!((applied.fixes.len(), applied.skipped), (1, 1));
        // Self-overlapping edits are skipped; a file without source drops it.
        let applied = apply(
            &sources,
            vec![
                rename(vec![("a", 5, 8), ("a", 6, 7)]),
                rename(vec![("a", 5, 8), ("missing", 1, 1)]),
            ],
        );
        assert!(applied.texts.is_empty());
        assert_eq!((applied.fixes.len(), applied.skipped), (0, 1));
    }

    #[test]
    fn an_insertion_at_a_replaced_ranges_start_lands_before_it() {
        let (text, applied, _) = one("abc", vec![fix(1, 1, 1, "X"), fix(1, 1, 3, "Y")]);
        assert_eq!((text.as_str(), applied), ("XYc", 2));
    }

    proptest! {
        #[test]
        fn insertions_preserve_the_rest(text in "[a-z]{1,30}", at in 1u32..30, ins in "[A-Z]{0,4}") {
            let col = at.min(u32::try_from(text.len()).unwrap_or(1) + 1);
            let (out, _, _) = one(&text, vec![fix(1, col, col, &ins)]);
            prop_assert_eq!(out.replace(&ins, ""), text.replace(&ins, ""));
            prop_assert_eq!(out.len(), text.len() + ins.len());
        }
    }
}
