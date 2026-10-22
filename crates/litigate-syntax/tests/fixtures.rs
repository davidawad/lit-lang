//! The SPEC §12 worked examples are the shared W1 fixtures: they must exist
//! and already satisfy the layout invariants of canonical formatting (§10).
use litigate_syntax::syntax_kind::SyntaxKind;
use std::path::Path;

const FIXTURES: [&str; 3] = [
    "lit/examples/rule12.lit",
    "lit/examples/Lit.toml",
    "cases/examples/acme.case",
];

fn read(relative: &str) -> String {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    std::fs::read_to_string(root.join(relative)).unwrap_or_default()
}

#[test]
fn fixtures_exist_and_follow_canonical_layout() {
    for fixture in FIXTURES {
        let text = read(fixture);
        assert!(!text.is_empty(), "{fixture} is missing or empty");
        assert!(text.ends_with('\n') && !text.ends_with("\n\n"), "{fixture}");
        for (number, line) in text.lines().enumerate() {
            let at = format!("{fixture}:{}", number + 1);
            assert!(line.chars().count() <= 100, "{at} exceeds 100 columns");
            assert!(!line.contains('\t') && !line.contains('\r'), "{at}");
            assert_eq!(line.trim_end(), line, "{at} has trailing whitespace");
        }
    }
}

#[test]
fn fixtures_lead_with_their_file_keyword() {
    let first_word = |text: &str| text.split_whitespace().next().map(str::to_owned);
    let lit = first_word(&read("lit/examples/rule12.lit"));
    let case = first_word(&read("cases/examples/acme.case"));
    assert_eq!(
        lit.as_deref().and_then(SyntaxKind::from_keyword),
        Some(SyntaxKind::KwModule)
    );
    assert_eq!(
        case.as_deref().and_then(SyntaxKind::from_keyword),
        Some(SyntaxKind::KwCase)
    );
}
