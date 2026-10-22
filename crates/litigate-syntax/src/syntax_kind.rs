//! Rowan-compatible token and node kinds for the complete language grammar.
use rowan::Language;

/// Token and syntax-node tags. Discriminants are stable within this crate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u16)]
pub enum SyntaxKind {
    /// Whitespace trivia.
    Whitespace = 0,
    /// Line or block comment trivia.
    Comment,
    /// Documentation comment trivia (`///`).
    DocComment,
    /// Identifier.
    Ident,
    /// Integer, decimal, or rational numeric literal.
    Number,
    /// Quoted string literal.
    String,
    /// Date literal `YYYY-MM-DD`.
    Date,
    /// Legal citation token.
    Cite,
    /// `module` keyword.
    KwModule,
    /// `start` keyword.
    KwStart,
    /// `import` keyword.
    KwImport,
    /// `package` keyword.
    KwPackage,
    /// `type` keyword.
    KwType,
    /// `enum` keyword.
    KwEnum,
    /// `match` keyword.
    KwMatch,
    /// `state` keyword.
    KwState,
    /// `decision` keyword.
    KwDecision,
    /// `chance` keyword.
    KwChance,
    /// `terminal` keyword.
    KwTerminal,
    /// `deadline` keyword.
    KwDeadline,
    /// `counting` keyword.
    KwCounting,
    /// `calendar` keyword.
    KwCalendar,
    /// `transition` keyword.
    KwTransition,
    /// `authority` keyword.
    KwAuthority,
    /// `default` keyword.
    KwDefault,
    /// `exception` keyword.
    KwException,
    /// `duty` keyword.
    KwDuty,
    /// `power` keyword.
    KwPower,
    /// `liability` keyword.
    KwLiability,
    /// `test` keyword.
    KwTest,
    /// `case` keyword.
    KwCase,
    /// `uses` keyword.
    KwUses,
    /// `as_of` keyword.
    KwAsOf,
    /// `perspective` keyword.
    KwPerspective,
    /// `parties` keyword.
    KwParties,
    /// `events` keyword.
    KwEvents,
    /// `facts` keyword.
    KwFacts,
    /// `stakes` keyword.
    KwStakes,
    /// `beliefs` keyword.
    KwBeliefs,
    /// `{` punctuation.
    LBrace,
    /// `}` punctuation.
    RBrace,
    /// `(` punctuation.
    LParen,
    /// `)` punctuation.
    RParen,
    /// `[` punctuation.
    LBracket,
    /// `]` punctuation.
    RBracket,
    /// `:` punctuation.
    Colon,
    /// `;` punctuation.
    Semicolon,
    /// `,` punctuation.
    Comma,
    /// `.` punctuation.
    Dot,
    /// `..` punctuation.
    DotDot,
    /// `->` punctuation.
    Arrow,
    /// `=` punctuation.
    Eq,
    /// `==` punctuation.
    EqEq,
    /// `>` punctuation.
    Greater,
    /// `<` punctuation.
    Less,
    /// `?` hole token.
    Question,
    /// `@` punctuation.
    At,
    /// `+` punctuation.
    Plus,
    /// `-` punctuation.
    Minus,
    /// `subject_to` relation keyword.
    KwSubjectTo,
    /// `notwithstanding` relation keyword.
    KwNotwithstanding,
    /// `supersedes` relation keyword.
    KwSupersedes,
    /// `revives` relation keyword.
    KwRevives,
    /// `delegable_to` relation keyword.
    KwDelegableTo,
    /// `override` relation keyword.
    KwOverride,
    /// `priority` keyword.
    KwPriority,
    /// `when` keyword.
    KwWhen,
    /// `given` keyword.
    KwGiven,
    /// `expect` keyword.
    KwExpect,
    /// `from` keyword.
    KwFrom,
    /// `counted_by` keyword.
    KwCountedBy,
    /// `outcome` keyword.
    KwOutcome,
    /// `group` keyword.
    KwGroup,
    /// `entry` keyword.
    KwEntry,
    /// `fact` keyword.
    KwFact,
    /// `by` keyword.
    KwBy,
    /// `on` keyword.
    KwOn,
    /// `cite` keyword.
    KwCite,
    /// `p` probability field keyword.
    KwProbability,
    /// `hours` field keyword.
    KwHours,
    /// `cost` field keyword.
    KwCost,
    /// `within` field keyword.
    KwWithin,
    /// `duration` field keyword.
    KwDuration,
    /// `sets` field keyword.
    KwSets,
    /// `clears` field keyword.
    KwClears,
    /// `requires` field keyword.
    KwRequires,
    /// `forbids` field keyword.
    KwForbids,
    /// `tags` field keyword.
    KwTags,
    /// `attrs` field keyword.
    KwAttrs,
    /// `effective` keyword.
    KwEffective,
    /// `observed` keyword.
    KwObserved,
    /// `holidays` keyword.
    KwHolidays,
    /// `roll_forward_on` keyword.
    KwRollForwardOn,
    /// `exclude_trigger_day` keyword.
    KwExcludeTriggerDay,
    /// `include_last_day` keyword.
    KwIncludeLastDay,
    /// `service_extension` keyword.
    KwServiceExtension,
    /// `anchor` keyword.
    KwAnchor,
    /// `short_period` keyword.
    KwShortPeriod,
    /// `rebuts` keyword.
    KwRebuts,
    /// `undercuts` keyword.
    KwUndercuts,
    /// `payoff` keyword.
    KwPayoff,
    /// `as` keyword.
    KwAs,
    /// `!=` comparison token.
    NotEq,
    /// `<=` comparison token.
    LessEq,
    /// `>=` comparison token.
    GreaterEq,
    /// `&&` logical token.
    AndAnd,
    /// `||` logical token.
    OrOr,
    /// `/` operator token.
    Slash,
    /// `*` operator token.
    Star,
    /// `%` operator token.
    Percent,
    /// `!` unary token.
    Bang,
    /// `^` version-range token.
    Caret,
    /// `~` version-range token.
    Tilde,
    /// Dedicated enum declaration node.
    EnumDecl,
    /// Package declaration node.
    PackageDecl,
    /// Start-node declaration.
    StartDecl,
    /// Node-group declaration.
    GroupDecl,
    /// Import declaration node.
    ImportDecl,
    /// Transition declaration node.
    TransitionDecl,
    /// Default declaration node.
    DefaultDecl,
    /// Unknown token for parser recovery.
    Unknown,
    /// Root of a rule module.
    SourceFile,
    /// Module/import/package declaration.
    ModuleDecl,
    /// Rule declaration.
    RuleDecl,
    /// Type/enum declaration.
    TypeDecl,
    /// State declaration.
    StateDecl,
    /// Decision declaration.
    DecisionDecl,
    /// Chance declaration.
    ChanceDecl,
    /// Terminal declaration.
    TerminalDecl,
    /// Edge declaration.
    EdgeDecl,
    /// Deadline declaration.
    DeadlineDecl,
    /// Counting declaration.
    CountingDecl,
    /// Calendar declaration.
    CalendarDecl,
    /// Authority declaration.
    AuthorityDecl,
    /// Default/exception declaration.
    ExceptionDecl,
    /// Test declaration.
    TestDecl,
    /// Expression.
    Expr,
    /// Root of a case file.
    CaseFile,
    /// Case event declaration.
    CaseEvent,
    /// Party declaration.
    PartyDecl,
    /// Fact, stake, or belief assignment.
    CaseAssignment,
}

/// Rowan language marker for [`SyntaxKind`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LitLanguage {}

const RAW_KINDS: [SyntaxKind; 141] = [
    SyntaxKind::Whitespace,
    SyntaxKind::Comment,
    SyntaxKind::DocComment,
    SyntaxKind::Ident,
    SyntaxKind::Number,
    SyntaxKind::String,
    SyntaxKind::Date,
    SyntaxKind::Cite,
    SyntaxKind::KwModule,
    SyntaxKind::KwStart,
    SyntaxKind::KwImport,
    SyntaxKind::KwPackage,
    SyntaxKind::KwType,
    SyntaxKind::KwEnum,
    SyntaxKind::KwMatch,
    SyntaxKind::KwState,
    SyntaxKind::KwDecision,
    SyntaxKind::KwChance,
    SyntaxKind::KwTerminal,
    SyntaxKind::KwDeadline,
    SyntaxKind::KwCounting,
    SyntaxKind::KwCalendar,
    SyntaxKind::KwTransition,
    SyntaxKind::KwAuthority,
    SyntaxKind::KwDefault,
    SyntaxKind::KwException,
    SyntaxKind::KwDuty,
    SyntaxKind::KwPower,
    SyntaxKind::KwLiability,
    SyntaxKind::KwTest,
    SyntaxKind::KwCase,
    SyntaxKind::KwUses,
    SyntaxKind::KwAsOf,
    SyntaxKind::KwPerspective,
    SyntaxKind::KwParties,
    SyntaxKind::KwEvents,
    SyntaxKind::KwFacts,
    SyntaxKind::KwStakes,
    SyntaxKind::KwBeliefs,
    SyntaxKind::LBrace,
    SyntaxKind::RBrace,
    SyntaxKind::LParen,
    SyntaxKind::RParen,
    SyntaxKind::LBracket,
    SyntaxKind::RBracket,
    SyntaxKind::Colon,
    SyntaxKind::Semicolon,
    SyntaxKind::Comma,
    SyntaxKind::Dot,
    SyntaxKind::DotDot,
    SyntaxKind::Arrow,
    SyntaxKind::Eq,
    SyntaxKind::EqEq,
    SyntaxKind::Greater,
    SyntaxKind::Less,
    SyntaxKind::Question,
    SyntaxKind::At,
    SyntaxKind::Plus,
    SyntaxKind::Minus,
    SyntaxKind::KwSubjectTo,
    SyntaxKind::KwNotwithstanding,
    SyntaxKind::KwSupersedes,
    SyntaxKind::KwRevives,
    SyntaxKind::KwDelegableTo,
    SyntaxKind::KwOverride,
    SyntaxKind::KwPriority,
    SyntaxKind::KwWhen,
    SyntaxKind::KwGiven,
    SyntaxKind::KwExpect,
    SyntaxKind::KwFrom,
    SyntaxKind::KwCountedBy,
    SyntaxKind::KwOutcome,
    SyntaxKind::KwGroup,
    SyntaxKind::KwEntry,
    SyntaxKind::KwFact,
    SyntaxKind::KwBy,
    SyntaxKind::KwOn,
    SyntaxKind::KwCite,
    SyntaxKind::KwProbability,
    SyntaxKind::KwHours,
    SyntaxKind::KwCost,
    SyntaxKind::KwWithin,
    SyntaxKind::KwDuration,
    SyntaxKind::KwSets,
    SyntaxKind::KwClears,
    SyntaxKind::KwRequires,
    SyntaxKind::KwForbids,
    SyntaxKind::KwTags,
    SyntaxKind::KwAttrs,
    SyntaxKind::KwEffective,
    SyntaxKind::KwObserved,
    SyntaxKind::KwHolidays,
    SyntaxKind::KwRollForwardOn,
    SyntaxKind::KwExcludeTriggerDay,
    SyntaxKind::KwIncludeLastDay,
    SyntaxKind::KwServiceExtension,
    SyntaxKind::KwAnchor,
    SyntaxKind::KwShortPeriod,
    SyntaxKind::KwRebuts,
    SyntaxKind::KwUndercuts,
    SyntaxKind::KwPayoff,
    SyntaxKind::KwAs,
    SyntaxKind::NotEq,
    SyntaxKind::LessEq,
    SyntaxKind::GreaterEq,
    SyntaxKind::AndAnd,
    SyntaxKind::OrOr,
    SyntaxKind::Slash,
    SyntaxKind::Star,
    SyntaxKind::Percent,
    SyntaxKind::Bang,
    SyntaxKind::Caret,
    SyntaxKind::Tilde,
    SyntaxKind::EnumDecl,
    SyntaxKind::PackageDecl,
    SyntaxKind::StartDecl,
    SyntaxKind::GroupDecl,
    SyntaxKind::ImportDecl,
    SyntaxKind::TransitionDecl,
    SyntaxKind::DefaultDecl,
    SyntaxKind::Unknown,
    SyntaxKind::SourceFile,
    SyntaxKind::ModuleDecl,
    SyntaxKind::RuleDecl,
    SyntaxKind::TypeDecl,
    SyntaxKind::StateDecl,
    SyntaxKind::DecisionDecl,
    SyntaxKind::ChanceDecl,
    SyntaxKind::TerminalDecl,
    SyntaxKind::EdgeDecl,
    SyntaxKind::DeadlineDecl,
    SyntaxKind::CountingDecl,
    SyntaxKind::CalendarDecl,
    SyntaxKind::AuthorityDecl,
    SyntaxKind::ExceptionDecl,
    SyntaxKind::TestDecl,
    SyntaxKind::Expr,
    SyntaxKind::CaseFile,
    SyntaxKind::CaseEvent,
    SyntaxKind::PartyDecl,
    SyntaxKind::CaseAssignment,
];

impl Language for LitLanguage {
    type Kind = SyntaxKind;

    fn kind_from_raw(raw: rowan::SyntaxKind) -> Self::Kind {
        RAW_KINDS
            .get(usize::from(raw.0))
            .map_or(SyntaxKind::Unknown, |kind| *kind)
    }

    fn kind_to_raw(kind: Self::Kind) -> rowan::SyntaxKind {
        rowan::SyntaxKind(kind as u16)
    }
}

#[cfg(test)]
mod tests {
    use super::{LitLanguage, SyntaxKind};
    use rowan::Language;
    #[test]
    fn raw_kind_round_trip() {
        assert_eq!(
            LitLanguage::kind_from_raw(LitLanguage::kind_to_raw(SyntaxKind::ChanceDecl)),
            SyntaxKind::ChanceDecl
        );
    }
}
