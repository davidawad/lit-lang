//! Accessors for `.case` matters.
use super::{
    child, children, fields_of, token, tokens, CallExpr, CaseBlock, CaseEvent, CaseFile, Cite,
    Field, ForumDecl, GoverningLawDecl, Literal, Node, ObserveDecl, PartyDecl, PerspectiveDecl,
    PriorDecl, RecordExpr, RecordField, Ref, SourceDecl, StartDecl, Token, UseItem, UsesDecl,
    VersionReq,
};
use crate::syntax_kind::SyntaxKind;

/// A version requirement split into operator and version text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VersionText {
    /// `^`, `~`, `>=`, or `=`.
    pub op: String,
    /// `MAJOR.MINOR.PATCH[-pre]`.
    pub version: String,
}

fn node_of(node: &Node, kind: SyntaxKind) -> Option<Node> {
    node.children().find(|child| child.kind() == kind)
}

fn literal_token(node: &Node) -> Option<Token> {
    child::<Literal>(node).and_then(|literal| literal.token())
}

impl CaseFile {
    /// The first `uses` declaration.
    #[must_use]
    pub fn uses(&self) -> Option<UsesDecl> {
        child(self.syntax())
    }

    /// Every pin the matter writes, in order: the `forum` (the primary
    /// package), every pin of every `uses` declaration (a matter may write
    /// one `uses` per package), then the `governing_law`.
    #[must_use]
    pub fn use_items(&self) -> Vec<UseItem> {
        let forum = self.forum().and_then(|f| child::<UseItem>(f.syntax()));
        let law = self
            .governing_law()
            .and_then(|g| child::<UseItem>(g.syntax()));
        forum
            .into_iter()
            .chain(
                children::<UsesDecl>(self.syntax())
                    .iter()
                    .flat_map(UsesDecl::items),
            )
            .chain(law)
            .collect()
    }

    /// The `forum` declaration.
    #[must_use]
    pub fn forum(&self) -> Option<ForumDecl> {
        child(self.syntax())
    }

    /// The `governing_law` declaration.
    #[must_use]
    pub fn governing_law(&self) -> Option<GoverningLawDecl> {
        child(self.syntax())
    }

    /// `as_of` date.
    #[must_use]
    pub fn as_of(&self) -> Option<Token> {
        node_of(self.syntax(), SyntaxKind::AsOfDecl).and_then(|n| literal_token(&n))
    }

    /// `known_at` date.
    #[must_use]
    pub fn known_at(&self) -> Option<Token> {
        node_of(self.syntax(), SyntaxKind::KnownAtDecl).and_then(|n| literal_token(&n))
    }

    /// `summary` string token.
    #[must_use]
    pub fn summary(&self) -> Option<Token> {
        node_of(self.syntax(), SyntaxKind::SummaryDecl).and_then(|n| literal_token(&n))
    }

    /// `notes` string token.
    #[must_use]
    pub fn notes(&self) -> Option<Token> {
        node_of(self.syntax(), SyntaxKind::NotesDecl).and_then(|n| literal_token(&n))
    }

    /// `source` declarations.
    #[must_use]
    pub fn sources(&self) -> Vec<SourceDecl> {
        children(self.syntax())
    }

    /// Explicit `start`.
    #[must_use]
    pub fn start(&self) -> Option<StartDecl> {
        child(self.syntax())
    }

    /// `perspective`.
    #[must_use]
    pub fn perspective(&self) -> Option<PerspectiveDecl> {
        child(self.syntax())
    }

    /// Parties in order.
    #[must_use]
    pub fn parties(&self) -> Vec<PartyDecl> {
        node_of(self.syntax(), SyntaxKind::PartiesBlock).map_or_else(Vec::new, |n| children(&n))
    }

    /// Events in source order.
    #[must_use]
    pub fn events(&self) -> Vec<CaseEvent> {
        node_of(self.syntax(), SyntaxKind::EventsBlock).map_or_else(Vec::new, |n| children(&n))
    }

    /// The `facts`, `stakes`, `beliefs`, and `analysis` blocks.
    #[must_use]
    pub fn blocks(&self) -> Vec<CaseBlock> {
        children(self.syntax())
    }

    /// The block of `kind` (`FactsBlock`, `StakesBlock`, ...).
    #[must_use]
    pub fn block(&self, kind: SyntaxKind) -> Option<CaseBlock> {
        self.blocks()
            .into_iter()
            .find(|block| block.syntax().kind() == kind)
    }
}

impl UsesDecl {
    /// Pins in order (the first is the primary package).
    #[must_use]
    pub fn items(&self) -> Vec<UseItem> {
        children(self.syntax())
    }
}

impl UseItem {
    /// Package name.
    #[must_use]
    pub fn package(&self) -> Option<Token> {
        self.name()
    }

    /// Version requirement.
    #[must_use]
    pub fn requirement(&self) -> Option<VersionReq> {
        child(self.syntax())
    }
}

impl VersionReq {
    /// Operator and reassembled version text.
    #[must_use]
    pub fn text(&self) -> Option<VersionText> {
        let op = tokens(self.syntax()).find(|token| {
            matches!(
                token.kind(),
                SyntaxKind::Caret | SyntaxKind::Tilde | SyntaxKind::GreaterEq | SyntaxKind::Eq
            )
        })?;
        let version = node_of(self.syntax(), SyntaxKind::Version)?
            .text()
            .to_string();
        Some(VersionText {
            op: op.text().to_owned(),
            version,
        })
    }
}

impl SourceDecl {
    /// Source title string token.
    #[must_use]
    pub fn title(&self) -> Option<Token> {
        literal_token(self.syntax())
    }

    /// `url` / `as_of` fields.
    #[must_use]
    pub fn fields(&self) -> Vec<Field> {
        fields_of(self.syntax())
    }
}

impl PerspectiveDecl {
    /// `perspective party`.
    #[must_use]
    pub fn party(&self) -> Option<Ref> {
        child(self.syntax())
    }

    /// `perspective { actor: role }`.
    #[must_use]
    pub fn roles(&self) -> Option<RecordExpr> {
        child(self.syntax())
    }
}

impl PartyDecl {
    /// The party's actor (a package role).
    #[must_use]
    pub fn actor(&self) -> Option<Ref> {
        child(self.syntax())
    }

    /// Entity type.
    #[must_use]
    pub fn entity(&self) -> Option<Ref> {
        children::<Ref>(self.syntax()).into_iter().nth(1)
    }
}

impl CaseEvent {
    /// The declared id before `:`, if written (its timeline key, SPEC
    /// §5.6).
    #[must_use]
    pub fn id(&self) -> Option<Token> {
        token(self.syntax(), SyntaxKind::Ident)
    }

    /// Event date.
    #[must_use]
    pub fn date(&self) -> Option<Token> {
        literal_token(self.syntax())
    }

    /// The docket sequence number after `docket`, if written.
    #[must_use]
    pub fn docket(&self) -> Option<Token> {
        tokens(self.syntax())
            .skip_while(|token| token.kind() != SyntaxKind::KwDocket)
            .find(|token| token.kind() == SyntaxKind::Number)
    }

    /// Event constructor call.
    #[must_use]
    pub fn call(&self) -> Option<CallExpr> {
        child(self.syntax())
    }

    /// `by` or `on`.
    #[must_use]
    pub fn mode(&self) -> Option<Token> {
        tokens(self.syntax())
            .find(|token| matches!(token.kind(), SyntaxKind::KwBy | SyntaxKind::KwOn))
    }

    /// The acting (or served) party.
    #[must_use]
    pub fn party(&self) -> Option<Ref> {
        child(self.syntax())
    }

    /// Supporting citation.
    #[must_use]
    pub fn cite(&self) -> Option<Cite> {
        child(self.syntax())
    }
}

impl CaseBlock {
    /// Block keyword (`facts`, `stakes`, `beliefs`, `opponents`, `analysis`).
    #[must_use]
    pub fn keyword(&self) -> Option<Token> {
        super::keyword(self.syntax())
    }

    /// The braced record of `facts`, `stakes`, and `analysis`.
    #[must_use]
    pub fn record(&self) -> Option<RecordExpr> {
        child(self.syntax())
    }

    /// Record entries of `facts`/`stakes`/`analysis`.
    #[must_use]
    pub fn entries(&self) -> Vec<RecordField> {
        self.record()
            .map_or_else(Vec::new, |record| record.fields())
    }

    /// `observe` entries of `beliefs`.
    #[must_use]
    pub fn observations(&self) -> Vec<ObserveDecl> {
        children(self.syntax())
    }

    /// `prior` entries of `beliefs`.
    #[must_use]
    pub fn priors(&self) -> Vec<PriorDecl> {
        children(self.syntax())
    }
}

impl PriorDecl {
    /// The chance node the prior is over.
    #[must_use]
    pub fn node(&self) -> Option<Ref> {
        child(self.syntax())
    }

    /// Pseudo-counts by outcome edge.
    #[must_use]
    pub fn counts(&self) -> Option<RecordExpr> {
        child(self.syntax())
    }
}

impl ObserveDecl {
    /// Observed chance node.
    #[must_use]
    pub fn node(&self) -> Option<Ref> {
        child(self.syntax())
    }

    /// Outcome counts.
    #[must_use]
    pub fn counts(&self) -> Option<RecordExpr> {
        child(self.syntax())
    }
}

impl StartDecl {
    /// The `start` keyword token.
    #[must_use]
    pub fn keyword(&self) -> Option<Token> {
        token(self.syntax(), SyntaxKind::KwStart)
    }
}
