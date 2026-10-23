//! The one selection query (SPEC §5.5, design review §3.3 S4): which
//! declaration governs a logical declaration on a date. Lowering, case
//! replay, inline tests, `check`, and the LSP all ask it, so precedence
//! chooses everywhere instead of only explaining.
//!
//! Versions come from the one version-selection query
//! ([`crate::temporal::select`]); the winner of `key` is the claimant
//! [`super::conflict`] ranks first among those active on the date and
//! recorded by `known_at`. When that
//! winner is itself displaced by a claim on it, the selection follows it,
//! so a standing order overriding a local rule that overrides a federal
//! rule governs the federal rule's subject.
use super::conflict::{claimants_at, decide};
use crate::temporal::activity::Inactive;
use crate::temporal::version::Miss;
use crate::temporal::world::{error, DiagnosticExt, Key, World};
use litigate_base::{Diagnostic, DiagnosticCode as C, Span};
use std::collections::BTreeSet;

/// Why no declaration governs a key on a date.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Unselected {
    /// Nothing claims it on the date: why its own versions are inactive.
    Inactive(Inactive),
    /// Claimants no relation or canon orders (declaration index pairs).
    Unresolved(Vec<(usize, usize)>),
}

impl Unselected {
    /// Why nothing governs `what` (a spelled name) on `date`, as a
    /// sentence.
    #[must_use]
    pub fn describe(&self, world: &World<'_>, what: &str, date: &str) -> String {
        match self {
            Self::Inactive(Inactive::Miss(Miss::Ambiguous(windows))) => format!(
                "{} versions of {what} are in force on {date}",
                windows.len()
            ),
            Self::Inactive(_) => format!("no version of {what} is in force on {date}"),
            Self::Unresolved(pairs) => format!(
                "no single declaration governs {what} on {date}: {}",
                unordered(world, pairs)
            ),
        }
    }
}

/// Claimant pairs no relation or canon orders, as a sentence.
#[must_use]
pub fn unordered(world: &World<'_>, pairs: &[(usize, usize)]) -> String {
    let named: Vec<String> = pairs
        .iter()
        .map(|&(a, b)| {
            format!(
                "`{}` and `{}`",
                world.versioned(a, b),
                world.versioned(b, b)
            )
        })
        .collect();
    format!("{} both apply and neither prevails", named.join("; "))
}

/// The declaration (index into [`World::decls`]) governing `key` on
/// `date` among declarations recorded by `known_at`.
///
/// # Errors
/// [`Unselected`] when nothing is active or the claimants are unordered.
pub fn select(
    world: &World<'_>,
    key: &Key,
    date: &str,
    known_at: Option<&str>,
) -> Result<usize, Unselected> {
    let mut key = key.clone();
    let mut seen = BTreeSet::from([key.clone()]);
    loop {
        let w = one(world, &key, date, known_at)?;
        let next = world.key(w);
        // A cycle of claims is L3006; selection stops where it closes.
        if next == key || !seen.insert(next.clone()) {
            return Ok(w);
        }
        key = next;
    }
}

/// The winning claimant of `key` alone.
fn one(
    world: &World<'_>,
    key: &Key,
    date: &str,
    known_at: Option<&str>,
) -> Result<usize, Unselected> {
    let base = crate::temporal::select(world, key, date, known_at);
    let live = claimants_at(world, key, date, known_at);
    match (decide(world, world.lattice(), &live), base) {
        (Ok(w), _) => Ok(w),
        (Err(_), Err(why)) if live.is_empty() => Err(Unselected::Inactive(why)),
        (Err(pairs), _) => Err(Unselected::Unresolved(pairs)),
    }
}

/// [`select`] at a build date, as a diagnostic at `at` on failure: L3007
/// (no version in force, see [`crate::temporal::as_of`]) or L3003 (the
/// claimants are unordered).
///
/// # Errors
/// The diagnostic.
pub fn resolve(
    world: &World<'_>,
    key: &Key,
    date: &str,
    known_at: Option<&str>,
    at: Span,
) -> Result<usize, Diagnostic> {
    match select(world, key, date, known_at) {
        Ok(w) => Ok(w),
        Err(Unselected::Inactive(why)) => Err(crate::temporal::inactive(
            world, key, date, known_at, &why, at,
        )),
        Err(Unselected::Unresolved(pairs)) => Err(unresolved(world, key, date, &pairs, at)),
    }
}

/// L3003 at `at`: `key` has no single governing declaration on `date`.
#[must_use]
pub fn unresolved(
    world: &World<'_>,
    key: &Key,
    date: &str,
    pairs: &[(usize, usize)],
    at: Span,
) -> Diagnostic {
    let name = key.display(&key.pack);
    let mut d = error(
        C::UnresolvedPrecedence,
        at,
        format!("no single declaration governs `{name}` on {date}"),
    );
    for &(a, b) in pairs {
        d = d.with_note(
            world.decls[b].meta.span.clone(),
            format!(
                "`{}` and `{}` both apply, and neither prevails",
                world.versioned(a, b),
                world.versioned(b, b)
            ),
        );
    }
    d.with_note(
        None,
        "declare the order with `notwithstanding` or `subject_to` (`litigate check` reports it as L3003)",
    )
}
