//! A matter checked in a world (docs/litigate/CHANGE.md §5.1): the
//! world's layers join the law the matter's packages see. The Rule 59(e)
//! example (docs/litigate/examples/timelines) is late under the 2007 base
//! (10 days) and timely once the 2009 amendment, which supersedes the
//! base's period from December 1, 2009 (28 days), is a layer; a layer the
//! matter uses already is only used, and an amendment the matter uses
//! counts the base's instance instead of opening a second one (SPEC §5.4).
mod case_support;

use case_support::{clock, lock, render, repo_root};
use litigate_base::Profile;
use litigate_sema::case::{check_under, resolve, CheckedCase, Env, Status};
use litigate_sema::hir::{Declaration, Package, Relation};
use std::sync::OnceLock;

/// The example's packages: the post-judgment procedure, the FRCP base,
/// the 2009 amendment and `lit/core`.
fn packages() -> &'static [Package] {
    static P: OnceLock<Vec<Package>> = OnceLock::new();
    P.get_or_init(|| {
        let root = repo_root();
        let dirs = [
            root.join("docs/litigate/examples/timelines/post-judgment"),
            root.join("docs/litigate/examples/change/amendment"),
        ];
        let (db, ws) = litigate_sema::load(&root, &dirs, None).unwrap_or_else(|e| panic!("{e}"));
        litigate_sema::check_workspace(&db, ws)
            .iter()
            .map(|c| {
                c.package
                    .clone()
                    .unwrap_or_else(|| panic!("{:?}", c.diagnostics))
            })
            .collect()
    })
}

fn named(name: &str) -> &'static Package {
    packages()
        .iter()
        .find(|p| p.manifest.name == name)
        .unwrap_or_else(|| panic!("no package {name}"))
}

/// The example matter, with `uses` as given, checked with `layers`.
fn checked(uses: &str, layers: &[&str]) -> CheckedCase {
    checked_in(packages(), uses, layers)
}

/// [`checked`] over `packages`.
fn checked_in(packages: &[Package], uses: &str, layers: &[&str]) -> CheckedCase {
    let text =
        std::fs::read_to_string(repo_root().join("docs/litigate/examples/timelines/rule-59e.case"))
            .unwrap_or_default()
            .replace(
                "uses change-post-judgment ^1.0.0, change-frcp-base ^1.0.0",
                uses,
            );
    let refs: Vec<&Package> = packages.iter().collect();
    let lock = lock(packages);
    let resolved = resolve("cases/t.case", &text, &refs, Some(&lock));
    let case = resolved
        .case
        .unwrap_or_else(|| panic!("{:?}", resolved.diagnostics));
    let layers: Vec<&Package> = layers
        .iter()
        .map(|n| {
            refs.iter()
                .copied()
                .find(|p| p.manifest.name == *n)
                .unwrap_or_else(|| named(n))
        })
        .collect();
    let clock = clock(packages);
    let env = Env {
        packages: &refs,
        lock: Some(&lock),
        clock: &clock,
        profile: Profile::Dev,
    };
    check_under(case, &resolved.spans, env, &layers)
}

/// Each obligation as `deadline status`.
fn obligations(c: &CheckedCase) -> Vec<String> {
    c.obligations
        .iter()
        .map(|o| format!("{} {:?}", o.deadline, o.status))
        .collect()
}

const OWN: &str = "uses change-post-judgment ^1.0.0, change-frcp-base ^1.0.0";

#[test]
fn the_2009_amendment_as_a_layer_makes_the_motion_timely() {
    let base = checked(OWN, &[]);
    assert_eq!(
        obligations(&base),
        ["change-frcp-base::alter-amend-period Late"]
    );
    assert!(render(&base.diagnostics)
        .iter()
        .any(|d| d.contains("L4002")));
    let amended = checked(OWN, &["change-frcp-2009-amendment"]);
    assert_eq!(
        obligations(&amended),
        ["change-frcp-base::alter-amend-period Discharged"]
    );
    assert_eq!(render(&amended.diagnostics), Vec::<String>::new());
    // A layer the matter uses is only used; a layer named twice is one.
    let used = checked(
        OWN,
        &[
            "change-frcp-base",
            "change-frcp-2009-amendment",
            "change-frcp-2009-amendment",
        ],
    );
    assert_eq!(used.obligations, amended.obligations);
}

#[test]
fn a_used_amendment_counts_the_base_instance_under_its_period() {
    let c = checked(
        "uses change-post-judgment ^1.0.0, change-frcp-base ^1.0.0, change-frcp-2009-amendment ^1.0.0",
        &[],
    );
    assert_eq!(
        obligations(&c),
        ["change-frcp-base::alter-amend-period Discharged"]
    );
    assert!(c.obligations.iter().all(|o| o.status == Status::Discharged));
}

#[test]
fn a_relation_that_displaces_nothing_is_no_claim() {
    // `subject_to` yields to its target; it claims nothing, so the base's
    // period still opens its own instance.
    let mut edited = packages().to_vec();
    for p in edited
        .iter_mut()
        .filter(|p| p.manifest.name == "change-frcp-base")
    {
        for m in &mut p.modules {
            for d in &mut m.declarations {
                if let Declaration::Deadline(x) = d {
                    if x.meta.id == "alter-amend-period" {
                        x.meta
                            .relations
                            .push(Relation::SubjectTo("answer-period".into()));
                    }
                }
            }
        }
    }
    let c = checked_in(&edited, OWN, &[]);
    assert_eq!(
        obligations(&c),
        ["change-frcp-base::alter-amend-period Late"]
    );
}
