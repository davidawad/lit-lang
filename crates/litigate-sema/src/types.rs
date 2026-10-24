//! Domain types, exact values, the expression type checker (SPEC §6), and
//! construction of HIR from checked items.
pub mod build;
pub mod builtins;
pub mod date;
pub mod expr;
pub mod literal;
pub mod ops;
pub mod rational;
pub mod refine;
pub mod shape;
pub mod ty;
pub mod uncertain;
