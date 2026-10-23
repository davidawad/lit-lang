use super::{describe, describe_set, is_decl, is_trivia};
use crate::syntax_kind::SyntaxKind;

#[test]
fn describe_names_every_textless_token_kind() {
    let cases = [
        (SyntaxKind::Arrow, "`->`"),
        (SyntaxKind::KwModule, "`module`"),
        (SyntaxKind::Ident, "an identifier"),
        (SyntaxKind::Number, "a number"),
        (SyntaxKind::Money, "a money literal like `100.usd`"),
        (SyntaxKind::Duration, "a duration like `21.days`"),
        (SyntaxKind::String, "a string"),
        (SyntaxKind::Date, "a date `YYYY-MM-DD`"),
        (SyntaxKind::Eof, "end of file"),
        (SyntaxKind::RecordExpr, "RecordExpr"),
    ];
    for (kind, expected) in cases {
        assert_eq!(describe(kind), expected, "{kind:?}");
    }
}

#[test]
fn describe_set_joins_in_english() {
    assert_eq!(describe_set(&[]), "something else");
    assert_eq!(describe_set(&[SyntaxKind::Ident]), "an identifier");
    assert_eq!(
        describe_set(&[SyntaxKind::Ident, SyntaxKind::Eof]),
        "an identifier or end of file"
    );
    assert_eq!(
        describe_set(&[SyntaxKind::Comma, SyntaxKind::Colon, SyntaxKind::Eof]),
        "one of `,`, `:`, or end of file"
    );
}

#[test]
fn trivia_and_declaration_kinds() {
    assert!(is_trivia(SyntaxKind::DocComment));
    assert!(!is_trivia(SyntaxKind::Ident));
    assert!(is_decl(SyntaxKind::HolidayDecl));
    assert!(!is_decl(SyntaxKind::Field));
}
