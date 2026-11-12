# Strict L5002 federal package regression (lg-pb1fe.35)

## Result

**Partial.** The 20 requested packages were classified from saved strict JSON outputs. Source-backed changes in PTAB, SCOTUS, FRAP, CIT, CAVC, FRCP, and CoFC address explicit statutory locks, the 2012 PTAB rule text, CAVC/USCIT general extension rules, court-set fee deadlines, and 2009 FRAP citations. Remaining periods and source-version proofs are listed below with their source-based reasons. No extension authority was added solely to clear a diagnostic.

- Baseline strict check: 0 of 20 packages passed; all 20 had L5002 errors.
- Final strict check: 2 of 20 packages passed. Package errors remaining are shown below.
- Final `just lit-sync-check`: exit 0 (locks and shared clock library current).
- `target/release/litigate check lit cases --format json`: exit 0; `ok=true`, 0 errors, 3,496 diagnostics (warnings included).
- Targeted package tests: CIT 69 passed / 0 failed; CAVC 76 / 0; FRAP 42 / 0; FRCP 49 / 0; CoFC 46 / 0; PTAB 35 / 0; SCOTUS 18 / 0.
- Every final targeted test command exited 0.

## Strict results

| Package | Before errors | After errors | Strict passes after |
|---|---:|---:|:---:|
| `cit` | 12 | 0 | yes |
| `mpep` | 11 | 11 | no |
| `frap` | 10 | 8 | no |
| `caaf` | 9 | 9 | no |
| `cavc` | 9 | 0 | yes |
| `ptab` | 9 | 2 | no |
| `cofc` | 6 | 4 | no |
| `scotus` | 5 | 4 | no |
| `frcp` | 4 | 2 | no |
| `habeas` | 4 | 4 | no |
| `cafc` | 3 | 3 | no |
| `ca7` | 2 | 2 | no |
| `gamd` | 2 | 2 | no |
| `gand` | 2 | 2 | no |
| `gasd` | 2 | 2 | no |
| `msnd` | 2 | 2 | no |
| `mssd` | 2 | 2 | no |
| `sdd` | 2 | 2 | no |
| `itc` | 1 | 1 | no |
| `ttab` | 1 | 1 | no |

2 of 20 packages pass strict after these changes (CIT and CAVC). CIT and CAVC now have zero strict errors. FRAP changed from 10 to 8, PTAB from 9 to 2, SCOTUS from 5 to 4, CoFC from 6 to 4, and FRCP from 4 to 2; other package counts are unchanged. PTAB’s source-version proof is resolved; its remaining errors are the two institution clocks. Several baseline errors repeated the same period across edges.

## Kind A: extension availability

One row per distinct `within` period observed in the baseline diagnostics. A cited provision and vendored file are shown as the governing text reference. “Not typed” means the period remains a strict hole; the source question still needs resolution. `lg-pb1fe.36` and `lg-pb1fe.30` ownership is preserved as directed.

| Package | Period | Governing cite / vendored text reference | Action or reason |
|---|---|---|---|
| `cit` | `complaint-after-summons-period` (`lit/cit/procedure.lit:37`) | `USCIT R. 3(a)(2)` — `sources/cit/cit-rules-and-forms.txt` | Typed USCIT R. 6(b)(1): court may extend for good cause before expiry; on a motion after expiry, excusable neglect. USCIT R. 4(l) separately states good cause for service. |
| `cit` | `service-period` (`lit/cit/procedure.lit:52`) | `USCIT R. 4(l)` — `sources/cit/cit-rules-and-forms.txt` | Typed USCIT R. 6(b)(1): court may extend for good cause before expiry; on a motion after expiry, excusable neglect. USCIT R. 4(l) separately states good cause for service. |
| `cit` | `government-answer-period` (`lit/cit/procedure.lit:64`) | `USCIT R. 12(a)(1)(A)` — `sources/cit/cit-rules-and-forms.txt` | Typed USCIT R. 6(b)(1): court may extend for good cause before expiry; on a motion after expiry, excusable neglect. USCIT R. 4(l) separately states good cause for service. |
| `cit` | `answer-period` (`lit/cit/procedure.lit:68`) | `USCIT R. 12(a)(1)(B)(i)` — `sources/cit/cit-rules-and-forms.txt` | Typed USCIT R. 6(b)(1): court may extend for good cause before expiry; on a motion after expiry, excusable neglect. USCIT R. 4(l) separately states good cause for service. |
| `cit` | `answer-after-motion-period` (`lit/cit/procedure.lit:87`) | `USCIT R. 12(a)(2)(A)` — `sources/cit/cit-rules-and-forms.txt` | Typed USCIT R. 6(b)(1): court may extend for good cause before expiry; on a motion after expiry, excusable neglect. USCIT R. 4(l) separately states good cause for service. |
| `cit` | `summary-judgment-period` (`lit/cit/procedure.lit:103`) | `USCIT R. 56(b)` — `sources/cit/cit-rules-and-forms.txt` | Typed USCIT R. 6(b)(1): court may extend for good cause before expiry; on a motion after expiry, excusable neglect. USCIT R. 4(l) separately states good cause for service. |
| `cit` | `agency-record-motion-period` (`lit/cit/procedure.lit:114`) | `USCIT R. 56.2(d)` — `sources/cit/cit-rules-and-forms.txt` | Typed USCIT R. 6(b)(1): court may extend for good cause before expiry; on a motion after expiry, excusable neglect. USCIT R. 4(l) separately states good cause for service. |
| `cit` | `joint-status-report-period` (`lit/cit/procedure.lit:120`) | `USCIT R. 56.2(a)(5)(A)` — `sources/cit/cit-rules-and-forms.txt` | Typed USCIT R. 6(b)(1): court may extend for good cause before expiry; on a motion after expiry, excusable neglect. USCIT R. 4(l) separately states good cause for service. |
| `cit` | `agency-record-period` (`lit/cit/procedure.lit:124`) | `USCIT R. 73.2(a)` — `sources/cit/cit-rules-and-forms.txt` | Typed USCIT R. 6(b)(1): court may extend for good cause before expiry; on a motion after expiry, excusable neglect. USCIT R. 4(l) separately states good cause for service. |
| `cit` | `remand-opposition-period` (`lit/cit/procedure.lit:147`) | `USCIT R. 56.2(h)(2)` — `sources/cit/cit-rules-and-forms.txt` | Typed USCIT R. 6(b)(1): court may extend for good cause before expiry; on a motion after expiry, excusable neglect. USCIT R. 4(l) separately states good cause for service. |
| `mpep` | `provisional-pendency` (`lit/mpep/01-pre-filing.lit:36`) | `35 U.S.C. 111(b)` — `sources/usc-cfr-prosecution-fragments.txt` | Not typed: exact extension authority/prohibition remains undetermined; no inference was added. |
| `mpep` | `extension-month` (`lit/mpep/02-filing-to-allowance.lit:151`) | `37 CFR 1.136(a)` — `sources/cfr-37-part1.txt` | Owned by lg-pb1fe.36; do not redo the 37 C.F.R. §§ 1.136(a), 41.37(e) month-bound forms. |
| `mpep` | `appeal-brief` (`lit/mpep/02-filing-to-allowance.lit:418`) | `37 CFR 41.37` — `sources/cfr-37-part1.txt` | Owned by lg-pb1fe.36; do not redo the 37 C.F.R. §§ 1.136(a), 41.37(e) month-bound forms. |
| `mpep` | `appeal-forwarding-fee` (`lit/mpep/02-filing-to-allowance.lit:472`) | `37 CFR (2025) 41.20(b)(4)` — `sources/cfr-37-fees-2025-07-01.txt` | Not typed: exact extension authority/prohibition remains undetermined; no inference was added. |
| `mpep` | `broadening-reissue` (`lit/mpep/02-filing-to-allowance.lit:762`) | `35 U.S.C. § 251(d)` — `sources/usc-cfr-prosecution-fragments.txt` | Not typed: exact extension authority/prohibition remains undetermined; no inference was added. |
| `mpep` | `ipr-time-bar` (`lit/mpep/03-post-grant.lit:47`) | `35 U.S.C. 311–314` — `sources/usc-cfr-prosecution-fragments.txt` | Not typed: exact extension authority/prohibition remains undetermined; no inference was added. |
| `mpep` | `pgr-window` (`lit/mpep/03-post-grant.lit:56`) | `35 U.S.C. 321–324` — `sources/usc-cfr-prosecution-fragments.txt` | Not typed: exact extension authority/prohibition remains undetermined; no inference was added. |
| `caaf` | `petition-period` (`lit/caaf/procedure.lit:26`) | `CAAF R. 18(c)` — `sources/caaf/caaf-rules.txt` | Not typed: exact extension authority/prohibition remains undetermined; no inference was added. |
| `caaf` | `certificate-period` (`lit/caaf/procedure.lit:32`) | `CAAF R. 18(b)` — `sources/caaf/caaf-rules.txt` | Not typed: exact extension authority/prohibition remains undetermined; no inference was added. |
| `caaf` | `supplement-period` (`lit/caaf/procedure.lit:46`) | `CAAF R. 21(b)` — `sources/caaf/caaf-rules.txt` | Not typed: exact extension authority/prohibition remains undetermined; no inference was added. |
| `caaf` | `supplement-answer-period` (`lit/caaf/procedure.lit:57`) | `CAAF R. 21(c)(2)` — `sources/caaf/caaf-rules.txt` | Not typed: exact extension authority/prohibition remains undetermined; no inference was added. |
| `caaf` | `certified-brief-period` (`lit/caaf/procedure.lit:83`) | `CAAF R. 40(a)` — `sources/caaf/caaf-rules.txt` | Not typed: exact extension authority/prohibition remains undetermined; no inference was added. |
| `caaf` | `mandatory-brief-period` (`lit/caaf/procedure.lit:94`) | `CAAF R. 40(a)` — `sources/caaf/caaf-rules.txt` | Not typed: exact extension authority/prohibition remains undetermined; no inference was added. |
| `caaf` | `mandate-period` (`lit/caaf/procedure.lit:111`) | `CAAF R. 43A(a)` — `sources/caaf/caaf-rules.txt` | Not typed: exact extension authority/prohibition remains undetermined; no inference was added. |
| `caaf` | `reconsideration-period` (`lit/caaf/procedure.lit:119`) | `CAAF R. 31(a)` — `sources/caaf/caaf-rules.txt` | Not typed: exact extension authority/prohibition remains undetermined; no inference was added. |
| `cavc` | `untimeliness-motion-period` (`lit/cavc/procedure.lit:42`) | `CAVC R. 4(a)(3)(B)` — `sources/cavc/cavc-rules-of-practice-2026-10-01.txt` | Typed CAVC R. 26(b): court, GoodCause up to 45 total days and ExtraordinaryCircumstances beyond 45 days, before or after expiry. |
| `cavc` | `record-before-agency-period` (`lit/cavc/procedure.lit:60`) | `CAVC R. 10(a)` — `sources/cavc/cavc-rules-of-practice-2026-10-01.txt` | Typed CAVC R. 26(b): court, GoodCause up to 45 total days and ExtraordinaryCircumstances beyond 45 days, before or after expiry. |
| `cavc` | `appellant-brief-period` (`lit/cavc/procedure.lit:71`) | `CAVC R. 31(a)(1)` — `sources/cavc/cavc-rules-of-practice-2026-10-01.txt` | Typed CAVC R. 26(b): court, GoodCause up to 45 total days and ExtraordinaryCircumstances beyond 45 days, before or after expiry. |
| `cavc` | `secretary-brief-period` (`lit/cavc/procedure.lit:85`) | `CAVC R. 31(a)(2)` — `sources/cavc/cavc-rules-of-practice-2026-10-01.txt` | Typed CAVC R. 26(b): court, GoodCause up to 45 total days and ExtraordinaryCircumstances beyond 45 days, before or after expiry. |
| `cavc` | `uncontested-mandate-period` (`lit/cavc/procedure.lit:97`) | `CAVC R. 41(c)` — `sources/cavc/cavc-rules-of-practice-2026-10-01.txt` | Typed CAVC R. 26(b): court, GoodCause up to 45 total days and ExtraordinaryCircumstances beyond 45 days, before or after expiry. |
| `cavc` | `reconsideration-period` (`lit/cavc/procedure.lit:121`) | `CAVC R. 35(d)` — `sources/cavc/cavc-rules-of-practice-2026-10-01.txt` | Typed CAVC R. 26(b): court, GoodCause up to 45 total days and ExtraordinaryCircumstances beyond 45 days, before or after expiry. |
| `cavc` | `federal-circuit-appeal-period` (`lit/cavc/procedure.lit:144`) | `CAVC R. 36(a)` — `sources/cavc/cavc-rules-of-practice-2026-10-01.txt` | Typed `extend { never cite "CAVC R. 26(b)" }`; Rule 26(b) bars extending a Notice of Appeal except as Rule 4 provides; Rule 4 does not govern this Rule 36(a) appeal. |
| `cavc` | `mandate-period` (`lit/cavc/procedure.lit:148`) | `CAVC R. 41(b)` — `sources/cavc/cavc-rules-of-practice-2026-10-01.txt` | Typed CAVC R. 26(b): court, GoodCause up to 45 total days and ExtraordinaryCircumstances beyond 45 days, before or after expiry. |
| `cavc` | `eaja-response-period` (`lit/cavc/procedure.lit:169`) | `CAVC R. 39(a)(1)` — `sources/cavc/cavc-rules-of-practice-2026-10-01.txt` | Typed CAVC R. 26(b): court, GoodCause up to 45 total days and ExtraordinaryCircumstances beyond 45 days, before or after expiry. |
| `ptab` | `pgr-filing-period` (`lit/ptab/01-pre-petition-to-institution.lit:48`) | `35 U.S.C. § 321(c)` — `sources/usc-additional.txt` | Typed `extend { never ... }` on the period; pack diff limited to extension fields and removal of the hole. |
| `ptab` | `ipr-time-bar` (`lit/ptab/01-pre-petition-to-institution.lit:71`) | `35 U.S.C. § 315(b)` — `sources/usc-additional.txt` | Typed `extend { never ... }` on the period; pack diff limited to extension fields and removal of the hole. |
| `ptab` | `institution-period` (`lit/ptab/01-pre-petition-to-institution.lit:240`) | `35 U.S.C. § 314(b)` — `sources/usc-additional.txt` | Not typed: exact extension authority/prohibition remains undetermined; no inference was added. |
| `ptab` | `institution-period-on-waiver` (`lit/ptab/01-pre-petition-to-institution.lit:256`) | `35 U.S.C. 314(b)(2), 37 CFR 42.107(b)` — `sources/usc-additional.txt` | Not typed: exact extension authority/prohibition remains undetermined; no inference was added. |
| `cofc` | `protest-notice-window` (`lit/cofc/02-bid-protest.lit:10`) | `RCFC App. C 2` — `sources/rcfc-appendices.txt` | Not typed: exact extension authority/prohibition remains undetermined; no inference was added. |
| `cofc` | `settlement-discussion-period` (`lit/cofc/05-patent-track.lit:157`) | `PRCFC 16(a)` — `sources/rcfc-appendices.txt` | Typed RCFC 6(b)(1): court extension for good cause before expiry; excusable neglect on a motion after expiry. |
| `cofc` | `settlement-statement-period` (`lit/cofc/05-patent-track.lit:169`) | `PRCFC 17` — `sources/rcfc-appendices.txt` | Typed RCFC 6(b)(1): court extension for good cause before expiry; excusable neglect on a motion after expiry. |
| `cofc` | `post-judgment-motion-period` (`lit/cofc/06-trial-post-trial.lit:38`) | `RCFC 59(a)-(b), (e)` — `sources/rcfc.txt` | Not typed: exact extension authority/prohibition remains undetermined; no inference was added. |
| `cofc` | `costs-period` (`lit/cofc/06-trial-post-trial.lit:79`) | `RCFC 54(d)` — `sources/rcfc.txt` | Not typed: exact extension authority/prohibition remains undetermined; no inference was added. |
| `scotus` | `opposition-period` (`lit/scotus/cert.lit:268`) | `Sup. Ct. R. 15.3` — `sources/scotus/scotus-rules-2026-sectioned.txt` | Not typed: exact extension authority/prohibition remains undetermined; no inference was added. |
| `scotus` | `merits-brief-period` (`lit/scotus/cert.lit:310`) | `Sup. Ct. R. 25.1` — `sources/scotus/scotus-rules-2026-sectioned.txt` | Not typed: exact extension authority/prohibition remains undetermined; no inference was added. |
| `scotus` | `respondent-brief-period` (`lit/scotus/cert.lit:319`) | `Sup. Ct. R. 25.2` — `sources/scotus/scotus-rules-2026-sectioned.txt` | Not typed: exact extension authority/prohibition remains undetermined; no inference was added. |
| `scotus` | `rehearing-period` (`lit/scotus/cert.lit:354`) | `Sup. Ct. R. 44.1` — `sources/scotus/scotus-rules-2026-sectioned.txt` | Not typed: exact extension authority/prohibition remains undetermined; no inference was added. |
| `scotus` | `denial-rehearing-period` (`lit/scotus/cert.lit:373`) | `Sup. Ct. R. 44.2` — `sources/scotus/scotus-rules-2026-sectioned.txt` | Typed `extend { never ... }`; Rule 44.2 expressly says the time will not be extended. |
| `frcp` | `removal-period` (`lit/frcp/02-pleading-to-trial.lit:109`) | `28 U.S.C. 1446(a)` — `sources/usc-cfr-prosecution-fragments.txt` | Not typed: exact extension authority/prohibition remains undetermined; no inference was added. |
| `frcp` | `remand-motion-period` (`lit/frcp/02-pleading-to-trial.lit:135`) | `28 U.S.C. 1447(c)` — `sources/usc-cfr-prosecution-fragments.txt` | Not typed: exact extension authority/prohibition remains undetermined; no inference was added. |
| `frcp` | `fee-supplement-period` (`lit/frcp/03-post-trial-motions.lit:40`) | `FRCP 54(d)(2)(C)` — `sources/frcp.txt` | Typed FRCP 6(b)(1): court extension for good cause before expiry; excusable neglect on a motion after expiry. Added the court-order event that sets the revised due date. |
| `habeas` | `state-petition-limitation` (`lit/habeas/proceeding.lit:21`) | `28 U.S.C. § 2244(d)(1)` — `sources/habeas/usc-28-ch153.txt` | Not typed: exact extension authority/prohibition remains undetermined; no inference was added. |
| `habeas` | `federal-motion-limitation` (`lit/habeas/proceeding.lit:27`) | `28 U.S.C. § 2255(f)` — `sources/habeas/usc-28-ch153.txt` | Not typed: exact extension authority/prohibition remains undetermined; no inference was added. |
| `habeas` | `successive-authorization-ruling` (`lit/habeas/proceeding.lit:44`) | `28 U.S.C. § 2244(b)(3)(C)` — `sources/habeas/usc-28-ch153.txt` | Not typed: exact extension authority/prohibition remains undetermined; no inference was added. |
| `habeas` | `proposed-findings-objections` (`lit/habeas/proceeding.lit:69`) | `2254 Rule 8(b)` — `sources/habeas/rules-2254-2255-sectioned.txt` | Not typed: exact extension authority/prohibition remains undetermined; no inference was added. |
| `cafc` | `appearance-period` (`lit/cafc/02-docketing.lit:23`) | `Fed. Cir. R. 47.3(b)(1)` — `sources/fedcir-rules.txt` | Not typed: exact extension authority/prohibition remains undetermined; no inference was added. |
| `gamd` | `reply-period` (`lit/gamd/local-rules.lit:120`) | `M.D. Ga. L.R. 7.3` — `sources/md-ga/mdga-local-rules-sectioned.txt` | Not typed: exact extension authority/prohibition remains undetermined; no inference was added. |
| `gand` | `reply-period` (`lit/gand/local-rules.lit:237`) | `N.D. Ga. LR 7.1(B)` — `sources/nd-ga/ndga-civil-local-rules-sectioned.txt` | Not typed: exact extension authority/prohibition remains undetermined; no inference was added. |
| `gasd` | `reply-period` (`lit/gasd/local-rules.lit:143`) | `S.D. Ga. LR Civ. 7(b)` — `sources/sd-ga/sdga-local-rules-sectioned.txt` | Not typed: exact extension authority/prohibition remains undetermined; no inference was added. |
| `msnd` | `response-period` (`lit/msnd/local-rules.lit:322`) | `N.D. Miss. L.U. Civ. R. 7(b)` — `sources/msnd/local-uniform-civil-rules-sectioned.txt` | Not typed: exact extension authority/prohibition remains undetermined; no inference was added. |
| `msnd` | `rebuttal-period` (`lit/msnd/local-rules.lit:334`) | `N.D. Miss. L.U. Civ. R. 7(b)` — `sources/msnd/local-uniform-civil-rules-sectioned.txt` | Not typed: exact extension authority/prohibition remains undetermined; no inference was added. |
| `mssd` | `response-period` (`lit/mssd/local-rules.lit:322`) | `S.D. Miss. L.U. Civ. R. 7(b)` — `sources/msnd/local-uniform-civil-rules-sectioned.txt` | Not typed: exact extension authority/prohibition remains undetermined; no inference was added. |
| `mssd` | `rebuttal-period` (`lit/mssd/local-rules.lit:334`) | `S.D. Miss. L.U. Civ. R. 7(b)` — `sources/msnd/local-uniform-civil-rules-sectioned.txt` | Not typed: exact extension authority/prohibition remains undetermined; no inference was added. |
| `sdd` | `response-brief-period` (`lit/sdd/local-rules.lit:213`) | `D.S.D. Civ. LR 7.1(B)` — `sources/sdd/civil-local-rules-sectioned.txt` | Not typed: exact extension authority/prohibition remains undetermined; no inference was added. |
| `sdd` | `reply-brief-period` (`lit/sdd/local-rules.lit:222`) | `D.S.D. Civ. LR 7.1(B)` — `sources/sdd/civil-local-rules-sectioned.txt` | Not typed: exact extension authority/prohibition remains undetermined; no inference was added. |
## Kind B: source-version proof

One row per distinct package, rule/section and trigger date. A version proof is changed only where the vendored historical text directly supports the binding.

| Package | Item / date | Pre-epic status | Vendored text and finding | Action or remaining evidence |
|---|---|---|---|
| `frap` | `FRAP 26 / 2009-12-01` | Unknown (not checked against 038a190cb) | `frap-rules` — `sources/frap.txt` | Unresolved: source history records later amendments; no historical FRAP 26 text is vendored for the 2009/2018 version dates. |
| `frap` | `FRAP 11 / 2009-12-01` | Unknown (not checked against 038a190cb) | `frap-rules` — `sources/frap.txt` | Fixed: cite rebound to the vendored 2009 edition in `sources/usc-28-app-frap-editions.txt`; diagnostic gone. |
| `frap` | `FRAP 31 / 2009-12-01` | Unknown (not checked against 038a190cb) | `frap-rules` — `sources/frap.txt` | Unresolved: `sources/usc-28-app-frap-editions.txt` does not include Rule 31 text, and `sources/rule-amendment-history.txt` records an intervening 2018 amendment; no 2009 Rule 31 text is vendored. |
| `frap` | `FRAP 26 / 2018-12-01` | Unknown (not checked against 038a190cb) | `frap-rules` — `sources/frap.txt` | Unresolved: source history records later amendments; no historical FRAP 26 text is vendored for the 2009/2018 version dates. |
| `frap` | `FRAP 41 / 2009-12-01` | Unknown (not checked against 038a190cb) | `frap-rules` — `sources/frap.txt` | Fixed: cite rebound to the vendored 2009 edition in `sources/usc-28-app-frap-editions.txt`; diagnostic gone. |
| `ptab` | `37 C.F.R. § 42.107 / 2012-09-16 (2012 version)` | Unknown (not checked against 038a190cb) | `cfr-42` — `sources/cfr-37-part42-2012.txt` | Fixed: the delegation cite now uses the vendored 2012 § 42.5(c)(1) text; the current-version diagnostic is gone. |
| `cofc` | `RCFC 6 / 2010-01-11` | Unknown (not checked against 038a190cb) | `rcfc-rules` — `sources/rcfc.txt` | Unresolved: history lists a 2010 version and later amendments, but no 2010 RCFC 6 text binding is available in the package. |
| `cafc` | `Fed. Cir. R. 26 / 2009-12-01` | Unknown (not checked against 038a190cb) | `fedcir-rules` — `sources/fedcir-rules.txt` | Unresolved: `sources/rule-amendment-history.txt` records no version on or before these dates; no historical Federal Circuit Rule 26 text is vendored. |
| `cafc` | `Fed. Cir. R. 26 / 2018-12-01` | Unknown (not checked against 038a190cb) | `fedcir-rules` — `sources/fedcir-rules.txt` | Unresolved: `sources/rule-amendment-history.txt` records no version on or before these dates; no historical Federal Circuit Rule 26 text is vendored. |
| `ca7` | `FRAP 26 / 2009-12-01` | Unknown (not checked against 038a190cb) | `frap-rules` — `sources/frap.txt` | Unresolved: source history records later amendments; no historical FRAP 26 text is vendored for the 2009/2018 version dates. |
| `ca7` | `FRAP 26 / 2018-12-01` | Unknown (not checked against 038a190cb) | `frap-rules` — `sources/frap.txt` | Unresolved: source history records later amendments; no historical FRAP 26 text is vendored for the 2009/2018 version dates. |
| `itc` | `19 C.F.R. § 210.6 / 2025-02-03` | Unknown (not checked against 038a190cb) | `cfr-210` — `sources/cfr-19-part210-2025.txt` | Unresolved: `sources/ecfr-versions-title-19.txt` does not pin a § 210.6 amendment history; no historical text binding proves the 2025-02-03 version. The inline ITC period remains owned by lg-pb1fe.30 and was not changed. |
| `ttab` | `Rule 6 / 2017-01-14` | Unknown (not checked against 038a190cb) | `frcp-rules` — `sources/frcp.txt` | Unresolved: the FRCP history records an amendment effective 2023-12-01; no FRCP 6 text for the full 2017-to-present period is bound. |
| `ttab` | `37 C.F.R. § 2.116 / 2017-01-14` | Unknown (not checked against 038a190cb) | `cfr2` — `sources/cfr-37-part2.txt` | Unresolved: `sources/cfr-37-part2-answer-time-history.txt` carries the 2017-01-14 version and unchanged-text note, but its inline version metadata is not in the `## versions <heading>` format recognized by cite checking; no binding change was made. |

## Lowered pack diffs

`scripts/lowered-json-diff.py` compared baseline pack JSON built from launch commit `8657133159e84104739176e62acee4545fb3af62` with the final packs. Every changed leaf is an extension field (`extendable`, `extensionAuthority`, `extensions/*`) or removal of `lowered.holes/deadline/extendable`; no duration, counting rule, probability, edge, or test expectation changed.

- PTAB (`ptab-patent-trial-appeal-board`): 30 changed leaves across six edge instances.
- SCOTUS (`supreme-court-of-the-united-states`): 5 changed leaves on the denial-rehearing edge.
- FRAP (`frap-appellate-procedure`): 3 changed `deadline/extensionAuthority` citations for Rules 11 and 41, rebound to the vendored 2009 edition.
- CIT (`cit`): 204 changed leaves across ten Rule 6 periods.
- CAVC (`cavc`): 197 changed leaves across the Rule 26(b) periods and Rule 36(a) lock.
- FRCP (`frcp-civil-procedure`): 34 changed leaves for the court-set fee-supplement period.
- CoFC (`cofc`): 34 changed leaves for the two patent-track periods.
- Every other requested package: no lowered pack change.

## Verification and worktree

- `cargo build --release`: exit 0 (one build at launch).
- `just lit-sync-check`: exit 0; Lit.lock files and the shared clock library are current, generated files remain uncommitted.
- `target/release/litigate check lit cases --format json`: exit 0; `ok=true`, 0 errors, 3,496 diagnostics (warnings included).
- Final strict sweep over the 20 packages: 2 pass (CIT, CAVC); 18 retain L5002 errors as shown above.
- Targeted tests: CIT 69 passed / 0 failed; CAVC 76 / 0; FRAP 42 / 0; FRCP 49 / 0; CoFC 46 / 0; PTAB 35 / 0; SCOTUS 18 / 0. Every command exited 0.
- Changed paths are in `lit/`, package `Lit.lock` files, and this report. Generated pack JSON remains uncommitted.

## Decisions

- DECIDED: unresolved extension availability stays a hole unless the cited vendored text establishes an exact grant/delegation or prohibition; no inferred GoodCause grant was added.
- DECIDED: history-backed citations use the already-vendored historic text where it covers the cited rule (FRAP 11/41 and PTAB 2012 § 42.5(c)(1)).
- Known ownership: MPEP §§ 1.136(a)/41.37(e) month-bound periods are listed for lg-pb1fe.36; frcrimp/ITC inline deadlines remain with lg-pb1fe.30.
