use super::DiagnosticCode;
use crate::{Profile, Severity};
use proptest::prelude::*;

#[test]
fn code_registry_round_trips_and_explains() {
    let code = DiagnosticCode::MissingDelegation;
    assert_eq!(DiagnosticCode::parse(code.as_str()), Some(code));
    assert!(code.explain().contains("delegation"));
    assert_eq!(DiagnosticCode::parse("L3999"), None);
}

#[test]
fn registry_is_sorted_unique_and_well_formed() {
    let codes: Vec<&str> = DiagnosticCode::ALL.iter().map(|c| c.as_str()).collect();
    assert!(codes.windows(2).all(|w| w[0] < w[1]), "{codes:?}");
    for (code, variant) in codes.iter().zip(DiagnosticCode::ALL) {
        assert_eq!(code.len(), 5);
        assert!(code.starts_with('L') && code[1..].bytes().all(|b| b.is_ascii_digit()));
        assert!(matches!(
            &code[1..2],
            "0" | "1" | "2" | "3" | "4" | "5" | "6" | "9"
        ));
        assert!(!variant.explain().is_empty() && !variant.explain().ends_with('.'));
        assert_eq!(DiagnosticCode::parse(code), Some(variant));
    }
}

#[test]
fn every_registered_code_exercises_its_stable_views() {
    for code in DiagnosticCode::ALL {
        assert_eq!(DiagnosticCode::parse(code.as_str()), Some(code));
        assert!(!code.explain().is_empty());
        assert_eq!(
            serde_json::to_string(&code).unwrap(),
            format!("\"{}\"", code.as_str())
        );
        assert_eq!(
            serde_json::from_str::<DiagnosticCode>(&format!("\"{}\"", code.as_str())).unwrap(),
            code
        );
        assert_eq!(code.is_lint(), code.as_str().starts_with("L9"));
        for profile in [Profile::Dev, Profile::Strict] {
            let severity = code.severity(profile);
            if code == DiagnosticCode::AnonymousEdge {
                assert_eq!(severity, Severity::Info);
            } else if matches!(code, DiagnosticCode::UnusedDeclaration)
                || (profile == Profile::Dev
                    && matches!(
                        code,
                        DiagnosticCode::RuleWithoutTest
                            | DiagnosticCode::HolePresent
                            | DiagnosticCode::NonCanonicalFormat
                    ))
            {
                assert_eq!(severity, Severity::Warning);
            } else {
                assert_eq!(severity, Severity::Error);
            }
        }
    }
    assert!(serde_json::from_str::<DiagnosticCode>("\"L3999\"").is_err());
}

#[test]
fn severities_follow_the_profile_table() {
    use DiagnosticCode as C;
    assert_eq!(C::HolePresent.severity(Profile::Dev), Severity::Warning);
    assert_eq!(
        C::RuleWithoutTest.severity(Profile::Strict),
        Severity::Error
    );
    assert_eq!(C::AnonymousEdge.severity(Profile::Strict), Severity::Info);
    assert_eq!(C::TypeMismatch.severity(Profile::Dev), Severity::Error);
    assert!(C::UnusedDeclaration.is_lint() && !C::HoleInStrict.is_lint());
}

proptest! {
    #[test]
    fn unknown_codes_never_parse(s in "L[0-9]{4}") {
        let known = DiagnosticCode::ALL.iter().any(|c| c.as_str() == s);
        prop_assert_eq!(DiagnosticCode::parse(&s).is_some(), known);
    }
}

#[test]
fn json_schema_is_the_closed_code_set() {
    let schema = schemars::schema_for!(DiagnosticCode);
    let value = serde_json::to_value(schema).unwrap();
    assert_eq!(value["title"], "DiagnosticCode");
    let codes = value["enum"].as_array().unwrap();
    assert_eq!(codes.len(), DiagnosticCode::ALL.len());
    assert_eq!(codes[0], DiagnosticCode::ALL[0].as_str());
}
