//! Matter facts read from the docket (lg-flu.26): the Federal Circuit's
//! remand destination is the tribunal the appeal came from, which replay
//! derives from the origin's own entry event, so the solved line follows
//! that route with certainty instead of the router's prior.
// Test helpers below may unwrap: a panic is the failed assertion.
#![allow(clippy::unwrap_used)]
mod support;

use support::{copy_dir, litigate_in, repo, scratch};

/// The solved best line's first edge and its probability for a Federal
/// Circuit matter standing at `remand-destination` whose history is
/// `event`.
fn remand(name: &str, event: &str) -> (String, f64) {
    let dir = scratch(name);
    for pkg in ["core", "cafc", "mpep", "ttab"] {
        copy_dir(&repo().join("lit").join(pkg), &dir.join("lit").join(pkg));
    }
    copy_dir(&repo().join("sources"), &dir.join("sources"));
    std::fs::create_dir_all(dir.join("cases")).unwrap();
    std::fs::write(
        dir.join("cases/remand.case"),
        format!(
            "case remand\nuses cafc ^0.1.0\nas_of 2026-06-01\nsummary \"s\"\n\
             start cafc::remand-destination\n\n\
             parties {{\n  appellant: applicant Corporation\n}}\n\n\
             events {{\n  {event}\n}}\n\n\
             stakes {{ cafc::remanded-to-cofc: 0.usd }}\n"
        ),
    )
    .unwrap();
    let lock = litigate_in(&dir, &["lock", "cases"]);
    assert_eq!(lock.code, 0, "{}", lock.stdout);
    let run = litigate_in(&dir, &["run", "cases/remand.case", "--op", "solve"]);
    assert_eq!(run.code, 0, "{}", run.stdout);
    let first = &run.json["result"]["result"]["best_line"][0];
    (
        first["id"].as_str().unwrap().to_owned(),
        first["p"].as_f64().unwrap(),
    )
}

#[test]
fn a_cofc_judgment_remands_to_the_cofc() {
    assert_eq!(
        remand(
            "remand-cofc",
            "2026-03-02 Entered(CofcJudgment) on appellant"
        ),
        ("cafc::remand-route-cofc".to_owned(), 1.0)
    );
}

#[test]
fn an_itc_determination_remands_to_the_itc() {
    assert_eq!(
        remand("remand-itc", "2026-03-02 BecameFinal() on appellant"),
        ("cafc::remand-route-itc".to_owned(), 1.0)
    );
}
