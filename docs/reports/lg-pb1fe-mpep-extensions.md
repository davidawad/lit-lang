# MPEP extension grants (lg-pb1fe.36)

## Source and typed periods

| Period | Text relied on | Grant |
|---|---|---|
| `provisional-pendency` | `sources/cfr-37-part1.txt:2410` (§ 1.78(a)(1), twelve months for a provisional benefit) and 35 U.S.C. § 119(e)(1) | Locked against extensions. Restoration under § 1.78(b) is a separate petition remedy, not an extension of this filing deadline. |
| `missing-parts-reply` | `sources/cfr-37-part1.txt:3395` (§ 1.136(a)(1): nonstatutory or shortened statutory reply period; extension by petition and fee) and `sources/usc-cfr-prosecution-fragments.txt:650-651` (§ 133: six months after an action) | Applicant, petition motion, trigger + 6 months, after expiry, no discretion. |
| `election-reply` | Same § 1.136(a)(1) and § 133 text | Applicant, petition motion, trigger + 6 months, after expiry, no discretion. |
| `ssp-reply` | Same § 1.136(a)(1) and § 133 text | Applicant, petition motion, trigger + 6 months, after expiry, no discretion. |
| `final-reply` | Same § 1.136(a)(1) and § 133 text | Applicant, petition motion, trigger + 6 months, after expiry, no discretion. |
| `extension-month` | § 1.136(a)(1), § 133, and `sources/mpep.txt` MPEP 710.01(a): the extended date corresponds to the Office action date | Applicant, petition motion, trigger + 3 months from the current one-month increment's trigger (the initial reply due date), after expiry, no discretion. This preserves the action's six-month ceiling for the SSP extension path. |
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
petition beyond the grant window for each of the seven grant families; each
late petition leaves the period unextended. The MPEP month-end cases verify
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

Strict baseline on `origin/main` (`70b15de0`): 11 L5002 undeclared-extension
errors. Strict check after these edits: 0 L5002 errors, 0 other errors, and
0 warnings. Each period with vendored governing extension or no-extension
text is typed or locked; no strict extension holes remain.

## Open for David

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

## Gate results

- `cargo nextest run -p litigate-sema -p litigate-lower -p litigate-engine -p litigate-verify -p litigate-syntax --no-fail-fast`: 1,792 passed, 1 skipped, 0 failed.
- `cargo nextest run -p litigate-cli --no-fail-fast -E 'test(/extension|extend|mpep|given_by|inline_tests|grant/)'`: 34 run, 17 passed, 17 failed. All 17 failures are the known `lg-pb1fe.26` main-branch failures identified in the PM ruling; none are new MPEP failures.
- `cargo clippy --workspace --all-targets -- -D warnings`: passed.
- `just tree-sitter-test`: passed, 88 syntax/highlighting cases plus merge test; grammar did not change.
- `litigate check lit cases`: completed with no errors; it emitted the existing L4016 `as_of` warnings for dates before today.
- `litigate test lit/mpep`: 56 passed, 0 failed (before the seven late-petition cases: 49 passed, 0 failed).
- Strict MPEP check: baseline 11 L5002 errors; after 0 L5002 errors, 0 other errors, 0 warnings.
- `litigate fmt --check lit cases`: exit 2 on the known main-branch L9004 in `lit/ptab/03-trial-to-appeal.lit`; `litigate fmt --check lit/mpep`: passed.
- `just lit-sync`: passed; regenerated the MPEP `Lit.lock` and built packs/scenarios.
- `just lit-sync-check`: passed on the committed tree; all locks and the shared clock library are current, and no generated files are committed.
- `cargo test -p litigate-verify --test refinement_bounds`: 10 passed, 0 failed.
