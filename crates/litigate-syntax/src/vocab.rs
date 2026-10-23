//! Grammar vocabulary: item keyword sets, declaration kinds, trivia, and
//! human descriptions of token kinds for diagnostics.
use crate::syntax_kind::SyntaxKind::{self, *};

/// Top-level item keywords of `.lit` files.
pub const LIT_ITEM_KEYWORDS: &[SyntaxKind] = &[
    KwModule,
    KwImport,
    KwPackage,
    KwStart,
    KwGroup,
    KwType,
    KwEnum,
    KwEvent,
    KwState,
    KwDecision,
    KwChance,
    KwTerminal,
    KwLink,
    KwInstance,
    KwRule,
    KwDeadline,
    KwCounting,
    KwCalendar,
    KwTransition,
    KwAuthority,
    KwDefault,
    KwException,
    KwTest,
];

/// Top-level item keywords of `.case` files.
pub const CASE_ITEM_KEYWORDS: &[SyntaxKind] = &[
    KwCase,
    KwUses,
    KwAsOf,
    KwKnownAt,
    KwSummary,
    KwNotes,
    KwSource,
    KwStart,
    KwPerspective,
    KwParties,
    KwEvents,
    KwFacts,
    KwStakes,
    KwBeliefs,
    KwAnalysis,
];

/// Declarations that keep their leading doc comments inside their node.
pub(crate) const fn is_decl(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        ModuleDecl
            | ImportDecl
            | PackageDecl
            | StartDecl
            | GroupDecl
            | TypeDecl
            | EnumDecl
            | EventDecl
            | StateDecl
            | DecisionDecl
            | ChanceDecl
            | TerminalDecl
            | EdgeDecl
            | LinkDecl
            | InstanceDecl
            | RuleDecl
            | DeadlineDecl
            | CountingDecl
            | CalendarDecl
            | HolidayDecl
            | TransitionDecl
            | AuthorityDecl
            | DefaultDecl
            | ExceptionDecl
            | TestDecl
    )
}

/// Whitespace and comments (doc comments included).
#[must_use]
pub const fn is_trivia(kind: SyntaxKind) -> bool {
    matches!(kind, Whitespace | Comment | DocComment)
}

/// Human description of an expected token kind.
#[must_use]
pub fn describe(kind: SyntaxKind) -> std::string::String {
    match kind.text() {
        Some(text) => format!("`{text}`"),
        None => match kind {
            Ident => "an identifier".into(),
            Number => "a number".into(),
            Money => "a money literal like `100.usd`".into(),
            Duration => "a duration like `21.days`".into(),
            String => "a string".into(),
            Date => "a date `YYYY-MM-DD`".into(),
            Eof => "end of file".into(),
            other => format!("{other:?}"),
        },
    }
}

pub(crate) fn describe_set(expected: &[SyntaxKind]) -> std::string::String {
    let parts: Vec<std::string::String> = expected.iter().map(|kind| describe(*kind)).collect();
    match parts.as_slice() {
        [] => "something else".into(),
        [one] => one.clone(),
        [first, second] => format!("{first} or {second}"),
        [init @ .., last] => format!("one of {}, or {last}", init.join(", ")),
    }
}

#[cfg(test)]
mod tests;
