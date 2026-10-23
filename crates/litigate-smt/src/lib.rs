//! SMT backend for litigate refinement types.
//!
//! A small, decidable surface: variables of sort `Int` (dates are day
//! numbers) or `Real` (exact rationals), linear comparisons between them,
//! and `not`/`and`/`or` over those (`QF_LIRA`). Build a [`Query`], ask a
//! [`Backend`] — [`Z3`], pinned and statically linked — and get a model,
//! an unsat core, or `unknown`. [`Cached`] memoises answers by a
//! [`ContentHash`] of the canonical question and the solver fingerprint.
//!
//! ```
//! use litigate_smt::{Backend, Formula, LinExpr, Outcome, Query, Z3};
//!
//! let mut q = Query::new();
//! let from = q.int("from").unwrap();
//! let to = q.int("to").unwrap();
//! q.assert("ordered", &Formula::lt(&from, &to)).unwrap();
//! q.assert("backwards", &Formula::ge(&from, LinExpr::from(&to).plus_const(3))).unwrap();
//! let outcome = Z3::new().unwrap().check(&q).unwrap();
//! assert_eq!(outcome, Outcome::Unsat(vec!["ordered".into(), "backwards".into()]));
//! ```
//!
//! See `README.md` for why Z3 (not cvc5), how the solver is pinned and
//! built, and what determinism is guaranteed.
mod backend;
mod cache;
mod query;
mod rational;
mod z3_backend;

use std::fmt;

pub use backend::{Backend, Model, Outcome};
pub use cache::{Cached, ContentHash};
pub use query::{Cmp, Formula, LinExpr, Query, Sort, Var};
pub use rational::Rational;
pub use z3_backend::{DEFAULT_RLIMIT, PINNED_Z3_VERSION, Z3};

/// Everything that can go wrong building or deciding a [`Query`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// A rational with denominator zero.
    ZeroDenominator,
    /// Exact arithmetic left the `i64` numerator/denominator range.
    Overflow,
    /// A variable name or label that is empty or contains `|` or `\`.
    InvalidName(String),
    /// A variable re-declared with the other sort.
    SortConflict(String),
    /// A variable not declared in this query (with this sort).
    UndeclaredVar(String),
    /// Two assertions with one label.
    DuplicateLabel(String),
    /// A model value too large for [`Rational`].
    ValueOutOfRange(String),
    /// The linked solver is not the pinned version.
    VersionMismatch {
        /// The pinned version.
        expected: String,
        /// The linked version.
        found: String,
    },
    /// The solver failed.
    Solver(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ZeroDenominator => f.write_str("rational with zero denominator"),
            Self::Overflow => f.write_str("exact arithmetic overflowed i64"),
            Self::InvalidName(n) => write!(f, "invalid name {n:?}: empty, or contains | or \\"),
            Self::SortConflict(n) => {
                write!(f, "variable {n:?} already declared with the other sort")
            }
            Self::UndeclaredVar(n) => write!(
                f,
                "variable {n:?} is not declared in this query with this sort"
            ),
            Self::DuplicateLabel(n) => write!(f, "assertion label {n:?} is already used"),
            Self::ValueOutOfRange(n) => {
                write!(f, "model value of {n:?} does not fit an i64 rational")
            }
            Self::VersionMismatch { expected, found } => {
                write!(f, "linked Z3 {found}, expected the pinned {expected}")
            }
            Self::Solver(m) => write!(f, "solver failure: {m}"),
        }
    }
}

impl std::error::Error for Error {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_messages() {
        let cases = [
            (Error::ZeroDenominator, "rational with zero denominator"),
            (Error::Overflow, "exact arithmetic overflowed i64"),
            (
                Error::InvalidName("a|b".into()),
                r#"invalid name "a|b": empty, or contains | or \"#,
            ),
            (
                Error::SortConflict("x".into()),
                r#"variable "x" already declared with the other sort"#,
            ),
            (
                Error::UndeclaredVar("x".into()),
                r#"variable "x" is not declared in this query with this sort"#,
            ),
            (
                Error::DuplicateLabel("l".into()),
                r#"assertion label "l" is already used"#,
            ),
            (
                Error::ValueOutOfRange("x".into()),
                r#"model value of "x" does not fit an i64 rational"#,
            ),
            (
                Error::VersionMismatch {
                    expected: "5.1.0".into(),
                    found: "4.8.0".into(),
                },
                "linked Z3 4.8.0, expected the pinned 5.1.0",
            ),
            (Error::Solver("boom".into()), "solver failure: boom"),
        ];
        for (e, msg) in cases {
            assert_eq!(e.to_string(), msg);
        }
    }
}
