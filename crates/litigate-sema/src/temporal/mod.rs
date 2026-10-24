//! Effective-dated rules (SPEC §5.4, DESIGN §4.4): half-open in-force
//! windows, bitemporal (`in_force`, `known_at`) version sets, `as_of`
//! resolution, supersession and revival, and transitions for proceedings
//! pending across an amendment.
//!
//! Everything here is pure over HIR: [`world::World`] indexes the
//! declarations of a package and its dependencies, and the checks return
//! diagnostics. [`crate::precedence::query`] runs them per package.
pub mod activity;
pub mod check;
pub mod condition;
pub mod interval;
pub mod transition;
pub mod version;
pub mod world;

pub use activity::{active, Inactive, Suspension};
pub use interval::Window;
pub use transition::{governing, Governing, MatterEvent, Ungoverned};
pub use version::{Miss, Version, VersionSet};
pub use world::{Key, World};

use litigate_base::{Diagnostic, DiagnosticCode as C, Span};

/// The one version-selection query (SPEC §5.4): the version of `key` in
/// force on `as_of` among those recorded by `known_at` (`None`: the latest
/// corpus), with supersession and revival applied, as an index into
/// [`World::decls`]. Precedence ([`crate::precedence::select`]) asks it for
/// each claimant's own version and then picks the winner; lowering, case
/// replay (through [`governing`]), `check`, `test`, `explain-value` and the
/// LSP all go through that, so they cannot describe different law.
///
/// # Errors
/// [`Inactive`] when no version is in force, or the one in force is
/// superseded.
pub fn select(
    world: &World<'_>,
    key: &Key,
    as_of: &str,
    known_at: Option<&str>,
) -> Result<usize, Inactive> {
    active(world, key, &world.versions(key), as_of, known_at).map(|v| v.decl)
}

/// [`select`] as an L3007 diagnostic: the version of `key` in force on
/// `as_of` among those recorded by `known_at`, with supersession applied.
///
/// # Errors
/// An L3007 diagnostic at `at` naming why no single version applies.
pub fn as_of(
    world: &World<'_>,
    key: &Key,
    as_of: &str,
    known_at: Option<&str>,
    at: Span,
) -> Result<usize, Diagnostic> {
    select(world, key, as_of, known_at)
        .map_err(|why| inactive(world, key, as_of, known_at, &why, at))
}

/// L3007 at `at`: why no version of `key` is active on `as_of` (`why`, as
/// [`active`] reported it).
#[must_use]
pub fn inactive(
    world: &World<'_>,
    key: &Key,
    as_of: &str,
    known_at: Option<&str>,
    why: &Inactive,
    at: Span,
) -> Diagnostic {
    use world::DiagnosticExt;
    let set = world.versions(key);
    let name = key.display(&key.pack);
    let base = error_at(
        at.clone(),
        format!("no version of `{name}` is in force on {as_of}"),
    );
    match why {
        Inactive::Miss(Miss::Before { earliest }) => error_at(
            at,
            format!("`{name}` is not encoded before {earliest}: no version is in force on {as_of}"),
        )
        .with_note(None, not_encoded(earliest)),
        Inactive::Miss(Miss::NotEncoded { before }) => {
            let mut d = Diagnostic::error(
                C::NotEncoded,
                base.span,
                format!("the law of `{name}` is not encoded before {before}: no version is recorded for {as_of}"),
            )
            .with_note(
                None,
                format!("`unknown_before {before}` marks {as_of} as a hole in the corpus, not a day with no law; encode the earlier version to answer it"),
            );
            d.severity = litigate_base::Severity::Warning;
            d
        }
        Inactive::Miss(Miss::NotInForce { hidden }) if hidden.is_empty() => base.with_note(
            None,
            format!(
                "versions: {}",
                set.versions
                    .iter()
                    .map(|v| v.in_force.display())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        ),
        Inactive::Miss(Miss::NotInForce { hidden }) => base.with_note(
            None,
            format!(
                "{} in force then, but recorded after known_at {}",
                hidden.join(", "),
                known_at.unwrap_or("")
            ),
        ),
        Inactive::Miss(Miss::Ambiguous(windows)) => base.with_note(
            None,
            format!(
                "overlapping versions {} (L3001)",
                windows
                    .iter()
                    .map(Window::display)
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        ),
        Inactive::Superseded(s) => base.with_note(
            world.decls[s.by].meta.span.clone(),
            format!(
                "superseded by `{}` from {}{}",
                world.key(s.by).display(&key.pack),
                s.window.from,
                if s.revived {
                    String::new()
                } else {
                    format!("; repeal does not revive it (declare `revives {name}`)")
                }
            ),
        ),
    }
}

/// Why a date before `earliest`, the earliest recorded version, selects
/// nothing (`unknown_before`, SPEC §5.4): a hole in the corpus.
#[must_use]
pub fn not_encoded(earliest: &str) -> String {
    format!(
        "law not encoded before {earliest} (unknown_before): the corpus records no earlier \
         version, a hole in the corpus, not a finding that no law applied; vendor the earlier \
         text and declare its version"
    )
}

fn error_at(at: Span, message: String) -> Diagnostic {
    Diagnostic::error(C::NoVersionInForce, at, message)
}
