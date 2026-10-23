//! Rowan-compatible token and node kinds for the complete language grammar
//! (SPEC §2-§3).
//!
//! Keywords are contextual: the lexer emits [`SyntaxKind::Ident`] for every
//! word and the parser remaps a token to its keyword kind (see
//! [`SyntaxKind::from_keyword`]) only where the grammar expects that keyword.
use rowan::Language;

macro_rules! syntax_kinds {
    (
        tokens { $($tok:ident => $tok_doc:literal,)+ }
        punct { $($punct:ident = $punct_text:literal,)+ }
        keywords { $($kw:ident = $kw_text:literal,)+ }
        nodes { $($node:ident => $node_doc:literal,)+ }
    ) => {
        /// Token and syntax-node tags. Discriminants are stable within a
        /// compiler version; they are never serialized.
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        #[repr(u16)]
        pub enum SyntaxKind {
            $(#[doc = $tok_doc] $tok,)+
            $(#[doc = concat!("`", $punct_text, "` token.")] $punct,)+
            $(#[doc = concat!("`", $kw_text, "` contextual keyword.")] $kw,)+
            $(#[doc = $node_doc] $node,)+
        }

        impl SyntaxKind {
            /// Every kind in discriminant order.
            pub const ALL: &'static [SyntaxKind] = &[
                $(Self::$tok,)+ $(Self::$punct,)+ $(Self::$kw,)+ $(Self::$node,)+
            ];

            /// Fixed source text of a punctuation or keyword kind.
            #[must_use]
            pub const fn text(self) -> Option<&'static str> {
                match self {
                    $(Self::$punct => Some($punct_text),)+
                    $(Self::$kw => Some($kw_text),)+
                    _ => None,
                }
            }

            /// Keyword kind spelled `word`, if any.
            #[must_use]
            pub fn from_keyword(word: &str) -> Option<Self> {
                match word {
                    $($kw_text => Some(Self::$kw),)+
                    _ => None,
                }
            }

            /// True for contextual keyword kinds.
            #[must_use]
            pub const fn is_keyword(self) -> bool {
                matches!(self, $(Self::$kw)|+)
            }

            /// True for composite (node) kinds.
            #[must_use]
            pub const fn is_node(self) -> bool {
                matches!(self, $(Self::$node)|+)
            }
        }
    };
}

syntax_kinds! {
    tokens {
        Whitespace => "Whitespace trivia.",
        Comment => "Line or block comment trivia.",
        DocComment => "Documentation comment (`///`), attached to the next declaration.",
        Ident => "Identifier, possibly kebab-case; also every unremapped keyword.",
        Number => "Decimal number `21`, `2_000`, or `0.35`.",
        Money => "Money literal `2_000_000.usd`.",
        Duration => "Duration literal `21.days`, `5.court_days`, `3.business_days`, `8.hours`.",
        String => "Quoted string literal.",
        Date => "Date literal `YYYY-MM-DD`.",
        CiteText => "Citation text following the `cite` keyword.",
        ErrorToken => "Unrecognized input kept for lossless recovery.",
        Eof => "End-of-input lookahead sentinel; never stored in a tree (additive, W1 syntax).",
    }
    punct {
        LBrace = "{", RBrace = "}", LParen = "(", RParen = ")",
        LBracket = "[", RBracket = "]", Colon = ":", ColonColon = "::",
        Comma = ",", Dot = ".", DotDot = "..", Arrow = "->", FatArrow = "=>",
        Eq = "=", EqEq = "==", NotEq = "!=", Less = "<", LessEq = "<=",
        Greater = ">", GreaterEq = ">=", Plus = "+", Minus = "-", Star = "*",
        Slash = "/", Percent = "%", Bang = "!", AndAnd = "&&", OrOr = "||",
        Question = "?", At = "@", Caret = "^", Tilde = "~", Underscore = "_",
    }
    keywords {
        KwModule = "module", KwImport = "import", KwPackage = "package",
        KwForum = "forum", KwStart = "start", KwGroup = "group",
        KwType = "type", KwEnum = "enum", KwMatch = "match",
        KwState = "state", KwDecision = "decision", KwChance = "chance",
        KwTerminal = "terminal", KwRule = "rule", KwDuty = "duty",
        KwPower = "power", KwLiability = "liability", KwDeadline = "deadline",
        KwCounting = "counting", KwCalendar = "calendar", KwTransition = "transition",
        KwAuthority = "authority", KwDefault = "default", KwException = "exception",
        KwTest = "test", KwLink = "link", KwInstance = "instance",
        KwEvent = "event", KwAmends = "amends", KwAllowHours = "allow_hours",
        KwEndOfMonth = "end_of_month",
        KwBelow = "below", KwCount = "count",
        KwCase = "case", KwUses = "uses", KwAsOf = "as_of", KwKnownAt = "known_at",
        KwSummary = "summary", KwNotes = "notes", KwSource = "source",
        KwPerspective = "perspective", KwParties = "parties", KwEvents = "events",
        KwFacts = "facts", KwStakes = "stakes", KwBeliefs = "beliefs",
        KwObserve = "observe", KwAnalysis = "analysis", KwUnvalued = "unvalued",
        KwBy = "by", KwOn = "on", KwDocket = "docket", KwFrom = "from", KwAfter = "after",
        KwBefore = "before", KwCountedBy = "counted_by", KwCite = "cite",
        KwOutcome = "outcome", KwPayoff = "payoff", KwPayoffByFlag = "payoff_by_flag",
        KwDiscretion = "discretion", KwP = "p", KwEffort = "effort", KwCost = "cost",
        KwWithin = "within", KwDuration = "duration", KwMin = "min", KwMode = "mode",
        KwMax = "max", KwSets = "sets", KwClears = "clears", KwRequires = "requires",
        KwForbids = "forbids", KwTags = "tags", KwAttrs = "attrs", KwNote = "note",
        KwValence = "valence", KwCourtListenerUrl = "court_listener_url",
        KwActionId = "action_id", KwExtendable = "extendable",
        KwExtensionAuthority = "extension_authority", KwReplaces = "replaces",
        KwPack = "pack", KwRemoveEdges = "remove_edges",
        KwProbabilities = "probabilities", KwRoles = "roles",
        KwPayoffTransform = "payoff_transform", KwStake = "stake", KwPatch = "patch",
        KwUnset = "unset",
        KwEffective = "effective", KwExcludeTriggerDay = "exclude_trigger_day",
        KwIncludeLastDay = "include_last_day", KwRollForwardOn = "roll_forward_on",
        KwServiceExtension = "service_extension", KwHolidays = "holidays",
        KwAnchor = "anchor", KwShortPeriod = "short_period", KwObserved = "observed",
        KwSubjectTo = "subject_to", KwNotwithstanding = "notwithstanding",
        KwSupersedes = "supersedes", KwRevives = "revives",
        KwDelegableTo = "delegable_to", KwOverride = "override", KwWhen = "when",
        KwRebuts = "rebuts", KwUndercuts = "undercuts", KwPriority = "priority",
        KwUnless = "unless", KwOtherwise = "otherwise", KwGiven = "given",
        KwExpect = "expect", KwTrue = "true", KwFalse = "false", KwUnit = "unit",
        KwUrl = "url", KwEdition = "edition", KwBearer = "bearer", KwTrigger = "trigger",
        KwExtend = "extend", KwWaive = "waive", KwAfterExpiry = "after_expiry", KwNever = "never",
        KwRemoves = "removes", KwElects = "elects", KwExtends = "extends",
        KwWaives = "waives", KwTo = "to",
    }
    nodes {
        SourceFile => "Root of a `.lit` module.",
        CaseFile => "Root of a `.case` matter.",
        ModuleDecl => "`module a.b`.",
        ImportDecl => "`import a.b` or `import a.{x, y}`.",
        PackageDecl => "`package name { forum f authority A }`.",
        StartDecl => "`start node-id` (lit) or `start pack::node` (case).",
        GroupDecl => "`group id: \"Label\"`.",
        TypeDecl => "`type Name = Type`.",
        EnumDecl => "`enum Name { A, B }`.",
        EventDecl => "`event Name(param: Type, ...)`, with an optional effect clause (`removes p`, `elects p`, `extends d to p`, `waives d`).",
        ParamList => "Parenthesized `name: Type` parameter list.",
        StateDecl => "`state` node declaration.",
        DecisionDecl => "`decision` node declaration.",
        ChanceDecl => "`chance` node declaration.",
        TerminalDecl => "`terminal` node declaration.",
        FieldList => "Braced, comma-separated field list.",
        Field => "One `keyword value` field.",
        EdgeBlock => "Braced edge list of a node.",
        EdgeDecl => "`id \"label\" -> target ...`.",
        LinkDecl => "`link` cross-package edge.",
        InstanceDecl => "`instance base@origin of pack { ... }`.",
        PatchDecl => "`patch edge-id { ... }` inside an instance.",
        RuleDecl => "`rule duty|power|liability id { ... }`.",
        DeadlineDecl => "`deadline id: 21.days from Trigger counted_by rule`.",
        CountingDecl => "`counting id { ... }`.",
        CalendarDecl => "`calendar id { ... }`.",
        HolidayDecl => "One holiday line of a calendar.",
        TransitionDecl => "`transition id amends ref { unless ... -> v, otherwise -> v }`.",
        TransitionArm => "One transition arm.",
        AuthorityDecl => "`authority A > B > C`.",
        DefaultDecl => "`default id when cond => value`.",
        ExceptionDecl => "`exception id rebuts|undercuts target when ...`.",
        TestDecl => "`test \"name\" { ... }`.",
        GivenStmt => "`given expr on date`.",
        ExpectStmt => "`expect lhs op rhs`.",
        VersionClause => "`effective [from, to)` and/or `known_at date`.",
        Interval => "`[date, date|..)`.",
        Relation => "`subject_to x`, `supersedes x`, ... .",
        Ref => "Dotted or `::`-qualified name, optionally `@version`.",
        Version => "Version after `@` or in a `uses` pin.",
        VersionReq => "`^1.2.0`, `~1.2.0`, `>=1.2.0`, or `=1.2.0`.",
        IdList => "Bracketed identifier/ref list `[a, b]`.",
        TypeExpr => "Type expression.",
        TypeArgs => "`<...>` type arguments.",
        Literal => "Literal expression.",
        HoleExpr => "`?` or `?(\"reason\")`.",
        CiteExpr => "`cite ...` citation.",
        RefExpr => "Reference expression.",
        CallExpr => "Call `f(a, b)`.",
        ArgList => "Parenthesized argument list.",
        UnaryExpr => "Prefix `!`/`-`.",
        BinaryExpr => "Infix operator expression.",
        ParenExpr => "`(expr)`.",
        RecordExpr => "`{ a: x, b: y }`.",
        RecordField => "`name: expr` (name may be a qualified ref in case blocks).",
        ListExpr => "`[a, b]`.",
        MatchExpr => "`match e { ... }`.",
        MatchArm => "`pattern => expr`.",
        UsesDecl => "`uses pkg ^1.0.0, ...`.",
        UseItem => "One `pkg version-req` pin.",
        AsOfDecl => "`as_of date`.",
        KnownAtDecl => "`known_at date` in a case.",
        SummaryDecl => "`summary \"...\"`.",
        NotesDecl => "`notes \"...\"`.",
        SourceDecl => "`source \"title\" { ... }`.",
        PerspectiveDecl => "`perspective party` or `perspective { actor: role }`.",
        PartiesBlock => "`parties { ... }`.",
        PartyDecl => "`id: Actor EntityType`.",
        EventsBlock => "`events { ... }`.",
        CaseEvent => "`date [docket N] Call by|on party [cite ...]`.",
        FactsBlock => "`facts { node: edge }`.",
        StakesBlock => "`stakes { terminal: money|unvalued }`.",
        BeliefsBlock => "`beliefs { observe ... }`.",
        ObserveDecl => "`observe node: { edge: count }`.",
        AnalysisBlock => "`analysis { field: value }`.",
        Error => "Recovered parse error node.",
        ImportList => "Braced name list of `import m.{x, y}` (additive, W1 syntax).",
        Param => "One `name: Type` event parameter (additive, W1 syntax).",
        FieldExpr => "Field projection `expr.field` after a non-ref primary (additive, W1 syntax).",
        Ratio => "Probability ratio `n/d` written without spaces (additive, W1 syntax).",
        EditionDecl => "Leading edition marker `edition 2026` of a `.lit` or `.case` file (additive).",
    }
}

/// Rowan language marker for [`SyntaxKind`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LitLanguage {}

impl Language for LitLanguage {
    type Kind = SyntaxKind;

    fn kind_from_raw(raw: rowan::SyntaxKind) -> Self::Kind {
        SyntaxKind::ALL
            .get(usize::from(raw.0))
            .map_or(SyntaxKind::ErrorToken, |kind| *kind)
    }

    fn kind_to_raw(kind: Self::Kind) -> rowan::SyntaxKind {
        rowan::SyntaxKind(kind as u16)
    }
}

impl From<SyntaxKind> for rowan::SyntaxKind {
    fn from(kind: SyntaxKind) -> Self {
        LitLanguage::kind_to_raw(kind)
    }
}

#[cfg(test)]
mod tests {
    use super::{LitLanguage, SyntaxKind};
    use rowan::Language;

    #[test]
    fn every_kind_round_trips_through_raw() {
        for (index, kind) in SyntaxKind::ALL.iter().enumerate() {
            assert_eq!(*kind as usize, index);
            assert_eq!(
                LitLanguage::kind_from_raw(LitLanguage::kind_to_raw(*kind)),
                *kind
            );
        }
    }

    #[test]
    fn keywords_are_unique_and_looked_up_by_text() {
        let keywords: Vec<SyntaxKind> = SyntaxKind::ALL
            .iter()
            .copied()
            .filter(|k| k.is_keyword())
            .collect();
        for kind in &keywords {
            let text = kind.text().unwrap_or_default();
            assert_eq!(SyntaxKind::from_keyword(text), Some(*kind), "{text}");
        }
        assert_eq!(SyntaxKind::from_keyword("as_of"), Some(SyntaxKind::KwAsOf));
        assert_eq!(SyntaxKind::from_keyword("answer-due"), None);
        assert!(!SyntaxKind::LBrace.is_keyword());
        assert!(SyntaxKind::EdgeDecl.is_node());
    }
}
