//! Accessors for `.case` matters.
use super::{
    child, children, fields_of, token, tokens, CallExpr, CaseBlock, CaseEvent, CaseFile, Cite,
    Field, Literal, Node, ObserveDecl, PartyDecl, PerspectiveDecl, RecordExpr, RecordField, Ref,
    SourceDecl, StartDecl, Token, UseItem, UsesDecl, VersionReq,
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
    /// `uses` pins.
    #[must_use]
    pub fn uses(&self) -> Option<UsesDecl> {
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
    /// Event date.
    #[must_use]
    pub fn date(&self) -> Option<Token> {
        literal_token(self.syntax())
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
    /// Block keyword (`facts`, `stakes`, `beliefs`, `analysis`).
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
