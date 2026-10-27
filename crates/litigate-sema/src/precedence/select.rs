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
use litigate_base::{Derivation, Diagnostic, DiagnosticCode as C, Span};
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
            Self::Inactive(Inactive::Miss(Miss::Before { earliest })) => {
                format!("{what} is not encoded before {earliest}: no version is in force on {date}")
            }
            Self::Inactive(_) => format!("no version of {what} is in force on {date}"),
            Self::Unresolved(pairs) => format!(
                "no single declaration governs {what} on {date}: {}",
                unordered(world, pairs, "")
            ),
        }
    }
}

/// Claimant pairs no relation or canon orders, as a sentence, each named
/// as seen from pack `from` (`""`: always qualified).
#[must_use]
pub fn unordered(world: &World<'_>, pairs: &[(usize, usize)], from: &str) -> String {
    let named: Vec<String> = pairs
        .iter()
        .map(|&(a, b)| {
            format!(
                "`{}` and `{}`",
                spelled(world, a, from),
                spelled(world, b, from)
            )
        })
        .collect();
    format!("{} both apply and neither prevails", named.join("; "))
}

/// Declaration `i` as `x@from`, named as seen from pack `from`.
fn spelled(world: &World<'_>, i: usize, from: &str) -> String {
    format!(
        "{}@{}",
        world.key(i).display(from),
        world.decls[i].meta.effective.from
    )
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
    chain(world, key, date, known_at).map(|c| c[c.len() - 1])
}

/// The declarations [`select`] passes through for `key` on `date`, in
/// order: the winner of `key`, then the winner of that winner's own key,
/// and so on, each once; the last governs. A layer overriding a layer
/// that overrides the base is the second of three (the base's own
/// version is first only when nothing displaces it). Never empty.
///
/// # Errors
/// [`Unselected`] when nothing is active or the claimants are unordered
/// at any step.
pub fn chain(
    world: &World<'_>,
    key: &Key,
    date: &str,
    known_at: Option<&str>,
) -> Result<Vec<usize>, Unselected> {
    let mut key = key.clone();
    let mut seen = BTreeSet::from([key.clone()]);
    let mut out: Vec<usize> = Vec::new();
    loop {
        let w = one(world, &key, date, known_at)?;
        if out.last() != Some(&w) {
            out.push(w);
        }
        let next = world.key(w);
        // A cycle of claims is L3006; selection stops where it closes.
        if next == key || !seen.insert(next.clone()) {
            return Ok(out);
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
    let mut derivation = Derivation::new("conflict", format!("conflict(`{name}`) on {date}"))
        .at(Some(d.span.clone()));
    for &(a, b) in pairs {
        let (na, nb) = (spelled(world, a, &key.pack), spelled(world, b, &key.pack));
        let undefeated = |x: &str, y: &str, at: Option<Span>| {
            Derivation::new(
                "undefeated",
                format!("undefeated(`{x}`, `{name}`) on {date}"),
            )
            .at(at)
            .premise(Derivation::new(
                "no-defeat",
                format!("¬defeats(`{y}`, `{x}`)"),
            ))
        };
        derivation = derivation
            .premise(undefeated(&na, &nb, world.decls[a].meta.span.clone()))
            .premise(undefeated(&nb, &na, world.decls[b].meta.span.clone()));
        d = d.with_note(
            world.decls[b].meta.span.clone(),
            format!("`{na}` and `{nb}` both apply, and neither prevails"),
        );
    }
    d = d.derived(derivation);
    d.with_note(
        None,
        "declare the order with `notwithstanding` or `subject_to` (`litigate check` reports it as L3003)",
    )
}
