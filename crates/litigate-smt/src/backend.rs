//! The solver-facing trait and its answers.
use std::collections::BTreeMap;

use crate::{Error, Query, Rational};

/// A satisfying assignment: every declared variable's value, by name.
pub type Model = BTreeMap<String, Rational>;

/// A solver's answer to a [`Query`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    /// Satisfiable, with a model covering every declared variable (an
    /// `Int` variable's value is an integer).
    Sat(Model),
    /// Unsatisfiable; the labels of an unsatisfiable subset of the
    /// assertions, in assertion order.
    Unsat(Vec<String>),
    /// The solver gave up (for example its resource limit was reached),
    /// with its reason. Never a proof either way.
    Unknown(String),
}

/// A decision procedure for [`Query`]s.
///
/// Implementations must be deterministic: the same query against the same
/// [`Backend::fingerprint`] gives the same [`Outcome`], which is what makes
/// caching by content hash ([`crate::Cached`]) sound.
pub trait Backend {
    /// Identifies the solver, its exact version and every option that can
    /// change an answer. Part of the cache key.
    fn fingerprint(&self) -> String;

    /// Decides `query`.
    ///
    /// # Errors
    ///
    /// [`Error::ValueOutOfRange`] when a model value does not fit
    /// [`Rational`]; [`Error::Solver`] for a solver failure.
    fn check(&mut self, query: &Query) -> Result<Outcome, Error>;
}
