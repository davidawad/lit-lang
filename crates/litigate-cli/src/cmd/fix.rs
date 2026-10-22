//! `litigate fix PATHS` (DESIGN §8 "an agent edits by fixes"): apply the
//! fixes `check` attaches to diagnostics. Only `machine_applicable` fixes
//! by default; `--include-maybe` adds `maybe_incorrect`. Overlapping edits
//! keep the one nearest the end of the file; run `fix` again for the rest.
use super::build::write_atomic;
use super::check::checked;
use crate::out::{Failure, Outcome, Report};
use crate::span::byte_range;
use litigate_base::{Applicability, DiagnosticCode, Fix, Profile};
use serde_json::json;
use std::collections::BTreeMap;
use std::path::Path;

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

/// Applies `fixes` (one file) to `text`, latest first, skipping overlaps.
/// Returns the new text, the applied fixes, and how many were skipped.
#[must_use]
pub fn apply(
    text: &str,
    mut fixes: Vec<(DiagnosticCode, Fix)>,
) -> (String, Vec<(DiagnosticCode, Fix)>, usize) {
    fixes.sort_by_key(|(_, f)| std::cmp::Reverse(byte_range(text, &f.span).start));
    let mut out = text.to_owned();
    let mut floor = usize::MAX;
    let mut applied = Vec::new();
    let mut skipped = 0;
    for (code, fix) in fixes {
        let range = byte_range(text, &fix.span);
        if range.end > floor
            || applied
                .iter()
                .any(|(_, f): &(DiagnosticCode, Fix)| *f == fix)
        {
            skipped += 1;
            continue;
        }
        out.replace_range(range.clone(), &fix.replacement);
        floor = range.start;
        applied.push((code, fix));
    }
    applied.reverse();
    (out, applied, skipped)
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
    let c = checked(&args.paths, args.profile, None, &args.lib)?;
    let allowed = |a: Applicability| {
        a == Applicability::MachineApplicable
            || (args.include_maybe && a == Applicability::MaybeIncorrect)
    };
    let mut by_file: BTreeMap<String, Vec<(DiagnosticCode, Fix)>> = BTreeMap::new();
    for d in c
        .diagnostics
        .iter()
        .filter(|d| wanted.is_none_or(|w| d.code == w))
    {
        for f in d.fixes.iter().filter(|f| allowed(f.applicability)) {
            by_file
                .entry(f.span.file.0.clone())
                .or_default()
                .push((d.code, f.clone()));
        }
    }
    let sources: BTreeMap<String, String> = c.session.sources().into_iter().collect();
    let mut edits = Vec::new();
    let mut changed = Vec::new();
    let mut skipped = 0;
    for (file, fixes) in by_file {
        let Some(text) = sources.get(&file) else {
            continue;
        };
        let (fixed, applied, rest) = apply(text, fixes);
        skipped += rest;
        if fixed != *text {
            if !args.dry_run {
                write_atomic(Path::new(&file), &fixed)?;
            }
            changed.push(file.clone());
        }
        edits.extend(applied.into_iter().map(|(code, f)| json!({"code": code, "span": f.span, "replacement": f.replacement, "applicability": f.applicability})));
    }
    Ok(Report {
        result: json!({"edits": edits, "changed_files": changed, "skipped_overlapping": skipped, "dry_run": args.dry_run}),
        provenance: c.provenance(None, None),
        ..Report::default()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use litigate_base::Span;
    use proptest::prelude::*;

    fn fix(line: u32, col: u32, end_col: u32, replacement: &str) -> (DiagnosticCode, Fix) {
        let span = Span {
            file: "f".into(),
            line,
            col,
            end_line: line,
            end_col,
        };
        (
            DiagnosticCode::UnexpectedToken,
            Fix {
                span,
                replacement: replacement.into(),
                applicability: Applicability::MachineApplicable,
            },
        )
    }

    #[test]
    fn applies_from_the_end_and_skips_overlaps() {
        let (text, applied, skipped) = apply(
            "ab cd\nef",
            vec![fix(1, 1, 3, "X"), fix(2, 1, 2, "Y"), fix(1, 2, 4, "Z")],
        );
        assert_eq!(text, "aZcd\nYf");
        assert_eq!(applied.len(), 2);
        assert_eq!(skipped, 1);
    }

    proptest! {
        #[test]
        fn insertions_preserve_the_rest(text in "[a-z]{1,30}", at in 1u32..30, ins in "[A-Z]{0,4}") {
            let col = at.min(u32::try_from(text.len()).unwrap_or(1) + 1);
            let (out, _, _) = apply(&text, vec![fix(1, col, col, &ins)]);
            prop_assert_eq!(out.replace(&ins, ""), text.replace(&ins, ""));
            prop_assert_eq!(out.len(), text.len() + ins.len());
        }
    }
}
