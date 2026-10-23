//! Designated deadlines in replay (SPEC §5.3; lg-flu.27): the due date is
//! the trigger's `Date` argument, rolled under the counting rule; a notice
//! designating less than the deadline's period is rejected; an unknown
//! date (`?`) leaves the deadline unknown; and a version whose trigger is
//! another event designates nothing for an instance it did not open.
mod cov_case_support;

use cov_case_support::{check, compile, lock, outcome, package};
use litigate_base::Profile;
use litigate_sema::case::{Clock, DueRequest};
use litigate_sema::hir::Package;
use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::sync::OnceLock;

const SIGMA: &str = "module sigma
start open

event Noticed(due: Date)
event Reset(due: Date)
event Answered()

counting oc cite FRCP 6(a) { exclude_trigger_day, include_last_day, holidays ocal }
calendar ocal { new-year: fixed(1, 1) effective [1870-06-28, ..) observed nearest_weekday }

rule duty answer-duty cite FRCP 12(a)
{
  deadline answer: 30.days from Noticed() counted_by oc effective [2009-12-01, 2012-01-01) cite FRCP 12(a) {
    bearer Defendant
    designated due
  }
  deadline answer: 0.days from Reset() counted_by oc effective [2012-01-01, ..) {
    bearer Defendant
    designated due
  }
}

state open: \"Open\"
{
  answer \"Answer\" -> done by Defendant { event Answered(), within answer }
}

terminal done: \"Done\" { payoff 0.usd }
";

fn sigma() -> &'static [Package] {
    static P: OnceLock<Vec<Package>> = OnceLock::new();
    P.get_or_init(|| compile(&[package("sigma", &[], "", SIGMA)]))
}

/// Due dates by calendar arithmetic, rolling a January 1 to January 2.
struct Plus;

impl Clock for Plus {
    fn due(&self, r: &DueRequest) -> Result<String, String> {
        let due = litigate_sema::types::date::add_days(&r.trigger, r.amount).unwrap_or_default();
        Ok(if due.ends_with("-01-01") {
            due.replace("-01-01", "-01-02")
        } else {
            due
        })
    }
}

fn run(as_of: &str, events: &[&str]) -> Vec<String> {
    let events = events.iter().fold(String::new(), |mut s, e| {
        let _ = writeln!(s, "  {e}");
        s
    });
    let text = format!(
        "case t\nuses sigma ^1.0.0\nas_of {as_of}\nsummary \"s\"\n\n\
         parties {{\n  d: Defendant Corporation\n}}\n\
         events {{\n{events}}}\nstakes {{ done: 0.usd }}\n"
    );
    let checked = check(&text, sigma(), Some(&lock(sigma())), &Plus, Profile::Dev);
    let mut out = outcome(&checked);
    out.extend(
        checked
            .obligations
            .iter()
            .map(|o| format!("obligation {} #{} {:?}", o.deadline, o.trigger, o.status)),
    );
    out
}

#[test]
fn the_trigger_designates_the_due_date() {
    let rows: BTreeMap<&str, Vec<String>> = [
        (
            "an answer by the designated date is timely",
            run(
                "2011-03-01",
                &[
                    "2011-01-04 Noticed(2011-02-10) on d",
                    "2011-02-10 Answered() by d",
                ],
            ),
        ),
        (
            "an answer after the designated date is late",
            run(
                "2011-03-01",
                &[
                    "2011-01-04 Noticed(2011-02-10) on d",
                    "2011-02-11 Answered() by d",
                ],
            ),
        ),
        (
            "a designated January 1 rolls to January 2",
            run(
                "2011-03-01",
                &[
                    "2010-12-01 Noticed(2011-01-01) on d",
                    "2011-01-02 Answered() by d",
                ],
            ),
        ),
        (
            "past the designated date with no answer the duty is breached",
            run("2011-03-01", &["2011-01-04 Noticed(2011-02-10) on d"]),
        ),
        (
            "a notice designating less than 30 days is rejected",
            run("2011-01-10", &["2011-01-04 Noticed(2011-02-02) on d"]),
        ),
        (
            "an unknown designated date leaves the deadline open, never counted",
            run("2011-12-01", &["2011-01-04 Noticed(?) on d"]),
        ),
        (
            "an answer to an unknown date cannot be judged",
            run(
                "2011-12-01",
                &["2011-01-04 Noticed(?) on d", "2011-02-10 Answered() by d"],
            ),
        ),
        (
            "a later version's trigger designates nothing for the notice's instance",
            run(
                "2012-03-01",
                &[
                    "2011-12-01 Noticed(2012-01-05) on d",
                    "2012-02-01 Answered() by d",
                ],
            ),
        ),
    ]
    .into_iter()
    .collect();
    insta::assert_debug_snapshot!(rows);
}
