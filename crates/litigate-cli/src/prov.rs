//! Envelope provenance (SPEC §9.2): no timestamps, no absolute paths.
use serde_json::{json, Value};

/// The language version this compiler implements.
pub const LANGUAGE_VERSION: u32 = 1;

/// `{name, version, digest}` of one package.
pub type PackageId = (String, String, String);

/// Provenance for `packages` (sorted by name) at the given dates.
#[must_use]
pub fn of(mut packages: Vec<PackageId>, as_of: Option<&str>, known_at: Option<&str>) -> Value {
    packages.sort();
    packages.dedup();
    json!({
        "compiler_version": env!("CARGO_PKG_VERSION"),
        "language_version": LANGUAGE_VERSION,
        "packages": packages
            .into_iter()
            .map(|(name, version, digest)| json!({"name": name, "version": version, "digest": digest}))
            .collect::<Vec<_>>(),
        "as_of": as_of,
        "known_at": known_at,
    })
}

/// Provenance with no packages.
#[must_use]
pub fn empty() -> Value {
    of(Vec::new(), None, None)
}

/// Provenance for packages of a session.
#[must_use]
pub fn session<'a>(
    packages: impl IntoIterator<Item = &'a crate::ws::Pkg>,
    as_of: Option<&str>,
    known_at: Option<&str>,
) -> Value {
    of(
        packages
            .into_iter()
            .filter(|p| !p.name.is_empty())
            .map(|p| (p.name.clone(), p.version.clone(), p.digest.clone()))
            .collect(),
        as_of,
        known_at,
    )
}
