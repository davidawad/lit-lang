//! Matters before the law is encoded (SPEC §5.4 `unknown_before`;
//! lg-flu.26): a date before every recorded version of a stage or a
//! deadline, and before its package's `in_force`, is a hole in the corpus,
//! "law not encoded before D", not a finding that no law applied. A
//! trigger then opens an instance whose due date is unknown (a warning),
//! never counted under the later version, so an act against it is neither
//! timely nor late and it is never breached. Inside the package's corpus a
//! declaration enacted later was simply not yet law.
mod cov_case_support;

use cov_case_support::{check, compile, lock, outcome, package, At};
use litigate_base::Profile;
use litigate_sema::hir::Package;
use std::fmt::Write as _;
use std::sync::OnceLock;

const TAU: &str = "module tau
start open

event Noticed()
event Answered()
event Enacted()

counting oc cite \"FRCP 6(a)\" { exclude_trigger_day, include_last_day, holidays ocal }
calendar ocal { new-year: fixed(1, 1) effective [1870-06-28, ..) observed nearest_weekday }

rule duty answer-duty cite \"FRCP 12(a)\"
{
  deadline answer: 30.days from Noticed() counted_by oc cite \"FRCP 12(a)\" {
    bearer Defendant
  }
}

deadline fresh: 30.days from Enacted() counted_by oc effective [2012-01-01, ..) cite \"FRCP 12(a)\"

state open: \"Open\" effective [2000-01-01, ..)
{
  answer \"Answer\" -> done by Defendant { event Answered(), within answer }
}

terminal done: \"Done\" { payoff 0.usd }
";

fn tau() -> &'static [Package] {
    static P: OnceLock<Vec<Package>> = OnceLock::new();
    P.get_or_init(|| compile(&[package("tau", &[], "", TAU)]))
}

fn run(as_of: &str, events: &[&str]) -> Vec<String> {
    let events = events.iter().fold(String::new(), |mut s, e| {
        let _ = writeln!(s, "  {e}");
        s
    });
    let text = format!(
        "case t\nuses tau ^1.0.0\nas_of {as_of}\nsummary \"s\"\n\n\
         parties {{\n  d: Defendant Corporation\n}}\n\
         events {{\n{events}}}\nstakes {{ done: 0.usd }}\n"
    );
    let checked = check(
        &text,
        tau(),
        Some(&lock(tau())),
        &At("2010-02-01"),
        Profile::Dev,
    );
    let mut out = outcome(&checked);
    out.extend(checked.obligations.iter().map(|o| {
        format!(
            "obligation {} #{} {:?} unencoded {:?}",
            o.deadline, o.trigger, o.status, o.unencoded
        )
    }));
    out
}

#[test]
fn a_trigger_before_the_earliest_version_opens_an_unknown_instance() {
    insta::assert_debug_snapshot!(run("2013-01-01", &["2008-06-02 Noticed() on d"]));
}

#[test]
fn an_act_after_the_law_is_encoded_is_neither_timely_nor_late() {
    insta::assert_debug_snapshot!(run(
        "2013-01-01",
        &["2008-06-02 Noticed() on d", "2010-03-01 Answered() by d"]
    ));
}

#[test]
fn an_act_before_the_law_is_encoded_names_the_date() {
    insta::assert_debug_snapshot!(run(
        "2013-01-01",
        &["2008-06-02 Noticed() on d", "2008-07-01 Answered() by d"]
    ));
}

#[test]
fn a_stage_before_its_first_version_names_the_date() {
    insta::assert_debug_snapshot!(run("2013-01-01", &["1999-06-01 Noticed() on d"]));
}

#[test]
fn a_trigger_once_the_law_is_encoded_counts() {
    insta::assert_debug_snapshot!(run(
        "2013-01-01",
        &["2010-01-04 Noticed() on d", "2010-01-20 Answered() by d"]
    ));
}

#[test]
fn a_deadline_enacted_inside_the_corpus_was_not_yet_law() {
    insta::assert_debug_snapshot!(run("2013-01-01", &["2011-06-01 Enacted() on d"]));
}
