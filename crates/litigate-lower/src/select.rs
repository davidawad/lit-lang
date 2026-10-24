//! Which version of each declaration a build emits (SPEC §8.1 "Multiple
//! versions"): the one sema's precedence selection picks at the build's
//! `(as_of, known_at)` over the versions [`temporal::select`] finds, with
//! supersession and displacing claims applied.
//! A reference resolves to the selected claimant, and a declaration some
//! claimant displaces is absent, exactly as replay sees them.
use litigate_base::{Diagnostic, Span};
use litigate_sema::hir::{self, Declaration};
use litigate_sema::precedence::select::{self, Unselected};
use litigate_sema::temporal::world::meta_of;
use litigate_sema::temporal::{self, Inactive, Key, Miss, World};

/// The build date pair and the declarations of every loaded package.
pub(crate) struct Selection<'a> {
    world: World<'a>,
    /// Law-in-force date.
    pub as_of: String,
    /// Corpus date (`None`: the latest corpus).
    pub known_at: Option<String>,
}

impl<'a> Selection<'a> {
    /// A selection over `world` at `(as_of, known_at)`.
    pub fn new(world: World<'a>, as_of: String, known_at: Option<String>) -> Self {
        Self {
            world,
            as_of,
            known_at,
        }
    }

    /// The authority lattice every loaded package shares.
    pub fn lattice(&self) -> &litigate_sema::precedence::Lattice {
        self.world.lattice()
    }

    /// The emitted version of the node of pack `pack` with edge `edge`.
    pub fn node_with_edge(&self, pack: &str, edge: &str) -> Option<&'a hir::Node> {
        self.world.decls.iter().find_map(|d| match d.decl {
            Declaration::Node(n)
                if self.world.packs[d.pack].pack == pack
                    && n.edges.iter().any(|e| e.id.as_deref() == Some(edge))
                    && self.emits(pack, d.decl) == Ok(true) =>
            {
                Some(n)
            }
            _ => None,
        })
    }

    /// Whether `decl` of pack `pack` is the version this build emits.
    /// Declarations without versions (links, instances, types) always are;
    /// a declaration with no version in force, or displaced by a claimant
    /// that prevails over it, is simply absent.
    ///
    /// # Errors
    /// L3007 when overlapping versions make the choice ambiguous, L3003
    /// when its claimants are unordered.
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

    /// The declaration selected for `key` at the build dates when it is a
    /// version of `key` itself; `Ok(None)` when none is in force, it is
    /// superseded, or another claimant displaces it.
    fn active(&self, key: &Key, at: Option<Span>) -> Result<Option<usize>, Box<Diagnostic>> {
        let at = at.unwrap_or_else(|| temporal::world::nowhere(&key.name));
        match select::select(&self.world, key, &self.as_of, self.known_at.as_deref()) {
            Ok(w) => Ok((self.world.key(w) == *key).then_some(w)),
            Err(Unselected::Inactive(why @ Inactive::Miss(Miss::Ambiguous(_)))) => {
                Err(Box::new(temporal::inactive(
                    &self.world,
                    key,
                    &self.as_of,
                    self.known_at.as_deref(),
                    &why,
                    at,
                )))
            }
            Err(Unselected::Inactive(_)) => Ok(None),
            Err(Unselected::Unresolved(pairs)) => Err(Box::new(select::unresolved(
                &self.world,
                key,
                &self.as_of,
                &pairs,
                at,
            ))),
        }
    }

    /// The declaration a reference written in pack `pack` names at the
    /// build dates (`x` or `q::x`: the selected claimant; `x@D`: that
    /// version).
    ///
    /// # Errors
    /// L3007 at `at` when nothing is in force or the `@D` version does not
    /// exist, L3003 when the claimants are unordered.
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
            None => select::resolve(
                &self.world,
                &target.key,
                &self.as_of,
                self.known_at.as_deref(),
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
