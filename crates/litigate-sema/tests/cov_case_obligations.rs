//! Obligation instances in replay (SPEC §5.3, §5.6) over the `omega`
//! fixture: bearers, trigger policies, discharge by the bearing party,
//! breach and lapse at `as_of`, and extension and waiver licensed by a
//! deadline's capabilities. Each row renders the diagnostics, the trace,
//! and every instance with how it ended.
mod cov_case_support;

use cov_case_support::{check, compile, lock, outcome, package};
use litigate_base::Profile;
use litigate_sema::case::{Clock, DueRequest, NoClock, Obligation};
use litigate_sema::hir::Package;
use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::sync::OnceLock;

const OMEGA: &str = "module omega
start open

enum Doc { complaint, amended, motion }

event Served(doc: Doc)
event Answered(doc: Doc)
event Noticed()
event Replied()
event Closed()
event Ordered(until: Date) extends answer to until
event OrderedLate(until: Date) extends late to until
event OrderedReply(until: Date) extends reply to until
event OrderedStale(until: Date) extends stale to until
event Released() waives answer
event ReleasedReply() waives reply
event ReleasedLate() waives late
event Entered()
event OrderedPost(until: Date) extends post to until
event OrderedLocked(until: Date) extends locked to until
event OrderedMotion(until: Date) extends motion-only to until
event ReleasedLocked() waives locked
event MovedPost()
event MovedMotion()
event MovedAppeal()
event OrderedAppeal(until: Date) extends appeal to until
event OrderedCapped(until: Date) extends capped to until
event MovedRequest()
event OrderedRequested(until: Date) extends requested to until
event OrderedInterpreted(until: Date) extends interpreted to until
event MovedTimed()
event DeniedTimed()
event OrderedTimed(until: Date) extends timed to until
event Struck(option: EdgeRef) removes option
event Elected(option: EdgeRef) elects option

counting oc cite \"FRCP 6(a)\" { exclude_trigger_day, include_last_day, holidays ocal, end_of_month clamp }
calendar ocal { new-year: fixed(1, 1) effective [1870-06-28, ..) observed nearest_weekday }

rule duty answer-duty cite \"FRCP 12(a)\"
{
  deadline answer: 10.days from Served(complaint) counted_by oc {
    bearer defendant
    extend { by court, max 10.days }
    waive { by court }
  }
  deadline amended: 2.days from Served(amended) counted_by oc { bearer defendant, trigger later_of(answer) }
  deadline late: 10.days from Served(motion) counted_by oc {
    trigger each
    extend { by court, max 30.days, after_expiry }
    waive { by court, after_expiry }
  }
  deadline stale: 10.days from Noticed() counted_by oc effective [2009-12-01, 2011-01-10) {
    extend { by court, max 10.days }
  }
  deadline fresh: 10.days from Noticed() counted_by oc effective [2011-01-10, ..) supersedes stale
}

deadline reply: 5.days from Noticed() counted_by oc { trigger latest }

deadline post: 10.days from Entered() counted_by oc {
  extend { by court, discretion GoodCause }
  extend { by court, discretion ExcusableNeglect, after_expiry, on MovedPost() }
  extend { by plaintiff, max 3.days }
}
deadline locked: 10.days from Entered() counted_by oc { extend { never cite \"FRCP 6(b)(2)\" } }
deadline motion-only: 10.days from Entered() counted_by oc {
  extend { by court, discretion ExcusableNeglect, after_expiry, on MovedMotion() }
}
deadline appeal: 10.days from Entered() counted_by oc {
  extend { by court, discretion GoodCauseOrExcusableNeglect, to max(prescribed_due + 30.days, grant_order_entered + 14.days), after_expiry, on MovedAppeal(), within prescribed_due + 30.days }
  extend { never cite \"FRAP 26(b)(1)\" }
}
deadline requested: 10.days from Entered() counted_by oc {
  extend { by court, discretion GoodCause, on MovedRequest() }
}
deadline interpreted: 10.days from Entered() counted_by oc {
  extend { by court, max 5.days, interpretation \"a bounded reading\" }
  extend { by court, discretion GoodCause, interpretation \"the rule reaches it only as read\" }
}
deadline timed: 20.days from Entered() counted_by oc {
  extend { by court, discretion GoodCause, max 30.days, on MovedTimed(), within current_due - 10.days, unless DeniedTimed() }
  extend { by court, discretion ExtraordinaryCircumstances, max 30.days, on MovedTimed(), within current_due, unless DeniedTimed() }
}
deadline capped: 10.days from Entered() counted_by oc {
  extend { by court, to min(prescribed_due + 5.days, (grant_order_entered + 2.days)) }
}

state open: \"Open\"
{
  answer \"Answer\" -> open by defendant { event Answered(complaint), within answer }
  answer-amended \"Answer the amendment\" -> open by defendant { event Answered(amended), within amended }
  answer-motion \"Answer the motion\" -> open by defendant { event Answered(motion), within late }
  reply \"Reply\" -> open by plaintiff { event Replied(), within reply }
  close \"Close\" -> done by court { event Closed() }
}

terminal done: \"Done\" { payoff 0.usd }
";

fn omega() -> &'static [Package] {
    static P: OnceLock<Vec<Package>> = OnceLock::new();
    P.get_or_init(|| compile(&[package("omega", &[], "", OMEGA)]))
}

const MONTHLY: &str = "module monthly\nstart open\n\
event Entered()\nevent OrderedMonth(until: Date) extends month-bound to until\n\
calendar cal { new-year: fixed(1, 1) effective [1870-06-28, ..) }\n\
counting oc cite \"Rule 6(a)\" { exclude_trigger_day, include_last_day, roll_forward_on [weekend], holidays cal, end_of_month clamp }\n\
deadline month-bound: 3.months from Entered() counted_by oc { extend { by court, to trigger_date + 4.months, after_expiry } }\n\
state open: \"Open\" { _ \"stay\" -> open }\nterminal done: \"Done\" { payoff 0.usd }\n";

fn monthly() -> &'static [Package] {
    static P: OnceLock<Vec<Package>> = OnceLock::new();
    P.get_or_init(|| compile(&[package("monthly", &[], "", MONTHLY)]))
}

/// Due dates by calendar arithmetic alone: the trigger plus the period.
struct Plus;

impl Clock for Plus {
    fn due(&self, r: &DueRequest) -> Result<String, String> {
        if r.unit == litigate_sema::hir::DayUnit::Month {
            return month_due(&r.trigger, r.amount);
        }
        litigate_sema::types::date::add_days(&r.trigger, r.amount).ok_or_else(|| "bad".into())
    }

    /// One day before the due date, so a chain's two ends differ.
    fn nominal(&self, r: &DueRequest) -> Result<String, String> {
        litigate_sema::types::date::add_days(&r.trigger, r.amount.saturating_sub(1))
            .ok_or_else(|| "bad".into())
    }
}

fn month_due(trigger: &str, months: u64) -> Result<String, String> {
    let (year, month, day) = litigate_sema::types::date::parse(trigger).ok_or("bad")?;
    let index = u64::from(year - 1) * 12 + u64::from(month - 1) + months;
    let year = u32::try_from(index / 12 + 1).map_err(|_| "bad")?;
    let month = u32::try_from(index % 12 + 1).map_err(|_| "bad")?;
    let day = day.min(litigate_sema::types::date::days_in_month(year, month));
    let nominal = format!("{year:04}-{month:02}-{day:02}");
    let weekday =
        (litigate_sema::types::date::day_number(&nominal).ok_or("bad")? + 4).rem_euclid(7);
    let rolled = match weekday {
        6 => litigate_sema::types::date::add_days(&nominal, 2),
        0 => litigate_sema::types::date::add_days(&nominal, 1),
        _ => Some(nominal),
    };
    rolled.ok_or_else(|| "bad".into())
}

/// A matter over `omega` with two defendants, `d` and `e`.
fn matter(as_of: &str, events: &[&str]) -> String {
    let events = events.iter().fold(String::new(), |mut s, e| {
        let _ = writeln!(s, "  {e}");
        s
    });
    format!(
        "case t\nuses omega ^1.0.0\nas_of {as_of}\nsummary \"s\"\n\n\
         parties {{\n  p: plaintiff Corporation\n  d: defendant Corporation\n  e: defendant Corporation\n  c: court Tribunal\n}}\n\
         events {{\n{events}}}\nstakes {{ done: 0.usd }}\n"
    )
}

fn instance(o: &Obligation) -> String {
    let mut s = format!("obligation {} #{} {:?}", o.deadline, o.trigger, o.status);
    if let Some(b) = &o.bearer {
        let _ = write!(s, " bearer {b}");
    }
    if let Some(f) = &o.floor {
        let _ = write!(s, " floor {f}");
    }
    if let Some(x) = &o.extended_to {
        let _ = write!(s, " extended {x}");
    }
    if let Some(x) = &o.extended_under {
        let _ = write!(s, " under {}", x.name());
    }
    if let Some(x) = &o.request_made {
        let _ = write!(s, " requested {x}");
    }
    if let Some(c) = o.closed_by {
        let _ = write!(s, " by #{c}");
    }
    if o.linear {
        s.push_str(" linear");
    }
    s
}

fn run(text: &str, clock: &dyn Clock) -> Vec<String> {
    let checked = check(text, omega(), Some(&lock(omega())), clock, Profile::Dev);
    let mut out = outcome(&checked);
    out.extend(checked.obligations.iter().map(instance));
    out
}

fn table(rows: &[(&str, &str, &[&str])]) -> BTreeMap<String, Vec<String>> {
    rows.iter()
        .map(|(name, as_of, events)| ((*name).to_owned(), run(&matter(as_of, events), &Plus)))
        .collect()
}

const SERVED_D: &str = "2011-01-04 Served(complaint) on d";
const ENTERED: &str = "2011-01-04 Entered() by c";

#[test]
#[allow(clippy::too_many_lines)] // one table row per case
fn bearers_and_trigger_policies() {
    let rows: &[(&str, &str, &[&str])] = &[
        (
            "bearer: service on d binds d; e's answer cannot discharge it",
            "2011-02-01",
            &[SERVED_D, "2011-01-10 Answered(complaint) by e"],
        ),
        (
            "bearer: service by the court binds each defendant",
            "2011-02-01",
            &[
                "2011-01-04 Served(complaint) by c",
                "2011-01-10 Answered(complaint) by d",
            ],
        ),
        (
            "bearer: none declared, so the period is unbound and lapses",
            "2011-02-01",
            &["2011-01-04 Served(motion) on d"],
        ),
        (
            "first: a second service does not restart the period",
            "2011-02-01",
            &[
                SERVED_D,
                "2011-01-10 Served(complaint) on d",
                "2011-01-15 Answered(complaint) by d",
            ],
        ),
        (
            "first: once discharged, a new service opens a new instance",
            "2011-01-20",
            &[
                SERVED_D,
                "2011-01-05 Answered(complaint) by d",
                "2011-01-10 Served(complaint) on d",
            ],
        ),
        (
            "latest: a second notice restarts the reply period",
            "2011-02-01",
            &[
                "2011-01-04 Noticed() by p",
                "2011-01-07 Noticed() by p",
                "2011-01-12 Replied() by p",
            ],
        ),
        (
            "each: two motions are two instances",
            "2011-02-01",
            &[
                "2011-01-04 Served(motion) on d",
                "2011-01-05 Served(motion) on d",
                "2011-01-06 Answered(motion) by d",
            ],
        ),
        (
            "later_of: the amendment absorbs the open answer and its later due date",
            "2011-02-01",
            &[
                SERVED_D,
                "2011-01-06 Served(amended) on d",
                "2011-01-13 Answered(amended) by d",
            ],
        ),
        (
            "later_of: with nothing to absorb, its own period runs",
            "2011-02-01",
            &[
                "2011-01-06 Served(amended) on d",
                "2011-01-09 Answered(amended) by d",
            ],
        ),
        (
            "later_of: the own period is later than the absorbed one",
            "2011-02-01",
            &[
                SERVED_D,
                "2011-01-13 Served(amended) on d",
                "2011-01-15 Answered(amended) by d",
            ],
        ),
        (
            "breach: not yet due at as_of stays open",
            "2011-01-10",
            &[SERVED_D],
        ),
        (
            "ended: no edgeless event after the matter ends",
            "2011-02-01",
            &[
                "2011-01-04 Closed() by c",
                "2011-01-05 Served(complaint) on d",
            ],
        ),
        (
            "ended: no declared effect after the matter ends",
            "2011-02-01",
            &[
                "2011-01-04 Closed() by c",
                "2011-01-05 Struck(answer) by c cite \"FRCP 12(f)\"",
            ],
        ),
    ];
    insta::assert_debug_snapshot!(table(rows));
}

#[test]
#[allow(clippy::too_many_lines)] // one table row per case
fn extensions_and_waivers() {
    let rows: &[(&str, &str, &[&str])] = &[
        (
            "extend: within the bound, the late answer is timely",
            "2011-02-01",
            &[
                SERVED_D,
                "2011-01-12 Ordered(2011-01-20) by c cite \"FRCP 6(b)(1)(A)\"",
                "2011-01-19 Answered(complaint) by d",
            ],
        ),
        (
            "extend: one order reaches every defendant's instance",
            "2011-02-01",
            &[
                "2011-01-04 Served(complaint) by c",
                "2011-01-12 Ordered(2011-01-20) by c cite \"FRCP 6(b)(1)(A)\"",
            ],
        ),
        (
            "extend: beyond the bound is rejected and the answer is late",
            "2011-02-01",
            &[
                SERVED_D,
                "2011-01-12 Ordered(2011-01-30) by c cite \"FRCP 6(b)(1)(A)\"",
                "2011-01-19 Answered(complaint) by d",
            ],
        ),
        (
            "extend: an earlier date extends nothing",
            "2011-02-01",
            &[
                SERVED_D,
                "2011-01-12 Ordered(2011-01-10) by c cite \"FRCP 6(b)(1)(A)\"",
            ],
        ),
        (
            "extend: after expiry without `after_expiry` is rejected",
            "2011-02-01",
            &[
                SERVED_D,
                "2011-01-16 Ordered(2011-01-20) by c cite \"FRCP 6(b)(1)(A)\"",
            ],
        ),
        (
            "extend: after expiry with `after_expiry` is allowed",
            "2011-02-01",
            &[
                "2011-01-04 Served(motion) on d",
                "2011-01-16 OrderedLate(2011-01-20) by c cite \"FRCP 6(b)(1)(B)\"",
                "2011-01-19 Answered(motion) by d",
            ],
        ),
        (
            "extend: a second extension is bounded from the counted date",
            "2011-02-01",
            &[
                SERVED_D,
                "2011-01-12 Ordered(2011-01-20) by c cite \"FRCP 6(b)(1)(A)\"",
                "2011-01-18 Ordered(2011-01-24) by c cite \"FRCP 6(b)(1)(A)\"",
                "2011-01-19 Ordered(2011-01-25) by c cite \"FRCP 6(b)(1)(A)\"",
            ],
        ),
        (
            "extend: a deadline with no `extend` capability admits none",
            "2011-02-01",
            &[
                "2011-01-04 Noticed() by p",
                "2011-01-05 OrderedReply(2011-01-12) by c cite \"FRCP 6(b)\"",
            ],
        ),
        (
            "extend: only the capability's actor may extend",
            "2011-02-01",
            &[
                SERVED_D,
                "2011-01-05 Ordered(2011-01-20) by p cite \"FRCP 6(b)\"",
                "2011-01-06 Ordered(2011-01-20) on d cite \"FRCP 6(b)\"",
            ],
        ),
        (
            "extend: nothing open to extend",
            "2011-02-01",
            &["2011-01-05 Ordered(2011-01-20) by c cite \"FRCP 6(b)\""],
        ),
        (
            "extend: uncited",
            "2011-02-01",
            &[SERVED_D, "2011-01-05 Ordered(2011-01-20) by c"],
        ),
        (
            "extend: no version of the deadline governs the order",
            "2011-02-01",
            &[
                "2011-01-04 Noticed() by p",
                "2011-01-12 OrderedStale(2011-01-20) by c cite \"FRCP 6(b)\"",
            ],
        ),
        (
            "extend: discretion before expiry has no ceiling",
            "2011-02-01",
            &[
                ENTERED,
                "2011-01-10 OrderedPost(2011-06-30) by c cite \"FRCP 6(b)(1)(A)\"",
            ],
        ),
        (
            "extend: discretion after expiry, on motion",
            "2011-02-01",
            &[
                ENTERED,
                "2011-01-18 MovedPost() by p",
                "2011-01-20 OrderedPost(2011-03-01) by c cite \"FRCP 6(b)(1)(B)\"",
            ],
        ),
        (
            "extend: discretion after expiry without a motion after expiry is rejected",
            "2011-02-01",
            &[
                ENTERED,
                "2011-01-12 MovedPost() by p",
                "2011-01-20 OrderedPost(2011-03-01) by c cite \"FRCP 6(b)(1)(B)\"",
            ],
        ),
        (
            "extend: FRAP 4(a)(5)(C), an order on day 40 reaches day 54",
            "2011-04-01",
            &[
                ENTERED,
                "2011-01-20 MovedAppeal() by p",
                "2011-02-23 OrderedAppeal(2011-03-09) by c cite \"FRAP 4(a)(5)\"",
            ],
        ),
        (
            "extend: FRAP 4(a)(5)(C), not day 55",
            "2011-04-01",
            &[
                ENTERED,
                "2011-01-20 MovedAppeal() by p",
                "2011-02-23 OrderedAppeal(2011-03-10) by c cite \"FRAP 4(a)(5)\"",
            ],
        ),
        (
            "extend: FRAP 4(a)(5)(A)(i), a motion past 30 days is no motion",
            "2011-04-01",
            &[
                ENTERED,
                "2011-02-14 MovedAppeal() by p",
                "2011-02-15 OrderedAppeal(2011-02-20) by c cite \"FRAP 4(a)(5)\"",
            ],
        ),
        (
            "extend: a motion before the trigger or after the order is no motion",
            "2011-04-01",
            &[
                "2011-01-03 MovedAppeal() by p",
                ENTERED,
                "2011-01-20 OrderedAppeal(2011-02-01) by c cite \"FRAP 4(a)(5)\"",
                "2011-01-25 MovedAppeal() by p",
            ],
        ),
        (
            "extend: a `min` bound takes the earlier date",
            "2011-02-01",
            &[
                ENTERED,
                "2011-01-14 OrderedCapped(2011-01-16) by c cite \"L.R. 6.1\"",
                "2011-01-15 OrderedCapped(2011-01-18) by c cite \"L.R. 6.1\"",
            ],
        ),
        (
            "extend: a stipulated cap binds only its own grant",
            "2011-02-01",
            &[
                ENTERED,
                "2011-01-10 OrderedPost(2011-01-17) by p cite \"L.R. 6.1\"",
                "2011-01-11 OrderedPost(2011-01-30) by p cite \"L.R. 6.1\"",
                "2011-01-12 OrderedPost(2011-01-30) by d cite \"L.R. 6.1\"",
            ],
        ),
        (
            "extend: the refusal past the window is the one reported",
            "2011-02-01",
            &[
                ENTERED,
                "2011-01-10 OrderedPost(2011-01-12) by c cite \"FRCP 6(b)(1)(A)\"",
            ],
        ),
        (
            "extend: a motion-only grant does not reach an unexpired period",
            "2011-02-01",
            &[
                ENTERED,
                "2011-01-10 OrderedMotion(2011-01-30) by c cite \"FRCP 6(b)(1)(B)\"",
            ],
        ),
        (
            "extend: a request before expiry licenses an order after it",
            "2011-02-01",
            &[
                ENTERED,
                "2011-01-13 MovedRequest() by p",
                "2011-01-20 OrderedRequested(2011-02-10) by c cite \"FRCP 6(b)(1)(A)\"",
            ],
        ),
        (
            "extend: a request after expiry is not one before it",
            "2011-02-01",
            &[
                ENTERED,
                "2011-01-15 MovedRequest() by p",
                "2011-01-20 OrderedRequested(2011-02-10) by c cite \"FRCP 6(b)(1)(A)\"",
            ],
        ),
        (
            "extend: a bounded grant resting on an interpretation stands, with a warning",
            "2011-02-01",
            &[
                ENTERED,
                "2011-01-10 OrderedInterpreted(2011-01-18) by c cite \"FRCP 6(b)(1)(A)\"",
            ],
        ),
        (
            "extend: a discretionary grant resting on an interpretation stands, with a warning",
            "2011-02-01",
            &[
                ENTERED,
                "2011-01-10 OrderedInterpreted(2011-01-30) by c cite \"FRCP 6(b)(1)(A)\"",
            ],
        ),
        (
            "extend: `never` admits no extension",
            "2011-02-01",
            &[
                ENTERED,
                "2011-01-10 OrderedLocked(2011-01-30) by c cite \"FRCP 6(b)(1)(A)\"",
            ],
        ),
        (
            "waive: `extend { never cite \"...\" }` grants no waiver either",
            "2011-02-01",
            &[
                ENTERED,
                "2011-01-10 ReleasedLocked() by c cite \"FRCP 6(b)\"",
            ],
        ),
        (
            "waive: releases the instance, so nothing is breached",
            "2011-02-01",
            &[SERVED_D, "2011-01-05 Released() by c cite \"FRCP 6(b)\""],
        ),
        (
            "waive: after expiry without `after_expiry` is rejected",
            "2011-02-01",
            &[SERVED_D, "2011-01-20 Released() by c cite \"FRCP 6(b)\""],
        ),
        (
            "waive: after expiry with `after_expiry` is allowed",
            "2011-02-01",
            &[
                "2011-01-04 Served(motion) on d",
                "2011-01-20 ReleasedLate() by c cite \"FRCP 6(b)\"",
            ],
        ),
        (
            "waive: a deadline with no `waive` capability admits none",
            "2011-02-01",
            &[
                "2011-01-04 Noticed() by p",
                "2011-01-05 ReleasedReply() by c cite \"FRCP 6(b)\"",
            ],
        ),
    ];
    insta::assert_debug_snapshot!(table(rows));
}

#[test]
fn month_bound_is_trigger_relative_and_uses_the_clock_roll() {
    assert_eq!(month_due("2024-01-31", 3).as_deref(), Ok("2024-04-30"));
    assert_eq!(month_due("2024-01-31", 4).as_deref(), Ok("2024-05-31"));
    assert_eq!(month_due("2024-11-30", 4).as_deref(), Ok("2025-03-31"));

    let timely = matter(
        "2024-06-01",
        &[
            "2024-01-31 Entered() by c",
            "2024-05-31 OrderedMonth(2024-05-31) by c cite \"Rule 1\"",
        ],
    );
    let timely = timely.replace("uses omega ^1.0.0", "uses monthly ^1.0.0");
    let checked = check(
        &timely,
        monthly(),
        Some(&lock(monthly())),
        &Plus,
        Profile::Dev,
    );
    assert!(checked.ok(), "{:#?}", outcome(&checked));

    let late = matter(
        "2024-06-02",
        &[
            "2024-01-31 Entered() by c",
            "2024-06-01 OrderedMonth(2024-06-01) by c cite \"Rule 1\"",
        ],
    );
    let late = late.replace("uses omega ^1.0.0", "uses monthly ^1.0.0");
    let checked = check(
        &late,
        monthly(),
        Some(&lock(monthly())),
        &Plus,
        Profile::Dev,
    );
    assert!(
        outcome(&checked)
            .iter()
            .any(|line| line.contains("exceeds the bound")
                && line.contains("trigger_date + 4.months = 2024-05-31")),
        "{:#?}",
        outcome(&checked)
    );
}

/// `timed`, due 2011-01-24: a motion at least 10 days before the due date
/// as last extended licenses a good-cause extension, a later one only an
/// extraordinary one, a denial bars both, and a motion is spent by the
/// extension it licenses (Sup. Ct. R. 13.5, 30.2, 30.3).
#[test]
fn timed_motions_denials_and_spent_motions() {
    let rows: &[(&str, &str, &[&str])] = &[
        (
            "a motion 10 days before the due date is timely for good cause",
            "2011-03-01",
            &[
                ENTERED,
                "2011-01-14 MovedTimed() by p",
                "2011-01-16 OrderedTimed(2011-02-01) by c cite \"R 13.5\"",
            ],
        ),
        (
            "a later motion licenses only the extraordinary grant",
            "2011-03-01",
            &[
                ENTERED,
                "2011-01-15 MovedTimed() by p",
                "2011-01-16 OrderedTimed(2011-02-01) by c cite \"R 13.5\"",
            ],
        ),
        (
            "a denial bars a renewed motion",
            "2011-03-01",
            &[
                "2011-01-03 DeniedTimed() by c",
                ENTERED,
                "2011-01-10 MovedTimed() by p",
                "2011-01-11 DeniedTimed() by c",
                "2011-01-12 MovedTimed() by p",
                "2011-01-13 OrderedTimed(2011-02-01) by c cite \"R 13.5\"",
            ],
        ),
        (
            "a denial before the trigger or after the order bars nothing",
            "2011-03-01",
            &[
                "2011-01-03 DeniedTimed() by c",
                ENTERED,
                "2011-01-10 MovedTimed() by p",
                "2011-01-12 OrderedTimed(2011-02-01) by c cite \"R 13.5\"",
                "2011-01-13 DeniedTimed() by c",
            ],
        ),
        (
            "a second extension reads a fresh motion against the extended due date",
            "2011-03-01",
            &[
                ENTERED,
                "2011-01-10 MovedTimed() by p",
                "2011-01-12 OrderedTimed(2011-02-01) by c cite \"R 13.5\"",
                "2011-01-22 MovedTimed() by p",
                "2011-01-23 OrderedTimed(2011-02-20) by c cite \"R 13.5\"",
            ],
        ),
        (
            "a motion is spent by the extension it licensed",
            "2011-03-01",
            &[
                ENTERED,
                "2011-01-10 MovedTimed() by p",
                "2011-01-12 OrderedTimed(2011-02-01) by c cite \"R 13.5\"",
                "2011-01-23 OrderedTimed(2011-02-20) by c cite \"R 13.5\"",
            ],
        ),
        (
            "the cap still runs from the prescribed due date",
            "2011-03-01",
            &[
                ENTERED,
                "2011-01-10 MovedTimed() by p",
                "2011-01-12 OrderedTimed(2011-02-01) by c cite \"R 13.5\"",
                "2011-01-22 MovedTimed() by p",
                "2011-01-23 OrderedTimed(2011-02-24) by c cite \"R 13.5\"",
            ],
        ),
    ];
    insta::assert_debug_snapshot!(table(rows));
}

#[test]
fn uncountable_instances_are_neither_breached_nor_extended() {
    let rows: BTreeMap<&str, Vec<String>> = [
        ("open at as_of", vec![SERVED_D]),
        (
            "extension",
            vec![
                SERVED_D,
                "2011-01-12 Ordered(2011-01-20) by c cite \"FRCP 6(b)(1)(A)\"",
            ],
        ),
        (
            "later_of over an uncountable instance",
            vec![SERVED_D, "2011-01-06 Served(amended) on d"],
        ),
    ]
    .into_iter()
    .map(|(name, events)| (name, run(&matter("2011-02-01", &events), &NoClock)))
    .collect();
    insta::assert_debug_snapshot!(rows);
}

#[test]
fn bearers_no_party_plays_and_late_floors() {
    // No party plays `Defendant`: the answer period is unbound, and lapses.
    let text = matter("2011-02-01", &[SERVED_D]).replace(
        "  d: defendant Corporation\n  e: defendant Corporation\n",
        "",
    );
    let rows: BTreeMap<&str, Vec<String>> = [
        (
            "no party plays the bearer",
            run(&text.replace(" on d", " by c"), &Plus),
        ),
        (
            "unbound: an amendment absorbs the unbound answer",
            run(
                &text.replace(
                    "  2011-01-04 Served(complaint) on d\n",
                    "  2011-01-04 Served(complaint) by c\n  2011-01-06 Served(amended) by c\n",
                ),
                &Plus,
            ),
        ),
        (
            "an answer after the extended date names the extension",
            run(
                &matter(
                    "2011-02-01",
                    &[
                        SERVED_D,
                        "2011-01-12 Ordered(2011-01-20) by c cite \"FRCP 6(b)(1)(A)\"",
                        "2011-01-22 Answered(complaint) by d",
                    ],
                ),
                &Plus,
            ),
        ),
        (
            "a late answer to an amendment names its `later_of` floor",
            run(
                &matter(
                    "2011-02-01",
                    &[
                        SERVED_D,
                        "2011-01-06 Served(amended) on d",
                        "2011-01-20 Answered(amended) by d",
                    ],
                ),
                &Plus,
            ),
        ),
    ]
    .into_iter()
    .collect();
    insta::assert_debug_snapshot!(rows);
}

/// A matter resolved from source, with event `index`'s arguments replaced
/// by `args`, checked as HIR (arguments the resolver would reject).
fn with_args(events: &[&str], index: usize, args: Vec<litigate_sema::hir::Expr>) -> Vec<String> {
    use litigate_sema::case::{check as check_hir, resolve, Env};
    use litigate_sema::hir::ExprKind;
    let refs: Vec<&Package> = omega().iter().collect();
    let lock = lock(omega());
    let resolved = resolve(
        "cases/t.case",
        &matter("2011-02-01", events),
        &refs,
        Some(&lock),
    );
    let mut case = resolved
        .case
        .unwrap_or_else(|| panic!("{:#?}", resolved.diagnostics));
    if let ExprKind::Call { args: a, .. } = &mut case.events[index].action.kind {
        *a = args;
    }
    let checked = check_hir(
        case,
        &resolved.spans,
        Env {
            packages: &refs,
            lock: Some(&lock),
            clock: &Plus,
            profile: Profile::Dev,
        },
    );
    let mut out = outcome(&checked);
    out.extend(checked.obligations.iter().map(instance));
    out
}

#[test]
fn malformed_effect_arguments_do_nothing() {
    use litigate_sema::hir::{Expr, ExprKind, Type, Value};
    let number = Expr {
        ty: Type::Number,
        kind: ExprKind::Literal {
            value: Value::Number(litigate_sema::hir::Rational::integer(3)),
        },
        span: None,
    };
    let rows: BTreeMap<&str, Vec<String>> = [
        (
            "an extension whose date is not a date",
            with_args(
                &[
                    SERVED_D,
                    "2011-01-05 Ordered(2011-01-20) by c cite \"FRCP 6(b)\"",
                ],
                1,
                vec![number],
            ),
        ),
        (
            "a removal with no argument",
            with_args(
                &["2011-01-05 Struck(answer) by c cite \"FRCP 12(f)\""],
                0,
                Vec::new(),
            ),
        ),
        (
            "an election with no argument",
            with_args(
                &["2011-01-05 Elected(answer) by c cite \"FRCP 12(f)\""],
                0,
                Vec::new(),
            ),
        ),
    ]
    .into_iter()
    .collect();
    insta::assert_debug_snapshot!(rows);
}
