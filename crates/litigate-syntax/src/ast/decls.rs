//! Accessors for rule-bearing declarations.
use super::{
    child, child_after, children, fields_of, keyword, token, AstNode, CalendarDecl, CallExpr, Cite,
    CountingDecl, DeadlineDecl, DefaultDecl, ExceptionDecl, ExpectStmt, Expr, Field, GivenStmt,
    GrantDecl, Hole, HolidayDecl, InstanceDecl, InvalidateDecl, Literal, PatchDecl,
    RecharacterizeDecl, Ref, RuleDecl, TestDecl, Token, TransitionArm, TransitionDecl,
    VersionClause,
};
use crate::ast::items::Item;
use crate::syntax_kind::SyntaxKind;

impl RuleDecl {
    /// `duty`, `power`, or `liability`.
    #[must_use]
    pub fn rule_kind(&self) -> Option<Token> {
        super::tokens(self.syntax()).find(|token| {
            matches!(
                token.kind(),
                SyntaxKind::KwDuty | SyntaxKind::KwPower | SyntaxKind::KwLiability
            )
        })
    }

    /// Items declared in the body (hoisted into module scope by sema).
    #[must_use]
    pub fn items(&self) -> Vec<Item> {
        self.syntax().children().filter_map(Item::cast).collect()
    }
}

impl DeadlineDecl {
    /// The period literal (`21.days`).
    #[must_use]
    pub fn period(&self) -> Option<Token> {
        child::<Literal>(self.syntax()).and_then(|literal| literal.token())
    }

    /// `from`, `after`, or `before`.
    #[must_use]
    pub fn direction(&self) -> Option<Token> {
        super::tokens(self.syntax()).find(|token| {
            matches!(
                token.kind(),
                SyntaxKind::KwFrom | SyntaxKind::KwAfter | SyntaxKind::KwBefore
            )
        })
    }

    /// The trigger event pattern.
    #[must_use]
    pub fn trigger(&self) -> Option<Expr> {
        self.syntax().children().filter_map(Expr::cast).nth(1)
    }

    /// The counting rule.
    #[must_use]
    pub fn counted_by(&self) -> Option<Ref> {
        child_after(self.syntax(), SyntaxKind::KwCountedBy)
    }

    /// Deadline options (`extendable`, `extension_authority`, `note`).
    #[must_use]
    pub fn options(&self) -> Vec<Field> {
        fields_of(self.syntax())
    }
}

impl CountingDecl {
    /// Counting items (`exclude_trigger_day`, `holidays`, ...).
    #[must_use]
    pub fn items(&self) -> Vec<Field> {
        fields_of(self.syntax())
    }
}

impl CalendarDecl {
    /// Holidays in order.
    #[must_use]
    pub fn holidays(&self) -> Vec<HolidayDecl> {
        children(self.syntax())
    }
}

impl HolidayDecl {
    /// Date rule call (`fixed(1, 1)`, `nth(3, monday, 1)`, ...).
    #[must_use]
    pub fn rule(&self) -> Option<CallExpr> {
        child(self.syntax())
    }

    /// The `effective` interval clause.
    #[must_use]
    pub fn effective(&self) -> Option<VersionClause> {
        child(self.syntax())
    }

    /// `observed` policy.
    #[must_use]
    pub fn observed(&self) -> Option<Ref> {
        child::<Field>(self.syntax()).and_then(|field| field.value_ref())
    }
}

impl GrantDecl {
    /// The deadlines it reaches (`on [d, ...]`).
    #[must_use]
    pub fn targets(&self) -> Option<super::IdList> {
        child::<Field>(self.syntax()).and_then(|field| field.id_list())
    }

    /// Body fields (`unlock`, `extend`, `waive`, `delegable`).
    #[must_use]
    pub fn fields(&self) -> Vec<Field> {
        fields_of(self.syntax())
    }
}

impl RecharacterizeDecl {
    /// The deadline it recharacterizes.
    #[must_use]
    pub fn target(&self) -> Option<Ref> {
        child(self.syntax())
    }

    /// The new characterization (`jurisdictional` or `claim_processing`).
    #[must_use]
    pub fn characterization(&self) -> Option<Ref> {
        child::<Field>(self.syntax()).and_then(|field| field.value_ref())
    }
}

impl InvalidateDecl {
    /// The declaration it invalidates.
    #[must_use]
    pub fn target(&self) -> Option<Ref> {
        child(self.syntax())
    }
}

impl TransitionDecl {
    /// The amended declaration version (`d@2009-12-01`).
    #[must_use]
    pub fn amends(&self) -> Option<Ref> {
        child(self.syntax())
    }

    /// Arms in order.
    #[must_use]
    pub fn arms(&self) -> Vec<TransitionArm> {
        children(self.syntax())
    }
}

impl TransitionArm {
    /// True for the `otherwise` arm.
    #[must_use]
    pub fn is_otherwise(&self) -> bool {
        keyword(self.syntax()).is_some_and(|token| token.kind() == SyntaxKind::KwOtherwise)
    }

    /// Condition of an `unless` arm.
    #[must_use]
    pub fn condition(&self) -> Option<Expr> {
        child(self.syntax())
    }

    /// Selected version.
    #[must_use]
    pub fn target(&self) -> Option<Ref> {
        child_after(self.syntax(), SyntaxKind::Arrow)
    }
}

impl DefaultDecl {
    /// Condition after `when`.
    #[must_use]
    pub fn condition(&self) -> Option<Expr> {
        child_after(self.syntax(), SyntaxKind::KwWhen)
    }

    /// Concluded value after `=>`.
    #[must_use]
    pub fn value(&self) -> Option<Expr> {
        child_after(self.syntax(), SyntaxKind::FatArrow)
    }
}

impl ExceptionDecl {
    /// `rebuts` or `undercuts`.
    #[must_use]
    pub fn exception_kind(&self) -> Option<Token> {
        super::tokens(self.syntax())
            .find(|token| matches!(token.kind(), SyntaxKind::KwRebuts | SyntaxKind::KwUndercuts))
    }

    /// The default (or other target) it applies to.
    #[must_use]
    pub fn target(&self) -> Option<Ref> {
        child(self.syntax())
    }

    /// Condition after `when`.
    #[must_use]
    pub fn condition(&self) -> Option<Expr> {
        child_after(self.syntax(), SyntaxKind::KwWhen)
    }

    /// Concluded value after `=>` (rebutters only).
    #[must_use]
    pub fn value(&self) -> Option<Expr> {
        child_after(self.syntax(), SyntaxKind::FatArrow)
    }

    /// `priority N` literal.
    #[must_use]
    pub fn priority(&self) -> Option<Token> {
        child::<Field>(self.syntax())
            .and_then(|field| field.value_expr())
            .and_then(|expr| Literal::cast(expr.syntax().clone()))
            .and_then(|literal| literal.token())
    }
}

impl TestDecl {
    /// The quoted test name.
    #[must_use]
    pub fn title(&self) -> Option<Token> {
        token(self.syntax(), SyntaxKind::String)
    }

    /// `given` statements.
    #[must_use]
    pub fn givens(&self) -> Vec<GivenStmt> {
        children(self.syntax())
    }

    /// `expect` statements.
    #[must_use]
    pub fn expects(&self) -> Vec<ExpectStmt> {
        children(self.syntax())
    }
}

impl GivenStmt {
    /// The event expression.
    #[must_use]
    pub fn event(&self) -> Option<Expr> {
        child(self.syntax())
    }

    /// The event date.
    #[must_use]
    pub fn date(&self) -> Option<Token> {
        child_after::<Literal>(self.syntax(), SyntaxKind::KwOn).and_then(|l| l.token())
    }
}

impl ExpectStmt {
    /// The asserted expression.
    #[must_use]
    pub fn expr(&self) -> Option<Expr> {
        child(self.syntax())
    }
}

impl InstanceDecl {
    /// Base pack id (before `@`).
    #[must_use]
    pub fn base(&self) -> Option<Token> {
        self.name()
    }

    /// Origin pack id (after `@`).
    #[must_use]
    pub fn origin(&self) -> Option<Token> {
        super::tokens(self.syntax())
            .filter(|token| token.kind() == SyntaxKind::Ident)
            .nth(1)
    }

    /// Instance fields (`pack`, `note`, `probabilities`, ...).
    #[must_use]
    pub fn fields(&self) -> Vec<Field> {
        fields_of(self.syntax())
    }

    /// `patch` entries.
    #[must_use]
    pub fn patches(&self) -> Vec<PatchDecl> {
        child::<super::FieldList>(self.syntax())
            .map_or_else(Vec::new, |list| children(list.syntax()))
    }
}

impl PatchDecl {
    /// The patched edge id.
    #[must_use]
    pub fn edge(&self) -> Option<Token> {
        self.name()
    }

    /// The typed edge fields, each replacing the base edge's.
    #[must_use]
    pub fn fields(&self) -> Vec<Field> {
        fields_of(self.syntax())
    }
}

impl Cite {
    /// The citation string's value, escapes decoded (SPEC §2).
    #[must_use]
    pub fn text(&self) -> Option<String> {
        token(self.syntax(), SyntaxKind::String).map(|token| crate::ast::unescape(token.text()))
    }

    /// True for `cite "..." historical`: a deliberate cite of an
    /// expression of the work dated outside the declaration's window.
    #[must_use]
    pub fn historical(&self) -> bool {
        token(self.syntax(), SyntaxKind::KwHistorical).is_some()
    }

    /// The hole of a `cite ?("reason")` clause, if this is one.
    #[must_use]
    pub fn hole(&self) -> Option<Hole> {
        child(self.syntax())
    }
}
