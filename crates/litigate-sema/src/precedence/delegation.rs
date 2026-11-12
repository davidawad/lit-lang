//! `delegation-respected` (SPEC §5.5 step 3, L3004): a lower authority may
//! change a higher one's rule only by a typed capability the rule declares
//! (`delegable { to L, op O, max N.unit, discretion S, on M() }`, design
//! review §2.4) or an emergency `grant` gives it for a window. Every
//! relation that displaces its target needs one covering the operations
//! it performs: `override` and `notwithstanding` (the value it replaces
//! the target's with), `supersedes` and `revives` (suspension of another
//! declaration). An unlicensed relation is L3004 here and has no effect on
//! selection ([`licensed`]); in a matter, neither has one exercised
//! without the motion its capability names ([`unmoved`]).
use super::lattice::{Lattice, Order};
use crate::hir::{
    Declaration, DelegatedOp, Delegation, Expr, ExprKind, Extension, Relation, Type, Value,
};
use crate::temporal::condition::matches;
use crate::temporal::interval::Window;
use crate::temporal::world::{error, nowhere, span_of, Decl, DiagnosticExt, Key, Locate, World};
use litigate_base::{Applicability, Derivation, Diagnostic, DiagnosticCode as C, Span};

include!("delegation/operations.rs");
include!("delegation/diagnostics.rs");
include!("delegation/claims.rs");
