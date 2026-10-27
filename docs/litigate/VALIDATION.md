# Validation: the deadline domain

Status: lg-fq2.5, measured 2026-10-27 on the corpus of lg-fq2.2 (rule
history) and lg-fq2.3 (rulings); D1 fixed and the rates re-measured by
lg-fq2.6 the same day, whose change files replay the rulings again through
separately versioned layers (CHANGE-REPORT.md). Scope: federal deadline computation,
FRCP 6 and FRAP 26 and the periods they count (lg-fq2; CHANGE.md). Every
number below is checked by `crates/litigate-cli/tests/change_validation.rs`
against the tests themselves, so this file cannot state a rate the tests
do not give.

## 1. Method

- **The rules in force that day.** A test gives each event on the date the
  source states and counts the period under the versions in force on its
  trigger date (SPEC §5.4): the period's own (FRCP 59(e) is 10 days before
  December 1, 2009 and 28 after) and lit/core's FRCP 6 or FRAP 26 (the
  short-period rule, the service days, the holidays of that year). Nothing
  is chosen per test.
- **Where.** `docs/litigate/validation/` (package `change-validation`):
  `note-2009.lit` (the Notes' examples), `rulings.lit` (one test per
  ruling, named by its `MANIFEST.json` id), `splits.lit` (the circuit
  split); `cases/` the Bowles matter and world replayed before and after
  an overruling (§4). What a `.lit` test cannot state, a time of day or an
  inaccessible clerk's office, the harness counts on the clock IR the same
  packages lower to (`litigate_lower`, then `litigate_engine::clock`).
  `litigate test docs/litigate/validation` runs the golden tests; `cargo
  nextest run -p litigate-cli --test change_validation` runs everything.
- **Agreement.** A ruling agrees when the model's due date is the court's
  (or, where the opinion states no date, the corpus's DERIVED one, marked
  below) and the filing is timely in the model exactly when the court held
  it timely: an untimely filing is `deadline(d) < FILED`, a timely one
  `deadline(d) >= FILED`. A Note example agrees when the model gives the
  Note's day (and, for an hours period, its time).
- **Disagreements** (§5) are modelling errors or source ambiguities, each
  with its evidence, never a parameter tuned to a ruling. A disagreement
  the model can compute is pinned by a test named `DISAGREES Dn` that holds
  what the model does today, so a fix fails it and must update this file.

## 2. The 2009 Committee Note examples

The Notes to FRCP 6 and FRAP 26 as amended December 1, 2009
(`sources/change-corpus/rules/committee-notes/note-civ-2009.txt`,
`note-app-2009.txt`; printed in the 2011 edition of 28 U.S.C. App.,
`sources/usc-28-app-time-history.txt`). The Notes illustrate the 2009 rules
on 2007-2008 dates, before the rules took effect, and a test counts under
the version in force on its trigger date. So each example also runs on the
first later dates with the same calendar, under the 2009 rule: September 1,
2012 is a Saturday before Labor Day as September 1, 2007 was; November 1,
2012 a Thursday as November 1, 2007 was; November 2, 2012 a Friday before
the end of daylight saving time as November 2, 2007 was; April 21, 2014
Patriot's Day, a Monday, as April 21, 2008 was. Where the old rule gives
the same day (C6, A12) the Note's own dates run too.

| Id | Example | Where | Result |
|---|---|---|---|
| C1 | "no later than November 1, 2007": Rule 6(a) does not govern a date certain | `note-2009.lit` | agrees (see D4) |
| C2 | former rule: a 10-day and a 14-day period usually end the same day, and the 10-day one "not infrequently" later | `note-2009.lit` | agrees: March 3, 2008 (both March 17); November 4, 2008 (Veterans Day: November 19 against 18) |
| C3 | the 14th day after a Monday is a Monday | `note-2009.lit` | agrees |
| C4 | 72 hours from 10:23 a.m. Friday, November 2 run to 9:23 a.m. Monday, November 5 (daylight time ends) | harness | agrees: 72 elapsed hours, 14:23 UTC to 14:23 UTC |
| C5 | an hours period ending 2:17 p.m. on a weekend or holiday runs to 2:17 p.m. on the next open day | harness | agrees: 48 hours from Saturday, September 1, 2012, 2:17 p.m. run to Tuesday, 2:17 p.m. |
| C6 | 30 days after an event, the 30th day Saturday, September 1: due Tuesday, September 4 (Labor Day) | `note-2009.lit` | agrees, on 2007 (the 2005 rule) and 2012 dates |
| C7 | 21 days before an event, the 21st day Saturday, September 1: due Friday, August 31 | `note-2009.lit` | agrees |
| C8 | clerk's office inaccessible August 31: forward to Tuesday, September 4 | harness | agrees (see D6) |
| C9 | 14 days after an event, the 14th day Patriot's Day: due Tuesday | `note-2009.lit` | agrees, on a fixture rule with the state holiday (D3) |
| C10 | 14 days before an event, the 14th day Patriot's Day: due that Monday | `note-2009.lit` | agrees |
| C11 | clerk's office inaccessible on that Monday: forward to Tuesday | harness | agrees |
| A1 | as C1 | `note-2009.lit` | agrees |
| A2 | as C2 | `note-2009.lit` | agrees |
| A3 | as C3 | `note-2009.lit` | agrees |
| A4 | 72 hours from 10:00 a.m. run to 9:00 a.m. | harness | agrees |
| A5 | as C5 | harness | agrees |
| A6 | 10 days after an event, the 10th day Saturday, September 1: due Tuesday, September 4 | `note-2009.lit` | agrees |
| A7 | 10 days before: due Friday, August 31 | `note-2009.lit` | agrees |
| A8 | as C8 | harness | agrees (see D6) |
| A9 | as C9, FRAP 26(a)(6)(C) | `note-2009.lit` | agrees |
| A10 | as C10 | `note-2009.lit` | agrees |
| A11 | as C11 | harness | agrees |
| A12 | 26(c): served by mail Thursday, November 1, 2007; 30 days end Monday, December 3; plus three days, Thursday, December 6 | `note-2009.lit` | agrees, on 2007 (the 2002 rule) and 2012 dates |
| A13 | 26(a)(4)(C): an inmate's last day ends when the institution's legal mail closes (5:00 p.m.) | none | cannot be stated (D5) |
| A14 | 26(a)(4)(C): a drop-box filer's last day ends at the box's last scheduled pickup | none | cannot be stated (D5) |

## 3. The timeliness rulings

The 35 validation rulings of `sources/change-corpus/rulings/MANIFEST.json`.
"Court" is the court's own computed last day; † marks a date the opinion
does not state and the corpus DERIVED. "Model" is the due date the model
counts under the rules in force on the trigger date.

| Ruling | Period (rule in force) | Trigger | Court | Model | Filed | Court held | Agrees |
|---|---|---|---|---|---|---|---|
| lashley-5cir-1975 | FRAP 4(a) 30 days (FRAP 26, 1968) | 1975-04-24 | 1975-05-27 | 1975-05-27 | 1975-05-28 | late | yes (D7) |
| cavaliere-11cir-1993 | FRCP 59(b) 10 days (FRCP 6, 1985) | 1992-03-24 | 1992-04-07 | 1992-04-07 | 1992-04-09 | late | yes |
| parker-10cir-1996 | 59(e) 10 days (1985) | 1994-08-01 | 1994-08-15 † | 1994-08-15 | 1994-08-18 | late | yes |
| derrington-bey-dccir-1994 | 59(e) 10 days (1985) | 1993-07-01 | 1993-07-16 | 1993-07-16 | 1993-07-19 | late | yes |
| hong-5cir-1997 | clerk's letter, 15 days (FRAP 26, 1996) | 1997-07-28 | 1997-08-12 † | 1997-08-12 | 1997-08-15 | late | yes |
| halicki-5cir-1998 | 59(e) 10 days (1985) | 1997-03-05 | 1997-03-19 † | 1997-03-19 | 1997-03-21 † | late | yes |
| midwest-employers-5cir-1998 | 59(b) 10 days (1985) | 1997-10-15 (dissent: 10-16) | 1997-10-29 † (10-30) | 1997-10-29 (10-30) | 1997-11-03 | late | yes, on either entry date |
| albright-3cir-2001 | 59(e) 10 days (1985) | 2000-11-13 | 2000-11-28 † | 2000-11-28 | 2000-11-30 | late | yes |
| jackson-v-crosby-11cir-2004 | 59(e) 10 days (2001); notice of appeal 30 days | 2003-12-15 | 2003-12-30; 2004-01-14 | 2003-12-30; 2004-01-14 | 2004-01-05 | late | yes (D8) |
| kyle-9cir-1994 | E.D. Cal. L.R. 293(a) 30 days (1985) | 1992-05-21 | 1992-06-22 | 1992-06-22 | 1992-06-24 | late | yes |
| bowles-scotus-2007 | FRAP 4(a)(6) 14 days (FRAP 26, 2002) | 2004-02-10 | 2004-02-24 | 2004-02-24 | 2004-02-26 | late | yes |
| ultimate-appliance-6cir-2010 | FRAP 4(a)(1), then 4(a)(5)(A)(i) 30 days (2002) | 2008-09-30 | 2008-10-30; 2008-12-01 | 2008-10-30; 2008-12-01 | 2008-12-02 | late | yes |
| benavides-dccir-1996 | FRAP 4(a)(6) 7 days (FRAP 26, 1989) | 1993-03-12 | 1993-03-19 | 1993-03-19 | 1993-03-19 | timely | yes |
| ryan-2cir-1999 | 4(a)(6) 7 days (1996) | 1998-06-30 | 1998-07-07 | 1998-07-07 | 1998-07-30 | late | yes |
| bass-5cir-2000 | 4(a)(6) 7 days (1996) | 1999-05-28 | timely (06-02 if May 26) | 1999-06-04 | 1999-06-03 | timely | yes |
| wilkens-5cir-2001 | 4(a)(6) 7 days (1996) | 1999-09-07 | 1999-09-14 † | 1999-09-14 | 1999-10-04 | late | yes |
| vencor-11cir-2002 | 4(a)(6) 180 days (1996) | 1999-10-26 | 2000-04-24 † | 2000-04-24 | 2000-10-17 | late | yes |
| flanagan-5cir-1998 | AEDPA grace year (FRCP 6, 1985) | 1996-04-24 | 1997-04-24 | 1997-04-24 | 1997-04-24 | timely | yes (D1) |
| hernandez-4cir-2000 | AEDPA grace year, tolled (1985) | 1996-04-24 | 1997-04-24; 1997-09-09 | 1997-04-24; 1997-09-09 | 1997-09-09 | timely | yes (D1) |
| patterson-9cir-2001 | § 2244(d) year after tolling (1985) | 1997-06-19 | 1998-06-19 | 1998-06-19 | 1998-06-19 | timely | yes (D1) |
| moore-8cir-1999 | AEDPA grace year (1985) | 1996-04-24 | 1997-04-24 | 1997-04-24 | 1997-04-24 | timely | yes (D1) |
| mickens-2cir-1998 | AEDPA grace year (1985) | 1996-04-24 | 1997-04-24 | 1997-04-24 | 1997-04-23 | timely | yes (D1) |
| burns-3cir-1998 | AEDPA grace year (1985) | 1996-04-24 | "on or before April 23" (dictum) | 1997-04-24 | 1997-04-22 | timely | yes, on the holding (§4) |
| fletcher-5cir-2000 | 42 U.S.C. § 405(g) 60 days (1985) | 1997-02-25 | 1997-05-07, assuming tolling | 1997-04-28 | 1997-05-08 | late | yes (D9) |
| eastman-3cir-2013 | FRCP 23(f) 14 days (FRCP 6, 2009) | 2013-07-31 | 2013-08-14 | 2013-08-14 | 2013-08-19 | late | yes |
| serna-5cir-2013 | 15 U.S.C. § 1692k(d) one year (2009) | 2010-08-14 | 2011-08-15 | 2011-08-15 | 2011-08-12 | timely | yes (D9) |
| justice-7cir-2012 | 59(e) 28 days (2009) | 2011-10-25 | 2011-11-22 | 2011-11-22 | 2011-11-23 | late | yes |
| lorenzo-4cir-2015 | 23(f) 14 days (2009) | 2014-03-24 | 2014-04-07 † | 2014-04-07 | 2014-04-10 | late | yes |
| ramos-lopez-5cir-2016 | 8 U.S.C. § 1252(b)(1) 30 days (FRAP 26, 2009) | 2015-01-30 | 2015-03-02 | 2015-03-02 | 2015-03-03 | late | yes |
| organic-cannabis-9cir-2020 | I.R.C. § 6213(a) 90 days (FRCP 6, 2009) | 2015-01-22 | 2015-04-22 | 2015-04-22 | 2015-04-23 | late | yes |
| pierce-6cir-2021 | FRAP 4(a)(1) 30 days (FRAP 26, 2016) | 2020-08-12 | 2020-09-11 | 2020-09-11 | 2020-09-11 | timely | yes |
| goncalves-1cir-2025 | § 1252(b)(1) 30 days (2016) | 2024-04-17 | 2024-05-17 | 2024-05-17 | 2024-05-20 | late | yes |
| circuitronix-11cir-2021 | FRCP 50(b) 28 days (FRCP 6, 2016), courthouse closed July 5 | 2019-06-06 | 2019-07-08 | 2019-07-08 (harness) | 2019-07-08 | timely | yes (D2) |
| blackman-9cir-2024 | § 2244(d) one year plus 102 tolled days (2016) | 2018-11-13 | 2020-02-24 | 2020-02-24 | 2021-03-23 | late | yes |
| malek-2cir-2024 | 59(e) 28 days (2016); appeal ends Sunday 04-30 | 2023-03-31 | 2023-04-28 † | 2023-04-28 | 2023-05-05 | late | yes |

Every ruling that rejects three added days for service (Lashley, Cavaliere,
Parker, Derrington-Bey, Hong, Halicki, Midwest Employers, Albright, Jackson,
Ultimate Appliance, Eastman, Lorenzo, Ramos-Lopez) agrees for the same
reason: the period runs from entry or from a dated letter, so its trigger
carries no service method and 6(d) / 26(c) add nothing.

## 4. Circuit splits and overrulings

**`frap-4a6-notice`.** Which notice of entry opens FRAP 4(a)(6)(A)'s 7-day
window (1991-2005 text). `splits.lit` states the text once,
`reopen-window`, opened by any notice of entry (`NoticeReceived(form,
source)`), and each circuit's precedent is a layer that `interprets` it
(SPEC §5.7; `cases/precedents`): Benavides (cadc) from a written notice
from the clerk or a party (reading A), Bass (ca5) from any written notice
(reading B), Nunley (ca9) from any notice, as the text reads (reading C).
`cases/benavides.case` and `cases/bass.case` give each split case's facts
every notice it records and the motion to reopen (`cases/reopen`), and
`litigate worlds` replays each in a world per circuit
(`cases/split-CIRCUIT.world`) holding all three precedents: only the one
of that circuit binds there (the others are persuasive, L9009), so the
engine selects the reading:

| Facts | cadc (A: Benavides) | ca5 (B: Bass) | ca9 (C: Nunley) | ca3 (none binds: the text) |
|---|---|---|---|---|
| Benavides: told February 17, 1993; copy from another matter's lawyer March 12; motion March 19 | untimed: the window never opened, as held (the 180 days ran to March 30) | timely (due March 19) | late (due February 24) | late (due February 24) |
| Bass: telephone May 26, 1999; clerk's copy May 28; motion June 3 | timely (due June 4) | timely (due June 4), as held | late (due June 2, the date Bass computes for this reading) | late (due June 2) |

The outcome differs by circuit on both cases' facts, and the selection is
the engine's: a notice the binding reading does not count opens nothing
(`read-out`). A precedent that no longer binds at its circuit loses its
reading there, and the replay fails
(`the_split_cases_replay_under_the_reading_binding_at_each_circuit`). Zimmer
(8th Cir.) is a dictum, not a layer. From December 1, 2005 the rule's own
text settles the split: only formal notice of entry under Civil Rule 77(d)
opens the window, still 7 days (14 from December 1, 2009), in every
circuit; the readings, each of the 1991 text, end there (two more golden
tests; D10).

**`aedpa-grace-end`.** One side counted. The April 24 side counts the year
under FRCP 6(a) as in force in 1996-1998, to its anniversary, and each of
its rulings agrees (Flanagan, Moore, Mickens, Hernandez, Patterson;
`rulings.lit`). The April 23 side counts the effective date as the year's
first day, which no counting rule can state (D3); its statements are dicta
in cases the date did not decide (the split's own caveat), and Burns, its
one validation ruling, held a petition of April 22 timely, which agrees on
either side.

**Replay before an overruling.** Bowles v. Russell (2007) overrules
Thompson v. INS (1964) from June 14, 2007, and `invalidate`s its "unique
circumstances" default (docs/litigate/examples/change/overruling).
`cases/bowles.case` is Bowles's appeal on the opinion's facts: the order
reopening the time to appeal entered February 10, 2004, the notice of
appeal filed February 26 (the district court had allowed 17 days).
`litigate worlds cases/bowles.case --world cases/bowles.world` (the two
precedents, at the Sixth Circuit; `cases/replay` is the one step the
filing takes) replays it at an `as_of` and `known_at`:

| Replayed at | Thompson | Reopened period (FRAP 4(a)(6)) | Notice | Decided at the time |
|---|---|---|---|---|
| 2007-06-13, before the overruling | in force | due 2004-02-24 | late | "The Sixth Circuit held that the notice was untimely" (551 U.S. at 205) |
| 2007-06-14, the overruling's date | withdrawn | due 2004-02-24 | late | untimely (the Supreme Court) |

The world before the overruling reproduces the outcome decided then
(`bowles_replayed_before_the_overruling_is_late_as_the_sixth_circuit_held`).
It does not discriminate the doctrine: Thompson's excuse has no effect on
a deadline in language 1 (D3), so the notice is late in both worlds for
the same reason, and the corpus has no ruling that Thompson saved.

## 5. Disagreements and gaps

- **D1, modelling error (lit/core), fixed by lg-fq2.6.** `frcp6@1963-07-01`
  and `frcp6@1985-08-01` had no month or year provision (`end_of_month`),
  so the six AEDPA rulings of 1998-2001 (Flanagan, Hernandez, Patterson,
  Moore, Mickens, Burns), which count a year from 1996-1997 dates, were
  refused. Both versions now count a year to its anniversary, the method
  Ross v. Artuz states for Rule 6(a) ("When a statute of limitations is
  measured in years, the last day for instituting the action is the
  anniversary date of the start of the limitations period", 150 F.3d at
  103; `ross-2cir-1998`, not itself a validation ruling), and as lit/core's
  later versions count years. All six agree (`rulings.lit`;
  `d1_the_1985_rule_counts_the_aedpa_year_to_its_anniversary`). The
  February 29 reading (`end_of_month clamp`) stays UNVERIFIED: no ruling in
  the corpus counts from one.
- **D2, modelling gap (language).** SPEC §5.3 step 4 says inaccessible
  days "come from case events", but no event form states one, so neither a
  `.lit` test nor a matter can say the clerk's office was closed. The
  engine's clock applies 6(a)(3) on request; the harness counts Note
  examples C8, C11, A8, A11 and Circuitronix's closed courthouse there, and
  they agree.
- **D3, language gap, partly closed by lg-i1qw.** A binding precedent's
  reading of a deadline's trigger or counting is now a form (`interprets`,
  SPEC §5.7), applied when a world's layers merge, so a world computes
  which reading of `frap-4a6-notice` binds at a circuit (§4). Still open:
  `recharacterize` and `invalidate` are not applied when a world's layers
  merge, so Thompson's "unique circumstances" excuse, a default no
  deadline consults, cannot make a late notice timely in the world before
  Bowles. Also: no counting rule
  includes the trigger day (SPEC §5.3 step 2; L6001), so the April 23
  reading of the AEDPA split cannot be counted; and lit/core declares no
  state's holidays (FRCP 6(a)(6)(C)), so C9-C11 and A9-A11 use fixture
  rules `frcp6-patriots-day` and `frap26-patriots-day`.
- **D4, modelling error (SPEC §5.3), pinned.** The Notes: the computation
  rules "do not apply when a fixed time to act is set" (citing Violette v.
  P.A. Days, Inc., 427 F.3d 1015 (6th Cir. 2005)). SPEC §5.3 rolls a
  `designated` date under the deadline's counting rule, right for TTAB's 37
  C.F.R. § 2.196 and wrong under FRCP 6 and FRAP 26: a date certain on
  Saturday, September 1, 2012 is moved to Tuesday, September 4. The Notes'
  own example (Thursday, November 1, 2007) does not roll, so C1 and A1
  agree; `DISAGREES D4` pins the Saturday case.
- **D5, modelling gap.** Matters and tests carry dates, not times of day:
  the end of a last day (FRCP 6(a)(4), FRAP 26(a)(4)), a filing at 3 a.m.
  (Justice: a date suffices), and hours periods are counted only on the
  engine, whose instants are UTC (daylight saving time is the caller's
  conversion). A13 and A14 fix the last day's end by the filer's mail room
  or carrier, which nothing states.
- **D6, source ambiguity.** For the inaccessible August 31 the civil Note
  says the deadline moves "no later than Tuesday, September 4", the
  appellate Note "no earlier than". The rule's text gives exactly the next
  accessible day that is not a weekend or holiday, Tuesday, September 4
  when the office is open then; the model gives that day.
- **D7, source discrepancy, no effect on the outcome.** Lashley: "The
  thirtieth day from April 25 was May 25": April 25 is the first counted
  day, so the 30th is May 24, a Saturday; the model's last day as counted
  is May 24, and the court's and the model's due date is the same Tuesday,
  May 27.
- **D8, source ambiguity.** Jackson: whether Friday, December 26, 2003, the
  day the President closed executive departments, was a day "appointed as
  a holiday by the President" under former Rule 6(a). The court left it
  open (December 30 or 31; late either way). lit/core declares no such day
  (SPEC §5.3 step 8), so the model gives December 30.
- **D9, outside the computation.** Fletcher's May 7 date assumes IFP
  tolling and three days "arguendo"; the model counts the statute alone
  (April 28) and agrees on the outcome. Serna's refiling with the fee
  after the IFP denial is not modelled; the model judges the complaint
  filed with the IFP application, as the court did.
- **D10, modelling error (this package), fixed by lg-fq2.6.** FRAP
  4(a)(6)'s window to move to reopen was 14 days from December 1, 2005,
  opened by "written notice of the entry from any source" (`rulings.lit`,
  `splits.lit`). That was the published draft of the 2005 amendment, not
  the rule: the order of April 25, 2005 kept 7 days, opened only by formal
  notice under Civil Rule 77(d) (the 2005 Committee Note, "Changes Made
  After Publication"); the order of March 26, 2009 made it 14
  (`sources/change-frap-2005-2009-rule-4-excerpt.txt`). No ruling counts a
  window opened after 2005, so no ruling's outcome changes.

## 6. Rates

- 2009 Committee Note examples: 23 of 25 agree, 0 disagree, 2 cannot be stated.
- Timeliness rulings: 35 of 35 agree with the court, 0 disagree, 0 are refused.
- Circuit split `frap-4a6-notice`: 8 of 8 world replays give the outcome of the reading the engine selects at the circuit, differing by circuit on both split cases' facts; 3 golden tests.
- Replay before an overruling (Bowles of Thompson): 1 of 1 reproduces the outcome decided at the time.
- Circuit split `aedpa-grace-end`: the April 24 side counted, every ruling agreeing; the April 23 side not countable (D3).

Of the 35 agreeing rulings, 9 compare against a DERIVED date (†), where the
opinion holds the filing late or timely without stating the last day, and
Burns agrees on its holding only: its April 23 is a dictum of the other
side of the split (§4).
