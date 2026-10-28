# Expect corpus plan: leading decision per circuit per rule (lg-wvlf.7)

The plan for growing the expect corpus (`tests/expect/`, tests/README.md
"Expect tests") from the timeliness rulings of lg-fq2.3 to a leading
decision for every rule `lit/frcp`, `lit/frap` and `lit/frcrimp` model, in
every federal circuit, and for the circuit packages (`lit/ca1` ... `lit/ca11`,
`lit/cadc`, `lit/cafc`). This file is the plan only. The PM files one bead
per batch (section 4); no batch bead exists yet.

## 1. Scope and counts

- 37 rule rows: the FRCP, FRAP and FRCrimP provisions the three packages cite
  (counted from their `.lit` sources), plus the one circuit rule a circuit
  package models that has its own case law (Fed. Cir. R. 36).
- 45 cells are already stored in `sources/change-corpus/rulings`
  (status `corpus`); those with a `tests/expect/<forum>/<id>.toml` are
  `corpus+expect`. They are listed so the matrix is complete and are not
  re-batched.
- 233 cells are planned (status `planned`), in 23 batches of about 10.
- 230 cells are open: no leading decision is named, because none could be
  verified here. They are research, not guesses; a batch that finds one adds a
  row and verifies it the same way (section 2).

Rows run splits first (section 3.1), then the consensus rules (section 3.2),
in the order the batches take them.

## 2. Verification (never invented)

Every named decision was checked on 2026-10-28 against a reporter, not
written from memory:

- decisions in F.2d, F.3d (through 2019) and U.S. (through vol. 572):
  the Caselaw Access Project static metadata
  (`https://static.case.law/<reporter>/<volume>/CasesMetadata.json`), with
  the reporter, volume and first page, the court, the decision year and a
  party name all matching;
- later decisions (Kemp, Microsoft v. Baker, Banister, City of San Antonio,
  TecSec): the CourtListener citation lookup
  (`https://www.courtlistener.com/c/<reporter>/<volume>/<page>/`), which
  resolves the citation to a single opinion of that name;
- the `corpus` rows: their stored opinion and `MANIFEST.json` entry.

What was verified is the citation: that the decision exists under that name
in that court and year at that page. The "position" column is the planner's
reading of the holding. It is not yet verified; the batch confirms it from
the opinion text before encoding anything. A row whose opinion does not hold
what the plan says is dropped and reported in the batch, never swapped for
another case without the same verification.

## 3. Matrix

Courts: the U.S. Supreme Court row, where there is one, is the binding
anchor and the dated precedent layer every circuit world inherits. The
Federal Circuit applies regional-circuit law to procedural questions not
unique to patent law, so FRCP cells are not opened for it; it has no
criminal jurisdiction, so FRCrimP and FRAP 4(b) cells are not opened either.

### 3.1 Rules with circuit splits

#### FRAP 4(a)(6) (`lit/frap`, `frap4a6`)

What notice of entry opens the short window to move to reopen the time to appeal (clerk/party written notice, any written notice, or actual notice)? **Split:** confirmed from the opinions in sources/change-corpus/rulings, `fraP-4a6-notice`; the 2005 amendment (written notice under Rule 77(d)) is the dated layer that ends it.

| court | decision | citation | position (planner; batch confirms) | status |
|---|---|---|---|---|
| 1st Cir. | Scott-Harris v. City of Fall River | 134 F.3d 427 (1st Cir. 1997) | A: written notice from clerk or party | corpus (`scott-harris-1cir-1997`) |
| 2d Cir. | Avolio v. County of Suffolk | 29 F.3d 50 (2d Cir. 1994) | A | corpus (`avolio-2cir-1994`) |
| 2d Cir. | Ryan v. First Unum Life Ins. Co. | 174 F.3d 302 (2d Cir. 1999) | B: any written notice | corpus+expect (`ryan-2cir-1999`) |
| 5th Cir. | Bass v. U.S. Dep't of Agriculture | 211 F.3d 959 (5th Cir. 2000) | A | corpus+expect (`bass-5cir-2000`) |
| 5th Cir. | Wilkens v. Johnson | 238 F.3d 328 (5th Cir. 2001) | B | corpus+expect (`wilkens-5cir-2001`) |
| 8th Cir. | Zimmer St. Louis, Inc. v. Zimmer Co. | 32 F.3d 357 (8th Cir. 1994) | C: actual notice | corpus (`zimmer-8cir-1994`) |
| 9th Cir. | Nunley v. City of Los Angeles | 52 F.3d 792 (9th Cir. 1995) | C | corpus (`nunley-9cir-1995`) |
| 11th Cir. | Vencor Hospitals, Inc. v. Standard Life & Accident Ins. Co. | 279 F.3d 1306 (11th Cir. 2002) | 180-day outer limit; untimely | corpus (`vencor-11cir-2002`) |
| D.C. Cir. | Benavides v. Bureau of Prisons | 79 F.3d 1211 (D.C. Cir. 1996) | A | corpus+expect (`benavides-dccir-1996`) |

Open (no verified leading decision named yet; research, never fill by guess): 3d Cir., 4th Cir., 6th Cir., 7th Cir., 10th Cir., Fed. Cir.

#### FRCP 6(a) (AEDPA grace period) (`lit/frcp`, `frcp6a_aedpa`)

Does Rule 6(a) put the end of the one-year AEDPA grace period on April 24, 1997 (anniversary method) or April 23? **Split:** in announced rules (confirmed, `aedpa-grace-end`); April 23 side is dicta.

| court | decision | citation | position (planner; batch confirms) | status |
|---|---|---|---|---|
| 2d Cir. | Ross v. Artuz | 150 F.3d 97 (2d Cir. 1998) | April 24 (Rule 6(a)) | corpus (`ross-2cir-1998`) |
| 2d Cir. | Mickens v. United States | 148 F.3d 145 (2d Cir. 1998) | April 24 (Rule 6(a)) | corpus (`mickens-2cir-1998`) |
| 3d Cir. | Burns v. Morton | 134 F.3d 109 (3d Cir. 1998) | April 23 (dicta) | corpus (`burns-3cir-1998`) |
| 4th Cir. | Brown v. Angelone | 150 F.3d 370 (4th Cir. 1998) | April 23 (dicta) | corpus (`brown-v-angelone-4cir-1998`) |
| 4th Cir. | Hernandez v. Caldwell | 225 F.3d 435 (4th Cir. 2000) | April 24 (overrides own dicta) | corpus (`hernandez-4cir-2000`) |
| 5th Cir. | Flanagan v. Johnson | 154 F.3d 196 (5th Cir. 1998) | April 24 | corpus (`flanagan-5cir-1998`) |
| 8th Cir. | Moore v. United States | 173 F.3d 1131 (8th Cir. 1999) | April 24 | corpus (`moore-8cir-1999`) |
| 9th Cir. | Patterson v. Stewart | 251 F.3d 1243 (9th Cir. 2001) | April 24 | corpus (`patterson-9cir-2001`) |
| 10th Cir. | Miller v. Marr | 141 F.3d 976 (10th Cir. 1998) | "prior to April 24" (no Rule 6(a) analysis) | corpus (`miller-v-marr-10cir-1998`) |

Open (no verified leading decision named yet; research, never fill by guess): 1st Cir., 6th Cir., 7th Cir., 11th Cir., D.C. Cir. Fed. Cir.: applies regional-circuit law to procedural questions not unique to patent law, so the regional cell governs.

#### FRAP 4(a)(4)(A) with FRCP 59(e) / 6(b)(2) (`lit/frap`, `frap4a4`)

Does an untimely post-judgment motion toll the time to appeal when the opponent does not object (claim-processing forfeiture) or never? **Split:** pre- and post-Hamer; positions to confirm from the opinions.

| court | decision | citation | position (planner; batch confirms) | status |
|---|---|---|---|---|
| U.S. | Browder v. Director, Dep't of Corrections of Illinois | 434 U.S. 257 (1978) |  | planned |
| U.S. | Hamer v. Neighborhood Housing Services of Chicago | 583 U.S. 17 (2017) | time limits not in a statute are claim-processing rules | planned |
| 3d Cir. | Lizardo v. United States | 619 F.3d 273 (3d Cir. 2010) | does not toll | planned |
| 6th Cir. | National Ecological Foundation v. Alexander | 496 F.3d 466 (6th Cir. 2007) | forfeitable: unobjected untimely motion tolls | planned |
| 7th Cir. | Blue v. International Brotherhood of Electrical Workers Local 159 | 676 F.3d 579 (7th Cir. 2012) |  | planned |
| 11th Cir. | Green v. Drug Enforcement Administration | 606 F.3d 1296 (11th Cir. 2010) |  | planned |
| D.C. Cir. | Wilburn v. Robinson | 480 F.3d 1140 (D.C. Cir. 2007) |  | planned |
| D.C. Cir. | Obaydullah v. Obama | 688 F.3d 784 (D.C. Cir. 2012) | does not toll | planned |

Open (no verified leading decision named yet; research, never fill by guess): 1st Cir., 2d Cir., 4th Cir., 5th Cir., 8th Cir., 9th Cir., 10th Cir., Fed. Cir.

#### FRCP 11(c)(2) (`lit/frcp`, `frcp11`)

Does anything short of serving the motion itself (a warning letter) satisfy the 21-day safe harbor? **Split:** strict service vs. substantial compliance; positions to confirm.

| court | decision | citation | position (planner; batch confirms) | status |
|---|---|---|---|---|
| U.S. | Cooter & Gell v. Hartmarx Corp. | 496 U.S. 384 (1990) |  | corpus (`cooter-gell-scotus-1990`); not encoded: no dated filing (lg-wvlf.4) |
| 2d Cir. | In re Pennie & Edmonds LLP | 323 F.3d 86 (2d Cir. 2003) | related: sua sponte sanctions | planned |
| 2d Cir. | Star Mark Management, Inc. v. Koon Chun Hing Kee Soy & Sauce Factory, Ltd. | 682 F.3d 170 (2d Cir. 2012) |  | planned |
| 3d Cir. | In re Schaefer Salt Recovery, Inc. | 542 F.3d 90 (3d Cir. 2008) |  | planned |
| 4th Cir. | Brickwood Contractors, Inc. v. Datanet Engineering, Inc. | 369 F.3d 385 (4th Cir. 2004) | related: sua sponte sanctions | planned |
| 5th Cir. | Elliott v. Tilton | 64 F.3d 213 (5th Cir. 1995) |  | planned |
| 6th Cir. | Ridder v. City of Springfield | 109 F.3d 288 (6th Cir. 1997) | motion must be served before disposition | planned |
| 7th Cir. | Nisenbaum v. Milwaukee County | 333 F.3d 804 (7th Cir. 2003) | letter can substantially comply | planned |
| 7th Cir. | Matrix IV, Inc. v. American National Bank & Trust Co. of Chicago | 649 F.3d 539 (7th Cir. 2011) |  | planned |
| 8th Cir. | Gordon v. Unifund CCR Partners | 345 F.3d 1028 (8th Cir. 2003) |  | planned |
| 9th Cir. | Barber v. Miller | 146 F.3d 707 (9th Cir. 1998) | letter insufficient | planned |
| 9th Cir. | Radcliffe v. Rainbow Construction Co. | 254 F.3d 772 (9th Cir. 2001) |  | planned |
| 10th Cir. | Roth v. Green | 466 F.3d 1179 (10th Cir. 2006) | letter insufficient | planned |

Open (no verified leading decision named yet; research, never fill by guess): 1st Cir., 11th Cir., D.C. Cir. Fed. Cir.: applies regional-circuit law to procedural questions not unique to patent law, so the regional cell governs.

#### FRCP 4(m) (`lit/frcp`, `frcp4m`)

May the court extend the service period without good cause? **Split:** largely closed by the 1993 amendment and Henderson (dicta); the Fourth Circuit line is the outlier.

| court | decision | citation | position (planner; batch confirms) | status |
|---|---|---|---|---|
| U.S. | Henderson v. United States | 517 U.S. 654 (1996) |  | planned |
| 2d Cir. | Zapata v. City of New York | 502 F.3d 192 (2d Cir. 2007) |  | planned |
| 3d Cir. | Petrucelli v. Bohringer & Ratzinger | 46 F.3d 1298 (3d Cir. 1995) |  | planned |
| 4th Cir. | Mendez v. Elliot | 45 F.3d 75 (4th Cir. 1995) | good cause required (outlier) | planned |
| 5th Cir. | Thompson v. Brown | 91 F.3d 20 (5th Cir. 1996) |  | planned |
| 7th Cir. | Panaras v. Liquid Carbonic Industries Corp. | 94 F.3d 338 (7th Cir. 1996) |  | planned |
| 8th Cir. | Adams v. AlliedSignal General Aviation Avionics | 74 F.3d 882 (8th Cir. 1996) |  | planned |
| 9th Cir. | Efaw v. Williams | 473 F.3d 1038 (9th Cir. 2007) |  | planned |
| 10th Cir. | Espinoza v. United States | 52 F.3d 838 (10th Cir. 1995) |  | planned |
| 11th Cir. | Horenkamp v. Van Winkle & Co. | 402 F.3d 1129 (11th Cir. 2005) |  | planned |
| D.C. Cir. | Mann v. Castiel | 681 F.3d 368 (D.C. Cir. 2012) |  | planned |

Open (no verified leading decision named yet; research, never fill by guess): 1st Cir., 6th Cir. Fed. Cir.: applies regional-circuit law to procedural questions not unique to patent law, so the regional cell governs.

#### FRCP 41(a)(1) (`lit/frcp`, `frcp41a`)

May a notice or stipulation of voluntary dismissal drop fewer than all claims (or must the plaintiff amend under Rule 15)? **Split:** action/party vs. claim; positions to confirm.

| court | decision | citation | position (planner; batch confirms) | status |
|---|---|---|---|---|
| U.S. | Link v. Wabash Railroad Co. | 370 U.S. 626 (1962) |  | planned |
| 2d Cir. | Harvey Aluminum, Inc. v. American Cyanamid Co. | 203 F.2d 105 (2d Cir. 1953) | Rule 41(a) not for single claims | planned |
| 3d Cir. | Young v. Wilky Carrier Corp. | 150 F.2d 764 (3d Cir. 1945) |  | planned |
| 3d Cir. | Manze v. State Farm Insurance Co. | 817 F.2d 1062 (3d Cir. 1987) | related: effect of a pending summary-judgment motion | planned |
| 5th Cir. | Exxon Corp. v. Maryland Casualty Co. | 599 F.2d 659 (5th Cir. 1979) |  | planned |
| 6th Cir. | Philip Carey Manufacturing Co. v. Taylor | 286 F.2d 782 (6th Cir. 1961) |  | planned |
| 7th Cir. | Taylor v. Brown | 787 F.3d 851 (7th Cir. 2015) | whole action only; amend under Rule 15 | planned |
| 9th Cir. | Hells Canyon Preservation Council v. U.S. Forest Service | 403 F.3d 683 (9th Cir. 2005) | Rule 15, not Rule 41, drops claims | planned |
| 10th Cir. | Gobbo Farms & Orchards v. Poole Chemical Co. | 81 F.3d 122 (10th Cir. 1996) |  | planned |
| 11th Cir. | Perry v. Schumacher Group of Louisiana | 891 F.3d 954 (11th Cir. 2018) | whole action only | planned |

Open (no verified leading decision named yet; research, never fill by guess): 1st Cir., 4th Cir., 8th Cir., D.C. Cir. Fed. Cir.: applies regional-circuit law to procedural questions not unique to patent law, so the regional cell governs.

#### FRCP 15(c)(1)(C) (`lit/frcp`, `frcp15c`)

Is replacing a "John Doe" defendant a "mistake concerning the proper party's identity" that relates back, and how does Krupski change the test? **Split:** majority: lack of knowledge is not a mistake; positions to confirm.

| court | decision | citation | position (planner; batch confirms) | status |
|---|---|---|---|---|
| U.S. | Mayle v. Felix | 545 U.S. 644 (2005) | habeas: same "conduct, transaction, or occurrence" | planned |
| U.S. | Krupski v. Costa Crociere S.p.A. | 560 U.S. 538 (2010) |  | planned |
| 1st Cir. | Wilson v. United States Government | 23 F.3d 559 (1st Cir. 1994) |  | planned |
| 2d Cir. | Barrow v. Wethersfield Police Department | 66 F.3d 466 (2d Cir. 1995) | Doe substitution is not a mistake | planned |
| 3d Cir. | Singletary v. Pennsylvania Department of Corrections | 266 F.3d 186 (3d Cir. 2001) | questions the majority view | planned |
| 4th Cir. | Goodman v. Praxair, Inc. | 494 F.3d 458 (4th Cir. 2007) |  | planned |
| 5th Cir. | Jacobsen v. Osborne | 133 F.3d 315 (5th Cir. 1998) | not a mistake | planned |
| 7th Cir. | Hall v. Norfolk Southern Railway Co. | 469 F.3d 590 (7th Cir. 2006) |  | planned |
| 8th Cir. | Heglund v. Aitkin County | 871 F.3d 572 (8th Cir. 2017) | not a mistake (post-Krupski) | planned |
| 10th Cir. | Garrett v. Fleming | 362 F.3d 692 (10th Cir. 2004) | not a mistake | planned |
| 11th Cir. | Powers v. Graff | 148 F.3d 1223 (11th Cir. 1998) |  | planned |

Open (no verified leading decision named yet; research, never fill by guess): 6th Cir., 9th Cir., D.C. Cir. Fed. Cir.: applies regional-circuit law to procedural questions not unique to patent law, so the regional cell governs.

#### FRCrimP 12(b)(3), 12(c)(3) (`lit/frcrimp`, `frcrimp12`)

After the 2014 amendment, is an untimely Rule 12(b)(3) argument reviewed for plain error or only on good cause? **Split:** positions to confirm.

| court | decision | citation | position (planner; batch confirms) | status |
|---|---|---|---|---|
| U.S. | Davis v. United States | 411 U.S. 233 (1973) |  | planned |
| 3d Cir. | United States v. Fattah | 858 F.3d 801 (3d Cir. 2017) |  | planned |
| 5th Cir. | United States v. Vasquez | 899 F.3d 363 (5th Cir. 2018) |  | planned |
| 6th Cir. | United States v. Soto | 794 F.3d 635 (6th Cir. 2015) | plain error | planned |
| 7th Cir. | United States v. McMillian | 786 F.3d 630 (7th Cir. 2015) |  | planned |
| 8th Cir. | United States v. Anderson | 783 F.3d 727 (8th Cir. 2015) |  | planned |
| 9th Cir. | United States v. Guerrero | 921 F.3d 895 (9th Cir. 2019) |  | planned |
| 10th Cir. | United States v. Bowline | 917 F.3d 1227 (10th Cir. 2019) |  | planned |
| 11th Cir. | United States v. Sperrazza | 804 F.3d 1113 (11th Cir. 2015) | good cause, not plain error | planned |

Open (no verified leading decision named yet; research, never fill by guess): 1st Cir., 2d Cir., 4th Cir., D.C. Cir. Fed. Cir.: no criminal jurisdiction.

#### FRCrimP 35(a) (`lit/frcrimp`, `frcrimp35`)

Is the 14-day (formerly 7-day) correction period jurisdictional after Eberhart? **Split:** reported split; positions to confirm.

| court | decision | citation | position (planner; batch confirms) | status |
|---|---|---|---|---|
| 3d Cir. | United States v. Higgs | 504 F.3d 456 (3d Cir. 2007) |  | planned |
| 4th Cir. | United States v. Shank | 395 F.3d 466 (4th Cir. 2005) |  | planned |
| 7th Cir. | United States v. Wisch | 275 F.3d 620 (7th Cir. 2001) |  | planned |
| 10th Cir. | United States v. Green | 405 F.3d 1180 (10th Cir. 2005) |  | planned |

Open (no verified leading decision named yet; research, never fill by guess): 1st Cir., 2d Cir., 5th Cir., 6th Cir., 8th Cir., 9th Cir., 11th Cir., D.C. Cir. Fed. Cir.: no criminal jurisdiction.

#### FRAP 5 (Rule 23(f) petitions) (`lit/frap`, `frap5`)

Is the 14-day Rule 23(f) petition period subject to equitable tolling (e.g., by a reconsideration motion), and what standard governs the grant? **Split:** on tolling, resolved by Nutraceutical (2019): the dated precedent layer.

| court | decision | citation | position (planner; batch confirms) | status |
|---|---|---|---|---|
| U.S. | Microsoft Corp. v. Baker | 582 U.S. 23 (2017) |  | planned |
| U.S. | Nutraceutical Corp. v. Lambert | 586 U.S. 188 (2019) |  | planned |
| 1st Cir. | Mowbray v. Waste Management Holdings, Inc. | 208 F.3d 288 (1st Cir. 2000) |  | planned |
| 2d Cir. | In re Sumitomo Copper Litigation | 262 F.3d 134 (2d Cir. 2001) |  | planned |
| 3d Cir. | Gutierrez v. Johnson & Johnson | 523 F.3d 187 (3d Cir. 2008) |  | planned |
| 3d Cir. | Eastman v. First Data Corp. | 736 F.3d 675 (3d Cir. 2013) |  | corpus+expect (`eastman-3cir-2013`) |
| 4th Cir. | Lienhart v. Dryvit Systems, Inc. | 255 F.3d 138 (4th Cir. 2001) |  | planned |
| 4th Cir. | Lorenzo v. Prime Communications, L.P. | 806 F.3d 777 (4th Cir. 2015) |  | corpus+expect (`lorenzo-4cir-2015`) |
| 5th Cir. | McNamara v. Felderhof | 410 F.3d 277 (5th Cir. 2005) |  | planned |
| 6th Cir. | In re Delta Air Lines | 310 F.3d 953 (6th Cir. 2002) |  | planned |
| 7th Cir. | Gary v. Sheahan | 188 F.3d 891 (7th Cir. 1999) |  | planned |
| 9th Cir. | Lambert v. Nutraceutical Corp. | 870 F.3d 1170 (9th Cir. 2017) | equitable tolling allowed (reversed by Nutraceutical) | planned |
| 10th Cir. | Carpenter v. Boeing Co. | 456 F.3d 1183 (10th Cir. 2006) |  | planned |
| 11th Cir. | Shin v. Cobb County Board of Education | 248 F.3d 1061 (11th Cir. 2001) |  | planned |
| 11th Cir. | Jenkins v. BellSouth Corp. | 491 F.3d 1288 (11th Cir. 2007) |  | planned |
| D.C. Cir. | In re D.C. Water & Sewer Authority | 561 F.3d 494 (D.C. Cir. 2009) |  | planned |

Open (no verified leading decision named yet; research, never fill by guess): 8th Cir., Fed. Cir.

#### FRCP 60(b)(1), 60(c)(1) (`lit/frcp`, `frcp60b`)

Does "mistake" include a judge's legal error, and within what time? **Split:** resolved by Kemp (2022): the dated precedent layer.

| court | decision | citation | position (planner; batch confirms) | status |
|---|---|---|---|---|
| U.S. | Ackermann v. United States | 340 U.S. 193 (1950) |  | planned |
| U.S. | Liljeberg v. Health Services Acquisition Corp. | 486 U.S. 847 (1988) |  | planned |
| U.S. | Gonzalez v. Crosby | 545 U.S. 524 (2005) |  | corpus (`gonzalez-v-crosby-scotus-2005`); not encoded: Rule 60(b)(6) sets no counted period (lg-wvlf.4) |
| U.S. | Kemp v. United States | 596 U.S. 528 (2022) |  | planned |
| 2d Cir. | In re 310 Associates | 346 F.3d 31 (2d Cir. 2003) |  | planned |
| 3d Cir. | Page v. Schweiker | 786 F.2d 150 (3d Cir. 1986) |  | planned |
| 5th Cir. | Chick Kam Choo v. Exxon Corp. | 699 F.2d 693 (5th Cir. 1983) |  | planned |
| 6th Cir. | Pierce v. United Mine Workers of America Welfare & Retirement Fund | 770 F.2d 449 (6th Cir. 1985) |  | planned |
| 8th Cir. | Fox v. Brewer | 620 F.2d 177 (8th Cir. 1980) |  | planned |
| 9th Cir. | Liberty Mutual Insurance Co. v. EEOC | 691 F.2d 438 (9th Cir. 1982) |  | planned |
| 9th Cir. | Briones v. Riviera Hotel & Casino | 116 F.3d 379 (9th Cir. 1997) | Pioneer applies to 60(b)(1) | planned |
| 10th Cir. | Cashner v. Freedom Stores, Inc. | 98 F.3d 572 (10th Cir. 1996) |  | planned |
| 11th Cir. | Parks v. U.S. Life & Credit Corp. | 677 F.2d 838 (11th Cir. 1982) |  | planned |

Open (no verified leading decision named yet; research, never fill by guess): 1st Cir., 4th Cir., 7th Cir., D.C. Cir. Fed. Cir.: applies regional-circuit law to procedural questions not unique to patent law, so the regional cell governs.

### 3.2 Consensus rules and Supreme Court anchors

#### FRAP 4(a)(5) (`lit/frap`, `frap4a5`)

Excusable neglect or good cause for extending the time to appeal (Pioneer factors).

| court | decision | citation | position (planner; batch confirms) | status |
|---|---|---|---|---|
| U.S. | Pioneer Investment Services Co. v. Brunswick Associates Ltd. Partnership | 507 U.S. 380 (1993) |  | planned |
| 1st Cir. | Graphic Communications International Union, Local 12-N v. Quebecor Printing Providence, Inc. | 270 F.3d 1 (1st Cir. 2001) |  | planned |
| 2d Cir. | Silivanch v. Celebrity Cruises, Inc. | 333 F.3d 355 (2d Cir. 2003) |  | planned |
| 3d Cir. | Ragguette v. Premier Wines & Spirits | 691 F.3d 315 (3d Cir. 2012) |  | planned |
| 4th Cir. | Thompson v. E.I. DuPont de Nemours & Co. | 76 F.3d 530 (4th Cir. 1996) |  | planned |
| 5th Cir. | Halicki v. Louisiana Casino Cruises, Inc. | 151 F.3d 465 (5th Cir. 1998) | also in rulings corpus (59(e) arithmetic) | corpus+expect (`halicki-5cir-1998`) |
| 6th Cir. | Nicholson v. City of Warren | 467 F.3d 525 (6th Cir. 2006) |  | planned |
| 6th Cir. | Ultimate Appliance CC v. Kirby Co. | 601 F.3d 414 (6th Cir. 2010) |  | corpus+expect (`ultimate-appliance-6cir-2010`) |
| 7th Cir. | Prizevoits v. Indiana Bell Telephone Co. | 76 F.3d 132 (7th Cir. 1996) |  | planned |
| 8th Cir. | Lowry v. McDonnell Douglas Corp. | 211 F.3d 457 (8th Cir. 2000) |  | planned |
| 9th Cir. | Pincay v. Andrews | 389 F.3d 853 (9th Cir. 2004) |  | planned |
| 10th Cir. | City of Chanute v. Williams Natural Gas Co. | 31 F.3d 1041 (10th Cir. 1994) |  | planned |
| 10th Cir. | United States v. Torres | 372 F.3d 1159 (10th Cir. 2004) |  | planned |
| 11th Cir. | Advanced Estimating System, Inc. v. Riney | 130 F.3d 996 (11th Cir. 1997) |  | planned |
| D.C. Cir. | In re Vitamins Antitrust Class Actions | 327 F.3d 1207 (D.C. Cir. 2003) |  | planned |

Open (no verified leading decision named yet; research, never fill by guess): Fed. Cir.

#### FRAP 4(b) (`lit/frap`, `frap4b`)

Is the criminal notice-of-appeal period a forfeitable claim-processing rule?

| court | decision | citation | position (planner; batch confirms) | status |
|---|---|---|---|---|
| U.S. | Eberhart v. United States | 546 U.S. 12 (2005) |  | planned |
| 2d Cir. | United States v. Frias | 521 F.3d 229 (2d Cir. 2008) |  | planned |
| 3d Cir. | Government of the Virgin Islands v. Martinez | 620 F.3d 321 (3d Cir. 2010) |  | planned |
| 4th Cir. | United States v. Urutyan | 564 F.3d 679 (4th Cir. 2009) |  | planned |
| 5th Cir. | United States v. Martinez | 496 F.3d 387 (5th Cir. 2007) |  | planned |
| 6th Cir. | United States v. Gaytan-Garza | 652 F.3d 680 (6th Cir. 2011) |  | planned |
| 7th Cir. | United States v. Neff | 598 F.3d 320 (7th Cir. 2010) |  | planned |
| 8th Cir. | United States v. Watson | 623 F.3d 542 (8th Cir. 2010) |  | planned |
| 9th Cir. | United States v. Sadler | 480 F.3d 932 (9th Cir. 2007) |  | planned |
| 10th Cir. | United States v. Garduno | 506 F.3d 1287 (10th Cir. 2007) |  | planned |
| 11th Cir. | United States v. Lopez | 562 F.3d 1309 (11th Cir. 2009) |  | planned |
| D.C. Cir. | United States v. Byfield | 522 F.3d 400 (D.C. Cir. 2008) |  | planned |

Open (no verified leading decision named yet; research, never fill by guess): 1st Cir. Fed. Cir.: no criminal jurisdiction.

#### FRAP 4(c) (`lit/frap`, `frap4c`)

Inmate filing (prison mailbox) rule: when is a paper filed?

| court | decision | citation | position (planner; batch confirms) | status |
|---|---|---|---|---|
| U.S. | Houston v. Lack | 487 U.S. 266 (1988) |  | planned |
| 1st Cir. | Morales-Rivera v. United States | 184 F.3d 109 (1st Cir. 1999) |  | planned |
| 2d Cir. | Noble v. Kelly | 246 F.3d 93 (2d Cir. 2001) |  | planned |
| 3d Cir. | Burns v. Morton | 134 F.3d 109 (3d Cir. 1998) | also an AEDPA date case | corpus (`burns-3cir-1998`) |
| 4th Cir. | Lewis v. Richmond City Police Department | 947 F.2d 733 (4th Cir. 1991) |  | planned |
| 5th Cir. | Spotville v. Cain | 149 F.3d 374 (5th Cir. 1998) |  | planned |
| 6th Cir. | Gerald Pierce v. Ocwen Loan Servicing, LLC | 987 F.3d 577 (6th Cir. 2021) |  | corpus+expect (`pierce-6cir-2021`) |
| 7th Cir. | Ingram v. Jones | 507 F.3d 640 (7th Cir. 2007) |  | planned |
| 8th Cir. | Moore v. United States | 173 F.3d 1131 (8th Cir. 1999) | also an AEDPA date case | corpus (`moore-8cir-1999`) |
| 9th Cir. | Huizar v. Carey | 273 F.3d 1220 (9th Cir. 2001) |  | planned |
| 10th Cir. | United States v. Ceballos-Martinez | 387 F.3d 1140 (10th Cir. 2004) |  | planned |
| 11th Cir. | Washington v. United States | 243 F.3d 1299 (11th Cir. 2001) |  | planned |

Open (no verified leading decision named yet; research, never fill by guess): D.C. Cir., Fed. Cir.

#### FRAP 4(a)(1) (`lit/frap`, `frap4a1`)

The civil appeal period and its jurisdictional character.

| court | decision | citation | position (planner; batch confirms) | status |
|---|---|---|---|---|
| U.S. | Bowles v. Russell | 551 U.S. 205 (2007) |  | corpus+expect (`bowles-scotus-2007`; tests/expect/bowles-v-russell, every circuit) |
| 5th Cir. | Lashley v. Ford Motor Co. | 518 F.2d 749 (5th Cir. 1975) |  | corpus+expect (`lashley-5cir-1975`) |

Open (no verified leading decision named yet; research, never fill by guess): 1st Cir., 2d Cir., 3d Cir., 4th Cir., 6th Cir., 7th Cir., 8th Cir., 9th Cir., 10th Cir., 11th Cir., D.C. Cir., Fed. Cir.

#### FRAP 26(a), 26(c) (`lit/frap`, `frap26`)

Computing appellate periods; the three added days.

| court | decision | citation | position (planner; batch confirms) | status |
|---|---|---|---|---|
| 1st Cir. | Goncalves v. Bondi | 138 F.4th 58 (1st Cir. 2025) |  | corpus (`goncalves-1cir-2025`) |
| 5th Cir. | Yong Un Hong v. Smith | 129 F.3d 824 (5th Cir. 1997) |  | corpus (`hong-5cir-1997`) |
| 5th Cir. | Ramos-Lopez v. Lynch | 823 F.3d 1024 (5th Cir. 2016) |  | corpus (`ramos-lopez-5cir-2016`) |

Open (no verified leading decision named yet; research, never fill by guess): 2d Cir., 3d Cir., 4th Cir., 6th Cir., 7th Cir., 8th Cir., 9th Cir., 10th Cir., 11th Cir., D.C. Cir., Fed. Cir.

#### FRCP 6(a), 6(d) (`lit/frcp`, `frcp6`)

Computing periods; the three added days after service.

| court | decision | citation | position (planner; batch confirms) | status |
|---|---|---|---|---|
| 5th Cir. | Fletcher v. Apfel | 210 F.3d 510 (5th Cir. 2000) |  | corpus (`fletcher-5cir-2000`) |
| 5th Cir. | Serna v. Law Office of Joseph Onwut-Eaka, P.C. | 732 F.3d 440 (5th Cir. 2013) |  | corpus (`serna-5cir-2013`) |
| 9th Cir. | Kyle v. Campbell Soup Co. | 28 F.3d 928 (9th Cir. 1994) |  | corpus (`kyle-9cir-1994`) |
| 9th Cir. | Organic Cannabis Foundation v. Cir | 962 F.3d 1082 (9th Cir. 2020) |  | corpus (`organic-cannabis-9cir-2020`) |
| 9th Cir. | Chance Blackman v. Theresa Cisneros | 122 F.4th 377 (9th Cir. 2024) |  | corpus (`blackman-9cir-2024`) |

Open (no verified leading decision named yet; research, never fill by guess): 1st Cir., 2d Cir., 3d Cir., 4th Cir., 6th Cir., 7th Cir., 8th Cir., 10th Cir., 11th Cir., D.C. Cir. Fed. Cir.: applies regional-circuit law to procedural questions not unique to patent law, so the regional cell governs.

#### FRCP 6(b) (`lit/frcp`, `frcp6b`)

Extending time; excusable neglect after the period expires.

| court | decision | citation | position (planner; batch confirms) | status |
|---|---|---|---|---|
| U.S. | Lujan v. National Wildlife Federation | 497 U.S. 871 (1990) |  | planned |
| 6th Cir. | Nafziger v. McDermott International, Inc. | 467 F.3d 514 (6th Cir. 2006) |  | planned |
| 7th Cir. | Raymond v. Ameritech Corp. | 442 F.3d 600 (7th Cir. 2006) |  | planned |

Open (no verified leading decision named yet; research, never fill by guess): 1st Cir., 2d Cir., 3d Cir., 4th Cir., 5th Cir., 8th Cir., 9th Cir., 10th Cir., 11th Cir., D.C. Cir. Fed. Cir.: applies regional-circuit law to procedural questions not unique to patent law, so the regional cell governs.

#### FRCP 59(b), 59(e) (`lit/frcp`, `frcp59`)

The 28-day (formerly 10-day) new-trial / amend-judgment period, which no court may extend.

| court | decision | citation | position (planner; batch confirms) | status |
|---|---|---|---|---|
| U.S. | Banister v. Davis | 590 U.S. 504 (2020) |  | planned |
| 1st Cir. | Feinstein v. Moses | 951 F.2d 16 (1st Cir. 1991) |  | planned |
| 2d Cir. | Malek v. Feigenbaum | 116 F.4th 118 (2d Cir. 2024) |  | corpus+expect (`malek-2cir-2024`) |
| 3d Cir. | Albright v. Virtue | 273 F.3d 564 (3d Cir. 2001) |  | corpus+expect (`albright-3cir-2001`) |
| 4th Cir. | Panhorst v. United States | 241 F.3d 367 (4th Cir. 2001) |  | planned |
| 5th Cir. | Halicki v. Louisiana Casino Cruises, Inc. | 151 F.3d 465 (5th Cir. 1998) |  | corpus+expect (`halicki-5cir-1998`) |
| 5th Cir. | Midwest Employers Casualty Co. v. Williams | 161 F.3d 877 (5th Cir. 1998) |  | corpus+expect (`midwest-employers-5cir-1998`) |
| 7th Cir. | Justice v. Town of Cicero | 682 F.3d 662 (7th Cir. 2012) |  | corpus+expect (`justice-7cir-2012`) |
| 8th Cir. | Arnold v. Wood | 238 F.3d 992 (8th Cir. 2001) |  | planned |
| 9th Cir. | Carter v. United States | 973 F.2d 1479 (9th Cir. 1992) |  | planned |
| 10th Cir. | Parker v. Board of Public Utilities of Kansas City | 77 F.3d 1289 (10th Cir. 1996) |  | corpus+expect (`parker-10cir-1996`) |
| 11th Cir. | Cavaliere v. Allstate Insurance | 996 F.2d 1111 (11th Cir. 1993) |  | corpus+expect (`cavaliere-11cir-1993`) |
| 11th Cir. | Jackson v. Crosby | 375 F.3d 1291 (11th Cir. 2004) |  | corpus+expect (`jackson-v-crosby-11cir-2004`) |
| D.C. Cir. | Derrington-Bey v. District of Columbia Department of Corrections | 39 F.3d 1224 (D.C. Cir. 1994) |  | corpus+expect (`derrington-bey-dccir-1994`) |

Open (no verified leading decision named yet; research, never fill by guess): 6th Cir. Fed. Cir.: applies regional-circuit law to procedural questions not unique to patent law, so the regional cell governs.

#### FRCP 50(b) (`lit/frcp`, `frcp50`)

Renewed motion for judgment as a matter of law: timing and preservation.

| court | decision | citation | position (planner; batch confirms) | status |
|---|---|---|---|---|
| U.S. | Unitherm Food Systems, Inc. v. Swift-Eckrich, Inc. | 546 U.S. 394 (2006) |  | planned |
| 11th Cir. | Circuitronix, LLC v. Shenzen Kinwong Electronic Co., Ltd. | 993 F.3d 1299 (11th Cir. 2021) |  | corpus+expect (`circuitronix-11cir-2021`) |

Open (no verified leading decision named yet; research, never fill by guess): 1st Cir., 2d Cir., 3d Cir., 4th Cir., 5th Cir., 6th Cir., 7th Cir., 8th Cir., 9th Cir., 10th Cir., D.C. Cir. Fed. Cir.: applies regional-circuit law to procedural questions not unique to patent law, so the regional cell governs.

#### FRCP 58 (`lit/frcp`, `frcp58`)

The separate-document requirement and the 150-day rule (FRAP 4(a)(7)).

| court | decision | citation | position (planner; batch confirms) | status |
|---|---|---|---|---|
| U.S. | United States v. Indrelunas | 411 U.S. 216 (1973) |  | planned |
| U.S. | Bankers Trust Co. v. Mallis | 435 U.S. 381 (1978) |  | planned |
| 1st Cir. | Fiore v. Washington County Community Mental Health Center | 960 F.2d 229 (1st Cir. 1992) |  | planned |
| 3d Cir. | In re Cendant Corp. Securities Litigation | 454 F.3d 235 (3d Cir. 2006) |  | planned |
| 7th Cir. | Perry v. Sheet Metal Workers' Local No. 73 Pension Fund | 585 F.3d 358 (7th Cir. 2009) |  | planned |
| 9th Cir. | Vernon v. Heckler | 811 F.2d 1274 (9th Cir. 1987) |  | planned |
| 10th Cir. | Clough v. Rush | 959 F.2d 182 (10th Cir. 1992) |  | planned |
| 11th Cir. | Reynolds v. Golden Corral Corp. | 213 F.3d 1344 (11th Cir. 2000) |  | planned |
| D.C. Cir. | Kidd v. District of Columbia | 206 F.3d 35 (D.C. Cir. 2000) |  | planned |

Open (no verified leading decision named yet; research, never fill by guess): 2d Cir., 4th Cir., 5th Cir., 6th Cir., 8th Cir. Fed. Cir.: applies regional-circuit law to procedural questions not unique to patent law, so the regional cell governs.

#### FRCP 54(b) (`lit/frcp`, `frcp54b`)

Partial final judgment: "no just reason for delay".

| court | decision | citation | position (planner; batch confirms) | status |
|---|---|---|---|---|
| U.S. | Sears, Roebuck & Co. v. Mackey | 351 U.S. 427 (1956) |  | planned |
| U.S. | Curtiss-Wright Corp. v. General Electric Co. | 446 U.S. 1 (1980) |  | planned |
| 1st Cir. | Spiegel v. Trustees of Tufts College | 843 F.2d 38 (1st Cir. 1988) |  | planned |
| 2d Cir. | Ginett v. Computer Task Group, Inc. | 962 F.2d 1085 (2d Cir. 1992) |  | planned |
| 3d Cir. | Berckeley Investment Group, Ltd. v. Colkitt | 455 F.3d 195 (3d Cir. 2006) |  | planned |
| 4th Cir. | Braswell Shipyards, Inc. v. Beazer East, Inc. | 2 F.3d 1331 (4th Cir. 1993) |  | planned |
| 5th Cir. | PYCA Industries, Inc. v. Harrison County Waste Water Management District | 81 F.3d 1412 (5th Cir. 1996) |  | planned |
| 6th Cir. | Akers v. Alvey | 338 F.3d 491 (6th Cir. 2003) |  | planned |
| 7th Cir. | Lottie v. West American Insurance Co. | 408 F.3d 935 (7th Cir. 2005) |  | planned |
| 8th Cir. | Interstate Power Co. v. Kansas City Power & Light Co. | 992 F.2d 804 (8th Cir. 1993) |  | planned |
| 9th Cir. | Wood v. GCC Bend, LLC | 422 F.3d 873 (9th Cir. 2005) |  | planned |
| 10th Cir. | Oklahoma Turnpike Authority v. Bruner | 259 F.3d 1236 (10th Cir. 2001) |  | planned |
| 11th Cir. | Lloyd Noland Foundation, Inc. v. Tenet Health Care Corp. | 483 F.3d 773 (11th Cir. 2007) |  | planned |
| D.C. Cir. | Brooks v. District Hospital Partners, L.P. | 606 F.3d 800 (D.C. Cir. 2010) |  | planned |
| Fed. Cir. | W.L. Gore & Associates, Inc. v. International Medical Prosthetics Research Associates | 975 F.2d 858 (Fed. Cir. 1992) |  | planned |

Fed. Cir.: applies regional-circuit law to procedural questions not unique to patent law, so the regional cell governs.

#### FRCP 16(b)(4) with 15(a)(2) (`lit/frcp`, `frcp16b`)

Amending after the scheduling-order deadline: good cause first.

| court | decision | citation | position (planner; batch confirms) | status |
|---|---|---|---|---|
| U.S. | Foman v. Davis | 371 U.S. 178 (1962) |  | corpus (`foman-scotus-1962`); not encoded: FRCP 73(a) and 6(a) before 1963 are not modeled (lg-wvlf.4) |
| 1st Cir. | O'Connell v. Hyatt Hotels of Puerto Rico | 357 F.3d 152 (1st Cir. 2004) |  | planned |
| 2d Cir. | Parker v. Columbia Pictures Industries | 204 F.3d 326 (2d Cir. 2000) |  | planned |
| 3d Cir. | Eastern Minerals & Chemicals Co. v. Mahan | 225 F.3d 330 (3d Cir. 2000) |  | planned |
| 4th Cir. | Nourison Rug Corp. v. Parvizian | 535 F.3d 295 (4th Cir. 2008) |  | planned |
| 5th Cir. | S&W Enterprises, L.L.C. v. SouthTrust Bank of Alabama | 315 F.3d 533 (5th Cir. 2003) |  | planned |
| 6th Cir. | Leary v. Daeschner | 349 F.3d 888 (6th Cir. 2003) |  | planned |
| 7th Cir. | Trustmark Insurance Co. v. General & Cologne Life Re of America | 424 F.3d 542 (7th Cir. 2005) |  | planned |
| 8th Cir. | Sherman v. Winco Fireworks, Inc. | 532 F.3d 709 (8th Cir. 2008) |  | planned |
| 9th Cir. | Johnson v. Mammoth Recreations, Inc. | 975 F.2d 604 (9th Cir. 1992) |  | planned |
| 10th Cir. | Gorsuch, Ltd., B.C. v. Wells Fargo National Bank Ass'n | 771 F.3d 1230 (10th Cir. 2014) |  | planned |
| 11th Cir. | Sosa v. Airprint Systems, Inc. | 133 F.3d 1417 (11th Cir. 1998) |  | planned |

Open (no verified leading decision named yet; research, never fill by guess): D.C. Cir. Fed. Cir.: applies regional-circuit law to procedural questions not unique to patent law, so the regional cell governs.

#### FRCP 55(c) (`lit/frcp`, `frcp55c`)

Setting aside an entry of default for good cause.

| court | decision | citation | position (planner; batch confirms) | status |
|---|---|---|---|---|
| 1st Cir. | KPS & Associates, Inc. v. Designs by FMC, Inc. | 318 F.3d 1 (1st Cir. 2003) |  | planned |
| 2d Cir. | Enron Oil Corp. v. Diakuhara | 10 F.3d 90 (2d Cir. 1993) |  | planned |
| 3d Cir. | United States v. $55,518.05 in U.S. Currency | 728 F.2d 192 (3d Cir. 1984) |  | planned |
| 4th Cir. | Payne ex rel. Estate of Calzada v. Brake | 439 F.3d 198 (4th Cir. 2006) |  | planned |
| 5th Cir. | Lacy v. Sitel Corp. | 227 F.3d 290 (5th Cir. 2000) |  | planned |
| 6th Cir. | United Coin Meter Co. v. Seaboard Coastline Railroad | 705 F.2d 839 (6th Cir. 1983) |  | planned |
| 7th Cir. | Cracco v. Vitran Express, Inc. | 559 F.3d 625 (7th Cir. 2009) |  | planned |
| 8th Cir. | Johnson v. Dayton Electric Manufacturing Co. | 140 F.3d 781 (8th Cir. 1998) |  | planned |
| 9th Cir. | TCI Group Life Insurance Plan v. Knoebber | 244 F.3d 691 (9th Cir. 2001) |  | planned |
| 11th Cir. | Compania Interamericana Export-Import, S.A. v. Compania Dominicana de Aviacion | 88 F.3d 948 (11th Cir. 1996) |  | planned |
| D.C. Cir. | Jackson v. Beech | 636 F.2d 831 (D.C. Cir. 1980) |  | planned |

Open (no verified leading decision named yet; research, never fill by guess): 10th Cir. Fed. Cir.: applies regional-circuit law to procedural questions not unique to patent law, so the regional cell governs.

#### FRCP 24 (`lit/frcp`, `frcp24`)

Timeliness of a motion to intervene.

| court | decision | citation | position (planner; batch confirms) | status |
|---|---|---|---|---|
| U.S. | NAACP v. New York | 413 U.S. 345 (1973) |  | planned |
| U.S. | Cameron v. EMW Women's Surgical Center, P.S.C. | 595 U.S. 267 (2022) |  | planned |
| 1st Cir. | R & G Mortgage Corp. v. Federal Home Loan Mortgage Corp. | 584 F.3d 1 (1st Cir. 2009) |  | planned |
| 2d Cir. | United States v. Pitney Bowes, Inc. | 25 F.3d 66 (2d Cir. 1994) |  | planned |
| 4th Cir. | Gould v. Alleco, Inc. | 883 F.2d 281 (4th Cir. 1989) |  | planned |
| 5th Cir. | Stallworth v. Monsanto Co. | 558 F.2d 257 (5th Cir. 1977) |  | planned |
| 6th Cir. | Stupak-Thrall v. Glickman | 226 F.3d 467 (6th Cir. 2000) |  | planned |
| 7th Cir. | Sokaogon Chippewa Community v. Babbitt | 214 F.3d 941 (7th Cir. 2000) |  | planned |
| 9th Cir. | League of United Latin American Citizens v. Wilson | 131 F.3d 1297 (9th Cir. 1997) |  | planned |
| 10th Cir. | Utah Ass'n of Counties v. Clinton | 255 F.3d 1246 (10th Cir. 2001) |  | planned |
| 11th Cir. | Chiles v. Thornburgh | 865 F.2d 1197 (11th Cir. 1989) |  | planned |
| D.C. Cir. | Smoke v. Norton | 252 F.3d 468 (D.C. Cir. 2001) |  | planned |

Open (no verified leading decision named yet; research, never fill by guess): 3d Cir., 8th Cir. Fed. Cir.: applies regional-circuit law to procedural questions not unique to patent law, so the regional cell governs.

#### FRCP 37(c)(1) (`lit/frcp`, `frcp37c`)

Exclusion for a failure to disclose unless substantially justified or harmless.

| court | decision | citation | position (planner; batch confirms) | status |
|---|---|---|---|---|
| 1st Cir. | Esposito v. Home Depot U.S.A., Inc. | 590 F.3d 72 (1st Cir. 2009) |  | planned |
| 2d Cir. | Design Strategy, Inc. v. Davis | 469 F.3d 284 (2d Cir. 2006) |  | planned |
| 4th Cir. | Southern States Rack & Fixture, Inc. v. Sherwin-Williams Co. | 318 F.3d 592 (4th Cir. 2003) |  | planned |
| 5th Cir. | Hamburger v. State Farm Mutual Automobile Insurance Co. | 361 F.3d 875 (5th Cir. 2004) |  | planned |
| 6th Cir. | Roberts ex rel. Johnson v. Galen of Virginia, Inc. | 325 F.3d 776 (6th Cir. 2003) |  | planned |
| 7th Cir. | David v. Caterpillar, Inc. | 324 F.3d 851 (7th Cir. 2003) |  | planned |
| 8th Cir. | Wegener v. Johnson | 527 F.3d 687 (8th Cir. 2008) |  | planned |
| 9th Cir. | Yeti by Molly, Ltd. v. Deckers Outdoor Corp. | 259 F.3d 1101 (9th Cir. 2001) |  | planned |
| 10th Cir. | Woodworker's Supply, Inc. v. Principal Mutual Life Insurance Co. | 170 F.3d 985 (10th Cir. 1999) |  | planned |

Open (no verified leading decision named yet; research, never fill by guess): 3d Cir., 11th Cir., D.C. Cir. Fed. Cir.: applies regional-circuit law to procedural questions not unique to patent law, so the regional cell governs.

#### FRCP 26(a)(2) (`lit/frcp`, `frcp26a2`)

Expert disclosure: when a treating physician needs a written report.

| court | decision | citation | position (planner; batch confirms) | status |
|---|---|---|---|---|
| 1st Cir. | Downey v. Bob's Discount Furniture Holdings, Inc. | 633 F.3d 1 (1st Cir. 2011) |  | planned |
| 7th Cir. | Meyers v. National Railroad Passenger Corp. | 619 F.3d 729 (7th Cir. 2010) |  | planned |
| 9th Cir. | Goodman v. Staples The Office Superstore, LLC | 644 F.3d 817 (9th Cir. 2011) |  | planned |

Open (no verified leading decision named yet; research, never fill by guess): 2d Cir., 3d Cir., 4th Cir., 5th Cir., 6th Cir., 8th Cir., 10th Cir., 11th Cir., D.C. Cir. Fed. Cir.: applies regional-circuit law to procedural questions not unique to patent law, so the regional cell governs.

#### FRCP 12(b)(6), 8(a)(2) (`lit/frcp`, `frcp12b6`)

Plausibility pleading.

| court | decision | citation | position (planner; batch confirms) | status |
|---|---|---|---|---|
| U.S. | Bell Atlantic Corp. v. Twombly | 550 U.S. 544 (2007) |  | planned |
| U.S. | Ashcroft v. Iqbal | 556 U.S. 662 (2009) |  | planned |
| 1st Cir. | Ocasio-Hernandez v. Fortuno-Burset | 640 F.3d 1 (1st Cir. 2011) |  | planned |
| 2d Cir. | Arista Records, LLC v. Doe 3 | 604 F.3d 110 (2d Cir. 2010) |  | planned |
| 3d Cir. | Fowler v. UPMC Shadyside | 578 F.3d 203 (3d Cir. 2009) |  | planned |
| 4th Cir. | Francis v. Giacomelli | 588 F.3d 186 (4th Cir. 2009) |  | planned |
| 6th Cir. | 16630 Southfield Ltd. Partnership v. Flagstar Bank, F.S.B. | 727 F.3d 502 (6th Cir. 2013) |  | planned |
| 7th Cir. | Swanson v. Citibank, N.A. | 614 F.3d 400 (7th Cir. 2010) |  | planned |
| 8th Cir. | Braden v. Wal-Mart Stores, Inc. | 588 F.3d 585 (8th Cir. 2009) |  | planned |
| 9th Cir. | Starr v. Baca | 652 F.3d 1202 (9th Cir. 2011) |  | planned |
| 10th Cir. | Khalik v. United Air Lines | 671 F.3d 1188 (10th Cir. 2012) |  | planned |
| 11th Cir. | American Dental Ass'n v. Cigna Corp. | 605 F.3d 1283 (11th Cir. 2010) |  | planned |

Open (no verified leading decision named yet; research, never fill by guess): 5th Cir., D.C. Cir. Fed. Cir.: applies regional-circuit law to procedural questions not unique to patent law, so the regional cell governs.

#### FRCP 56 (`lit/frcp`, `frcp56`)

Summary judgment standard.

| court | decision | citation | position (planner; batch confirms) | status |
|---|---|---|---|---|
| U.S. | Celotex Corp. v. Catrett | 477 U.S. 317 (1986) |  | planned |
| U.S. | Anderson v. Liberty Lobby, Inc. | 477 U.S. 242 (1986) |  | planned |

Open (no verified leading decision named yet; research, never fill by guess): 1st Cir., 2d Cir., 3d Cir., 4th Cir., 5th Cir., 6th Cir., 7th Cir., 8th Cir., 9th Cir., 10th Cir., 11th Cir., D.C. Cir. Fed. Cir.: applies regional-circuit law to procedural questions not unique to patent law, so the regional cell governs.

#### FRCP 52(a)(6) (`lit/frcp`, `frcp52`)

Clear-error review of findings.

| court | decision | citation | position (planner; batch confirms) | status |
|---|---|---|---|---|
| U.S. | Anderson v. City of Bessemer City | 470 U.S. 564 (1985) |  | corpus (`anderson-bessemer-scotus-1985`); not encoded: no dated filing (lg-wvlf.4) |

Open (no verified leading decision named yet; research, never fill by guess): 1st Cir., 2d Cir., 3d Cir., 4th Cir., 5th Cir., 6th Cir., 7th Cir., 8th Cir., 9th Cir., 10th Cir., 11th Cir., D.C. Cir. Fed. Cir.: applies regional-circuit law to procedural questions not unique to patent law, so the regional cell governs.

#### FRCP 13(a) (`lit/frcp`, `frcp13`)

Compulsory counterclaims.

| court | decision | citation | position (planner; batch confirms) | status |
|---|---|---|---|---|
| U.S. | Baker v. Gold Seal Liquors, Inc. | 417 U.S. 467 (1974) |  | planned |

Open (no verified leading decision named yet; research, never fill by guess): 1st Cir., 2d Cir., 3d Cir., 4th Cir., 5th Cir., 6th Cir., 7th Cir., 8th Cir., 9th Cir., 10th Cir., 11th Cir., D.C. Cir. Fed. Cir.: applies regional-circuit law to procedural questions not unique to patent law, so the regional cell governs.

#### FRAP 3(c) (`lit/frap`, `frap3c`)

Contents of the notice of appeal; functional equivalents.

| court | decision | citation | position (planner; batch confirms) | status |
|---|---|---|---|---|
| U.S. | Torres v. Oakland Scavenger Co. | 487 U.S. 312 (1988) |  | corpus+expect (`torres-scotus-1988`; tests/expect/torres-v-oakland-scavenger, every circuit) |
| U.S. | Smith v. Barry | 502 U.S. 244 (1992) |  | corpus+expect (`smith-v-barry-scotus-1992`; tests/expect/smith-v-barry, every circuit) |
| U.S. | Becker v. Montgomery | 532 U.S. 757 (2001) |  | planned |

Open (no verified leading decision named yet; research, never fill by guess): 1st Cir., 2d Cir., 3d Cir., 4th Cir., 5th Cir., 6th Cir., 7th Cir., 8th Cir., 9th Cir., 10th Cir., 11th Cir., D.C. Cir., Fed. Cir.

#### FRAP 39 (`lit/frap`, `frap39`)

Costs on appeal.

| court | decision | citation | position (planner; batch confirms) | status |
|---|---|---|---|---|
| U.S. | City of San Antonio v. Hotels.com, L.P. | 593 U.S. 330 (2021) |  | planned |

Open (no verified leading decision named yet; research, never fill by guess): 1st Cir., 2d Cir., 3d Cir., 4th Cir., 5th Cir., 6th Cir., 7th Cir., 8th Cir., 9th Cir., 10th Cir., 11th Cir., D.C. Cir., Fed. Cir.

#### FRAP 41 (`lit/frap`, `frap41`)

Issuance and recall of the mandate.

| court | decision | citation | position (planner; batch confirms) | status |
|---|---|---|---|---|
| U.S. | Calderon v. Thompson | 523 U.S. 538 (1998) |  | planned |
| U.S. | Bell v. Thompson | 545 U.S. 794 (2005) |  | planned |

Open (no verified leading decision named yet; research, never fill by guess): 1st Cir., 2d Cir., 3d Cir., 4th Cir., 5th Cir., 6th Cir., 7th Cir., 8th Cir., 9th Cir., 10th Cir., 11th Cir., D.C. Cir., Fed. Cir.

#### FRCrimP 29(c), 33, 45(b) (`lit/frcrimp`, `frcrimp33`)

Post-verdict motion periods and their claim-processing character.

| court | decision | citation | position (planner; batch confirms) | status |
|---|---|---|---|---|
| U.S. | Carlisle v. United States | 517 U.S. 416 (1996) |  | planned |

Open (no verified leading decision named yet; research, never fill by guess): 1st Cir., 2d Cir., 3d Cir., 4th Cir., 5th Cir., 6th Cir., 7th Cir., 8th Cir., 9th Cir., 10th Cir., 11th Cir., D.C. Cir. Fed. Cir.: no criminal jurisdiction.

#### FRCrimP 11(d) (`lit/frcrimp`, `frcrimp11`)

Withdrawing a guilty plea.

| court | decision | citation | position (planner; batch confirms) | status |
|---|---|---|---|---|
| U.S. | United States v. Hyde | 520 U.S. 670 (1997) |  | planned |

Open (no verified leading decision named yet; research, never fill by guess): 1st Cir., 2d Cir., 3d Cir., 4th Cir., 5th Cir., 6th Cir., 7th Cir., 8th Cir., 9th Cir., 10th Cir., 11th Cir., D.C. Cir. Fed. Cir.: no criminal jurisdiction.

#### Fed. Cir. R. 36 (`lit/cafc`, `cafc36`)

Judgment of affirmance without opinion: what it decides.

| court | decision | citation | position (planner; batch confirms) | status |
|---|---|---|---|---|
| Fed. Cir. | Rates Technology, Inc. v. Mediatrix Telecom, Inc. | 688 F.3d 742 (Fed. Cir. 2012) |  | planned |
| Fed. Cir. | TecSec, Inc. v. Adobe Inc. | 978 F.3d 1278 (Fed. Cir. 2020) |  | planned |

### 3.3 Circuit packages

Each circuit package lays its circuit rules over `lit/frap`, so its FRAP
cells above (one per circuit column) are its leading decisions, run in the
circuit's world (`circuit-appellate-<circuit>.world`, as
`docs/litigate/change/worlds/` already has for ca2, ca5, ca6 and cadc). The
circuit rules themselves (briefing and appendix periods, rehearing,
mandate, dismissal for want of prosecution, oral argument) are clerk-run
deadlines that reported decisions rarely construe. The one with a verified
leading decision is Fed. Cir. R. 36 (`cafc36`, above). For every other
modeled circuit rule the cell is open:

| package | circuit rules modeled (most cited) | leading decision |
|---|---|---|
| `lit/ca1` | 1st Cir. R. 31.0, 34.0 | open |
| `lit/ca2` | 2d Cir. R. 12.1, 27.1, 28.1, 31.2, 32.1, 34.1, 40.1; IOP 32.1, 40.1 | open |
| `lit/ca3` | 3d Cir. L.A.R. 26.1, 28.1; I.O.P. 2.1, 5.1, 8.1-8.3, 9.1-9.5, 10.6 | open |
| `lit/ca4` | 4th Cir. R. 3, 10, 11, 30, 31, 34, 36, 40, 45; I.O.P. 34.3 | open |
| `lit/ca5` | 5th Cir. R. 12, 30.1, 31.2-31.4, 32.4, 32.5, 34.2, 40.1, 42.3, 42.4 | open |
| `lit/ca6` | 6th Cir. R. 3, 10, 26, 30, 31, 34, 36, 40, 41, 45; I.O.P. 3, 40 | open |
| `lit/ca7` | 7th Cir. R. 3, 10, 22, 26.1, 31, 34, 40, 51, 54; I.O.P. 5, 7 | open |
| `lit/ca8` | 8th Cir. R. 34A | open |
| `lit/ca9` | 9th Cir. R. 3-1, 3-3, 3-4, 10-3, 27-10, 31-2, 32-1, 39-1, 40-1, 41-1, 41-2, 42-1 | open |
| `lit/ca10` | 10th Cir. R. 3.3, 3.4, 10.2, 10.3, 27.3, 27.6, 31.1, 34.1, 39.2, 40.2, 42.1, 46.1 | open |
| `lit/ca11` | 11th Cir. R. 12-1, 12-2, 30-1, 31-1, 32-4, 33-1, 34-3, 36-2, 39-2, 40-2, 40-11, 42-1 | open |
| `lit/cadc` | D.C. Cir. R. 31 | open |
| `lit/cafc` | Fed. Cir. R. 8, 26, 28.1, 30, 31, 36, 40, 41, 47.3, 47.4, 47.6 | Fed. Cir. R. 36: `cafc36` |

### lg-wvlf.4 batch disposition (2026-10-28)

The batch contains ten appellate candidates. Bowles, Smith v. Barry, and
Torres are encoded as every-circuit expect tests under `tests/expect/`.
Under the MVP drop rule, the other six candidates without a dated filing
event that the current model can count are retained in the opinion corpus
but have no expect test: Cohen v. Beneficial Industrial Loan Corp. (337 U.S.
541 (1949); interlocutory appealability), Cooter & Gell v. Hartmarx Corp.
(496 U.S. 384 (1990); sanctions review), Gonzalez v. Crosby (545 U.S. 524
(2005); Rule 60(b)(6)), Anderson v. City of Bessemer City (470 U.S. 564
(1985); review standard), Ornelas v. United States (517 U.S. 690 (1996);
review standard), and United States v. American Railway Express Co. (265
U.S. 425 (1924); cross-appeal scope). These opinions remain sourced under
`sources/change-corpus/rulings/`; their holdings are not expected as
dated-deadline outcomes by this batch.

Foman v. Davis (371 U.S. 178 (1962)) is also retained without an expect
test. Its opinion supplies dated events (judgment December 19, 1960; Rule
59 motion December 20, 1960; notices of appeal January 17 and 26, 1961),
but the applicable FRCP 73(a) appeal-tolling rule and contemporaneous Rule
6(a) computation predate the modeled timeline. Modeling the deadline needs
a sourced, dated historical rule layer; this batch does not infer one from
the opinion. See the FRCP 16(b)(4) row above.

The three encoded cases use opinion-cited record entries and dates. Their
RECAP/PACER docket entries were not independently retrieved in this run;
the expect files identify that provenance limitation rather than treating
the docket as reviewed.

## 4. Batches

One bead per batch. Each takes about 10 `planned` cells, in matrix order
(splits first). Per cell the batch:

1. stores the as-served opinion under `sources/change-corpus/rulings/` with
   its `MANIFEST.json` entry (source URL, retrieval time, sha256, pin cites),
   as lg-fq2.3 did;
2. confirms the citation again and the position from the text, and drops the
   cell (reporting why) if the opinion does not hold it;
3. writes the matter (`.case`) and the world it runs in, and
   `tests/expect/<forum>/<id>.toml` naming them, so `litigate test --impact`
   reads every later change of the rule against it;
4. where the decision changes the rule (a split's resolution, Hamer,
   Nutraceutical, Kemp, the 2005 FRAP 4(a)(6) text), encodes it as a new
   dated layer file over the base, never an edit of the base.

Public law only; no carbon builds. Batches are independent and may run in
any order, but a split's batches should land before its resolving Supreme
Court layer is relied on.

| batch | cells | rules | decisions |
|---|---|---|---|
| B01 | 10 | `frap4a4`, `frcp11` | Browder (U.S.); Hamer (U.S.); Lizardo (3d Cir.); National Ecological Foundation (6th Cir.); Blue (7th Cir.); Green (11th Cir.); Wilburn (D.C. Cir.); Obaydullah (D.C. Cir.); Cooter & Gell (U.S.); In re Pennie & Edmonds LLP (2d Cir.) |
| B02 | 10 | `frcp11` | Star Mark Management, Inc. (2d Cir.); In re Schaefer Salt Recovery, Inc. (3d Cir.); Brickwood Contractors, Inc. (4th Cir.); Elliott (5th Cir.); Ridder (6th Cir.); Nisenbaum (7th Cir.); Matrix IV, Inc. (7th Cir.); Gordon (8th Cir.); Barber (9th Cir.); Radcliffe (9th Cir.) |
| B03 | 10 | `frcp11`, `frcp4m` | Roth (10th Cir.); Henderson (U.S.); Zapata (2d Cir.); Petrucelli (3d Cir.); Mendez (4th Cir.); Thompson (5th Cir.); Panaras (7th Cir.); Adams (8th Cir.); Efaw (9th Cir.); Espinoza (10th Cir.) |
| B04 | 10 | `frcp4m`, `frcp41a` | Horenkamp (11th Cir.); Mann (D.C. Cir.); Link (U.S.); Harvey Aluminum, Inc. (2d Cir.); Young (3d Cir.); Manze (3d Cir.); Exxon Corp. (5th Cir.); Philip Carey Manufacturing Co. (6th Cir.); Taylor (7th Cir.); Hells Canyon Preservation Council (9th Cir.) |
| B05 | 10 | `frcp41a`, `frcp15c` | Gobbo Farms & Orchards (10th Cir.); Perry (11th Cir.); Mayle (U.S.); Krupski (U.S.); Wilson (1st Cir.); Barrow (2d Cir.); Singletary (3d Cir.); Goodman (4th Cir.); Jacobsen (5th Cir.); Hall (7th Cir.) |
| B06 | 10 | `frcp15c`, `frcrimp12` | Heglund (8th Cir.); Garrett (10th Cir.); Powers (11th Cir.); Davis (U.S.); United States v. Fattah (3d Cir.); United States v. Vasquez (5th Cir.); United States v. Soto (6th Cir.); United States v. McMillian (7th Cir.); United States v. Anderson (8th Cir.); United States v. Guerrero (9th Cir.) |
| B07 | 10 | `frcrimp12`, `frcrimp35`, `frap5` | United States v. Bowline (10th Cir.); United States v. Sperrazza (11th Cir.); United States v. Higgs (3d Cir.); United States v. Shank (4th Cir.); United States v. Wisch (7th Cir.); United States v. Green (10th Cir.); Microsoft Corp. (U.S.); Nutraceutical Corp. (U.S.); Mowbray (1st Cir.); In re Sumitomo Copper Litigation (2d Cir.) |
| B08 | 10 | `frap5` | Gutierrez (3d Cir.); Lienhart (4th Cir.); McNamara (5th Cir.); In re Delta Air Lines (6th Cir.); Gary (7th Cir.); Lambert (9th Cir.); Carpenter (10th Cir.); Shin (11th Cir.); Jenkins (11th Cir.); In re D.C. Water & Sewer Authority (D.C. Cir.) |
| B09 | 10 | `frcp60b` | Ackermann (U.S.); Liljeberg (U.S.); Gonzalez (U.S.); Kemp (U.S.); In re 310 Associates (2d Cir.); Page (3d Cir.); Chick Kam Choo (5th Cir.); Pierce (6th Cir.); Fox (8th Cir.); Liberty Mutual Insurance Co. (9th Cir.) |
| B10 | 10 | `frcp60b`, `frap4a5` | Briones (9th Cir.); Cashner (10th Cir.); Parks (11th Cir.); Pioneer Investment Services Co. (U.S.); Graphic Communications International Union, Local 12-N (1st Cir.); Silivanch (2d Cir.); Ragguette (3d Cir.); Thompson (4th Cir.); Nicholson (6th Cir.); Prizevoits (7th Cir.) |
| B11 | 10 | `frap4a5`, `frap4b` | Lowry (8th Cir.); Pincay (9th Cir.); City of Chanute (10th Cir.); United States v. Torres (10th Cir.); Advanced Estimating System, Inc. (11th Cir.); In re Vitamins Antitrust Class Actions (D.C. Cir.); Eberhart (U.S.); United States v. Frias (2d Cir.); Government of the Virgin Islands (3d Cir.); United States v. Urutyan (4th Cir.) |
| B12 | 10 | `frap4b`, `frap4c` | United States v. Martinez (5th Cir.); United States v. Gaytan-Garza (6th Cir.); United States v. Neff (7th Cir.); United States v. Watson (8th Cir.); United States v. Sadler (9th Cir.); United States v. Garduno (10th Cir.); United States v. Lopez (11th Cir.); United States v. Byfield (D.C. Cir.); Houston (U.S.); Morales-Rivera (1st Cir.) |
| B13 | 10 | `frap4c`, `frcp6b` | Noble (2d Cir.); Lewis (4th Cir.); Spotville (5th Cir.); Ingram (7th Cir.); Huizar (9th Cir.); United States v. Ceballos-Martinez (10th Cir.); Washington (11th Cir.); Lujan (U.S.); Nafziger (6th Cir.); Raymond (7th Cir.) |
| B14 | 10 | `frcp59`, `frcp50`, `frcp58` | Banister (U.S.); Feinstein (1st Cir.); Panhorst (4th Cir.); Arnold (8th Cir.); Carter (9th Cir.); Unitherm Food Systems, Inc. (U.S.); United States v. Indrelunas (U.S.); Bankers Trust Co. (U.S.); Fiore (1st Cir.); In re Cendant Corp. Securities Litigation (3d Cir.) |
| B15 | 10 | `frcp58`, `frcp54b` | Perry (7th Cir.); Vernon (9th Cir.); Clough (10th Cir.); Reynolds (11th Cir.); Kidd (D.C. Cir.); Sears, Roebuck & Co. (U.S.); Curtiss-Wright Corp. (U.S.); Spiegel (1st Cir.); Ginett (2d Cir.); Berckeley Investment Group, Ltd. (3d Cir.) |
| B16 | 10 | `frcp54b` | Braswell Shipyards, Inc. (4th Cir.); PYCA Industries, Inc. (5th Cir.); Akers (6th Cir.); Lottie (7th Cir.); Interstate Power Co. (8th Cir.); Wood (9th Cir.); Oklahoma Turnpike Authority (10th Cir.); Lloyd Noland Foundation, Inc. (11th Cir.); Brooks (D.C. Cir.); W.L. Gore & Associates, Inc. (Fed. Cir.) |
| B17 | 10 | `frcp16b` | Foman (U.S.); O'Connell (1st Cir.); Parker (2d Cir.); Eastern Minerals & Chemicals Co. (3d Cir.); Nourison Rug Corp. (4th Cir.); S&W Enterprises, L.L.C. (5th Cir.); Leary (6th Cir.); Trustmark Insurance Co. (7th Cir.); Sherman (8th Cir.); Johnson (9th Cir.) |
| B18 | 10 | `frcp16b`, `frcp55c` | Gorsuch, Ltd., B.C. (10th Cir.); Sosa (11th Cir.); KPS & Associates, Inc. (1st Cir.); Enron Oil Corp. (2d Cir.); United States v. $55,518.05 in U.S. Currency (3d Cir.); Payne ex rel. Estate of Calzada (4th Cir.); Lacy (5th Cir.); United Coin Meter Co. (6th Cir.); Cracco (7th Cir.); Johnson (8th Cir.) |
| B19 | 10 | `frcp55c`, `frcp24` | TCI Group Life Insurance Plan (9th Cir.); Compania Interamericana Export-Import, S.A. (11th Cir.); Jackson (D.C. Cir.); NAACP (U.S.); Cameron (U.S.); R & G Mortgage Corp. (1st Cir.); United States v. Pitney Bowes, Inc. (2d Cir.); Gould (4th Cir.); Stallworth (5th Cir.); Stupak-Thrall (6th Cir.) |
| B20 | 10 | `frcp24`, `frcp37c` | Sokaogon Chippewa Community (7th Cir.); League of United Latin American Citizens (9th Cir.); Utah Ass'n of Counties (10th Cir.); Chiles (11th Cir.); Smoke (D.C. Cir.); Esposito (1st Cir.); Design Strategy, Inc. (2d Cir.); Southern States Rack & Fixture, Inc. (4th Cir.); Hamburger (5th Cir.); Roberts ex rel. Johnson (6th Cir.) |
| B21 | 10 | `frcp37c`, `frcp26a2`, `frcp12b6` | David (7th Cir.); Wegener (8th Cir.); Yeti by Molly, Ltd. (9th Cir.); Woodworker's Supply, Inc. (10th Cir.); Downey (1st Cir.); Meyers (7th Cir.); Goodman (9th Cir.); Bell Atlantic Corp. (U.S.); Ashcroft (U.S.); Ocasio-Hernandez (1st Cir.) |
| B22 | 10 | `frcp12b6`, `frcp56` | Arista Records, LLC (2d Cir.); Fowler (3d Cir.); Francis (4th Cir.); 16630 Southfield Ltd. Partnership (6th Cir.); Swanson (7th Cir.); Braden (8th Cir.); Starr (9th Cir.); Khalik (10th Cir.); American Dental Ass'n (11th Cir.); Celotex Corp. (U.S.) |
| B23 | 13 | `frcp56`, `frcp52`, `frcp13`, `frap3c`, `frap39`, `frap41`, `frcrimp33`, `frcrimp11`, `cafc36` | Anderson v. Liberty Lobby (U.S.); Anderson v. City of Bessemer City (U.S.); Baker (U.S.); Torres (U.S.); Smith (U.S.); Becker (U.S.); City of San Antonio (U.S.); Calderon (U.S.); Bell (U.S.); Carlisle (U.S.); United States v. Hyde (U.S.); Rates Technology, Inc. (Fed. Cir.); TecSec, Inc. (Fed. Cir.) |

Open cells (section 3) are not batched. When a batch or a later research
bead names and verifies one, it joins the next batch.
