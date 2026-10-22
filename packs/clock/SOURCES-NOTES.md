# Clock library: sources and interpretive choices

The counting rules and holiday calendars in this directory were ported from
the hard-coded `crates/litgraph/src/clock/{general,itc,holidays}.rs` modules
(themselves ported from `civ-pro-the-gathering`'s `src/engine/clock/`,
re-verified against the rule text 2026-09-28). The computed dates and every
trace string are unchanged; see "Changes from the hard-coded clock" for the
one deliberate exception. Field reference: `docs/PACK_SCHEMA.md`, "Clock
library".

## `us-federal-holidays.json`

Source: `5 U.S.C. § 6103` (<https://www.law.cornell.edu/uscode/text/5/6103>).

- **`us-federal-holidays`**: the eleven § 6103(a) holidays. § 6103(b): a
  holiday on Saturday is observed the preceding Friday, on Sunday the
  following Monday (`observed: nearest-weekday` on the fixed-date ones; the
  nth-weekday ones always fall on a weekday).
  - Juneteenth National Independence Day: effective from 2021-06-17 (Pub.
    L. 117-17, signed and effective that day; first observed Friday
    2021-06-18).
  - Birthday of Martin Luther King, Jr.: effective from 1986-01-01 (Pub. L.
    98-144 § 1(b): the first January 1 after the two-year period following
    enactment on 1983-11-02; first observed 1986-01-20).
  - Not modeled: the Uniform Monday Holiday Act's 1971 move of Washington's
    Birthday, Memorial Day and Columbus Day to Mondays, so dates before 1971
    use today's calendar.
- **`us-inauguration-day`**: § 6103(c), "January 20 of each fourth year after
  1965", enacted by Pub. L. 89-554 (1966-09-06), so `cycle {every: 4,
  anchorYear: 1965}` with `effective.from` 1966-09-06 (first occurrence
  1969). Only `rcfc6` uses this calendar: `RCFC 6(a)(6)(A)` names
  Inauguration Day and `FRCP 6(a)(6)` does not.
  **Interpretive choice:** § 6103(c) states the Sunday shift and is silent on
  Saturday; the calendar applies (b)'s Saturday-to-Friday shift too (next
  occurrence: Saturday 2029-01-20, observed Friday 2029-01-19).

## `frcp6.json`, `rcfc6.json`

- `FRCP 6`: <https://www.law.cornell.edu/rules/frcp/rule_6>
- `RCFC 6`: <https://www.uscfc.uscourts.gov/rules> (rules effective
  2026-07-27): text-for-text identical to FRCP 6(a)/(d) after the 2009/2016/
  2023 restyling, except the Inauguration Day holiday above.

Pipeline: exclude the trigger day (6(a)(1)(A)); count calendar days
(6(a)(1)(B)) or, for a pack-authored `court` unit, business days; add the
6(d) service extension (mail, leaving with the clerk, other consented means:
+3); roll a closed last day forward (6(a)(1)(C)) or, for a backward period,
earlier (6(a)(5)); then the 6(a)(3) clerk-inaccessibility extension. The
hours provision is 6(a)(2).

**Ordering of 6(d) and the roll.** 6(d) adds 3 days "after the period would
otherwise expire under Rule 6(a)". Following the TypeScript port (citing the
2009 and 2016 Advisory Committee Notes), the 3 days are added to the
*unrolled* count and the result is rolled once (`service` before `base` and
`roll`).

The `court` unit is not itself a numbered subsection of FRCP/RCFC/FRAP; it is
a pack-authored escape hatch for a forum whose own rule counts business
days: exclude the trigger day, then advance `n` business days.

## `frap26.json`

Source: <https://www.law.cornell.edu/rules/frap/rule_26>. 26(a)(1) day
mechanics match FRCP 6(a)(1). 26(c)'s 3-day extension is keyed to mail,
third-party commercial carrier and prison mailing system (not leaving with
the clerk or other consented means, which are FRCP/RCFC Rule 5(b)(2)(D)/(F)
methods that FRAP 25 does not define). FRAP has no hours provision and no
clerk-inaccessibility extension in the fetched text, so the rule has no
`hours` and its `clerkInaccessible` step has only `ignoredText`. The 26(c)
extension is ordered like FRCP 6(d) by direct analogy (not separately
verified against Advisory Committee commentary on FRAP 26). Governs the
Federal Circuit via Fed. Cir. R. 47.1.

## `itc210.json`

Sources: `19 CFR 210.6(a)` (<https://www.law.cornell.edu/cfr/text/19/210.6>)
incorporating `201.14(a)` (<https://www.law.cornell.edu/cfr/text/19/201.14>)
and `201.16(d)`/`(e)` (<https://www.law.cornell.edu/cfr/text/19/201.16>).

Two real differences from FRCP 6, both data here:

1. **Anchor.** 201.14(a): computation "shall begin with the first business
   day following" the event (`anchor: adjacent-business-day`, counted as day
   1), not the next calendar day.
2. **Short periods.** "When the period ... is less than 7 days, intermediate
   Saturdays, Sundays, and Federal legal holidays shall be excluded": a
   `count` period with `below: 7` counting business days.

These compound: a 3-day ITC period after a Friday event runs Mon/Tue/Wed;
the FRCP period runs Sat/Sun/Mon.

201.16(d)/(e): mail +3 domestic / +10 foreign; express +1 domestic / +5
foreign.

**Interpretive choices:**

- 201.16 does not say whether 201.14(a)'s roll is reapplied after the
  service extension. The rule re-rolls (a second `roll` step), by analogy to
  FRCP 6(d)'s ordering. Its trace text still reads "see module doc"; that
  module doc is this section.
- 201.14(a) rolls forward ("extends until the end of the next business
  day") regardless of direction, as `19 CFR 210.18(a)`'s backward-counted
  summary-determination window confirms; so both `roll` entries are
  `later`. 201.14(a) does not define a backward start point; the anchor for
  a backward count is the last business day before the trigger, by analogy.

## `ttab` and packs without a rule

The TTAB pack names `frcp6`: TTAB periods are counted in calendar days with a
weekend/holiday roll (37 C.F.R. § 2.196), the same arithmetic. The PTAB,
MPEP and FRCrimP packs name no rule yet, so their deadlines are reported as
`deadline-uncomputable` warnings rather than computed under a guessed rule.

## Changes from the hard-coded clock

A zero-day *backward* `court`-unit period whose trigger is a closed day used
to roll *forward* (to a date after the trigger) while its trace said "roll
backward". It now rolls earlier, as 6(a)(5) says and the trace always
claimed. No pack authors backward deadlines, so no `deadlines` output
changes.
