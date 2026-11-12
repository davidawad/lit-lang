//! `basis` on nodes and edges (design review 2026-10-23 §2.10): one of
//! four forms, typed when the package builds, with diagnostics
//! snapshotted.
mod common;
use common::check_body;

#[test]
fn every_basis_form_types() {
    let out = check_body(
        "chance n: \"N\" {\n  basis expert(\"Jane Roe\")\n  a \"A\" -> t p 1/2 { basis calibrated(\"fjc-idb\", 2024-09-30) }\n  b \"B\" -> t p 1/2 { basis estimate }\n}\nterminal u: \"U\" { payoff 1.usd, basis sourced(\"FRCP 41(b)\") }",
    );
    insta::assert_debug_snapshot!(out);
}

#[test]
fn a_malformed_basis_says_what_a_basis_is() {
    let out = check_body(
        "state n: \"N\" {\n  basis estimat\n  a \"A\" -> t { basis a.b }\n  b \"B\" -> t { basis x.expert(\"J\") }\n  c \"C\" -> t { basis guess(\"J\") }\n  d \"D\" -> t { basis expert(1) }\n  e \"E\" -> t { basis expert(\"  \") }\n  f \"F\" -> t { basis calibrated(\"d\", \"2024-09-30\") }\n  g \"G\" -> t { basis sourced(cite \"FRCP 1\") }\n  h \"H\" -> t { basis 1 }\n  i \"I\" -> t { basis expert(\"a\", \"b\") }\n}",
    );
    insta::assert_debug_snapshot!(out);
}

#[test]
fn numeric_suffix_uses_object_basis_semantics_and_requires_vintage_for_estimate() {
    let invalid = check_body("terminal t: \"T\" { payoff 1.usd basis mystery }\n");
    assert!(
        invalid.iter().any(|line| line.contains("L2001")),
        "{invalid:#?}"
    );

    let missing_vintage = check_body("terminal t: \"T\" { payoff 1.usd basis estimate }\n");
    assert!(
        missing_vintage.iter().any(|line| line.contains("L9012")),
        "{missing_vintage:#?}"
    );
}

#[test]
fn a_number_suffix_uses_the_same_calibration_scope_diagnostic_as_object_basis() {
    let object = check_body("terminal t: \"T\" { basis calibrated(missing) payoff 1.usd }\n");
    let suffix = check_body(
        "terminal t: \"T\" { payoff 1.usd basis calibrated(missing) vintage 2026-01-01 }\n",
    );
    let object = object.iter().find(|line| line.contains("L9012")).unwrap();
    let suffix = suffix.iter().find(|line| line.contains("L9012")).unwrap();
    assert!(object.contains("basis names no calibration set `missing` in scope"));
    assert!(suffix.contains("basis names no calibration set `missing` in scope"));
}
