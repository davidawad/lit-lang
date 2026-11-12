//! Deadlines, counting rules, holiday calendars, and transitions lowered to
//! HIR (SPEC §5.3-§5.4). Structural checks only; overlap, coverage, and
//! counting semantics are verified downstream.
use super::cx::{first_expr, first_ref, flag, Cx};
use crate::db::parse::tree::{ArmItem, Atom, Expr, ExprKind, Field, Item, Name, RefPath};
use crate::db::span::Report;
use crate::hir::{
    Adoption, Anchor, Chain, CharacterKind, Counting, DayUnit, Deadline, Derivation, Designated,
    Direction, DueEnd, EndOfMonth, Rational, RollOn, ServiceTime, ShortPeriod, Transition,
    TransitionArm, Type, Value, SERVICE_METHODS,
};
use crate::resolve::sig::{clauses_of, TypeSig, ValueKind};
use crate::resolve::suggest;
use litigate_base::{Applicability, DiagnosticCode as C};
use std::collections::BTreeMap;
include!("clock/basic.rs");
include!("clock/deadlines.rs");
include!("clock/transitions.rs");
include!("clock/helpers.rs");
