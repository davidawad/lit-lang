# Timeliness rulings corpus (lg-fq2.3)

Primary-source opinions deciding whether a federal filing was timely under FRCP 6 / FRAP 26 time computation (and the periods that feed it), collected to validate later: the deadline computed under the rules in force on the relevant dates must match the court. Machine-readable record: `MANIFEST.json` (per entry: case, court, docket, decided, citation, source URL, retrieved_at UTC, sha256 of every stored file, rule and version in force, trigger event and date, service method, filing date, court-computed deadline, holding, pin cites, notes). Fields the opinion does not state, and that the researcher computed, are listed in the entry's `derived` array and written "DERIVED" in the text.

## Counts

- 35 validation rulings: 24 with events before the Dec. 1, 2009 amendments, 5 with events between 2009 and the Dec. 1, 2016 amendments, 6 with events after Dec. 1, 2016 (decided 2020-2025, plus Organic Cannabis whose 2015 events were decided in 2020).
- 7 further entries are kept only as the opposing or supporting opinions of a circuit split (`counts_as_validation_ruling: false`).
- 2 circuit splits, confirmed from the opinions themselves (below).
- 10 Supreme Court criminal decisions lit/frcrimp and lit/frcp cite (lg-wvlf.6, `counts_as_validation_ruling: false`): stored and their positions confirmed, below; none is a timeliness ruling.

## Sources and provenance

- Reporter-paginated text (most entries): Caselaw Access Project static files (`static.case.law`), the official-reporter text with star pages; `.html` is the stored artefact, `.json` the CAP metadata, `.txt` a tag-stripped copy with `[*N]` reporter page markers used for pin cites. Pin cites are reporter pages.
- Modern opinions without CAP coverage (decided after 2018): the court-issued PDF from CourtListener storage (`storage.courtlistener.com`), found through the authenticated CourtListener v4 search API; pin cites are slip-opinion pages (PDF pages).
- Five opinions also keep the CourtListener PDF as `*.cl.pdf` (the 2010-2016 ones). Bowles is the GPO U.S. Reports volume.
- Every stored file has a sha256 in the manifest; `retrieved_at` is the file download time (UTC).
- No treatises or secondary summaries were used as input. The API key is not recorded anywhere in this repository.

## Rulings

| id | court | decided | trigger | filed | computed deadline (as the court or DERIVED) | holding |
| --- | --- | --- | --- | --- | --- | --- |
| lashley-5cir-1975 | 5th Cir. | 1975-09-08 | 1975-04-24 | notice of appeal mailed 1975-05-28, r... | 1975-05-27: day 30 counted from Apr 25 is Sun May 25, May 26 is Mem... | untimely (jurisdictional |
| cavaliere-11cir-1993 | 11th Cir. | 1993-07-30 | 1992-03-24 | motion for new trial served 1992-04-09 | 1992-04-07 (district court computation excluding intermediate Satur... | untimely by two days |
| parker-10cir-1996 | 10th Cir. | 1996-03-06 | 1994-08-01 | Rule 59(e) motion filed 1994-08-18 (1... | not stated as a date. Opinion footnote: day of entry and intermedia... | untimely |
| derrington-bey-dccir-1994 | D.C. Cir. | 1994-11-22 | 1993-07-01 (Thursday) | first reconsideration motion served 1... | Friday 1993-07-16: Monday July 5, 1993 was a legal holiday; with th... | untimely |
| hong-5cir-1997 | 5th Cir. | 1997-12-09 | 1997-07-28 | transcript ordered and paid 1997-08-1... | not stated as a date; 15-day period from the letter date. DERIVED: ... | untimely |
| halicki-5cir-1998 | 5th Cir. | 1998-09-01 | 1997-03-05 | Rule 59(e) motion mailed on the tenth... | not stated as a date. DERIVED: 1997-03-19 (10 counted days, weekend... | untimely |
| midwest-employers-5cir-1998 | 5th Cir. | 1998-11-24 | 1997-10-15 | motion for new trial filed 1997-11-03 | not stated as a date. DERIVED: 1997-10-29 (from Oct 15) or 1997-10-... | untimely |
| albright-3cir-2001 | 3d Cir. | 2001-12-06 | 2000-11-13 | motion for reconsideration filed 2000... | ten counted days, excluding weekends and the Thanksgiving holiday; ... | untimely |
| jackson-v-crosby-11cir-2004 | 11th Cir. | 2004-07-09 | 2003-12-15 (Monday) | Rule 59(e) motion filed 2004-01-05; n... | Tuesday 2003-12-30 for the Rule 59(e) motion (ten days not counting... | untimely |
| kyle-9cir-1994 | 9th Cir. | 1994-01-04 | 1992-05-21 | motion for attorney fees filed 1992-0... | 30th day was Saturday 1992-06-20, so deadline Monday 1992-06-22 (Ru... | untimely by two days |
| bowles-scotus-2007 | U.S. | 2007-06-14 | 2004-02-10 | notice of appeal filed 2004-02-26 | 2004-02-24 (14 days); the district court had told him 2004-02-27 | untimely |
| ultimate-appliance-6cir-2010 | 6th Cir. | 2010-04-01 | 2008-09-30 | motion for extension of time to appea... | notice of appeal due 2008-10-30; extension motion due 2008-12-01 | untimely |
| benavides-dccir-1996 | D.C. Cir. | 1996-04-02 | 1993-03-12 (written c... | notice of appeal and Rule 4(a)(6) mot... | 7 days from 1993-03-12 = 1993-03-19; the oral notice of 1993-02-17 ... | timely |
| ryan-2cir-1999 | 2d Cir. | 1999-05-14 | 1998-06-30 (judgment ... | motion to reopen filed 1998-07-30 | 7 days after 1998-06-30 = 1998-07-07 | untimely |
| bass-5cir-2000 | 5th Cir. | 2000-05-23 | 1999-05-28 (written c... | motion to reopen filed 1999-06-03 | if triggered 05-26, due 1999-06-02 (court computation); triggered 0... | timely |
| wilkens-5cir-2001 | 5th Cir. | 2001-01-04 | 1999-09-07 (judgment ... | motion for leave to file late notice ... | 7 days after 1999-09-07 = 1999-09-14 (DERIVED) | untimely |
| vencor-11cir-2002 | 11th Cir. | 2002-01-24 | 1999-10-26 | Rule 60(b) motion (treated as a 4(a)(... | 180 days after 1999-10-26 = Sun 2000-04-23, so Mon 2000-04-24 (DERI... | untimely |
| flanagan-5cir-1998 | 5th Cir. | 1998-09-01 | 1996-04-24 | habeas petition filed 1997-04-24 | 1997-04-24 (Rule 6(a) excludes April 24, 1996; April 23, 1997 view ... | timely |
| hernandez-4cir-2000 | 4th Cir. | 2000-08-28 | 1996-04-24 | federal habeas petition filed 1997-09-09 | limitations period ran through 1997-04-24 under Rule 6(a), leaving ... | timely |
| patterson-9cir-2001 | 9th Cir. | 2001-05-30 | 1997-06-19 | federal habeas petitions filed 1998-0... | 1998-06-19: excluding the day of denial, the clock restarted 1997-0... | timely (district court had held it one day late) |
| moore-8cir-1999 | 8th Cir. | 1999-04-16 | 1996-04-24 | section 2255 motion placed in prison ... | 1997-04-24 | timely |
| mickens-2cir-1998 | 2d Cir. | 1998-06-24 | 1996-04-24 | section 2255 motion filed 1997-04-23 | 1997-04-24 | not time-barred |
| burns-3cir-1998 | 3d Cir. | 1998-01-09 | 1996-04-24 | petition delivered to prison official... | "motions on file on or before April 23, 1997" are timely (dictum on... | timely: petition delivered to prison officials ... |
| fletcher-5cir-2000 | 5th Cir. | 2000-04-21 | 1997-02-20 (presumed ... | civil action treated as filed on paym... | 1997-05-07 even assuming IFP tolling and three days under Rule 6(e) | untimely |
| eastman-3cir-2013 | 3d Cir. | 2013-12-04 | 2013-07-31 | Rule 23(f) petition filed 2013-08-19 | 2013-08-14 (fourteen calendar days; Saturdays, Sundays and holidays... | untimely, petition dismissed |
| serna-5cir-2013 | 5th Cir. | 2013-10-07 | 2010-08-14 | complaint and IFP application 2011-08... | Sunday 2011-08-14 rolls to Monday 2011-08-15 (6(a)); 6(d) adds 3 da... | timely (Smith, J., dissents on the accrual ques... |
| justice-7cir-2012 | 7th Cir. | 2012-06-05 | 2011-10-25 | 2011-11-23 at 3 AM Central | 2011-11-22 (28 days) | untimely |
| lorenzo-4cir-2015 | 4th Cir. | 2015-11-24 | 2014-03-24 | Rule 23(f) petition filed 2014-04-10 ... | 14 days after 2014-03-24; the opinion states 17 days > 14 (2014-04-... | untimely, petition dismissed |
| ramos-lopez-5cir-2016 | 5th Cir. | 2016-05-24 | 2015-01-30 | petition for review received 2015-03-03 | 30 days from 2015-01-30 is Sunday 2015-03-01, so Monday 2015-03-02 ... | untimely |
| pierce-6cir-2021 | 6th Cir. | 2021-02-04 | 2020-08-12 | paper notice of appeal placed in the ... | Friday 2020-09-11 (30 days from 2020-08-12) | timely |
| goncalves-1cir-2025 | 1st Cir. | 2025-05-20 | 2024-04-17 | received by the clerk 2024-05-20 | 2024-05-17 (30 days after 2024-04-17); petitioner concedes receipt ... | untimely |
| circuitronix-11cir-2021 | 11th Cir. | 2021-04-08 | 2019-06-06 | renewed Rule 50(b) motion filed Monda... | 28 days ran to Thursday 2019-07-04 (legal holiday), rolling to Frid... | timely |
| blackman-9cir-2024 | 9th Cir. | 2024-11-29 | 2018-11-13 | federal habeas petition filed 2021-03-23 | limitations began 2018-11-14; plus 102 tolled days (2019-04-21 to 2... | untimely (more than a year late) |
| organic-cannabis-9cir-2020 | 9th Cir. | 2020-06-18 | 2015-01-22 | petitions sent by FedEx, delivered 7:... | 2015-04-22 (90 days after 2015-01-22) | untimely |
| malek-2cir-2024 | 2d Cir. | 2024-09-11 | 2023-03-31 | Rule 59(e) motion filed on the docket... | Friday 2023-04-28 (28 days after 2023-03-31); fn. 10 also computes ... | untimely |

## Circuit splits

### fraP-4a6-notice

What counts as "notice of the entry" that opens the short window (7 days in the 1991-2005 text) to move to reopen the appeal time under FRAP 4(a)(6): only written notice from the clerk or a party, any written notice, or actual notice of any kind?

- A: written notice from the clerk or a party required: benavides-dccir-1996, scott-harris-1cir-1997, avolio-2cir-1994, bass-5cir-2000
- B: any written notice received counts, whoever sends it, no Rule 77(d) mailing needed: wilkens-5cir-2001, ryan-2cir-1999
- C: actual notice (e.g. counsel viewing the docket) counts: nunley-9cir-1995, zimmer-8cir-1994

Acknowledged in: bass-5cir-2000, wilkens-5cir-2001, benavides-dccir-1996.

Bass (5th Cir. 2000) surveys the disagreement expressly ("Some circuits have expressly held ... On the other hand, the Eighth and Ninth Circuits speak in terms of actual notice"). Wilkens (5th Cir. 2001) says "we disagree with the portion of the Benavides opinion" requiring notice from the court or a party. Benavides distinguishes the Eighth and Ninth Circuit positions. Outcome differs: the same facts (a lawyer learning of the order from someone other than the clerk or a party) open the window in B and C but not in A.

### aedpa-grace-end

When did the one-year grace period for pre-AEDPA convictions (28 U.S.C. 2244(d), 2255) end: April 23, 1997 (one year counted without Fed. R. Civ. P. 6(a)) or April 24, 1997 (Rule 6(a): exclude the day of the event, last day is the anniversary)?

- April 23, 1997 (Rule 6(a) not analysed; mostly dicta): brown-v-angelone-4cir-1998, burns-3cir-1998
- April 24, 1997 (Rule 6(a) anniversary method): ross-2cir-1998, mickens-2cir-1998, flanagan-5cir-1998, moore-8cir-1999, hernandez-4cir-2000, patterson-9cir-2001
- "prior to April 24, 1997" (no Rule 6(a) analysis): miller-v-marr-10cir-1998

Acknowledged in: flanagan-5cir-1998, hernandez-4cir-2000, patterson-9cir-2001.

Flanagan (5th Cir. 1998) surveys circuits that "simply announced that the reasonable time period will expire on April 23 or April 24" and notes the Second Circuit alone had addressed Rule 6(a). Hernandez (4th Cir. 2000) overrides the Fourth Circuit's own April 23 dicta (Brown v. Angelone and later cases) because the date was outcome-determinative. Patterson (9th Cir. 2001), footnote: "The only Circuits espousing the April 23, 1997 deadline are those that have not specifically addressed the applicability of Fed. R. Civ. P. 6(a)" (naming Calderon (Beeler), Burns, Goodman, Simmonds).

Caveat: The April 23 statements are dicta in cases where the date did not decide the result; the split is a split in announced rules, resolved by about 2001 in favour of April 24. The rulings on the April 24 side (Patterson, Hernandez, Flanagan) are outcome-determinative.

## Criminal leading cases (lg-wvlf.6)

The ten Supreme Court decisions of expect batch 3, each cited in `lit/`. Stored as the other CAP entries are (`.html` as served, `.json` CAP metadata, `.txt` with `[*N]` pages); Booker also keeps its Supreme Court docket (`booker-scotus-2005.docket.html`, No. 04-104). Each manifest entry adds `lit_cites`: where `lit/` cites the decision, the position it is cited for, and whether the opinion holds it, with the pin cite.

| id | decided | holding (pin) | cited in lit/ for | confirmed |
| --- | --- | --- | --- | --- |
| arizona-v-washington-scotus-1978 | 1978-02-21 | retrial after a manifest-necessity mistrial is not barred; a deadlocked jury is the classic case (434 U.S. at 509) | frcrimp `mistrial-hung-jury` | yes |
| blockburger-scotus-1932 | 1932-01-04 | same-elements test for two offenses (284 U.S. at 304) | frcrimp `no-bill` (FRCrimP 6(f)) | no: nothing on grand juries |
| brady-scotus-1963 | 1963-05-13 | suppression of material favorable evidence upon request violates due process (373 U.S. at 87) | frcrimp `brady-giglio-disclosure` | yes; the Rule 16 note's "regardless of request" is not Brady's holding |
| burks-scotus-1978 | 1978-06-14 | reversal for insufficient evidence bars retrial (437 U.S. at 18) | frcrimp `conviction-reversed-remanded` | yes |
| davis-scotus-1973 | 1973-04-17 | a Rule 12(b)(2) waiver binds a later 2255 motion absent cause (411 U.S. at 242) | frcrimp `rule12c-waiver-trap` | yes |
| giglio-scotus-1972 | 1972-02-24 | an undisclosed promise to a key witness falls within Brady (405 U.S. at 154) | frcrimp `brady-giglio-disclosure` | yes |
| jackson-v-virginia-scotus-1979 | 1979-06-28 | any rational trier of fact standard (443 U.S. at 319) | frcrimp `rule29-motion` | yes |
| luce-scotus-1984 | 1984-12-10 | a defendant must testify to preserve a Rule 609 impeachment claim (469 U.S. at 43) | frcp motion-in-limine note (renewal under FRE 103(b)) | no: that is the 2000 FRE 103 amendment |
| mapp-scotus-1961 | 1961-06-19 | the exclusionary rule binds the states (367 U.S. at 655) | frcrimp `suppression-granted` | yes (a state case) |
| booker-scotus-2005 | 2005-01-12 | the Guidelines are advisory (543 U.S. at 245) | frcrimp `departure-variance` | yes |

No expect test is written for them yet. None decides whether a filing was timely, and none states the date of a trigger or filing that a `lit/frcrimp` period counts (verdict, sentence, judgment, notice of appeal): the opinions give argument and decision dates only. The trial and circuit dockets predate PACER, except Booker's (W.D. Wis. 2003, 7th Cir. No. 03-4225), which needs the authenticated CourtListener API. And `lit/frcrimp`'s periods and its FRCrimP 45 counting take effect in 2009-2024, so a trigger from these years has no period in force; a dated historical base layer (FRCrimP 45 and FRAP 4(b) before 2009) would be needed first.

## Gaps and cautions

- Rule text history (what FRCP 6 / FRAP 26 said on each date) is NOT verified in this corpus; `rule_version` is derived from the event dates and the rule text each opinion quotes. lg-fq2.2 owns the version history; lg-fq2.5 should check each ruling against it.
- No RECAP docket entries were needed or available: the opinions state the operative dates, except those flagged DERIVED (Parker, Hong, Halicki, Midwest Employers, Albright, Wilkens, Vencor, Lorenzo, Malek: deadline arithmetic only; Halicki: filing date). The 1990s circuit dockets are not on RECAP.
- Weekend/holiday arithmetic in the DERIVED fields uses the pre-2009 Rule 6(a) (periods under 11 days exclude intermediate weekends and holidays) for pre-2009 events; check before using as golden values.
- Several rulings turn on an argument that Rule 6(e)/6(d)/26(c) added three days. They are a coherent group (no added days when the period runs from entry of an order), not independent evidence of a split: every circuit cited agrees.
- The AEDPA split's April 23 side consists of dicta; see the split's caveat.
- Rulings 2009-2016 that exercise the e-service +3 days rule in the litigant's favour are thin (Serna only); a pre-2016 ruling applying 6(d) to ECF service would strengthen the 2016 boundary.
- Weak/partial entries: fletcher (assumes tolling and Rule 6(e) arguendo), midwest-employers (the dispute is excusable neglect; arithmetic is DERIVED), justice (Rule 6(a)(3) discussed in dictum).
- Dropped from the first landing: Vazquez-Rivera (administrative deadline) and LeGras (ERISA contract period), replaced by pure Rule 6 / FRAP 26 rulings.
