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
        assert!(code.starts_with('L'));
        assert!(code[1..].bytes().all(|b| b.is_ascii_digit()));
        assert!(matches!(
            &code[1..2],
            "0" | "1" | "2" | "3" | "4" | "5" | "6" | "9"
        ));
        assert!(!variant.explain().is_empty());
        assert!(!variant.explain().ends_with('.'));
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
            } else if matches!(
                code,
                DiagnosticCode::UnusedDeclaration
                    | DiagnosticCode::ObligationBreached
                    | DiagnosticCode::TimelinessUnknown
            ) || (profile == Profile::Dev
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
    assert!(serde_json::from_str::<DiagnosticCode>("\"L39990\"").is_err());
}

#[test]
fn well_formed_unknown_codes_decode_tolerantly() {
    let unknown = DiagnosticCode::decode("L3999").unwrap();
    assert!(!unknown.is_known());
    assert!(DiagnosticCode::TypeMismatch.is_known());
    assert_eq!(unknown.as_str(), "L3999");
    assert_eq!(DiagnosticCode::decode("L0000").unwrap().as_str(), "L0000");
    assert!(unknown.explain().contains("does not know"));
    assert_eq!(unknown.severity(Profile::Dev), Severity::Error);
    assert!(!unknown.is_lint());
    assert!(DiagnosticCode::decode("L9999").unwrap().is_lint());
    assert_eq!(
        DiagnosticCode::decode("L3004"),
        Some(DiagnosticCode::MissingDelegation)
    );
    assert_eq!(DiagnosticCode::parse("L3999"), None);
    for malformed in ["", "L", "L399", "L39999", "M3999", "L39a9", "L+999"] {
        assert_eq!(DiagnosticCode::decode(malformed), None, "{malformed}");
    }
    assert_eq!(
        serde_json::from_str::<DiagnosticCode>("\"L3999\"").unwrap(),
        unknown
    );
    assert_eq!(serde_json::to_string(&unknown).unwrap(), "\"L3999\"");
}

#[test]
fn exactly_the_l3_codes_name_a_derivation_rule() {
    for code in DiagnosticCode::ALL {
        assert_eq!(
            code.derivation_rule().is_some(),
            code.as_str().starts_with("L3"),
            "{}",
            code.as_str()
        );
    }
    assert_eq!(
        DiagnosticCode::decode("L3999").unwrap().derivation_rule(),
        None
    );
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
    assert!(C::UnusedDeclaration.is_lint());
    assert!(!C::HoleInStrict.is_lint());
}

proptest! {
    #[test]
    fn unknown_codes_never_parse_but_always_decode(s in "L[0-9]{4}") {
        let known = DiagnosticCode::ALL.iter().any(|c| c.as_str() == s);
        prop_assert_eq!(DiagnosticCode::parse(&s).is_some(), known);
        let decoded = DiagnosticCode::decode(&s).unwrap();
        prop_assert_eq!(decoded.as_str(), s.as_str());
        prop_assert_eq!(decoded.is_known(), known);
    }
}

#[test]
fn json_schema_is_the_closed_code_set() {
    let schema = schemars::schema_for!(DiagnosticCode);
    let value = serde_json::to_value(schema).unwrap();
    assert_eq!(value["title"], "DiagnosticCode");
    assert_eq!(value["anyOf"][1]["pattern"], "^L[0-9]{4}$");
    let codes = value["anyOf"][0]["enum"].as_array().unwrap();
    assert_eq!(codes.len(), DiagnosticCode::ALL.len());
    assert_eq!(codes[0], DiagnosticCode::ALL[0].as_str());
}
