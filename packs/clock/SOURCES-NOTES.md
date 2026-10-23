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
(6(a)(1)(B)) or, for a pack-authored `court` unit, business days; roll a
closed last day under 6(a)(1)(C) (or earlier under 6(a)(5) for a backward
period); add the 6(d) service extension (mail, leaving with the clerk, other
consented means: +3); roll the resulting last day again; then apply the
6(a)(3) clerk-inaccessibility extension. The hours provision is 6(a)(2).

**Months and years.** FRCP 6 and RCFC 6 are silent on periods stated in
months or years (FRCP 60(c)(1)'s "a year"; the PTAB's statutory and Part 42
months, which the PTAB pack counts under `rcfc6`). Both rules carry a
`months` provision with `endOfMonth: clamp`: the period ends on the
corresponding day of the later month, or on that month's last day when it
has none, as MPEP 710.01(a) counts the Patent Office's months ("a 3-month
shortened statutory period dated November 30 is due on the following
February 28"). **Interpretive choice:** for FRCP/RCFC this is the earlier
of the two readings of a February 29 anniversary; UNVERIFIED (no rule text
fixes it). The last day then rolls under 6(a)(1)(C) like any other. Matches
`end_of_month clamp` on `lit/core`'s `frcp6` and `rcfc6`.

**Ordering of 6(d) and the rolls.** 6(d) adds 3 days "after the period would
otherwise expire under Rule 6(a)". The 6(a) period therefore includes its
initial last-day roll; then the three calendar days are added, and the new
last day is rolled again if needed. This is also explicit in the FRCP 6(d)
2005 Committee Note. FRAP 26(c) has the same ordering: its 2009 Committee
Note instructs users to calculate the prescribed period using Rule 26(a),
then add three calendar days, and roll the extension end day if necessary.
Thus each pipeline places a roll before service and another after it.

The `court` unit is not itself a numbered subsection of FRCP/RCFC/FRAP; it is
a pack-authored escape hatch for a forum whose own rule counts business
days: exclude the trigger day, then advance `n` business days.

## `frap26.json`

Source: <https://www.law.cornell.edu/rules/frap/rule_26>, vendored as the
`## FRAP 26` section of `sources/frap.txt`. 26(a)(1) day mechanics match
FRCP 6(a)(1). 26(c)'s 3-day extension is keyed to mail, third-party
commercial carrier and prison mailing system (not leaving with the clerk or
other consented means, which are FRCP/RCFC Rule 5(b)(2)(D)/(F) methods that
FRAP 25 does not define). The 26(c) extension follows the ordering stated in
the 2009 Advisory Committee Note to Rule 26: first calculate the prescribed
period using Rule 26(a), including its last-day roll, then add three
calendar days and roll the extension end day if necessary (¶¶ 135–137).
Governs the Federal Circuit via Fed. Cir. R. 47.1.

- **Hours (26(a)(2)).** `hours`: begin counting immediately on the
  triggering event and count every hour (26(a)(2)(A)-(B)); an end on a
  Saturday, Sunday or legal holiday continues to the same time on the next
  day that is not one (26(a)(2)(C)).
- **Inaccessible clerk's office (26(a)(3)).** On the last day of a
  26(a)(1) period the `clerkInaccessible` step extends to the first
  accessible day that is not a Saturday, Sunday or legal holiday
  (26(a)(3)(A)), including a period measured backward; during the last hour
  of a 26(a)(2) period,
  `hours.inaccessibleText` extends to the same time on that day
  (26(a)(3)(B)). The engine's `clerk_inaccessible` input says the office is
  inaccessible on the last day (or hour) only, so the extension is the next
  open day; an office closed for several days needs those days as
  `additional_holidays`.

An earlier version of this file said FRAP has no hours provision and no
clerk-inaccessibility extension "in the fetched text"; the vendored rule has
both (since the 2009 amendment; inaccessibility since 1989), and `lit/core`'s
`frap26` already followed it.

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

## Service methods are total

Every rule's `service` step lists each engine service method once: in
`methods` with the days it adds (`0` when the rule recognizes it and adds
nothing), or in `unrecognized` (the rule does not name it, so a period
served that way is refused). Personal delivery adds nothing under every
rule. Electronic service adds nothing under `frcp6` and `frap26` (FRCP 6(d)
and FRAP 26(c) as amended December 1, 2016), `rcfc6` (RCFC 6(d) as amended
August 1, 2017) and `itc210` (19 C.F.R. § 201.16(f)); `lit/core` declares the
earlier versions that added 3 days for it. The other methods each rule
recognizes are the ones it listed before.

## `frcrimp45.json`, `sup-ct-30.json`

Generated once from `lit/core`'s `frcrimp45` and `sup-ct-30` (the versions in
force today) and kept beside the other shared rules, as `core_stdlib`
checks: `FRCrimP 45` (`sources/frcrimp.txt`, as amended to December 1,
2023: mail, leaving with the clerk and other consented nonelectronic means
add 3 days, 45(c) with Rule 49(a)(4)(C)-(E)) and `Sup. Ct. R. 30.1`
(`sources/scotus-rules.txt`, the Rules effective January 1, 2023: no time is
added for any manner of service, Rule 29.3).

## Packs counted by their own forums' rules

The TTAB pack counts under its own `ttab-2-196` (37 C.F.R. § 2.196 with §
2.119(c), which adds no time for mail since January 14, 2017), generated
into `ttab-trademark-trial-and-appeal-board.json`; the PTAB pack under
`uspto-1-7` (37 C.F.R. § 1.7, which § 42.1(a) applies to the Board); the
FRCrimP pack under `frcrimp45`; and each certiorari window under
`sup-ct-30` (the window's own `deadline.counting`). No pack borrows another
forum's computation rule.

## Changes from the hard-coded clock

A zero-day *backward* `court`-unit period whose trigger is a closed day used
to roll *forward* (to a date after the trigger) while its trace said "roll
backward". It now rolls earlier, as 6(a)(5) says and the trace always
claimed. No pack authors backward deadlines, so no `deadlines` output
changes.
