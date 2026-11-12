# MPEP extension grants (lg-pb1fe.36)

DECIDED: leave `provisional-pendency` extension undeclared — § 1.78(b)'s
unintentional-delay standard is not expressible by the closed cause type, so
typing a grant or a lock would misstate the rule.

## Source and typed periods

| Period | Text relied on | Grant |
|---|---|---|
| `provisional-pendency` | `sources/cfr-37-part1.txt:2428` (§ 1.78(b): restoration within two months after the 12-month period if delay was unintentional) | Undeclared: the closed extension-cause vocabulary cannot express unintentional delay. |
| `missing-parts-reply` | `sources/cfr-37-part1.txt:3395` (§ 1.136(a): five months after the period set for reply) and `sources/mpep.txt:11130-11136` (§ 710.02(d)) | Applicant, petition motion, trigger + 7 months (two-month set period plus five months), after expiry. |
| `election-reply` | Same § 1.136(a)(1) and § 133 text | Applicant, petition motion, trigger + 6 months, after expiry, no discretion. |
| `ssp-reply` | Same § 1.136(a)(1) and § 133 text | Applicant, petition motion, trigger + 6 months, after expiry, no discretion. |
| `final-reply` | Same § 1.136(a)(1) and § 133 text | Applicant, petition motion, trigger + 6 months, after expiry, no discretion. |
| `extension-month` | § 1.136(a)(1), § 133, and `sources/mpep.txt` MPEP 710.01(a): the extended date corresponds to the Office action date | Applicant, petition motion, trigger + 1 month per increment from `ReplyPeriodEnded()`. The SSP path permits three such increments after its initial three-month period, reaching the six-month statutory ceiling. |
| `appeal-brief` | `sources/cfr-37-part41.txt:321` (§ 41.37(e), applies § 1.136) and `sources/cfr-37-part1.txt:3395` (§ 1.136(a)(1), five months after the period set for reply); `lit/mpep/deadlines.lit` documents § 133 does not govern this period | Applicant, petition motion, trigger + 7 months (two-month original period plus five months), after expiry, no discretion. |
| `appeal-forwarding-fee` | `sources/cfr-37-part41.txt:379` (§ 41.45(c), § 1.136(a) does not apply; § 1.136(b) does) and `sources/cfr-37-part1.txt:3411` (§ 1.136(b): sufficient cause, reasonable time specified, request on or before due) | Office, `GoodCause` (sufficient cause), `ExtensionForCauseRequested()` within `prescribed_due`; discretionary length set by the Office. |
| `ipr-time-bar` | `sources/usc-additional.txt:409` (35 U.S.C. § 315(b)) | Locked against extensions. The statute has a joinder exception; it is not an extension of the one-year period. |
| `pgr-window` | `sources/usc-additional.txt:504` (35 U.S.C. § 321(c): petition may only be filed within nine months) | Locked against extensions. |
| `broadening-reissue` | `sources/usc-additional.txt:319` (35 U.S.C. § 251(d): broadening reissue must be applied for within two years) | Locked against extensions. |

The bound for § 1.136(a) uses its express ceiling: “applicant may extend the
time period for reply up to the earlier of the expiration of any maximum
period set by statute or five months after the time period set for reply”
(`sources/cfr-37-part1.txt:3395`). For the four three-month shortened
statutory reply periods, that earlier statutory maximum is six months after
the action that started the period (35 U.S.C. § 133, `sources/usc-cfr-prosecution-fragments.txt:650-651`), hence trigger + six months. For
`appeal-brief`, § 41.37(e) makes § 1.136 applicable, so the two-month period
can extend five months from its trigger, hence trigger + seven months.

MPEP 710.02(d) states: “The 2-month time period for reply to A Notice to File
Missing Parts of an Application is not identified on the Notice as a statutory
period subject to 35 U.S.C. 133.” Accordingly, `missing-parts-reply` follows
the five-month § 1.136(a) allowance and does not use § 133's six-month cap.
MPEP § 710.02(b) identifies a restriction/election period as a “shortened
statutory period” and a three-month reply to an Office action on the merits;
§ 710.02(d) says of “extensions of the statutory period under 35 U.S.C. 133”:
“provided the extension does not go beyond the 6-month statutory period from
the date of the Office action.” Thus `election-reply` is statutory because
§ 710.02(b)(A) expressly classifies the restriction/election period that way;
`ssp-reply` and `final-reply` are statutory because they are shortened reply
periods for Office actions (the SSP is on the merits, and final reply follows
a final Office action). All three are capped at six months from the action.

`ReplyPeriodEnded()` feeds `extension-month` from the then-current reply due
date in the SSP path in `lit/mpep/02-filing-to-allowance.lit`; it does not feed
`missing-parts-reply`, `election-reply`, or `final-reply` in this package. Each
`extension-month` is one month. The three available increments after the
initial three-month SSP reach the six-month § 133 ceiling, so this SSP path
does not understate that ceiling. The separately typed direct reply grants
use their own six-month-from-action cap.

The four existing `delegable ... cite "37 CFR 1.136(a)"` periods
(`missing-parts-reply`, `election-reply`, `ssp-reply`, `final-reply`) each
receive a separate applicant extension grant as well as the `CaseOrder`
delegation that sets their original period. Section 1.136(a)(1) expressly
reaches a required nonstatutory or shortened-statutory reply period. The
typed grants use a petition motion and an applicant actor; there is no fee
field in this language, so the motion's legal source includes the required
§ 1.17(a) fee condition.

## Inline examples and replay

Inline tests in `lit/mpep/deadlines.lit` cover an accepted grant and a
petition after the window for each of the seven grant families. Those
“petition after the window” tests do not exercise a refusal: no extension
event is given, so they only show the unextended date. Refusal is covered in
`crates/litigate-sema/tests/cov_case_obligations.rs`. The MPEP month-end cases verify
that the bound is counted from its trigger. The language replay test in
`crates/litigate-sema/tests/cov_case_obligations.rs` covers the bound refusal:
a proposal one day beyond a January 31 trigger's May 31 ceiling is rejected;
it also checks a November 30 trigger, whose four-month nominal end is March
30 before the weekend roll. The existing clock test at
`crates/litigate-engine/tests/clock_months.rs` checks November 30 plus four
months under the MPEP counting rule.

## MPEP edges from lg-pb1fe.30

The following five MPEP edges use the named deadline grants above and lower
`extendable: true` with their governing authority:

| Edge | Period | Authority |
|---|---|---|
| `edge-applicant-response-window-extension-taken` | `extension-month` | 37 C.F.R. § 1.136(a); 35 U.S.C. § 133 |
| `edge-extension-taken-amendment-filed` | `extension-month` | 37 C.F.R. § 1.136(a); 35 U.S.C. § 133 |
| `edge-extension-taken-extension-taken` | `extension-month` | 37 C.F.R. § 1.136(a); 35 U.S.C. § 133 |
| `edge-notice-of-appeal-appeal-brief-filed` | `appeal-brief` | 37 C.F.R. §§ 1.136(a), 41.37(e) |
| `edge-notice-of-appeal-appeal-dismissed` | `appeal-brief` | 37 C.F.R. §§ 1.136(a), 41.37(e) |

## Strict check and remaining holes

Strict check on the final tree (`target/release/litigate check --profile
strict lit/mpep`): exit 2, 2 total errors. Both are L5002 hole diagnostics
from the single undeclared `provisional-pendency` period, referenced at
`lit/mpep/01-pre-filing.lit:36` and `lit/mpep/01-pre-filing.lit:74`; there
are 0 text-in-force proof errors. The check reported no other diagnostics.

## Open for David

- MPEP 710.02(d) permits `missing-parts-reply` to receive additional time
  under § 1.136(b), when appropriate; the same section says an appeal brief
  may be further extended in an exceptional situation under § 1.136(b). The
  typed `appeal-forwarding-fee` form is specific to § 41.45(c), so neither
  additional route is modeled without an applicable form that avoids adding
  a fact.
- Section 1.136(b)'s distinct sufficient-cause route is typed on
  `appeal-forwarding-fee` because § 41.45(c) expressly invokes it. The request
  must be filed no later than the due date, and the Office actor has
  `GoodCause` discretion. The language has no fee field; the § 1.17(g) fee
  therefore remains part of the cited rule text rather than an independently
  checked matter event.
- Section 1.136(a)(2) permits a written standing authorization or fee payment
  to operate as a constructive petition. The authored motion is the explicit
  petition form; the package does not model constructive petitions or fees
  as independent procedural events.
- `provisional-pendency`: § 1.78(b) permits restoration for unintentional
  delay, but the closed `CauseStandard` cannot express that standard. Whether
  to add an `Unintentional` member is an open language decision.
- `ipr-time-bar`, `pgr-window`, and `broadening-reissue` are locks resting on
  prohibitive statutory wording: § 315(b) says institution “may not be
  instituted,” § 321(c) says a petition “may only be filed” within nine
  months, and § 251(d) says a broadening reissue must be applied for “within
  two years.”

## Gate results

- `target/release/litigate test lit/mpep`: exit 0, 56 passed, 0 failed.
- `target/release/litigate check lit cases`: exit 0, 0 errors.
- `target/release/litigate fmt --check lit/mpep`: exit 0.
- `just lit-sync`: exit 0; regenerated the MPEP `Lit.lock` and built packs and scenarios.
- `just lit-sync-check`: final run exit 0; Lit.lock files and the shared clock
  library are current, and no generated files are committed.
