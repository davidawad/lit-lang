// SPDX-License-Identifier: GPL-3.0-or-later
//! The deadline clock: a data-driven interpreter for deadline-counting
//! rules. The engine holds no forum knowledge here: every counting rule
//! (anchor, count unit, short-period rule, last-day roll, service-method
//! extensions, clerk-inaccessibility and hours provisions, and the cited
//! text of each step) and every holiday calendar (fixed and nth-weekday
//! holidays, weekend observance, multi-year cycles, effective intervals)
//! is authored in `.lit` (`counting` and `calendar` declarations, lowered
//! to `clock/*.json` fragments: `lit/core`'s and a package's own) and
//! referenced from a pack's
//! `counting` field. See `docs/PACK_SCHEMA.md` ("Clock library") and
//! `docs/ARCHITECTURE.md` ("Deadline clock").
//!
//! No date-library dependency: [`date::Date`] is a small, exact,
//! allocation-free proleptic-Gregorian day-count type (see its module doc
//! for why this beats pulling in `time`/`chrono` here — the crate's own
//! `deny.toml`/dependency-minimalism convention, and the fact that every
//! operation needed is exact integer arithmetic with no timezone concept
//! at all, which is precisely the case a general-purpose date crate is
//! *not* simpler for).
//!
//! Layout:
//! - [`date`]: calendar arithmetic.
//! - [`calendar`]: the `HolidayCalendar` IR.
//! - [`rule`]: the `CountingRule` IR.
//! - [`library`]: loading, merging and resolving the clock library.
//! - [`interp`]: the generic interpreter.
//! - [`cycle`]: properties of every date, decided over one Gregorian cycle.

pub mod calendar;
pub mod cycle;
pub mod date;
pub mod interp;
pub mod library;
pub mod rule;
mod types;

#[cfg(kani)]
mod kani;

pub use calendar::HolidayCalendar;
pub use date::Date;
pub use interp::{compute_due_date, compute_hour_deadline, Clock};
pub use library::ClockLibrary;
pub use rule::CountingRule;
pub use types::{
    ClockConfig, ComputedDeadline, ComputedHourDeadline, DayDeadline, Direction, HourDeadline,
    ServiceMethod, Unit,
};
