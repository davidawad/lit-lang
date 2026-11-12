# Strict federal extension and version regression (lg-pb1fe.35)

## Status

**Partial.** The source-backed fixes below are committed, but strict is not clean across the requested 20 packages. Open periods remain undeclared where the vendored governing text is silent, statutory, or cannot be represented by the typed extension forms. The strict sweep was run with `LITGRAPH_SOURCES=/work/repo/sources` using `target/release/litigate`.

The previous attempt's strict counts are treated as the origin/main baseline (the PM provided them). No independent pre-epic `038a190cb` build was made, so per-item pre-epic status is unknown. Main baseline: 0/20 strict passes.

| Package | Main baseline errors | Current strict errors | Current strict pass |
|---|---:|---:|:---:|
| ca7 | 2 | 0 | yes |
| caaf | 9 | 9 | no |
| cafc | 3 | 3 | no |
| cavc | 9 | 1 | no |
| cit | 12 | 2 | no |
| cofc | 6 | 3 | no |
| frap | 10 | 0 | yes |
| frcp | 4 | 2 | no |
| gamd | 2 | 1 | no |
| gand | 2 | 1 | no |
| gasd | 2 | 1 | no |
| habeas | 4 | 3 | no |
| itc | 1 | 0 | yes |
| mpep | 11 | 60 | no |
| msnd | 2 | 1 | no |
| mssd | 2 | 1 | no |
| ptab | 9 | 7 | no |
| scotus | 5 | 4 | no |
| sdd | 2 | 1 | no |
| ttab | 1 | 0 | yes |

Current `cafc`, `cofc`, `gamd`, `gand`, `gasd`, `msnd`, `mssd`, `ptab`, and `sdd` totals include dependency errors caused by open FRCP/Cafc/MPEP diagnostics or stale MPEP lock state; the `ptab` count also includes its known `L9004` owned by lg-pb1fe.33. The MPEP package itself is owned by lg-pb1fe.36 and was not edited.

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

`FOLLOWUP: vendor Fed. Cir. R. 26 (2009 edition)` — official archive page checked: https://www.cafc.uscourts.gov/home/rules-procedures-forms/rules-of-practice-archives/ .

## Other acceptance evidence and remaining gaps

- The single `cargo build --release` attempt failed in the litgraph-packs build script (exit 101). The existing release binary was available and used for scoped authoring commands.
- `just lit-sync` completed. `just lit-sync-check` exited 1: it regenerated `lit/mpep/Lit.lock` and `lit/ptab/Lit.lock`, then reported those locks stale. The PTAB lock change is from the § 2.116 version-history binding in this bead and is staged for commit. The MPEP lock change reflects `sources/ecfr-versions-title-37.txt` work owned by lg-pb1fe.36; David directed no MPEP edits, so it is not committed here. This leaves the required sync-check acceptance unmet.
- `target/release/litigate fmt --check lit cases --format json` exited 2 on the known `L9004` in `lit/ptab/03-trial-to-appeal.lit:66`, owned by lg-pb1fe.33.
- `target/release/litigate check lit cases --format json` exited 0; `ok=true`, 0 errors.
- Targeted package tests passed: CAVC 76/0, CIT 69/0, PTAB 35/0, SCOTUS 18/0, CoFC 46/0, FRAP 42/0, CAFC 36/0, GAMD 18/0, GAND 36/0, GASD 22/0, Habeas 10/0, ITC 33/0, TTAB 30/0, MSND 0 failures, MSSD 0 failures, SDD 0 failures. FRCP 49/0. These are package test counts; all commands exited 0.
- Lowered-pack diff was not produced for this retry. Source edits only changed typed extension fields/cites, source bindings, and explanatory notes; durations, counting rules, probabilities, graph edges, and expectations were not intentionally changed. A machine diff remains an acceptance gap.
- `target/release/litigate solve cases/frcp-civil-case.case` with `LITGRAPH_SOURCES=/work/repo/sources` exited 0 and returned value `-16655.0`. The origin/main value was not captured in this attempt, so an unchanged-value comparison remains unverified.
- Inline tests accepting/refusing an extension were not added; the existing DSL tests deadline arithmetic, not extension decisions. This remains an acceptance gap.
- Open for David: CAVC Federal Circuit Rule 36(a) appeal period; CIT statutory 30-day complaint clock; CAAF periods listed above; SCOTUS periods listed above; statutory FRCP removal/remand; habeas statutory periods; PTAB institution clocks; PTAB locks under prohibitory reading; Federal Circuit 2009 Rule 26 source proof; and the CIT standard enum's narrower representation of Rule 6(b)(1)(B).

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
