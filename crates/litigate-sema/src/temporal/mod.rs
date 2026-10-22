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

/// The version of `key` in force on `as_of` among those recorded by
/// `known_at`, with supersession applied.
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
    let set = world.versions(key);
    let name = key.display(&key.pack);
    active(world, key, &set, as_of, known_at)
        .map(|v| v.decl)
        .map_err(|why| {
            use world::DiagnosticExt;
            let base = error_at(at, format!("no version of `{name}` is in force on {as_of}"));
            match why {
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
        })
}

fn error_at(at: Span, message: String) -> Diagnostic {
    Diagnostic::error(C::NoVersionInForce, at, message)
}
