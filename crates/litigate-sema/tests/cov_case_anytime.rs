//! `anytime` edges (SPEC §5.6; lg-xqu8): an edge legal from any posture
//! while an instance of its own `within` deadline is open. Off every track
//! the act discharges that instance and the marking does not move; a track
//! that later reaches the edge's source takes the edge. The edge needs an
//! `event` and a `within` deadline, spawns and joins nothing, and a
//! `patch` may not write it.
mod common;
mod cov_case_support;

use cov_case_support::{check, compile, lock, outcome, package};
use litigate_base::Profile;
use litigate_sema::hir::Package;
use std::fmt::Write as _;
use std::sync::OnceLock;

const AT: &str = "module at
start pleading

event Answered()
event Opposed()
event Moved()
event Replied()
event Ruled()

counting days { holidays cal }
calendar cal { new-year: fixed(1, 1) effective [1870-06-28, ..) }

deadline answer-period: 21.days after Moved() counted_by days
deadline opposition-period: 14.days after Moved() counted_by days
deadline reply-period: 7.days after Opposed() counted_by days

state pleading: \"Pleading\"
{
  move \"Move to dismiss\" -> answer-due by Defendant { event Moved() }
}
decision answer-due: \"Answer\" by Plaintiff
{
  answer \"Answer\" -> opposition-due { event Answered(), within answer-period }
}
decision opposition-due: \"Oppose\" by Plaintiff
{
  oppose \"Oppose\" -> reply-due { event Opposed(), within opposition-period, anytime }
}
decision reply-due: \"Reply\" by Defendant
{
  reply \"Reply\" -> ruling { event Replied(), within reply-period, anytime }
}
state ruling: \"Ruling\" { rule \"Rule\" -> ruled by Court { event Ruled() } }
terminal ruled: \"Ruled\" { payoff 0.usd }
";

fn at() -> &'static [Package] {
    static P: OnceLock<Vec<Package>> = OnceLock::new();
    P.get_or_init(|| compile(&[package("at", &[], "", AT)]))
}

fn run(events: &[&str]) -> Vec<String> {
    let events = events.iter().fold(String::new(), |mut s, e| {
        let _ = writeln!(s, "  {e}");
        s
    });
    let text = format!(
        "case t\nuses at ^1.0.0\nas_of 2011-06-01\nsummary \"s\"\n\n\
         parties {{\n  p: Plaintiff Corporation\n  d: Defendant Corporation\n  c: Court Tribunal\n}}\n\
         events {{\n{events}}}\n"
    );
    let checked = check(
        &text,
        at(),
        Some(&lock(at())),
        &cov_case_support::At("2011-02-01"),
        Profile::Dev,
    );
    let mut out = outcome(&checked);
    out.push(format!("marking {:?}", checked.marking));
    out
}

/// The rendered diagnostics of a run's outcome (`line:col CODE message`).
fn heads(out: &[String]) -> Vec<&str> {
    out.iter()
        .filter(|l| !["ok:", "trace", "marking"].iter().any(|p| l.starts_with(p)))
        .map(|l| l.split_once(' ').map_or(l.as_str(), |(_, rest)| rest))
        .collect()
}

#[test]
fn an_early_opposition_is_taken_when_the_track_arrives() {
    let out = run(&[
        "2011-01-04 Moved() by d",
        "2011-01-10 Opposed() by p",
        "2011-01-12 Answered() by p",
    ]);
    assert!(heads(&out).is_empty(), "{out:#?}");
    let trace: Vec<&str> = out
        .iter()
        .filter(|l| l.starts_with("trace"))
        .map(String::as_str)
        .collect();
    assert_eq!(
        trace,
        [
            "trace Took Moved() at::pleading -> at::answer-due via at::move",
            "trace Anytime Opposed() at::answer-due -> at::answer-due via at::oppose",
            "trace Took Answered() at::answer-due -> at::opposition-due via at::answer",
            "trace Took Opposed() at::opposition-due -> at::reply-due via at::oppose",
        ],
        "{out:#?}"
    );
    assert_eq!(
        out.last().unwrap(),
        "marking [\"at::reply-due\"]",
        "{out:#?}"
    );
}

#[test]
fn an_anytime_edge_needs_its_deadline_open_for_the_party() {
    // The reply period opens only with the opposition, and only the
    // defendant replies.
    for events in [
        &["2011-01-04 Moved() by d", "2011-01-10 Replied() by d"][..],
        &[
            "2011-01-04 Moved() by d",
            "2011-01-10 Opposed() by p",
            "2011-01-11 Replied() by p",
        ][..],
    ] {
        let out = run(events);
        assert_eq!(heads(&out).len(), 1, "{out:#?}");
        assert!(heads(&out)[0].starts_with("L4001"), "{out:#?}");
        assert!(
            out.iter().any(|l| l.contains("`at::reply` is `anytime`")),
            "{out:#?}"
        );
    }
    let early_reply = run(&[
        "2011-01-04 Moved() by d",
        "2011-01-10 Opposed() by p",
        "2011-01-11 Replied() by d",
        "2011-01-12 Answered() by p",
    ]);
    assert!(heads(&early_reply).is_empty(), "{early_reply:#?}");
    assert_eq!(
        early_reply.last().unwrap(),
        "marking [\"at::ruling\"]",
        "{early_reply:#?}"
    );
}

#[test]
fn an_anytime_declaration_needs_an_event_and_its_deadline() {
    let out = common::check_body(
        "deadline d: 14.days after Served(complaint) counted_by c6\n\
state a: \"A\" { go \"Go\" -> t { within d, anytime } }\n\
state b: \"B\" { go \"Go\" -> t { event Served(answer), deadline 14.days, anytime } }\n\
state e: \"E\" { go \"Go\" -> t { event Served(answer), within d, spawn [s], anytime } }\n\
state f: \"F\" { go \"Go\" -> t { event Served(answer, Mail), within d, anytime } }",
    );
    let rejected: Vec<&String> = out
        .iter()
        .filter(|l| l.contains("`anytime` is not allowed"))
        .collect();
    assert_eq!(rejected.len(), 3, "{out:#?}");
    for why in [
        "without an `event`",
        "without a `within`",
        "`spawn`s or `joins`",
    ] {
        assert!(rejected.iter().any(|l| l.contains(why)), "{why}: {out:#?}");
    }
}
