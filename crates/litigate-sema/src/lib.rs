//! Semantic analysis for litigate: the salsa database, packages and
//! lockfiles, name resolution, type checking, and HIR (SPEC §4-§6, §11).
pub mod db;
pub mod hir;
pub mod types;
