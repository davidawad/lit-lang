use super::{cite, duration, is_cite_text, is_ident, is_ref, money, number, string};
use litigate_syntax::lexer::lex;
use litigate_syntax::SyntaxKind;
use proptest::prelude::*;

/// The significant tokens `text` lexes to, as `(kind, text)`.
fn tokens(text: &str) -> Vec<(SyntaxKind, String)> {
    let (tokens, _) = lex(text);
    tokens
        .iter()
        .filter(|t| t.kind != SyntaxKind::Whitespace)
        .map(|t| (t.kind, text[t.range].to_owned()))
        .collect()
}

#[test]
fn identifiers_follow_the_lexer() {
    for good in [
        "a",
        "answer-due",
        "12b-motion-filed",
        "_x",
        "a_b-c1",
        "default",
    ] {
        assert!(is_ident(good), "{good}");
    }
    for bad in [
        "", "_", "cite", "a-", "-a", "1", "1-a", "a--b", "a.b", "a b", "é",
    ] {
        assert!(!is_ident(bad), "{bad}");
    }
    assert!(is_ref("cafc@ptab::window-blown"));
    assert!(!is_ref("pack::a->b#0"));
}

#[test]
fn numbers_money_and_durations_spell_exactly() {
    assert_eq!(number(0.35).as_deref(), Some("0.35"));
    assert_eq!(number(1.0).as_deref(), Some("1"));
    assert_eq!(number(-0.0).as_deref(), Some("0"));
    assert_eq!(number(f64::NAN), None);
    assert_eq!(money(-2_000_000.0).as_deref(), Some("-2_000_000.usd"));
    assert_eq!(money(12.5).as_deref(), Some("12.5.usd"));
    assert_eq!(money(405.0).as_deref(), Some("405.usd"));
    assert_eq!(money(0.125), None);
    assert_eq!(duration(4.5, "days", false).as_deref(), Some("4.5.days"));
    assert_eq!(duration(4.5, "days", true), None);
    assert_eq!(duration(-1.0, "days", false), None);
    assert_eq!(
        duration(21.0, "court_days", true).as_deref(),
        Some("21.court_days")
    );
}

#[test]
fn cites_fall_back_to_holes() {
    assert_eq!(cite("FRCP 12(b)(6)", true), "cite FRCP 12(b)(6)");
    assert_eq!(cite("FRCP 12(b)(6)", false), "cite ?(\"FRCP 12(b)(6)\")");
    assert_eq!(
        cite("RCFC App. J, Rule 3", true),
        "cite ?(\"RCFC App. J, Rule 3\")"
    );
    for bad in [
        "a, b", "a {b}", "a // b", "a  b", " a", "a ", "?a", "a\nb", "",
    ] {
        assert!(!is_cite_text(bad), "{bad:?}");
    }
}

#[test]
fn strings_escape_what_the_lexer_requires() {
    assert_eq!(string("a\"b\\c\nd\te\r"), r#""a\"b\\c\nd\te\r""#);
}

proptest! {
    #[test]
    fn prop_string_literals_decode_to_their_input(value in "\\PC*") {
        let spelled = string(&value);
        let lexed = tokens(&spelled);
        prop_assert_eq!(lexed.len(), 1);
        prop_assert_eq!(lexed[0].0, SyntaxKind::String);
        prop_assert_eq!(litigate_syntax::ast::unescape(&spelled), value);
    }

    #[test]
    fn prop_identifier_check_agrees_with_the_lexer(text in "[a-z0-9_-]{1,12}") {
        let lexed = tokens(&text);
        let one_ident = lexed.len() == 1 && lexed[0].0 == SyntaxKind::Ident && text != "cite";
        prop_assert_eq!(is_ident(&text), one_ident, "{:?}", lexed);
    }

    #[test]
    fn prop_numbers_round_trip_through_their_decimal_text(value in proptest::num::f64::NORMAL | proptest::num::f64::ZERO) {
        let spelled = number(value).unwrap_or_default();
        let back: f64 = spelled.parse().unwrap_or(f64::NAN);
        prop_assert!(back == value, "{} -> {}", value, spelled);
        let unsigned = spelled.trim_start_matches('-');
        prop_assert_eq!(tokens(unsigned).len(), 1);
    }

    #[test]
    fn prop_money_is_exact_cents(cents in -10_000_000_000i64..10_000_000_000) {
        let value = cents as f64 / 100.0;
        let spelled = money(value).unwrap_or_default();
        let digits = spelled.trim_end_matches(".usd").replace('_', "");
        let back: f64 = digits.parse().unwrap_or(f64::NAN);
        prop_assert!(back == value, "{} -> {}", value, spelled);
        let lexed = tokens(spelled.trim_start_matches('-'));
        prop_assert_eq!(lexed.len(), 1);
        prop_assert_eq!(lexed[0].0, SyntaxKind::Money);
    }

    #[test]
    fn prop_cite_text_survives_citation_mode(text in "[A-Za-z0-9.§()\\- ;]{1,30}") {
        if is_cite_text(&text) {
            let lexed = tokens(&format!("cite {text}"));
            prop_assert_eq!(lexed.len(), 2);
            prop_assert_eq!(&lexed[1].1, &text);
        }
    }
}
