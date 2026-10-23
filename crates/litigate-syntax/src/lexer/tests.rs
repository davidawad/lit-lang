#![allow(clippy::enum_glob_use)]
use super::{lex, valid_date};
use crate::syntax_kind::SyntaxKind::{self, *};
use litigate_base::{Applicability, DiagnosticCode};

fn kinds(text: &str) -> Vec<(SyntaxKind, std::string::String)> {
    let (tokens, _) = lex(text);
    tokens
        .iter()
        .filter(|token| token.kind != Whitespace)
        .map(|token| (token.kind, text[token.range].to_owned()))
        .collect()
}

fn codes(text: &str) -> Vec<DiagnosticCode> {
    lex(text).1.iter().map(|error| error.code).collect()
}

#[test]
fn identifiers_take_hyphens_only_between_continue_characters() {
    assert_eq!(kinds("answer-due"), [(Ident, "answer-due".into())]);
    assert_eq!(
        kinds("answer- a->b"),
        [
            (Ident, "answer".into()),
            (Minus, "-".into()),
            (Ident, "a".into()),
            (Arrow, "->".into()),
            (Ident, "b".into()),
        ]
    );
    assert_eq!(
        kinds("_ _x"),
        [(Underscore, "_".into()), (Ident, "_x".into())]
    );
}

#[test]
fn typed_literals_are_single_tokens() {
    assert_eq!(
        kinds("2_000_000.usd 21.days 5.court_days 3.business_days 8.5.hours 0.35 2025-03-03"),
        [
            (Money, "2_000_000.usd".into()),
            (Duration, "21.days".into()),
            (Duration, "5.court_days".into()),
            (Duration, "3.business_days".into()),
            (Duration, "8.5.hours".into()),
            (Number, "0.35".into()),
            (Date, "2025-03-03".into()),
        ]
    );
}

#[test]
fn semver_lexes_as_number_dot_number() {
    assert_eq!(
        kinds("2024.12.0"),
        [
            (Number, "2024.12".into()),
            (Dot, ".".into()),
            (Number, "0".into())
        ]
    );
}

#[test]
fn literal_validation_reports_stable_codes() {
    assert_eq!(codes("1_"), [DiagnosticCode::InvalidNumber]);
    assert_eq!(codes("1__0"), [DiagnosticCode::InvalidNumber]);
    // Whole days are a parser check (`duration` estimates may be decimal).
    assert_eq!(codes("1.5.days"), []);
    assert_eq!(codes("2025-02-29"), [DiagnosticCode::InvalidDate]);
    assert_eq!(codes("2024-02-29"), []);
    assert_eq!(codes("2025-01-01x"), [DiagnosticCode::InvalidToken]);
    assert_eq!(codes("\"abc"), [DiagnosticCode::UnterminatedLiteral]);
    assert_eq!(codes("\"a\\q\""), [DiagnosticCode::InvalidToken]);
    assert_eq!(codes("/* open"), [DiagnosticCode::UnterminatedLiteral]);
    assert_eq!(codes("\u{feff}x"), [DiagnosticCode::InvalidToken]);
    assert_eq!(
        codes("§§ $"),
        [DiagnosticCode::InvalidToken, DiagnosticCode::InvalidToken]
    );
}

#[test]
fn unknown_suffix_suggests_the_nearest_unit() {
    let (_, errors) = lex("21.dayz");
    let fix = &errors[0].fixes[0];
    assert_eq!(errors[0].code, DiagnosticCode::InvalidNumber);
    assert_eq!(fix.replacement, "days");
    assert_eq!(fix.applicability, Applicability::MachineApplicable);
}

#[test]
fn cite_switches_to_citation_mode() {
    assert_eq!(
        kinds("cite  FRCP 12(a)(1)  {"),
        [
            (KwCite, "cite".into()),
            (CiteText, "FRCP 12(a)(1)".into()),
            (LBrace, "{".into())
        ]
    );
    assert_eq!(
        kinds("cite 28 U.S.C. § 1498, x"),
        [
            (KwCite, "cite".into()),
            (CiteText, "28 U.S.C. § 1498".into()),
            (Comma, ",".into()),
            (Ident, "x".into())
        ]
    );
    assert_eq!(
        kinds("cite R 4 // note\n"),
        [
            (KwCite, "cite".into()),
            (CiteText, "R 4".into()),
            (Comment, "// note".into())
        ]
    );
    assert_eq!(codes("cite\n"), [DiagnosticCode::UnexpectedToken]);
}

#[test]
fn comments_split_into_line_doc_and_block() {
    assert_eq!(
        kinds("// a\n/// b\n/* c */"),
        [
            (Comment, "// a".into()),
            (DocComment, "/// b".into()),
            (Comment, "/* c */".into())
        ]
    );
}

#[test]
fn lexing_is_lossless_on_odd_input() {
    for text in [
        "",
        "\r\n",
        "é-ü ~^@",
        "\"\\",
        "cite",
        "2025-13-45-",
        "1.x.y",
    ] {
        let (tokens, _) = lex(text);
        let joined: std::string::String = tokens.iter().map(|t| &text[t.range]).collect();
        assert_eq!(joined, text);
    }
}

#[test]
fn dates_follow_the_gregorian_calendar() {
    assert!(valid_date("2000-02-29"));
    assert!(!valid_date("1900-02-29"));
    assert!(!valid_date("2025-04-31"));
    assert!(!valid_date("2025-00-10"));
}

#[test]
fn digits_then_a_letter_start_an_identifier() {
    assert_eq!(
        kinds("12b-motion-filed"),
        [(Ident, "12b-motion-filed".into())]
    );
    assert_eq!(kinds("12"), [(Number, "12".into())]);
    assert_eq!(kinds("2_000.usd"), [(Money, "2_000.usd".into())]);
    assert_eq!(
        kinds("0-rc"),
        [
            (Number, "0".into()),
            (Minus, "-".into()),
            (Ident, "rc".into())
        ]
    );
}

#[test]
fn a_question_mark_after_cite_is_a_hole_not_citation_text() {
    assert_eq!(
        kinds("cite ?(\"A, B\")"),
        [
            (KwCite, "cite".into()),
            (Question, "?".into()),
            (LParen, "(".into()),
            (String, "\"A, B\"".into()),
            (RParen, ")".into()),
        ]
    );
    assert_eq!(codes("cite ?"), []);
}

#[test]
fn two_character_operators_are_single_tokens() {
    assert_eq!(
        kinds("a != b == c <= d >= e && f || g :: h .. i => j -> k"),
        [
            (Ident, "a".into()),
            (NotEq, "!=".into()),
            (Ident, "b".into()),
            (EqEq, "==".into()),
            (Ident, "c".into()),
            (LessEq, "<=".into()),
            (Ident, "d".into()),
            (GreaterEq, ">=".into()),
            (Ident, "e".into()),
            (AndAnd, "&&".into()),
            (Ident, "f".into()),
            (OrOr, "||".into()),
            (Ident, "g".into()),
            (ColonColon, "::".into()),
            (Ident, "h".into()),
            (DotDot, "..".into()),
            (Ident, "i".into()),
            (FatArrow, "=>".into()),
            (Ident, "j".into()),
            (Arrow, "->".into()),
            (Ident, "k".into()),
        ]
    );
}

#[test]
fn runs_of_invalid_input_merge_into_one_token_and_error() {
    let ranges = |text: &str| {
        let (tokens, errors) = lex(text);
        let tokens: Vec<_> = tokens
            .iter()
            .map(|token| (token.kind, text[token.range].to_owned()))
            .collect();
        let errors: Vec<_> = errors
            .iter()
            .map(|error| (error.code, text[error.range].to_owned()))
            .collect();
        (tokens, errors)
    };
    let invalid = DiagnosticCode::InvalidToken;
    assert_eq!(
        ranges("$§$"),
        (
            vec![(ErrorToken, "$§$".into())],
            vec![(invalid, "$§$".into())]
        )
    );
    // A byte order mark and the invalid input after it are one error.
    assert_eq!(
        ranges("\u{feff}$x"),
        (
            vec![(ErrorToken, "\u{feff}$".into()), (Ident, "x".into())],
            vec![(invalid, "\u{feff}$".into())]
        )
    );
    // Invalid input glued to a bad unit literal joins its token; the unit
    // error already covers the run.
    assert_eq!(
        ranges("1.dayz$"),
        (
            vec![(ErrorToken, "1.dayz$".into())],
            vec![(DiagnosticCode::InvalidNumber, "dayz".into())]
        )
    );
}

#[test]
fn malformed_date_text_is_not_a_date() {
    for text in [
        "",
        "2025",
        "2025-01",
        "2025-01-01-01",
        "y-01-01",
        "2025-x-01",
        "2025-01-x",
    ] {
        assert!(!valid_date(text), "{text}");
    }
}
