//! Replay (SPEC §5.6) over the `alpha` fixture: deadlines in every day
//! unit and counting situation, typestate failures, `facts` chains, link
//! edges, instances, and option removal.
mod cov_case_support;

use cov_case_support::{matter, run, At, ALPHA_ONLY, STAKES};
use litigate_base::Profile;
use litigate_sema::case::{Clock, NoClock};
use std::collections::BTreeMap;

const SERVED: &str = "2010-01-04 Served(complaint, Mail) on d";
const FILED: &str = "2010-01-04 Filed(complaint, \"by mail\") by p";

fn table(rows: &[(&str, String, &dyn Clock)]) -> BTreeMap<String, Vec<String>> {
    rows.iter()
        .map(|(name, text, clock)| ((*name).to_owned(), run(text, *clock, Profile::Dev)))
        .collect()
}

fn alpha(events: &[&str]) -> String {
    matter(ALPHA_ONLY, "", events, STAKES)
}

#[test]
fn deadline_table() {
    let rows: Vec<(&str, String, &dyn Clock)> = vec![
        (
            "ok: court days, met",
            alpha(&[SERVED, "2010-01-08 Answered(complaint) by d"]),
            &At("2010-01-11"),
        ),
        (
            "missed: court days",
            alpha(&[SERVED, "2010-01-12 Answered(complaint) by d"]),
            &At("2010-01-11"),
        ),
        (
            "excused: a cited excusal between trigger and event",
            alpha(&[
                SERVED,
                "2010-01-06 Excused(complaint) by c cite FRCP 6(b)",
                "2010-01-12 Answered(complaint) by d",
            ]),
            &At("2010-01-11"),
        ),
        (
            "missed: a cited excusal before the trigger does not cover it",
            alpha(&[
                "2010-01-02 Excused(complaint) by c cite FRCP 6(b)",
                SERVED,
                "2010-01-12 Answered(complaint) by d",
            ]),
            &At("2010-01-11"),
        ),
        (
            "missed: a cited event that is not an excusal does not cover it",
            alpha(&[
                SERVED,
                "2010-01-06 Forfeited(\"late\") by d cite FRCP 12(h)",
                "2010-01-12 Answered(complaint) by d",
            ]),
            &At("2010-01-11"),
        ),
        (
            "uncountable: court days without a clock",
            alpha(&[SERVED, "2010-01-08 Answered(complaint) by d"]),
            &NoClock,
        ),
        (
            "uncountable: business days, a trigger with a string argument",
            alpha(&[FILED, "2010-01-08 Answered(complaint) by d"]),
            &NoClock,
        ),
        (
            "uncountable: one day under a rule of the matter's package",
            alpha(&[FILED, "2010-01-08 Reopened(complaint) by d"]),
            &NoClock,
        ),
        (
            "uncountable: a rule of a package the matter does not use",
            alpha(&[FILED, "2010-01-08 Twin(complaint) by d"]),
            &NoClock,
        ),
        (
            "ungoverned: the deadline is not yet in force",
            alpha(&[SERVED, "2010-01-08 Twin(motion) by d"]),
            &NoClock,
        ),
        (
            "untriggered: no period runs without a trigger",
            matter(
                ALPHA_ONLY,
                "start respond",
                &["2010-01-08 Answered(complaint) by d"],
                STAKES,
            ),
            &NoClock,
        ),
    ];
    insta::assert_debug_snapshot!(table(&rows));
}

#[test]
fn typestate_table() {
    let rows: Vec<(&str, String, &dyn Clock)> = vec![
        (
            "ambiguous: two edges take the event",
            alpha(&["2010-01-04 Twin(complaint) by p"]),
            &NoClock,
        ),
        (
            "ended: an event after a terminal",
            alpha(&[
                "2010-01-04 Ruled(2010-01-01, true, 3, 5.usd) by p",
                "2010-01-05 Benched(complaint) by p",
            ]),
            &NoClock,
        ),
        (
            "cited: a cited event that is not an excusal is still illegal",
            alpha(&[SERVED, "2010-01-05 Benched(complaint) by p cite FRCP 1"]),
            &NoClock,
        ),
        (
            "uncited: an excusal without a cite does not excuse",
            alpha(&[SERVED, "2010-01-05 Excused(complaint) by c"]),
            &NoClock,
        ),
        (
            "actor: the plaintiff cannot answer",
            alpha(&[SERVED, "2010-01-05 Answered(complaint) by p"]),
            &NoClock,
        ),
        (
            "either: anyone takes an edge of a node with no actor",
            alpha(&[FILED, "2010-01-05 Answered(complaint) on c"]),
            &At("2010-01-07"),
        ),
        (
            "anonymous: an edge with no id is taken by its event",
            alpha(&[FILED, "2010-01-05 Noted(motion) by d"]),
            &NoClock,
        ),
        (
            "ungoverned: the stage is not yet in force",
            alpha(&[
                "2010-01-04 Answered(motion) by p",
                "2010-02-04 Answered(motion) by p",
            ]),
            &NoClock,
        ),
        (
            "fact: an undecided fact stops the next event",
            alpha(&[
                "2010-01-04 Noted(motion) by p",
                "2010-01-05 Benched(complaint) by p",
            ]),
            &NoClock,
        ),
    ];
    insta::assert_debug_snapshot!(table(&rows));
}

#[test]
fn facts_table() {
    let probe = [
        "2010-01-04 Noted(motion) by p",
        "2010-01-05 Benched(complaint) by p",
    ];
    let rows: Vec<(&str, String, &dyn Clock)> = vec![
        (
            "cycle: facts that lead back are followed once each",
            matter(
                ALPHA_ONLY,
                "",
                &probe,
                &format!("facts {{ f1: to-f2, f2: to-f1 }}\n{STAKES}"),
            ),
            &NoClock,
        ),
        (
            "not a fact: the entry does not move the stage",
            matter(
                ALPHA_ONLY,
                "",
                &[SERVED, "2010-01-05 Answered(complaint) by d"],
                &format!("facts {{ respond: default }}\n{STAKES}"),
            ),
            &At("2010-01-11"),
        ),
        (
            "removed: a fact cannot resolve through a removed edge",
            matter(
                ALPHA_ONLY,
                "",
                &["2010-01-04 Forfeited(\"to-f2\") by d cite FRCP 12(h)"],
                &format!("facts {{ f1: to-f2 }}\n{STAKES}"),
            ),
            &NoClock,
        ),
    ];
    insta::assert_debug_snapshot!(table(&rows));
}

#[test]
#[allow(clippy::too_many_lines)] // one table row per case
fn links_and_instances_table() {
    let all = "alpha ^1.0.0, delta ^1.0.0, gamma ^1.0.0";
    let linked = "alpha ^1.0.0, delta ^1.0.0";
    let benched = "2010-01-04 Benched(complaint) by p";
    let hop = "2010-01-05 Noted(complaint) by c";
    let gamma_stakes = "stakes { answered: 0.usd, defaulted: 0.usd, gamma::g1: 0.usd }";
    let rows: Vec<(&str, String, &dyn Clock)> = vec![
        (
            "ok: a link edge's event, written in the links package, takes it",
            matter(
                all,
                "",
                &[benched, hop, "2010-01-06 gamma::Pinged(a) by p"],
                gamma_stakes,
            ),
            &NoClock,
        ),
        (
            "ok: through a link into an instance, whose events read as its base's",
            matter(
                linked,
                "",
                &[
                    "2010-01-04 Benched(motion) by p",
                    hop,
                    SERVED.replace("01-04", "01-06").as_str(),
                    "2010-01-07 Answered(complaint) by d",
                ],
                STAKES,
            ),
            &At("2010-01-11"),
        ),
        (
            "unused: a link into a package the matter does not use",
            matter(
                linked,
                "",
                &[benched, hop, SERVED.replace("01-04", "01-06").as_str()],
                STAKES,
            ),
            &NoClock,
        ),
        (
            "ok: an option-removing name without an option parameter is an event",
            matter(
                all,
                "",
                &[benched, hop, "2010-01-06 gamma::Struck(a) by p"],
                gamma_stakes,
            ),
            &NoClock,
        ),
        (
            "missed: a link's deadline, declared and triggered in the links package's view",
            matter(
                linked,
                "",
                &[
                    "2010-01-04 Benched(motion) by p",
                    "2010-01-09 Noted(motion) by c",
                ],
                STAKES,
            ),
            &At("2010-01-07"),
        ),
        (
            "removed: a link edge removed by id in its from node's package",
            matter(
                all,
                "",
                &[
                    benched,
                    "2010-01-05 Forfeited(\"hop\") by d cite FRCP 12(h)",
                    "2010-01-06 Noted(complaint) by c",
                ],
                gamma_stakes,
            ),
            &NoClock,
        ),
        (
            "removed: an option far from every edge id",
            matter(
                all,
                "",
                &[
                    benched,
                    "2010-01-05 Forfeited(\"zzzzzzzzzzzzzzzz\") by d cite FRCP 12(h)",
                ],
                gamma_stakes,
            ),
            &NoClock,
        ),
    ];
    insta::assert_debug_snapshot!(table(&rows));
}
