//! The authority lattice: the strict partial order the `authority A > B`
//! chains declare (SPEC §5.5; `lit/core` declares
//! `Statute > FederalRule > LocalRule > StandingOrder > CaseOrder`).
use crate::temporal::world::{error, DiagnosticExt, Locate, Rank, World};
use litigate_base::{Diagnostic, DiagnosticCode as C};
use std::collections::{BTreeMap, BTreeSet};

/// How two authority levels compare.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Order {
    /// The same level.
    Equal,
    /// The first outranks the second.
    Above,
    /// The second outranks the first.
    Below,
    /// Neither outranks the other (or one is undeclared).
    Incomparable,
}

/// The transitive closure of declared ranks.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Lattice {
    below: BTreeMap<String, BTreeSet<String>>,
}

impl Lattice {
    /// The closure of `ranks`.
    #[must_use]
    pub fn new(ranks: &[Rank]) -> Self {
        let mut direct: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
        for r in ranks {
            direct.entry(&r.above).or_default().insert(&r.below);
            direct.entry(&r.below).or_default();
        }
        let below = direct
            .keys()
            .map(|&a| {
                let mut seen = BTreeSet::new();
                let mut work: Vec<&str> = direct.get(a).into_iter().flatten().copied().collect();
                while let Some(n) = work.pop() {
                    if seen.insert(n.to_owned()) {
                        work.extend(direct.get(n).into_iter().flatten().copied());
                    }
                }
                (a.to_owned(), seen)
            })
            .collect();
        Self { below }
    }

    /// True when `name` appears in some chain.
    #[must_use]
    pub fn declares(&self, name: &str) -> bool {
        self.below.contains_key(name)
    }

    /// True when `a` strictly outranks `b`.
    #[must_use]
    pub fn outranks(&self, a: &str, b: &str) -> bool {
        self.below.get(a).is_some_and(|s| s.contains(b))
    }

    /// Compares two levels.
    #[must_use]
    pub fn compare(&self, a: &str, b: &str) -> Order {
        match (a == b, self.outranks(a, b), self.outranks(b, a)) {
            (true, _, _) => Order::Equal,
            (false, true, false) => Order::Above,
            (false, false, true) => Order::Below,
            _ => Order::Incomparable,
        }
    }

    /// The `authority` declarations that close a cycle (`A > B` where B
    /// already reaches A), owned by pack `pack` (`precedence-acyclic`).
    #[must_use]
    pub fn check(&self, world: &World<'_>, loc: &dyn Locate, pack: usize) -> Vec<Diagnostic> {
        let mut seen: BTreeSet<BTreeSet<String>> = BTreeSet::new();
        world
            .ranks
            .iter()
            .filter(|r| r.pack == pack && (r.above == r.below || self.outranks(&r.below, &r.above)))
            .filter_map(|r| {
                let members: BTreeSet<String> = self
                    .below
                    .get(&r.above)
                    .into_iter()
                    .flatten()
                    .filter(|n| self.outranks(n, &r.above) || **n == r.above)
                    .cloned()
                    .chain([r.above.clone()])
                    .collect();
                seen.insert(members.clone()).then(|| {
                    let span = loc
                        .rank(r)
                        .unwrap_or_else(|| crate::temporal::world::nowhere("authority"));
                    error(
                        C::PrecedenceCycle,
                        span,
                        format!(
                            "authority `{} > {}` makes the order cyclic",
                            r.above, r.below
                        ),
                    )
                    .with_note(
                        None,
                        format!(
                            "levels on the cycle: {}",
                            members.into_iter().collect::<Vec<_>>().join(", ")
                        ),
                    )
                    .with_note(
                        None,
                        "authority is a strict partial order: no level may outrank itself",
                    )
                })
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rank(a: &str, b: &str) -> Rank {
        Rank {
            above: a.into(),
            below: b.into(),
            pack: 0,
        }
    }

    #[test]
    fn closure_orders_every_chain_member() {
        let l = Lattice::new(&[
            rank("Statute", "FederalRule"),
            rank("FederalRule", "LocalRule"),
            rank("LocalRule", "CaseOrder"),
            rank("Agency", "CaseOrder"),
        ]);
        assert_eq!(l.compare("Statute", "CaseOrder"), Order::Above);
        assert_eq!(l.compare("CaseOrder", "FederalRule"), Order::Below);
        assert_eq!(l.compare("LocalRule", "LocalRule"), Order::Equal);
        assert_eq!(l.compare("Agency", "Statute"), Order::Incomparable);
        assert_eq!(l.compare("Unclassified", "Statute"), Order::Incomparable);
        assert!(l.declares("Agency"));
        assert!(!l.declares("Unclassified"));
    }

    #[test]
    fn a_cycle_is_incomparable_everywhere_on_it() {
        let l = Lattice::new(&[rank("A", "B"), rank("B", "C"), rank("C", "A")]);
        assert!(l.outranks("A", "A"));
        assert_eq!(l.compare("A", "B"), Order::Incomparable);
    }
}
