//! The indexed rule type `Rule[a, s, v, k, ap]` (SPEC §3.1, design review
//! §2.3, §6 item 20) as surface sugar: `v`, `k` and `ap` stand for the
//! rule's `effective`, `known_at` and `commenced` clauses (lowering writes
//! them), and `a` and `s` state the authority and the instrument class the
//! rule's declarations already have from their package, checked here.
use super::cx::Cx;
use crate::db::parse::tree::{Atom, ExprKind, Field, Item};
use crate::db::span::{Report, TextRange};
use crate::hir::Kind;
use crate::resolve::sig::clauses_of;
use litigate_base::DiagnosticCode as C;

/// The indices in order, as an L2007 message names them.
const INDICES: [&str; 5] = [
    "authority",
    "scope",
    "valid",
    "transaction",
    "applicability",
];

/// The version clause each interval index stands for.
const CLAUSES: [&str; 5] = ["", "", "effective", "known_at", "commenced"];

impl Cx<'_, '_> {
    /// L2007 for each ill-formed index of rule `item`'s `Rule[...]`: not
    /// five of them, a name where an interval stands or the reverse, an
    /// authority or class other than its package's, a closed transaction
    /// interval, or an interval index also written as its clause; L1002
    /// for an authority no `authority` chain declares.
    pub(super) fn rule_index(&mut self, item: &Item) {
        let clauses = clauses_of(item);
        let Some(index) = clauses.iter().find(|f| f.key.text == "Rule") else {
            return;
        };
        if index.values.len() != INDICES.len() {
            self.report(Report::new(
                C::InvalidField,
                index.range,
                format!(
                    "a rule's index is `Rule[{}]`, five indices with `_` for a default; this one has {}",
                    INDICES.join(", "),
                    index.values.len()
                ),
            ));
            return;
        }
        for (i, (atom, range)) in index.values.iter().enumerate() {
            match (i, atom) {
                (_, Atom::Fields(_)) => {}
                (0, Atom::Expr(e)) => match &e.kind {
                    ExprKind::Ref(r) if r.is_simple() => {
                        self.authority_index(&r.head.text, r.head.clone())
                    }
                    _ => self.misplaced(i, *range),
                },
                (1, Atom::Expr(e)) => match &e.kind {
                    ExprKind::Ref(r) if r.is_simple() => self.scope_index(&r.head.text, *range),
                    _ => self.misplaced(i, *range),
                },
                (2..=4, Atom::Interval(_, to)) => {
                    if let (3, Some(to)) = (i, to) {
                        self.report(Report::new(
                            C::InvalidField,
                            to.range,
                            "a rule's transaction index is open, `[known_at, ..)`: a retraction belongs to the version it retracts, `retracted D` on that declaration",
                        ));
                    }
                    twice(self, clauses, index, CLAUSES[i]);
                }
                _ => self.misplaced(i, *range),
            }
        }
    }

    /// L1002 for an undeclared authority, L2007 for one other than the
    /// package's.
    fn authority_index(&mut self, level: &str, name: crate::db::parse::tree::Name) {
        if let Err(e) = self.ck.scope.authority(&name) {
            self.ck.report(e);
            return;
        }
        let own = self
            .manifest
            .map(|m| m.authority.as_str())
            .unwrap_or_default();
        if level != own {
            self.report(Report::new(
                C::InvalidField,
                name.range,
                format!(
                    "the authority index `{level}` is not this rule's authority `{own}`: a rule's declarations have their package's (`authority` in Lit.toml)"
                ),
            ));
        }
    }

    /// L2007 for a class name that is not the package's `kind`.
    fn scope_index(&mut self, class: &str, range: TextRange) {
        let own = self.manifest.map_or("base", |m| m.kind.name());
        if Kind::from_name(class).is_none() {
            self.report(Report::new(
                C::InvalidField,
                range,
                format!(
                    "the scope index `{class}` is not an instrument class: expected {}",
                    Kind::NAMES.map(|n| format!("`{n}`")).join(", ")
                ),
            ));
        } else if class != own {
            self.report(Report::new(
                C::InvalidField,
                range,
                format!(
                    "the scope index `{class}` is not this rule's class `{own}`: a rule's declarations have their package's (`kind` in Lit.toml)"
                ),
            ));
        }
    }

    /// L2007 for index `i` of the wrong shape.
    fn misplaced(&mut self, i: usize, range: TextRange) {
        let want = if i < 2 {
            "a name"
        } else {
            "an interval `[from, to)`"
        };
        self.report(Report::new(
            C::InvalidField,
            range,
            format!("the {} index is {want}, or `_` for its default", INDICES[i]),
        ));
    }
}

/// L2007 for clause `key` written beside the index standing for it.
fn twice(cx: &mut Cx<'_, '_>, clauses: &[Field], index: &Field, key: &str) {
    let inside = |r: TextRange| r.start >= index.range.start && r.end <= index.range.end;
    for f in clauses
        .iter()
        .filter(|f| f.key.text == key && !inside(f.key.range))
    {
        cx.report(Report::new(
            C::InvalidField,
            f.key.range,
            format!("`{key}` is given twice: by the rule's `Rule[...]` index and as a clause"),
        ));
    }
}
