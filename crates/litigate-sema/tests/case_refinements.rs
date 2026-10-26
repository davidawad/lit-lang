//! Refinements in a matter (SPEC §3.5): a case event's arguments are
//! checked exactly against its type's refinements (L2013 at the
//! argument), and a refinement naming a bound date (`prescribed_due`,
//! `current_due`, `grant_order_entered`) when replay counts the dates it
//! reads, refusing the extension it does not admit.
mod cov_case_support;

use cov_case_support::{check, compile, lock, matter, outcome, package};
use litigate_base::Profile;
use litigate_sema::case::{Clock, DueRequest};
use litigate_sema::hir::Package;
use std::collections::BTreeMap;
use std::sync::OnceLock;

const SIGMA: &str = "module sigma
start open

enum Method { Mail, Personal }

/// A stipulation extends by at most 30 days.
type Cap = {n: Number | n > 0 && n <= 30}

event Served(method: Method)
event Answered()
event Stipulated(days: Cap, method: Method where method == Mail || days <= 3)
/// An order may extend the answer at most 14 days past its prescribed
/// due date, though the rule allows 30.
event Ordered(until: Date where until <= prescribed_due + 14.days && until > grant_order_entered) extends answer to until

counting oc cite \"FRCP 6(a)\" { exclude_trigger_day, include_last_day, holidays ocal }
calendar ocal { new-year: fixed(1, 1) effective [1870-06-28, ..) observed nearest_weekday }

deadline answer: 10.days from Served(Mail) counted_by oc {
  bearer Defendant
  extend { by Court, max 30.days }
}

state open: \"Open\"
{
  answer \"Answer\" -> done by Defendant { event Answered(), within answer }
  stipulate \"Stipulate\" -> open by Plaintiff { event Stipulated() }
}

terminal done: \"Done\" { payoff 0.usd }
";

fn sigma() -> &'static [Package] {
    static P: OnceLock<Vec<Package>> = OnceLock::new();
    P.get_or_init(|| compile(&[package("sigma", &[], "", SIGMA)]))
}

/// Due dates by calendar arithmetic alone.
struct Plus;

impl Clock for Plus {
    fn due(&self, r: &DueRequest) -> Result<String, String> {
        litigate_sema::types::date::add_days(&r.trigger, r.amount).ok_or_else(|| "bad".into())
    }

    fn nominal(&self, r: &DueRequest) -> Result<String, String> {
        self.due(r)
    }
}

fn run(events: &[&str]) -> Vec<String> {
    let text = matter("sigma ^1.0.0", "", events, "stakes { done: 0.usd }");
    let packages = sigma();
    let checked = check(&text, packages, Some(&lock(packages)), &Plus, Profile::Dev);
    let mut out = outcome(&checked);
    out.extend(checked.obligations.iter().map(|o| {
        format!(
            "obligation {} {:?} {:?}",
            o.deadline, o.status, o.extended_to
        )
    }));
    out
}

#[test]
fn case_arguments_and_extensions_are_checked_against_refinements() {
    let rows: &[(&str, &[&str])] = &[
        (
            "an argument outside its alias's refinement and another's: both at their argument",
            &[
                "2011-01-04 Served(Mail) on d",
                "2011-01-06 Stipulated(45, Personal) by p",
            ],
        ),
        (
            "within every refinement: the stipulation is taken",
            &[
                "2011-01-04 Served(Mail) on d",
                "2011-01-06 Stipulated(2, Personal) by p",
            ],
        ),
        (
            "a refinement reading two arguments",
            &[
                "2011-01-04 Served(Mail) on d",
                "2011-01-06 Stipulated(10, Personal) by p",
            ],
        ),
        (
            "an order past its refinement, within the rule's bound: refused at replay",
            &[
                "2011-01-04 Served(Mail) on d",
                "2011-01-10 Ordered(2011-02-10) by c cite \"Order\"",
            ],
        ),
        (
            "an order to an unknown date: the refinement is not judged",
            &[
                "2011-01-04 Served(Mail) on d",
                "2011-01-10 Ordered(?) by c cite \"Order\"",
            ],
        ),
        (
            "an order within its refinement extends the answer",
            &[
                "2011-01-04 Served(Mail) on d",
                "2011-01-10 Ordered(2011-01-20) by c cite \"Order\"",
                "2011-01-19 Answered() by d",
            ],
        ),
    ];
    let out: BTreeMap<&str, Vec<String>> = rows
        .iter()
        .map(|(name, events)| (*name, run(events)))
        .collect();
    insta::assert_debug_snapshot!(out);
}
