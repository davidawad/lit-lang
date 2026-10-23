//! Static matter checks around replay (SPEC §5.6): pins, parties and
//! perspective, observations, `analysis` removals, and stakes.
mod cov_case_support;

use cov_case_support::{matter, run, ALPHA_ONLY, STAKES};
use litigate_base::Profile;
use litigate_sema::case::NoClock;
use std::collections::BTreeMap;

const BENCHED: &str = "2010-01-04 Benched(complaint) by p";

/// A matter whose only parties are the plaintiff and the defendant.
fn two_parties(head: &str, tail: &str) -> String {
    format!(
        "case t\nuses alpha ^1.0.0\nas_of 2011-06-01\nsummary \"s\"\n{head}\n\
         parties {{ p: Plaintiff Corporation, d: Defendant Corporation }}\n{tail}\n"
    )
}

#[test]
fn parties_and_perspective_table() {
    let rows: Vec<(&str, String)> = vec![
        (
            "uses: the same package twice",
            matter("alpha ^1.0.0, alpha ^1.0.0", "", &[BENCHED], STAKES),
        ),
        (
            "parties: an actor nothing like the package's",
            two_parties("", STAKES).replace("p: Plaintiff", "p: Zzzzzzzzzzzz"),
        ),
        (
            "perspective: names no party",
            two_parties("perspective nobody", STAKES),
        ),
        (
            "perspective: sets an actor the package does not declare",
            two_parties("perspective { Plaintiff: self, Judge: nature }", STAKES),
        ),
        (
            "ok: perspective sets every actor it names",
            two_parties("perspective { Plaintiff: self, Court: nature }", STAKES),
        ),
    ];
    let table: BTreeMap<&str, Vec<String>> = rows
        .iter()
        .map(|(name, text)| (*name, run(text, &NoClock, Profile::Dev)))
        .collect();
    insta::assert_debug_snapshot!(table);
}

#[test]
fn observations_table() {
    let rows: Vec<(&str, &str, &str)> = vec![
        (
            "ok: a decision of an actor no party plays is a draw",
            "perspective p",
            "observe bench: { allow: 2, deny: 1 }",
        ),
        (
            "ok: an actor's role set explicitly",
            "perspective { Court: nature }",
            "observe bench: { allow: 2, deny: 1 }",
        ),
        (
            "outcome: a link edge is not the node's own outcome",
            "perspective p",
            "observe bench: { allow: 2, hop: 1 }",
        ),
        (
            "not a draw: a decision with one edge",
            "perspective p",
            "observe lone: { only: 1 }",
        ),
        (
            "not a draw: a state",
            "perspective p",
            "observe intake: { serve: 1 }",
        ),
        (
            "not a draw: a decision an actor of the perspective takes",
            "perspective { Court: opponent }",
            "observe bench: { allow: 2, deny: 1 }",
        ),
    ];
    let table: BTreeMap<&str, Vec<String>> = rows
        .iter()
        .map(|(name, head, observe)| {
            let text = two_parties(head, &format!("{STAKES}\nbeliefs {{ {observe} }}"));
            (*name, run(&text, &NoClock, Profile::Dev))
        })
        .collect();
    insta::assert_debug_snapshot!(table);
}

#[test]
fn stakes_and_removals_table() {
    let rows: Vec<(&str, String)> = vec![
        (
            "stakes: no block, so no fix to insert the missing ones",
            matter(ALPHA_ONLY, "", &[BENCHED], ""),
        ),
        (
            "stakes: unvalued in strict",
            matter(
                ALPHA_ONLY,
                "",
                &[BENCHED],
                "stakes { answered: 0.usd, defaulted: unvalued }",
            ),
        ),
        (
            "analysis: entries that are not edge names are skipped, a repeat is not",
            matter(
                ALPHA_ONLY,
                "",
                &[BENCHED],
                &format!(
                    "{STAKES}\nanalysis {{ remove_edges: [\"deny\", 3, \"alpha::deny\", \"gamma::ping\"] }}"
                ),
            ),
        ),
        (
            "analysis: removing an edge an event already removed",
            matter(
                ALPHA_ONLY,
                "",
                &["2010-01-04 Forfeited(\"twin-a\") by d cite FRCP 12(h)"],
                &format!("{STAKES}\nanalysis {{ remove_edges: [\"twin-a\"] }}"),
            ),
        ),
        (
            "analysis: a removed edge prunes what is reachable",
            matter(
                ALPHA_ONLY,
                "",
                &[BENCHED],
                "stakes { answered: 0.usd }\nanalysis { remove_edges: [\"deny\"] }",
            ),
        ),
    ];
    let table: BTreeMap<&str, Vec<String>> = rows
        .iter()
        .map(|(name, text)| (*name, run(text, &NoClock, Profile::Strict)))
        .collect();
    insta::assert_debug_snapshot!(table);
}
