//! Accessors for `.lit` headers, items, and graph declarations.
use super::{
    child, child_after, children, cites_of, fields_of, keyword, token, tokens, AuthorityDecl, Cite,
    EdgeBlock, EdgeDecl, EnumDecl, EventDecl, Expr, Field, FieldList, GroupDecl, IdList,
    ImportDecl, ImportList, Interval, LinkDecl, LitFile, Literal, ModuleDecl, Node, NodeDecl,
    PackageDecl, Param, RecordExpr, Ref, Relation, StartDecl, Token, TypeDecl, TypeExpr,
    VersionClause,
};
use crate::syntax_kind::SyntaxKind;

/// Any top-level `.lit` item (also the items of a `rule` body).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Item(Node);

impl Item {
    /// Views `node` as an item when its kind is a declaration.
    #[must_use]
    pub fn cast(node: Node) -> Option<Self> {
        #[allow(clippy::enum_glob_use)]
        use SyntaxKind::*;
        matches!(
            node.kind(),
            StartDecl
                | GroupDecl
                | TypeDecl
                | EnumDecl
                | EventDecl
                | StateDecl
                | DecisionDecl
                | ChanceDecl
                | TerminalDecl
                | LinkDecl
                | InstanceDecl
                | RuleDecl
                | DeadlineDecl
                | CountingDecl
                | CalendarDecl
                | TransitionDecl
                | AuthorityDecl
                | DefaultDecl
                | ExceptionDecl
                | TestDecl
                | GrantDecl
        )
        .then_some(Self(node))
    }

    /// The item's syntax node; cast it to the concrete wrapper by kind.
    #[must_use]
    pub fn syntax(&self) -> &Node {
        &self.0
    }

    /// The item's node kind.
    #[must_use]
    pub fn kind(&self) -> SyntaxKind {
        self.0.kind()
    }
}

impl LitFile {
    /// `module` declaration.
    #[must_use]
    pub fn module(&self) -> Option<ModuleDecl> {
        child(self.syntax())
    }

    /// `import` declarations in source order.
    #[must_use]
    pub fn imports(&self) -> Vec<ImportDecl> {
        children(self.syntax())
    }

    /// `package` declaration, if present.
    #[must_use]
    pub fn package(&self) -> Option<PackageDecl> {
        child(self.syntax())
    }

    /// Items after the header, in source order.
    #[must_use]
    pub fn items(&self) -> Vec<Item> {
        self.syntax().children().filter_map(Item::cast).collect()
    }
}

impl ModuleDecl {
    /// Dotted module path.
    #[must_use]
    pub fn path(&self) -> Option<Ref> {
        child(self.syntax())
    }
}

impl ImportDecl {
    /// Imported module path.
    #[must_use]
    pub fn path(&self) -> Option<Ref> {
        child(self.syntax())
    }

    /// Names of `import m.{x, y}`; empty for a whole-module import.
    #[must_use]
    pub fn names(&self) -> Vec<Token> {
        child::<ImportList>(self.syntax()).map_or_else(Vec::new, |list| {
            tokens(list.syntax())
                .filter(|token| token.kind() == SyntaxKind::Ident)
                .collect()
        })
    }
}

impl PackageDecl {
    /// `forum` and `authority` fields.
    #[must_use]
    pub fn fields(&self) -> Vec<Field> {
        fields_of(self.syntax())
    }
}

impl StartDecl {
    /// The start node (qualified in a case).
    #[must_use]
    pub fn target(&self) -> Option<Ref> {
        child(self.syntax())
    }
}

impl GroupDecl {
    /// The quoted label token.
    #[must_use]
    pub fn label(&self) -> Option<Token> {
        token(self.syntax(), SyntaxKind::String)
    }
}

impl TypeDecl {
    /// The aliased type.
    #[must_use]
    pub fn ty(&self) -> Option<TypeExpr> {
        child(self.syntax())
    }
}

impl TypeExpr {
    /// Type arguments of `Name<A, B>`.
    #[must_use]
    pub fn args(&self) -> Vec<TypeExpr> {
        self.syntax()
            .children()
            .find(|node| node.kind() == SyntaxKind::TypeArgs)
            .map_or_else(Vec::new, |args| children(&args))
    }
}

impl EnumDecl {
    /// Variant names in order.
    #[must_use]
    pub fn variants(&self) -> Vec<Token> {
        tokens(self.syntax())
            .filter(|token| token.kind() == SyntaxKind::Ident)
            .skip(1)
            .collect()
    }
}

impl EventDecl {
    /// Declared parameters.
    #[must_use]
    pub fn params(&self) -> Vec<Param> {
        self.syntax()
            .children()
            .find(|node| node.kind() == SyntaxKind::ParamList)
            .map_or_else(Vec::new, |list| children(&list))
    }
}

impl Param {
    /// Parameter type.
    #[must_use]
    pub fn ty(&self) -> Option<TypeExpr> {
        child(self.syntax())
    }
}

impl AuthorityDecl {
    /// Authority levels from highest to lowest.
    #[must_use]
    pub fn levels(&self) -> Vec<Token> {
        tokens(self.syntax())
            .filter(|token| token.kind() == SyntaxKind::Ident)
            .collect()
    }
}

/// Clause accessors shared by declarations that accept clauses.
macro_rules! clause_accessors {
    ($($ty:ty),+) => {$(
        impl $ty {
            /// Citation clauses.
            #[must_use]
            pub fn cites(&self) -> Vec<Cite> {
                cites_of(self.syntax())
            }

            /// `effective` / `known_at` clauses.
            #[must_use]
            pub fn version_clauses(&self) -> Vec<VersionClause> {
                children(self.syntax())
            }

            /// Precedence relations.
            #[must_use]
            pub fn relations(&self) -> Vec<Relation> {
                children(self.syntax())
            }
        }
    )+};
}
clause_accessors!(
    NodeDecl,
    super::RuleDecl,
    super::DeadlineDecl,
    super::CountingDecl,
    super::CalendarDecl,
    super::TransitionDecl,
    super::DefaultDecl,
    super::ExceptionDecl,
    super::TestDecl,
    super::GrantDecl
);

impl NodeDecl {
    /// `state`, `decision`, `chance`, or `terminal` keyword token.
    #[must_use]
    pub fn keyword(&self) -> Option<Token> {
        keyword(self.syntax())
    }

    /// The quoted label token.
    #[must_use]
    pub fn label(&self) -> Option<Token> {
        token(self.syntax(), SyntaxKind::String)
    }

    /// Header clause fields (`by`, `discretion`, `info`, `outcome`).
    #[must_use]
    pub fn clause_fields(&self) -> Vec<Field> {
        children(self.syntax())
    }

    /// `by` actor.
    #[must_use]
    pub fn by(&self) -> Option<Ref> {
        clause(&self.clause_fields(), SyntaxKind::KwBy).and_then(|field| field.value_ref())
    }

    /// True when `discretion` is present.
    #[must_use]
    pub fn discretion(&self) -> bool {
        clause(&self.clause_fields(), SyntaxKind::KwDiscretion).is_some()
    }

    /// `outcome [...]` list.
    #[must_use]
    pub fn outcome(&self) -> Option<IdList> {
        clause(&self.clause_fields(), SyntaxKind::KwOutcome).and_then(|field| field.id_list())
    }

    /// The braced body.
    #[must_use]
    pub fn body(&self) -> Option<EdgeBlock> {
        child(self.syntax())
    }

    /// Body fields (`group`, `note`, `tags`, ...).
    #[must_use]
    pub fn fields(&self) -> Vec<Field> {
        self.body()
            .map_or_else(Vec::new, |body| children(body.syntax()))
    }

    /// Outgoing edges in order.
    #[must_use]
    pub fn edges(&self) -> Vec<EdgeDecl> {
        self.body()
            .map_or_else(Vec::new, |body| children(body.syntax()))
    }
}

fn clause(fields: &[Field], kw: SyntaxKind) -> Option<Field> {
    fields
        .iter()
        .find(|field| field.keyword_kind() == Some(kw))
        .cloned()
}

/// Edge accessors shared by `EdgeDecl` and `LinkDecl`.
macro_rules! edge_accessors {
    ($($ty:ty),+) => {$(
        impl $ty {
            /// Edge id; `None` for an anonymous `_` edge.
            #[must_use]
            pub fn id(&self) -> Option<Token> {
                tokens(self.syntax())
                    .find(|t| matches!(t.kind(), SyntaxKind::Ident | SyntaxKind::Underscore))
                    .filter(|t| t.kind() == SyntaxKind::Ident)
            }

            /// The quoted label token.
            #[must_use]
            pub fn label(&self) -> Option<Token> {
                token(self.syntax(), SyntaxKind::String)
            }

            /// Edge clauses `by` and `p`.
            #[must_use]
            pub fn clause_fields(&self) -> Vec<Field> {
                children(self.syntax())
            }

            /// Edge-level `by` actor.
            #[must_use]
            pub fn by(&self) -> Option<Ref> {
                clause(&self.clause_fields(), SyntaxKind::KwBy).and_then(|f| f.value_ref())
            }

            /// `p` value (`Literal`, `Ratio`, hole, or parenthesized expression).
            #[must_use]
            pub fn probability(&self) -> Option<Expr> {
                clause(&self.clause_fields(), SyntaxKind::KwP).and_then(|f| f.value_expr())
            }

            /// Braced edge fields.
            #[must_use]
            pub fn fields(&self) -> Vec<Field> {
                child::<FieldList>(self.syntax())
                    .map_or_else(Vec::new, |list| children(list.syntax()))
            }
        }
    )+};
}
edge_accessors!(EdgeDecl, LinkDecl);

impl EdgeDecl {
    /// Target node.
    #[must_use]
    pub fn target(&self) -> Option<Ref> {
        child(self.syntax())
    }
}

impl LinkDecl {
    /// Source node (`pack::node`).
    #[must_use]
    pub fn from(&self) -> Option<Ref> {
        child(self.syntax())
    }

    /// Target node (`pack::node`).
    #[must_use]
    pub fn to(&self) -> Option<Ref> {
        child_after(self.syntax(), SyntaxKind::Arrow)
    }
}

impl Field {
    /// The field keyword token.
    #[must_use]
    pub fn keyword(&self) -> Option<Token> {
        keyword(self.syntax())
    }

    /// The field keyword kind.
    #[must_use]
    pub fn keyword_kind(&self) -> Option<SyntaxKind> {
        self.keyword().map(|token| token.kind())
    }

    /// The first value node (a `Ref`, `Literal`, `IdList`, ...).
    #[must_use]
    pub fn value(&self) -> Option<Node> {
        self.syntax().children().next()
    }

    /// The value as a reference.
    #[must_use]
    pub fn value_ref(&self) -> Option<Ref> {
        child(self.syntax())
    }

    /// The value as an expression (literal, hole, ratio, call, ...).
    #[must_use]
    pub fn value_expr(&self) -> Option<Expr> {
        child(self.syntax())
    }

    /// The value as an id list.
    #[must_use]
    pub fn id_list(&self) -> Option<IdList> {
        child(self.syntax())
    }

    /// The value as a record (`attrs`, `probabilities`, ...).
    #[must_use]
    pub fn record(&self) -> Option<RecordExpr> {
        child(self.syntax())
    }

    /// Nested braced options (`within d { ... }`, `duration { ... }`).
    #[must_use]
    pub fn options(&self) -> Vec<Field> {
        child::<FieldList>(self.syntax()).map_or_else(Vec::new, |list| children(list.syntax()))
    }

    /// A trailing citation (`authority cite ...`, `service_extension {...} cite ...`).
    #[must_use]
    pub fn cite(&self) -> Option<Cite> {
        child(self.syntax())
    }
}

impl IdList {
    /// Listed references.
    #[must_use]
    pub fn items(&self) -> Vec<Ref> {
        children(self.syntax())
    }

    /// Listed string entries (a `replaces` list naming anonymous edges by
    /// derived id), decoded.
    #[must_use]
    pub fn strings(&self) -> Vec<String> {
        children::<Literal>(self.syntax())
            .iter()
            .filter_map(Literal::string_value)
            .collect()
    }
}

impl VersionClause {
    /// `effective` or `known_at` keyword.
    #[must_use]
    pub fn keyword(&self) -> Option<Token> {
        keyword(self.syntax())
    }

    /// Interval of an `effective` clause.
    #[must_use]
    pub fn interval(&self) -> Option<Interval> {
        child(self.syntax())
    }

    /// Date of a `known_at` clause.
    #[must_use]
    pub fn date(&self) -> Option<Token> {
        child::<super::Literal>(self.syntax()).and_then(|literal| literal.token())
    }
}

impl Interval {
    /// Inclusive start date.
    #[must_use]
    pub fn from(&self) -> Option<Token> {
        token(self.syntax(), SyntaxKind::Date)
    }

    /// Exclusive end date; `None` for an open `..` end.
    #[must_use]
    pub fn to(&self) -> Option<Token> {
        tokens(self.syntax())
            .filter(|token| token.kind() == SyntaxKind::Date)
            .nth(1)
    }
}

impl Relation {
    /// Relation keyword (`subject_to`, `delegable`, ...).
    #[must_use]
    pub fn keyword(&self) -> Option<Token> {
        keyword(self.syntax())
    }

    /// Target of a ref-valued relation.
    #[must_use]
    pub fn target(&self) -> Option<Ref> {
        child(self.syntax())
    }

    /// Authorities of the legacy `delegable_to [...]`.
    #[must_use]
    pub fn delegates(&self) -> Option<IdList> {
        child(self.syntax())
    }

    /// Fields of a `delegable { to L, op O, ... }` capability.
    #[must_use]
    pub fn delegation(&self) -> Option<FieldList> {
        child(self.syntax())
    }
}
