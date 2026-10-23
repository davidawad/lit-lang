use super::Parser;
use crate::lexer::lex;
use crate::syntax_kind::SyntaxKind;
use crate::vocab::LIT_ITEM_KEYWORDS;
use rowan::SyntaxNode;

/// Grammar rules check the current token before bumping, but the cursor
/// itself stays total: bumping at end of file emits nothing.
#[test]
fn bumping_at_end_of_file_is_a_no_op() {
    let text = "module m";
    let (tokens, errors) = lex(text);
    assert!(errors.is_empty());
    let mut p = Parser::new(text, tokens, LIT_ITEM_KEYWORDS);
    p.start(SyntaxKind::SourceFile);
    assert!(p.eat_kw(SyntaxKind::KwModule));
    p.bump();
    assert!(p.at_end());
    p.bump();
    p.bump_as(SyntaxKind::Ident);
    assert!(p.at_end());
    let (green, errors) = p.finish_file();
    assert!(errors.is_empty());
    let root = SyntaxNode::<crate::syntax_kind::LitLanguage>::new_root(green);
    assert_eq!(root.text().to_string(), text);
    let kinds: Vec<SyntaxKind> = root
        .children_with_tokens()
        .map(|element| element.kind())
        .collect();
    assert_eq!(
        kinds,
        [
            SyntaxKind::KwModule,
            SyntaxKind::Whitespace,
            SyntaxKind::Ident
        ]
    );
}
