//! One obligation table for a whole `check` (SPEC §7): `litigate-verify`
//! reports each package and matter; this folds those reports, plus the
//! findings of checks the CLI runs around them (`Lit.lock` freshness), into
//! one finding per obligation.
use litigate_base::{Diagnostic, Severity};
use litigate_verify::{obligation_of, Finding, Obligation, ObligationStatus as S, MAX_EXAMPLES};

fn rank(status: S) -> u8 {
    match status {
        S::Skipped => 0,
        S::Passed => 1,
        S::Warning => 2,
        S::Failed => 3,
    }
}

fn example(d: &Diagnostic) -> String {
    format!(
        "{}:{}:{}: {} {}",
        d.span.file.0,
        d.span.line,
        d.span.col,
        d.code.as_str(),
        d.message
    )
}

/// The findings `diagnostics` make, one per obligation they bear on, as
/// `litigate-verify` derives them: failed on an error, warning on a
/// warning, passed otherwise. Obligations no diagnostic bears on are left
/// out.
#[must_use]
pub fn findings_of(diagnostics: &[Diagnostic]) -> Vec<Finding> {
    Obligation::ALL
        .iter()
        .filter_map(|&name| {
            let mut found: Vec<&Diagnostic> = diagnostics
                .iter()
                .filter(|d| obligation_of(d.code) == Some(name))
                .collect();
            if found.is_empty() {
                return None;
            }
            found.sort_by_key(|d| example(d));
            let status = if found.iter().any(|d| d.severity == Severity::Error) {
                S::Failed
            } else if found.iter().any(|d| d.severity == Severity::Warning) {
                S::Warning
            } else {
                S::Passed
            };
            Some(Finding {
                name,
                status,
                count: found.len(),
                examples: found
                    .iter()
                    .take(MAX_EXAMPLES)
                    .map(|d| example(d))
                    .collect(),
            })
        })
        .collect()
}

/// Folds `parts` (each a report's findings, any order or subset) into one
/// finding per obligation of [`Obligation::ALL`]. An obligation that ran
/// anywhere takes the worst status where it ran, the summed count, and the
/// first [`MAX_EXAMPLES`] examples in part order; one that ran nowhere is
/// skipped with every distinct reason given.
#[must_use]
pub fn merge(parts: &[&[Finding]]) -> Vec<Finding> {
    Obligation::ALL
        .iter()
        .map(|&name| {
            let all: Vec<&Finding> = parts
                .iter()
                .flat_map(|p| p.iter().filter(|f| f.name == name))
                .collect();
            let ran: Vec<&Finding> = all
                .iter()
                .copied()
                .filter(|f| f.status != S::Skipped)
                .collect();
            let Some(status) = ran.iter().map(|f| f.status).max_by_key(|s| rank(*s)) else {
                let mut reasons: Vec<String> = Vec::new();
                for r in all.iter().flat_map(|f| &f.examples) {
                    if !reasons.contains(r) {
                        reasons.push(r.clone());
                    }
                }
                if reasons.is_empty() {
                    reasons.push("nothing was checked".to_owned());
                }
                return Finding {
                    name,
                    status: S::Skipped,
                    count: 0,
                    examples: reasons,
                };
            };
            Finding {
                name,
                status,
                count: ran.iter().map(|f| f.count).sum(),
                examples: ran
                    .iter()
                    .flat_map(|f| f.examples.iter().cloned())
                    .take(MAX_EXAMPLES)
                    .collect(),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use litigate_base::{DiagnosticCode as C, Span};
    use proptest::prelude::*;

    fn status() -> impl Strategy<Value = S> {
        prop_oneof![
            Just(S::Skipped),
            Just(S::Passed),
            Just(S::Warning),
            Just(S::Failed)
        ]
    }

    fn report() -> impl Strategy<Value = Vec<Finding>> {
        proptest::collection::vec((status(), 0..5usize), Obligation::ALL.len()).prop_map(|v| {
            Obligation::ALL
                .iter()
                .zip(v)
                .map(|(&name, (status, count))| {
                    let count = if status == S::Skipped { 0 } else { count };
                    Finding {
                        name,
                        status,
                        count,
                        examples: (0..count.max(1)).map(|i| format!("{i}")).collect(),
                    }
                })
                .collect()
        })
    }

    proptest! {
        #[test]
        fn merging_keeps_the_worst_status_and_the_total_count(
            reports in proptest::collection::vec(report(), 0..4),
        ) {
            let parts: Vec<&[Finding]> = reports.iter().map(Vec::as_slice).collect();
            let merged = merge(&parts);
            prop_assert_eq!(merged.len(), Obligation::ALL.len());
            for (i, f) in merged.iter().enumerate() {
                prop_assert_eq!(f.name, Obligation::ALL[i]);
                let worst = reports.iter().map(|r| r[i].status).max_by_key(|s| rank(*s));
                prop_assert_eq!(Some(f.status), worst.or(Some(S::Skipped)));
                prop_assert_eq!(f.count, reports.iter().map(|r| r[i].count).sum::<usize>());
                prop_assert!(f.examples.len() <= MAX_EXAMPLES.max(reports.len()));
            }
        }
    }

    #[test]
    fn a_stale_lock_fails_well_typed() {
        let d = Diagnostic::error(
            C::StaleLock,
            Span {
                file: "Lit.lock".into(),
                line: 1,
                col: 1,
                end_line: 1,
                end_col: 1,
            },
            "stale",
        );
        let found = findings_of(&[d]);
        assert_eq!(found.len(), 1);
        assert_eq!(
            (found[0].name, found[0].status),
            (Obligation::WellTyped, S::Failed)
        );
        let merged = merge(&[&found]);
        assert_eq!(merged[0].status, S::Failed);
        assert_eq!(merged[1].examples, ["nothing was checked"]);
    }
}
