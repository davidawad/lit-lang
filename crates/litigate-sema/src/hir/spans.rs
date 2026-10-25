//! Every source [`Span`] of a declaration, visited mutably: how a module
//! locates the HIR its per-declaration queries built with position-free
//! spans ([`crate::db::span::Encoded`]).
//!
//! Each struct is destructured without `..`, so a field added to HIR is a
//! compile error here until it is visited or explicitly skipped.
use super::{
    Calendar, Counting, Declaration, DefaultRule, Edge, EdgePatch, Exception, Expectation, Expr,
    ExprKind, Extension, Instance, Link, Metadata, Motion, Node, Rule, Test, Transition,
    TransitionArm,
};
use litigate_base::Span;

/// Calls `f` on every span of a HIR value.
pub trait Spans {
    /// Visits every span, in no particular order.
    fn spans_mut(&mut self, f: &mut dyn FnMut(&mut Span));
}

impl Spans for Span {
    fn spans_mut(&mut self, f: &mut dyn FnMut(&mut Span)) {
        f(self);
    }
}

impl<T: Spans> Spans for Option<T> {
    fn spans_mut(&mut self, f: &mut dyn FnMut(&mut Span)) {
        if let Some(x) = self {
            x.spans_mut(f);
        }
    }
}

impl<T: Spans> Spans for Vec<T> {
    fn spans_mut(&mut self, f: &mut dyn FnMut(&mut Span)) {
        for x in self {
            x.spans_mut(f);
        }
    }
}

impl Spans for Metadata {
    fn spans_mut(&mut self, f: &mut dyn FnMut(&mut Span)) {
        let Self {
            id: _,
            label: _,
            cite: _,
            historical: _,
            effective: _,
            authority: _,
            relations: _,
            span,
            imported: _,
        } = self;
        span.spans_mut(f);
    }
}

impl Spans for Expr {
    fn spans_mut(&mut self, f: &mut dyn FnMut(&mut Span)) {
        let Self { ty: _, kind, span } = self;
        span.spans_mut(f);
        match kind {
            ExprKind::Literal { .. } | ExprKind::Ref { .. } => {}
            ExprKind::Call { args, .. } => args.spans_mut(f),
            ExprKind::Unary { operand, .. } => operand.spans_mut(f),
            ExprKind::Binary { lhs, rhs, .. } => {
                lhs.spans_mut(f);
                rhs.spans_mut(f);
            }
            ExprKind::Record { fields } => {
                for (_, e) in fields {
                    e.spans_mut(f);
                }
            }
            ExprKind::List { items } => items.spans_mut(f),
            ExprKind::Field { base, .. } => base.spans_mut(f),
            ExprKind::Match { scrutinee, arms } => {
                scrutinee.spans_mut(f);
                for (_, e) in arms {
                    e.spans_mut(f);
                }
            }
        }
    }
}

impl Spans for Edge {
    fn spans_mut(&mut self, f: &mut dyn FnMut(&mut Span)) {
        let Self {
            id: _,
            to: _,
            label: _,
            actor: _,
            authority: _,
            probability: _,
            hours: _,
            cost: _,
            event,
            deadline: _,
            duration: _,
            valence: _,
            court_listener_url: _,
            action_id: _,
            note: _,
            tags: _,
            attrs: _,
            sets: _,
            clears: _,
            requires: _,
            forbids: _,
            guard,
            span,
            holes: _,
            ..
        } = self;
        event.spans_mut(f);
        guard.spans_mut(f);
        span.spans_mut(f);
    }
}

impl Spans for Node {
    fn spans_mut(&mut self, f: &mut dyn FnMut(&mut Span)) {
        let Self {
            meta,
            kind: _,
            group: _,
            note: _,
            valence: _,
            court_listener_url: _,
            tags: _,
            attrs: _,
            actor: _,
            info: _,
            edges,
            holes: _,
            ..
        } = self;
        meta.spans_mut(f);
        edges.spans_mut(f);
    }
}

impl Spans for Link {
    fn spans_mut(&mut self, f: &mut dyn FnMut(&mut Span)) {
        let Self {
            from: _,
            edge,
            replaces: _,
        } = self;
        edge.spans_mut(f);
    }
}

impl Spans for EdgePatch {
    fn spans_mut(&mut self, f: &mut dyn FnMut(&mut Span)) {
        let Self {
            authority: _,
            hours: _,
            cost: _,
            deadline: _,
            duration: _,
            valence: _,
            court_listener_url: _,
            action_id: _,
            note: _,
            tags: _,
            attrs: _,
            sets: _,
            clears: _,
            requires: _,
            forbids: _,
            span,
            holes: _,
            ..
        } = self;
        span.spans_mut(f);
    }
}

impl Spans for Instance {
    fn spans_mut(&mut self, f: &mut dyn FnMut(&mut Span)) {
        let Self {
            name: _,
            pack: _,
            note: _,
            remove_edges: _,
            probabilities: _,
            roles: _,
            payoff_transform,
            stake: _,
            patch_edges,
        } = self;
        payoff_transform.spans_mut(f);
        for patch in patch_edges.values_mut() {
            patch.spans_mut(f);
        }
    }
}

impl Spans for Motion {
    fn spans_mut(&mut self, f: &mut dyn FnMut(&mut Span)) {
        let Self { event, .. } = self;
        event.spans_mut(f);
    }
}

impl Spans for Extension {
    fn spans_mut(&mut self, f: &mut dyn FnMut(&mut Span)) {
        match self {
            Self::Bounded { motion, .. } | Self::Discretionary { motion, .. } => {
                motion.spans_mut(f);
            }
            Self::NonExtendable { .. } => {}
        }
    }
}

impl Spans for super::Deadline {
    fn spans_mut(&mut self, f: &mut dyn FnMut(&mut Span)) {
        let Self {
            meta,
            amount: _,
            unit: _,
            direction: _,
            trigger,
            chain: _,
            counting: _,
            forum: _,
            extendable: _,
            extension_authority: _,
            note: _,
            bearer: _,
            policy: _,
            linear: _,
            extend,
            waive: _,
            designated: _,
            ..
        } = self;
        meta.spans_mut(f);
        trigger.spans_mut(f);
        extend.spans_mut(f);
    }
}

impl Spans for Counting {
    fn spans_mut(&mut self, f: &mut dyn FnMut(&mut Span)) {
        let Self {
            meta,
            exclude_trigger_day: _,
            include_last_day: _,
            roll_on: _,
            service_extensions: _,
            calendar: _,
            forward_calendar: _,
            declared_calendar: _,
            forums: _,
            anchor: _,
            short_period: _,
            hours: _,
            end_of_month: _,
        } = self;
        meta.spans_mut(f);
    }
}

impl Spans for Calendar {
    fn spans_mut(&mut self, f: &mut dyn FnMut(&mut Span)) {
        let Self { meta, holidays: _ } = self;
        meta.spans_mut(f);
    }
}

impl Spans for Rule {
    fn spans_mut(&mut self, f: &mut dyn FnMut(&mut Span)) {
        let Self {
            meta,
            rule_kind: _,
            declarations: _,
        } = self;
        meta.spans_mut(f);
    }
}

impl Spans for DefaultRule {
    fn spans_mut(&mut self, f: &mut dyn FnMut(&mut Span)) {
        let Self {
            meta,
            condition,
            conclusion,
        } = self;
        meta.spans_mut(f);
        condition.spans_mut(f);
        conclusion.spans_mut(f);
    }
}

impl Spans for Exception {
    fn spans_mut(&mut self, f: &mut dyn FnMut(&mut Span)) {
        let Self {
            meta,
            target: _,
            kind: _,
            condition,
            conclusion,
            priority: _,
        } = self;
        meta.spans_mut(f);
        condition.spans_mut(f);
        conclusion.spans_mut(f);
    }
}

impl Spans for TransitionArm {
    fn spans_mut(&mut self, f: &mut dyn FnMut(&mut Span)) {
        let Self { unless, selects: _ } = self;
        unless.spans_mut(f);
    }
}

impl Spans for Transition {
    fn spans_mut(&mut self, f: &mut dyn FnMut(&mut Span)) {
        let Self {
            meta,
            amendment: _,
            arms,
        } = self;
        meta.spans_mut(f);
        arms.spans_mut(f);
    }
}

impl Spans for Expectation {
    fn spans_mut(&mut self, f: &mut dyn FnMut(&mut Span)) {
        let Self { lhs, op: _, rhs } = self;
        lhs.spans_mut(f);
        rhs.spans_mut(f);
    }
}

impl Spans for Test {
    fn spans_mut(&mut self, f: &mut dyn FnMut(&mut Span)) {
        let Self {
            name: _,
            cite: _,
            givens,
            expects,
            span,
        } = self;
        for (e, _) in givens {
            e.spans_mut(f);
        }
        expects.spans_mut(f);
        span.spans_mut(f);
    }
}

impl Spans for Declaration {
    fn spans_mut(&mut self, f: &mut dyn FnMut(&mut Span)) {
        match self {
            Self::Node(x) => x.spans_mut(f),
            Self::Link(x) => x.spans_mut(f),
            Self::Instance(x) => x.spans_mut(f),
            Self::Deadline(x) => x.spans_mut(f),
            Self::Counting(x) => x.spans_mut(f),
            Self::Calendar(x) => x.spans_mut(f),
            Self::Rule(x) => x.spans_mut(f),
            Self::Default(x) => x.spans_mut(f),
            Self::Exception(x) => x.spans_mut(f),
            Self::Transition(x) => x.spans_mut(f),
            Self::Test(x) => x.spans_mut(f),
            Self::Authority(_) | Self::TypeAlias(_) | Self::Enum(_) | Self::Event(_) => {}
        }
    }
}
