//! Markings (SPEC §5.6; design review §2.8): a posture is a marking, one
//! token per concurrent track. An edge may `spawn` tracks and `joins`
//! others; a `.case` is legal only as a firing sequence: each event takes
//! the one enabled edge its pattern matches on some track, and an edge
//! that joins a track is enabled only while that track's token is there.
mod common;
mod cov_case_support;

use cov_case_support::{check, compile, lock, outcome, package};
use litigate_base::Profile;
use litigate_sema::hir::Package;
use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::sync::OnceLock;

const MU: &str = "module mu
start pleading

event Moved()
event Ruled()
event Closed()
event Stayed()
event Tried()
event Noted()

state pleading: \"Pleading\"
{
  motion \"Move to dismiss\" -> motion-pending by Defendant { event Moved(), spawn [discovery] }
}
state motion-pending: \"Motion pending\"
{
  ruling \"Ruling\" -> ruled by Court { event Ruled() }
  stay \"Stay the motion\" -> ruled by Court { event Stayed() }
}
state discovery: \"Discovery\"
{
  close \"Close discovery\" -> discovery-closed by Court { event Closed() }
  stay \"Stay discovery\" -> discovery-closed by Court { event Stayed() }
}
state discovery-closed: \"Discovery closed\" { _ \"Wait\" -> discovery-closed by Court { event Noted() } }
state ruled: \"Ruled\"
{
  trial \"Trial\" -> tried by Court { event Tried(), joins [discovery-closed] }
}
terminal tried: \"Tried\" { payoff 0.usd }
";

fn mu() -> &'static [Package] {
    static P: OnceLock<Vec<Package>> = OnceLock::new();
    P.get_or_init(|| compile(&[package("mu", &[], "", MU)]))
}

fn run(events: &[&str]) -> Vec<String> {
    let events = events.iter().fold(String::new(), |mut s, e| {
        let _ = writeln!(s, "  {e}");
        s
    });
    let text = format!(
        "case t\nuses mu ^1.0.0\nas_of 2012-01-01\nsummary \"s\"\n\n\
         parties {{\n  p: Plaintiff Corporation\n  d: Defendant Corporation\n  c: Court Tribunal\n}}\n\
         events {{\n{events}}}\n"
    );
    let checked = check(
        &text,
        mu(),
        Some(&lock(mu())),
        &cov_case_support::At("2011-12-31"),
        Profile::Dev,
    );
    let mut out = outcome(&checked);
    out.push(format!("marking {:?}", checked.marking));
    out
}

#[test]
fn a_case_is_a_firing_sequence_over_its_tracks() {
    let rows: &[(&str, &[&str])] = &[
        (
            "a motion spawns discovery; the trial joins it once closed",
            &[
                "2011-01-04 Moved() by d",
                "2011-02-01 Closed() by c",
                "2011-03-01 Ruled() by c",
                "2011-04-01 Tried() by c",
            ],
        ),
        (
            "the trial is not enabled while discovery is open",
            &[
                "2011-01-04 Moved() by d",
                "2011-03-01 Ruled() by c",
                "2011-04-01 Tried() by c",
            ],
        ),
        (
            "an event both tracks can take is ambiguous",
            &["2011-01-04 Moved() by d", "2011-02-01 Stayed() by c"],
        ),
        (
            "two live tracks at as_of: the first is the lowered start",
            &["2011-01-04 Moved() by d", "2011-03-01 Ruled() by c"],
        ),
        (
            "an event no track can take is illegal at every stage",
            &["2011-01-04 Moved() by d", "2011-03-01 Tried() by c"],
        ),
        (
            "an event of the second track by the wrong party",
            &["2011-01-04 Moved() by d", "2011-02-01 Closed() by d"],
        ),
    ];
    let out: BTreeMap<&str, Vec<String>> = rows
        .iter()
        .map(|(name, events)| (*name, run(events)))
        .collect();
    insta::assert_debug_snapshot!(out);
}

#[test]
fn tracks_are_other_nodes_and_are_not_patched() {
    let out = common::check_body(
        "state a: \"A\" { go \"Go\" -> t { spawn [a] } }\n\
state b: \"B\" { go \"Go\" -> t { spawn [t] } }\n\
state c: \"C\" { go \"Go\" -> t { joins [nowhere, s, s] } }",
    );
    insta::assert_debug_snapshot!(out);
}
