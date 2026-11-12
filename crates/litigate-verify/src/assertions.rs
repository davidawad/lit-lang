//! `tests-pass` (SPEC §7): every inline `test` assertion holds.
//!
//! `deadline(d)` is computed by litgraph's own clock interpreter
//! (`clock::compute_due_date`) running the counting rule exactly as
//! lowering emits it for the engine (SPEC §8.2), from the latest `given`
//! event matching `d`'s trigger, with that event's service method, under
//! the versions of `d` and of its counting rule in force on the trigger
//! date (the rule lowered at its own `from`, as `litigate test` does), so
//! a test can never pass here and the engine compute a different date.
//! Both are selected by sema's one query (`precedence::select` over the
//! versions `temporal::select` finds: a version, or a claimant displacing
//! it) in the law the package sees, among those recorded by the build's
//! `known_at`, as lowering selects them; a counting rule no loaded
//! package declares is the shared
//! library's. A later `given` whose type `extends d` moves the adjusted
//! date as replay would (sema's `case::grant::extensions`): licensed by
//! one of the selected version's grants held by the actor it names
//! (`given e by court on date`). `has_event(p)` is true when a `given`
//! matches `p`. An assertion this evaluator cannot compute fails: an
//! unevaluated test is not a passing one.
use crate::diag::{emit, note};
use crate::{same_expr, Context};
use litigate_base::{Diagnostic, DiagnosticCode as C};
use litigate_engine::clock::{self, ClockConfig, ClockLibrary, DayDeadline, Unit};
use litigate_lower::Lowerer;
use litigate_sema::case::grant;
use litigate_sema::hir::{self, BinaryOp, Declaration, Expr, ExprKind, Value};
use litigate_sema::precedence::select::select;
use litigate_sema::temporal::{Key, World};
use std::cell::RefCell;
use std::collections::BTreeMap;

include!("assertions/counting.rs");
include!("assertions/expect.rs");
include!("assertions/clock.rs");
include!("assertions/tests.rs");
