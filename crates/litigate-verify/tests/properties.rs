//! Report invariants over arbitrary diagnostics and holes: every report
//! lists every obligation once, in order; acceptance is exactly "no error
//! attributed to an obligation, and no hole in strict"; counts never
//! depend on the profile.
use litigate_base::{Diagnostic, DiagnosticCode, FileId, Profile, Severity, Span};
use litigate_sema::{Hole, PackageCheck};
use litigate_verify::{obligation_of, verify_unchecked, Obligation, MAX_EXAMPLES};
use proptest::prelude::*;

fn span(line: u32) -> Span {
    Span {
        file: FileId("pkgs/demo/main.lit".to_owned()),
        line,
        col: 1,
        end_line: line,
        end_col: 2,
    }
}

fn check(codes: &[(usize, u32)], holes: u32, profile: Profile) -> PackageCheck {
    let diagnostics = codes
        .iter()
        .map(|&(i, line)| {
            let code = DiagnosticCode::ALL[i % DiagnosticCode::ALL.len()];
            Diagnostic {
                severity: code.severity(profile),
                ..Diagnostic::error(code, span(line), code.explain())
            }
        })
        .collect();
    PackageCheck {
        package: None,
        diagnostics,
        holes: (0..holes)
            .map(|line| Hole {
                span: span(line + 1),
                reason: None,
            })
            .collect(),
    }
}

fn profile() -> impl Strategy<Value = Profile> {
    prop_oneof![Just(Profile::Dev), Just(Profile::Strict)]
}

proptest! {
    #[test]
    fn reports_list_every_obligation_and_accept_exactly_the_clean(
        codes in prop::collection::vec((0usize..64, 1u32..400), 0..24),
        holes in 0u32..4,
        profile in profile(),
    ) {
        let c = check(&codes, holes, profile);
        let v = verify_unchecked(&c, profile);
        let names: Vec<Obligation> = v.report.obligations.iter().map(|f| f.name).collect();
        prop_assert_eq!(names, Obligation::ALL.to_vec());
        prop_assert_eq!(v.report.profile, profile);
        let errors = c.diagnostics.iter().any(|d| {
            d.severity == Severity::Error && obligation_of(d.code).is_some()
        });
        let strict_holes = profile == Profile::Strict && holes > 0;
        prop_assert_eq!(v.is_accepted(), !errors && !strict_holes);
        for f in &v.report.obligations {
            prop_assert!(f.examples.len() <= MAX_EXAMPLES);
            let attributed = c.diagnostics.iter().filter(|d| obligation_of(d.code) == Some(f.name)).count();
            let own = if f.name == Obligation::Holes { holes as usize } else { 0 };
            prop_assert_eq!(f.count, attributed + own, "{:?}", f.name);
        }
    }

    #[test]
    fn counts_do_not_depend_on_the_profile(
        codes in prop::collection::vec((0usize..64, 1u32..400), 0..24),
        holes in 0u32..4,
    ) {
        let count = |p: Profile| -> Vec<usize> {
            verify_unchecked(&check(&codes, holes, p), p)
                .report
                .obligations
                .iter()
                .map(|f| f.count)
                .collect()
        };
        prop_assert_eq!(count(Profile::Dev), count(Profile::Strict));
    }
}
