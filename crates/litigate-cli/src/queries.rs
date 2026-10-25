//! The check pipeline as salsa queries over the session's database
//! (design review §4.4: one database for everything, lowering,
//! verification and replay included). A host that keeps the database
//! across edits, such as the language server, re-verifies a package only
//! when the packages it can see changed, and re-replays a matter only when
//! its text, its lock, or a loaded package changed.
use crate::clock::{self, Clocks, Fragments};
use crate::matter::{self, CaseCheck};
use crate::verdict::{self, Verdict};
use crate::ws::{self, CaseText};
use litigate_base::Profile;
use litigate_lower::{EngineClock, Lowerer, Options};
use litigate_sema::db::inputs::{CaseFile, PackageInput, Workspace};
use litigate_sema::db::queries::{check_package, manifest};
use litigate_sema::hir;
use litigate_sema::temporal::World;
use std::collections::BTreeSet;

#[allow(missing_docs)] // salsa-generated constructor and accessors
mod key {
    /// What a check is run with: the profile and the package build dates.
    #[salsa::interned(debug)]
    pub struct Build<'db> {
        /// `strict` rather than `dev`.
        #[returns(copy)]
        pub strict: bool,
        /// Law-in-force date (`--as-of`).
        #[returns(ref)]
        pub as_of: Option<String>,
        /// Corpus date (`--known-at`).
        #[returns(ref)]
        pub known_at: Option<String>,
    }
}
pub use key::Build;

impl<'db> Build<'db> {
    /// The key for `profile` and `options`.
    #[must_use]
    pub fn of(db: &'db dyn salsa::Database, profile: Profile, options: &Options) -> Self {
        Self::new(
            db,
            profile == Profile::Strict,
            options.as_of.clone(),
            options.known_at.clone(),
        )
    }

    /// The profile.
    #[must_use]
    pub fn profile(self, db: &dyn salsa::Database) -> Profile {
        if self.strict(db) {
            Profile::Strict
        } else {
            Profile::Dev
        }
    }

    /// The build dates.
    #[must_use]
    pub fn options(self, db: &dyn salsa::Database) -> Options {
        Options {
            as_of: self.as_of(db).clone(),
            known_at: self.known_at(db).clone(),
        }
    }
}

/// The HIR of every package of `ws` that checked, in load order.
fn loaded(db: &dyn salsa::Database, ws: Workspace) -> Vec<&hir::Package> {
    ws.packages(db)
        .iter()
        .filter_map(|p| check_package(db, *p).package.as_ref())
        .collect()
}

/// The engine clock over every loaded counting rule, else the shared
/// library alone (a counting rule that does not lower is reported by its
/// package's verification).
#[salsa::tracked(returns(ref), no_eq)]
pub fn engine_clock(db: &dyn salsa::Database, ws: Workspace) -> EngineClock {
    let all = loaded(db, ws);
    EngineClock::lowered(&Lowerer::new(&all), &all).unwrap_or_else(|_| EngineClock::new([]))
}

/// Every loaded counting version's clock fragment, lowered once per
/// workspace for every [`Clocks`] the checks build.
#[salsa::tracked(returns(ref))]
pub fn clock_fragments(db: &dyn salsa::Database, ws: Workspace) -> Fragments {
    clock::fragments(&loaded(db, ws))
}

/// The packages `pkg` can see: every loaded package for a links package
/// (it links them), else its dependency closure and `core`, read through
/// the manifests so no other package's check is a dependency.
pub(crate) fn visible(
    db: &dyn salsa::Database,
    ws: Workspace,
    pkg: PackageInput,
) -> Vec<&hir::Package> {
    let Ok(own) = manifest(db, pkg) else {
        return Vec::new();
    };
    if own.manifest.links {
        return loaded(db, ws);
    }
    litigate_sema::precedence::query::closure(db, pkg, &own.manifest)
        .into_iter()
        .filter_map(|(p, _)| check_package(db, p).package.as_ref())
        .collect()
}

/// One package's verdict under `build`.
#[salsa::tracked(returns(ref))]
pub fn package_verdict<'db>(
    db: &'db dyn salsa::Database,
    ws: Workspace,
    pkg: PackageInput,
    build: Build<'db>,
) -> Verdict {
    package_verdict_with_cache(db, ws, pkg, build, None)
}

/// [`package_verdict`] with an optional disk cache for command sessions.
pub(crate) fn package_verdict_with_cache<'db>(
    db: &'db dyn salsa::Database,
    ws: Workspace,
    pkg: PackageInput,
    build: Build<'db>,
    cache: Option<&crate::cache::Cache>,
) -> Verdict {
    let own = check_package(db, pkg).package.as_ref();
    let pkg = ws::package(db, pkg, &BTreeSet::new());
    // Lowering and verification know a package by its address: the
    // package verified is the one they see.
    let all: Vec<&hir::Package> = visible(db, ws, pkg.input)
        .into_iter()
        .map(|p| match (own, &pkg.check.package) {
            (Some(o), Some(copy)) if std::ptr::eq(p, o) => copy,
            _ => p,
        })
        .collect();
    let clocks = Clocks::with_fragments(&all, clock_fragments(db, ws));
    verdict::package_with_cache(
        db,
        ws,
        &pkg,
        &all,
        &clocks,
        build.profile(db),
        &build.options(db),
        cache,
    )
}

/// One matter checked under `build`'s profile, at its own dates.
#[salsa::tracked(returns(ref))]
pub fn case_check<'db>(
    db: &'db dyn salsa::Database,
    ws: Workspace,
    case: CaseFile,
    build: Build<'db>,
) -> CaseCheck {
    let all = loaded(db, ws);
    let text = CaseText {
        path: case.path(db).clone(),
        text: case.text(db).clone(),
        lock: case.lock(db).clone(),
    };
    // The schedule counts in the world of the packages the matter uses.
    let uses = ws::uses_of(&text.text);
    let roots: Vec<&hir::Package> = all
        .iter()
        .copied()
        .filter(|p| uses.contains(&p.manifest.name))
        .collect();
    let clocks = Clocks::with_fragments(
        &World::closure_packages(&roots, &all),
        clock_fragments(db, ws),
    );
    matter::check_packages(
        &World::closure_packages(&roots, &all),
        &text,
        &clocks,
        build.profile(db),
        matter::At::default(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testkit::{self, case, package, ACME, MANIFEST, RULE12};
    use crate::ws::Session;
    use litigate_sema::LitDatabase;
    use salsa::plumbing::AsId;

    fn ran(log: &[String], query: &str) -> usize {
        log.iter()
            .filter(|e| e.starts_with(&format!("{query}(")))
            .count()
    }

    /// The `dev` key, interned again after each edit: the same values,
    /// the same key.
    fn dev(db: &LitDatabase) -> (salsa::Id, Build<'_>) {
        let build = Build::of(db, Profile::Dev, &Options::default());
        (build.as_id(), build)
    }

    fn logged(cases: Vec<CaseText>) -> Session {
        let (db, _) = LitDatabase::with_event_log();
        let module = format!("{RULE12}\n\n");
        Session::from_loaded_in(
            db,
            &[package(
                "lit/examples",
                MANIFEST,
                &[("rule12.lit", &module)],
            )],
            cases,
        )
    }

    #[test]
    fn engine_clock_uses_the_loaded_rules_or_an_empty_clock_on_lowering_error() {
        let session = testkit::example_with("");
        let _clock = engine_clock(&session.db, session.ws);

        let broken = testkit::example_with(
            "counting broken cite \"XYZ 6(a)\" {\n  exclude_trigger_day\n  include_last_day\n  holidays federal-holidays\n}",
        );
        assert!(broken.packages[0].check.package.is_some());
        let _clock = engine_clock(&broken.db, broken.ws);
    }

    #[test]
    fn a_build_key_carries_the_profile_and_dates() {
        let db = LitDatabase::default();
        let options = Options {
            as_of: Some("2025-01-01".into()),
            known_at: None,
        };
        let strict = Build::of(&db, Profile::Strict, &options);
        assert_eq!(strict.profile(&db), Profile::Strict);
        assert_eq!(strict.options(&db), options);
        assert_eq!(
            Build::of(&db, Profile::Dev, &options).profile(&db),
            Profile::Dev
        );
    }

    #[test]
    fn a_check_reruns_only_what_an_edit_reaches() {
        let unlocked = testkit::example_and("", vec![case(ACME, None)]);
        let lock = unlocked.matter_lock(&unlocked.cases).render();
        let mut session = logged(vec![case(ACME, Some(&lock))]);
        let inputs = session.case_inputs();
        let pkg = session.packages[0].input;
        let (id, build) = dev(&session.db);
        let first = package_verdict(&session.db, session.ws, pkg, build).clone();
        let replayed = case_check(&session.db, session.ws, inputs[0], build).clone();
        assert!(replayed.lowered.is_some(), "{:?}", replayed.diagnostics);
        let _ = session.db.take_log();

        // Checked again unchanged: nothing re-runs.
        assert_eq!(session.case_inputs(), inputs);
        let (again_id, build) = dev(&session.db);
        assert_eq!(again_id, id);
        let _ = package_verdict(&session.db, session.ws, pkg, build);
        let _ = case_check(&session.db, session.ws, inputs[0], build);
        assert_eq!(session.db.take_log(), Vec::<String>::new());

        // Editing the matter replays it, never re-verifying the package.
        session.cases[0].text = ACME.replace("650", "700");
        assert_eq!(
            session.case_inputs(),
            inputs,
            "the input is kept, its text set"
        );
        let (_, build) = dev(&session.db);
        let _ = package_verdict(&session.db, session.ws, pkg, build);
        let again = case_check(&session.db, session.ws, inputs[0], build).clone();
        let log = session.db.take_log();
        assert_eq!(
            (ran(&log, "package_verdict"), ran(&log, "case_check")),
            (0, 1),
            "{log:?}"
        );
        assert_ne!(again.lowered, replayed.lowered);

        // Another world re-verifies the package, never re-checking sema.
        let dated = Build::of(
            &session.db,
            Profile::Dev,
            &Options {
                as_of: Some("2025-06-01".into()),
                known_at: None,
            },
        );
        let other = package_verdict(&session.db, session.ws, pkg, dated);
        let log = session.db.take_log();
        assert_eq!(ran(&log, "package_verdict"), 1, "{log:?}");
        assert_eq!(ran(&log, "check_package") + ran(&log, "check_module"), 0);
        assert_eq!(other.diagnostics, first.diagnostics);
    }

    #[test]
    fn a_cancelled_check_stops_and_says_so() {
        let session = testkit::example_and("", vec![case(ACME, None)]);
        let done = crate::check_session(session, Profile::Dev, &Options::default());
        assert!(done.complete);
        let session = testkit::example_and("", vec![case(ACME, None)]);
        let stopped =
            crate::check_session_with(session, Profile::Dev, &Options::default(), &|| true);
        assert!(!stopped.complete);
        assert_eq!((stopped.verdicts.len(), stopped.cases.len()), (0, 0));
        // Packages checked, then cancelled before the matter.
        let session = testkit::example_and("", vec![case(ACME, None)]);
        let calls = std::cell::Cell::new(0);
        let half = crate::check_session_with(session, Profile::Dev, &Options::default(), &|| {
            calls.set(calls.get() + 1);
            calls.get() > 1
        });
        assert!(!half.complete);
        assert_eq!((half.verdicts.len(), half.cases.len()), (1, 0));
    }

    #[test]
    fn matters_keep_their_inputs_as_they_change() {
        let mut session = logged(vec![case("a", None), case("b", None)]);
        let inputs = session.case_inputs();
        session.cases[0].path = "cases/other.case".into();
        session.cases[0].lock = Some("version = 1\n".into());
        session.cases.truncate(1);
        let kept = session.case_inputs();
        assert_eq!(kept, inputs[..1]);
        assert_eq!(kept[0].path(&session.db), "cases/other.case");
        assert_eq!(kept[0].lock(&session.db).as_deref(), Some("version = 1\n"));
        assert_eq!(kept[0].text(&session.db), "a");
    }

    #[test]
    fn a_package_without_a_manifest_sees_nothing() {
        let session = Session::from_loaded(
            &[package("lit/x", "not toml", &[("a.lit", "module a\n")])],
            Vec::new(),
        );
        let pkg = session.packages[0].input;
        assert!(visible(&session.db, session.ws, pkg).is_empty());
        let build = Build::of(&session.db, Profile::Dev, &Options::default());
        let verdict = package_verdict(&session.db, session.ws, pkg, build);
        assert!(verdict.lowered.is_none());
    }

    #[test]
    fn clocks_borrow_only_the_fragments_lowered() {
        let session = testkit::example_with("");
        let all = session.hir();
        let shared = clock::fragments(&all);
        assert!(!shared.is_empty());
        let none = Fragments::new();
        let lowered = Clocks::with_fragments(&all, &shared);
        let empty = Clocks::with_fragments(&all, &none);
        assert_eq!(
            (lowered.lowered_versions(), empty.lowered_versions()),
            (shared.len(), 0)
        );
    }
}
