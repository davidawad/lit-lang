//! Chained periods in replay (SPEC §5.3): a deadline triggered by
//! `deadline(d).nominal` or `.adjusted` opens with the event starting `d`'s
//! period and runs from that end of it; a cycle opens nothing, and a chain
//! whose base is not counted is not counted either.
mod cov_case_support;

use cov_case_support::{check, compile, lock, outcome, package};
use litigate_base::Profile;
use litigate_sema::case::{Clock, DueRequest};
use litigate_sema::hir::Package;
use std::fmt::Write as _;
use std::sync::OnceLock;

const CHAIN: &str = "module chain
start open

enum Doc { complaint, motion }

event Served(doc: Doc)
event Opposed()
event Replied()
event Rejoined()

counting oc cite FRCP 6(a) { exclude_trigger_day, include_last_day, holidays ocal }
calendar ocal { new-year: fixed(1, 1) effective [1870-06-28, ..) observed nearest_weekday }

deadline base: 10.days from Served(complaint) counted_by oc
deadline after-nominal: 20.days from deadline(base).nominal counted_by oc
deadline after-adjusted: 5.days from deadline(after-nominal).adjusted counted_by oc
deadline cycle-a: 1.days from deadline(cycle-b).nominal counted_by oc
deadline cycle-b: 1.days from deadline(cycle-a).nominal counted_by oc
deadline huge: 99999999.days from Served(motion) counted_by oc
deadline orphan: 3.days from deadline(huge).adjusted counted_by oc
deadline orphan2: 1.days from deadline(orphan).nominal counted_by oc

state open: \\\"Open\\\"
{
  oppose \\\"Oppose\\\" -> open by Defendant { event Opposed(), within after-nominal }
  reply \\\"Reply\\\" -> open by Plaintiff { event Replied(), within orphan }
  rejoin \\\"Rejoin\\\" -> open by Defendant { event Rejoined(), within orphan2 }
}
";

fn chain() -> &'static [Package] {
    static P: OnceLock<Vec<Package>> = OnceLock::new();
    P.get_or_init(|| compile(&[package("chain", &[], "", &CHAIN.replace("\\\"", "\""))]))
}

/// The trigger plus the period, adjusted; one day less, nominal.
struct Plus;

impl Clock for Plus {
    fn due(&self, r: &DueRequest) -> Result<String, String> {
        litigate_sema::types::date::add_days(&r.trigger, r.amount).ok_or_else(|| "bad".into())
    }

    fn nominal(&self, r: &DueRequest) -> Result<String, String> {
        litigate_sema::types::date::add_days(&r.trigger, r.amount.saturating_sub(1))
            .ok_or_else(|| "bad".into())
    }
}

fn run(events: &[&str]) -> Vec<String> {
    let events = events.iter().fold(String::new(), |mut s, e| {
        let _ = writeln!(s, "  {e}");
        s
    });
    let text = format!(
        "case t\nuses chain ^1.0.0\nas_of 2011-06-01\nsummary \"s\"\n\n\
         parties {{\n  p: Plaintiff Corporation\n  d: Defendant Corporation\n}}\n\
         events {{\n{events}}}\nstakes {{ open: 0.usd }}\n"
    );
    let checked = check(&text, chain(), Some(&lock(chain())), &Plus, Profile::Dev);
    let mut out = outcome(&checked);
    out.extend(
        checked
            .obligations
            .iter()
            .map(|o| format!("obligation {} #{} {:?}", o.deadline, o.trigger, o.status)),
    );
    out
}

/// Served Jan 4: `base` ends Jan 13 (nominal) and Jan 14 (adjusted);
/// `after-nominal` runs 20 days from Jan 13 to Feb 2; `after-adjusted` 5
/// days from Feb 2 to Feb 7. An opposition on Feb 3 is a day late.
#[test]
fn a_chained_period_runs_from_the_end_it_names() {
    insta::assert_debug_snapshot!((
        run(&[
            "2011-01-04 Served(complaint) on d",
            "2011-02-02 Opposed() by d",
        ]),
        run(&[
            "2011-01-04 Served(complaint) on d",
            "2011-02-03 Opposed() by d",
        ]),
    ));
}

/// `orphan` follows `huge`, a period no clock counts: the reply cannot be
/// judged.
#[test]
fn a_chain_whose_base_is_not_counted_is_not_counted() {
    insta::assert_debug_snapshot!(run(&[
        "2011-01-04 Served(motion) on d",
        "2011-03-01 Replied() by p",
        "2011-03-02 Rejoined() by d",
    ]));
}
