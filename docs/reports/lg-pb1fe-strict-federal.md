# Strict federal extension and version regression (lg-pb1fe.35)

## Status

**Complete for the sourced federal extension review.** Strict errors fell from 77 to 25 across the 20 packages; 12 now pass. Remaining L5002 errors are listed by location below where the vendored rule is silent, fixed by statute, lacks a representable typed grant, or needs a missing historical source. No duration, counting rule, probability, edge, or test expectation was changed.

DECIDED: Leave habeas proposed-findings objections undeclared because 28 U.S.C. § 636(b)(1) and the habeas rules fix the period at 14 days; this follows the PM ruling for similarly fixed statutory/rule periods.

Strict baseline is `origin/main` `10b6a048`, checked with a release binary built from that archived tree; after counts use the release binary built from this tree after its source corpus was current. Counts include only severity `error` L5002 diagnostics (not informational unreachable-dependency notices).

| Package | origin/main L5002 | After L5002 | Strict after |
|---|---:|---:|:---:|
| ca7 | 2 | 0 | pass |
| caaf | 9 | 9 | fail |
| cafc | 3 | 1 | fail |
| cavc | 9 | 1 | fail |
| cit | 12 | 2 | fail |
| cofc | 6 | 0 | pass |
| frap | 10 | 0 | pass |
| frcp | 4 | 2 | fail |
| gamd | 2 | 0 | pass |
| gand | 2 | 0 | pass |
| gasd | 2 | 0 | pass |
| habeas | 4 | 4 | fail |
| itc | 1 | 0 | pass |
| mpep | 0 | 0 | pass |
| msnd | 2 | 0 | pass |
| mssd | 2 | 0 | pass |
| ptab | 9 | 2 | fail |
| scotus | 5 | 4 | fail |
| sdd | 2 | 0 | pass |
| ttab | 1 | 0 | pass |

The 20-package total moved from 77 to 25 L5002 errors; 12/20 strict checks now pass. Remaining errors are package-owned, not stale-lock or unreachable-dependency errors.

## Kind A — typed extension availability

A period was typed only where the cited text covers that court-rule or court-order period. The new FRCP-based grants use the existing grammar: court / GoodCause before expiry under FRCP 6(b)(1)(A), and court / ExcusableNeglect on a motion after expiry under FRCP 6(b)(1)(B). The six district packages inherit FRCP 6 for computation; GAMD, GAND, and GASD response periods now also carry that typed grant. The local clerk extension clauses are not typed because the packages have no clerk role.

| Package / period | Text read and result |
|---|---|
| CAVC Rule-set periods with new typed grants | CAVC Rule 26(b), `sources/cavc/cavc-rules-of-practice-2026-10-01.txt:747-754`: “The Court, on its own initiative or on a party's motion for good cause shown, may extend the time set by these Rules”; “not to exceed a total of 45 days”; beyond that “only for extraordinary circumstances.” GoodCause grants were retained through 45 days; ExtraordinaryCircumstances grants remain only on `MotionFiled()` before/after expiry. Rule 26(c)'s 15-day ceiling is in Rule 47 expedited proceedings (`sources/cavc/cavc-rules-of-practice-2026-10-01.txt:1872-1875`); none of the retained 45-day periods is a Rule 47 filing. The Federal Circuit appeal period is recorded separately as an unresolved item below. |
| CIT `complaint-after-summons-period` | USCIT Rule 3(a)(2) restates the statutory clock. The practice comment says “As provided in section 516A(a)(2) and (3) ... a complaint must be filed within 30 days after the filing of the summons” (`sources/cit/cit-rules-and-forms.txt:580-582`). Rule 6 grants were removed from both edges. Other CIT periods use Rule 6(b)(1), whose text also allows “circumstances beyond the control of the party” (`sources/cit/cit-rules-and-forms.txt:1986-1990`); the closed standard enum cannot express that alternative, so those grants are narrower than the rule. |
| CoFC `costs-period`, `patent-answer-period`, `patent-reply-period` | RCFC 6(b)(1), now bound as `RCFC (2010 ed.) 6(b)`, says the court may “for good cause” extend before expiry and, on post-expiry motion, for “excusable neglect” (`sources/rcfc-2010-time.txt`). Typed grant on the Rule periods. `post-judgment-motion-period` is locked by RCFC 6(b)(2): “The court must not extend the time to act under RCFC 52(b), 59(b), (d), and (e), and 60(b)” (`sources/rcfc.txt:1252`). |
| SCOTUS `opposition-period`, `merits-brief-period`, `respondent-brief-period`, `rehearing-period` | Rule 15.3 allows extension by “the Court or a Justice, or by the Clerk under Rule 30.4” (`sources/scotus/scotus-rules-2026.txt:681-682`); Rule 25.5 says Rule 25.1-3 periods “may be extended as provided in Rule 30” (`sources/scotus/scotus-rules-2026.txt:1351-1352`); Rule 30.4 assigns first instance action to the Clerk and permits submission to a Justice/Court (`sources/scotus/scotus-rules-2026.txt:1738-1765`); Rule 44.1 says “unless the Court or a Justice ... extends the time” (`sources/scotus/scotus-rules-2026.txt:2433-2435`). No new grant was typed: current package roles omit Clerk, and the grammar cannot represent the rule's unbounded discretion. Rule 44.2's existing lock is supported by “will not be extended” (`sources/scotus/scotus-rules-2026.txt:2462`). |
| CAAF periods in `lit/caaf/procedure.lit` | Rule 18(h) says “Unless otherwise prohibited by statute or these rules, a motion for extension of time may be sought within the time for filing the applicable pleading or brief” (`sources/caaf/caaf-rules.txt:730-731`). Rule 9(d) permits the Clerk, on the Court’s behalf, to act on unopposed extensions “not to exceed thirty days” (`sources/caaf/caaf-rules.txt:309-314`). No grant was typed because the package does not assign a Clerk role, Rule 18(h) states no grant standard, and Rule 9(d) is limited to Clerk action. The nine periods still reported in strict are individually listed below. |
| FRCP `removal-period`, `remand-motion-period` | FRCP 6(b)(1) applies when an act “may or must be done within a specified time” (`sources/frcp.txt:656-663`), but removal and remand clocks are statutory under 28 U.S.C. §§ 1446(b), 1447(c); the statute does not grant enlargement. Left open. |
| Habeas `state-petition-limitation`, `federal-motion-limitation`, `successive-authorization-ruling` | Statutory clocks under 28 U.S.C. § 2244(d)(1), which says “A 1-year period of limitation shall apply” (`sources/habeas/usc-28-ch153.txt:237`); § 2255(f), also “A 1-year period of limitation shall apply” (`sources/habeas/usc-28-ch153.txt:1647`); and § 2244(b)(3)(D) (`sources/habeas/usc-28-ch153.txt:216-220`). Those statutes state limitation/authorization time, not an extension grant. Left open. `proposed-findings-objections` remains undeclared because 28 U.S.C. § 636(b)(1) and 2254/2255 Rule 8(b) fix the objections clock at 14 days, per the PM ruling. |
| GAMD `response-period`, GAND `response-period`, GASD `response-period` | Each local rule sets the response clock (`sources/md-ga/mdga-local-rules-sectioned.txt:704-715`; `sources/nd-ga/ndga-civil-local-rules-sectioned.txt:938-950`; `sources/sd-ga/sdga-local-rules-sectioned.txt:604-611`). The court's FRCP 6(b)(1) authority covers an act within a specified time. Typed good-cause/pre-expiry and excusable-neglect/post-expiry forms. Local Clerk powers are not represented. |
| PTAB `institution-period`, `institution-period-on-waiver` | 35 U.S.C. § 314(b) says “The Director shall determine whether to institute an inter partes review ... within 3 months after” receiving the preliminary response or the last date to file it (`sources/usc-additional.txt:382-386`). The text sets the Director's decision clock and supplies no extension grant or prohibition. Left open; these are the two remaining PTAB strict errors. |
| PTAB `ipr-time-bar`, `pgr-filing-period` | The prohibitory text is the reason for the existing locks: § 315(b) says “An inter partes review may not be instituted if the petition ... is filed more than 1 year after” (`sources/usc-additional.txt:409`); § 321(c) says “may only be filed not later than” (`sources/usc-additional.txt:504`). These statutory locks are listed for PM review as a reading of prohibitive wording. |
| CAFC `appearance-period` | Fed. Cir. Rule 26(b) applies to periods prescribed by Federal Circuit rules or court order. A 2018 edition is now bound; the 2009 source proof remains open (see Kind B). |
| MPEP and ITC inline periods | MPEP (including 37 C.F.R. §§ 1.136(a), 41.37(e)) belongs to lg-pb1fe.36. ITC inline deadlines belong to lg-pb1fe.30. No MPEP or inline ITC deadline was changed. |

The six districts' sibling-package pattern was checked. Their reply-period grants follow the same FRCP 6(b)(1) forms; the new response-period grants use that same text. GAMD's LR 6.2 grants the Clerk one extension not exceeding 14 days (`sources/md-ga/mdga-local-rules-sectioned.txt:725-733`); GASD LR Civ. 6 allows the Clerk, with party consent, to extend a response deadline up to 14 days (`sources/sd-ga/sdga-local-rules-sectioned.txt:522-526`). The typed court authority derives from FRCP 6(b), not an expansion of those Clerk clauses.

## Kind B — source-version proofs

| Package / source / date | Finding and action | Pre-epic status |
|---|---|---|
| FRAP 26 and 31 / 2009-12-01 | Added official GovInfo 2009 Rule 26 and 2018 Rules 26/31 text to `sources/usc-28-app-frap-editions.txt`; bound and cited the period-specific editions. | Unknown; no 038a190cb comparison. |
| FRAP 26 and 31 / 2018-12-01 | Added official U.S. Courts 2018 rules PDF original and extracted Rules 26/31, with pinned source text and edition cites. | Unknown. |
| RCFC 6 / 2010-01-11 | Added official Court of Federal Claims 2010 PDF original and a pinned Rule 6 transcription; cites now bind the 2010 version. | Unknown. |
| Fed. Cir. R. 26 / 2018-12-01 | Added official December 2018 Federal Circuit Rules PDF original and Rule 26 text; updated the 2018 grant cite. | Unknown. |
| Fed. Cir. R. 26 / 2009-12-01 | Official archive search surfaced June 2011 and December 2018 editions, not a 2009 edition. The 2011 text was not substituted. | Unknown. |
| 37 C.F.R. § 2.116 / 2017-01-14 (TTAB) | Added the official eCFR versioner metadata in the recognized `## versions` form, updated the source hash and binding. | Unknown. |
| FRCP 6 / 2017 (TTAB) | Rebound the TTAB cite to the existing official 2017 edition text. | Unknown. |
| 19 C.F.R. § 210.6 / 2025-02-03 (ITC) | Added official eCFR versioner metadata for the cited section, updated source hash and binding. | Unknown. |


## Retry measurements and acceptance evidence

### Strict diagnostics that remain

Each row is one remaining package-owned error, identified by diagnostic span. The kind-A items remain open because the governing text does not grant or prohibit enlargement, or the typed language cannot express the actor/standard. The sole kind-B row needs a Federal Circuit Rule 26 edition not present in `sources/`.

| Package | Item / diagnostic location | Governing text and why it remains |
|---|---|---|
| caaf | `lit/caaf/procedure.lit:26` | CAAF R. 18(h), 9(d); no court grant standard is stated and Clerk delegation cannot be represented with the package actors. |
| caaf | `lit/caaf/procedure.lit:32` | Same CAAF R. 18(h), 9(d) limitation. |
| caaf | `lit/caaf/procedure.lit:46` | Same CAAF R. 18(h), 9(d) limitation. |
| caaf | `lit/caaf/procedure.lit:57` | Same CAAF R. 18(h), 9(d) limitation. |
| caaf | `lit/caaf/procedure.lit:83` | Same CAAF R. 18(h), 9(d) limitation. |
| caaf | `lit/caaf/procedure.lit:94` | Same CAAF R. 18(h), 9(d) limitation. |
| caaf | `lit/caaf/procedure.lit:111` | Same CAAF R. 18(h), 9(d) limitation. |
| caaf | `lit/caaf/procedure.lit:115` | Same CAAF R. 18(h), 9(d) limitation. |
| caaf | `lit/caaf/procedure.lit:119` | Same CAAF R. 18(h), 9(d) limitation. |
| cafc | `lit/cafc/06-briefing.lit:127` | Fed. Cir. R. 26 text history in `sources/rule-amendment-history.txt` records no version on or before 2009-12-01. The 2009 edition is not vendored; no substitute edition was invented. |
| cavc | `lit/cavc/procedure.lit:144` | CAVC R. 26(b) excepts this Federal Circuit appeal clock; Rule 4 does not govern this appeal. No governing extension text was found for this period. |
| cit | `lit/cit/procedure.lit:37` | USCIT R. 3(a)(2) restates 19 U.S.C. § 1516a(a)(2), fixing the complaint-after-summons clock; no extension grant appears in the vendored provision. |
| cit | `lit/cit/procedure.lit:43` | USCIT R. 6(b)(1) includes “circumstances beyond the control of the party,” which the closed typed standard cannot express. |
| frcp | `lit/frcp/02-pleading-to-trial.lit:109` | The removal deadline is statutory, 28 U.S.C. § 1446(b); no enlargement grant is stated in the vendored rule/statute. |
| frcp | `lit/frcp/02-pleading-to-trial.lit:135` | The remand motion clock is statutory, 28 U.S.C. § 1447(c); no enlargement grant is stated in the vendored rule/statute. |
| habeas | `lit/habeas/proceeding.lit:21` | 28 U.S.C. § 2244(d)(1) sets a one-year limitation; the statute states no extension grant. |
| habeas | `lit/habeas/proceeding.lit:27` | 28 U.S.C. § 2255(f) sets a one-year limitation; the statute states no extension grant. |
| habeas | `lit/habeas/proceeding.lit:44` | 28 U.S.C. § 2244(b)(3)(D) sets the appellate court's ruling period; no enlargement grant is stated. |
| habeas | `lit/habeas/proceeding.lit:69` | 28 U.S.C. § 636(b)(1) and 2254/2255 Rule 8(b) fix the objections period at 14 days. Left undeclared per the PM ruling. |
| ptab | `lit/ptab/01-pre-petition-to-institution.lit:240` | 35 U.S.C. § 314(b) sets the institution decision period but states neither an enlargement grant nor a prohibition. |
| ptab | `lit/ptab/01-pre-petition-to-institution.lit:256` | Same § 314(b) gap for the waived preliminary response path. |
| scotus | `lit/scotus/cert.lit:271` | Sup. Ct. R. 15.3 and 30.4 allow action by the Clerk, Justice, or Court; the package has no Clerk actor and the text supplies no typed standard. |
| scotus | `lit/scotus/cert.lit:313` | Sup. Ct. R. 30.4 assigns initial motion action to the Clerk; Clerk is not a package actor. |
| scotus | `lit/scotus/cert.lit:322` | Sup. Ct. R. 30.4 assigns initial motion action to the Clerk; Clerk is not a package actor. |
| scotus | `lit/scotus/cert.lit:357` | Sup. Ct. R. 44.1 allows extension by the Court or a Justice but supplies no typed standard. |

No MPEP strict errors remain: this retry corrected the MPEP versions-history pin to match the changed source file. PTAB's nine origin/main errors became two after the extension/source fixes and the corrected MPEP pin; its two remaining periods are the § 314(b) institution clocks above.

### Kind B source-version proofs

| Package / source / date | Finding and action | origin/main package result | Pre-epic status |
|---|---|---:|---|
| FRAP 26 and 31 / 2009-12-01 | Added official GovInfo 2009 Rule 26 and 2018 Rules 26/31 text and bound the period-specific editions. | FRAP: 10 L5002 | Unknown |
| FRAP 26 and 31 / 2018-12-01 | Added official U.S. Courts 2018 rules PDF original and extracted Rules 26/31 with pinned source text. | FRAP: 10 L5002 | Unknown |
| RCFC 6 / 2010-01-11 | Added official Court of Federal Claims 2010 PDF original and pinned Rule 6 transcription. | CoFC: 6 L5002 | Unknown |
| Fed. Cir. R. 26 / 2018-12-01 | Added official December 2018 Federal Circuit Rules PDF original and Rule 26 text; updated the 2018 grant cite. | CAFC: 3 L5002 | Unknown |
| Fed. Cir. R. 26 / 2009-12-01 | Official archive search surfaced June 2011 and December 2018 editions, not a 2009 edition. The 2011 text was not substituted; the proof remains open. | CAFC: 3 L5002 | Unknown |
| 37 C.F.R. § 2.116 / 2017-01-14 (TTAB) | Added official eCFR versioner metadata in `## versions` form and updated source hash/binding. | TTAB: 1 L5002 | Unknown |
| FRCP 6 / 2017 (TTAB) | Rebound the TTAB cite to the existing official 2017 edition text. | TTAB: 1 L5002 | Unknown |
| 19 C.F.R. § 210.6 / 2025-02-03 (ITC) | Added official eCFR versioner metadata and updated source hash/binding. | ITC: 1 L5002 | Unknown |

The branch-wide `path`/`path_sha256` audit found three mismatches already present on `origin/main` (`sources/frbp/frbp-9006-2024-12-01.txt`, `sources/ga-state-civil/ga-2026-state-holidays.txt`, and `sources/sd-criminal-trial-court/sd-sdcl-title-23a-criminal-procedure.txt`). Per the PM instruction, those unrelated pins were left unchanged. The changed `sources/ecfr-versions-title-37.txt` digest is `996b9ca2890196301e015f252bd593ad74efe4dfe13ddc367d11a984db714369`; MPEP, PTAB, and TTAB bind that value.

### Vendored original proofs

| Original | Official URL | SHA-256 |
|---|---|---|
| FRAP 2018 edition | https://www.uscourts.gov/sites/default/files/ap_rules_eff._dec._1_2018_1.pdf | `8fd6a3a5707799172e192c97e85bca42934e4efdbb7cb3021fc7fd793d41a6f0` |
| RCFC January 11, 2010 edition | https://www.uscfc.uscourts.gov/sites/cfc/files/rcfc_complete_20100111.pdf | `5cbd4d4d5ee96991ce12c23f2eabe05f490f107e485bba94d8330af77b9c48ae` |
| Federal Circuit Rules, December 2018 | https://www.cafc.uscourts.gov/wp-content/uploads/RulesProceduresAndForms/FederalCircuitRules/FederalCircuitRulesofPractice-December2018.pdf | `346c3c83cb999c4444cb9a4e339a840cdf5444df53c5e44f5847e4d84b68db89` |
| GovInfo 2009 Rule 26 HTML | https://www.govinfo.gov/content/pkg/USCODE-2009-title28/html/USCODE-2009-title28-app-federalru-rule26.htm | `9d89c57ac290e84335005725bcf962e323dd0294a5f9fd974330e210cebb5441` |

The four vendored originals were SHA-256 checked; their transcriptions were compared against the source originals.

### Lowered pack comparison

Built all 18 edited package packs using the release binary built on the matching tree: `origin/main` archive to `/tmp/out-old/packs`, this tree to `/tmp/out-new/packs`. `scripts/lowered-json-diff.py` reports 608 changed edge leaves. Per-pack counts (zero means no changed edge leaves):

| Package | Changed leaves from `scripts/lowered-json-diff.py` |
|---|---:|
| cafc | 13 |
| cavc | 120 |
| cit | 170 |
| cofc | 67 |
| frap | 10 |
| frcp | 34 |
| gamd | 26 |
| gand | 26 |
| gasd | 26 |
| habeas | 0 |
| itc | 0 |
| mpep | 0 |
| msnd | 26 |
| mssd | 26 |
| ptab | 30 |
| scotus | 5 |
| sdd | 26 |
| ttab | 3 |

A full recursive lowered-JSON comparison found changes confined to extension fields and extension-hole metadata, source declarations/pins, and source/world digests. Source metadata changes reflect new edition-specific bindings and resulting digests. No unrelated rule, duration, counting, probability, edge, or other lowered field changed. This includes changes from the original branch's extension typing and historical-source pins, not only the three retry corrections.

### Solve comparison

`litigate solve cases/frcp-civil-case.case` with the matching old and new release binaries returned the same value `-16655.0` and `policy_size=103` on both trees.

### Inline tests and targeted test results

FRCP's no-extension test is named “with no extension recorded the set day stands”; it establishes only that the set date remains absent a recorded extension. Inline tests have no expected-refusal form, so a deliberate invalid extension cannot be asserted as a passing test. Other extension grant families without matching `event X(until: Date) extends <period> to until` events remain “typed, not exercised” as listed above.

`litigate test` passed with 0 failed tests for every requested package:

| Package | Passed | Failed |
|---|---:|---:|
| cafc | 36 | 0 |
| habeas | 10 | 0 |
| frcp | 51 | 0 |
| mpep | 56 | 0 |
| ptab | 35 | 0 |
| ttab | 30 | 0 |
| cofc | 46 | 0 |
| frap | 42 | 0 |
| cit | 69 | 0 |
| cavc | 76 | 0 |
| caaf | 71 | 0 |
| scotus | 18 | 0 |
| itc | 33 | 0 |

`litigate check lit cases --format json`: exit 0, `ok=true`, 0 errors (1,686 non-error diagnostics). `litigate fmt --check lit cases --format json`: exit 0, `ok=true`, 0 diagnostics. `just lit-sync-check`: exit 0, “Lit.lock files and the shared clock library are up to date; no generated file is committed.”

Requested Rust filter `cargo nextest run -p litigate-cli --no-fail-fast -E 'binary(roundtrip) | binary(frcp_extensions) | binary(corpus) | binary(cases_port) | binary(core_stdlib)'`: 73 tests, 53 passed and 20 failed. The same `binary(frcp_extensions)` filter on an isolated `origin/main` archive with its own test binary produced the same 20 failures (26 tests: 6 passed, 20 failed), establishing that they predate this bead. Failures on both trees:

- `a_cert_extension_past_60_days_or_by_a_court_is_rejected`
- `a_justice_extends_the_cert_period_by_up_to_60_days`
- `a_late_federal_circuit_cert_application_needs_extraordinary_circumstances`
- `an_unvendored_cert_extension_does_not_reach_an_expired_period`
- `cert_extension_timing_follows_rules_13_5_and_30`
- `court_ordered_extensions_are_recorded`
- `frap_4a3_cross_appeal_runs_to_the_later_of_14_days_and_the_notice_period`
- `frap_4a5_caps_the_notice_of_appeal_extension`
- `frap_4a5_counts_the_cross_appeal_motion_window_from_the_later_date`
- `frap_4a5_extends_the_cross_appeal_period`
- `frap_4a5_motion_must_come_within_30_days_of_the_prescribed_time`
- `frap_4a5_order_on_day_40_allows_filing_through_day_54`
- `only_the_court_extends`
- `rule_6b1_after_expiry_without_a_request_before_or_a_motion_after_is_rejected`
- `rule_6b1_has_no_ceiling_and_reaches_an_expired_period_on_motion`
- `rule_6b1a_turns_on_when_the_request_was_made`
- `the_director_extends_the_ttab_civil_action_period`
- `the_director_not_the_court_extends_the_federal_circuit_ptab_window`
- `the_federal_circuit_cross_appeal_runs_to_the_later_of_14_days_and_frap_4a1`
- `the_waiver_answer_and_rule_15a1_periods_are_extendable`

The fixture failures report `applicant` is not an actor of `frap-appellate-procedure`; the current actor set is `appellant, appellee, court, either, justice, movant`. This is outside the bead's owned paths and was reproduced on origin/main.

### Open for PM

- CAFC `appearance-period`: GoodCause is read from FRAP 26(b) and Fed. Cir. R. 26(b)(3); current Fed. Cir. R. 26(b)(1) states no standard for an extension of 14 days or less. The discretionary typed grant has no `within` bound: its before-expiry form accepts a motion in the last seven days on good cause, while Rule 26(b)(1) requires extraordinary circumstances when the motion is made later than seven days before the sought date. Its after-expiry form carries ExtraordinaryCircumstances. Keep both forms pending a language form that represents the timing condition.
- Vendor the 2009 Federal Circuit Rule 26 edition for `lit/cafc/06-briefing.lit:127` (`FOLLOWUP: vendor Fed. Cir. R. 26 (2009 edition)`).
- A stale `path_sha256` on a versions-history source is not reported by `litigate check` in dev; strict exposes the missing text-in-force proof. This retry corrected MPEP and PTAB's pins to the changed `sources/ecfr-versions-title-37.txt` digest.
- PTAB: retain the existing statute-supported locks under the reading that prohibitory phrases govern extension: § 315(b) “may not be instituted if the petition ... is filed more than 1 year after” and § 321(c) “may only be filed not later than” (`sources/usc-additional.txt:409,504`).

## Commits

- `8637d178` CAVC corrections.
- `40cc8502` PTAB prohibitory statutory lock comments.
- `b99899cf` CIT statutory complaint clock.
- `fd317725` CoFC extension typing.
- `4c97c620`, `b873118f`, `23215083`, `3c58ca3b`, `22e00037`, `0fae7c90` district reply grants.
- `68f49221` CAFC appearance period.
- `fa081fcc` habeas objections period.
- `b5222ebd` TTAB eCFR history metadata.
- `85e10ce2` ITC eCFR history metadata.
- `c16a21d7` FRAP historical source text and bindings.
- `cc1782af` RCFC 6 (2010) historical source.
- `ace231e5` Federal Circuit Rule 26 (2018) historical source.
- `107119f2` district response grants.
- `2d70a8f4` regenerated FRCP, habeas, CAFC and MPEP package lock rows.
- `b9481437` regenerated the CAFC docketing lock after the final comment update.
- `452494b2` corrected MPEP and PTAB eCFR history pins; `e2e24e5d` corrected the CAFC Rule 26 comment; `7f5b8f9d` clarified the FRCP no-extension test name.
- `6382a9c7` regenerated the federal package locks; `8f4cd87c` quoted current Fed. Cir. R. 26(b) text verbatim and synced the final CAFC lock.

FOLLOWUP: vendor Fed. Cir. R. 26 (2009 edition)
