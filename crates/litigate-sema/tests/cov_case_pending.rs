//! Pending moves in replay (SPEC §9.3): a request an `extend` grant reads
//! (`on M()`) that the matter filed, awaiting the order, until the order
//! extending the instance or the denial the grant names (`unless D()`)
//! arrives, or an event ends the instance.
mod cov_case_support;

use cov_case_support::{check, compile, lock, package};
use litigate_base::Profile;
use litigate_sema::case::{Clock, DueRequest};
use litigate_sema::hir::Package;
use std::fmt::Write as _;
use std::sync::OnceLock;

const PEND: &str = "module pend
start open

event Entered()
event Moved()
event Denied()
event Filed()
event Ordered(until: Date) extends period to until

counting pc cite \"FRCP 6(a)\" { exclude_trigger_day, include_last_day, holidays pcal }
calendar pcal { new-year: fixed(1, 1) effective [1870-06-28, ..) observed nearest_weekday }

deadline period: 10.days from Entered() counted_by pc {
  extend { by Court, discretion GoodCause, on Moved(), unless Denied() }
  extend { by Court, discretion GoodCause }
}
deadline plain: 10.days from Entered() counted_by pc {
  extend { by Court, discretion GoodCause }
}

state open: \"Open\"
{
  file \"File\" -> done by Plaintiff { event Filed(), within period }
}

terminal done: \"Done\" { payoff 0.usd }
";

fn pend() -> &'static [Package] {
    static P: OnceLock<Vec<Package>> = OnceLock::new();
    P.get_or_init(|| compile(&[package("pend", &[], "", PEND)]))
}

/// Due dates by calendar arithmetic alone: the trigger plus the period.
struct Plus;

impl Clock for Plus {
    fn due(&self, r: &DueRequest) -> Result<String, String> {
        litigate_sema::types::date::add_days(&r.trigger, r.amount).ok_or_else(|| "bad".into())
    }

    fn nominal(&self, r: &DueRequest) -> Result<String, String> {
        self.due(r)
    }
}

/// Each instance of a matter over `pend` at `as_of` (entered 2011-01-04,
/// so due 2011-01-14), with its status and pending move; or the errors.
fn run(as_of: &str, events: &[&str]) -> Vec<String> {
    let events = events.iter().fold(String::new(), |mut s, e| {
        let _ = writeln!(s, "  {e}");
        s
    });
    let text = format!(
        "case t\nuses pend ^1.0.0\nas_of {as_of}\nsummary \"s\"\n\n\
         parties {{\n  p: Plaintiff Corporation\n  c: Court Tribunal\n}}\n\
         events {{\n  2011-01-04 Entered() by c\n{events}}}\nstakes {{ done: 0.usd }}\n"
    );
    let checked = check(&text, pend(), Some(&lock(pend())), &Plus, Profile::Dev);
    assert!(checked.ok(), "{:?}", checked.diagnostics);
    checked
        .obligations
        .iter()
        .map(|o| {
            let pending = o.pending.as_ref().map_or_else(
                || "-".to_owned(),
                |p| format!("{} on {}", p.request, p.date),
            );
            format!("{} {:?} {pending}", o.deadline, o.status)
        })
        .collect()
}

const MOVED: &str = "2011-01-06 Moved() by p";
const ORDERED: &str = "2011-01-08 Ordered(2011-02-01) by c cite \"FRCP 6(b)(1)(A)\"";

#[test]
fn a_filed_request_awaits_its_order() {
    assert_eq!(
        run("2011-01-10", &[MOVED]),
        [
            "pend::period Open Moved() on 2011-01-06",
            // A deadline no grant of which reads a request has none.
            "pend::plain Open -",
        ]
    );
}

#[test]
fn the_order_or_the_denial_answers_the_request() {
    for answer in [ORDERED, "2011-01-08 Denied() by c"] {
        assert_eq!(
            run("2011-01-10", &[MOVED, answer]),
            ["pend::period Open -", "pend::plain Open -"],
            "{answer}"
        );
    }
}

#[test]
fn a_request_after_the_last_order_is_pending_again() {
    // The first order rests on the motion; the second on no request at
    // all, and still answers the motion before it.
    const AGAIN: &str = "2011-01-09 Moved() by p";
    for events in [&[MOVED, ORDERED, AGAIN][..], &[ORDERED, AGAIN]] {
        assert_eq!(
            run("2011-01-10", events)[0],
            "pend::period Open Moved() on 2011-01-09",
            "{events:?}"
        );
    }
}

#[test]
fn an_ended_instance_awaits_nothing_and_a_lapsed_one_still_does() {
    assert_eq!(
        run("2011-01-10", &[MOVED, "2011-01-08 Filed() by p"])[0],
        "pend::period Discharged -"
    );
    assert_eq!(
        run("2011-01-20", &[MOVED])[0],
        "pend::period Lapsed Moved() on 2011-01-06"
    );
}
