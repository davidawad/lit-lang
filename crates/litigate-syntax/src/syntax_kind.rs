//! Rowan-compatible token and node kinds for the complete language grammar
//! (SPEC §2-§3).
//!
//! Keywords are contextual: the lexer emits [`SyntaxKind::Ident`] for every
//! word and the parser remaps a token to its keyword kind (see
//! [`SyntaxKind::from_keyword`]) only where the grammar expects that keyword.
//! The keyword kinds are generated from `keywords.txt`
//! ([`crate::keyword_table`]); add a keyword there, never here.
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

// The keyword variants come from `keywords.txt` (`build.rs`).
include!(concat!(env!("OUT_DIR"), "/keywords.rs"));

syntax_kinds_with_keywords! {
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
        Pipe = "|",
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
        EventDecl => "`event Name(param: Type, ...)`, with optional effect clauses (`removes p`, `elects p`, `extends d to p`, `waives d`) and the kind of paper it files (`paper k`).",
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
        CiteExpr => "`cite \"...\" [historical]` citation.",
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
        ForumDecl => "`forum pkg ^1.0.0`: the matter's procedure package, primary.",
        GoverningLawDecl => "`governing_law pkg ^1.0.0`: the package rooting the tree whose substantive layers govern.",
        AsOfDecl => "`as_of date`.",
        KnownAtDecl => "`known_at date` in a case.",
        SummaryDecl => "`summary \"...\"`.",
        NotesDecl => "`notes \"...\"`.",
        SourceDecl => "`source \"title\" { ... }`.",
        PerspectiveDecl => "`perspective party` or `perspective { actor: role }`.",
        PartiesBlock => "`parties { ... }`.",
        PartyDecl => "`id: Actor EntityType`.",
        EventsBlock => "`events { ... }`.",
        CaseEvent => "`[id:] date [docket N] Call by|on party [cite \"...\"]`.",
        FactsBlock => "`facts { node: edge }`.",
        StakesBlock => "`stakes { terminal: money|unvalued [fee_shift fraction] }`.",
        BeliefsBlock => "`beliefs { prior ... observe ... }`.",
        ObserveDecl => "`observe node: { edge: count }`.",
        PriorDecl => "`prior node: { edge: pseudo-count }`: a Dirichlet prior (Beta for two outcomes).",
        AnalysisBlock => "`analysis { field: value }`.",
        Error => "Recovered parse error node.",
        ImportList => "Braced name list of `import m.{x, y}` (additive, W1 syntax).",
        Param => "One `name: Type` event parameter (additive, W1 syntax).",
        FieldExpr => "Field projection `expr.field` after a non-ref primary (additive, W1 syntax).",
        Ratio => "Probability ratio `n/d` written without spaces (additive, W1 syntax).",
        EditionDecl => "Leading edition marker `edition 2026` of a `.lit` or `.case` file (additive).",
        RefinedType => "Refinement type `{x: T | pred}`: base type `T` restricted by a predicate over `x` (SPEC §3.5).",
        WhereClause => "Refinement clause `where pred` after a parameter's or a type declaration's type (SPEC §3.5).",
        GuardOp => "Reserved typed guard operation in a patch: `when and pred`, `when or pred` or `supersedes when pred`, with its authority; always L0010 (SPEC §3.6).",
        LimitDecl => "Filing-requirement declaration `limit paper-kind { ... }` with its versions and authority (SPEC §3.7).",
        LimitBound => "Length bound `words|pages|lines <= N` of a `limit` body (SPEC §3.7).",
        CertificateReq => "`requires certificate kind [for [measures]]` of a `limit` body (SPEC §3.7).",
        Exclusions => "`excludes [item, ?(\"reason\"), ...]` of a `limit` body (SPEC §3.7).",
        EcfCode => "`ecf \"code\"` ECF event code of a `limit` body, as data (SPEC §3.7).",
        GrantDecl => "`grant id on [targets] { ... }`: an emergency capability grant for a bounded window (additive).",
        IndexClause => "`Rule[a, s, v, k, ap]` after a rule's name: its authority, scope (class), valid, transaction and applicability indices, each `_` for its default (additive).",
        RecharacterizeDecl => "`recharacterize id: target characterization jurisdictional|claim_processing`: a new characterization of another package's deadline (additive).",
        InvalidateDecl => "`invalidate id: target`: withdraws another package's declaration from its window (additive).",
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
