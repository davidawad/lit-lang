//! The additive forms `litigate import` needs to spell legacy pack data
//! losslessly (SPEC §2-§3): cite holes, decimal `duration` estimates,
//! partial duration ranges, and identifiers that start with digits.
use litigate_base::DiagnosticCode;
use litigate_syntax::{format, parse, FileKind};

fn codes(text: &str) -> Vec<DiagnosticCode> {
    parse(text, FileKind::Lit)
        .errors()
        .iter()
        .map(|error| error.code)
        .collect()
}

/// Parses cleanly, and formatting is idempotent and keeps it clean.
fn canonical(text: &str) {
    assert_eq!(codes(text), [], "{text}");
    let once = format(text, FileKind::Lit).unwrap_or_default();
    assert_eq!(codes(&once), [], "{once}");
    assert_eq!(format(&once, FileKind::Lit).as_deref(), Ok(once.as_str()));
}

#[test]
fn cite_holes_keep_text_that_has_no_cite_spelling() {
    canonical(
        "module m\n\nstart a\n\nstate a: \"A\" cite ?(\"RCFC Appendix J, Rule 3\") {\n  \
         x \"X\" -> a { authority cite ?(\"35 U.S.C. §§ 141, 319\") }\n}\n",
    );
    canonical("module m\n\nstart a\n\nstate a: \"A\" cite ? { group g }\n");
}

#[test]
fn duration_estimates_may_use_decimal_days() {
    canonical(
        "module m\n\nstart a\n\nstate a: \"A\" {\n  \
         x \"Wait\" -> a { duration { min 1.5.days, mode 4.5.days, max 10.5.days }, note \"n\" }\n}\n",
    );
    canonical("module m\n\nstart a\n\nstate a: \"A\" { x \"X\" -> a { duration 2.1.days } }\n");
}

#[test]
fn other_day_amounts_must_stay_whole() {
    let invalid = [DiagnosticCode::InvalidNumber];
    assert_eq!(
        codes("module m\nstate a: \"A\" { x \"X\" -> a { deadline 1.5.days } }\n"),
        invalid
    );
    assert_eq!(
        codes("module m\ndeadline d: 2.5.court_days from F() counted_by c\n"),
        invalid
    );
    assert_eq!(
        codes("module m\ntest \"t\" { expect 1.5.days == 2.days }\n"),
        invalid
    );
    assert_eq!(
        codes("module m\nstate a: \"A\" { x \"X\" -> a { effort 0.5.hours } }\n"),
        []
    );
}

#[test]
fn duration_ranges_may_omit_min_or_max_but_not_mode() {
    canonical(
        "module m\n\nstart a\n\nstate a: \"A\" { x \"X\" -> a { duration { mode 3.days, max 9.days } } }\n",
    );
    canonical(
        "module m\n\nstart a\n\nstate a: \"A\" { x \"X\" -> a { duration { min 1.days, mode 3.days } } }\n",
    );
    assert_eq!(
        codes(
            "module m\nstate a: \"A\" { x \"X\" -> a { duration { min 1.days, max 3.days } } }\n"
        ),
        [DiagnosticCode::UnexpectedToken]
    );
    assert_eq!(
        codes(
            "module m\nstate a: \"A\" { x \"X\" -> a { duration { max 3.days, mode 1.days } } }\n"
        ),
        [
            DiagnosticCode::UnexpectedToken,
            DiagnosticCode::UnexpectedToken
        ]
    );
}

#[test]
fn edge_ids_may_start_with_digits() {
    canonical("module m\n\nstart a\n\nstate a: \"A\" { 12b-motion-filed \"Move\" -> a }\n");
}
