//! Verus proofs (`just verify-verus`) of the counting lemmas in design
//! review §3.7 item 6, over a model of the day-count semantics (SPEC §5,
//! FRCP 6(a)(1) and 6(d)): a day is a day number, a counting rule is its
//! set of open days, and `roll` moves a day forward to the first open one.
//!
//! The model is not the implementation. `just verify-kani` checks the
//! implementation's date arithmetic (crates/litigate-sema), and the
//! engine's interpreter is tested against these laws.
use vstd::prelude::*;

verus! {

/// Some day in `[d, d + gap)` is open.
pub open spec fn open_within(open: spec_fn(int) -> bool, d: int, gap: int) -> bool {
    exists|e: int| d <= e < d + gap && #[trigger] open(e)
}

/// Open days recur: every `gap` consecutive days hold one. A rule that
/// closes some weekdays and leaves one open has gap 7 (`weekly_gap`).
pub open spec fn bounded_gap(open: spec_fn(int) -> bool, gap: int) -> bool {
    gap >= 1 && forall|d: int| #[trigger] open_within(open, d, gap)
}

/// The first open day from `d` on, looking at most `fuel` days ahead.
pub open spec fn roll(open: spec_fn(int) -> bool, d: int, fuel: nat) -> int
    decreases fuel,
{
    if open(d) || fuel == 0 {
        d
    } else {
        roll(open, d + 1, (fuel - 1) as nat)
    }
}

/// The last open day up to `d`, looking at most `fuel` days back.
pub open spec fn roll_back(open: spec_fn(int) -> bool, d: int, fuel: nat) -> int
    decreases fuel,
{
    if open(d) || fuel == 0 {
        d
    } else {
        roll_back(open, d - 1, (fuel - 1) as nat)
    }
}

/// The rule read backwards in time.
pub open spec fn mirror(open: spec_fn(int) -> bool) -> spec_fn(int) -> bool {
    |d: int| open(-d)
}

/// What `roll` does, whatever the rule: it skips only closed days, and
/// stops on an open day or when its fuel runs out.
proof fn lemma_roll(open: spec_fn(int) -> bool, d: int, fuel: nat)
    ensures
        d <= roll(open, d, fuel) <= d + fuel,
        open(roll(open, d, fuel)) || roll(open, d, fuel) == d + fuel,
        forall|e: int| d <= e < roll(open, d, fuel) ==> !#[trigger] open(e),
    decreases fuel,
{
    if !open(d) && fuel > 0 {
        lemma_roll(open, d + 1, (fuel - 1) as nat);
    }
}

/// Rolling lands on an open day, before the gap runs out, and every day
/// it skips is closed: the result is the first open day from `d` on, so
/// a deadline is never a closed day.
pub proof fn roll_is_first_open_day(open: spec_fn(int) -> bool, gap: int, d: int)
    requires
        bounded_gap(open, gap),
    ensures
        open(roll(open, d, gap as nat)),
        d <= roll(open, d, gap as nat) < d + gap,
        forall|e: int| d <= e < roll(open, d, gap as nat) ==> !#[trigger] open(e),
{
    lemma_roll(open, d, gap as nat);
    assert(open_within(open, d, gap));
    let e = choose|e: int| d <= e < d + gap && #[trigger] open(e);
    assert(roll(open, d, gap as nat) <= e);
}

/// Rolling is monotone: a later day never rolls to an earlier one.
pub proof fn roll_is_monotone(open: spec_fn(int) -> bool, gap: int, a: int, b: int)
    requires
        bounded_gap(open, gap),
        a <= b,
    ensures
        roll(open, a, gap as nat) <= roll(open, b, gap as nat),
{
    roll_is_first_open_day(open, gap, a);
    roll_is_first_open_day(open, gap, b);
    let rb = roll(open, b, gap as nat);
    if rb < roll(open, a, gap as nat) {
        assert(!open(rb));
    }
}

/// A period of `n` days from trigger `t`, counted per FRCP 6(a)(1):
/// count every day, then roll a closed last day forward.
pub open spec fn count(open: spec_fn(int) -> bool, gap: int, t: int, n: nat) -> int {
    roll(open, t + n, gap as nat)
}

/// The deadline is an open day no earlier than `t + n`, and a longer
/// period never ends sooner.
pub proof fn count_is_open_and_monotone(
    open: spec_fn(int) -> bool,
    gap: int,
    t: int,
    n: nat,
    m: nat,
)
    requires
        bounded_gap(open, gap),
        n <= m,
    ensures
        open(count(open, gap, t, n)),
        t + n <= count(open, gap, t, n),
        count(open, gap, t, n) <= count(open, gap, t, m),
{
    roll_is_first_open_day(open, gap, t + n);
    roll_is_monotone(open, gap, t + n, t + m);
}

/// FRCP 6(d) as amended in 2005: `s` service days are added after the
/// period would otherwise end under 6(a), then a closed day rolls again.
pub open spec fn count_served(open: spec_fn(int) -> bool, gap: int, t: int, n: nat, s: nat) -> int {
    roll(open, count(open, gap, t, n) + s, gap as nat)
}

/// Adding service days after the roll gives an open day, never earlier
/// than adding them before it (the pre-2005 reading), and never earlier
/// than the unserved deadline.
pub proof fn service_days_follow_the_roll(
    open: spec_fn(int) -> bool,
    gap: int,
    t: int,
    n: nat,
    s: nat,
)
    requires
        bounded_gap(open, gap),
    ensures
        open(count_served(open, gap, t, n, s)),
        count(open, gap, t, n + s) <= count_served(open, gap, t, n, s),
        count(open, gap, t, n) <= count_served(open, gap, t, n, s),
{
    let due = count(open, gap, t, n);
    count_is_open_and_monotone(open, gap, t, n, n);
    roll_is_first_open_day(open, gap, due + s);
    roll_is_monotone(open, gap, t + n + s, due + s);
}

/// Rolling back under a rule is rolling forward under its mirror image,
/// so every forward-roll law has a backward dual.
pub proof fn roll_back_is_mirrored_roll(open: spec_fn(int) -> bool, d: int, fuel: nat)
    ensures
        roll_back(open, d, fuel) == -roll(mirror(open), -d, fuel),
    decreases fuel,
{
    assert(mirror(open)(-d) == open(d));
    if !open(d) && fuel > 0 {
        roll_back_is_mirrored_roll(open, d - 1, (fuel - 1) as nat);
        assert(-d + 1 == -(d - 1));
    }
}

/// The mirror of a rule with gap `gap` has the same gap, so the backward
/// roll lands on the last open day at most `gap - 1` days back.
pub proof fn roll_back_is_last_open_day(open: spec_fn(int) -> bool, gap: int, d: int)
    requires
        bounded_gap(open, gap),
    ensures
        open(roll_back(open, d, gap as nat)),
        d - gap < roll_back(open, d, gap as nat) <= d,
{
    let m = mirror(open);
    assert forall|x: int| #[trigger] open_within(m, x, gap) by {
        assert(open_within(open, -x - gap + 1, gap));
        let e = choose|e: int| -x - gap + 1 <= e < -x - gap + 1 + gap && #[trigger] open(e);
        assert(m(-e) == open(e));
    }
    roll_is_first_open_day(m, gap, -d);
    roll_back_is_mirrored_roll(open, d, gap as nat);
    assert(m(roll(m, -d, gap as nat)) == open(-roll(m, -d, gap as nat)));
}

/// Day of the week of day number `d` (days since 1970-01-01, a Thursday),
/// Monday = 0.
pub open spec fn weekday(d: int) -> int {
    (d + 3) % 7
}

/// A weekly rule that leaves weekday `w` open has gap 7.
pub proof fn weekly_gap(closed: spec_fn(int) -> bool, w: int)
    requires
        0 <= w < 7,
        !closed(w),
    ensures
        bounded_gap(|d: int| !closed(weekday(d)), 7),
{
    let open = |d: int| !closed(weekday(d));
    assert forall|d: int| #[trigger] open_within(open, d, 7) by {
        let e = d + (w - weekday(d) + 7) % 7;
        assert(weekday(e) == w);
        assert(open(e));
    }
}

/// Days since 1970-01-01 of `(y, m, d)` (proleptic Gregorian), the same
/// arithmetic as `days_from_civil` in crates/litigate-sema/src/types/date.rs.
pub open spec fn days_from_civil(y: int, m: int, d: int) -> int {
    let y = if m <= 2 { y - 1 } else { y };
    let era = y / 400;
    let yoe = y - era * 400;
    let mp = (m + 9) % 12;
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146097 + doe - 719468
}

/// The Gregorian calendar repeats every 400 years, which is 146,097 days,
/// a whole number of weeks: every weekday and fixed-date rule is periodic
/// with that period, so enumerating one cycle decides it for all dates.
pub proof fn gregorian_periodicity(y: int, m: int, d: int)
    ensures
        146097int % 7 == 0,
        days_from_civil(y + 400, m, d) == days_from_civil(y, m, d) + 146097,
        weekday(days_from_civil(y + 400, m, d)) == weekday(days_from_civil(y, m, d)),
{
    let y0 = if m <= 2 { y - 1 } else { y };
    assert((y0 + 400) / 400 == y0 / 400 + 1) by (nonlinear_arith);
}

fn main() {
}

} // verus!
