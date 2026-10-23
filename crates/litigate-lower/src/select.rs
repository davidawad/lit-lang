//! Which version of each declaration a build emits (SPEC §8.1 "Multiple
//! versions"): the one active at the build's `(as_of, known_at)`, with
//! supersession applied, through sema's own bitemporal index.
use litigate_base::{Diagnostic, Span};
use litigate_sema::hir::{self, Declaration};
use litigate_sema::temporal::world::meta_of;
use litigate_sema::temporal::{self, active, Inactive, Key, Miss, World};

/// The build date pair and the declarations of every loaded package.
pub(crate) struct Selection<'a> {
    world: World<'a>,
    /// Law-in-force date.
    pub as_of: String,
    /// Corpus date.
    pub known_at: String,
}

impl<'a> Selection<'a> {
    /// A selection over `packages` at `(as_of, known_at)`.
    pub fn new(packages: &[&'a hir::Package], as_of: String, known_at: String) -> Self {
        Self {
            world: World::from_packages(packages),
            as_of,
            known_at,
        }
    }

    /// Whether `decl` of pack `pack` is the version this build emits.
    /// Declarations without versions (links, instances, types) always are;
    /// a declaration with no version in force is simply absent.
    ///
    /// # Errors
    /// L3007 when overlapping versions make the choice ambiguous.
    pub fn emits(&self, pack: &str, decl: &Declaration) -> Result<bool, Diagnostic> {
        let Some(meta) = meta_of(decl) else {
            return Ok(true);
        };
        let key = Key {
            pack: pack.to_owned(),
            name: meta.id.clone(),
        };
        match self.active(&key, meta.span.clone()) {
            Ok(Some(i)) => Ok(std::ptr::eq(self.world.decls[i].decl, decl)),
            Ok(None) => Ok(false),
            Err(d) => Err(*d),
        }
    }

    /// The version of `key` active at the build dates; `Ok(None)` when none
    /// is in force (or it is superseded).
    fn active(&self, key: &Key, at: Option<Span>) -> Result<Option<usize>, Box<Diagnostic>> {
        let set = self.world.versions(key);
        match active(&self.world, key, &set, &self.as_of, Some(&self.known_at)) {
            Ok(v) => Ok(Some(v.decl)),
            // `as_of` repeats the same query, so it fails the same way and
            // explains the ambiguity.
            Err(Inactive::Miss(Miss::Ambiguous(_))) => temporal::as_of(
                &self.world,
                key,
                &self.as_of,
                Some(&self.known_at),
                at.unwrap_or_else(|| temporal::world::nowhere(&key.name)),
            )
            .map(Some)
            .map_err(Box::new),
            Err(_) => Ok(None),
        }
    }

    /// The declaration a reference written in pack `pack` names at the
    /// build dates (`x`, `q::x`, or `x@D` for one version).
    ///
    /// # Errors
    /// L3007 at `at` when no version is in force or the `@D` version does
    /// not exist.
    pub fn resolve(&self, pack: &str, text: &str, at: Span) -> Result<&'a Declaration, Diagnostic> {
        let target = self.world.target(pack, text);
        let set = self.world.versions(&target.key);
        let index = match &target.version {
            Some(from) => set.starting(from).map(|v| v.decl).ok_or_else(|| {
                temporal::world::error(
                    litigate_base::DiagnosticCode::NoVersionInForce,
                    at.clone(),
                    format!("`{text}` names no version starting on {from}"),
                )
            })?,
            None => temporal::as_of(
                &self.world,
                &target.key,
                &self.as_of,
                Some(&self.known_at),
                at,
            )?,
        };
        Ok(self.world.decls[index].decl)
    }

    /// `id@from` of the version of `meta` (the source map's `decl`).
    pub fn versioned(meta: &hir::Metadata) -> String {
        format!("{}@{}", meta.id, meta.effective.from)
    }
}
