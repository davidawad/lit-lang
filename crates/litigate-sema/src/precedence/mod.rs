//! Precedence, delegation, and defeasibility (SPEC §5.5, DESIGN §4.5): the
//! authority lattice, two-stage conflict resolution, `override` /
//! `delegable_to`, rebutting and undercutting exceptions, rule kinds, and
//! direction-dependent deadline typing. [`select`] is the selection query
//! every consumer (lowering, replay, tests) asks which declaration governs.
//!
//! [`query::legal_checks`] runs these and the [`crate::temporal`] checks
//! for one package; `check_package` folds its diagnostics in.
pub mod conflict;
pub mod cycles;
pub mod defeasible;
pub mod delegation;
pub mod lattice;
pub mod query;
pub mod select;
pub mod shape;

pub use conflict::{beats, winner, Reason};
pub use defeasible::{conclude, Conclusion};
pub use lattice::{Lattice, Order};
pub use select::{select, Unselected};

use crate::temporal::world::{Locate, World};
use litigate_base::Diagnostic;

/// Every temporal and precedence diagnostic for pack `pack` of `world`.
#[must_use]
pub fn check(world: &World<'_>, loc: &dyn Locate, pack: usize) -> Vec<Diagnostic> {
    let lattice = Lattice::new(&world.ranks);
    let mut out = crate::temporal::check::check(world, loc, pack);
    out.extend(crate::temporal::transition::check(world, loc, pack));
    out.extend(lattice.check(world, loc, pack));
    out.extend(cycles::check(world, loc, pack));
    out.extend(conflict::check(world, &lattice, loc, pack));
    out.extend(delegation::check(world, &lattice, loc, pack));
    out.extend(defeasible::check(world, loc, pack));
    out.extend(shape::check(world, loc, pack));
    out.extend(crate::guard::check(world, pack));
    out
}
