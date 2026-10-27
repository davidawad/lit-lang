# Validation: the deadline domain

Status: lg-fq2.5, measured 2026-10-27 on the corpus of lg-fq2.2 (rule
history) and lg-fq2.3 (rulings). Scope: federal deadline computation,
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
  split). What a `.lit` test cannot state, a time of day or an
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
| flanagan-5cir-1998 | AEDPA grace year (FRCP 6, 1985) | 1996-04-24 | 1997-04-24 | refused | 1997-04-24 | timely | refused (D1) |
| hernandez-4cir-2000 | AEDPA grace year, tolled (1985) | 1996-04-24 | 1997-09-09 | refused | 1997-09-09 | timely | refused (D1) |
| patterson-9cir-2001 | § 2244(d) year after tolling (1985) | 1997-06-19 | 1998-06-19 | refused | 1998-06-19 | timely | refused (D1) |
| moore-8cir-1999 | AEDPA grace year (1985) | 1996-04-24 | 1997-04-24 | refused | 1997-04-24 | timely | refused (D1) |
| mickens-2cir-1998 | AEDPA grace year (1985) | 1996-04-24 | 1997-04-24 | refused | 1997-04-23 | timely | refused (D1) |
| burns-3cir-1998 | AEDPA grace year (1985) | 1996-04-24 | "on or before April 23" (dictum) | refused | 1997-04-22 | timely | refused (D1) |
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
window (1991-2005 text). `splits.lit` states each reading as its own period
on the notice it counts (A: written, from the clerk or a party; B: any
written notice; C: actual notice), gives each split case's facts every
notice it records, and expects, per circuit, the outcome under the reading
that circuit's precedent adopts:

| Facts | cadc (A: Benavides) | ca5 (B: Bass, Wilkens) | ca8 / ca9 (C: Zimmer, Nunley) |
|---|---|---|---|
| Benavides: told February 17, 1993; copy March 12; motion March 19 | timely (180 days to March 30) | timely (due March 19) | late (due February 24) |
| Bass: telephone May 26, 1999; clerk's copy May 28; motion June 3 | timely (due June 4) | timely (due June 4), as held | late (due June 2, the date Bass computes for this reading) |

The outcome differs by circuit on both cases' facts. The selection of the
reading per circuit is the test's, not the engine's (D3). From December 1,
2005 the rule's own text settles the split ("written notice of the entry
from any source", 14 days): the readings end there.

**`aedpa-grace-end`.** Not counted. The April 24 side counts the year under
FRCP 6(a) as in force in 1996-1998, which the model refuses (D1); the April
23 side counts the effective date as the year's first day, which no
counting rule can state (D3).

**Replay before an overruling.** Not shown. The corpus's one overruling in
the deadline domain, Bowles v. Russell (2007) of Thompson v. INS (1964),
withdraws an equitable excuse of a late notice, which a precedent cannot
state in language 1 (its row has no `grant-cap`, CHANGE.md §2.8), and the
corpus has no ruling decided under Thompson. A world replays a withdrawn
precedent before the overruling's date (`Layer::in_force_at`, tested in
`litigate-sema`), but no corpus precedent has a deadline effect for the
replay to reproduce (D3).

## 5. Disagreements and gaps

- **D1, modelling error (lit/core).** `frcp6@1963-07-01` and
  `frcp6@1985-08-01` have no month or year provision (`end_of_month`), so a
  year period counted under them is refused, never counted. The six AEDPA
  rulings of 1998-2001 (Flanagan, Hernandez, Patterson, Moore, Mickens,
  Burns) count a year from 1996-1997 dates and are refused. The rulings
  themselves state the method: "When a statute of limitations is measured
  in years, the last day for instituting the action is the anniversary
  date of the start of the limitations period" (Ross, 2d Cir. 1998, in the
  corpus as `ross-2cir-1998`); counted that way, as lit/core's 2001 version counts years, all
  six agree (`d1_the_aedpa_rulings_are_refused_under_the_1985_rule`). The
  fix belongs to the rule history (lg-fq2.6), with these rulings as its
  authority and not as its validation.
- **D2, modelling gap (language).** SPEC §5.3 step 4 says inaccessible
  days "come from case events", but no event form states one, so neither a
  `.lit` test nor a matter can say the clerk's office was closed. The
  engine's clock applies 6(a)(3) on request; the harness counts Note
  examples C8, C11, A8, A11 and Circuitronix's closed courthouse there, and
  they agree.
- **D3, language gap.** No declaration form lets a binding precedent change
  a deadline's trigger or its counting (CHANGE.md §7: `recharacterize` and
  `interpret` of a deadline have rows but no form; a precedent's only
  deadline effect is a waiver under a delegation, which neither FRAP
  4(a)(6) nor AEDPA's period grants). So a world cannot compute which
  reading binds at a circuit, and a replay before an overruling has no
  precedent effect on a deadline to reproduce. Also: no counting rule
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

## 6. Rates

- 2009 Committee Note examples: 23 of 25 agree, 0 disagree, 2 cannot be stated.
- Timeliness rulings: 29 of 35 agree with the court, 0 disagree, 6 are refused (D1).
- Circuit split `frap-4a6-notice`: 6 golden tests, the outcome differing by circuit on both split cases' facts.
- Circuit split `aedpa-grace-end` and replay before an overruling: not counted (D1, D3).

Of the 29 agreeing rulings, 9 compare against a DERIVED date (†), where the
opinion holds the filing late or timely without stating the last day.
