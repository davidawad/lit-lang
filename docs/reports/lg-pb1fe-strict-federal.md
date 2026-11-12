# Strict federal extension and version regression (lg-pb1fe.35)

## Status

**Partial.** The source-backed fixes are committed; 11 of the 20 per-package strict commands pass, while remaining package-owned holes and inherited dependency diagnostics are listed below. Open periods remain undeclared where the vendored governing text is silent, statutory, or cannot be represented by the typed extension forms. The strict sweep was run with `LITGRAPH_SOURCES=/work/repo/sources` using `target/release/litigate`.

Main baseline counts are from PM measurement at `5f745fe8`; no independent pre-epic `038a190cb` build was made, so per-item pre-epic status is unknown.

| Package | Main baseline L5002 | Remeasured L5002 | `check --profile strict` |
|---|---:|---:|:---:|
| ca7 | 2 | 0 | pass |
| caaf | 9 | 9 | fail |
| cafc | 3 | 2 | fail |
| cavc | 9 | 1 | fail |
| cit | 12 | 2 | fail |
| cofc | 6 | 2 | pass |
| frap | 10 | 0 | pass |
| frcp | 4 | 2 | fail |
| gamd | 2 | 1 | pass |
| gand | 2 | 1 | pass |
| gasd | 2 | 1 | pass |
| habeas | 4 | 4 | fail |
| itc | 1 | 0 | pass |
| mpep | 11 | 59 | fail |
| msnd | 2 | 1 | pass |
| mssd | 2 | 1 | pass |
| ptab | 9 | 4 | fail |
| scotus | 5 | 4 | fail |
| sdd | 2 | 1 | pass |
| ttab | 1 | 0 | pass |

The remeasured counts include dependency-only `unreachable dependency holes` where the strict command reports them; these are separated from package-owned holes in the detailed list. PTAB’s unrelated `L9004` is excluded from the L5002 table. MPEP source was not edited.

## Kind A — typed extension availability

A period was typed only where the cited text covers that court-rule or court-order period. The new FRCP-based grants use the existing grammar: court / GoodCause before expiry under FRCP 6(b)(1)(A), and court / ExcusableNeglect on a motion after expiry under FRCP 6(b)(1)(B). The six district packages inherit FRCP 6 for computation; GAMD, GAND, and GASD response periods now also carry that typed grant. The local clerk extension clauses are not typed because the packages have no clerk role.

| Package / period | Text read and result |
|---|---|
| CAVC `untimeliness-motion-period`, `record-before-agency-period`, `appellant-brief-period`, `secretary-brief-period`, and other Rule-set filings | CAVC Rule 26(b), `sources/cavc/cavc-rules-of-practice-2026-10-01.txt:747-754`: “The Court, on its own initiative or on a party's motion for good cause shown, may extend the time set by these Rules”; “not to exceed a total of 45 days”; beyond that “only for extraordinary circumstances.” GoodCause grants were retained through 45 days; ExtraordinaryCircumstances grants remain only on `MotionFiled()` before/after expiry. The Rule 36(a) Federal Circuit appeal period at `lit/cavc/procedure.lit:144` remains open: Rule 26(b) excepts a Notice of Appeal except as provided by Rule 4, and Rule 4 does not govern this Federal Circuit appeal. Rule 26(c)'s 15-day ceiling is in Rule 47 expedited proceedings (`sources/cavc/cavc-rules-of-practice-2026-10-01.txt:1872-1875`); none of the retained 45-day periods is a Rule 47 filing. |
| CIT `complaint-after-summons-period` | USCIT Rule 3(a)(2) restates the statutory clock. The practice comment says “As provided in section 516A(a)(2) and (3) ... a complaint must be filed within 30 days after the filing of the summons” (`sources/cit/cit-rules-and-forms.txt:580-582`). Rule 6 grants were removed from both edges. Other CIT periods use Rule 6(b)(1), whose text also allows “circumstances beyond the control of the party” (`sources/cit/cit-rules-and-forms.txt:1986-1990`); the closed standard enum cannot express that alternative, so those grants are narrower than the rule. |
| CoFC `costs-period`, `patent-answer-period`, `patent-reply-period` | RCFC 6(b)(1), now bound as `RCFC (2010 ed.) 6(b)`, says the court may “for good cause” extend before expiry and, on post-expiry motion, for “excusable neglect” (`sources/rcfc-2010-time.txt`). Typed grant on the Rule periods. `post-judgment-motion-period` is locked by RCFC 6(b)(2): “The court must not extend the time to act under RCFC 52(b), 59(b), (d), and (e), and 60(b)” (`sources/rcfc.txt:1252`). |
| SCOTUS `opposition-period`, `merits-brief-period`, `respondent-brief-period`, `rehearing-period` | Rule 15.3 allows extension by “the Court or a Justice, or by the Clerk under Rule 30.4” (`sources/scotus/scotus-rules-2026.txt:681-682`); Rule 25.5 says Rule 25.1-3 periods “may be extended as provided in Rule 30” (`sources/scotus/scotus-rules-2026.txt:1351-1352`); Rule 30.4 assigns first instance action to the Clerk and permits submission to a Justice/Court (`sources/scotus/scotus-rules-2026.txt:1738-1765`); Rule 44.1 says “unless the Court or a Justice ... extends the time” (`sources/scotus/scotus-rules-2026.txt:2433-2435`). No new grant was typed: current package roles omit Clerk, and the grammar cannot represent the rule's unbounded discretion. Rule 44.2's existing lock is supported by “will not be extended” (`sources/scotus/scotus-rules-2026.txt:2462`). |
| CAAF `petition-period`, `certificate-period`, `supplement-period`, `supplement-answer-period`, `certified-brief-period`, `mandatory-brief-period`, `mandate-period`, `reconsideration-period` | Rule 18(h) says “Unless otherwise prohibited by statute or these rules, a motion for extension of time may be sought within the time for filing the applicable pleading or brief” (`sources/caaf/caaf-rules.txt:730-731`). Rule 9(d) permits the Clerk, on the Court’s behalf, to act on unopposed extensions “not to exceed thirty days” (`sources/caaf/caaf-rules.txt:309-314`). No grant was typed because the package does not assign a Clerk role, Rule 18(h) states no grant standard, and Rule 9(d) is limited to Clerk action. These periods remain honest holes. |
| FRCP `removal-period`, `remand-motion-period` | FRCP 6(b)(1) applies when an act “may or must be done within a specified time” (`sources/frcp.txt:656-663`), but removal and remand clocks are statutory under 28 U.S.C. §§ 1446(b), 1447(c); the statute does not grant enlargement. Left open. |
| Habeas `state-petition-limitation`, `federal-motion-limitation`, `successive-authorization-ruling` | Statutory clocks under 28 U.S.C. § 2244(d)(1), which says “A 1-year period of limitation shall apply” (`sources/habeas/usc-28-ch153.txt:237`); § 2255(f), also “A 1-year period of limitation shall apply” (`sources/habeas/usc-28-ch153.txt:1647`); and § 2244(b)(3)(C) (`sources/habeas/usc-28-ch153.txt:216-220`). Those statutes state limitation/authorization time, not an extension grant. Left open. `proposed-findings-objections` was typed under 28 U.S.C. § 636(b)(1) and 2254 Rule 12; its period is set by the court. |
| GAMD `response-period`, GAND `response-period`, GASD `response-period` | Each local rule sets the response clock (`sources/md-ga/mdga-local-rules-sectioned.txt:704-715`; `sources/nd-ga/ndga-civil-local-rules-sectioned.txt:938-950`; `sources/sd-ga/sdga-local-rules-sectioned.txt:604-611`). The court's FRCP 6(b)(1) authority covers an act within a specified time. Typed good-cause/pre-expiry and excusable-neglect/post-expiry forms. Local Clerk powers are not represented. |
| PTAB `institution-period`, `institution-period-on-waiver` | 35 U.S.C. § 314(b) says “The Director shall determine whether to institute an inter partes review ... within 3 months after” receiving the preliminary response or the last date to file it (`sources/usc-additional.txt:382-386`). The text sets the Director's decision clock and supplies no extension grant or prohibition. Left open. |
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


## Attempt 3 measurements and acceptance evidence

- `cargo build --release -p litigate-cli` exited 0 (3m57s).
- `just lit-sync` exited 0. `just lit-sync-check` first reported the four regenerated, uncommitted locks stale (exit 1); after committing the regenerated rows in `2d70a8f4` and the final CAFC row in `b9481437`, it exited 0: “Lit.lock files and the shared clock library are up to date; no generated file is committed.” The lock commits change only `lit/cafc/Lit.lock`, `lit/frcp/Lit.lock`, `lit/habeas/Lit.lock`, and `lit/mpep/Lit.lock`; no MPEP source or package roles were edited.
- Targeted `litigate test`: CAFC 36 passed / 0 failed, habeas 10 / 0, FRCP 51 / 0. `litigate check lit/cafc`, `lit/habeas`, and `lit/frcp` each exited 0 with 0 errors. `litigate check lit cases` exited 0 with `ok=true`, 0 errors. `litigate fmt --check lit cases` exited 2 only for `L9004 lit/ptab/03-trial-to-appeal.lit:66` (owned by lg-pb1fe.33).
- The strict commands were measured separately for all 20 packages using `target/release/litigate check --profile strict lit/<pkg> --format json`, with `LITGRAPH_SOURCES=/work/repo/sources`. The table above gives the emitted L5002 count and command verdict; dependency-only `unreachable dependency holes` are included. Package-owned remaining holes and their one-line reasons follow.

| Package | Remaining L5002 location | Reason |
|---|---|---|
| caaf | `lit/caaf/procedure.lit:26` | No typed extension grant, delegation, or non-extendable lock is declared. |
| caaf | `lit/caaf/procedure.lit:32` | No typed extension grant, delegation, or non-extendable lock is declared. |
| caaf | `lit/caaf/procedure.lit:46` | No typed extension grant, delegation, or non-extendable lock is declared. |
| caaf | `lit/caaf/procedure.lit:57` | No typed extension grant, delegation, or non-extendable lock is declared. |
| caaf | `lit/caaf/procedure.lit:83` | No typed extension grant, delegation, or non-extendable lock is declared. |
| caaf | `lit/caaf/procedure.lit:94` | No typed extension grant, delegation, or non-extendable lock is declared. |
| caaf | `lit/caaf/procedure.lit:111` | No typed extension grant, delegation, or non-extendable lock is declared. |
| caaf | `lit/caaf/procedure.lit:115` | No typed extension grant, delegation, or non-extendable lock is declared. |
| caaf | `lit/caaf/procedure.lit:119` | No typed extension grant, delegation, or non-extendable lock is declared. |
| cafc | `lit/cafc/../mpep/01-pre-filing.lit:36` | Dependency package mpep-prosecution has extension/source holes. |
| cafc | `lit/cafc/06-briefing.lit:127` | Cited rule text is not proved in force for the effective date. |
| cavc | `lit/cavc/procedure.lit:144` | No typed extension grant, delegation, or non-extendable lock is declared. |
| cit | `lit/cit/procedure.lit:37` | No typed extension grant, delegation, or non-extendable lock is declared. |
| cit | `lit/cit/procedure.lit:43` | No typed extension grant, delegation, or non-extendable lock is declared. |
| cofc | `lit/cofc/../cafc/../mpep/01-pre-filing.lit:36` | Dependency package mpep-prosecution has extension/source holes. |
| cofc | `lit/cofc/../cafc/06-briefing.lit:127` | Dependency package cafc has extension/source holes. |
| frcp | `lit/frcp/02-pleading-to-trial.lit:109` | No typed extension grant, delegation, or non-extendable lock is declared. |
| frcp | `lit/frcp/02-pleading-to-trial.lit:135` | No typed extension grant, delegation, or non-extendable lock is declared. |
| gamd | `lit/gamd/../frcp/02-pleading-to-trial.lit:109` | Dependency package frcp-civil-procedure has extension/source holes. |
| gand | `lit/gand/../frcp/02-pleading-to-trial.lit:109` | Dependency package frcp-civil-procedure has extension/source holes. |
| gasd | `lit/gasd/../frcp/02-pleading-to-trial.lit:109` | Dependency package frcp-civil-procedure has extension/source holes. |
| habeas | `lit/habeas/proceeding.lit:21` | No typed extension grant, delegation, or non-extendable lock is declared. |
| habeas | `lit/habeas/proceeding.lit:27` | No typed extension grant, delegation, or non-extendable lock is declared. |
| habeas | `lit/habeas/proceeding.lit:44` | No typed extension grant, delegation, or non-extendable lock is declared. |
| habeas | `lit/habeas/proceeding.lit:69` | No typed extension grant, delegation, or non-extendable lock is declared. |
| mpep | `lit/mpep/01-pre-filing.lit:36` | No typed extension grant, delegation, or non-extendable lock is declared. |
| mpep | `lit/mpep/01-pre-filing.lit:74` | No typed extension grant, delegation, or non-extendable lock is declared. |
| mpep | `lit/mpep/02-filing-to-allowance.lit:25` | No pinned amendment history for 37 C.F.R. § 1.53. |
| mpep | `lit/mpep/02-filing-to-allowance.lit:28` | No pinned amendment history for 37 C.F.R. § 1.104. |
| mpep | `lit/mpep/02-filing-to-allowance.lit:33` | No pinned amendment history for 37 C.F.R. § 1.53. |
| mpep | `lit/mpep/02-filing-to-allowance.lit:37` | No pinned amendment history for 37 C.F.R. § 1.53. |
| mpep | `lit/mpep/02-filing-to-allowance.lit:77` | No pinned amendment history for 37 C.F.R. § 1.104. |
| mpep | `lit/mpep/02-filing-to-allowance.lit:81` | No pinned amendment history for 37 C.F.R. § 1.104. |
| mpep | `lit/mpep/02-filing-to-allowance.lit:88` | No pinned amendment history for 37 C.F.R. § 1.142. |
| mpep | `lit/mpep/02-filing-to-allowance.lit:94` | No pinned amendment history for 37 C.F.R. § 1.104. |
| mpep | `lit/mpep/02-filing-to-allowance.lit:98` | No pinned amendment history for 37 C.F.R. § 1.134. |
| mpep | `lit/mpep/02-filing-to-allowance.lit:117` | No pinned amendment history for 37 C.F.R. § 1.17. |
| mpep | `lit/mpep/02-filing-to-allowance.lit:132` | No pinned amendment history for 37 C.F.R. § 1.133. |
| mpep | `lit/mpep/02-filing-to-allowance.lit:151` | No typed extension grant, delegation, or non-extendable lock is declared. |
| mpep | `lit/mpep/02-filing-to-allowance.lit:172` | No pinned amendment history for 37 C.F.R. § 1.17. |
| mpep | `lit/mpep/02-filing-to-allowance.lit:182` | No pinned amendment history for 37 C.F.R. § 1.17. |
| mpep | `lit/mpep/02-filing-to-allowance.lit:198` | No typed extension grant, delegation, or non-extendable lock is declared. |
| mpep | `lit/mpep/02-filing-to-allowance.lit:204` | No typed extension grant, delegation, or non-extendable lock is declared. |
| mpep | `lit/mpep/02-filing-to-allowance.lit:221` | No pinned amendment history for 37 C.F.R. § 1.104. |
| mpep | `lit/mpep/02-filing-to-allowance.lit:229` | No pinned amendment history for 37 C.F.R. § 1.133. |
| mpep | `lit/mpep/02-filing-to-allowance.lit:279` | No pinned amendment history for 37 C.F.R. § 1.134. |
| mpep | `lit/mpep/02-filing-to-allowance.lit:408` | No pinned amendment history for 37 C.F.R. § 1.104. |
| mpep | `lit/mpep/02-filing-to-allowance.lit:418` | No typed extension grant, delegation, or non-extendable lock is declared. |
| mpep | `lit/mpep/02-filing-to-allowance.lit:427` | No typed extension grant, delegation, or non-extendable lock is declared. |
| mpep | `lit/mpep/02-filing-to-allowance.lit:472` | No typed extension grant, delegation, or non-extendable lock is declared. |
| mpep | `lit/mpep/02-filing-to-allowance.lit:571` | No pinned amendment history for 37 C.F.R. § 1.142. |
| mpep | `lit/mpep/02-filing-to-allowance.lit:575` | No pinned amendment history for 37 C.F.R. § 1.142. |
| mpep | `lit/mpep/02-filing-to-allowance.lit:583` | No pinned amendment history for 37 C.F.R. § 1.143. |
| mpep | `lit/mpep/02-filing-to-allowance.lit:593` | No pinned amendment history for 37 C.F.R. § 1.143. |
| mpep | `lit/mpep/02-filing-to-allowance.lit:597` | No pinned amendment history for 37 C.F.R. § 1.104. |
| mpep | `lit/mpep/02-filing-to-allowance.lit:627` | No pinned amendment history for 37 C.F.R. § 1.104. |
| mpep | `lit/mpep/02-filing-to-allowance.lit:636` | No pinned amendment history for 37 C.F.R. § 1.104. |
| mpep | `lit/mpep/02-filing-to-allowance.lit:656` | No pinned amendment history for 37 C.F.R. § 1.18. |
| mpep | `lit/mpep/02-filing-to-allowance.lit:696` | No pinned amendment history for 37 C.F.R. § 1.18. |
| mpep | `lit/mpep/02-filing-to-allowance.lit:717` | No pinned amendment history for 37 C.F.R. § 1.171. |
| mpep | `lit/mpep/02-filing-to-allowance.lit:762` | No typed extension grant, delegation, or non-extendable lock is declared. |
| mpep | `lit/mpep/02-filing-to-allowance.lit:785` | No pinned amendment history for 37 C.F.R. § 1.20. |
| mpep | `lit/mpep/02-filing-to-allowance.lit:823` | No pinned amendment history for 37 C.F.R. § 1.137. |
| mpep | `lit/mpep/02-filing-to-allowance.lit:865` | No pinned amendment history for 37 C.F.R. § 1.137. |
| mpep | `lit/mpep/03-post-grant.lit:36` | No pinned amendment history for 37 C.F.R. § 1.510. |
| mpep | `lit/mpep/03-post-grant.lit:47` | No typed extension grant, delegation, or non-extendable lock is declared. |
| mpep | `lit/mpep/03-post-grant.lit:56` | No typed extension grant, delegation, or non-extendable lock is declared. |
| mpep | `lit/mpep/03-post-grant.lit:199` | No pinned amendment history for 37 C.F.R. § 1.510. |
| mpep | `lit/mpep/03-post-grant.lit:204` | No pinned amendment history for 37 C.F.R. § 1.515. |
| mpep | `lit/mpep/03-post-grant.lit:207` | No pinned amendment history for 37 C.F.R. § 1.515. |
| mpep | `lit/mpep/03-post-grant.lit:213` | No pinned amendment history for 37 C.F.R. § 1.530. |
| mpep | `lit/mpep/03-post-grant.lit:217` | No pinned amendment history for 37 C.F.R. § 1.530. |
| mpep | `lit/mpep/03-post-grant.lit:226` | No pinned amendment history for 37 C.F.R. § 1.530. |
| mpep | `lit/mpep/03-post-grant.lit:233` | No pinned amendment history for 37 C.F.R. § 1.550. |
| mpep | `lit/mpep/03-post-grant.lit:237` | No pinned amendment history for 37 C.F.R. § 1.570. |
| mpep | `lit/mpep/03-post-grant.lit:240` | No pinned amendment history for 37 C.F.R. § 1.570. |
| mpep | `lit/mpep/03-post-grant.lit:243` | No pinned amendment history for 37 C.F.R. § 1.570. |
| mpep | `lit/mpep/04-reissue.lit:6` | No pinned amendment history for 37 C.F.R. § 1.171. |
| mpep | `lit/mpep/deadlines.lit:110` | No pinned amendment history for 37 C.F.R. § 1.53. |
| mpep | `lit/mpep/deadlines.lit:140` | No pinned amendment history for 37 C.F.R. § 1.142. |
| mpep | `lit/mpep/deadlines.lit:164` | No pinned amendment history for 37 C.F.R. § 1.134. |
| mpep | `lit/mpep/deadlines.lit:269` | No pinned amendment history for 37 C.F.R. § 1.134. |
| mpep | `lit/mpep/deadlines.lit:465` | No pinned amendment history for 37 C.F.R. § 1.530. |
| mpep | `lit/mpep/deadlines.lit:465` | No pinned amendment history for 37 C.F.R. § 1.550. |
| msnd | `lit/msnd/../frcp/02-pleading-to-trial.lit:109` | Dependency package frcp-civil-procedure has extension/source holes. |
| mssd | `lit/mssd/../frcp/02-pleading-to-trial.lit:109` | Dependency package frcp-civil-procedure has extension/source holes. |
| ptab | `lit/ptab/../cafc/06-briefing.lit:127` | Dependency package cafc has extension/source holes. |
| ptab | `lit/ptab/../mpep/01-pre-filing.lit:36` | Dependency package mpep-prosecution has extension/source holes. |
| ptab | `lit/ptab/01-pre-petition-to-institution.lit:240` | No typed extension grant, delegation, or non-extendable lock is declared. |
| ptab | `lit/ptab/01-pre-petition-to-institution.lit:256` | No typed extension grant, delegation, or non-extendable lock is declared. |
| scotus | `lit/scotus/cert.lit:271` | No typed extension grant, delegation, or non-extendable lock is declared. |
| scotus | `lit/scotus/cert.lit:313` | No typed extension grant, delegation, or non-extendable lock is declared. |
| scotus | `lit/scotus/cert.lit:322` | No typed extension grant, delegation, or non-extendable lock is declared. |
| scotus | `lit/scotus/cert.lit:357` | No typed extension grant, delegation, or non-extendable lock is declared. |
| sdd | `lit/sdd/../frcp/02-pleading-to-trial.lit:109` | Dependency package frcp-civil-procedure has extension/source holes. |

### Source proofs

| Original | Official URL | SHA-256 |
|---|---|---|
| FRAP 2018 edition | https://www.uscourts.gov/sites/default/files/ap_rules_eff._dec._1_2018_1.pdf | `8fd6a3a5707799172e192c97e85bca42934e4efdbb7cb3021fc7fd793d41a6f0` |
| RCFC January 11, 2010 edition | https://www.uscfc.uscourts.gov/sites/cfc/files/rcfc_complete_20100111.pdf | `5cbd4d4d5ee96991ce12c23f2eabe05f490f107e485bba94d8330af77b9c48ae` |
| Federal Circuit Rules, December 2018 | https://www.cafc.uscourts.gov/wp-content/uploads/RulesProceduresAndForms/FederalCircuitRules/FederalCircuitRulesofPractice-December2018.pdf | `346c3c83cb999c4444cb9a4e339a840cdf5444df53c5e44f5847e4d84b68db89` |
| GovInfo 2009 Rule 26 HTML | https://www.govinfo.gov/content/pkg/USCODE-2009-title28/html/USCODE-2009-title28-app-federalru-rule26.htm | `9d89c57ac290e84335005725bcf962e323dd0294a5f9fd974330e210cebb5441` |

The four originals were SHA-256 checked in this run. PM review recorded that each vendored original is byte-identical to a fresh official download and that each transcription was compared sentence by sentence with the PDF text.

### Main-to-branch lowered-pack diff

Built all 18 edited packages (including the MPEP lock-only package) with the same release binary, once on this branch into `/tmp/out-new/packs` and once in a separate clone checked out at `origin/main` (`5f745fe8`) into `/tmp/out-old/packs`. `scripts/lowered-json-diff.py` reports 626 changed lowered leaves. For all changed packs, the edge changes are typed extension fields (`extendable`, `extensionAuthority`, `extensions`) and the relevant source/pin/world digests. A full JSON leaf comparison found no changed duration, counting rule, probability, graph edge, or non-extension field, except for MPEP’s new `/lowered/holes/authority` leaves: 68 authority holes appear because the shared eCFR version-source digest changed while the cited MPEP sections still lack their own pinned `## versions` histories. This is a text-in-force proof diagnostic, not an extension change; MPEP source remains owned by lg-pb1fe.36. The generated diff table is `/tmp/lg-pb1fe-lowered-diff.md.

### Solve regression

`target/release/litigate solve cases/frcp-civil-case.case` exited 0 with value `-16655.0`, `policy_size=103`. Repeating the same command and binary in the separate `origin/main` clone gave the same value and policy size.

### Inline extension coverage

FRCP `FeeSupplementExtended(until)` now has an inline accepted before-expiry extension case and a no-extension post-expiry case, which leaves the scheduled date unchanged absent a motion. The test language evaluates `given` extension orders against their grants and rejects an invalid order; it has no assertion form for an expected diagnostic, so a deliberately invalid order cannot be an inline test that passes. The other typed grant families without `event X(until: Date) extends <period> to until` events are typed, not exercised: CAVC, CIT, CoFC, the six district packages, habeas, ITC, SCOTUS, and most CAFC periods; FRAP’s event-bearing cert period, FRCP period events, PTAB appeal, and TTAB answer/civil-action period do have matching extension events.

### Open for David

DECIDED: keep habeas proposed-findings objections undeclared because 28 U.S.C. § 636(b)(1) and the habeas rules both fix the 14-day period, matching the PM ruling for similarly fixed statutory/rule periods.

- CAFC: vendor the 2009 Federal Circuit Rule 26 edition for `lit/cafc/06-briefing.lit:127` (`FOLLOWUP: vendor Fed. Cir. R. 26 (2009 edition)`).
- PTAB: retain the existing locks under the reading that prohibitory phrases govern extension: § 315(b) “may not be instituted if the petition ... is filed more than 1 year after” and § 321(c) “may only be filed not later than” (`sources/usc-additional.txt:409,504`).
- CAVC: Rule 36(a) Federal Circuit appeal clock at `lit/cavc/procedure.lit:144`; CAVC Rule 26(b) excepts a Notice of Appeal except as Rule 4 provides, and Rule 4 does not govern this Federal Circuit appeal.
- CIT: complaint-after-summons clock is fixed by statute (USCIT Rule 3(a)(2) practice comment: “a complaint must be filed within 30 days after the filing of the summons,” `sources/cit/cit-rules-and-forms.txt:580-582`). Other Rule 6(b)(1) extension typing cannot express the rule’s “circumstances beyond the control of the party” standard (`sources/cit/cit-rules-and-forms.txt:1986-1990`).
- Habeas: keep `proposed-findings-objections` undeclared because statute and rule both fix 14 days. 28 U.S.C. § 636(b)(1): “Within fourteen days after being served with a copy, any party may serve and file written objections”; 2254/2255 Rule 8(b): “Within 14 days after being served, a party may file objections as provided by local court rule.”
- SCOTUS: the grant cannot encode Clerk as the actor or the unstated standard. Rule 15.3: “Any brief in opposition shall be filed within 30 days after the case is placed on the docket, unless the time is extended by the Court or a Justice, or by the Clerk under Rule 30.4.” Rule 30.4: “The motion may be acted on by the Clerk in the first instance, and any party aggrieved by the Clerk’s action may request that the motion be submitted to a Justice or to the Court.” (`sources/scotus/scotus-rules-2026.txt:681-682,1755-1762`).
- CAAF: Rule 9(d) delegates only to Clerk and Rule 18(h) states no standard: “Unless otherwise prohibited by statute or these rules, a motion for extension of time may be sought within the time for filing the applicable pleading or brief” (`sources/caaf/caaf-rules.txt:730-731`); Rule 9(d): “The Clerk, on the Court’s behalf, may entertain and act on the following motions, provided such motion is not opposed and such action does not substantially affect the rights of the parties or the ultimate decision in the case: any motion seeking an extension of time not to exceed thirty days” (`sources/caaf/caaf-rules.txt:309-314`).
- CoFC: RCFC 6(b)(1) extension forms are on-motion-only; the statute/rule source requires motion facts, so no no-motion court grant was added. See `sources/rcfc-2010-time.txt`.
- PTAB institution decision clocks under 35 U.S.C. § 314(b) set a deadline but state no enlargement grant or prohibition. Habeas statutory limitation/authorization clocks remain open as listed above.

`litigate check lit cases`: exit 0, 0 errors. `litigate fmt --check lit cases`: exit 2 solely for the PTAB L9004 above. `just lit-sync-check`: exit 0 after the lock commits `2d70a8f4` and `b9481437`.
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

FOLLOWUP: vendor Fed. Cir. R. 26 (2009 edition)
