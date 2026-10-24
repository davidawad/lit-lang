//! Bitemporal claims (SPEC §5.4; design review 2026-10-23 §2.6, P9, L5):
//! a version is held by the corpus from its `known_at` until it is
//! `retracted`, so a correction restates the window it corrects in a later
//! transaction-time slice; disjointness is checked per slice; and before a
//! declaration's `unknown_before` a query no version answers is a hole
//! (L3010), not a day with no law.
mod cov_case_support;
mod cov_rules_support;
mod legal;

use cov_rules_support::{demo, world};
use legal::table;
use litigate_sema::hir::Declaration;
use litigate_sema::temporal::{self, Key, World};
use std::collections::BTreeMap;

fn d(id: &str, days: u32, clauses: &str) -> String {
    format!("deadline {id}: {days}.days from Served(complaint) counted_by c6 {clauses}\n")
}

fn key(name: &str) -> Key {
    Key {
        pack: "demo".into(),
        name: name.into(),
    }
}

/// The amount `as_of` selects for `name` on `date` as the corpus held it
/// on `known_at`, or the diagnostic code and message.
fn amount(world: &World<'_>, name: &str, date: &str, known_at: Option<&str>) -> String {
    let at = temporal::world::nowhere("query");
    match temporal::as_of(world, &key(name), date, known_at, at) {
        Ok(i) => match world.decls[i].decl {
            Declaration::Deadline(x) => format!("{}.days", x.amount),
            _ => "not a deadline".into(),
        },
        Err(e) => format!("{} {:?} {}", e.code.as_str(), e.severity, e.message),
    }
}

#[test]
fn a_correction_restates_its_window_in_a_later_slice() {
    insta::assert_debug_snapshot!(table(&[
        (
            "1 a correction recorded when the original is retracted checks",
            &(d(
                "p",
                20,
                "effective [2009-12-01, ..) known_at 2009-03-26 retracted 2020-01-01"
            ) + &d("p", 21, "effective [2009-12-01, ..) known_at 2020-01-01")),
        ),
        (
            "2 a correction recorded while the original is still held overlaps",
            &(d(
                "p",
                20,
                "effective [2009-12-01, ..) known_at 2009-03-26 retracted 2020-01-01"
            ) + &d("p", 21, "effective [2009-12-01, ..) known_at 2019-01-01")),
        ),
        (
            "3 a retraction must come after the version is recorded",
            &d("p", 20, "known_at 2009-03-26 retracted 2009-03-26"),
        ),
        (
            "4 a retraction with no correction leaves the window uncovered",
            &(d("p", 20, "effective [2009-12-01, 2012-01-01)")
                + &d("p", 21, "effective [2012-01-01, ..) retracted 2026-01-01")),
        ),
        (
            "5 a retraction with no date is a syntax error",
            &d("p", 20, "retracted")
        ),
    ]));
    let packs = demo(
        &(d(
            "p",
            20,
            "effective [2009-12-01, ..) known_at 2009-03-26 retracted 2020-01-01",
        ) + &d("p", 21, "effective [2009-12-01, ..) known_at 2020-01-01")),
    );
    let w = world(&packs);
    let rows: BTreeMap<&str, String> = [
        (
            "as recorded in 2015",
            amount(&w, "p", "2012-06-01", Some("2015-01-01")),
        ),
        (
            "as recorded in 2020",
            amount(&w, "p", "2012-06-01", Some("2020-01-01")),
        ),
        ("the latest corpus", amount(&w, "p", "2012-06-01", None)),
        (
            "before either was recorded",
            amount(&w, "p", "2012-06-01", Some("2009-01-01")),
        ),
    ]
    .into_iter()
    .collect();
    insta::assert_debug_snapshot!(rows);
}

#[test]
fn before_unknown_before_a_query_is_a_hole() {
    let packs = demo(
        &(d(
            "p",
            20,
            "effective [2012-01-01, ..) unknown_before 2012-01-01",
        ) + "rule duty r unknown_before 2011-01-01 {\n"
            + &d("q", 9, "effective [2012-01-01, ..)")
            + "}\n"
            + &d("u", 9, "effective [2012-01-01, ..)")),
    );
    let w = world(&packs);
    let rows: BTreeMap<&str, String> = [
        (
            "own unknown_before, before it",
            amount(&w, "p", "2010-06-01", None),
        ),
        (
            "own unknown_before, after it",
            amount(&w, "p", "2013-06-01", None),
        ),
        ("the rule's, before it", amount(&w, "q", "2010-06-01", None)),
        (
            "the rule's, between it and the version",
            amount(&w, "q", "2011-06-01", None),
        ),
        ("none declared", amount(&w, "u", "2010-06-01", None)),
    ]
    .into_iter()
    .collect();
    insta::assert_debug_snapshot!(rows);
}

#[test]
fn a_package_unknown_before_is_every_declarations_default() {
    use legal::{pkg, render, workspace};
    let text = format!(
        "{}{}",
        legal::PRELUDE,
        d("p", 20, "effective [2012-01-01, ..)")
    );
    let mut loaded = pkg(
        "demo",
        "FederalRule",
        "2009-12-01",
        &[],
        &[("main.lit", &text)],
    );
    loaded.manifest = loaded.manifest.replace(
        "known_at = \"2025-01-01\"\n",
        "known_at = \"2025-01-01\"\nunknown_before = \"2012-01-01\"\n",
    );
    let (db, ws) = workspace(&[loaded.clone()]);
    let checks = litigate_sema::check_workspace(&db, ws);
    let packages: Vec<&litigate_sema::hir::Package> =
        checks.iter().filter_map(|c| c.package.as_ref()).collect();
    let w = World::from_packages(&packages);
    assert_eq!(
        amount(&w, "p", "2010-06-01", None),
        "L3010 Warning the law of `p` is not encoded before 2012-01-01: no version is recorded for 2010-06-01"
    );
    // A malformed date is a manifest error.
    loaded.manifest = loaded.manifest.replace(
        "unknown_before = \"2012-01-01\"",
        "unknown_before = \"2012-13-01\"",
    );
    let (db, ws) = workspace(&[loaded]);
    let found = render(&db, ws);
    assert!(
        found
            .iter()
            .any(|d| d.contains("L1004") && d.contains("unknown_before")),
        "{found:#?}"
    );
}

/// Due dates by calendar arithmetic alone.
struct Plus;

impl litigate_sema::case::Clock for Plus {
    fn due(&self, r: &litigate_sema::case::DueRequest) -> Result<String, String> {
        litigate_sema::types::date::add_days(&r.trigger, r.amount).ok_or_else(|| "bad".into())
    }

    fn nominal(&self, r: &litigate_sema::case::DueRequest) -> Result<String, String> {
        self.due(r)
    }
}

#[test]
fn replay_reports_a_trigger_whose_law_is_not_encoded() {
    use cov_case_support::{check, compile, lock, outcome, package};
    use litigate_base::Profile;
    const OLD: &str = "module old
start open
enum Doc { complaint }
event Served(doc: Doc)
event Answered(doc: Doc)
event Noted()
counting oc cite \"FRCP 6(a)\" { exclude_trigger_day, include_last_day, holidays ocal }
calendar ocal { new-year: fixed(1, 1) effective [1870-06-28, ..) observed nearest_weekday }
deadline answer: 21.days from Served(complaint) counted_by oc effective [2012-01-01, ..) unknown_before 2012-01-01
deadline other: 21.days from Noted() counted_by oc effective [2012-01-01, ..) unknown_before 2012-01-01
state open: \"Open\" effective [2009-12-01, ..)
{
  answer \"Answer\" -> open by Defendant { event Answered(complaint), within answer }
}
";
    let packages = compile(&[package("old", &[], "", OLD)]);
    let matter = |events: &str| {
        format!(
            "case t\nuses old ^1.0.0\nas_of 2013-01-01\nsummary \"s\"\n\n\
             parties {{\n  p: Plaintiff Corporation\n  d: Defendant Corporation\n}}\n\
             events {{\n{events}}}\n"
        )
    };
    let run = |events: &str| {
        let checked = check(
            &matter(events),
            &packages,
            Some(&lock(&packages)),
            &Plus,
            Profile::Dev,
        );
        outcome(&checked)
    };
    let rows: BTreeMap<&str, Vec<String>> = [
        (
            "a trigger before the encoded law is a hole, and the act is not judged",
            run("  2011-01-04 Served(complaint) on d\n  2011-01-20 Answered(complaint) by d\n"),
        ),
        (
            "after it, the period runs",
            run("  2012-01-04 Served(complaint) on d\n  2012-01-20 Answered(complaint) by d\n"),
        ),
    ]
    .into_iter()
    .collect();
    insta::assert_debug_snapshot!(rows);
}

#[test]
fn a_version_applies_to_the_proceedings_commenced_in_its_window() {
    let split = d(
        "p",
        30,
        "effective [2017-01-14, ..) commenced [2017-01-14, 2026-06-01)",
    ) + &d(
        "p",
        60,
        "effective [2026-06-01, ..) commenced [2026-06-01, ..)",
    );
    insta::assert_debug_snapshot!(table(&[
        ("1 two versions split one window by commencement", &split),
        (
            "2 commencement windows that overlap overlap",
            &(d(
                "p",
                30,
                "effective [2017-01-14, ..) commenced [2017-01-14, 2027-01-01)"
            ) + &d(
                "p",
                60,
                "effective [2026-06-01, ..) commenced [2026-06-01, ..)"
            )),
        ),
        (
            "3 an empty commencement window",
            &d("p", 30, "commenced [2026-06-01, 2026-06-01)"),
        ),
        (
            "4 a rule's window applies to its declarations",
            &("rule duty r commenced [2017-01-14, ..) {\n".to_owned() + &d("p", 30, "") + "}\n"),
        ),
    ]));
    // A package query concerns a proceeding commenced on its own date.
    let packs = demo(&split);
    let w = world(&packs);
    assert_eq!(amount(&w, "p", "2026-07-01", None), "60.days");
    assert_eq!(amount(&w, "p", "2020-07-01", None), "30.days");
    // A matter commenced before the cut keeps the old version throughout.
    let mut pending = World::default();
    for (m, modules) in &packs {
        pending.add(litigate_sema::temporal::world::PackInfo::of(m), modules);
    }
    pending.set_commenced(Some("2026-01-05".into()));
    assert_eq!(amount(&pending, "p", "2026-07-01", None), "30.days");
}
