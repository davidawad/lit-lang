//! Property checks for `.case` matter dates.
use litigate_syntax::{ast::AstNode, parse, FileKind};
use proptest::prelude::*;

proptest! {
    #[test]
    fn prop_case_has_one_stable_as_of_date(year in 1_u32..=9999, month in 1_u32..=12, day in 1_u32..=28) {
        let date = format!("{year:04}-{month:02}-{day:02}");
        let text = format!("case m\nuses p ^1.0.0\nas_of {date}\nsummary \"matter\"\n");
        let parsed = parse(&text, FileKind::Case);
        prop_assert!(parsed.is_ok(), "{:#?}", parsed.errors());
        let case = litigate_syntax::ast::CaseFile::cast(parsed.syntax().clone());
        prop_assert_eq!(case.and_then(|c| c.as_of()).map(|token| token.text().to_owned()), Some(date));
    }

    #[test]
    fn prop_duplicate_as_of_fields_are_rejected(first_day in 1_u32..=28, second_day in 1_u32..=28) {
        let text = format!(
            "case m\nuses p ^1.0.0\nas_of 2025-02-{first_day:02}\nas_of 2025-03-{second_day:02}\nsummary \"matter\"\n"
        );
        prop_assert!(!parse(&text, FileKind::Case).is_ok(), "duplicate as_of accepted: {text}");
    }
}
