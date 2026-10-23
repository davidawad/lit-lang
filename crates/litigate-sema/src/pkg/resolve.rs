//! Dependency resolution with `PubGrub` (SPEC §4.2). Every loaded package
//! is a candidate; the solver picks one version per package name for the
//! root and its transitive dependencies, the highest that satisfies every
//! requirement, or explains why none does. Load order never decides.
use super::semver::{Requirement, Version};
use pubgrub::{
    DefaultStringReporter, Dependencies, DependencyProvider, PackageResolutionStatistics,
    PubGrubError, Ranges, Reporter, VersionSet,
};
use std::cmp::Reverse;
use std::collections::BTreeMap;
use std::fmt;

/// One loaded package as the resolver sees it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Candidate {
    /// Repository-relative directory (for messages).
    pub root: String,
    /// Package name.
    pub name: String,
    /// Exact version.
    pub version: Version,
    /// Dependency name to requirement.
    pub dependencies: BTreeMap<String, Requirement>,
}

/// Why the root's dependencies do not resolve.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Failure {
    /// Direct dependencies no loaded package is named (L1006).
    pub missing: Vec<String>,
    /// Why no choice of versions satisfies every other requirement
    /// (L1011): the `PubGrub` explanation.
    pub conflict: Option<String>,
}

/// The resolver's answer for one root.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Resolved {
    /// Package name to candidate index: the whole solution (root
    /// included) on success; after a failure, the root plus each direct
    /// dependency's highest satisfying candidate, so that later checks
    /// still see the dependencies that exist.
    pub selected: BTreeMap<String, usize>,
    /// The failure, if any.
    pub failure: Option<Failure>,
}

/// A requirement as a set of versions: exactly the loaded versions it
/// matches (so `SemVer` pre-release rules hold), shown as written.
#[derive(Debug, Clone)]
pub struct Req {
    set: Ranges<Version>,
    text: Option<String>,
}

impl Req {
    fn plain(set: Ranges<Version>) -> Self {
        Self { set, text: None }
    }
}

impl PartialEq for Req {
    fn eq(&self, other: &Self) -> bool {
        self.set == other.set
    }
}

impl Eq for Req {}

impl fmt::Display for Req {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.text {
            Some(text) => f.write_str(text),
            None => write!(f, "{}", self.set),
        }
    }
}

impl VersionSet for Req {
    type V = Version;

    fn empty() -> Self {
        Self::plain(Ranges::empty())
    }

    fn singleton(v: Version) -> Self {
        Self::plain(Ranges::singleton(v))
    }

    fn complement(&self) -> Self {
        Self::plain(self.set.complement())
    }

    fn intersection(&self, other: &Self) -> Self {
        Self::plain(self.set.intersection(&other.set))
    }

    fn contains(&self, v: &Version) -> bool {
        self.set.contains(v)
    }
}

/// Two loaded packages share a name and version, so neither can be chosen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ambiguous(String);

impl fmt::Display for Ambiguous {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for Ambiguous {}

struct Provider<'a> {
    universe: &'a [Candidate],
    root: usize,
    by_name: BTreeMap<&'a str, BTreeMap<&'a Version, Vec<usize>>>,
}

impl<'a> Provider<'a> {
    fn new(universe: &'a [Candidate], root: usize) -> Self {
        let mut by_name: BTreeMap<&str, BTreeMap<&Version, Vec<usize>>> = BTreeMap::new();
        for (i, c) in universe.iter().enumerate() {
            by_name
                .entry(&c.name)
                .or_default()
                .entry(&c.version)
                .or_default()
                .push(i);
        }
        Self {
            universe,
            root,
            by_name,
        }
    }

    fn versions(&self, name: &str) -> impl DoubleEndedIterator<Item = &&'a Version> {
        self.by_name.get(name).into_iter().flat_map(BTreeMap::keys)
    }

    /// The candidate for a version the solver chose (so it is loaded):
    /// the root itself, else the only package with that name and version.
    fn pick(&self, name: &str, version: &Version) -> Result<usize, Ambiguous> {
        let ids = &self.by_name[name][version];
        if ids.contains(&self.root) {
            return Ok(self.root);
        }
        match ids[..] {
            [one] => Ok(one),
            _ => Err(Ambiguous(format!(
                "`{name}` {version} is loaded from more than one directory ({})",
                ids.iter()
                    .map(|i| self.universe[*i].root.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            ))),
        }
    }

    fn requirement(&self, name: &str, req: &Requirement) -> Req {
        let set = self
            .versions(name)
            .filter(|v| req.matches(v))
            .fold(Ranges::empty(), |set, v| {
                set.union(&Ranges::singleton((*v).clone()))
            });
        Req {
            set,
            text: Some(req.to_string()),
        }
    }
}

impl DependencyProvider for Provider<'_> {
    type P = String;
    type V = Version;
    type VS = Req;
    type M = String;
    type Priority = Reverse<usize>;
    type Err = Ambiguous;

    fn prioritize(
        &self,
        package: &String,
        range: &Req,
        _: &PackageResolutionStatistics,
    ) -> Self::Priority {
        Reverse(self.versions(package).filter(|v| range.contains(v)).count())
    }

    fn choose_version(&self, package: &String, range: &Req) -> Result<Option<Version>, Ambiguous> {
        Ok(self
            .versions(package)
            .rev()
            .find(|v| range.contains(v))
            .map(|v| (*v).clone()))
    }

    fn get_dependencies(
        &self,
        package: &String,
        version: &Version,
    ) -> Result<Dependencies<String, Req, String>, Ambiguous> {
        let c = &self.universe[self.pick(package, version)?];
        Ok(Dependencies::Available(
            c.dependencies
                .iter()
                .map(|(name, req)| (name.clone(), self.requirement(name, req)))
                .collect(),
        ))
    }
}

/// Resolves `universe[root]`'s dependencies over every candidate. A
/// direct dependency no loaded package is named is reported as missing and
/// left out, so the rest still resolve; so is a dependency on the root's
/// own name, which no solution can hold beside the root.
#[must_use]
pub fn resolve(universe: &[Candidate], root: usize) -> Resolved {
    let names: std::collections::BTreeSet<&str> =
        universe.iter().map(|c| c.name.as_str()).collect();
    let (missing, present): (BTreeMap<_, _>, BTreeMap<_, _>) = universe[root]
        .dependencies
        .clone()
        .into_iter()
        .partition(|(name, _)| !names.contains(name.as_str()));
    let mut universe = universe.to_vec();
    let own = universe[root].name.clone();
    let (selfish, present): (BTreeMap<_, _>, BTreeMap<_, _>) =
        present.into_iter().partition(|(name, _)| *name == own);
    universe[root].dependencies = present;
    let provider = Provider::new(&universe, root);
    let c = &universe[root];
    let solved =
        pubgrub::resolve(&provider, c.name.clone(), c.version.clone()).map_err(|e| match e {
            PubGrubError::NoSolution(mut tree) => {
                tree.collapse_no_versions();
                DefaultStringReporter::report(&tree)
            }
            other => format!(
                "{other}: {}",
                std::error::Error::source(&other)
                    .map(ToString::to_string)
                    .unwrap_or_default()
            ),
        });
    let selected = solved.as_ref().map_or_else(
        |_| fallback(&provider, root),
        |solution| {
            solution
                .iter()
                .filter_map(|(name, v)| provider.pick(name, v).ok().map(|i| (name.clone(), i)))
                .collect()
        },
    );
    let failure = Failure {
        missing: missing.into_keys().collect(),
        conflict: selfish
            .into_keys()
            .map(|name| {
                format!("`{name}` depends on a package of its own name: one name is one package")
            })
            .next()
            .or(solved.err()),
    };
    Resolved {
        selected,
        failure: (failure != Failure::default()).then_some(failure),
    }
}

/// The root plus, per direct dependency, the highest satisfying
/// candidate taken alone.
fn fallback(provider: &Provider<'_>, root: usize) -> BTreeMap<String, usize> {
    let c = &provider.universe[root];
    let mut out = BTreeMap::from([(c.name.clone(), root)]);
    for (name, req) in &c.dependencies {
        let req = provider.requirement(name, req);
        let best = provider
            .versions(name)
            .rev()
            .find(|v| req.contains(v))
            .and_then(|v| provider.by_name[name.as_str()][*v].first());
        out.extend(best.map(|i| (name.clone(), *i)));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cand(root: &str, name: &str, version: &str, deps: &[(&str, &str)]) -> Candidate {
        Candidate {
            root: root.to_owned(),
            name: name.to_owned(),
            version: Version::parse(version).expect("a version"),
            dependencies: deps
                .iter()
                .map(|(n, r)| {
                    (
                        (*n).to_owned(),
                        Requirement::parse(r).expect("a requirement"),
                    )
                })
                .collect(),
        }
    }

    fn chosen(r: &Resolved, universe: &[Candidate]) -> Vec<String> {
        r.selected
            .iter()
            .map(|(n, i)| format!("{n} {}", universe[*i].version))
            .collect()
    }

    #[test]
    fn picks_the_highest_satisfying_version_not_the_first_loaded() {
        let u = [
            cand("a", "app", "1.0.0", &[("core", "^1.0.0")]),
            cand("c1", "core", "1.0.0", &[]),
            cand("c2", "core", "1.4.0", &[]),
            cand("c3", "core", "2.0.0", &[]),
            cand("c4", "core", "1.5.0-rc.1", &[]),
        ];
        let r = resolve(&u, 0);
        assert_eq!(r.failure, None);
        assert_eq!(chosen(&r, &u), ["app 1.0.0", "core 1.4.0"]);
    }

    #[test]
    fn backtracks_across_transitive_requirements() {
        let u = [
            cand(
                "a",
                "app",
                "1.0.0",
                &[("core", "^1.0.0"), ("frcp", ">=1.0.0")],
            ),
            cand("f2", "frcp", "2.0.0", &[("core", "^2.0.0")]),
            cand("f1", "frcp", "1.1.0", &[("core", "~1.2.0")]),
            cand("c1", "core", "1.2.5", &[]),
            cand("c2", "core", "1.3.0", &[]),
            cand("c3", "core", "2.0.0", &[]),
        ];
        let r = resolve(&u, 0);
        assert_eq!(r.failure, None);
        assert_eq!(chosen(&r, &u), ["app 1.0.0", "core 1.2.5", "frcp 1.1.0"]);
    }

    #[test]
    fn a_conflict_is_explained_and_falls_back_per_dependency() {
        let u = [
            cand(
                "a",
                "app",
                "1.0.0",
                &[("core", "^1.0.0"), ("frcp", "^2.0.0")],
            ),
            cand("f", "frcp", "2.0.0", &[("core", "^2.0.0")]),
            cand("c1", "core", "1.0.0", &[]),
            cand("c2", "core", "2.0.0", &[]),
        ];
        let r = resolve(&u, 0);
        let why = r
            .failure
            .as_ref()
            .and_then(|f| f.conflict.clone())
            .unwrap_or_default();
        assert!(why.contains("frcp 2.0.0 depends on core ^2.0.0"), "{why}");
        assert!(why.contains("app 1.0.0 depends on core ^1.0.0"), "{why}");
        assert_eq!(chosen(&r, &u), ["app 1.0.0", "core 1.0.0", "frcp 2.0.0"]);
    }

    #[test]
    fn missing_dependencies_are_named_and_the_rest_still_resolve() {
        let u = [
            cand(
                "a",
                "app",
                "1.0.0",
                &[("core", "^1.0.0"), ("nope", "^1.0.0")],
            ),
            cand("c", "core", "1.3.0", &[]),
        ];
        let r = resolve(&u, 0);
        let missing = Failure {
            missing: vec!["nope".to_owned()],
            conflict: None,
        };
        assert_eq!(r.failure, Some(missing));
        assert_eq!(chosen(&r, &u), ["app 1.0.0", "core 1.3.0"]);
        let both = resolve(
            &[
                cand(
                    "a",
                    "app",
                    "1.0.0",
                    &[("core", "^2.0.0"), ("nope", "^1.0.0")],
                ),
                u[1].clone(),
            ],
            0,
        );
        let failure = both.failure.unwrap_or_default();
        assert_eq!(failure.missing, ["nope"]);
        assert!(failure.conflict.is_some_and(|c| c.contains("core ^2.0.0")));
    }

    #[test]
    fn duplicate_name_and_version_is_ambiguous_except_for_the_root() {
        let u = [
            cand("a", "app", "1.0.0", &[("core", "=1.0.0")]),
            cand("x", "core", "1.0.0", &[]),
            cand("y", "core", "1.0.0", &[]),
            cand("b", "app", "1.0.0", &[]),
        ];
        let r = resolve(&u, 0);
        assert_eq!(
            r.failure.and_then(|f| f.conflict).as_deref(),
            Some(
                "Retrieving dependencies of core 1.0.0 failed: `core` 1.0.0 is loaded from \
                 more than one directory (x, y)"
            )
        );
        assert_eq!(r.selected.get("core"), Some(&1));
        let root = resolve(&u, 3);
        assert_eq!((root.failure, root.selected.get("app")), (None, Some(&3)));
    }

    #[test]
    fn a_dependency_on_the_roots_own_name_is_a_conflict() {
        let u = [
            cand(
                "a",
                "links",
                "1.0.0",
                &[("links", "^1.0.0"), ("core", "^1.0.0")],
            ),
            cand("b", "links", "1.0.0", &[]),
            cand("c", "core", "1.0.0", &[]),
        ];
        let r = resolve(&u, 0);
        assert_eq!(
            r.failure.clone().and_then(|f| f.conflict).as_deref(),
            Some("`links` depends on a package of its own name: one name is one package")
        );
        assert_eq!(chosen(&r, &u), ["core 1.0.0", "links 1.0.0"]);
    }

    #[test]
    fn version_sets_render_as_written_or_as_ranges() {
        let one = Req::singleton(Version::parse("1.0.0").expect("a version"));
        assert_eq!(one.to_string(), "1.0.0");
        assert_eq!(Req::empty().complement().to_string(), "*");
        assert_eq!(one.intersection(&Req::empty()), Req::empty());
    }
}
