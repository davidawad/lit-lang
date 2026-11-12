//! Deadline computation through litgraph's own clock interpreter, over the
//! counting rules the loaded packages lower to (each version lowered at
//! its own `from` date), else litgraph's shared clock library. One code
//! path serves `test`, case replay, and `explain-value`, so they cannot
//! disagree with the engine.
use crate::hirx;
use litigate_engine::clock::{
    compute_due_date, ClockConfig, ClockLibrary, ComputedDeadline, Date, DayDeadline, Direction,
    ServiceMethod, Unit,
};
use litigate_lower::Lowerer;
use litigate_sema::hir::{self, DayUnit, Declaration, DueEnd, Expr};
use litigate_sema::precedence::select::unordered;
use litigate_sema::temporal::{self, Inactive, Key, MatterEvent, Miss, Ungoverned, World};
use std::borrow::Cow;
use std::collections::BTreeMap;

/// Every version of every declaration the loaded packages declare, and
/// the clock fragment of each counting version. Declarations are selected
/// by sema's one query (`temporal::governing`: precedence over the
/// versions `temporal::select` finds), in the world of the packages asking
/// ([`Clocks::scope`]), as lowering and case replay select them.
#[derive(Debug, Default)]
pub struct Clocks<'a> {
    packages: Vec<&'a hir::Package>,
    /// Each counting version's fragment (by declaration address, which
    /// every scope shares), or why it does not lower.
    fragments: BTreeMap<usize, Cow<'a, Result<ClockLibrary, String>>>,
}
include!("clock/fragments.rs");
include!("clock/due.rs");
#[cfg(test)]
include!("clock/tests.rs");
