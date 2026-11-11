# Extension metadata disagreement census (lg-pb1fe.5)

Baseline: `cfa5438754575bc1167491e8424318e48d2f50b7`, `litigate 0.7.1`. The baseline filing-linter command accepted `PreliminaryResponseFiled()`, discharged `popr-period` due 2026-09-18, and opened `institution-period` due 2026-12-10; its complete JSON is `/tmp/lg-pb1fe5-lint-before.txt`. `rg -n '^\s+(extendable|extension_authority) ' lit --glob '*.lit' | wc -l` returned **316** across 39 files. Of these, 196 were edge fields; the other 120 were duplicate options on period declarations.

The table is a source scan of the baseline edge fields, joined to matching typed period declarations in that forum's package. `true` means a typed grant/delegation includes `extend` or `set`; `false` means an explicit typed `extend { never ... }` lock; `?` means neither exists. Each row carries the original source location, old boolean and old authority string, the current typed declaration location/cite, and the resolution. A named-authority `lock against [...] cite "X"` with no extension grant or extension/set delegation lowers `extendable: false` and authority `X`; it is not a hole. For other `?` rows, there is no typed authority to cite, so the old text is retained here as the recorded claim and the lowered value is an explicit hole.

Conflicts found: **78**. Typed grants/delegations resolving to true: **17**. Typed locks resolving to false: **0**. Missing typed grants/locks resolving to holes: **61**. The 17 true resolutions contradict legacy false assertions. The 61 holes replace legacy booleans that were not backed by a typed declaration (including false values, which are no longer silently encoded).

| Baseline edge source | Period | Old flag | Old authority string | Typed declaration and cite | Resolution |
|---|---|---:|---|---|---|
| `lit/cafc/01-entry.lit:88` | `itc-noa-period` | `false` | — | lit/cafc/01-entry.lit:228 cite `19 U.S.C. § 1337(c)` | typed grant/delegation says true; old false discarded |
| `lit/cafc/02-docketing.lit:28` | `appearance-period` | `false` | — | lit/cafc/02-docketing.lit:66 cite `Fed. Cir. R. 47.3(b)(1); Fed. Cir. R. 47.4(b)` | hole: no typed grant, delegation, or explicit lock; old boolean discarded |
| `lit/cafc/09-decision-to-rehearing.lit:186` | `rehearing-period` | `false` | — | lit/cafc/09-decision-to-rehearing.lit:253 cite `Fed. Cir. R. 40(f)(1); FRAP 40(d)(1)` | typed grant/delegation says true; old false discarded |
| `lit/cofc/03-pleading.lit:63` | `answer-after-ruling` | `false` | — | lit/cofc/rules-pleading.lit:59 cite `RCFC 12 (2010 Amendment note)`; lit/cofc/rules-pleading.lit:66 cite `RCFC 12(a)(4)(A)` | typed grant/delegation says true; old false discarded |
| `lit/cofc/05-patent-track.lit:181` | `settlement-discussion-period` | `false` | — | lit/cofc/rules-patent.lit:100 cite `PRCFC 16(a)` | hole: no typed grant, delegation, or explicit lock; old boolean discarded |
| `lit/cofc/05-patent-track.lit:194` | `settlement-statement-period` | `false` | — | lit/cofc/rules-patent.lit:105 cite `PRCFC 17` | hole: no typed grant, delegation, or explicit lock; old boolean discarded |
| `lit/cofc/06-trial-post-trial.lit:42` | `post-judgment-motion-period` | `false` | none -- RCFC 6(b) forbids extension | lit/cofc/rules-trial.lit:23 cite `RCFC 59 (2010 Amendment note)`; lit/cofc/rules-trial.lit:27 cite `RCFC 59 (2010 Amendment note)`; lit/cofc/rules-trial.lit:31 cite `RCFC 59(b)(1)` | hole: no typed grant, delegation, or explicit lock; old boolean discarded |
| `lit/cofc/06-trial-post-trial.lit:85` | `costs-period` | `false` | — | lit/cofc/rules-trial.lit:41 cite `RCFC 54 (2010 Amendment note)` | hole: no typed grant, delegation, or explicit lock; old boolean discarded |
| `lit/frap/01-threshold.lit:88` | `permission-petition-period` | `false` | Fed. R. App. P. 5(a)(3) | lit/frap/01-threshold.lit:118 cite `FRAP (2009 ed.) 5(a)(2)` | hole: no typed grant, delegation, or explicit lock; old boolean discarded |
| `lit/frap/02-notice.lit:77` | `notice-civil-period` | `false` | FRAP 4(a)(5) (may extend up to 30 additional days only via motion before expiration) | lit/frap/appeal-periods.lit:43 cite `FRAP (2002 ed.) 4(a)(1)(A)`; lit/frap/appeal-periods.lit:115 cite `28 U.S.C. § 2107(a)` | typed grant/delegation says true; old false discarded |
| `lit/frap/02-notice.lit:90` | `notice-us-party-period` | `false` | FRAP 4(a)(5) (may extend up to 30 additional days only via motion before expiration) | lit/frap/appeal-periods.lit:57 cite `FRAP (2002 ed.) 4(a)(1)(B)`; lit/frap/appeal-periods.lit:130 cite `28 U.S.C. § 2107(b)` | typed grant/delegation says true; old false discarded |
| `lit/frap/02-notice.lit:103` | `notice-criminal-period` | `false` | FRAP 4(b)(4) (may be extended for excusable neglect or good cause, subject to Bowles jurisdictional limit) | lit/frap/02-notice.lit:21 cite `FRAP (2009 ed.) 4(b)(1)(A)` | typed grant/delegation says true; old false discarded |
| `lit/frap/02-notice.lit:115` | `tolling-motion-period` | `false` | FRCP 6(b) may extend the tolling-motion deadline by up to 28 days total (only for FRCP 50(b), 59(e), 60(b) motions) | lit/frap/02-notice.lit:39 cite `FRCP 59(e)` | hole: no typed grant, delegation, or explicit lock; old boolean discarded |
| `lit/frap/02-notice.lit:169` | `notice-civil-period` | `false` | FRAP 4(a)(5) (extension may apply to the restarted deadline) | lit/frap/appeal-periods.lit:43 cite `FRAP (2002 ed.) 4(a)(1)(A)`; lit/frap/appeal-periods.lit:115 cite `28 U.S.C. § 2107(a)` | typed grant/delegation says true; old false discarded |
| `lit/frap/04-record.lit:67` | `transcript-order-period` | `false` | — | lit/frap/04-record.lit:18 cite `FRAP 10(b)(1)` | typed grant/delegation says true; old false discarded |
| `lit/frap/04-record.lit:77` | `transcript-order-period` | `false` | — | lit/frap/04-record.lit:18 cite `FRAP 10(b)(1)` | typed grant/delegation says true; old false discarded |
| `lit/frap/09-rehearing.lit:75` | `rehearing-period` | `false` | FRAP 26(b) is silent on FRAP 40 deadlines; extension is theoretically possible for good cause but extremely rarely granted in practice | lit/frap/09-rehearing.lit:19 cite `FRAP (2009 ed.) 40(a)(1); FRAP (2009 ed.) 35(c)`; lit/frap/09-rehearing.lit:26 cite `FRAP 40(d)(1)` | typed grant/delegation says true; old false discarded |
| `lit/frcp/02-pleading-to-trial.lit:121` | `removal-period` | `false` | — | lit/frcp/periods.lit:251 cite `28 U.S.C. 1446(b)(1)` | hole: no typed grant, delegation, or explicit lock; old boolean discarded |
| `lit/frcp/02-pleading-to-trial.lit:150` | `remand-motion-period` | `false` | — | lit/frcp/periods.lit:256 cite `28 U.S.C. 1447(c)` | hole: no typed grant, delegation, or explicit lock; old boolean discarded |
| `lit/frcp/03-post-trial-motions.lit:62` | `jmol-renewal-period` | `false` | — | lit/frcp/periods.lit:419 cite `FRCP 50(b)` | hole: no typed grant, delegation, or explicit lock; old boolean discarded |
| `lit/frcp/03-post-trial-motions.lit:72` | `new-trial-period` | `false` | — | lit/frcp/periods.lit:427 cite `FRCP 59(b)` | hole: no typed grant, delegation, or explicit lock; old boolean discarded |
| `lit/frcp/03-post-trial-motions.lit:82` | `alter-amend-period` | `false` | — | lit/frcp/periods.lit:435 cite `FRCP 59(e)` | hole: no typed grant, delegation, or explicit lock; old boolean discarded |
| `lit/frcp/03-post-trial-motions.lit:92` | `relief-from-judgment-period` | `false` | — | lit/frcp/periods.lit:444 cite `FRCP 60(c)(1)` | hole: no typed grant, delegation, or explicit lock; old boolean discarded |
| `lit/frcrimp/01-investigation-to-appeal.lit:678` | `rule29-period` | `false` | — | lit/frcrimp/deadlines.lit:89 cite `FRCrimP 29(c)(1); FRCrimP 45(b)(1)` | typed grant/delegation says true; old false discarded |
| `lit/frcrimp/01-investigation-to-appeal.lit:772` | `rule35-period` | `false` | — | lit/frcrimp/deadlines.lit:119 cite `FRCrimP 35(a); FRCrimP 45(b)(2)` | hole: no typed grant, delegation, or explicit lock; old boolean discarded |
| `lit/habeas/proceeding.lit:23` | `state-petition-limitation` | `false` | — | lit/habeas/limitations.lit:84 cite `28 U.S.C. § 2244(d)(1)`; lit/habeas/limitations.lit:92 cite `28 U.S.C. § 2244(d)(1)` | hole: no typed grant, delegation, or explicit lock; old boolean discarded |
| `lit/habeas/proceeding.lit:30` | `federal-motion-limitation` | `false` | — | lit/habeas/limitations.lit:111 cite `28 U.S.C. § 2255(f)`; lit/habeas/limitations.lit:119 cite `28 U.S.C. § 2255(f)` | hole: no typed grant, delegation, or explicit lock; old boolean discarded |
| `lit/habeas/proceeding.lit:48` | `successive-authorization-ruling` | `false` | — | lit/habeas/limitations.lit:132 cite `28 U.S.C. § 2244(b)(3)(D)` | hole: no typed grant, delegation, or explicit lock; old boolean discarded |
| `lit/links/links.lit:74` | `rehearing-period-united-states-party` | `false` | — | lit/links/links.lit:27 cite `Fed. Cir. R. 40(f)(1); FRAP 40(d)(1)(A); FRAP 40(d)(1)(B)` | hole: no typed grant, delegation, or explicit lock; old boolean discarded |
| `lit/links/links.lit:106` | `rehearing-period-united-states-party` | `false` | — | lit/links/links.lit:27 cite `Fed. Cir. R. 40(f)(1); FRAP 40(d)(1)(A); FRAP 40(d)(1)(B)` | hole: no typed grant, delegation, or explicit lock; old boolean discarded |
| `lit/links/links.lit:258` | `rehearing-period-united-states-party` | `false` | — | lit/links/links.lit:27 cite `Fed. Cir. R. 40(f)(1); FRAP 40(d)(1)(A); FRAP 40(d)(1)(B)` | hole: no typed grant, delegation, or explicit lock; old boolean discarded |
| `lit/mpep/01-pre-filing.lit:18` | `provisional-pendency` | `false` | — | lit/mpep/deadlines.lit:90 cite `35 U.S.C. 119(e)(1)` | hole: no typed grant, delegation, or explicit lock; old boolean discarded |
| `lit/mpep/01-pre-filing.lit:41` | `provisional-pendency` | `false` | — | lit/mpep/deadlines.lit:90 cite `35 U.S.C. 119(e)(1)` | hole: no typed grant, delegation, or explicit lock; old boolean discarded |
| `lit/mpep/01-pre-filing.lit:64` | `provisional-pendency` | `false` | — | lit/mpep/deadlines.lit:90 cite `35 U.S.C. 119(e)(1)` | hole: no typed grant, delegation, or explicit lock; old boolean discarded |
| `lit/mpep/01-pre-filing.lit:81` | `provisional-pendency` | `false` | — | lit/mpep/deadlines.lit:90 cite `35 U.S.C. 119(e)(1)` | hole: no typed grant, delegation, or explicit lock; old boolean discarded |
| `lit/mpep/02-filing-to-allowance.lit:127` | `extension-month` | `true` | 37 CFR 1.136(a) | lit/mpep/deadlines.lit:204 cite `37 CFR 1.136(a); MPEP 710.01(a)` | hole: no typed grant, delegation, or explicit lock; old boolean discarded |
| `lit/mpep/02-filing-to-allowance.lit:165` | `extension-month` | `true` | 37 CFR 1.136(a) | lit/mpep/deadlines.lit:204 cite `37 CFR 1.136(a); MPEP 710.01(a)` | hole: no typed grant, delegation, or explicit lock; old boolean discarded |
| `lit/mpep/02-filing-to-allowance.lit:191` | `extension-month` | `true` | 37 CFR 1.136(a) | lit/mpep/deadlines.lit:204 cite `37 CFR 1.136(a); MPEP 710.01(a)` | hole: no typed grant, delegation, or explicit lock; old boolean discarded |
| `lit/mpep/02-filing-to-allowance.lit:202` | `extension-month` | `false` | — | lit/mpep/deadlines.lit:204 cite `37 CFR 1.136(a); MPEP 710.01(a)` | hole: no typed grant, delegation, or explicit lock; old boolean discarded |
| `lit/mpep/02-filing-to-allowance.lit:218` | `extension-month` | `true` | 37 CFR 1.136(a) | lit/mpep/deadlines.lit:204 cite `37 CFR 1.136(a); MPEP 710.01(a)` | hole: no typed grant, delegation, or explicit lock; old boolean discarded |
| `lit/mpep/02-filing-to-allowance.lit:229` | `extension-month` | `false` | — | lit/mpep/deadlines.lit:204 cite `37 CFR 1.136(a); MPEP 710.01(a)` | hole: no typed grant, delegation, or explicit lock; old boolean discarded |
| `lit/mpep/02-filing-to-allowance.lit:469` | `appeal-brief` | `true` | 37 CFR 1.136(a) | lit/mpep/deadlines.lit:282 cite `37 CFR 41.37(a)` | hole: no typed grant, delegation, or explicit lock; old boolean discarded |
| `lit/mpep/02-filing-to-allowance.lit:480` | `appeal-brief` | `true` | 37 CFR 1.136(a) | lit/mpep/deadlines.lit:282 cite `37 CFR 41.37(a)` | hole: no typed grant, delegation, or explicit lock; old boolean discarded |
| `lit/mpep/02-filing-to-allowance.lit:507` | `appeal-forwarding-fee` | `false` | — | lit/mpep/deadlines.lit:300 cite `37 CFR 41.45(a)` | hole: no typed grant, delegation, or explicit lock; old boolean discarded |
| `lit/mpep/02-filing-to-allowance.lit:531` | `appeal-forwarding-fee` | `false` | — | lit/mpep/deadlines.lit:300 cite `37 CFR 41.45(a)` | hole: no typed grant, delegation, or explicit lock; old boolean discarded |
| `lit/mpep/02-filing-to-allowance.lit:575` | `court-review` | `false` | — | lit/mpep/deadlines.lit:317 cite `37 CFR 90.3(a)` | hole: no typed grant, delegation, or explicit lock; old boolean discarded |
| `lit/mpep/02-filing-to-allowance.lit:584` | `court-review` | `false` | — | lit/mpep/deadlines.lit:317 cite `37 CFR 90.3(a)` | hole: no typed grant, delegation, or explicit lock; old boolean discarded |
| `lit/mpep/02-filing-to-allowance.lit:730` | `issue-fee` | `false` | — | lit/mpep/deadlines.lit:334 cite `37 CFR 1.311(a)` | hole: no typed grant, delegation, or explicit lock; old boolean discarded |
| `lit/mpep/02-filing-to-allowance.lit:753` | `issue-fee` | `false` | — | lit/mpep/deadlines.lit:334 cite `37 CFR 1.311(a)` | hole: no typed grant, delegation, or explicit lock; old boolean discarded |
| `lit/mpep/02-filing-to-allowance.lit:801` | `broadening-reissue` | `false` | — | lit/mpep/deadlines.lit:453 cite `35 U.S.C. 251(d)` | hole: no typed grant, delegation, or explicit lock; old boolean discarded |
| `lit/mpep/02-filing-to-allowance.lit:841` | `broadening-reissue` | `false` | — | lit/mpep/deadlines.lit:453 cite `35 U.S.C. 251(d)` | hole: no typed grant, delegation, or explicit lock; old boolean discarded |
| `lit/mpep/03-post-grant.lit:21` | `ipr-time-bar` | `false` | — | lit/mpep/deadlines.lit:351 cite `35 U.S.C. 315(b)` | hole: no typed grant, delegation, or explicit lock; old boolean discarded |
| `lit/mpep/03-post-grant.lit:31` | `pgr-window` | `false` | — | lit/mpep/deadlines.lit:367 cite `35 U.S.C. 321(c)` | hole: no typed grant, delegation, or explicit lock; old boolean discarded |
| `lit/mpep/03-post-grant.lit:52` | `ipr-time-bar` | `false` | — | lit/mpep/deadlines.lit:351 cite `35 U.S.C. 315(b)` | hole: no typed grant, delegation, or explicit lock; old boolean discarded |
| `lit/mpep/03-post-grant.lit:62` | `pgr-window` | `false` | — | lit/mpep/deadlines.lit:367 cite `35 U.S.C. 321(c)` | hole: no typed grant, delegation, or explicit lock; old boolean discarded |
| `lit/mpep/03-post-grant.lit:83` | `ipr-preliminary-response` | `false` | — | lit/mpep/deadlines.lit:388 cite `37 CFR 42.107(b)` | typed grant/delegation says true; old false discarded |
| `lit/mpep/03-post-grant.lit:151` | `pgr-preliminary-response` | `false` | — | lit/mpep/deadlines.lit:410 cite `37 CFR 42.207(b)` | typed grant/delegation says true; old false discarded |
| `lit/ptab/01-pre-petition-to-institution.lit:53` | `pgr-filing-period` | `false` | — | lit/ptab/deadlines.lit:52 cite `35 U.S.C. § 321(c)` | hole: no typed grant, delegation, or explicit lock; old boolean discarded |
| `lit/ptab/01-pre-petition-to-institution.lit:77` | `ipr-time-bar` | `false` | — | lit/ptab/deadlines.lit:79 cite `35 U.S.C. § 315(b)` | hole: no typed grant, delegation, or explicit lock; old boolean discarded |
| `lit/ptab/01-pre-petition-to-institution.lit:110` | `pgr-filing-period` | `false` | — | lit/ptab/deadlines.lit:52 cite `35 U.S.C. § 321(c)` | hole: no typed grant, delegation, or explicit lock; old boolean discarded |
| `lit/ptab/01-pre-petition-to-institution.lit:178` | `ipr-time-bar` | `false` | — | lit/ptab/deadlines.lit:79 cite `35 U.S.C. § 315(b)` | hole: no typed grant, delegation, or explicit lock; old boolean discarded |
| `lit/ptab/01-pre-petition-to-institution.lit:196` | `petition-correction-period` | `false` | — | lit/ptab/deadlines.lit:115 cite `37 C.F.R. § 42.106(b); 37 C.F.R. § 42.5(c)(1)` | typed grant/delegation says true; old false discarded |
| `lit/ptab/01-pre-petition-to-institution.lit:249` | `institution-period` | `false` | — | lit/ptab/deadlines.lit:172 cite `35 U.S.C. § 314(b)` | hole: no typed grant, delegation, or explicit lock; old boolean discarded |
| `lit/ptab/01-pre-petition-to-institution.lit:266` | `institution-period-on-waiver` | `false` | — | lit/ptab/deadlines.lit:191 cite `35 U.S.C. § 314(b); 37 C.F.R. (2012) § 42.107(b)`; lit/ptab/deadlines.lit:194 cite `35 U.S.C. § 314(b); 37 C.F.R. § 42.107(b)` | hole: no typed grant, delegation, or explicit lock; old boolean discarded |
| `lit/ptab/01-pre-petition-to-institution.lit:317` | `ipr-time-bar` | `false` | — | lit/ptab/deadlines.lit:79 cite `35 U.S.C. § 315(b)` | hole: no typed grant, delegation, or explicit lock; old boolean discarded |
| `lit/ptab/01-pre-petition-to-institution.lit:336` | `ipr-time-bar` | `false` | — | lit/ptab/deadlines.lit:79 cite `35 U.S.C. § 315(b)` | hole: no typed grant, delegation, or explicit lock; old boolean discarded |
| `lit/ptab/03-trial-to-appeal.lit:342` | `rehearing-period` | `false` | — | lit/ptab/deadlines.lit:308 cite `37 C.F.R. (2012) § 42.71(d)(2)`; lit/ptab/deadlines.lit:312 cite `37 C.F.R. (2012) § 42.71(d)(2); 80 FR 28565`; lit/ptab/deadlines.lit:316 cite `37 C.F.R. § 42.71(d)(2); 37 C.F.R. § 42.5(c)(1)` | typed grant/delegation says true; old false discarded |
| `lit/ptab/03-trial-to-appeal.lit:351` | `director-review-period` | `false` | — | lit/ptab/deadlines.lit:351 cite `37 C.F.R. § 42.75(c)(1); 37 C.F.R. § 42.71(d)(2)` | typed grant/delegation says true; old false discarded |
| `lit/ptab/03-trial-to-appeal.lit:404` | `director-review-after-rehearing-period` | `false` | — | lit/ptab/deadlines.lit:355 cite `37 C.F.R. § 42.75(a); 37 C.F.R. § 42.75(c)(1); 37 C.F.R. § 42.71(d)(2)` | typed grant/delegation says true; old false discarded |
| `lit/ptab/03-trial-to-appeal.lit:486` | `appeal-period` | `false` | — | lit/ptab/deadlines.lit:424 cite `77 FR 48677; 77 FR 48612`; lit/ptab/deadlines.lit:430 cite `37 C.F.R. § 90.3(a)(1); 37 C.F.R. § 90.3(b)(1)` | typed grant/delegation says true; old false discarded |
| `lit/ptab/03-trial-to-appeal.lit:495` | `appeal-period` | `false` | — | lit/ptab/deadlines.lit:424 cite `77 FR 48677; 77 FR 48612`; lit/ptab/deadlines.lit:430 cite `37 C.F.R. § 90.3(a)(1); 37 C.F.R. § 90.3(b)(1)` | typed grant/delegation says true; old false discarded |
| `lit/ttab/01-pre.lit:30` | `opposition-period` | `true` | 37 CFR 2.102(c)(2)-(3) | lit/ttab/ttab.lit:140 cite `37 CFR 2.101(c)` | hole: no typed grant, delegation, or explicit lock; old boolean discarded |
| `lit/ttab/01-pre.lit:36` | `first-extension-90` | `true` | 37 CFR 2.102(c)(3) | lit/ttab/ttab.lit:168 cite `37 CFR 2.102(c)(1)(ii)` | hole: no typed grant, delegation, or explicit lock; old boolean discarded |
| `lit/ttab/01-pre.lit:42` | `opposition-period` | `true` | 37 CFR 2.102(c) | lit/ttab/ttab.lit:140 cite `37 CFR 2.101(c)` | hole: no typed grant, delegation, or explicit lock; old boolean discarded |
| `lit/ttab/01-pre.lit:54` | `second-extension-60` | `true` | 37 CFR 2.102(c)(3) | lit/ttab/ttab.lit:169 cite `37 CFR 2.102(c)(2)` | hole: no typed grant, delegation, or explicit lock; old boolean discarded |
| `lit/ttab/03-pre-2.lit:15` | `extended-period-120` | `true` | 37 CFR 2.102(c)(3) (applicant consent only) | lit/ttab/ttab.lit:173 cite `37 CFR 2.102(c)(3)` | hole: no typed grant, delegation, or explicit lock; old boolean discarded |
| `lit/ttab/04-standstill-2.lit:15` | `outer-limit-180` | `false` | — | lit/ttab/ttab.lit:174 cite `37 CFR 2.102(c)` | hole: no typed grant, delegation, or explicit lock; old boolean discarded |
| `lit/ttab/08-decision.lit:51` | `civil-action-period` | `false` | — | lit/ttab/ttab.lit:482 cite `37 CFR 2.145(d)(3)` | typed grant/delegation says true; old false discarded |

## `litigate diff` verification against the pre-change build

I ran `litigate diff lit/<package> --against <baseline-pack.json>` for every affected pack and the links package. The command reports exit 2 because differences exist; all 229 differences are the two expected derived deadline fields. No node, edge, extension-bound, link, or other deadline field changed. The table lists each changed JSON pointer and baseline value → derived value (the diff command was invoked as current source vs baseline JSON, so its `after` value is the baseline).

| Package | JSON pointer | Baseline value | Derived value |
|---|---|---|---|
| CAFc | `/edges/district-court-file-noa/deadline/extensionAuthority` | `"FRAP 4(a)(5) (up to 30 additional days, excusable neglect/good cause)"` | `"FRAP (2009 ed.) 4(a)(1)(A); FRAP (2009 ed.) 4(a)(5)(A)"` |
| CAFc | `/edges/ptab-file-noa/deadline/extensionAuthority` | `"37 C.F.R. \u00a7 90.3(c) (Director may extend on a timely, good-cause request filed before expiration, or post-expiration on a showing of excusable neglect)"` | `"37 C.F.R. \u00a7 90.3(a)(1); 37 C.F.R. \u00a7 90.3(b)(2)"` |
| CAFc | `/edges/itc-file-noa/deadline/extensionAuthority` | `null` | `"FRAP 26(b)(2)"` |
| CAFc | `/edges/cofc-file-noa/deadline/extensionAuthority` | `"FRAP 4(a)(5) (up to 30 additional days, excusable neglect/good cause)"` | `"28 U.S.C. \u00a7 2522; FRAP (2009 ed.) 4(a)(1)(B); FRAP (2009 ed.) 4(a)(5)(A)"` |
| CAFc | `/edges/ttab-file-noa/deadline/extensionAuthority` | `"37 C.F.R. \u00a7 2.145(e) (Director may extend on good cause if requested before expiry, or on excusable neglect if requested after)"` | `"37 C.F.R. \u00a7 2.145(d)(1); 37 C.F.R. \u00a7 2.145(d)(4)(ii)"` |
| CAFc | `/edges/docketed-to-appearance/deadline/extendable` | `false` | `null` |
| CAFc | `/edges/appearance-to-docketing-statement/deadline/extensionAuthority` | `"Fed. Cir. R. 26(b)"` | `"Fed. Cir. R. 47.6; Fed. Cir. R. 26(b)(1)"` |
| CAFc | `/edges/cross-appeal-file/deadline/extensionAuthority` | `"FRAP 4(a)(5): on motion no later than 30 days after the period expires, for excusable neglect or good cause"` | `"FRAP (2009 ed.) 4(a)(3)"` |
| CAFc | `/edges/appellant-brief-file/deadline/extensionAuthority` | `"Fed. Cir. R. 26(b)"` | `"Fed. Cir. R. 31(a)(1); Fed. Cir. R. 26(b)(1)"` |
| CAFc | `/edges/appellee-brief-file/deadline/extensionAuthority` | `"Fed. Cir. R. 26(b)"` | `"Fed. Cir. R. 31(a)(2); Fed. Cir. R. 26(b)(1)"` |
| CAFc | `/edges/reply-brief-file/deadline/extensionAuthority` | `"Fed. Cir. R. 26(b)"` | `"FRAP 31(a)(1); Fed. Cir. R. 26(b)(1)"` |
| CAFc | `/edges/briefing-to-appendix/deadline/extensionAuthority` | `"Fed. Cir. R. 26(b)"` | `"Fed. Cir. R. 30(a)(2); Fed. Cir. R. 26(b)(1)"` |
| CAFc | `/edges/rehearing-file/deadline/extendable` | `false` | `true` |
| CAFc | `/edges/rehearing-file/deadline/extensionAuthority` | `null` | `"Fed. Cir. R. 40(f)(1); FRAP 40(d)(1)"` |
| CAFc | `/edges/rehearing-time-expires/deadline/extendable` | `null` | `true` |
| CAFc | `/edges/rehearing-time-expires/deadline/extensionAuthority` | `null` | `"Fed. Cir. R. 40(f)(1); FRAP 40(d)(1)"` |
| CAFc | `/edges/rehearing-denied-to-mandate/deadline/extendable` | `null` | `true` |
| CAFc | `/edges/rehearing-denied-to-mandate/deadline/extensionAuthority` | `null` | `"FRAP 41(b)"` |
| CAFc | `/edges/mandate-to-costs/deadline/extensionAuthority` | `"Fed. Cir. R. 26(b)"` | `"FRAP 39(e)(3)(A); Fed. Cir. R. 39; Fed. Cir. R. 26(b)(1)"` |
| CAFc | `/edges/cert-window-file/deadline/extensionAuthority` | `"Sup. Ct. R. 13.5 (single application to a Circuit Justice, before the deadline, for good cause)"` | `"28 U.S.C. \u00a7 2101(c)"` |
| CoFC | `/edges/e-protest-notice-to-complaint/deadline/extendable` | `false` | `null` |
| CoFC | `/edges/e-responsefork-answer/deadline/extensionAuthority` | `"RCFC 6(b) stipulation or court order"` | `"RCFC 12(a)(1)(A)"` |
| CoFC | `/edges/e-responsefork-motion/deadline/extensionAuthority` | `"RCFC 6(b) stipulation or court order"` | `"RCFC 12(a)(1)(A)"` |
| CoFC | `/edges/e-ruling12b-denied/deadline/extendable` | `false` | `true` |
| CoFC | `/edges/e-ruling12b-denied/deadline/extensionAuthority` | `null` | `"RCFC 12(a)(4)(A)"` |
| CoFC | `/edges/e-jpsr-to-scheduling/deadline/extensionAuthority` | `"court order"` | `"RCFC App. A 4"` |
| CoFC | `/edges/e-discclosed-to-sj/deadline/extensionAuthority` | `"court order / local practice"` | `"RCFC 56(b)"` |
| CoFC | `/edges/e-14bfork-none/deadline/extensionAuthority` | `"court order (complexity/good cause, PRCFC Rule 1(b))"` | `"PRCFC 3"` |
| CoFC | `/edges/e-14bserved-to-jpsr/deadline/extensionAuthority` | `"court order (complexity/good cause, PRCFC Rule 1(b))"` | `"PRCFC 3"` |
| CoFC | `/edges/e-po-to-infringement/deadline/extensionAuthority` | `"PRCFC Rule 1(b) / Rule 24 good cause"` | `"PRCFC 4"` |
| CoFC | `/edges/e-infringement-to-invalidity/deadline/extensionAuthority` | `"PRCFC Rule 1(b) / Rule 24 good cause"` | `"PRCFC 6"` |
| CoFC | `/edges/e-invalidity-to-ccexchange/deadline/extensionAuthority` | `"PRCFC Rule 1(b) good cause"` | `"PRCFC 9"` |
| CoFC | `/edges/e-ccexchange-to-jcc/deadline/extensionAuthority` | `"PRCFC Rule 1(b) good cause"` | `"PRCFC 11"` |
| CoFC | `/edges/e-jcc-to-ccdiscoveryclosed/deadline/extensionAuthority` | `"PRCFC Rule 1(b) good cause"` | `"PRCFC 13"` |
| CoFC | `/edges/e-ccdiscoveryclosed-to-hearing/deadline/extensionAuthority` | `"court order"` | `"PRCFC 14"` |
| CoFC | `/edges/e-order-to-settlement/deadline/extendable` | `false` | `null` |
| CoFC | `/edges/e-settlementmeet-to-fork/deadline/extendable` | `false` | `null` |
| CoFC | `/edges/e-posttrialwindow-motion/deadline/extendable` | `false` | `null` |
| CoFC | `/edges/e-posttrialwindow-motion/deadline/extensionAuthority` | `"none -- RCFC 6(b) forbids extension"` | `null` |
| CoFC | `/edges/e-costs-to-prevailing/deadline/extendable` | `false` | `null` |
| FRAP | `/edges/edge-final-judgment-entered-appeal-clock-open/deadline/extensionAuthority` | `"Fed. R. App. P. 4(a)(5)"` | `"28 U.S.C. \u00a7 2107(a)"` |
| FRAP | `/edges/interlocutory-injunction-order/deadline/extensionAuthority` | `"Fed. R. App. P. 4(a)(5)"` | `"28 U.S.C. \u00a7 2107(a)"` |
| FRAP | `/edges/interlocutory-collateral-order/deadline/extensionAuthority` | `"Fed. R. App. P. 4(a)(5)"` | `"28 U.S.C. \u00a7 2107(a)"` |
| FRAP | `/edges/interlocutory-1292b-certified/deadline/extensionAuthority` | `"Fed. R. App. P. 5(a)(3)"` | `"FRAP 26(b)(1)"` |
| FRAP | `/edges/notice-of-appeal-civil/deadline/extendable` | `false` | `true` |
| FRAP | `/edges/notice-of-appeal-civil/deadline/extensionAuthority` | `"FRAP 4(a)(5) (may extend up to 30 additional days only via motion before expiration)"` | `"28 U.S.C. \u00a7 2107(a)"` |
| FRAP | `/edges/notice-of-appeal-us-party/deadline/extendable` | `false` | `true` |
| FRAP | `/edges/notice-of-appeal-us-party/deadline/extensionAuthority` | `"FRAP 4(a)(5) (may extend up to 30 additional days only via motion before expiration)"` | `"28 U.S.C. \u00a7 2107(b)"` |
| FRAP | `/edges/notice-of-appeal-criminal/deadline/extendable` | `false` | `true` |
| FRAP | `/edges/notice-of-appeal-criminal/deadline/extensionAuthority` | `"FRAP 4(b)(4) (may be extended for excusable neglect or good cause, subject to Bowles jurisdictional limit)"` | `"FRAP (2009 ed.) 4(b)(1)(A)"` |
| FRAP | `/edges/edge-appeal-clock-open-tolling-motion-filed/deadline/extendable` | `false` | `null` |
| FRAP | `/edges/edge-appeal-clock-open-tolling-motion-filed/deadline/extensionAuthority` | `"FRCP 6(b) may extend the tolling-motion deadline by up to 28 days total (only for FRCP 50(b), 59(e), 60(b) motions)"` | `null` |
| FRAP | `/edges/edge-appeal-clock-open-post-deadline-relief-motion/deadline/extensionAuthority` | `"FRAP 4(a)(5)(A) (court may extend if motion filed before deadline expires); FRAP 4(a)(6) (may reopen for 14 days if lack of notice of entry)"` | `"28 U.S.C. \u00a7 2107(a)"` |
| FRAP | `/edges/edge-tolling-motion-disposed-appeal-clock-open/deadline/extendable` | `false` | `true` |
| FRAP | `/edges/edge-tolling-motion-disposed-appeal-clock-open/deadline/extensionAuthority` | `"FRAP 4(a)(5) (extension may apply to the restarted deadline)"` | `"28 U.S.C. \u00a7 2107(a)"` |
| FRAP | `/edges/edge-cross-appeal-fork-cross-appeal-filed/deadline/extensionAuthority` | `"FRAP 4(a)(5): on motion no later than 30 days after the period expires, for excusable neglect or good cause"` | `"FRAP (2009 ed.) 4(a)(3)"` |
| FRAP | `/edges/edge-transcript-order-fork-transcript-ordered/deadline/extendable` | `false` | `true` |
| FRAP | `/edges/edge-transcript-order-fork-transcript-ordered/deadline/extensionAuthority` | `null` | `"FRAP 10(b)(1)"` |
| FRAP | `/edges/no-transcript-certificate/deadline/extendable` | `false` | `true` |
| FRAP | `/edges/no-transcript-certificate/deadline/extensionAuthority` | `null` | `"FRAP 10(b)(1)"` |
| FRAP | `/edges/edge-transcript-ordered-record-transmitted/deadline/extensionAuthority` | `"FRAP 11(b) (reporter may request extension for completion beyond 30 days; no formal party motion needed)"` | `"FRAP (2009 ed.) 11(b)(1)(B)"` |
| FRAP | `/edges/edge-stay-pending-appeal-fork-stay-granted/deadline/extensionAuthority` | `"FRAP 26(b); court order"` | `"FRAP (2009 ed.) 27(a)(3)(A)"` |
| FRAP | `/edges/edge-stay-pending-appeal-fork-stay-denied/deadline/extensionAuthority` | `"FRAP 26(b); court order"` | `"FRAP (2009 ed.) 27(a)(3)(A)"` |
| FRAP | `/edges/edge-appellant-brief-window-appellant-brief-filed/deadline/extensionAuthority` | `"FRAP 26(b); court discretion for good cause"` | `"FRAP 31(a)(1)"` |
| FRAP | `/edges/edge-appellant-brief-filed-appellee-brief-filed/deadline/extensionAuthority` | `"FRAP 26(b); court discretion for good cause"` | `"FRAP 31(a)(1)"` |
| FRAP | `/edges/edge-appellee-brief-filed-reply-brief-filed/deadline/extensionAuthority` | `"FRAP 26(b); court discretion for good cause; FRAP 31(a)(1) requires filing at least 7 days before argument"` | `"FRAP 31(a)(1)"` |
| FRAP | `/edges/edge-affirmed-rehearing-window/deadline/extensionAuthority` | `"FRAP 40(a)(1); circuit local rules may allow additional extensions by panel order or stipulation"` | `"FRAP 40(d)(1)"` |
| FRAP | `/edges/edge-reversed-rehearing-window/deadline/extensionAuthority` | `"FRAP 40(a)(1); circuit local rules"` | `"FRAP 40(d)(1)"` |
| FRAP | `/edges/edge-reversed-and-remanded-rehearing-window/deadline/extensionAuthority` | `"FRAP 40(a)(1); circuit local rules"` | `"FRAP 40(d)(1)"` |
| FRAP | `/edges/edge-vacated-and-remanded-rehearing-window/deadline/extensionAuthority` | `"FRAP 40(a)(1); circuit local rules"` | `"FRAP 40(d)(1)"` |
| FRAP | `/edges/edge-affirmed-in-part-reversed-in-part-rehearing-window/deadline/extensionAuthority` | `"FRAP 40(a)(1); circuit local rules"` | `"FRAP 40(d)(1)"` |
| FRAP | `/edges/edge-rehearing-window-rehearing-petition-filed/deadline/extendable` | `false` | `true` |
| FRAP | `/edges/edge-rehearing-window-rehearing-petition-filed/deadline/extensionAuthority` | `"FRAP 26(b) is silent on FRAP 40 deadlines; extension is theoretically possible for good cause but extremely rarely granted in practice"` | `"FRAP 40(d)(1)"` |
| FRAP | `/edges/edge-rehearing-window-mandate-issued/deadline/extendable` | `null` | `true` |
| FRAP | `/edges/edge-rehearing-window-mandate-issued/deadline/extensionAuthority` | `null` | `"FRAP 40(d)(1)"` |
| FRAP | `/edges/edge-rehearing-granted-amended-opinion-mandate-issued/deadline/extendable` | `null` | `true` |
| FRAP | `/edges/edge-rehearing-granted-amended-opinion-mandate-issued/deadline/extensionAuthority` | `null` | `"FRAP (2009 ed.) 41(b)"` |
| FRAP | `/edges/edge-rehearing-en-banc-both-denied-mandate-issued/deadline/extendable` | `null` | `true` |
| FRAP | `/edges/edge-rehearing-en-banc-both-denied-mandate-issued/deadline/extensionAuthority` | `null` | `"FRAP (2009 ed.) 41(b)"` |
| FRAP | `/edges/edge-mandate-issued-cert-petition-window/deadline/extensionAuthority` | `"Sup. Ct. R. 13.6"` | `"28 U.S.C. \u00a7 2101(c)"` |
| FRAP | `/edges/edge-mandate-stayed-pending-cert-cert-petition-window/deadline/extensionAuthority` | `"Sup. Ct. R. 13.6"` | `"28 U.S.C. \u00a7 2101(c)"` |
| FRAP | `/edges/edge-cert-petition-window-cert-petition-filed/deadline/extensionAuthority` | `"28 U.S.C. \u00a7 2101(c); Sup. Ct. R. 13.5, 30.3 (application to an individual Justice, up to 60 days; once denied, not renewable)"` | `"28 U.S.C. \u00a7 2101(c)"` |
| FRCP | `/edges/edge-service-window-service-completed/deadline/extensionAuthority` | `"FRCP 4(m) good-cause showing"` | `"FRCP 4(m)"` |
| FRCP | `/edges/edge-waiver-requested-waiver-response-window/deadline/extensionAuthority` | `"FRCP 6(b)(1): for good cause before the period expires, or on motion after it for excusable neglect"` | `"FRCP 12(a)(1)(A)(ii)"` |
| FRCP | `/edges/edge-service-completed-rule12-response-window/deadline/extensionAuthority` | `"FRCP 6(b) stipulation or court order"` | `"FRCP 12(a)(1)(A)(i)"` |
| FRCP | `/edges/edge-removal-window-removed-to-federal/deadline/extendable` | `false` | `null` |
| FRCP | `/edges/edge-removal-window-rule12-response-window/deadline/extensionAuthority` | `"FRCP 6(b) stipulation or court order"` | `"FRCP 12(a)(1)(A)(i)"` |
| FRCP | `/edges/edge-removed-to-federal-remand-ruling/deadline/extendable` | `false` | `null` |
| FRCP | `/edges/edge-removed-to-federal-removed-response-window/deadline/extensionAuthority` | `"FRCP 6(b) stipulation or court order"` | `"FRCP 81(c)(2)(A)"` |
| FRCP | `/edges/edge-remand-ruling-removed-response-window/deadline/extensionAuthority` | `"FRCP 6(b) stipulation or court order"` | `"FRCP 81(c)(2)(A)"` |
| FRCP | `/edges/edge-rule12-response-window-answer-filed/deadline/extensionAuthority` | `"FRCP 6(b) stipulation or court order"` | `"FRCP 12(a)(1)(A)(i)"` |
| FRCP | `/edges/edge-rule12-response-window-special-appearance/deadline/extensionAuthority` | `"FRCP 6(b) stipulation or court order"` | `"FRCP 12(a)(1)(A)(i)"` |
| FRCP | `/edges/12b-motion-filed/deadline/extensionAuthority` | `"FRCP 6(b) stipulation or court order"` | `"FRCP 12(a)(1)(A)(i)"` |
| FRCP | `/edges/edge-waiver-response-window-answer-filed/deadline/extensionAuthority` | `"FRCP 6(b)(1): for good cause before the period expires, or on motion after it for excusable neglect"` | `"FRCP 12(a)(1)(A)(ii)"` |
| FRCP | `/edges/waiver-12b-motion-filed/deadline/extensionAuthority` | `"FRCP 6(b)(1): for good cause before the period expires, or on motion after it for excusable neglect"` | `"FRCP 12(a)(1)(A)(ii)"` |
| FRCP | `/edges/edge-removed-response-window-answer-filed/deadline/extensionAuthority` | `"FRCP 6(b) stipulation or court order"` | `"FRCP 81(c)(2)(A)"` |
| FRCP | `/edges/removed-12b-motion-filed/deadline/extensionAuthority` | `"FRCP 6(b) stipulation or court order"` | `"FRCP 81(c)(2)(A)"` |
| FRCP | `/edges/edge-amended-response-window-answer-filed/deadline/extensionAuthority` | `"FRCP 6(b) stipulation or court order"` | `"FRCP 15(a)(3)"` |
| FRCP | `/edges/amended-12b-motion-filed/deadline/extensionAuthority` | `"FRCP 6(b) stipulation or court order"` | `"FRCP 15(a)(3)"` |
| FRCP | `/edges/edge-mtd-ruling-answer-filed/deadline/extensionAuthority` | `"FRCP 6(b) stipulation or court order"` | `"FRCP 12(a)(4)(A)"` |
| FRCP | `/edges/edge-amendment-as-of-right-window-amended-complaint-filed/deadline/extensionAuthority` | `"FRCP 6(b)(1): for good cause before the period expires, or on motion after it for excusable neglect"` | `"FRCP 15(a)(1)(B)"` |
| FRCP | `/edges/edge-amended-complaint-filed-amended-response-window/deadline/extensionAuthority` | `"FRCP 6(b) stipulation or court order"` | `"FRCP 15(a)(3)"` |
| FRCP | `/edges/edge-rule26f-conference-initial-disclosures/deadline/extensionAuthority` | `"court order"` | `"FRCP 26(a)(1)(C)"` |
| FRCP | `/edges/jury-demand-timely/deadline/extensionAuthority` | `"FRCP 6(b) good cause"` | `"FRCP 38(b)(1)"` |
| FRCP | `/edges/edge-final-judgment-entered-fee-motion-filed/deadline/extensionAuthority` | `"FRCP 6(b)(1): on a stipulated request or motion, or by the court"` | `"FRCP 54(d)(2)(B)"` |
| FRCP | `/edges/edge-post-trial-motion-window-rule50b-jmol-filed/deadline/extensionAuthority` | `null` | `"FRCP 6(b)(2)"` |
| FRCP | `/edges/edge-post-trial-motion-window-rule59-new-trial-filed/deadline/extensionAuthority` | `null` | `"FRCP 6(b)(2)"` |
| FRCP | `/edges/edge-post-trial-motion-window-rule59e-amend-filed/deadline/extensionAuthority` | `null` | `"FRCP 6(b)(2)"` |
| FRCP | `/edges/edge-post-trial-motion-window-rule60-relief-filed/deadline/extensionAuthority` | `null` | `"FRCP 6(b)(2)"` |
| FRCP | `/edges/edge-post-trial-motion-window-fee-motion-filed/deadline/extensionAuthority` | `"FRCP 6(b)(1): on a stipulated request or motion, or by the court"` | `"FRCP 54(d)(2)(B)"` |
| FRCP | `/edges/edge-notice-of-appeal-window-notice-of-appeal-filed/deadline/extensionAuthority` | `"FRAP 4(a)(5) excusable neglect or good cause, sought before or shortly after the deadline"` | `"28 U.S.C. \u00a7 2107(a)"` |
| FRCP | `/edges/edge-cross-appeal-window-cross-appeal-filed/deadline/extensionAuthority` | `"FRAP 4(a)(5): on motion no later than 30 days after the period expires, for excusable neglect or good cause"` | `"FRAP (2009 ed.) 4(a)(3)"` |
| FRCrimP | `/edges/edge-indictment-returned-arraignment/deadline/extensionAuthority` | `"18 U.S.C. \u00a7 3161(h)"` | `"18 U.S.C. \u00a7 3161(c)(1); 18 U.S.C. \u00a7 3161(h)"` |
| FRCrimP | `/edges/edge-initial-appearance-preliminary-hearing/deadline/extensionAuthority` | `"Fed. R. Crim. P. 5.1(c) \u2014 good cause, or extraordinary circumstances where justice requires the delay, even without the defendant's consent"` | `"FRCrimP 5.1(c); FRCrimP 5.1(d)"` |
| FRCrimP | `/edges/edge-bound-over-grand-jury-grand-jury-convened/deadline/extensionAuthority` | `"18 U.S.C. \u00a7 3161(b) \u2014 one additional 30-day extension if no grand jury convened during the initial 30-day period in the district"` | `"18 U.S.C. \u00a7 3161(b); 18 U.S.C. \u00a7 3161(h)"` |
| FRCrimP | `/edges/edge-proceed-to-trial-jury-selection/deadline/extensionAuthority` | `"18 U.S.C. \u00a7 3161(h) (excludable delay)"` | `"18 U.S.C. \u00a7 3161(c)(1); 18 U.S.C. \u00a7 3161(h)"` |
| FRCrimP | `/edges/edge-guilty-verdict-rule29-motion/deadline/extendable` | `false` | `true` |
| FRCrimP | `/edges/edge-guilty-verdict-rule29-motion/deadline/extensionAuthority` | `null` | `"FRCrimP 29(c)(1); FRCrimP 45(b)(1)"` |
| FRCrimP | `/edges/edge-guilty-verdict-rule33-motion/deadline/extensionAuthority` | `"Rule 33(b) allows extension of the 14-day deadline if the court finds grounds within the original period; the 3-year newly-discovered-evidence window requires no extension"` | `"FRCrimP 33(b)(2); FRCrimP 45(b)(1)"` |
| FRCrimP | `/edges/edge-presentence-report-sentencing-hearing/deadline/extendable` | `true` | `null` |
| FRCrimP | `/edges/edge-presentence-report-sentencing-hearing/deadline/extensionAuthority` | `"Fed. R. Crim. P. 32(e)(2) \u2014 parties may stipulate to a shorter period, or the court may set a different period for good cause"` | `null` |
| FRCrimP | `/edges/edge-judgment-entered-rule35-motion/deadline/extendable` | `false` | `null` |
| FRCrimP | `/edges/edge-judgment-entered-notice-of-appeal-window/deadline/extensionAuthority` | `"Fed. R. App. P. 4(b)(1)(B) \u2014 up to 30 additional days for good cause shown, before or after the deadline expires"` | `"FRAP 4(b)(1)(A); FRAP 4(b)(4)"` |
| FRCrimP | `/edges/edge-notice-of-appeal-window-notice-of-appeal-filed/deadline/extensionAuthority` | `"Fed. R. App. P. 4(b)(4) \u2014 on a finding of excusable neglect or good cause, the court may extend before or after the 14 days expire, with or without motion and notice, for up to 30 days from the original expiry"` | `"FRAP 4(b)(1)(A); FRAP 4(b)(4)"` |
| FRCrimP | `/edges/edge-notice-of-appeal-window-appeal-window-missed/deadline/extendable` | `true` | `null` |
| FRCrimP | `/edges/edge-notice-of-appeal-window-appeal-window-missed/deadline/extensionAuthority` | `"Fed. R. App. P. 4(b)(4)"` | `null` |
| ITC | `/edges/edge-response-period-open-response-filed/deadline/extensionAuthority` | `"ALJ good-cause order (Rule 210.6(a))"` | `"cfr210.13.a"` |
| ITC | `/edges/discovery-respond-interrogatories-rfp/deadline/extensionAuthority` | `"ALJ-specified period"` | `"cfr210.29.b.2"` |
| ITC | `/edges/discovery-rfa-deemed-admitted/deadline/extensionAuthority` | `"ALJ-specified period"` | `"cfr210.31.b"` |
| ITC | `/edges/discovery-motion-quash-subpoena/deadline/extensionAuthority` | `"ALJ-extended period"` | `"cfr210.32.d.2"` |
| ITC | `/edges/discovery-file-privilege-log/deadline/extensionAuthority` | `"good cause"` | `"cfr210.27.e.1"` |
| ITC | `/edges/edge-discovery-open-public-version-filed/deadline/extensionAuthority` | `"good cause"` | `"cfr210.5.f"` |
| ITC | `/edges/edge-prehearing-summary-determination-motion-filed/deadline/extendable` | `true` | `null` |
| ITC | `/edges/edge-prehearing-summary-determination-motion-filed/deadline/extensionAuthority` | `"ALJ good-cause exception"` | `null` |
| ITC | `/edges/edge-summary-determination-motion-filed-summary-determination-opposition/deadline/extensionAuthority` | `"ALJ discretion"` | `"cfr210.18.b"` |
| ITC | `/edges/edge-petition-for-review-window-petition-filed/deadline/extensionAuthority` | `"good cause"` | `"cfr210.43.a.1"` |
| ITC | `/edges/edge-petition-filed-petition-response/deadline/extensionAuthority` | `"good cause"` | `"cfr210.43.c"` |
| ITC | `/edges/edge-federal-circuit-appeal-window-appeal-filed/deadline/extensionAuthority` | `null` | `"FRAP 26(b)(2)"` |
| links | `/instances/cafc@cofc/patch_edges/rehearing-file/deadline/extendable` | `false` | `null` |
| links | `/instances/cafc@cofc/patch_edges/rehearing-file/deadline/extensionAuthority` | `null` | `null` |
| links | `/instances/cafc@cofc/patch_edges/rehearing-time-expires/deadline/extendable` | `null` | `null` |
| links | `/instances/cafc@cofc/patch_edges/rehearing-time-expires/deadline/extensionAuthority` | `null` | `null` |
| links | `/instances/cafc@cofc-gov/patch_edges/rehearing-file/deadline/extendable` | `false` | `null` |
| links | `/instances/cafc@cofc-gov/patch_edges/rehearing-file/deadline/extensionAuthority` | `null` | `null` |
| links | `/instances/cafc@cofc-gov/patch_edges/rehearing-time-expires/deadline/extendable` | `null` | `null` |
| links | `/instances/cafc@cofc-gov/patch_edges/rehearing-time-expires/deadline/extensionAuthority` | `null` | `null` |
| links | `/instances/cafc@itc/patch_edges/rehearing-file/deadline/extendable` | `false` | `null` |
| links | `/instances/cafc@itc/patch_edges/rehearing-file/deadline/extensionAuthority` | `null` | `null` |
| links | `/instances/cafc@itc/patch_edges/rehearing-time-expires/deadline/extendable` | `null` | `null` |
| links | `/instances/cafc@itc/patch_edges/rehearing-time-expires/deadline/extensionAuthority` | `null` | `null` |
| MPEP | `/edges/edge-pre-filing-provisional-filed/deadline/extendable` | `false` | `null` |
| MPEP | `/edges/edge-provisional-filed-nonprovisional-filed/deadline/extendable` | `false` | `null` |
| MPEP | `/edges/edge-missing-parts-pre-examination/deadline/extensionAuthority` | `"37 CFR 1.136(a)"` | `"37 CFR 1.53(f)"` |
| MPEP | `/edges/edge-applicant-response-window-amendment-filed/deadline/extensionAuthority` | `"37 CFR 1.136(a)"` | `"37 CFR 1.134"` |
| MPEP | `/edges/edge-applicant-response-window-extension-taken/deadline/extendable` | `true` | `null` |
| MPEP | `/edges/edge-applicant-response-window-extension-taken/deadline/extensionAuthority` | `"37 CFR 1.136(a)"` | `null` |
| MPEP | `/edges/edge-extension-taken-amendment-filed/deadline/extendable` | `true` | `null` |
| MPEP | `/edges/edge-extension-taken-amendment-filed/deadline/extensionAuthority` | `"37 CFR 1.136(a)"` | `null` |
| MPEP | `/edges/edge-extension-taken-extension-taken/deadline/extendable` | `false` | `null` |
| MPEP | `/edges/edge-after-final-decision-after-final-amendment/deadline/extensionAuthority` | `"37 CFR 1.136(a)"` | `"37 CFR 1.134"` |
| MPEP | `/edges/edge-after-final-decision-rce-filed/deadline/extensionAuthority` | `"37 CFR 1.136(a)"` | `"37 CFR 1.134"` |
| MPEP | `/edges/edge-after-final-decision-notice-of-appeal/deadline/extensionAuthority` | `"37 CFR 41.31(d); 37 CFR 1.136(a)"` | `"37 CFR 1.134"` |
| MPEP | `/edges/edge-after-final-decision-continuation-filed/deadline/extensionAuthority` | `"37 CFR 1.136(a)"` | `"37 CFR 1.134"` |
| MPEP | `/edges/edge-notice-of-appeal-appeal-brief-filed/deadline/extendable` | `true` | `null` |
| MPEP | `/edges/edge-notice-of-appeal-appeal-brief-filed/deadline/extensionAuthority` | `"37 CFR 1.136(a)"` | `null` |
| MPEP | `/edges/edge-notice-of-appeal-appeal-dismissed/deadline/extendable` | `true` | `null` |
| MPEP | `/edges/edge-notice-of-appeal-appeal-dismissed/deadline/extensionAuthority` | `"37 CFR 1.136(a)"` | `null` |
| MPEP | `/edges/edge-examiners-answer-ptab-decision/deadline/extendable` | `false` | `null` |
| MPEP | `/edges/edge-ptab-affirmed-federal-circuit-appeal/deadline/extendable` | `false` | `null` |
| MPEP | `/edges/edge-ptab-affirmed-sec145-action/deadline/extendable` | `false` | `null` |
| MPEP | `/edges/elect-without-traverse/deadline/extensionAuthority` | `"37 CFR 1.136(a)"` | `"37 CFR 1.142(a)"` |
| MPEP | `/edges/elect-with-traverse/deadline/extensionAuthority` | `"37 CFR 1.136(a)"` | `"37 CFR 1.142(a)"` |
| MPEP | `/edges/edge-noa-received-issue-fee-paid/deadline/extendable` | `false` | `null` |
| MPEP | `/edges/reissue-broadening-filed/deadline/extendable` | `false` | `null` |
| MPEP | `/edges/edge-patent-challenged-ipr-petition-filed/deadline/extendable` | `false` | `null` |
| MPEP | `/edges/edge-patent-challenged-pgr-petition-filed/deadline/extendable` | `false` | `null` |
| MPEP | `/edges/ipr-popr-filed/deadline/extendable` | `false` | `true` |
| MPEP | `/edges/ipr-popr-filed/deadline/extensionAuthority` | `null` | `"37 CFR 42.107(b)"` |
| MPEP | `/edges/pgr-popr-filed/deadline/extendable` | `false` | `true` |
| MPEP | `/edges/pgr-popr-filed/deadline/extensionAuthority` | `null` | `"37 CFR 42.207(b)"` |
| MPEP | `/edges/epr-owner-statement-filed/deadline/extensionAuthority` | `"37 CFR 1.550(c)"` | `"37 CFR 1.530(b)"` |
| PTAB | `/edges/edge-patent-granted-pgr-petition-filed/deadline/extendable` | `false` | `null` |
| PTAB | `/edges/edge-infringement-complaint-served-ipr-petition-filed/deadline/extendable` | `false` | `null` |
| PTAB | `/edges/edge-challenge-vehicle-decision-pgr-petition-filed/deadline/extendable` | `false` | `null` |
| PTAB | `/edges/e-time-bar-barred/deadline/extendable` | `false` | `null` |
| PTAB | `/edges/edge-petition-defect-noticed-ipr-petition-filed/deadline/extendable` | `false` | `true` |
| PTAB | `/edges/edge-petition-defect-noticed-ipr-petition-filed/deadline/extensionAuthority` | `null` | `"37 C.F.R. \u00a7 42.106(b); 37 C.F.R. \u00a7 42.5(c)(1)"` |
| PTAB | `/edges/edge-popr-window-popr-filed/deadline/extensionAuthority` | `"37 CFR 42.107(b) (one 30-day extension available without cause)"` | `"37 C.F.R. \u00a7 42.107(b); 37 C.F.R. \u00a7 42.207(b); 37 C.F.R. \u00a7 42.5(c)(1)"` |
| PTAB | `/edges/edge-popr-filed-institution-decision/deadline/extendable` | `false` | `null` |
| PTAB | `/edges/edge-popr-waived-institution-decision/deadline/extendable` | `false` | `null` |
| PTAB | `/edges/edge-institution-denied-merits-challenge-vehicle-decision/deadline/extendable` | `false` | `null` |
| PTAB | `/edges/edge-institution-denied-fintiv-challenge-vehicle-decision/deadline/extendable` | `false` | `null` |
| PTAB | `/edges/edge-mandatory-disclosures-patent-owner-response-window/deadline/extensionAuthority` | `"37 CFR 42.5(c)(2)"` | `"37 C.F.R. \u00a7 42.120(b); 37 C.F.R. \u00a7 42.5(c)(1)"` |
| PTAB | `/edges/edge-additional-discovery-motion-patent-owner-response-window/deadline/extensionAuthority` | `"37 CFR 42.5(c)(2)"` | `"37 C.F.R. \u00a7 42.120(b); 37 C.F.R. \u00a7 42.5(c)(1)"` |
| PTAB | `/edges/edge-protective-order-ipr-patent-owner-response-window/deadline/extensionAuthority` | `"37 CFR 42.5(c)(2)"` | `"37 C.F.R. \u00a7 42.120(b); 37 C.F.R. \u00a7 42.5(c)(1)"` |
| PTAB | `/edges/edge-patent-owner-response-window-patent-owner-response-filed/deadline/extensionAuthority` | `"37 CFR 42.103; scheduling order"` | `"37 C.F.R. \u00a7 42.120(b); 37 C.F.R. \u00a7 42.5(c)(1)"` |
| PTAB | `/edges/edge-patent-owner-response-filed-petitioner-reply-brief/deadline/extensionAuthority` | `"37 CFR 42.103; scheduling order"` | `"PTAB Trial Practice Guide App. A-1"` |
| PTAB | `/edges/edge-petitioner-reply-brief-patent-owner-sur-reply/deadline/extensionAuthority` | `"37 CFR 42.103; scheduling order"` | `"PTAB Trial Practice Guide App. A-1"` |
| PTAB | `/edges/edge-post-fwd-options-rehearing-request-filed/deadline/extendable` | `false` | `true` |
| PTAB | `/edges/edge-post-fwd-options-rehearing-request-filed/deadline/extensionAuthority` | `null` | `"37 C.F.R. \u00a7 42.71(d)(2); 37 C.F.R. \u00a7 42.5(c)(1)"` |
| PTAB | `/edges/edge-post-fwd-options-director-review-request-filed/deadline/extendable` | `false` | `true` |
| PTAB | `/edges/edge-post-fwd-options-director-review-request-filed/deadline/extensionAuthority` | `null` | `"37 C.F.R. \u00a7 42.75(c)(1); 37 C.F.R. \u00a7 42.71(d)(2)"` |
| PTAB | `/edges/edge-rehearing-denied-affirmed-director-review-request-filed/deadline/extendable` | `false` | `true` |
| PTAB | `/edges/edge-rehearing-denied-affirmed-director-review-request-filed/deadline/extensionAuthority` | `null` | `"37 C.F.R. \u00a7 42.75(a); 37 C.F.R. \u00a7 42.75(c)(1); 37 C.F.R. \u00a7 42.71(d)(2)"` |
| PTAB | `/edges/edge-federal-circuit-window-federal-circuit-notice-of-appeal/deadline/extendable` | `false` | `true` |
| PTAB | `/edges/edge-federal-circuit-window-federal-circuit-notice-of-appeal/deadline/extensionAuthority` | `null` | `"37 C.F.R. \u00a7 90.3(a)(1); 37 C.F.R. \u00a7 90.3(b)(1)"` |
| PTAB | `/edges/edge-federal-circuit-window-appeal-forfeited-unappealed/deadline/extendable` | `false` | `true` |
| PTAB | `/edges/edge-federal-circuit-window-appeal-forfeited-unappealed/deadline/extensionAuthority` | `null` | `"37 C.F.R. \u00a7 90.3(a)(1); 37 C.F.R. \u00a7 90.3(b)(1)"` |
| TTAB | `/edges/edge-01-pre-ext-first-30-1/deadline/extendable` | `true` | `null` |
| TTAB | `/edges/edge-01-pre-ext-first-30-1/deadline/extensionAuthority` | `"37 CFR 2.102(c)(2)-(3)"` | `null` |
| TTAB | `/edges/edge-01-pre-ext-first-90-1/deadline/extendable` | `true` | `null` |
| TTAB | `/edges/edge-01-pre-ext-first-90-1/deadline/extensionAuthority` | `"37 CFR 2.102(c)(3)"` | `null` |
| TTAB | `/edges/edge-01-pre-opposition-filed-1/deadline/extendable` | `true` | `null` |
| TTAB | `/edges/edge-01-pre-opposition-filed-1/deadline/extensionAuthority` | `"37 CFR 2.102(c)"` | `null` |
| TTAB | `/edges/edge-01-pre-ext-second-60-1/deadline/extendable` | `true` | `null` |
| TTAB | `/edges/edge-01-pre-ext-second-60-1/deadline/extensionAuthority` | `"37 CFR 2.102(c)(3)"` | `null` |
| TTAB | `/edges/edge-03-pre-2-opposition-filed-1/deadline/extendable` | `true` | `null` |
| TTAB | `/edges/edge-03-pre-2-opposition-filed-1/deadline/extensionAuthority` | `"37 CFR 2.102(c)(3) (applicant consent only)"` | `null` |
| TTAB | `/edges/edge-04-standstill-2-opposition-filed-1/deadline/extendable` | `false` | `null` |
| TTAB | `/edges/answer-plain/deadline/extendable` | `null` | `true` |
| TTAB | `/edges/answer-plain/deadline/extensionAuthority` | `"FRCP 6(b)(1)(A), applied through 37 CFR 2.116(a) (interpretation)"` | `"37 CFR 2.105(a)"` |
| TTAB | `/edges/answer-s18/deadline/extendable` | `null` | `true` |
| TTAB | `/edges/answer-s18/deadline/extensionAuthority` | `"FRCP 6(b)(1)(A), applied through 37 CFR 2.116(a) (interpretation)"` | `"37 CFR 2.105(a)"` |
| TTAB | `/edges/edge-07-opposition-to-trial-notice-of-default-1/deadline/extendable` | `null` | `true` |
| TTAB | `/edges/edge-07-opposition-to-trial-notice-of-default-1/deadline/extensionAuthority` | `"FRCP 6(b)(1)(A), applied through 37 CFR 2.116(a) (interpretation)"` | `"37 CFR 2.105(a)"` |
| TTAB | `/edges/edge-07-opposition-to-trial-initial-disclosures-1/deadline/extensionAuthority` | `"37 CFR 2.120(a)(2)(iv)"` | `"37 CFR 2.120(a)(2)(ii)"` |
| TTAB | `/edges/edge-07-opposition-to-trial-discovery-closed-1/deadline/extensionAuthority` | `"37 CFR 2.120(a)(2)(iv)"` | `"37 CFR 2.120(a)(2)(i)"` |
| TTAB | `/edges/edge-08-decision-district-outcome-1/deadline/extendable` | `false` | `true` |
| TTAB | `/edges/edge-08-decision-district-outcome-1/deadline/extensionAuthority` | `null` | `"37 CFR 2.145(d)(3)"` |

Per-package differences: CAFc 20, CoFC 20, FRAP 42, FRCP 30, FRCrimP 14, ITC 12, links 12, MPEP 31, PTAB 27, TTAB 21. Total: 229.

## lg-pb1fe.23 restoration status

The 77 baseline pointers identified by the PM review were traced to their existing typed period grants, delegations, or locks. Rule references were reduced, edition-qualified where needed, and written on those typed capabilities. Generic descriptions without a named source were not turned into citations. No grant, period cite, or grant behavior was changed.

**DECIDED:** Keep authority on the period's existing typed capability -- the bead's rule assigns authority to the period and forbids creating grants or changing period cites.

### Restored typed capabilities

Each listed capability cite covers the worklist edge or shared-period edges named in the baseline table in this report. Citations are on the existing declaration; the period's own cite remains as authored.

| Package | Existing typed period capability | Cite written |
|---|---|---|
| CAFc | District-court notice, PTAB notice, Court of Federal Claims notice, TTAB notice | `FRAP (2002 ed.) 4(a)(5)(A); FRAP (2009 ed.) 4(a)(5)(A)`; `37 CFR 90.3(c)(1)`; `FRAP (2002 ed.) 4(a)(5)(A); FRAP (2009 ed.) 4(a)(5)(A)`; `37 CFR 2.145(e)(1)` |
| CAFc | Docketing statement, cross appeal, appellant/appellee/reply briefs, appendix, mandate costs, cert petition | `Fed. Cir. R. 26(b)`; `FRAP (2002 ed.) 4(a)(5)(A); FRAP (2009 ed.) 4(a)(5)(A)`; `Fed. Cir. R. 26(b)` on each existing grant; `Sup. Ct. R. 13.5` |
| CoFC | Response fork answer period | `RCFC 6(b)` |
| CoFC | JPSR, disclosure, infringement/invalidity contentions, claim chart, joint claim construction, discovery, hearing schedule periods | `PRCFC Rule 1(b)` or `PRCFC Rule 1(b); RCFC 24` on the relevant existing capability |
| FRAP | Criminal notice, cross appeal, transcript completion, stay, appellant/appellee/reply briefs, rehearing | `FRAP (2009 ed.) 4(b)(4)`; `FRAP (2002 ed.) 4(a)(5)(A); FRAP (2009 ed.) 4(a)(5)(A)`; `FRAP 11(b)(1)(B)`; `FRAP 26(b)` on existing motion/brief grants; `FRAP 26(b); FRAP 31(a)(1)` for reply; `FRAP (2009 ed.) 40(a)(1)` |
| FRCP | Fee-motion periods; civil notice and cross-appeal periods | `FRCP 6(b)(1)`; `FRAP (2002 ed.) 4(a)(5)(A); FRAP (2009 ed.) 4(a)(5)(A)` |
| FRCrimP | Indictment, preliminary hearing, grand jury, speedy-trial exclusion, Rule 33 motion | `18 U.S.C. § 3161(h)`; `Fed. R. Crim. P. 5.1(c)`; `18 U.S.C. § 3161(b)`; `18 U.S.C. § 3161(h)`; `Fed. R. Crim. P. 33(b)(2)` |
| ITC | Response period | `cfr210.6.a` |
| MPEP | Missing parts, applicant response, after-final amendment/RCE/notice/continuation, election replies, reexamination owner statement | `37 CFR 1.136(a)` on the relevant existing grant; `37 CFR 41.31(d); 37 CFR 1.136(a)`; `37 CFR 1.550(c)` |
| PTAB | POPR, patent-owner response, petitioner reply, sur-reply | `37 CFR 42.107(b)`; `37 CFR 42.5(c)(2)`; `37 CFR 42.103` |
| TTAB | Answer; initial disclosures and discovery | `FRCP 6(b)(1)(A); 37 CFR 2.116(a)`; `37 CFR 2.120(a)(2)(iv)` |

The baseline table's prose was not promoted into typed limits or standards. Shared periods use one period-level authority reference; where a baseline edge listed more than one source, the cite contains the distinct references joined with `; `.

### Kept existing period cites

No existing period cite was overwritten. Examples where the restored authority is distinct from the period cite are:

| Period | Existing period cite retained | Restored authority cite |
|---|---|---|
| FRAP notice-civil periods | `28 U.S.C. § 2107(a)` / `28 U.S.C. § 2107(b)` | Edition-qualified FRAP 4(a)(5) reference |
| FRAP cert period | `28 U.S.C. § 2101(c)` | `Sup. Ct. R. 13.5` |
| FRCrimP criminal-appeal period | `FRAP 4(b)(1)(A); FRAP 4(b)(4)` | Existing period cite retained; no replacement was made |
| PTAB appeal period | `37 C.F.R. § 90.3(a)(1); 37 C.F.R. § 90.3(b)(1)` | `37 CFR 90.3(c)(1)` on the applicable typed grant |

### No source named

These baseline strings name no rule or statute as the source of extension power, so no cite was added. The rest of each typed declaration is unchanged.

| Package | Edge | Baseline string | Treatment |
|---|---|---|---|
| CoFC | `e-jpsr-to-scheduling` | `court order` | no cite: no source named |
| CoFC | `e-discclosed-to-sj` | `court order / local practice` | no cite: no source named |
| CoFC | `e-ccdiscoveryclosed-to-hearing` | `court order` | no cite: no source named |
| ITC | `discovery-respond-interrogatories-rfp` | `ALJ-specified period` | no cite: no source named |
| ITC | `discovery-rfa-deemed-admitted` | `ALJ-specified period` | no cite: no source named |
| ITC | `discovery-motion-quash-subpoena` | `ALJ-extended period` | no cite: no source named |
| ITC | `discovery-file-privilege-log` | `good cause` | no cite: no source named |
| ITC | `edge-discovery-open-public-version-filed` | `good cause` | no cite: no source named |
| ITC | `edge-summary-determination-motion-filed-summary-determination-opposition` | `ALJ discretion` | no cite: no source named |
| ITC | `edge-petition-for-review-window-petition-filed` | `good cause` | no cite: no source named |
| ITC | `edge-petition-filed-petition-response` | `good cause` | no cite: no source named |

### Residual extendable edges with null authority

The final pack build lists 27 edges with `extendable: true` and `extensionAuthority: null`. Eleven are the no-source rows above. The other sixteen had no baseline authority string, so no restoration was due. This corrects the previous report's false claim that all remaining nulls had no typed declaration.

| Package | Edge | Baseline | New value | Reason |
|---|---|---|---|---|
| CAFc | `rehearing-file` | no baseline string | `null` | no baseline authority pointer |
| CAFc | `rehearing-time-expires` | no baseline string | `null` | no baseline authority pointer |
| CAFc | `rehearing-denied-to-mandate` | no baseline string | `null` | no baseline authority pointer |
| CoFC | `e-ruling12b-denied` | no baseline string | `null` | no baseline authority pointer |
| FRAP | `edge-transcript-order-fork-transcript-ordered` | no baseline string | `null` | no baseline authority pointer |
| FRAP | `no-transcript-certificate` | no baseline string | `null` | no baseline authority pointer |
| FRAP | `edge-rehearing-granted-amended-opinion-mandate-issued` | no baseline string | `null` | no baseline authority pointer |
| FRAP | `edge-rehearing-en-banc-both-denied-mandate-issued` | no baseline string | `null` | no baseline authority pointer |
| FRCrimP | `edge-guilty-verdict-rule29-motion` | no baseline string | `null` | no baseline authority pointer |
| MPEP | `ipr-popr-filed` | no baseline string | `null` | no baseline authority pointer |
| MPEP | `pgr-popr-filed` | no baseline string | `null` | no baseline authority pointer |
| PTAB | `edge-petition-defect-noticed-ipr-petition-filed` | no baseline string | `null` | no baseline authority pointer |
| PTAB | `edge-post-fwd-options-rehearing-request-filed` | no baseline string | `null` | no baseline authority pointer |
| PTAB | `edge-post-fwd-options-director-review-request-filed` | no baseline string | `null` | no baseline authority pointer |
| PTAB | `edge-rehearing-denied-affirmed-director-review-request-filed` | no baseline string | `null` | no baseline authority pointer |
| TTAB | `edge-08-decision-district-outcome-1` | no baseline string | `null` | no baseline authority pointer |

### Measured counts and checks

The command that emitted the final per-pack counts and residual edge lists was:

```bash
/work/litigate-main build lit/frcp lit/frap lit/frcrimp lit/cafc lit/cofc lit/itc lit/mpep lit/ptab lit/ttab --out /tmp/out
```

It returned **RC=0**. Counts of edges with string `extensionAuthority`, by the same pack JSON field, were:

| Package | a9dfb1d6a baseline | Main before this bead | After this bead |
|---|---:|---:|---:|
| CAFc | 12 | 1 | 13 |
| CoFC | 13 | 0 | 9 |
| FRAP | 26 | 1 | 26 |
| FRCP | 24 | 4 | 28 |
| FRCrimP | 9 | 0 | 7 |
| ITC | 10 | 1 | 2 |
| MPEP | 13 | 0 | 9 |
| PTAB | 7 | 0 | 9 |
| TTAB | 10 | 0 | 5 |
| **Total** | **124** | **7** | **108** |

The historical counts are the compiled baseline counts recorded in the supplied `litigate diff` comparison; main counts are the pre-edit pack build; after counts are from the command above. The `FRCP 6(b)(2)` lock check confirmed all four fee/post-trial edges lower `extendable: false` with authority `FRCP 6(b)(2)`. No lowered authority equals its own period cite. `/work/litigate-main check lit/frcp lit/frap lit/frcrimp lit/cafc lit/cofc lit/itc lit/mpep lit/ptab lit/ttab --format json` returned **RC=0**, zero errors. `/work/litigate-main check lit cases --format json` returned **RC=0**, zero errors (zero diagnostics). `just lit-sync-check`, run after committing the regenerated locks, returned **RC=0**. The PTAB lint-filing replay used `/work/litigate-main lint-filing cases/ptab-ipr-defense.case 'PreliminaryResponseFiled()' --date 2026-09-10 --by owner --format json` against both the clean-main library and current library; both returned RC=0 and identical `result` JSON (accepted filing, replay, discharge, trigger and open-deadline rows).

The baseline total count includes pointers from the earlier conflict report; those pointers are the historical comparison set, while the after count measures the compiled typed-authority field requested by this bead.

### All baseline authority pointers that still differ

This table compares every extension-authority pointer in the original `litigate diff` baseline table with the final packs. It includes normalized/reduced citations, shared-period authorities, no-source values, and pre-existing differences from main. The rule column identifies the retry rule applied (or identifies values already present before this attempt).

| Package | Edge | Baseline value | Final value | Rule |
|---|---|---|---|---|
| CAFc | `district-court-file-noa` | `"FRAP 4(a)(5) (up to 30 additional days, excusable neglect/good cause)"` | `"FRAP (2009 ed.) 4(a)(5)(A)"` | rule 2: reduced/normalized named reference |
| CAFc | `ptab-file-noa` | `"37 C.F.R. § 90.3(c) (Director may extend on a timely, good-cause request filed before expiration, or post-expiration on a showing of excusable neglect)"` | `"37 C.F.R. § 90.3(c)(1)"` | rule 3: baseline names no authority source |
| CAFc | `itc-file-noa` | `null` | `"FRAP 26(b)(2)"` | outside this bead: authority already differed on main |
| CAFc | `cofc-file-noa` | `"FRAP 4(a)(5) (up to 30 additional days, excusable neglect/good cause)"` | `"FRAP (2009 ed.) 4(a)(5)(A)"` | rule 2: reduced/normalized named reference |
| CAFc | `ttab-file-noa` | `"37 C.F.R. § 2.145(e) (Director may extend on good cause if requested before expiry, or on excusable neglect if requested after)"` | `"37 C.F.R. § 2.145(e)(1)"` | rule 3: baseline names no authority source |
| CAFc | `cross-appeal-file` | `"FRAP 4(a)(5): on motion no later than 30 days after the period expires, for excusable neglect or good cause"` | `"FRAP (2009 ed.) 4(a)(5)(A)"` | rule 2: reduced/normalized named reference |
| CAFc | `cert-window-file` | `"Sup. Ct. R. 13.5 (single application to a Circuit Justice, before the deadline, for good cause)"` | `"Sup. Ct. R. 13.5"` | rule 3: baseline names no authority source |
| CoFC | `e-responsefork-answer` | `"RCFC 6(b) stipulation or court order"` | `"RCFC 6(b)"` | rule 2: reduced/normalized named reference |
| CoFC | `e-responsefork-motion` | `"RCFC 6(b) stipulation or court order"` | `"RCFC 6(b)"` | rule 2: reduced/normalized named reference |
| CoFC | `e-jpsr-to-scheduling` | `"court order"` | `null` | rule 3: baseline names no authority source |
| CoFC | `e-discclosed-to-sj` | `"court order / local practice"` | `null` | rule 3: baseline names no authority source |
| CoFC | `e-14bfork-none` | `"court order (complexity/good cause, PRCFC Rule 1(b))"` | `"PRCFC Rule 1(b)"` | rule 2: reduced/normalized named reference |
| CoFC | `e-14bserved-to-jpsr` | `"court order (complexity/good cause, PRCFC Rule 1(b))"` | `"PRCFC Rule 1(b)"` | rule 2: reduced/normalized named reference |
| CoFC | `e-po-to-infringement` | `"PRCFC Rule 1(b) / Rule 24 good cause"` | `"PRCFC Rule 1(b); RCFC 24"` | rule 2: reduced/normalized named reference |
| CoFC | `e-infringement-to-invalidity` | `"PRCFC Rule 1(b) / Rule 24 good cause"` | `"PRCFC Rule 1(b); RCFC 24"` | rule 2: reduced/normalized named reference |
| CoFC | `e-invalidity-to-ccexchange` | `"PRCFC Rule 1(b) good cause"` | `"PRCFC Rule 1(b)"` | rule 2: reduced/normalized named reference |
| CoFC | `e-ccexchange-to-jcc` | `"PRCFC Rule 1(b) good cause"` | `"PRCFC Rule 1(b)"` | rule 2: reduced/normalized named reference |
| CoFC | `e-jcc-to-ccdiscoveryclosed` | `"PRCFC Rule 1(b) good cause"` | `"PRCFC Rule 1(b)"` | rule 2: reduced/normalized named reference |
| CoFC | `e-ccdiscoveryclosed-to-hearing` | `"court order"` | `null` | rule 3: baseline names no authority source |
| CoFC | `e-posttrialwindow-motion` | `"none -- RCFC 6(b) forbids extension"` | `null` | rule 5: no typed declaration |
| FRAP | `edge-final-judgment-entered-appeal-clock-open` | `"Fed. R. App. P. 4(a)(5)"` | `"FRAP (2002 ed.) 4(a)(5)(A)"` | rule 3: baseline names no authority source |
| FRAP | `interlocutory-injunction-order` | `"Fed. R. App. P. 4(a)(5)"` | `"FRAP (2002 ed.) 4(a)(5)(A)"` | rule 3: baseline names no authority source |
| FRAP | `interlocutory-collateral-order` | `"Fed. R. App. P. 4(a)(5)"` | `"FRAP (2002 ed.) 4(a)(5)(A)"` | rule 3: baseline names no authority source |
| FRAP | `interlocutory-1292b-certified` | `"Fed. R. App. P. 5(a)(3)"` | `"FRAP 26(b)(1)"` | rule 3: baseline names no authority source |
| FRAP | `notice-of-appeal-civil` | `"FRAP 4(a)(5) (may extend up to 30 additional days only via motion before expiration)"` | `"FRAP (2002 ed.) 4(a)(5)(A)"` | rule 2: reduced/normalized named reference |
| FRAP | `notice-of-appeal-us-party` | `"FRAP 4(a)(5) (may extend up to 30 additional days only via motion before expiration)"` | `"FRAP (2009 ed.) 4(a)(5)(A)"` | rule 2: reduced/normalized named reference |
| FRAP | `notice-of-appeal-criminal` | `"FRAP 4(b)(4) (may be extended for excusable neglect or good cause, subject to Bowles jurisdictional limit)"` | `"FRAP (2009 ed.) 4(b)(4)"` | rule 2: reduced/normalized named reference |
| FRAP | `edge-appeal-clock-open-tolling-motion-filed` | `"FRCP 6(b) may extend the tolling-motion deadline by up to 28 days total (only for FRCP 50(b), 59(e), 60(b) motions)"` | `null` | rule 5: no typed declaration |
| FRAP | `edge-appeal-clock-open-post-deadline-relief-motion` | `"FRAP 4(a)(5)(A) (court may extend if motion filed before deadline expires); FRAP 4(a)(6) (may reopen for 14 days if lack of notice of entry)"` | `"FRAP (2002 ed.) 4(a)(5)(A)"` | rule 2: reduced/normalized named reference |
| FRAP | `edge-tolling-motion-disposed-appeal-clock-open` | `"FRAP 4(a)(5) (extension may apply to the restarted deadline)"` | `"FRAP (2002 ed.) 4(a)(5)(A)"` | rule 2: reduced/normalized named reference |
| FRAP | `edge-cross-appeal-fork-cross-appeal-filed` | `"FRAP 4(a)(5): on motion no later than 30 days after the period expires, for excusable neglect or good cause"` | `"FRAP (2009 ed.) 4(a)(5)(A)"` | rule 2: reduced/normalized named reference |
| FRAP | `edge-transcript-ordered-record-transmitted` | `"FRAP 11(b) (reporter may request extension for completion beyond 30 days; no formal party motion needed)"` | `"FRAP 11(b)(1)(B)"` | rule 2: reduced/normalized named reference |
| FRAP | `edge-stay-pending-appeal-fork-stay-granted` | `"FRAP 26(b); court order"` | `"FRAP 26(b)"` | rule 2: reduced/normalized named reference |
| FRAP | `edge-stay-pending-appeal-fork-stay-denied` | `"FRAP 26(b); court order"` | `"FRAP 26(b)"` | rule 2: reduced/normalized named reference |
| FRAP | `edge-appellant-brief-window-appellant-brief-filed` | `"FRAP 26(b); court discretion for good cause"` | `"FRAP 26(b); FRAP 26(b)"` | rule 2: reduced/normalized named reference |
| FRAP | `edge-appellant-brief-filed-appellee-brief-filed` | `"FRAP 26(b); court discretion for good cause"` | `"FRAP 26(b); FRAP 26(b)"` | rule 2: reduced/normalized named reference |
| FRAP | `edge-appellee-brief-filed-reply-brief-filed` | `"FRAP 26(b); court discretion for good cause; FRAP 31(a)(1) requires filing at least 7 days before argument"` | `"FRAP 26(b); FRAP 31(a)(1); FRAP 26(b); FRAP 31(a)(1)"` | rule 2: reduced/normalized named reference |
| FRAP | `edge-affirmed-rehearing-window` | `"FRAP 40(a)(1); circuit local rules may allow additional extensions by panel order or stipulation"` | `"FRAP (2009 ed.) 40(a)(1); FRAP (2009 ed.) 40(a)(1)"` | rule 2: reduced/normalized named reference |
| FRAP | `edge-reversed-rehearing-window` | `"FRAP 40(a)(1); circuit local rules"` | `"FRAP (2009 ed.) 40(a)(1); FRAP (2009 ed.) 40(a)(1)"` | rule 2: reduced/normalized named reference |
| FRAP | `edge-reversed-and-remanded-rehearing-window` | `"FRAP 40(a)(1); circuit local rules"` | `"FRAP (2009 ed.) 40(a)(1); FRAP (2009 ed.) 40(a)(1)"` | rule 2: reduced/normalized named reference |
| FRAP | `edge-vacated-and-remanded-rehearing-window` | `"FRAP 40(a)(1); circuit local rules"` | `"FRAP (2009 ed.) 40(a)(1); FRAP (2009 ed.) 40(a)(1)"` | rule 2: reduced/normalized named reference |
| FRAP | `edge-affirmed-in-part-reversed-in-part-rehearing-window` | `"FRAP 40(a)(1); circuit local rules"` | `"FRAP (2009 ed.) 40(a)(1); FRAP (2009 ed.) 40(a)(1)"` | rule 2: reduced/normalized named reference |
| FRAP | `edge-rehearing-window-rehearing-petition-filed` | `"FRAP 26(b) is silent on FRAP 40 deadlines; extension is theoretically possible for good cause but extremely rarely granted in practice"` | `"FRAP (2009 ed.) 40(a)(1); FRAP (2009 ed.) 40(a)(1)"` | rule 2: reduced/normalized named reference |
| FRAP | `edge-rehearing-window-mandate-issued` | `null` | `"FRAP (2009 ed.) 40(a)(1); FRAP (2009 ed.) 40(a)(1)"` | outside this bead: authority already differed on main |
| FRAP | `edge-mandate-issued-cert-petition-window` | `"Sup. Ct. R. 13.6"` | `"Sup. Ct. R. 13.5"` | rule 3: baseline names no authority source |
| FRAP | `edge-mandate-stayed-pending-cert-cert-petition-window` | `"Sup. Ct. R. 13.6"` | `"Sup. Ct. R. 13.5"` | rule 3: baseline names no authority source |
| FRAP | `edge-cert-petition-window-cert-petition-filed` | `"28 U.S.C. § 2101(c); Sup. Ct. R. 13.5, 30.3 (application to an individual Justice, up to 60 days; once denied, not renewable)"` | `"Sup. Ct. R. 13.5"` | rule 3: baseline names no authority source |
| FRCP | `edge-service-window-service-completed` | `"FRCP 4(m) good-cause showing"` | `"FRCP 6(b)(1)"` | rule 2: reduced/normalized named reference |
| FRCP | `edge-waiver-requested-waiver-response-window` | `"FRCP 6(b)(1): for good cause before the period expires, or on motion after it for excusable neglect"` | `"FRCP 6(b)(1)"` | rule 2: reduced/normalized named reference |
| FRCP | `edge-service-completed-rule12-response-window` | `"FRCP 6(b) stipulation or court order"` | `"FRCP 6(b)(1)"` | rule 2: reduced/normalized named reference |
| FRCP | `edge-removal-window-rule12-response-window` | `"FRCP 6(b) stipulation or court order"` | `"FRCP 6(b)(1)"` | rule 2: reduced/normalized named reference |
| FRCP | `edge-removed-to-federal-removed-response-window` | `"FRCP 6(b) stipulation or court order"` | `"FRCP 6(b)(1)"` | rule 2: reduced/normalized named reference |
| FRCP | `edge-remand-ruling-removed-response-window` | `"FRCP 6(b) stipulation or court order"` | `"FRCP 6(b)(1)"` | rule 2: reduced/normalized named reference |
| FRCP | `edge-rule12-response-window-answer-filed` | `"FRCP 6(b) stipulation or court order"` | `"FRCP 6(b)(1)"` | rule 2: reduced/normalized named reference |
| FRCP | `edge-rule12-response-window-special-appearance` | `"FRCP 6(b) stipulation or court order"` | `"FRCP 6(b)(1)"` | rule 2: reduced/normalized named reference |
| FRCP | `12b-motion-filed` | `"FRCP 6(b) stipulation or court order"` | `"FRCP 6(b)(1)"` | rule 2: reduced/normalized named reference |
| FRCP | `edge-waiver-response-window-answer-filed` | `"FRCP 6(b)(1): for good cause before the period expires, or on motion after it for excusable neglect"` | `"FRCP 6(b)(1)"` | rule 2: reduced/normalized named reference |
| FRCP | `waiver-12b-motion-filed` | `"FRCP 6(b)(1): for good cause before the period expires, or on motion after it for excusable neglect"` | `"FRCP 6(b)(1)"` | rule 2: reduced/normalized named reference |
| FRCP | `edge-removed-response-window-answer-filed` | `"FRCP 6(b) stipulation or court order"` | `"FRCP 6(b)(1)"` | rule 2: reduced/normalized named reference |
| FRCP | `removed-12b-motion-filed` | `"FRCP 6(b) stipulation or court order"` | `"FRCP 6(b)(1)"` | rule 2: reduced/normalized named reference |
| FRCP | `edge-amended-response-window-answer-filed` | `"FRCP 6(b) stipulation or court order"` | `"FRCP 6(b)(1)"` | rule 2: reduced/normalized named reference |
| FRCP | `amended-12b-motion-filed` | `"FRCP 6(b) stipulation or court order"` | `"FRCP 6(b)(1)"` | rule 2: reduced/normalized named reference |
| FRCP | `edge-mtd-ruling-answer-filed` | `"FRCP 6(b) stipulation or court order"` | `"FRCP 6(b)(1)"` | rule 2: reduced/normalized named reference |
| FRCP | `edge-amendment-as-of-right-window-amended-complaint-filed` | `"FRCP 6(b)(1): for good cause before the period expires, or on motion after it for excusable neglect"` | `"FRCP 6(b)(1)"` | rule 2: reduced/normalized named reference |
| FRCP | `edge-amended-complaint-filed-amended-response-window` | `"FRCP 6(b) stipulation or court order"` | `"FRCP 6(b)(1)"` | rule 2: reduced/normalized named reference |
| FRCP | `edge-rule26f-conference-initial-disclosures` | `"court order"` | `"FRCP 6(b)(1)"` | rule 3: baseline names no authority source |
| FRCP | `jury-demand-timely` | `"FRCP 6(b) good cause"` | `"FRCP 6(b)(1)"` | rule 2: reduced/normalized named reference |
| FRCP | `edge-final-judgment-entered-fee-motion-filed` | `"FRCP 6(b)(1): on a stipulated request or motion, or by the court"` | `"FRCP 6(b)(1)"` | rule 2: reduced/normalized named reference |
| FRCP | `edge-post-trial-motion-window-rule50b-jmol-filed` | `null` | `"FRCP 6(b)(2)"` | outside this bead: authority already differed on main |
| FRCP | `edge-post-trial-motion-window-rule59-new-trial-filed` | `null` | `"FRCP 6(b)(2)"` | outside this bead: authority already differed on main |
| FRCP | `edge-post-trial-motion-window-rule59e-amend-filed` | `null` | `"FRCP 6(b)(2)"` | outside this bead: authority already differed on main |
| FRCP | `edge-post-trial-motion-window-rule60-relief-filed` | `null` | `"FRCP 6(b)(2)"` | outside this bead: authority already differed on main |
| FRCP | `edge-post-trial-motion-window-fee-motion-filed` | `"FRCP 6(b)(1): on a stipulated request or motion, or by the court"` | `"FRCP 6(b)(1)"` | rule 2: reduced/normalized named reference |
| FRCP | `edge-notice-of-appeal-window-notice-of-appeal-filed` | `"FRAP 4(a)(5) excusable neglect or good cause, sought before or shortly after the deadline"` | `"FRAP (2009 ed.) 4(a)(5)(A)"` | rule 2: reduced/normalized named reference |
| FRCP | `edge-cross-appeal-window-cross-appeal-filed` | `"FRAP 4(a)(5): on motion no later than 30 days after the period expires, for excusable neglect or good cause"` | `"FRAP (2009 ed.) 4(a)(5)(A)"` | rule 2: reduced/normalized named reference |
| FRCrimP | `edge-initial-appearance-preliminary-hearing` | `"Fed. R. Crim. P. 5.1(c) — good cause, or extraordinary circumstances where justice requires the delay, even without the defendant's consent"` | `"Fed. R. Crim. P. 5.1(c)"` | rule 3: baseline names no authority source |
| FRCrimP | `edge-bound-over-grand-jury-grand-jury-convened` | `"18 U.S.C. § 3161(b) — one additional 30-day extension if no grand jury convened during the initial 30-day period in the district"` | `"18 U.S.C. § 3161(b)"` | rule 3: baseline names no authority source |
| FRCrimP | `edge-proceed-to-trial-jury-selection` | `"18 U.S.C. § 3161(h) (excludable delay)"` | `"18 U.S.C. § 3161(h)"` | rule 3: baseline names no authority source |
| FRCrimP | `edge-guilty-verdict-rule33-motion` | `"Rule 33(b) allows extension of the 14-day deadline if the court finds grounds within the original period; the 3-year newly-discovered-evidence window requires no extension"` | `"Fed. R. Crim. P. 33(b)(2)"` | rule 3: baseline names no authority source |
| FRCrimP | `edge-presentence-report-sentencing-hearing` | `"Fed. R. Crim. P. 32(e)(2) — parties may stipulate to a shorter period, or the court may set a different period for good cause"` | `null` | rule 3: baseline names no authority source |
| FRCrimP | `edge-judgment-entered-notice-of-appeal-window` | `"Fed. R. App. P. 4(b)(1)(B) — up to 30 additional days for good cause shown, before or after the deadline expires"` | `"FRAP 4(b)(4)"` | rule 3: baseline names no authority source |
| FRCrimP | `edge-notice-of-appeal-window-notice-of-appeal-filed` | `"Fed. R. App. P. 4(b)(4) — on a finding of excusable neglect or good cause, the court may extend before or after the 14 days expire, with or without motion and notice, for up to 30 days from the original expiry"` | `"FRAP 4(b)(4)"` | rule 3: baseline names no authority source |
| FRCrimP | `edge-notice-of-appeal-window-appeal-window-missed` | `"Fed. R. App. P. 4(b)(4)"` | `null` | rule 3: baseline names no authority source |
| ITC | `edge-response-period-open-response-filed` | `"ALJ good-cause order (Rule 210.6(a))"` | `"cfr210.6.a"` | rule 3: baseline names no authority source |
| ITC | `discovery-respond-interrogatories-rfp` | `"ALJ-specified period"` | `null` | rule 3: baseline names no authority source |
| ITC | `discovery-rfa-deemed-admitted` | `"ALJ-specified period"` | `null` | rule 3: baseline names no authority source |
| ITC | `discovery-motion-quash-subpoena` | `"ALJ-extended period"` | `null` | rule 3: baseline names no authority source |
| ITC | `discovery-file-privilege-log` | `"good cause"` | `null` | rule 3: baseline names no authority source |
| ITC | `edge-discovery-open-public-version-filed` | `"good cause"` | `null` | rule 3: baseline names no authority source |
| ITC | `edge-prehearing-summary-determination-motion-filed` | `"ALJ good-cause exception"` | `null` | rule 3: baseline names no authority source |
| ITC | `edge-summary-determination-motion-filed-summary-determination-opposition` | `"ALJ discretion"` | `null` | rule 3: baseline names no authority source |
| ITC | `edge-petition-for-review-window-petition-filed` | `"good cause"` | `null` | rule 3: baseline names no authority source |
| ITC | `edge-petition-filed-petition-response` | `"good cause"` | `null` | rule 3: baseline names no authority source |
| ITC | `edge-federal-circuit-appeal-window-appeal-filed` | `null` | `"FRAP 26(b)(2)"` | outside this bead: authority already differed on main |
| MPEP | `edge-applicant-response-window-extension-taken` | `"37 CFR 1.136(a)"` | `null` | rule 5: no typed declaration |
| MPEP | `edge-extension-taken-amendment-filed` | `"37 CFR 1.136(a)"` | `null` | rule 5: no typed declaration |
| MPEP | `edge-after-final-decision-notice-of-appeal` | `"37 CFR 41.31(d); 37 CFR 1.136(a)"` | `"37 CFR 1.136(a)"` | rule 2: reduced/normalized named reference |
| MPEP | `edge-notice-of-appeal-appeal-brief-filed` | `"37 CFR 1.136(a)"` | `null` | rule 5: no typed declaration |
| MPEP | `edge-notice-of-appeal-appeal-dismissed` | `"37 CFR 1.136(a)"` | `null` | rule 5: no typed declaration |
| PTAB | `edge-popr-window-popr-filed` | `"37 CFR 42.107(b) (one 30-day extension available without cause)"` | `"37 C.F.R. § 42.107(b)"` | rule 2: reduced/normalized named reference |
| PTAB | `edge-mandatory-disclosures-patent-owner-response-window` | `"37 CFR 42.5(c)(2)"` | `"37 C.F.R. § 42.5(c)(2)"` | rule 2: reduced/normalized named reference |
| PTAB | `edge-additional-discovery-motion-patent-owner-response-window` | `"37 CFR 42.5(c)(2)"` | `"37 C.F.R. § 42.5(c)(2)"` | rule 2: reduced/normalized named reference |
| PTAB | `edge-protective-order-ipr-patent-owner-response-window` | `"37 CFR 42.5(c)(2)"` | `"37 C.F.R. § 42.5(c)(2)"` | rule 2: reduced/normalized named reference |
| PTAB | `edge-patent-owner-response-window-patent-owner-response-filed` | `"37 CFR 42.103; scheduling order"` | `"37 C.F.R. § 42.5(c)(2)"` | rule 2: reduced/normalized named reference |
| PTAB | `edge-patent-owner-response-filed-petitioner-reply-brief` | `"37 CFR 42.103; scheduling order"` | `"37 C.F.R. § 42.103"` | rule 2: reduced/normalized named reference |
| PTAB | `edge-petitioner-reply-brief-patent-owner-sur-reply` | `"37 CFR 42.103; scheduling order"` | `"37 C.F.R. § 42.103"` | rule 2: reduced/normalized named reference |
| PTAB | `edge-federal-circuit-window-federal-circuit-notice-of-appeal` | `null` | `"37 C.F.R. § 90.3(c)(1)"` | outside this bead: authority already differed on main |
| PTAB | `edge-federal-circuit-window-appeal-forfeited-unappealed` | `null` | `"37 C.F.R. § 90.3(c)(1)"` | outside this bead: authority already differed on main |
| TTAB | `edge-01-pre-ext-first-30-1` | `"37 CFR 2.102(c)(2)-(3)"` | `null` | rule 5: no typed declaration |
| TTAB | `edge-01-pre-ext-first-90-1` | `"37 CFR 2.102(c)(3)"` | `null` | rule 5: no typed declaration |
| TTAB | `edge-01-pre-opposition-filed-1` | `"37 CFR 2.102(c)"` | `null` | rule 5: no typed declaration |
| TTAB | `edge-01-pre-ext-second-60-1` | `"37 CFR 2.102(c)(3)"` | `null` | rule 5: no typed declaration |
| TTAB | `edge-03-pre-2-opposition-filed-1` | `"37 CFR 2.102(c)(3) (applicant consent only)"` | `null` | rule 5: no typed declaration |
| TTAB | `answer-plain` | `"FRCP 6(b)(1)(A), applied through 37 CFR 2.116(a) (interpretation)"` | `"FRCP 6(b)(1)(A); 37 CFR 2.116(a)"` | rule 2: reduced/normalized named reference |
| TTAB | `answer-s18` | `"FRCP 6(b)(1)(A), applied through 37 CFR 2.116(a) (interpretation)"` | `"FRCP 6(b)(1)(A); 37 CFR 2.116(a)"` | rule 2: reduced/normalized named reference |
| TTAB | `edge-07-opposition-to-trial-notice-of-default-1` | `"FRCP 6(b)(1)(A), applied through 37 CFR 2.116(a) (interpretation)"` | `"FRCP 6(b)(1)(A); 37 CFR 2.116(a)"` | rule 2: reduced/normalized named reference |


## lg-pb1fe.19 decisions

DECIDED: Treat only the 68 changed package-edge pointers as decision rows; the six link-instance overlays are inherited copies and are not additional rule periods. Every line below cites the period rule read in the vendored text or records “rule silent.” The 19 false→true rows identify the typed grant and a short decisive quotation.

Baseline targeted command: `target/release/litigate check lit/frcp lit/frap lit/frcrimp lit/cafc lit/cofc lit/itc lit/mpep lit/ptab lit/ttab --profile strict --format json`; **69 errors**, including **54 L5002**, **0 L9002**. After decisions and edits, the same command returned **60 errors**, including **45 L5002**, **0 L9002**. The remaining errors are the pre-existing strict holes and canonical-format diagnostics; no new error code appeared.
Whole-corpus check: `target/release/litigate check lit cases --format json` exited 0 on clean-main baseline (0 errors) and on this tree (0 errors, 3,531 warnings; `ok: true`).

- `/edges/docketed-to-appearance/deadline/extendable` — `false → null` — rule silent; read Fed. Cir. R. 47.3(b)(1); Fed. Cir. R. 47.4(b) (the cited section states the period but grants no extension or forbids none).
- `/edges/rehearing-file/deadline/extendable` — `false → true` — extend by typed grant; cite Fed. Cir. R. 40(f)(1); “the time ... may be extended by order”.
- `/edges/rehearing-time-expires/deadline/extendable` — `null → true` — extend by case order under Fed. Cir. R. 40(f)(1); FRAP 40(d)(1); “the time ... may be extended by order”.
- `/edges/rehearing-denied-to-mandate/deadline/extendable` — `null → true` — extend by case order under FRAP 41(b); Fed. Cir. R. 41; “the court may shorten or extend the time by order”.
- `/edges/e-protest-notice-to-complaint/deadline/extendable` — `false → null` — rule silent; read RCFC App. C 2; RCFC 3 (the cited section states the period but grants no extension or forbids none).
- `/edges/e-ruling12b-denied/deadline/extendable` — `false → true` — extend by typed grant; cite RCFC 12(a)(4)(A); “Unless the court sets a different time”.
- `/edges/e-order-to-settlement/deadline/extendable` — `false → null` — rule silent; read PRCFC 16(a) (the cited section states the period but grants no extension or forbids none).
- `/edges/e-settlementmeet-to-fork/deadline/extendable` — `false → null` — rule silent; read PRCFC 17 (the cited section states the period but grants no extension or forbids none).
- `/edges/e-posttrialwindow-motion/deadline/extendable` — `false → null` — rule silent; read RCFC 59(a)-(b), (e) (the cited section states the period but grants no extension or forbids none).
- `/edges/e-costs-to-prevailing/deadline/extendable` — `false → null` — rule silent; read RCFC 54(d); 28 U.S.C. §2412(d)(1)(B) (the cited section states the period but grants no extension or forbids none).
- `/edges/notice-of-appeal-civil/deadline/extendable` — `false → true` — extend by typed grant; cite FRAP 4(a)(5)(A); “may extend the time to file a notice of appeal”.
- `/edges/notice-of-appeal-us-party/deadline/extendable` — `false → true` — extend by typed grant; cite FRAP 4(a)(5)(A); “may extend the time to file a notice of appeal”.
- `/edges/notice-of-appeal-criminal/deadline/extendable` — `false → true` — extend by typed grant; cite FRAP 4(b)(4); “the district court may extend the time”.
- `/edges/edge-appeal-clock-open-tolling-motion-filed/deadline/extendable` — `false → null` — locked against extension by FRCP 6(b)(2); “may not extend the time to act under Rule 50(b), 52(b), 59(b), (d), and (e)”.
- `/edges/edge-tolling-motion-disposed-appeal-clock-open/deadline/extendable` — `false → true` — extend by typed grant; cite FRAP 4(a)(5)(A); “may extend the time to file a notice of appeal”.
- `/edges/edge-transcript-order-fork-transcript-ordered/deadline/extendable` — `false → true` — extend by typed grant; cite FRAP 26(b)(1); “may extend any time prescribed by these rules for good cause”.
- `/edges/no-transcript-certificate/deadline/extendable` — `false → true` — extend by typed grant; cite FRAP 26(b)(1); “may extend any time prescribed by these rules for good cause”.
- `/edges/edge-rehearing-window-rehearing-petition-filed/deadline/extendable` — `false → true` — extend by typed grant; cite FRAP 40(d)(1); “unless the time is shortened or extended by order or local rule”.
- `/edges/edge-rehearing-window-mandate-issued/deadline/extendable` — `null → true` — extend by case order under FRAP 40(d)(1); “unless the time is shortened or extended by order or local rule”.
- `/edges/edge-rehearing-granted-amended-opinion-mandate-issued/deadline/extendable` — `null → true` — extend by case order under FRAP 41(b); “the court may shorten or extend the time by order”.
- `/edges/edge-rehearing-en-banc-both-denied-mandate-issued/deadline/extendable` — `null → true` — extend by case order under FRAP 41(b); “the court may shorten or extend the time by order”.
- `/edges/edge-removal-window-removed-to-federal/deadline/extendable` — `false → null` — rule silent; read 28 U.S.C. 1446(a); 1446(b)(1) (the cited section states the period but grants no extension or forbids none).
- `/edges/edge-removed-to-federal-remand-ruling/deadline/extendable` — `false → null` — rule silent; read 28 U.S.C. 1447(c) (the cited section states the period but grants no extension or forbids none).
- `/edges/edge-guilty-verdict-rule29-motion/deadline/extendable` — `false → true` — extend by typed grant; cite FRCrimP 45(b)(1); “may extend the time to file a motion under Rule 29”.
- `/edges/edge-presentence-report-sentencing-hearing/deadline/extendable` — `true → null` — court may change a Rule 32 time limit for good cause under FRCrimP 32(b)(2); the edge is an inline backward deadline, not linked to the typed period; “the court may change any time limits prescribed in this rule”.
- `/edges/edge-judgment-entered-rule35-motion/deadline/extendable` — `false → null` — locked against extension by FRCrimP 45(b)(2); “the court may not extend the time to take any action under Rule 35”.
- `/edges/edge-notice-of-appeal-window-appeal-window-missed/deadline/extendable` — `true → null` — extend by court under FRAP 4(b)(4); edge now references the typed `criminal-appeal` period; “the district court may extend the time”.
- `/edges/edge-prehearing-summary-determination-motion-filed/deadline/extendable` — `true → null` — ALJ may alter the motion window under 19 C.F.R. §§ 210.6(a), 210.18(a); inline deadline has no named typed period; “unless otherwise ordered by the administrative law judge”.
- `/edges/edge-pre-filing-provisional-filed/deadline/extendable` — `false → null` — rule silent; read 35 U.S.C. 111(b); 37 CFR 1.53(c) (the cited section states the period but grants no extension or forbids none).
- `/edges/edge-provisional-filed-nonprovisional-filed/deadline/extendable` — `false → null` — rule silent; read 35 U.S.C. 111(a); 35 U.S.C. 119(e); 37 CFR 1.53(b) (the cited section states the period but grants no extension or forbids none).
- `/edges/edge-applicant-response-window-extension-taken/deadline/extendable` — `true → null` — extendable by applicant through fee-paid monthly extensions, 37 C.F.R. § 1.136(a); period `extension-month` itself has no month-bound grant form in the schema; “the time for reply ... may be extended”.
- `/edges/edge-extension-taken-amendment-filed/deadline/extendable` — `true → null` — extendable by applicant under 37 C.F.R. § 1.136(a), within the six-month ceiling of 35 U.S.C. § 133; month-based grant cannot be encoded in the day-bound schema; “up to five months”.
- `/edges/edge-extension-taken-extension-taken/deadline/extendable` — `false → null` — extendable by applicant under 37 C.F.R. § 1.136(a), within the six-month ceiling of 35 U.S.C. § 133; month-based grant cannot be encoded in the day-bound schema; “up to five months”.
- `/edges/edge-notice-of-appeal-appeal-brief-filed/deadline/extendable` — `true → null` — extendable under 37 C.F.R. § 41.37(e) via § 1.136(a); five-month limit is month-based and cannot be encoded in the day-bound schema; “time periods set forth in this section are extendable”.
- `/edges/edge-notice-of-appeal-appeal-dismissed/deadline/extendable` — `true → null` — extendable under 37 C.F.R. § 41.37(e) via § 1.136(a); five-month limit is month-based and cannot be encoded in the day-bound schema; “time periods set forth in this section are extendable”.
- `/edges/edge-examiners-answer-ptab-decision/deadline/extendable` — `false → null` — rule silent; read 37 CFR 41.20(b)(4); 37 CFR 41.45; MPEP 1208.01 (the cited section states the period but grants no extension or forbids none).
- `/edges/edge-ptab-affirmed-federal-circuit-appeal/deadline/extendable` — `false → null` — extendable by Director under 37 C.F.R. § 90.3(c)(1); “The Director ... may extend the time”.
- `/edges/edge-ptab-affirmed-sec145-action/deadline/extendable` — `false → null` — extendable by Director under 37 C.F.R. § 90.3(c)(1); “The Director ... may extend the time”.
- `/edges/edge-noa-received-issue-fee-paid/deadline/extendable` — `false → null` — locked against extension by 37 C.F.R. § 1.311(a); “This three-month period is not extendable”.
- `/edges/reissue-broadening-filed/deadline/extendable` — `false → null` — rule silent; read 35 U.S.C. § 251(d); MPEP 1412 (the cited section states the period but grants no extension or forbids none).
- `/edges/edge-patent-challenged-ipr-petition-filed/deadline/extendable` — `false → null` — rule silent; read 35 U.S.C. 311–314; 37 CFR 42.100 (the cited section states the period but grants no extension or forbids none).
- `/edges/edge-patent-challenged-pgr-petition-filed/deadline/extendable` — `false → null` — rule silent; read 35 U.S.C. 321–324; 37 CFR 42.200 (the cited section states the period but grants no extension or forbids none).
- `/edges/ipr-popr-filed/deadline/extendable` — `false → true` — extend by typed grant; cite 37 C.F.R. §§ 42.107(b), 42.5(c)(1); “may request an extension ... for good cause”.
- `/edges/pgr-popr-filed/deadline/extendable` — `false → true` — extend by typed grant; cite 37 C.F.R. §§ 42.207(b), 42.5(c)(1); “may request an extension ... for good cause”.
- `/edges/edge-patent-granted-pgr-petition-filed/deadline/extendable` — `false → null` — rule silent; read 35 U.S.C. § 321(c) (the cited section states the period but grants no extension or forbids none).
- `/edges/edge-infringement-complaint-served-ipr-petition-filed/deadline/extendable` — `false → null` — rule silent; read 35 U.S.C. § 311; 35 U.S.C. § 315(b) (the cited section states the period but grants no extension or forbids none).
- `/edges/edge-challenge-vehicle-decision-pgr-petition-filed/deadline/extendable` — `false → null` — rule silent; read 35 U.S.C. § 321(c) (the cited section states the period but grants no extension or forbids none).
- `/edges/e-time-bar-barred/deadline/extendable` — `false → null` — rule silent; read 35 U.S.C. §315(b) (the cited section states the period but grants no extension or forbids none).
- `/edges/edge-petition-defect-noticed-ipr-petition-filed/deadline/extendable` — `false → true` — extend by typed grant; cite 37 C.F.R. § 42.5(c)(1); “Times set by rule ... may be modified by order”.
- `/edges/edge-popr-filed-institution-decision/deadline/extendable` — `false → null` — rule silent; read 35 U.S.C. § 314(b) (the cited section states the period but grants no extension or forbids none).
- `/edges/edge-popr-waived-institution-decision/deadline/extendable` — `false → null` — rule silent; read 35 U.S.C. 314(b)(2), 37 CFR 42.107(b) (the cited section states the period but grants no extension or forbids none).
- `/edges/edge-institution-denied-merits-challenge-vehicle-decision/deadline/extendable` — `false → null` — rule silent; read 35 U.S.C. § 314(a) (the cited section states the period but grants no extension or forbids none).
- `/edges/edge-institution-denied-fintiv-challenge-vehicle-decision/deadline/extendable` — `false → null` — rule silent; read 35 U.S.C. § 314(a); 35 U.S.C. § 325(d) (the cited section states the period but grants no extension or forbids none).
- `/edges/edge-post-fwd-options-rehearing-request-filed/deadline/extendable` — `false → true` — extend by typed grant; cite 37 C.F.R. § 42.5(c)(1); “Times set by rule ... may be modified by order”.
- `/edges/edge-post-fwd-options-director-review-request-filed/deadline/extendable` — `false → true` — extend by typed grant; cite 37 C.F.R. § 42.5(c)(1); “Times set by rule ... may be modified by order”.
- `/edges/edge-rehearing-denied-affirmed-director-review-request-filed/deadline/extendable` — `false → true` — extend by typed grant; cite 37 C.F.R. § 42.5(c)(1); “Times set by rule ... may be modified by order”.
- `/edges/edge-federal-circuit-window-federal-circuit-notice-of-appeal/deadline/extendable` — `false → true` — extend by typed grant; cite 37 C.F.R. § 90.3(c)(1); “The Director ... may extend the time”.
- `/edges/edge-federal-circuit-window-appeal-forfeited-unappealed/deadline/extendable` — `false → true` — extend by typed grant; cite 37 C.F.R. § 90.3(c)(1); “The Director ... may extend the time”.
- `/edges/edge-01-pre-ext-first-30-1/deadline/extendable` — `true → null` — extendable by Board under 37 C.F.R. § 2.102(c)(1)(i); “a single request for an additional thirty days”.
- `/edges/edge-01-pre-ext-first-90-1/deadline/extendable` — `true → null` — extendable by Board under 37 C.F.R. § 2.102(c)(2); “an additional sixty days for good cause”.
- `/edges/edge-01-pre-opposition-filed-1/deadline/extendable` — `true → null` — extendable by Board under 37 C.F.R. § 2.102(c)(1); “a request to extend must be filed before those thirty days expire”.
- `/edges/edge-01-pre-ext-second-60-1/deadline/extendable` — `true → null` — extendable by Board under 37 C.F.R. § 2.102(c)(3); “the applicant consents to the extension”.
- `/edges/edge-03-pre-2-opposition-filed-1/deadline/extendable` — `true → null` — extendable by Board under 37 C.F.R. § 2.102(c)(3); “the applicant consents to the extension”.
- `/edges/edge-04-standstill-2-opposition-filed-1/deadline/extendable` — `false → null` — locked at the 180-day outer limit by 37 C.F.R. § 2.102(c)(3); “in no case later than 180 days”.
- `/edges/answer-plain/deadline/extendable` — `null → true` — extend by Board for good cause through FRCP 6(b)(1)(A), applied by 37 C.F.R. § 2.116(a); “wherever applicable and appropriate”.
- `/edges/answer-s18/deadline/extendable` — `null → true` — extend by Board for good cause through FRCP 6(b)(1)(A), applied by 37 C.F.R. § 2.116(a); “wherever applicable and appropriate”.
- `/edges/edge-07-opposition-to-trial-notice-of-default-1/deadline/extendable` — `null → true` — extend by Board for good cause through FRCP 6(b)(1)(A), applied by 37 C.F.R. § 2.116(a); “wherever applicable and appropriate”.
- `/edges/edge-08-decision-district-outcome-1/deadline/extendable` — `false → true` — extend by typed grant; cite 37 C.F.R. § 2.145(e)(1); “The Director ... may extend the time”.
