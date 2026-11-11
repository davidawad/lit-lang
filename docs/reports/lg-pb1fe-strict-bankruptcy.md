# Strict profile: bankruptcy and BAP extension availability

Bead: `lg-pb1fe.34`. Baseline captured with `target/release/litigate check --profile strict lit/<package> --format json` before edits; after checks used the same command and binary. Baseline is 74 L5002 findings across 31 packages. After edits, 38 remain; 4 of 31 packages pass strict (bankr-cas, bankr-or, bankr-ut, bap9). There are no non-L5002 errors in the after results.

## Strict counts by package

| Package | L5002 before | L5002 after | After |
|---|---:|---:|---|
| bankr-ak | 3 | 3 | fails |
| bankr-az | 1 | 1 | fails |
| bankr-cac | 1 | 1 | fails |
| bankr-cae | 1 | 1 | fails |
| bankr-can | 1 | 1 | fails |
| bankr-cas | 1 | 0 | pass |
| bankr-co | 1 | 1 | fails |
| bankr-dc | 3 | 3 | fails |
| bankr-id | 1 | 1 | fails |
| bankr-ma | 1 | 1 | fails |
| bankr-mt | 2 | 2 | fails |
| bankr-nm | 1 | 1 | fails |
| bankr-nv | 1 | 1 | fails |
| bankr-okn | 1 | 1 | fails |
| bankr-or | 1 | 0 | pass |
| bankr-pae | 1 | 1 | fails |
| bankr-pam | 1 | 1 | fails |
| bankr-paw | 1 | 1 | fails |
| bankr-txe | 1 | 1 | fails |
| bankr-txn | 1 | 1 | fails |
| bankr-txs | 1 | 1 | fails |
| bankr-txw | 1 | 1 | fails |
| bankr-ut | 1 | 0 | pass |
| bankr-vi | 1 | 1 | fails |
| bankr-wae | 1 | 1 | fails |
| bankr-wy | 1 | 1 | fails |
| bap1 | 16 | 6 | fails |
| bap10 | 6 | 1 | fails |
| bap6 | 8 | 1 | fails |
| bap8 | 8 | 2 | fails |
| bap9 | 5 | 0 | pass |

## Source and typing decisions

Read FRBP 9006(b)(1) in `sources/frbp.txt` (Rule 9006), which permits the court to extend a period prescribed by the Bankruptcy Rules for cause on a request made before expiry, and permits an after-expiry extension only for excusable neglect. Rule 9006(b)(2) lists no Part VIII deadline among its exceptions; Rule 9006(b)(3) gives Rule 8002 its own specific limits. Accordingly, the shared deadlines in `lit/frbp/appeals.lit` now carry the Rule 9006(b)(1) before-expiry and after-expiry grants with the corresponding standard and motion event. The 28 U.S.C. § 158(c)(1) district-court election periods are statutory; each has a typed `never` lock cited to that statute. This shared change is why BAP9 now passes and the other four BAP packages lose most, but not all, L5002 findings.

Read the local rules governing the three local periods that now have typed grants: Southern District of California LBR 9006-1(d) (`sources/bankr-cas/casb-lbr-2025-12-01-sectioned.txt`), Oregon LBR 9006-1(a) (`sources/bankr-or/orb-lbr-2025-12-01-sectioned.txt`), and Utah LBR 9006-1(a) (`sources/bankr-ut/utb-lbr-2025-12-01-sectioned.txt`). Each is scoped to the affected local deadline. The Southern District of California rule expressly applies to extensions under local rules and incorporates FRBP 9006; Oregon states good cause, appropriate use of prior time, and a request at the earliest practical time; Utah allows a different response period by order for cause shown on motion.

For the remaining local periods, I read the cited provision and searched the same package’s vendored rule text for the provision governing enlargement. Where a provision says “unless ordered otherwise” or permits a court to shorten/extend time without a standard or window, the typed language cannot encode those missing terms without inference. Where no enlargement provision applies, FRBP 9006(a) supplies counting only; FRBP 9006(b)(1) does not itself authorize enlarging local-rule periods. Those periods are unchanged and remain L5002 strict failures. The original period and edge facts were not changed.

The `governing source / section` column gives the vendored file and exact section read for each period. `typed change / reason` names the written grant/lock or states why the period remains unchanged. Repeated periods across different packages are listed separately because each package layers its own rule source.

## Period-by-period record

Rows are deduplicated by package and referenced deadline; occurrence counts show the number of edges reporting its L5002 before and after. The cited section is the authority on the deadline declaration. Federal Part VIII deadlines and statutory election locks are authored in `lit/frbp/appeals.lit`; local deadlines are recorded per package.

| Package | Period | Governing source / section read | Typed change / reason |
|---|---|---|---|
| bankr-ak | `motion-reply-period` | `sources/bankr-ak/akb-lbr-2024-12-01-sectioned.txt` § `AK LBR 9013-1(c)` | Left unchanged: read `AK LBR 9013-1(c)` in the vendored source. It sets the period but does not state a sourced extension standard and request window for this deadline; FRBP 9006(b)(1) does not by its text resolve whether a locally prescribed period falls within its scope. Baseline/after L5002 occurrences: 1/1. |
| bankr-ak | `motion-opposition-period` | `sources/bankr-ak/akb-lbr-2024-12-01-sectioned.txt` § `AK LBR 9013-1(b)` | Left unchanged: read `AK LBR 9013-1(b)` in the vendored source. It sets the period but does not state a sourced extension standard and request window for this deadline; FRBP 9006(b)(1) does not by its text resolve whether a locally prescribed period falls within its scope. Baseline/after L5002 occurrences: 1/1. |
| bankr-ak | `proposed-order-period` | `sources/bankr-ak/akb-lbr-2024-12-01-sectioned.txt` § `AK LBR 9021-1(c)` | Left unchanged: read `AK LBR 9021-1(c)` in the vendored source. It sets the period but does not state a sourced extension standard and request window for this deadline; FRBP 9006(b)(1) does not by its text resolve whether a locally prescribed period falls within its scope. Baseline/after L5002 occurrences: 1/1. |
| bankr-az | `motion-response-period` | `sources/bankr-az/azb-lbr-2026-06-01-sectioned.txt` § `Bankr. D. Ariz. LR 9013-1(c)` | Left unchanged: read `Bankr. D. Ariz. LR 9013-1(c)` in the vendored source. It sets the period but does not state a sourced extension standard and request window for this deadline; FRBP 9006(b)(1) does not by its text resolve whether a locally prescribed period falls within its scope. Baseline/after L5002 occurrences: 1/1. |
| bankr-cac | `opportunity-response-period` | `sources/bankr-cac/cacb-lbr-2025-05-08-sectioned.txt` § `Bankr. C.D. Cal. LBR 9013-1(o)` | Left unchanged: read `Bankr. C.D. Cal. LBR 9013-1(o)` in the vendored source. It sets the period but does not state a sourced extension standard and request window for this deadline; FRBP 9006(b)(1) does not by its text resolve whether a locally prescribed period falls within its scope. Baseline/after L5002 occurrences: 1/1. |
| bankr-cae | `opportunity-objection-period` | `sources/bankr-cae/caeb-lbr-2026-04-sectioned.txt` § `Bankr. E.D. Cal. LBR 9014-1(k)` | Left unchanged: read `Bankr. E.D. Cal. LBR 9014-1(k)` in the vendored source. It sets the period but does not state a sourced extension standard and request window for this deadline; FRBP 9006(b)(1) does not by its text resolve whether a locally prescribed period falls within its scope. Baseline/after L5002 occurrences: 1/1. |
| bankr-can | `opportunity-objection-period` | `sources/bankr-can/canb-lbr-2026-05-04-sectioned.txt` § `Bankr. N.D. Cal. B.L.R. 9014-1(b)` | Left unchanged: read `Bankr. N.D. Cal. B.L.R. 9014-1(b)` in the vendored source. It sets the period but does not state a sourced extension standard and request window for this deadline; FRBP 9006(b)(1) does not by its text resolve whether a locally prescribed period falls within its scope. Baseline/after L5002 occurrences: 1/1. |
| bankr-cas | `motion-opposition-period` | `sources/bankr-cas/casb-lbr-2025-12-01-sectioned.txt` § `Bankr. S.D. Cal. LBR 9013-6(a)` | Left unchanged: read `Bankr. S.D. Cal. LBR 9013-6(a)` in the vendored source. It sets the period but does not state a sourced extension standard and request window for this deadline; FRBP 9006(b)(1) does not by its text resolve whether a locally prescribed period falls within its scope. Baseline/after L5002 occurrences: 1/0. |
| bankr-co | `presumptive-response-period` | `sources/bankr-co/cob-local-rules-2021-12-01-sectioned.txt` § `D. Colo. L.B.R. 9013-1` | Left unchanged: read `D. Colo. L.B.R. 9013-1` in the vendored source. It sets the period but does not state a sourced extension standard and request window for this deadline; FRBP 9006(b)(1) does not by its text resolve whether a locally prescribed period falls within its scope. Baseline/after L5002 occurrences: 1/1. |
| bankr-dc | `motion-reply-period` | `sources/bankr-dc/dc-local-rules-2026-02-01-sectioned.txt` § `Bankr. D.D.C. LBR 9013-1(d)` | Left unchanged: read `Bankr. D.D.C. LBR 9013-1(d)` in the vendored source. It sets the period but does not state a sourced extension standard and request window for this deadline; FRBP 9006(b)(1) does not by its text resolve whether a locally prescribed period falls within its scope. Baseline/after L5002 occurrences: 1/1. |
| bankr-dc | `motion-response-period` | `sources/bankr-dc/dc-local-rules-2026-02-01-sectioned.txt` § `Bankr. D.D.C. LBR 9013-1(d)` | Left unchanged: read `Bankr. D.D.C. LBR 9013-1(d)` in the vendored source. It sets the period but does not state a sourced extension standard and request window for this deadline; FRBP 9006(b)(1) does not by its text resolve whether a locally prescribed period falls within its scope. Baseline/after L5002 occurrences: 1/1. |
| bankr-dc | `proposed-order-period` | `sources/bankr-dc/dc-local-rules-2026-02-01-sectioned.txt` § `Bankr. D.D.C. LBR 9072-1(e)` | Left unchanged: read `Bankr. D.D.C. LBR 9072-1(e)` in the vendored source. It sets the period but does not state a sourced extension standard and request window for this deadline; FRBP 9006(b)(1) does not by its text resolve whether a locally prescribed period falls within its scope. Baseline/after L5002 occurrences: 1/1. |
| bankr-id | `negative-notice-period` | `sources/bankr-id/idb-lbr-rule-packet-2026-10-27-sectioned.txt` § `Bankr. D. Idaho LBR 2002-2(d)` | Left unchanged: read `Bankr. D. Idaho LBR 2002-2(d)` in the vendored source. It sets the period but does not state a sourced extension standard and request window for this deadline; FRBP 9006(b)(1) does not by its text resolve whether a locally prescribed period falls within its scope. Baseline/after L5002 occurrences: 1/1. |
| bankr-ma | `response-period` | `sources/bankr-ma/ma-local-rules-2025-05-01-sectioned.txt` § `MLBR 9013-1(c)` | Left unchanged: read `MLBR 9013-1(c)` in the vendored source. It sets the period but does not state a sourced extension standard and request window for this deadline; FRBP 9006(b)(1) does not by its text resolve whether a locally prescribed period falls within its scope. Baseline/after L5002 occurrences: 1/1. |
| bankr-mt | `no-objection-certificate-period` | `sources/bankr-mt/mtb-lbr-2025-12-01-sectioned.txt` § `Mont. LBR 9013-1(k)` | Left unchanged: read `Mont. LBR 9013-1(k)` in the vendored source. It sets the period but does not state a sourced extension standard and request window for this deadline; FRBP 9006(b)(1) does not by its text resolve whether a locally prescribed period falls within its scope. Baseline/after L5002 occurrences: 1/1. |
| bankr-mt | `motion-response-period` | `sources/bankr-mt/mtb-lbr-2025-12-01-sectioned.txt` § `Mont. LBR 9013-1(h)` | Left unchanged: read `Mont. LBR 9013-1(h)` in the vendored source. It sets the period but does not state a sourced extension standard and request window for this deadline; FRBP 9006(b)(1) does not by its text resolve whether a locally prescribed period falls within its scope. Baseline/after L5002 occurrences: 1/1. |
| bankr-nm | `objection-deadline-period` | `sources/bankr-nm/nmb-lbr-2025-12-01-sectioned.txt` § `NM LBR 9013-1(d)` | Left unchanged: read `NM LBR 9013-1(d)` in the vendored source. It sets the period but does not state a sourced extension standard and request window for this deadline; FRBP 9006(b)(1) does not by its text resolve whether a locally prescribed period falls within its scope. Baseline/after L5002 occurrences: 1/1. |
| bankr-nv | `negative-notice-objection-period` | `sources/bankr-nv/nvb-lbr-2021-03-02-sectioned.txt` § `Bankr. D. Nev. LR 9014.1(c)` | Left unchanged: read `Bankr. D. Nev. LR 9014.1(c)` in the vendored source. It sets the period but does not state a sourced extension standard and request window for this deadline; FRBP 9006(b)(1) does not by its text resolve whether a locally prescribed period falls within its scope. Baseline/after L5002 occurrences: 1/1. |
| bankr-okn | `motion-response-period` | `sources/bankr-okn/oknb-lbr-2025-03-21-sectioned.txt` § `N.D. Okla. LBR 9013-1(B)` | Left unchanged: read `N.D. Okla. LBR 9013-1(B)` in the vendored source. It sets the period but does not state a sourced extension standard and request window for this deadline; FRBP 9006(b)(1) does not by its text resolve whether a locally prescribed period falls within its scope. Baseline/after L5002 occurrences: 1/1. |
| bankr-or | `motion-objection-period` | `sources/bankr-or/orb-lbr-2025-12-01-sectioned.txt` § `Bankr. D. Or. LBR 9013-1(c)` | Left unchanged: read `Bankr. D. Or. LBR 9013-1(c)` in the vendored source. It sets the period but does not state a sourced extension standard and request window for this deadline; FRBP 9006(b)(1) does not by its text resolve whether a locally prescribed period falls within its scope. Baseline/after L5002 occurrences: 1/0. |
| bankr-pae | `motion-response-period` | `sources/bankr-pae/pae-local-rules-2025-12-01-sectioned.txt` § `Bankr. E.D. Pa. L.B.R. 9014-3(h)` | Left unchanged: read `Bankr. E.D. Pa. L.B.R. 9014-3(h)` in the vendored source. It sets the period but does not state a sourced extension standard and request window for this deadline; FRBP 9006(b)(1) does not by its text resolve whether a locally prescribed period falls within its scope. Baseline/after L5002 occurrences: 1/1. |
| bankr-pam | `participation-response-period` | `sources/bankr-pam/pam-local-rules-2025-10-01-sectioned.txt` § `Bankr. M.D. Pa. L.B.R. 9019-3(b)` | Left unchanged: read `Bankr. M.D. Pa. L.B.R. 9019-3(b)` in the vendored source. It sets the period but does not state a sourced extension standard and request window for this deadline; FRBP 9006(b)(1) does not by its text resolve whether a locally prescribed period falls within its scope. Baseline/after L5002 occurrences: 1/1. |
| bankr-paw | `contested-response-period` | `sources/bankr-paw/paw-local-rules-2026-sectioned.txt` § `W.PA.LBR 9014-1` | Left unchanged: read `W.PA.LBR 9014-1` in the vendored source. It sets the period but does not state a sourced extension standard and request window for this deadline; FRBP 9006(b)(1) does not by its text resolve whether a locally prescribed period falls within its scope. Baseline/after L5002 occurrences: 1/1. |
| bankr-txe | `negative-notice-response-period` | `sources/bankr-txe/txe-local-rules-2022-08-22-sectioned.txt` § `Bankr. E.D. Tex. LBR 9007-1(a)` | Left unchanged: read `Bankr. E.D. Tex. LBR 9007-1(a)` in the vendored source. It sets the period but does not state a sourced extension standard and request window for this deadline; FRBP 9006(b)(1) does not by its text resolve whether a locally prescribed period falls within its scope. Baseline/after L5002 occurrences: 1/1. |
| bankr-txn | `negative-notice-response-period` | `sources/bankr-txn/txn-local-rules-2025-12-01-sectioned.txt` § `Bankr. N.D. Tex. L.B.R. 9007-1(c)` | Left unchanged: read `Bankr. N.D. Tex. L.B.R. 9007-1(c)` in the vendored source. It sets the period but does not state a sourced extension standard and request window for this deadline; FRBP 9006(b)(1) does not by its text resolve whether a locally prescribed period falls within its scope. Baseline/after L5002 occurrences: 1/1. |
| bankr-txs | `motion-response-period` | `sources/bankr-txs/txs-bankruptcy-local-rules-2025-09-22-sectioned.txt` § `Bankr. S.D. Tex. BLR 9013-1(b)` | Left unchanged: read `Bankr. S.D. Tex. BLR 9013-1(b)` in the vendored source. It sets the period but does not state a sourced extension standard and request window for this deadline; FRBP 9006(b)(1) does not by its text resolve whether a locally prescribed period falls within its scope. Baseline/after L5002 occurrences: 1/1. |
| bankr-txw | `negative-notice-response-period` | `sources/bankr-txw/txw-local-rules-2025-02-03-sectioned.txt` § `Bankr. W.D. Tex. L. Rule 9014-1(a)` | Left unchanged: read `Bankr. W.D. Tex. L. Rule 9014-1(a)` in the vendored source. It sets the period but does not state a sourced extension standard and request window for this deadline; FRBP 9006(b)(1) does not by its text resolve whether a locally prescribed period falls within its scope. Baseline/after L5002 occurrences: 1/1. |
| bankr-ut | `response-period` | `sources/bankr-ut/utb-lbr-2025-12-01-sectioned.txt` § `Bankr. D. Utah LBR 9006-1(b)` | Left unchanged: read `Bankr. D. Utah LBR 9006-1(b)` in the vendored source. It sets the period but does not state a sourced extension standard and request window for this deadline; FRBP 9006(b)(1) does not by its text resolve whether a locally prescribed period falls within its scope. Baseline/after L5002 occurrences: 1/0. |
| bankr-vi | `motion-response-period` | `sources/bankr-vi/vi-local-rules-2026-02-01-sectioned.txt` § `D.V.I. LBR 9006-1(B)` | Left unchanged: read `D.V.I. LBR 9006-1(B)` in the vendored source. It sets the period but does not state a sourced extension standard and request window for this deadline; FRBP 9006(b)(1) does not by its text resolve whether a locally prescribed period falls within its scope. Baseline/after L5002 occurrences: 1/1. |
| bankr-wae | `objection-period` | `sources/bankr-wae/waeb-lbr-2026-07-17-sectioned.txt` § `Bankr. E.D. Wash. LBR 2002-1(c)` | Left unchanged: read `Bankr. E.D. Wash. LBR 2002-1(c)` in the vendored source. It sets the period but does not state a sourced extension standard and request window for this deadline; FRBP 9006(b)(1) does not by its text resolve whether a locally prescribed period falls within its scope. Baseline/after L5002 occurrences: 1/1. |
| bankr-wy | `objection-period` | `sources/bankr-wy/wyb-lbr-2012-07-01-sectioned.txt` § `Wyoming LBR 2002-1(B)` | Left unchanged: read `Wyoming LBR 2002-1(B)` in the vendored source. It sets the period but does not state a sourced extension standard and request window for this deadline; FRBP 9006(b)(1) does not by its text resolve whether a locally prescribed period falls within its scope. Baseline/after L5002 occurrences: 1/1. |
| bap1 | `fee-payment-period` | `sources/ca1-bap/ca1-bap-local-rules-sectioned.txt` § `1st Cir. BAP L.R. 8003-1(c)(1)` | Left unchanged: read `1st Cir. BAP L.R. 8003-1(c)(1)` in the vendored source. It sets the period but does not state a sourced extension standard and request window for this deadline; FRBP 9006(b)(1) does not by its text resolve whether a locally prescribed period falls within its scope. Baseline/after L5002 occurrences: 1/1. |
| bap1 | `reply-brief-period` | `sources/frbp.txt` § `FRBP 8018(a)(3)` | Wrote three FRBP 9006(b)(1) grants on `reply-brief-period` in `lit/frbp/appeals.lit`, each cited to `FRBP 9006(b)(1)`: court for good cause before expiry; court on a motion before expiry; court for excusable neglect on a motion after expiry. (The period is governed by 8018(a)(3).) Baseline/after L5002 occurrences: 1/0. |
| bap1 | `appellant-designation-period` | `sources/frbp.txt` § `FRBP 8009(a)(1)(B)` | Wrote three FRBP 9006(b)(1) grants on `appellant-designation-period` in `lit/frbp/appeals.lit`, each cited to `FRBP 9006(b)(1)`: court for good cause before expiry; court on a motion before expiry; court for excusable neglect on a motion after expiry. (The period is governed by 8009(a)(1)(B).) Baseline/after L5002 occurrences: 1/0. |
| bap1 | `appellant-election-period` | `sources/usc-28-158.txt` § `28 U.S.C. § 158(c)(1)(A)` | Wrote `extend { never cite "28 U.S.C. § 158(c)(1)" }` on `appellant-election-period` in `lit/frbp/appeals.lit`. Baseline/after L5002 occurrences: 1/0. |
| bap1 | `remand-report-period` | `sources/ca1-bap/ca1-bap-local-rules-sectioned.txt` § `1st Cir. BAP L.R. 8008-1(b)` | Left unchanged: read `1st Cir. BAP L.R. 8008-1(b)` in the vendored source. It sets the period but does not state a sourced extension standard and request window for this deadline; FRBP 9006(b)(1) does not by its text resolve whether a locally prescribed period falls within its scope. Baseline/after L5002 occurrences: 1/1. |
| bap1 | `appellee-brief-period` | `sources/frbp.txt` § `FRBP 8018(a)(2)` | Wrote three FRBP 9006(b)(1) grants on `appellee-brief-period` in `lit/frbp/appeals.lit`, each cited to `FRBP 9006(b)(1)`: court for good cause before expiry; court on a motion before expiry; court for excusable neglect on a motion after expiry. (The period is governed by 8018(a)(2).) Baseline/after L5002 occurrences: 1/0. |
| bap1 | `indicative-ruling-notice-period` | `sources/ca1-bap/ca1-bap-local-rules-sectioned.txt` § `1st Cir. BAP L.R. 8008-1(a)` | Left unchanged: read `1st Cir. BAP L.R. 8008-1(a)` in the vendored source. It sets the period but does not state a sourced extension standard and request window for this deadline; FRBP 9006(b)(1) does not by its text resolve whether a locally prescribed period falls within its scope. Baseline/after L5002 occurrences: 1/1. |
| bap1 | `appellant-brief-period` | `sources/frbp.txt` § `FRBP 8018(a)(1)` | Wrote three FRBP 9006(b)(1) grants on `appellant-brief-period` in `lit/frbp/appeals.lit`, each cited to `FRBP 9006(b)(1)`: court for good cause before expiry; court on a motion before expiry; court for excusable neglect on a motion after expiry. (The period is governed by 8018(a)(1).) Baseline/after L5002 occurrences: 1/0. |
| bap1 | `fee-after-waiver-denial-period` | `sources/ca1-bap/ca1-bap-local-rules-sectioned.txt` § `1st Cir. BAP L.R. 8003-1(c)(2)` | Left unchanged: read `1st Cir. BAP L.R. 8003-1(c)(2)` in the vendored source. It sets the period but does not state a sourced extension standard and request window for this deadline; FRBP 9006(b)(1) does not by its text resolve whether a locally prescribed period falls within its scope. Baseline/after L5002 occurrences: 1/1. |
| bap1 | `other-party-election-period` | `sources/usc-28-158.txt` § `28 U.S.C. § 158(c)(1)(B)` | Wrote `extend { never cite "28 U.S.C. § 158(c)(1)" }` on `other-party-election-period` in `lit/frbp/appeals.lit`. Baseline/after L5002 occurrences: 1/0. |
| bap1 | `motion-response-period` | `sources/frbp.txt` § `FRBP 8013(a)(3)(A)` | Wrote three FRBP 9006(b)(1) grants on `motion-response-period` in `lit/frbp/appeals.lit`, each cited to `FRBP 9006(b)(1)`: court for good cause before expiry; court on a motion before expiry; court for excusable neglect on a motion after expiry. (The period is governed by 8013(a)(3)(A).) Baseline/after L5002 occurrences: 1/0. |
| bap1 | `rehearing-response-period` | `sources/ca1-bap/ca1-bap-local-rules-sectioned.txt` § `1st Cir. BAP L.R. 8022-1(a)` | Left unchanged: read `1st Cir. BAP L.R. 8022-1(a)` in the vendored source. It sets the period but does not state a sourced extension standard and request window for this deadline; FRBP 9006(b)(1) does not by its text resolve whether a locally prescribed period falls within its scope. Baseline/after L5002 occurrences: 1/1. |
| bap1 | `direct-certification-request-period` | `sources/frbp.txt` § `FRBP 8006(f)(1)` | Wrote three FRBP 9006(b)(1) grants on `direct-certification-request-period` in `lit/frbp/appeals.lit`, each cited to `FRBP 9006(b)(1)`: court for good cause before expiry; court on a motion before expiry; court for excusable neglect on a motion after expiry. (The period is governed by 8006(f)(1).) Baseline/after L5002 occurrences: 1/0. |
| bap1 | `rehearing-period` | `sources/frbp.txt` § `FRBP 8022(a)(1)` | Wrote three FRBP 9006(b)(1) grants on `rehearing-period` in `lit/frbp/appeals.lit`, each cited to `FRBP 9006(b)(1)`: court for good cause before expiry; court on a motion before expiry; court for excusable neglect on a motion after expiry. (The period is governed by 8022(a)(1).) Baseline/after L5002 occurrences: 1/0. |
| bap1 | `reinstatement-period` | `sources/ca1-bap/ca1-bap-local-rules-sectioned.txt` § `1st Cir. BAP L.R. 8020-1(b)(1)` | Left unchanged: read `1st Cir. BAP L.R. 8020-1(b)(1)` in the vendored source. It sets the period but does not state a sourced extension standard and request window for this deadline; FRBP 9006(b)(1) does not by its text resolve whether a locally prescribed period falls within its scope. Baseline/after L5002 occurrences: 1/1. |
| bap1 | `election-validity-period` | `sources/frbp.txt` § `FRBP 8005(c)` | Wrote three FRBP 9006(b)(1) grants on `election-validity-period` in `lit/frbp/appeals.lit`, each cited to `FRBP 9006(b)(1)`: court for good cause before expiry; court on a motion before expiry; court for excusable neglect on a motion after expiry. (The period is governed by 8005(c).) Baseline/after L5002 occurrences: 1/0. |
| bap6 | `appellant-brief-period` | `sources/frbp.txt` § `FRBP 8018(a)(1)` | Wrote three FRBP 9006(b)(1) grants on `appellant-brief-period` in `lit/frbp/appeals.lit`, each cited to `FRBP 9006(b)(1)`: court for good cause before expiry; court on a motion before expiry; court for excusable neglect on a motion after expiry. (The period is governed by 8018(a)(1).) Baseline/after L5002 occurrences: 1/0. |
| bap6 | `default-cure-period` | `sources/ca6-bap/bap-local-rules-sectioned.txt` § `6th Cir. BAP LBR 8003-1(a)` | Left unchanged: read `6th Cir. BAP LBR 8003-1(a)` in the vendored source. It sets the period but does not state a sourced extension standard and request window for this deadline; FRBP 9006(b)(1) does not by its text resolve whether a locally prescribed period falls within its scope. Baseline/after L5002 occurrences: 1/1. |
| bap6 | `other-party-election-period` | `sources/usc-28-158.txt` § `28 U.S.C. § 158(c)(1)(B)` | Wrote `extend { never cite "28 U.S.C. § 158(c)(1)" }` on `other-party-election-period` in `lit/frbp/appeals.lit`. Baseline/after L5002 occurrences: 1/0. |
| bap6 | `direct-certification-request-period` | `sources/frbp.txt` § `FRBP 8006(f)(1)` | Wrote three FRBP 9006(b)(1) grants on `direct-certification-request-period` in `lit/frbp/appeals.lit`, each cited to `FRBP 9006(b)(1)`: court for good cause before expiry; court on a motion before expiry; court for excusable neglect on a motion after expiry. (The period is governed by 8006(f)(1).) Baseline/after L5002 occurrences: 1/0. |
| bap6 | `rehearing-period` | `sources/frbp.txt` § `FRBP 8022(a)(1)` | Wrote three FRBP 9006(b)(1) grants on `rehearing-period` in `lit/frbp/appeals.lit`, each cited to `FRBP 9006(b)(1)`: court for good cause before expiry; court on a motion before expiry; court for excusable neglect on a motion after expiry. (The period is governed by 8022(a)(1).) Baseline/after L5002 occurrences: 1/0. |
| bap6 | `appellant-designation-period` | `sources/frbp.txt` § `FRBP 8009(a)(1)(B)` | Wrote three FRBP 9006(b)(1) grants on `appellant-designation-period` in `lit/frbp/appeals.lit`, each cited to `FRBP 9006(b)(1)`: court for good cause before expiry; court on a motion before expiry; court for excusable neglect on a motion after expiry. (The period is governed by 8009(a)(1)(B).) Baseline/after L5002 occurrences: 1/0. |
| bap6 | `appellant-election-period` | `sources/usc-28-158.txt` § `28 U.S.C. § 158(c)(1)(A)` | Wrote `extend { never cite "28 U.S.C. § 158(c)(1)" }` on `appellant-election-period` in `lit/frbp/appeals.lit`. Baseline/after L5002 occurrences: 1/0. |
| bap6 | `election-validity-period` | `sources/frbp.txt` § `FRBP 8005(c)` | Wrote three FRBP 9006(b)(1) grants on `election-validity-period` in `lit/frbp/appeals.lit`, each cited to `FRBP 9006(b)(1)`: court for good cause before expiry; court on a motion before expiry; court for excusable neglect on a motion after expiry. (The period is governed by 8005(c).) Baseline/after L5002 occurrences: 1/0. |
| bap8 | `direct-authorization-petition-period` | `sources/frbp.txt` § `FRBP 8006(g)` | Wrote three FRBP 9006(b)(1) grants on `direct-authorization-petition-period` in `lit/frbp/appeals.lit`, each cited to `FRBP 9006(b)(1)`: court for good cause before expiry; court on a motion before expiry; court for excusable neglect on a motion after expiry. (The period is governed by 8006(g).) Baseline/after L5002 occurrences: 1/0. |
| bap8 | `appellant-brief-period` | `sources/frbp.txt` § `FRBP 8018(a)(1)` | Wrote three FRBP 9006(b)(1) grants on `appellant-brief-period` in `lit/frbp/appeals.lit`, each cited to `FRBP 9006(b)(1)`: court for good cause before expiry; court on a motion before expiry; court for excusable neglect on a motion after expiry. (The period is governed by 8018(a)(1).) Baseline/after L5002 occurrences: 1/0. |
| bap8 | `mandate-after-window-period` | `sources/ca8-bap-local-rules.txt` § `L.R. BAP 8th Cir. 8024A(b)` | Left unchanged: read `L.R. BAP 8th Cir. 8024A(b)` in the vendored source. It sets the period but does not state a sourced extension standard and request window for this deadline; FRBP 9006(b)(1) does not by its text resolve whether a locally prescribed period falls within its scope. Baseline/after L5002 occurrences: 1/1. |
| bap8 | `other-party-election-period` | `sources/usc-28-158.txt` § `28 U.S.C. § 158(c)(1)(B)` | Wrote `extend { never cite "28 U.S.C. § 158(c)(1)" }` on `other-party-election-period` in `lit/frbp/appeals.lit`. Baseline/after L5002 occurrences: 1/0. |
| bap8 | `mandate-after-denial-period` | `sources/ca8-bap-local-rules.txt` § `L.R. BAP 8th Cir. 8024A(c)(1)` | Left unchanged: read `L.R. BAP 8th Cir. 8024A(c)(1)` in the vendored source. It sets the period but does not state a sourced extension standard and request window for this deadline; FRBP 9006(b)(1) does not by its text resolve whether a locally prescribed period falls within its scope. Baseline/after L5002 occurrences: 1/1. |
| bap8 | `direct-certification-request-period` | `sources/frbp.txt` § `FRBP 8006(f)(1)` | Wrote three FRBP 9006(b)(1) grants on `direct-certification-request-period` in `lit/frbp/appeals.lit`, each cited to `FRBP 9006(b)(1)`: court for good cause before expiry; court on a motion before expiry; court for excusable neglect on a motion after expiry. (The period is governed by 8006(f)(1).) Baseline/after L5002 occurrences: 1/0. |
| bap8 | `rehearing-period` | `sources/frbp.txt` § `FRBP 8022(a)(1)` | Wrote three FRBP 9006(b)(1) grants on `rehearing-period` in `lit/frbp/appeals.lit`, each cited to `FRBP 9006(b)(1)`: court for good cause before expiry; court on a motion before expiry; court for excusable neglect on a motion after expiry. (The period is governed by 8022(a)(1).) Baseline/after L5002 occurrences: 1/0. |
| bap8 | `election-validity-period` | `sources/frbp.txt` § `FRBP 8005(c)` | Wrote three FRBP 9006(b)(1) grants on `election-validity-period` in `lit/frbp/appeals.lit`, each cited to `FRBP 9006(b)(1)`: court for good cause before expiry; court on a motion before expiry; court for excusable neglect on a motion after expiry. (The period is governed by 8005(c).) Baseline/after L5002 occurrences: 1/0. |
| bap10 | `rehearing-period` | `sources/frbp.txt` § `FRBP 8022(a)(1)` | Wrote three FRBP 9006(b)(1) grants on `rehearing-period` in `lit/frbp/appeals.lit`, each cited to `FRBP 9006(b)(1)`: court for good cause before expiry; court on a motion before expiry; court for excusable neglect on a motion after expiry. (The period is governed by 8022(a)(1).) Baseline/after L5002 occurrences: 1/0. |
| bap10 | `appellant-designation-period` | `sources/frbp.txt` § `FRBP 8009(a)(1)(B)` | Wrote three FRBP 9006(b)(1) grants on `appellant-designation-period` in `lit/frbp/appeals.lit`, each cited to `FRBP 9006(b)(1)`: court for good cause before expiry; court on a motion before expiry; court for excusable neglect on a motion after expiry. (The period is governed by 8009(a)(1)(B).) Baseline/after L5002 occurrences: 1/0. |
| bap10 | `appellant-election-period` | `sources/usc-28-158.txt` § `28 U.S.C. § 158(c)(1)(A)` | Wrote `extend { never cite "28 U.S.C. § 158(c)(1)" }` on `appellant-election-period` in `lit/frbp/appeals.lit`. Baseline/after L5002 occurrences: 1/0. |
| bap10 | `appellant-brief-period` | `sources/frbp.txt` § `FRBP 8018(a)(1)` | Wrote three FRBP 9006(b)(1) grants on `appellant-brief-period` in `lit/frbp/appeals.lit`, each cited to `FRBP 9006(b)(1)`: court for good cause before expiry; court on a motion before expiry; court for excusable neglect on a motion after expiry. (The period is governed by 8018(a)(1).) Baseline/after L5002 occurrences: 1/0. |
| bap10 | `other-party-election-period` | `sources/usc-28-158.txt` § `28 U.S.C. § 158(c)(1)(B)` | Wrote `extend { never cite "28 U.S.C. § 158(c)(1)" }` on `other-party-election-period` in `lit/frbp/appeals.lit`. Baseline/after L5002 occurrences: 1/0. |
| bap10 | `combined-statements-period` | `sources/ca10-bap/bap-local-rules-sectioned.txt` § `10th Cir. BAP L.R. 8003-2` | Left unchanged: read `10th Cir. BAP L.R. 8003-2` in the vendored source. It sets the period but does not state a sourced extension standard and request window for this deadline; FRBP 9006(b)(1) does not by its text resolve whether a locally prescribed period falls within its scope. Baseline/after L5002 occurrences: 1/1. |
| bap9 | `rehearing-period` | `sources/frbp.txt` § `FRBP 8022(a)(1)` | Wrote three FRBP 9006(b)(1) grants on `rehearing-period` in `lit/frbp/appeals.lit`, each cited to `FRBP 9006(b)(1)`: court for good cause before expiry; court on a motion before expiry; court for excusable neglect on a motion after expiry. (The period is governed by 8022(a)(1).) Baseline/after L5002 occurrences: 1/0. |
| bap9 | `appellant-designation-period` | `sources/frbp.txt` § `FRBP 8009(a)(1)(B)` | Wrote three FRBP 9006(b)(1) grants on `appellant-designation-period` in `lit/frbp/appeals.lit`, each cited to `FRBP 9006(b)(1)`: court for good cause before expiry; court on a motion before expiry; court for excusable neglect on a motion after expiry. (The period is governed by 8009(a)(1)(B).) Baseline/after L5002 occurrences: 1/0. |
| bap9 | `appellant-election-period` | `sources/usc-28-158.txt` § `28 U.S.C. § 158(c)(1)(A)` | Wrote `extend { never cite "28 U.S.C. § 158(c)(1)" }` on `appellant-election-period` in `lit/frbp/appeals.lit`. Baseline/after L5002 occurrences: 1/0. |
| bap9 | `appellant-brief-period` | `sources/frbp.txt` § `FRBP 8018(a)(1)` | Wrote three FRBP 9006(b)(1) grants on `appellant-brief-period` in `lit/frbp/appeals.lit`, each cited to `FRBP 9006(b)(1)`: court for good cause before expiry; court on a motion before expiry; court for excusable neglect on a motion after expiry. (The period is governed by 8018(a)(1).) Baseline/after L5002 occurrences: 1/0. |
| bap9 | `other-party-election-period` | `sources/usc-28-158.txt` § `28 U.S.C. § 158(c)(1)(B)` | Wrote `extend { never cite "28 U.S.C. § 158(c)(1)" }` on `other-party-election-period` in `lit/frbp/appeals.lit`. Baseline/after L5002 occurrences: 1/0. |

## Lowered pack diff

Generated baseline packs from commit `9cffd1d1b6856d8abcd66ccab575766c9e3a40c6` and after packs with the release `litigate build`, then compared with `scripts/lowered-json-diff.py`. The table below summarizes only changed `deadline/extendable`, `deadline/extensionAuthority`, and `deadline/extensions/**` leaves. Other lowered leaves are unchanged. The raw script output also reports removals of its synthetic `lowered.holes/deadline/extendable` rows; those are the corresponding L5002 holes disappearing and are not pack fields.

| Pack | Affected edges | `extendable` leaves | `extensionAuthority` leaves | `extensions` leaves |
|---|---:|---:|---:|---:|
| `bankr-cas-local-rules` | 1 | 1 | 1 | 14 |
| `bankr-or-local-rules` | 1 | 1 | 1 | 5 |
| `bankr-ut-local-rules` | 1 | 1 | 1 | 5 |
| `first-circuit-bankruptcy-appellate-panel` | 10 | 10 | 10 | 116 |
| `sixth-circuit-bankruptcy-appellate-panel` | 7 | 7 | 7 | 74 |
| `eighth-circuit-bankruptcy-appellate-panel` | 6 | 6 | 6 | 72 |
| `ninth-circuit-bankruptcy-appellate-panel` | 5 | 5 | 5 | 46 |
| `tenth-circuit-bankruptcy-appellate-panel` | 5 | 5 | 5 | 46 |

Changes: three local-rule packs gain one typed extension on their affected response period; five BAP packs lower the shared FRBP Part VIII grants and statutory election locks. `bap9` has no remaining L5002. The filtered diff is limited to the three requested extension field groups.

### Changed extension leaves

| Pack | Edge or pointer | Field | Old | New |
|---|---|---|---|---|
| bankr-cas-local-rules | `edge-motion-filed-opposed` | `deadline/extendable` | `<missing>` | `true` |
| bankr-cas-local-rules | `edge-motion-filed-opposed` | `deadline/extensionAuthority` | `<missing>` | `"Bankr. S.D. Cal. LBR 9006-1(d)"` |
| bankr-cas-local-rules | `edge-motion-filed-opposed` | `deadline/extensions/0/bound` | `<missing>` | `"discretionary"` |
| bankr-cas-local-rules | `edge-motion-filed-opposed` | `deadline/extensions/0/by` | `<missing>` | `"court"` |
| bankr-cas-local-rules | `edge-motion-filed-opposed` | `deadline/extensions/0/standard` | `<missing>` | `"GoodCause"` |
| bankr-cas-local-rules | `edge-motion-filed-opposed` | `deadline/extensions/0/window` | `<missing>` | `"beforeExpiry"` |
| bankr-cas-local-rules | `edge-motion-filed-opposed` | `deadline/extensions/1/bound` | `<missing>` | `"discretionary"` |
| bankr-cas-local-rules | `edge-motion-filed-opposed` | `deadline/extensions/1/by` | `<missing>` | `"court"` |
| bankr-cas-local-rules | `edge-motion-filed-opposed` | `deadline/extensions/1/motion/event` | `<missing>` | `"TimeLimitExtensionMoved()"` |
| bankr-cas-local-rules | `edge-motion-filed-opposed` | `deadline/extensions/1/standard` | `<missing>` | `"GoodCause"` |
| bankr-cas-local-rules | `edge-motion-filed-opposed` | `deadline/extensions/1/window` | `<missing>` | `"beforeExpiry"` |
| bankr-cas-local-rules | `edge-motion-filed-opposed` | `deadline/extensions/2/bound` | `<missing>` | `"discretionary"` |
| bankr-cas-local-rules | `edge-motion-filed-opposed` | `deadline/extensions/2/by` | `<missing>` | `"court"` |
| bankr-cas-local-rules | `edge-motion-filed-opposed` | `deadline/extensions/2/motion/event` | `<missing>` | `"TimeLimitExtensionMoved()"` |
| bankr-cas-local-rules | `edge-motion-filed-opposed` | `deadline/extensions/2/standard` | `<missing>` | `"ExcusableNeglect"` |
| bankr-cas-local-rules | `edge-motion-filed-opposed` | `deadline/extensions/2/window` | `<missing>` | `"afterExpiryOnMotion"` |
| bankr-or-local-rules | `edge-motion-filed-opposed` | `deadline/extendable` | `<missing>` | `true` |
| bankr-or-local-rules | `edge-motion-filed-opposed` | `deadline/extensionAuthority` | `<missing>` | `"Bankr. D. Or. LBR 9006-1(a)"` |
| bankr-or-local-rules | `edge-motion-filed-opposed` | `deadline/extensions/0/bound` | `<missing>` | `"discretionary"` |
| bankr-or-local-rules | `edge-motion-filed-opposed` | `deadline/extensions/0/by` | `<missing>` | `"court"` |
| bankr-or-local-rules | `edge-motion-filed-opposed` | `deadline/extensions/0/motion/event` | `<missing>` | `"TimeLimitExtensionMoved()"` |
| bankr-or-local-rules | `edge-motion-filed-opposed` | `deadline/extensions/0/standard` | `<missing>` | `"GoodCause"` |
| bankr-or-local-rules | `edge-motion-filed-opposed` | `deadline/extensions/0/window` | `<missing>` | `"beforeExpiry"` |
| bankr-ut-local-rules | `edge-motion-filed-opposed` | `deadline/extendable` | `<missing>` | `true` |
| bankr-ut-local-rules | `edge-motion-filed-opposed` | `deadline/extensionAuthority` | `<missing>` | `"Bankr. D. Utah LBR 9006-1(a)"` |
| bankr-ut-local-rules | `edge-motion-filed-opposed` | `deadline/extensions/0/bound` | `<missing>` | `"discretionary"` |
| bankr-ut-local-rules | `edge-motion-filed-opposed` | `deadline/extensions/0/by` | `<missing>` | `"court"` |
| bankr-ut-local-rules | `edge-motion-filed-opposed` | `deadline/extensions/0/motion/event` | `<missing>` | `"ResponsePeriodExtensionMoved()"` |
| bankr-ut-local-rules | `edge-motion-filed-opposed` | `deadline/extensions/0/standard` | `<missing>` | `"GoodCause"` |
| bankr-ut-local-rules | `edge-motion-filed-opposed` | `deadline/extensions/0/window` | `<missing>` | `"beforeExpiry"` |
| first-circuit-bankruptcy-appellate-panel | `edge-appealability-appellant-election` | `deadline/extendable` | `<missing>` | `false` |
| first-circuit-bankruptcy-appellate-panel | `edge-appealability-appellant-election` | `deadline/extensionAuthority` | `<missing>` | `"28 U.S.C. § 158(c)(1)"` |
| first-circuit-bankruptcy-appellate-panel | `edge-appealability-appellant-election` | `deadline/extensions/0/authority` | `<missing>` | `"28 U.S.C. § 158(c)(1)"` |
| first-circuit-bankruptcy-appellate-panel | `edge-appealability-appellant-election` | `deadline/extensions/0/bound` | `<missing>` | `"nonExtendable"` |
| first-circuit-bankruptcy-appellate-panel | `edge-appealability-direct-request` | `deadline/extendable` | `<missing>` | `true` |
| first-circuit-bankruptcy-appellate-panel | `edge-appealability-direct-request` | `deadline/extensionAuthority` | `<missing>` | `"FRBP 9006(b)(1)"` |
| first-circuit-bankruptcy-appellate-panel | `edge-appealability-direct-request` | `deadline/extensions/0/bound` | `<missing>` | `"discretionary"` |
| first-circuit-bankruptcy-appellate-panel | `edge-appealability-direct-request` | `deadline/extensions/0/by` | `<missing>` | `"court"` |
| first-circuit-bankruptcy-appellate-panel | `edge-appealability-direct-request` | `deadline/extensions/0/standard` | `<missing>` | `"GoodCause"` |
| first-circuit-bankruptcy-appellate-panel | `edge-appealability-direct-request` | `deadline/extensions/0/window` | `<missing>` | `"beforeExpiry"` |
| first-circuit-bankruptcy-appellate-panel | `edge-appealability-direct-request` | `deadline/extensions/1/bound` | `<missing>` | `"discretionary"` |
| first-circuit-bankruptcy-appellate-panel | `edge-appealability-direct-request` | `deadline/extensions/1/by` | `<missing>` | `"court"` |
| first-circuit-bankruptcy-appellate-panel | `edge-appealability-direct-request` | `deadline/extensions/1/motion/event` | `<missing>` | `"PartEightDeadlineExtensionMoved()"` |
| first-circuit-bankruptcy-appellate-panel | `edge-appealability-direct-request` | `deadline/extensions/1/standard` | `<missing>` | `"GoodCause"` |
| first-circuit-bankruptcy-appellate-panel | `edge-appealability-direct-request` | `deadline/extensions/1/window` | `<missing>` | `"beforeExpiry"` |
| first-circuit-bankruptcy-appellate-panel | `edge-appealability-direct-request` | `deadline/extensions/2/bound` | `<missing>` | `"discretionary"` |
| first-circuit-bankruptcy-appellate-panel | `edge-appealability-direct-request` | `deadline/extensions/2/by` | `<missing>` | `"court"` |
| first-circuit-bankruptcy-appellate-panel | `edge-appealability-direct-request` | `deadline/extensions/2/motion/event` | `<missing>` | `"PartEightDeadlineExtensionMoved()"` |
| first-circuit-bankruptcy-appellate-panel | `edge-appealability-direct-request` | `deadline/extensions/2/standard` | `<missing>` | `"ExcusableNeglect"` |
| first-circuit-bankruptcy-appellate-panel | `edge-appealability-direct-request` | `deadline/extensions/2/window` | `<missing>` | `"afterExpiryOnMotion"` |
| first-circuit-bankruptcy-appellate-panel | `edge-appealability-election-challenge` | `deadline/extendable` | `<missing>` | `true` |
| first-circuit-bankruptcy-appellate-panel | `edge-appealability-election-challenge` | `deadline/extensionAuthority` | `<missing>` | `"FRBP 9006(b)(1)"` |
| first-circuit-bankruptcy-appellate-panel | `edge-appealability-election-challenge` | `deadline/extensions/0/bound` | `<missing>` | `"discretionary"` |
| first-circuit-bankruptcy-appellate-panel | `edge-appealability-election-challenge` | `deadline/extensions/0/by` | `<missing>` | `"court"` |
| first-circuit-bankruptcy-appellate-panel | `edge-appealability-election-challenge` | `deadline/extensions/0/standard` | `<missing>` | `"GoodCause"` |
| first-circuit-bankruptcy-appellate-panel | `edge-appealability-election-challenge` | `deadline/extensions/0/window` | `<missing>` | `"beforeExpiry"` |
| first-circuit-bankruptcy-appellate-panel | `edge-appealability-election-challenge` | `deadline/extensions/1/bound` | `<missing>` | `"discretionary"` |
| first-circuit-bankruptcy-appellate-panel | `edge-appealability-election-challenge` | `deadline/extensions/1/by` | `<missing>` | `"court"` |
| first-circuit-bankruptcy-appellate-panel | `edge-appealability-election-challenge` | `deadline/extensions/1/motion/event` | `<missing>` | `"PartEightDeadlineExtensionMoved()"` |
| first-circuit-bankruptcy-appellate-panel | `edge-appealability-election-challenge` | `deadline/extensions/1/standard` | `<missing>` | `"GoodCause"` |
| first-circuit-bankruptcy-appellate-panel | `edge-appealability-election-challenge` | `deadline/extensions/1/window` | `<missing>` | `"beforeExpiry"` |
| first-circuit-bankruptcy-appellate-panel | `edge-appealability-election-challenge` | `deadline/extensions/2/bound` | `<missing>` | `"discretionary"` |
| first-circuit-bankruptcy-appellate-panel | `edge-appealability-election-challenge` | `deadline/extensions/2/by` | `<missing>` | `"court"` |
| first-circuit-bankruptcy-appellate-panel | `edge-appealability-election-challenge` | `deadline/extensions/2/motion/event` | `<missing>` | `"PartEightDeadlineExtensionMoved()"` |
| first-circuit-bankruptcy-appellate-panel | `edge-appealability-election-challenge` | `deadline/extensions/2/standard` | `<missing>` | `"ExcusableNeglect"` |
| first-circuit-bankruptcy-appellate-panel | `edge-appealability-election-challenge` | `deadline/extensions/2/window` | `<missing>` | `"afterExpiryOnMotion"` |
| first-circuit-bankruptcy-appellate-panel | `edge-appealability-other-election` | `deadline/extendable` | `<missing>` | `false` |
| first-circuit-bankruptcy-appellate-panel | `edge-appealability-other-election` | `deadline/extensionAuthority` | `<missing>` | `"28 U.S.C. § 158(c)(1)"` |
| first-circuit-bankruptcy-appellate-panel | `edge-appealability-other-election` | `deadline/extensions/0/authority` | `<missing>` | `"28 U.S.C. § 158(c)(1)"` |
| first-circuit-bankruptcy-appellate-panel | `edge-appealability-other-election` | `deadline/extensions/0/bound` | `<missing>` | `"nonExtendable"` |
| first-circuit-bankruptcy-appellate-panel | `edge-appellee-briefing-appellee` | `deadline/extendable` | `<missing>` | `true` |
| first-circuit-bankruptcy-appellate-panel | `edge-appellee-briefing-appellee` | `deadline/extensionAuthority` | `<missing>` | `"FRBP 9006(b)(1)"` |
| first-circuit-bankruptcy-appellate-panel | `edge-appellee-briefing-appellee` | `deadline/extensions/0/bound` | `<missing>` | `"discretionary"` |
| first-circuit-bankruptcy-appellate-panel | `edge-appellee-briefing-appellee` | `deadline/extensions/0/by` | `<missing>` | `"court"` |
| first-circuit-bankruptcy-appellate-panel | `edge-appellee-briefing-appellee` | `deadline/extensions/0/standard` | `<missing>` | `"GoodCause"` |
| first-circuit-bankruptcy-appellate-panel | `edge-appellee-briefing-appellee` | `deadline/extensions/0/window` | `<missing>` | `"beforeExpiry"` |
| first-circuit-bankruptcy-appellate-panel | `edge-appellee-briefing-appellee` | `deadline/extensions/1/bound` | `<missing>` | `"discretionary"` |
| first-circuit-bankruptcy-appellate-panel | `edge-appellee-briefing-appellee` | `deadline/extensions/1/by` | `<missing>` | `"court"` |
| first-circuit-bankruptcy-appellate-panel | `edge-appellee-briefing-appellee` | `deadline/extensions/1/motion/event` | `<missing>` | `"PartEightDeadlineExtensionMoved()"` |
| first-circuit-bankruptcy-appellate-panel | `edge-appellee-briefing-appellee` | `deadline/extensions/1/standard` | `<missing>` | `"GoodCause"` |
| first-circuit-bankruptcy-appellate-panel | `edge-appellee-briefing-appellee` | `deadline/extensions/1/window` | `<missing>` | `"beforeExpiry"` |
| first-circuit-bankruptcy-appellate-panel | `edge-appellee-briefing-appellee` | `deadline/extensions/2/bound` | `<missing>` | `"discretionary"` |
| first-circuit-bankruptcy-appellate-panel | `edge-appellee-briefing-appellee` | `deadline/extensions/2/by` | `<missing>` | `"court"` |
| first-circuit-bankruptcy-appellate-panel | `edge-appellee-briefing-appellee` | `deadline/extensions/2/motion/event` | `<missing>` | `"PartEightDeadlineExtensionMoved()"` |
| first-circuit-bankruptcy-appellate-panel | `edge-appellee-briefing-appellee` | `deadline/extensions/2/standard` | `<missing>` | `"ExcusableNeglect"` |
| first-circuit-bankruptcy-appellate-panel | `edge-appellee-briefing-appellee` | `deadline/extensions/2/window` | `<missing>` | `"afterExpiryOnMotion"` |
| first-circuit-bankruptcy-appellate-panel | `edge-appellee-briefing-reply` | `deadline/extendable` | `<missing>` | `true` |
| first-circuit-bankruptcy-appellate-panel | `edge-appellee-briefing-reply` | `deadline/extensionAuthority` | `<missing>` | `"FRBP 9006(b)(1)"` |
| first-circuit-bankruptcy-appellate-panel | `edge-appellee-briefing-reply` | `deadline/extensions/0/bound` | `<missing>` | `"discretionary"` |
| first-circuit-bankruptcy-appellate-panel | `edge-appellee-briefing-reply` | `deadline/extensions/0/by` | `<missing>` | `"court"` |
| first-circuit-bankruptcy-appellate-panel | `edge-appellee-briefing-reply` | `deadline/extensions/0/standard` | `<missing>` | `"GoodCause"` |
| first-circuit-bankruptcy-appellate-panel | `edge-appellee-briefing-reply` | `deadline/extensions/0/window` | `<missing>` | `"beforeExpiry"` |
| first-circuit-bankruptcy-appellate-panel | `edge-appellee-briefing-reply` | `deadline/extensions/1/bound` | `<missing>` | `"discretionary"` |
| first-circuit-bankruptcy-appellate-panel | `edge-appellee-briefing-reply` | `deadline/extensions/1/by` | `<missing>` | `"court"` |
| first-circuit-bankruptcy-appellate-panel | `edge-appellee-briefing-reply` | `deadline/extensions/1/motion/event` | `<missing>` | `"PartEightDeadlineExtensionMoved()"` |
| first-circuit-bankruptcy-appellate-panel | `edge-appellee-briefing-reply` | `deadline/extensions/1/standard` | `<missing>` | `"GoodCause"` |
| first-circuit-bankruptcy-appellate-panel | `edge-appellee-briefing-reply` | `deadline/extensions/1/window` | `<missing>` | `"beforeExpiry"` |
| first-circuit-bankruptcy-appellate-panel | `edge-appellee-briefing-reply` | `deadline/extensions/2/bound` | `<missing>` | `"discretionary"` |
| first-circuit-bankruptcy-appellate-panel | `edge-appellee-briefing-reply` | `deadline/extensions/2/by` | `<missing>` | `"court"` |
| first-circuit-bankruptcy-appellate-panel | `edge-appellee-briefing-reply` | `deadline/extensions/2/motion/event` | `<missing>` | `"PartEightDeadlineExtensionMoved()"` |
| first-circuit-bankruptcy-appellate-panel | `edge-appellee-briefing-reply` | `deadline/extensions/2/standard` | `<missing>` | `"ExcusableNeglect"` |
| first-circuit-bankruptcy-appellate-panel | `edge-appellee-briefing-reply` | `deadline/extensions/2/window` | `<missing>` | `"afterExpiryOnMotion"` |
| first-circuit-bankruptcy-appellate-panel | `edge-briefing-appellee` | `deadline/extendable` | `<missing>` | `true` |
| first-circuit-bankruptcy-appellate-panel | `edge-briefing-appellee` | `deadline/extensionAuthority` | `<missing>` | `"FRBP 9006(b)(1)"` |
| first-circuit-bankruptcy-appellate-panel | `edge-briefing-appellee` | `deadline/extensions/0/bound` | `<missing>` | `"discretionary"` |
| first-circuit-bankruptcy-appellate-panel | `edge-briefing-appellee` | `deadline/extensions/0/by` | `<missing>` | `"court"` |
| first-circuit-bankruptcy-appellate-panel | `edge-briefing-appellee` | `deadline/extensions/0/standard` | `<missing>` | `"GoodCause"` |
| first-circuit-bankruptcy-appellate-panel | `edge-briefing-appellee` | `deadline/extensions/0/window` | `<missing>` | `"beforeExpiry"` |
| first-circuit-bankruptcy-appellate-panel | `edge-briefing-appellee` | `deadline/extensions/1/bound` | `<missing>` | `"discretionary"` |
| first-circuit-bankruptcy-appellate-panel | `edge-briefing-appellee` | `deadline/extensions/1/by` | `<missing>` | `"court"` |
| first-circuit-bankruptcy-appellate-panel | `edge-briefing-appellee` | `deadline/extensions/1/motion/event` | `<missing>` | `"PartEightDeadlineExtensionMoved()"` |
| first-circuit-bankruptcy-appellate-panel | `edge-briefing-appellee` | `deadline/extensions/1/standard` | `<missing>` | `"GoodCause"` |
| first-circuit-bankruptcy-appellate-panel | `edge-briefing-appellee` | `deadline/extensions/1/window` | `<missing>` | `"beforeExpiry"` |
| first-circuit-bankruptcy-appellate-panel | `edge-briefing-appellee` | `deadline/extensions/2/bound` | `<missing>` | `"discretionary"` |
| first-circuit-bankruptcy-appellate-panel | `edge-briefing-appellee` | `deadline/extensions/2/by` | `<missing>` | `"court"` |
| first-circuit-bankruptcy-appellate-panel | `edge-briefing-appellee` | `deadline/extensions/2/motion/event` | `<missing>` | `"PartEightDeadlineExtensionMoved()"` |
| first-circuit-bankruptcy-appellate-panel | `edge-briefing-appellee` | `deadline/extensions/2/standard` | `<missing>` | `"ExcusableNeglect"` |
| first-circuit-bankruptcy-appellate-panel | `edge-briefing-appellee` | `deadline/extensions/2/window` | `<missing>` | `"afterExpiryOnMotion"` |
| first-circuit-bankruptcy-appellate-panel | `edge-record-briefing` | `deadline/extendable` | `<missing>` | `true` |
| first-circuit-bankruptcy-appellate-panel | `edge-record-briefing` | `deadline/extensionAuthority` | `<missing>` | `"FRBP 9006(b)(1)"` |
| first-circuit-bankruptcy-appellate-panel | `edge-record-briefing` | `deadline/extensions/0/bound` | `<missing>` | `"discretionary"` |
| first-circuit-bankruptcy-appellate-panel | `edge-record-briefing` | `deadline/extensions/0/by` | `<missing>` | `"court"` |
| first-circuit-bankruptcy-appellate-panel | `edge-record-briefing` | `deadline/extensions/0/standard` | `<missing>` | `"GoodCause"` |
| first-circuit-bankruptcy-appellate-panel | `edge-record-briefing` | `deadline/extensions/0/window` | `<missing>` | `"beforeExpiry"` |
| first-circuit-bankruptcy-appellate-panel | `edge-record-briefing` | `deadline/extensions/1/bound` | `<missing>` | `"discretionary"` |
| first-circuit-bankruptcy-appellate-panel | `edge-record-briefing` | `deadline/extensions/1/by` | `<missing>` | `"court"` |
| first-circuit-bankruptcy-appellate-panel | `edge-record-briefing` | `deadline/extensions/1/motion/event` | `<missing>` | `"PartEightDeadlineExtensionMoved()"` |
| first-circuit-bankruptcy-appellate-panel | `edge-record-briefing` | `deadline/extensions/1/standard` | `<missing>` | `"GoodCause"` |
| first-circuit-bankruptcy-appellate-panel | `edge-record-briefing` | `deadline/extensions/1/window` | `<missing>` | `"beforeExpiry"` |
| first-circuit-bankruptcy-appellate-panel | `edge-record-briefing` | `deadline/extensions/2/bound` | `<missing>` | `"discretionary"` |
| first-circuit-bankruptcy-appellate-panel | `edge-record-briefing` | `deadline/extensions/2/by` | `<missing>` | `"court"` |
| first-circuit-bankruptcy-appellate-panel | `edge-record-briefing` | `deadline/extensions/2/motion/event` | `<missing>` | `"PartEightDeadlineExtensionMoved()"` |
| first-circuit-bankruptcy-appellate-panel | `edge-record-briefing` | `deadline/extensions/2/standard` | `<missing>` | `"ExcusableNeglect"` |
| first-circuit-bankruptcy-appellate-panel | `edge-record-briefing` | `deadline/extensions/2/window` | `<missing>` | `"afterExpiryOnMotion"` |
| first-circuit-bankruptcy-appellate-panel | `edge-record-motion` | `deadline/extendable` | `<missing>` | `true` |
| first-circuit-bankruptcy-appellate-panel | `edge-record-motion` | `deadline/extensionAuthority` | `<missing>` | `"FRBP 9006(b)(1)"` |
| first-circuit-bankruptcy-appellate-panel | `edge-record-motion` | `deadline/extensions/0/bound` | `<missing>` | `"discretionary"` |
| first-circuit-bankruptcy-appellate-panel | `edge-record-motion` | `deadline/extensions/0/by` | `<missing>` | `"court"` |
| first-circuit-bankruptcy-appellate-panel | `edge-record-motion` | `deadline/extensions/0/standard` | `<missing>` | `"GoodCause"` |
| first-circuit-bankruptcy-appellate-panel | `edge-record-motion` | `deadline/extensions/0/window` | `<missing>` | `"beforeExpiry"` |
| first-circuit-bankruptcy-appellate-panel | `edge-record-motion` | `deadline/extensions/1/bound` | `<missing>` | `"discretionary"` |
| first-circuit-bankruptcy-appellate-panel | `edge-record-motion` | `deadline/extensions/1/by` | `<missing>` | `"court"` |
| first-circuit-bankruptcy-appellate-panel | `edge-record-motion` | `deadline/extensions/1/motion/event` | `<missing>` | `"PartEightDeadlineExtensionMoved()"` |
| first-circuit-bankruptcy-appellate-panel | `edge-record-motion` | `deadline/extensions/1/standard` | `<missing>` | `"GoodCause"` |
| first-circuit-bankruptcy-appellate-panel | `edge-record-motion` | `deadline/extensions/1/window` | `<missing>` | `"beforeExpiry"` |
| first-circuit-bankruptcy-appellate-panel | `edge-record-motion` | `deadline/extensions/2/bound` | `<missing>` | `"discretionary"` |
| first-circuit-bankruptcy-appellate-panel | `edge-record-motion` | `deadline/extensions/2/by` | `<missing>` | `"court"` |
| first-circuit-bankruptcy-appellate-panel | `edge-record-motion` | `deadline/extensions/2/motion/event` | `<missing>` | `"PartEightDeadlineExtensionMoved()"` |
| first-circuit-bankruptcy-appellate-panel | `edge-record-motion` | `deadline/extensions/2/standard` | `<missing>` | `"ExcusableNeglect"` |
| first-circuit-bankruptcy-appellate-panel | `edge-record-motion` | `deadline/extensions/2/window` | `<missing>` | `"afterExpiryOnMotion"` |
| first-circuit-bankruptcy-appellate-panel | `edge-rehearing-window-rehearing-ruling` | `deadline/extendable` | `<missing>` | `true` |
| first-circuit-bankruptcy-appellate-panel | `edge-rehearing-window-rehearing-ruling` | `deadline/extensionAuthority` | `<missing>` | `"FRBP 9006(b)(1)"` |
| first-circuit-bankruptcy-appellate-panel | `edge-rehearing-window-rehearing-ruling` | `deadline/extensions/0/bound` | `<missing>` | `"discretionary"` |
| first-circuit-bankruptcy-appellate-panel | `edge-rehearing-window-rehearing-ruling` | `deadline/extensions/0/by` | `<missing>` | `"court"` |
| first-circuit-bankruptcy-appellate-panel | `edge-rehearing-window-rehearing-ruling` | `deadline/extensions/0/standard` | `<missing>` | `"GoodCause"` |
| first-circuit-bankruptcy-appellate-panel | `edge-rehearing-window-rehearing-ruling` | `deadline/extensions/0/window` | `<missing>` | `"beforeExpiry"` |
| first-circuit-bankruptcy-appellate-panel | `edge-rehearing-window-rehearing-ruling` | `deadline/extensions/1/bound` | `<missing>` | `"discretionary"` |
| first-circuit-bankruptcy-appellate-panel | `edge-rehearing-window-rehearing-ruling` | `deadline/extensions/1/by` | `<missing>` | `"court"` |
| first-circuit-bankruptcy-appellate-panel | `edge-rehearing-window-rehearing-ruling` | `deadline/extensions/1/motion/event` | `<missing>` | `"PartEightDeadlineExtensionMoved()"` |
| first-circuit-bankruptcy-appellate-panel | `edge-rehearing-window-rehearing-ruling` | `deadline/extensions/1/standard` | `<missing>` | `"GoodCause"` |
| first-circuit-bankruptcy-appellate-panel | `edge-rehearing-window-rehearing-ruling` | `deadline/extensions/1/window` | `<missing>` | `"beforeExpiry"` |
| first-circuit-bankruptcy-appellate-panel | `edge-rehearing-window-rehearing-ruling` | `deadline/extensions/2/bound` | `<missing>` | `"discretionary"` |
| first-circuit-bankruptcy-appellate-panel | `edge-rehearing-window-rehearing-ruling` | `deadline/extensions/2/by` | `<missing>` | `"court"` |
| first-circuit-bankruptcy-appellate-panel | `edge-rehearing-window-rehearing-ruling` | `deadline/extensions/2/motion/event` | `<missing>` | `"PartEightDeadlineExtensionMoved()"` |
| first-circuit-bankruptcy-appellate-panel | `edge-rehearing-window-rehearing-ruling` | `deadline/extensions/2/standard` | `<missing>` | `"ExcusableNeglect"` |
| first-circuit-bankruptcy-appellate-panel | `edge-rehearing-window-rehearing-ruling` | `deadline/extensions/2/window` | `<missing>` | `"afterExpiryOnMotion"` |
| sixth-circuit-bankruptcy-appellate-panel | `edge-appealability-appellant-election` | `deadline/extendable` | `<missing>` | `false` |
| sixth-circuit-bankruptcy-appellate-panel | `edge-appealability-appellant-election` | `deadline/extensionAuthority` | `<missing>` | `"28 U.S.C. § 158(c)(1)"` |
| sixth-circuit-bankruptcy-appellate-panel | `edge-appealability-appellant-election` | `deadline/extensions/0/authority` | `<missing>` | `"28 U.S.C. § 158(c)(1)"` |
| sixth-circuit-bankruptcy-appellate-panel | `edge-appealability-appellant-election` | `deadline/extensions/0/bound` | `<missing>` | `"nonExtendable"` |
| sixth-circuit-bankruptcy-appellate-panel | `edge-appealability-direct-request` | `deadline/extendable` | `<missing>` | `true` |
| sixth-circuit-bankruptcy-appellate-panel | `edge-appealability-direct-request` | `deadline/extensionAuthority` | `<missing>` | `"FRBP 9006(b)(1)"` |
| sixth-circuit-bankruptcy-appellate-panel | `edge-appealability-direct-request` | `deadline/extensions/0/bound` | `<missing>` | `"discretionary"` |
| sixth-circuit-bankruptcy-appellate-panel | `edge-appealability-direct-request` | `deadline/extensions/0/by` | `<missing>` | `"court"` |
| sixth-circuit-bankruptcy-appellate-panel | `edge-appealability-direct-request` | `deadline/extensions/0/standard` | `<missing>` | `"GoodCause"` |
| sixth-circuit-bankruptcy-appellate-panel | `edge-appealability-direct-request` | `deadline/extensions/0/window` | `<missing>` | `"beforeExpiry"` |
| sixth-circuit-bankruptcy-appellate-panel | `edge-appealability-direct-request` | `deadline/extensions/1/bound` | `<missing>` | `"discretionary"` |
| sixth-circuit-bankruptcy-appellate-panel | `edge-appealability-direct-request` | `deadline/extensions/1/by` | `<missing>` | `"court"` |
| sixth-circuit-bankruptcy-appellate-panel | `edge-appealability-direct-request` | `deadline/extensions/1/motion/event` | `<missing>` | `"PartEightDeadlineExtensionMoved()"` |
| sixth-circuit-bankruptcy-appellate-panel | `edge-appealability-direct-request` | `deadline/extensions/1/standard` | `<missing>` | `"GoodCause"` |
| sixth-circuit-bankruptcy-appellate-panel | `edge-appealability-direct-request` | `deadline/extensions/1/window` | `<missing>` | `"beforeExpiry"` |
| sixth-circuit-bankruptcy-appellate-panel | `edge-appealability-direct-request` | `deadline/extensions/2/bound` | `<missing>` | `"discretionary"` |
| sixth-circuit-bankruptcy-appellate-panel | `edge-appealability-direct-request` | `deadline/extensions/2/by` | `<missing>` | `"court"` |
| sixth-circuit-bankruptcy-appellate-panel | `edge-appealability-direct-request` | `deadline/extensions/2/motion/event` | `<missing>` | `"PartEightDeadlineExtensionMoved()"` |
| sixth-circuit-bankruptcy-appellate-panel | `edge-appealability-direct-request` | `deadline/extensions/2/standard` | `<missing>` | `"ExcusableNeglect"` |
| sixth-circuit-bankruptcy-appellate-panel | `edge-appealability-direct-request` | `deadline/extensions/2/window` | `<missing>` | `"afterExpiryOnMotion"` |
| sixth-circuit-bankruptcy-appellate-panel | `edge-appealability-election-challenge` | `deadline/extendable` | `<missing>` | `true` |
| sixth-circuit-bankruptcy-appellate-panel | `edge-appealability-election-challenge` | `deadline/extensionAuthority` | `<missing>` | `"FRBP 9006(b)(1)"` |
| sixth-circuit-bankruptcy-appellate-panel | `edge-appealability-election-challenge` | `deadline/extensions/0/bound` | `<missing>` | `"discretionary"` |
| sixth-circuit-bankruptcy-appellate-panel | `edge-appealability-election-challenge` | `deadline/extensions/0/by` | `<missing>` | `"court"` |
| sixth-circuit-bankruptcy-appellate-panel | `edge-appealability-election-challenge` | `deadline/extensions/0/standard` | `<missing>` | `"GoodCause"` |
| sixth-circuit-bankruptcy-appellate-panel | `edge-appealability-election-challenge` | `deadline/extensions/0/window` | `<missing>` | `"beforeExpiry"` |
| sixth-circuit-bankruptcy-appellate-panel | `edge-appealability-election-challenge` | `deadline/extensions/1/bound` | `<missing>` | `"discretionary"` |
| sixth-circuit-bankruptcy-appellate-panel | `edge-appealability-election-challenge` | `deadline/extensions/1/by` | `<missing>` | `"court"` |
| sixth-circuit-bankruptcy-appellate-panel | `edge-appealability-election-challenge` | `deadline/extensions/1/motion/event` | `<missing>` | `"PartEightDeadlineExtensionMoved()"` |
| sixth-circuit-bankruptcy-appellate-panel | `edge-appealability-election-challenge` | `deadline/extensions/1/standard` | `<missing>` | `"GoodCause"` |
| sixth-circuit-bankruptcy-appellate-panel | `edge-appealability-election-challenge` | `deadline/extensions/1/window` | `<missing>` | `"beforeExpiry"` |
| sixth-circuit-bankruptcy-appellate-panel | `edge-appealability-election-challenge` | `deadline/extensions/2/bound` | `<missing>` | `"discretionary"` |
| sixth-circuit-bankruptcy-appellate-panel | `edge-appealability-election-challenge` | `deadline/extensions/2/by` | `<missing>` | `"court"` |
| sixth-circuit-bankruptcy-appellate-panel | `edge-appealability-election-challenge` | `deadline/extensions/2/motion/event` | `<missing>` | `"PartEightDeadlineExtensionMoved()"` |
| sixth-circuit-bankruptcy-appellate-panel | `edge-appealability-election-challenge` | `deadline/extensions/2/standard` | `<missing>` | `"ExcusableNeglect"` |
| sixth-circuit-bankruptcy-appellate-panel | `edge-appealability-election-challenge` | `deadline/extensions/2/window` | `<missing>` | `"afterExpiryOnMotion"` |
| sixth-circuit-bankruptcy-appellate-panel | `edge-appealability-other-election` | `deadline/extendable` | `<missing>` | `false` |
| sixth-circuit-bankruptcy-appellate-panel | `edge-appealability-other-election` | `deadline/extensionAuthority` | `<missing>` | `"28 U.S.C. § 158(c)(1)"` |
| sixth-circuit-bankruptcy-appellate-panel | `edge-appealability-other-election` | `deadline/extensions/0/authority` | `<missing>` | `"28 U.S.C. § 158(c)(1)"` |
| sixth-circuit-bankruptcy-appellate-panel | `edge-appealability-other-election` | `deadline/extensions/0/bound` | `<missing>` | `"nonExtendable"` |
| sixth-circuit-bankruptcy-appellate-panel | `edge-briefing-argument` | `deadline/extendable` | `<missing>` | `true` |
| sixth-circuit-bankruptcy-appellate-panel | `edge-briefing-argument` | `deadline/extensionAuthority` | `<missing>` | `"FRBP 9006(b)(1)"` |
| sixth-circuit-bankruptcy-appellate-panel | `edge-briefing-argument` | `deadline/extensions/0/bound` | `<missing>` | `"discretionary"` |
| sixth-circuit-bankruptcy-appellate-panel | `edge-briefing-argument` | `deadline/extensions/0/by` | `<missing>` | `"court"` |
| sixth-circuit-bankruptcy-appellate-panel | `edge-briefing-argument` | `deadline/extensions/0/standard` | `<missing>` | `"GoodCause"` |
| sixth-circuit-bankruptcy-appellate-panel | `edge-briefing-argument` | `deadline/extensions/0/window` | `<missing>` | `"beforeExpiry"` |
| sixth-circuit-bankruptcy-appellate-panel | `edge-briefing-argument` | `deadline/extensions/1/bound` | `<missing>` | `"discretionary"` |
| sixth-circuit-bankruptcy-appellate-panel | `edge-briefing-argument` | `deadline/extensions/1/by` | `<missing>` | `"court"` |
| sixth-circuit-bankruptcy-appellate-panel | `edge-briefing-argument` | `deadline/extensions/1/motion/event` | `<missing>` | `"PartEightDeadlineExtensionMoved()"` |
| sixth-circuit-bankruptcy-appellate-panel | `edge-briefing-argument` | `deadline/extensions/1/standard` | `<missing>` | `"GoodCause"` |
| sixth-circuit-bankruptcy-appellate-panel | `edge-briefing-argument` | `deadline/extensions/1/window` | `<missing>` | `"beforeExpiry"` |
| sixth-circuit-bankruptcy-appellate-panel | `edge-briefing-argument` | `deadline/extensions/2/bound` | `<missing>` | `"discretionary"` |
| sixth-circuit-bankruptcy-appellate-panel | `edge-briefing-argument` | `deadline/extensions/2/by` | `<missing>` | `"court"` |
| sixth-circuit-bankruptcy-appellate-panel | `edge-briefing-argument` | `deadline/extensions/2/motion/event` | `<missing>` | `"PartEightDeadlineExtensionMoved()"` |
| sixth-circuit-bankruptcy-appellate-panel | `edge-briefing-argument` | `deadline/extensions/2/standard` | `<missing>` | `"ExcusableNeglect"` |
| sixth-circuit-bankruptcy-appellate-panel | `edge-briefing-argument` | `deadline/extensions/2/window` | `<missing>` | `"afterExpiryOnMotion"` |
| sixth-circuit-bankruptcy-appellate-panel | `edge-record-briefing` | `deadline/extendable` | `<missing>` | `true` |
| sixth-circuit-bankruptcy-appellate-panel | `edge-record-briefing` | `deadline/extensionAuthority` | `<missing>` | `"FRBP 9006(b)(1)"` |
| sixth-circuit-bankruptcy-appellate-panel | `edge-record-briefing` | `deadline/extensions/0/bound` | `<missing>` | `"discretionary"` |
| sixth-circuit-bankruptcy-appellate-panel | `edge-record-briefing` | `deadline/extensions/0/by` | `<missing>` | `"court"` |
| sixth-circuit-bankruptcy-appellate-panel | `edge-record-briefing` | `deadline/extensions/0/standard` | `<missing>` | `"GoodCause"` |
| sixth-circuit-bankruptcy-appellate-panel | `edge-record-briefing` | `deadline/extensions/0/window` | `<missing>` | `"beforeExpiry"` |
| sixth-circuit-bankruptcy-appellate-panel | `edge-record-briefing` | `deadline/extensions/1/bound` | `<missing>` | `"discretionary"` |
| sixth-circuit-bankruptcy-appellate-panel | `edge-record-briefing` | `deadline/extensions/1/by` | `<missing>` | `"court"` |
| sixth-circuit-bankruptcy-appellate-panel | `edge-record-briefing` | `deadline/extensions/1/motion/event` | `<missing>` | `"PartEightDeadlineExtensionMoved()"` |
| sixth-circuit-bankruptcy-appellate-panel | `edge-record-briefing` | `deadline/extensions/1/standard` | `<missing>` | `"GoodCause"` |
| sixth-circuit-bankruptcy-appellate-panel | `edge-record-briefing` | `deadline/extensions/1/window` | `<missing>` | `"beforeExpiry"` |
| sixth-circuit-bankruptcy-appellate-panel | `edge-record-briefing` | `deadline/extensions/2/bound` | `<missing>` | `"discretionary"` |
| sixth-circuit-bankruptcy-appellate-panel | `edge-record-briefing` | `deadline/extensions/2/by` | `<missing>` | `"court"` |
| sixth-circuit-bankruptcy-appellate-panel | `edge-record-briefing` | `deadline/extensions/2/motion/event` | `<missing>` | `"PartEightDeadlineExtensionMoved()"` |
| sixth-circuit-bankruptcy-appellate-panel | `edge-record-briefing` | `deadline/extensions/2/standard` | `<missing>` | `"ExcusableNeglect"` |
| sixth-circuit-bankruptcy-appellate-panel | `edge-record-briefing` | `deadline/extensions/2/window` | `<missing>` | `"afterExpiryOnMotion"` |
| sixth-circuit-bankruptcy-appellate-panel | `edge-rehearing-window-rehearing-ruling` | `deadline/extendable` | `<missing>` | `true` |
| sixth-circuit-bankruptcy-appellate-panel | `edge-rehearing-window-rehearing-ruling` | `deadline/extensionAuthority` | `<missing>` | `"FRBP 9006(b)(1)"` |
| sixth-circuit-bankruptcy-appellate-panel | `edge-rehearing-window-rehearing-ruling` | `deadline/extensions/0/bound` | `<missing>` | `"discretionary"` |
| sixth-circuit-bankruptcy-appellate-panel | `edge-rehearing-window-rehearing-ruling` | `deadline/extensions/0/by` | `<missing>` | `"court"` |
| sixth-circuit-bankruptcy-appellate-panel | `edge-rehearing-window-rehearing-ruling` | `deadline/extensions/0/standard` | `<missing>` | `"GoodCause"` |
| sixth-circuit-bankruptcy-appellate-panel | `edge-rehearing-window-rehearing-ruling` | `deadline/extensions/0/window` | `<missing>` | `"beforeExpiry"` |
| sixth-circuit-bankruptcy-appellate-panel | `edge-rehearing-window-rehearing-ruling` | `deadline/extensions/1/bound` | `<missing>` | `"discretionary"` |
| sixth-circuit-bankruptcy-appellate-panel | `edge-rehearing-window-rehearing-ruling` | `deadline/extensions/1/by` | `<missing>` | `"court"` |
| sixth-circuit-bankruptcy-appellate-panel | `edge-rehearing-window-rehearing-ruling` | `deadline/extensions/1/motion/event` | `<missing>` | `"PartEightDeadlineExtensionMoved()"` |
| sixth-circuit-bankruptcy-appellate-panel | `edge-rehearing-window-rehearing-ruling` | `deadline/extensions/1/standard` | `<missing>` | `"GoodCause"` |
| sixth-circuit-bankruptcy-appellate-panel | `edge-rehearing-window-rehearing-ruling` | `deadline/extensions/1/window` | `<missing>` | `"beforeExpiry"` |
| sixth-circuit-bankruptcy-appellate-panel | `edge-rehearing-window-rehearing-ruling` | `deadline/extensions/2/bound` | `<missing>` | `"discretionary"` |
| sixth-circuit-bankruptcy-appellate-panel | `edge-rehearing-window-rehearing-ruling` | `deadline/extensions/2/by` | `<missing>` | `"court"` |
| sixth-circuit-bankruptcy-appellate-panel | `edge-rehearing-window-rehearing-ruling` | `deadline/extensions/2/motion/event` | `<missing>` | `"PartEightDeadlineExtensionMoved()"` |
| sixth-circuit-bankruptcy-appellate-panel | `edge-rehearing-window-rehearing-ruling` | `deadline/extensions/2/standard` | `<missing>` | `"ExcusableNeglect"` |
| sixth-circuit-bankruptcy-appellate-panel | `edge-rehearing-window-rehearing-ruling` | `deadline/extensions/2/window` | `<missing>` | `"afterExpiryOnMotion"` |
| eighth-circuit-bankruptcy-appellate-panel | `edge-appealability-direct-request` | `deadline/extendable` | `<missing>` | `true` |
| eighth-circuit-bankruptcy-appellate-panel | `edge-appealability-direct-request` | `deadline/extensionAuthority` | `<missing>` | `"FRBP 9006(b)(1)"` |
| eighth-circuit-bankruptcy-appellate-panel | `edge-appealability-direct-request` | `deadline/extensions/0/bound` | `<missing>` | `"discretionary"` |
| eighth-circuit-bankruptcy-appellate-panel | `edge-appealability-direct-request` | `deadline/extensions/0/by` | `<missing>` | `"court"` |
| eighth-circuit-bankruptcy-appellate-panel | `edge-appealability-direct-request` | `deadline/extensions/0/standard` | `<missing>` | `"GoodCause"` |
| eighth-circuit-bankruptcy-appellate-panel | `edge-appealability-direct-request` | `deadline/extensions/0/window` | `<missing>` | `"beforeExpiry"` |
| eighth-circuit-bankruptcy-appellate-panel | `edge-appealability-direct-request` | `deadline/extensions/1/bound` | `<missing>` | `"discretionary"` |
| eighth-circuit-bankruptcy-appellate-panel | `edge-appealability-direct-request` | `deadline/extensions/1/by` | `<missing>` | `"court"` |
| eighth-circuit-bankruptcy-appellate-panel | `edge-appealability-direct-request` | `deadline/extensions/1/motion/event` | `<missing>` | `"PartEightDeadlineExtensionMoved()"` |
| eighth-circuit-bankruptcy-appellate-panel | `edge-appealability-direct-request` | `deadline/extensions/1/standard` | `<missing>` | `"GoodCause"` |
| eighth-circuit-bankruptcy-appellate-panel | `edge-appealability-direct-request` | `deadline/extensions/1/window` | `<missing>` | `"beforeExpiry"` |
| eighth-circuit-bankruptcy-appellate-panel | `edge-appealability-direct-request` | `deadline/extensions/2/bound` | `<missing>` | `"discretionary"` |
| eighth-circuit-bankruptcy-appellate-panel | `edge-appealability-direct-request` | `deadline/extensions/2/by` | `<missing>` | `"court"` |
| eighth-circuit-bankruptcy-appellate-panel | `edge-appealability-direct-request` | `deadline/extensions/2/motion/event` | `<missing>` | `"PartEightDeadlineExtensionMoved()"` |
| eighth-circuit-bankruptcy-appellate-panel | `edge-appealability-direct-request` | `deadline/extensions/2/standard` | `<missing>` | `"ExcusableNeglect"` |
| eighth-circuit-bankruptcy-appellate-panel | `edge-appealability-direct-request` | `deadline/extensions/2/window` | `<missing>` | `"afterExpiryOnMotion"` |
| eighth-circuit-bankruptcy-appellate-panel | `edge-appealability-district` | `deadline/extendable` | `<missing>` | `false` |
| eighth-circuit-bankruptcy-appellate-panel | `edge-appealability-district` | `deadline/extensionAuthority` | `<missing>` | `"28 U.S.C. § 158(c)(1)"` |
| eighth-circuit-bankruptcy-appellate-panel | `edge-appealability-district` | `deadline/extensions/0/authority` | `<missing>` | `"28 U.S.C. § 158(c)(1)"` |
| eighth-circuit-bankruptcy-appellate-panel | `edge-appealability-district` | `deadline/extensions/0/bound` | `<missing>` | `"nonExtendable"` |
| eighth-circuit-bankruptcy-appellate-panel | `edge-appealability-election-challenge` | `deadline/extendable` | `<missing>` | `true` |
| eighth-circuit-bankruptcy-appellate-panel | `edge-appealability-election-challenge` | `deadline/extensionAuthority` | `<missing>` | `"FRBP 9006(b)(1)"` |
| eighth-circuit-bankruptcy-appellate-panel | `edge-appealability-election-challenge` | `deadline/extensions/0/bound` | `<missing>` | `"discretionary"` |
| eighth-circuit-bankruptcy-appellate-panel | `edge-appealability-election-challenge` | `deadline/extensions/0/by` | `<missing>` | `"court"` |
| eighth-circuit-bankruptcy-appellate-panel | `edge-appealability-election-challenge` | `deadline/extensions/0/standard` | `<missing>` | `"GoodCause"` |
| eighth-circuit-bankruptcy-appellate-panel | `edge-appealability-election-challenge` | `deadline/extensions/0/window` | `<missing>` | `"beforeExpiry"` |
| eighth-circuit-bankruptcy-appellate-panel | `edge-appealability-election-challenge` | `deadline/extensions/1/bound` | `<missing>` | `"discretionary"` |
| eighth-circuit-bankruptcy-appellate-panel | `edge-appealability-election-challenge` | `deadline/extensions/1/by` | `<missing>` | `"court"` |
| eighth-circuit-bankruptcy-appellate-panel | `edge-appealability-election-challenge` | `deadline/extensions/1/motion/event` | `<missing>` | `"PartEightDeadlineExtensionMoved()"` |
| eighth-circuit-bankruptcy-appellate-panel | `edge-appealability-election-challenge` | `deadline/extensions/1/standard` | `<missing>` | `"GoodCause"` |
| eighth-circuit-bankruptcy-appellate-panel | `edge-appealability-election-challenge` | `deadline/extensions/1/window` | `<missing>` | `"beforeExpiry"` |
| eighth-circuit-bankruptcy-appellate-panel | `edge-appealability-election-challenge` | `deadline/extensions/2/bound` | `<missing>` | `"discretionary"` |
| eighth-circuit-bankruptcy-appellate-panel | `edge-appealability-election-challenge` | `deadline/extensions/2/by` | `<missing>` | `"court"` |
| eighth-circuit-bankruptcy-appellate-panel | `edge-appealability-election-challenge` | `deadline/extensions/2/motion/event` | `<missing>` | `"PartEightDeadlineExtensionMoved()"` |
| eighth-circuit-bankruptcy-appellate-panel | `edge-appealability-election-challenge` | `deadline/extensions/2/standard` | `<missing>` | `"ExcusableNeglect"` |
| eighth-circuit-bankruptcy-appellate-panel | `edge-appealability-election-challenge` | `deadline/extensions/2/window` | `<missing>` | `"afterExpiryOnMotion"` |
| eighth-circuit-bankruptcy-appellate-panel | `edge-circuit-handoff-done` | `deadline/extendable` | `<missing>` | `true` |
| eighth-circuit-bankruptcy-appellate-panel | `edge-circuit-handoff-done` | `deadline/extensionAuthority` | `<missing>` | `"FRBP 9006(b)(1)"` |
| eighth-circuit-bankruptcy-appellate-panel | `edge-circuit-handoff-done` | `deadline/extensions/0/bound` | `<missing>` | `"discretionary"` |
| eighth-circuit-bankruptcy-appellate-panel | `edge-circuit-handoff-done` | `deadline/extensions/0/by` | `<missing>` | `"court"` |
| eighth-circuit-bankruptcy-appellate-panel | `edge-circuit-handoff-done` | `deadline/extensions/0/standard` | `<missing>` | `"GoodCause"` |
| eighth-circuit-bankruptcy-appellate-panel | `edge-circuit-handoff-done` | `deadline/extensions/0/window` | `<missing>` | `"beforeExpiry"` |
| eighth-circuit-bankruptcy-appellate-panel | `edge-circuit-handoff-done` | `deadline/extensions/1/bound` | `<missing>` | `"discretionary"` |
| eighth-circuit-bankruptcy-appellate-panel | `edge-circuit-handoff-done` | `deadline/extensions/1/by` | `<missing>` | `"court"` |
| eighth-circuit-bankruptcy-appellate-panel | `edge-circuit-handoff-done` | `deadline/extensions/1/motion/event` | `<missing>` | `"PartEightDeadlineExtensionMoved()"` |
| eighth-circuit-bankruptcy-appellate-panel | `edge-circuit-handoff-done` | `deadline/extensions/1/standard` | `<missing>` | `"GoodCause"` |
| eighth-circuit-bankruptcy-appellate-panel | `edge-circuit-handoff-done` | `deadline/extensions/1/window` | `<missing>` | `"beforeExpiry"` |
| eighth-circuit-bankruptcy-appellate-panel | `edge-circuit-handoff-done` | `deadline/extensions/2/bound` | `<missing>` | `"discretionary"` |
| eighth-circuit-bankruptcy-appellate-panel | `edge-circuit-handoff-done` | `deadline/extensions/2/by` | `<missing>` | `"court"` |
| eighth-circuit-bankruptcy-appellate-panel | `edge-circuit-handoff-done` | `deadline/extensions/2/motion/event` | `<missing>` | `"PartEightDeadlineExtensionMoved()"` |
| eighth-circuit-bankruptcy-appellate-panel | `edge-circuit-handoff-done` | `deadline/extensions/2/standard` | `<missing>` | `"ExcusableNeglect"` |
| eighth-circuit-bankruptcy-appellate-panel | `edge-circuit-handoff-done` | `deadline/extensions/2/window` | `<missing>` | `"afterExpiryOnMotion"` |
| eighth-circuit-bankruptcy-appellate-panel | `edge-decision-rehearing` | `deadline/extendable` | `<missing>` | `true` |
| eighth-circuit-bankruptcy-appellate-panel | `edge-decision-rehearing` | `deadline/extensionAuthority` | `<missing>` | `"FRBP 9006(b)(1)"` |
| eighth-circuit-bankruptcy-appellate-panel | `edge-decision-rehearing` | `deadline/extensions/0/bound` | `<missing>` | `"discretionary"` |
| eighth-circuit-bankruptcy-appellate-panel | `edge-decision-rehearing` | `deadline/extensions/0/by` | `<missing>` | `"court"` |
| eighth-circuit-bankruptcy-appellate-panel | `edge-decision-rehearing` | `deadline/extensions/0/standard` | `<missing>` | `"GoodCause"` |
| eighth-circuit-bankruptcy-appellate-panel | `edge-decision-rehearing` | `deadline/extensions/0/window` | `<missing>` | `"beforeExpiry"` |
| eighth-circuit-bankruptcy-appellate-panel | `edge-decision-rehearing` | `deadline/extensions/1/bound` | `<missing>` | `"discretionary"` |
| eighth-circuit-bankruptcy-appellate-panel | `edge-decision-rehearing` | `deadline/extensions/1/by` | `<missing>` | `"court"` |
| eighth-circuit-bankruptcy-appellate-panel | `edge-decision-rehearing` | `deadline/extensions/1/motion/event` | `<missing>` | `"PartEightDeadlineExtensionMoved()"` |
| eighth-circuit-bankruptcy-appellate-panel | `edge-decision-rehearing` | `deadline/extensions/1/standard` | `<missing>` | `"GoodCause"` |
| eighth-circuit-bankruptcy-appellate-panel | `edge-decision-rehearing` | `deadline/extensions/1/window` | `<missing>` | `"beforeExpiry"` |
| eighth-circuit-bankruptcy-appellate-panel | `edge-decision-rehearing` | `deadline/extensions/2/bound` | `<missing>` | `"discretionary"` |
| eighth-circuit-bankruptcy-appellate-panel | `edge-decision-rehearing` | `deadline/extensions/2/by` | `<missing>` | `"court"` |
| eighth-circuit-bankruptcy-appellate-panel | `edge-decision-rehearing` | `deadline/extensions/2/motion/event` | `<missing>` | `"PartEightDeadlineExtensionMoved()"` |
| eighth-circuit-bankruptcy-appellate-panel | `edge-decision-rehearing` | `deadline/extensions/2/standard` | `<missing>` | `"ExcusableNeglect"` |
| eighth-circuit-bankruptcy-appellate-panel | `edge-decision-rehearing` | `deadline/extensions/2/window` | `<missing>` | `"afterExpiryOnMotion"` |
| eighth-circuit-bankruptcy-appellate-panel | `edge-record-argument` | `deadline/extendable` | `<missing>` | `true` |
| eighth-circuit-bankruptcy-appellate-panel | `edge-record-argument` | `deadline/extensionAuthority` | `<missing>` | `"FRBP 9006(b)(1)"` |
| eighth-circuit-bankruptcy-appellate-panel | `edge-record-argument` | `deadline/extensions/0/bound` | `<missing>` | `"discretionary"` |
| eighth-circuit-bankruptcy-appellate-panel | `edge-record-argument` | `deadline/extensions/0/by` | `<missing>` | `"court"` |
| eighth-circuit-bankruptcy-appellate-panel | `edge-record-argument` | `deadline/extensions/0/standard` | `<missing>` | `"GoodCause"` |
| eighth-circuit-bankruptcy-appellate-panel | `edge-record-argument` | `deadline/extensions/0/window` | `<missing>` | `"beforeExpiry"` |
| eighth-circuit-bankruptcy-appellate-panel | `edge-record-argument` | `deadline/extensions/1/bound` | `<missing>` | `"discretionary"` |
| eighth-circuit-bankruptcy-appellate-panel | `edge-record-argument` | `deadline/extensions/1/by` | `<missing>` | `"court"` |
| eighth-circuit-bankruptcy-appellate-panel | `edge-record-argument` | `deadline/extensions/1/motion/event` | `<missing>` | `"PartEightDeadlineExtensionMoved()"` |
| eighth-circuit-bankruptcy-appellate-panel | `edge-record-argument` | `deadline/extensions/1/standard` | `<missing>` | `"GoodCause"` |
| eighth-circuit-bankruptcy-appellate-panel | `edge-record-argument` | `deadline/extensions/1/window` | `<missing>` | `"beforeExpiry"` |
| eighth-circuit-bankruptcy-appellate-panel | `edge-record-argument` | `deadline/extensions/2/bound` | `<missing>` | `"discretionary"` |
| eighth-circuit-bankruptcy-appellate-panel | `edge-record-argument` | `deadline/extensions/2/by` | `<missing>` | `"court"` |
| eighth-circuit-bankruptcy-appellate-panel | `edge-record-argument` | `deadline/extensions/2/motion/event` | `<missing>` | `"PartEightDeadlineExtensionMoved()"` |
| eighth-circuit-bankruptcy-appellate-panel | `edge-record-argument` | `deadline/extensions/2/standard` | `<missing>` | `"ExcusableNeglect"` |
| eighth-circuit-bankruptcy-appellate-panel | `edge-record-argument` | `deadline/extensions/2/window` | `<missing>` | `"afterExpiryOnMotion"` |
| ninth-circuit-bankruptcy-appellate-panel | `edge-appealability-appellant-election` | `deadline/extendable` | `<missing>` | `false` |
| ninth-circuit-bankruptcy-appellate-panel | `edge-appealability-appellant-election` | `deadline/extensionAuthority` | `<missing>` | `"28 U.S.C. § 158(c)(1)"` |
| ninth-circuit-bankruptcy-appellate-panel | `edge-appealability-appellant-election` | `deadline/extensions/0/authority` | `<missing>` | `"28 U.S.C. § 158(c)(1)"` |
| ninth-circuit-bankruptcy-appellate-panel | `edge-appealability-appellant-election` | `deadline/extensions/0/bound` | `<missing>` | `"nonExtendable"` |
| ninth-circuit-bankruptcy-appellate-panel | `edge-appealability-other-election` | `deadline/extendable` | `<missing>` | `false` |
| ninth-circuit-bankruptcy-appellate-panel | `edge-appealability-other-election` | `deadline/extensionAuthority` | `<missing>` | `"28 U.S.C. § 158(c)(1)"` |
| ninth-circuit-bankruptcy-appellate-panel | `edge-appealability-other-election` | `deadline/extensions/0/authority` | `<missing>` | `"28 U.S.C. § 158(c)(1)"` |
| ninth-circuit-bankruptcy-appellate-panel | `edge-appealability-other-election` | `deadline/extensions/0/bound` | `<missing>` | `"nonExtendable"` |
| ninth-circuit-bankruptcy-appellate-panel | `edge-briefing-argument` | `deadline/extendable` | `<missing>` | `true` |
| ninth-circuit-bankruptcy-appellate-panel | `edge-briefing-argument` | `deadline/extensionAuthority` | `<missing>` | `"FRBP 9006(b)(1)"` |
| ninth-circuit-bankruptcy-appellate-panel | `edge-briefing-argument` | `deadline/extensions/0/bound` | `<missing>` | `"discretionary"` |
| ninth-circuit-bankruptcy-appellate-panel | `edge-briefing-argument` | `deadline/extensions/0/by` | `<missing>` | `"court"` |
| ninth-circuit-bankruptcy-appellate-panel | `edge-briefing-argument` | `deadline/extensions/0/standard` | `<missing>` | `"GoodCause"` |
| ninth-circuit-bankruptcy-appellate-panel | `edge-briefing-argument` | `deadline/extensions/0/window` | `<missing>` | `"beforeExpiry"` |
| ninth-circuit-bankruptcy-appellate-panel | `edge-briefing-argument` | `deadline/extensions/1/bound` | `<missing>` | `"discretionary"` |
| ninth-circuit-bankruptcy-appellate-panel | `edge-briefing-argument` | `deadline/extensions/1/by` | `<missing>` | `"court"` |
| ninth-circuit-bankruptcy-appellate-panel | `edge-briefing-argument` | `deadline/extensions/1/motion/event` | `<missing>` | `"PartEightDeadlineExtensionMoved()"` |
| ninth-circuit-bankruptcy-appellate-panel | `edge-briefing-argument` | `deadline/extensions/1/standard` | `<missing>` | `"GoodCause"` |
| ninth-circuit-bankruptcy-appellate-panel | `edge-briefing-argument` | `deadline/extensions/1/window` | `<missing>` | `"beforeExpiry"` |
| ninth-circuit-bankruptcy-appellate-panel | `edge-briefing-argument` | `deadline/extensions/2/bound` | `<missing>` | `"discretionary"` |
| ninth-circuit-bankruptcy-appellate-panel | `edge-briefing-argument` | `deadline/extensions/2/by` | `<missing>` | `"court"` |
| ninth-circuit-bankruptcy-appellate-panel | `edge-briefing-argument` | `deadline/extensions/2/motion/event` | `<missing>` | `"PartEightDeadlineExtensionMoved()"` |
| ninth-circuit-bankruptcy-appellate-panel | `edge-briefing-argument` | `deadline/extensions/2/standard` | `<missing>` | `"ExcusableNeglect"` |
| ninth-circuit-bankruptcy-appellate-panel | `edge-briefing-argument` | `deadline/extensions/2/window` | `<missing>` | `"afterExpiryOnMotion"` |
| ninth-circuit-bankruptcy-appellate-panel | `edge-record-assembled-briefing` | `deadline/extendable` | `<missing>` | `true` |
| ninth-circuit-bankruptcy-appellate-panel | `edge-record-assembled-briefing` | `deadline/extensionAuthority` | `<missing>` | `"FRBP 9006(b)(1)"` |
| ninth-circuit-bankruptcy-appellate-panel | `edge-record-assembled-briefing` | `deadline/extensions/0/bound` | `<missing>` | `"discretionary"` |
| ninth-circuit-bankruptcy-appellate-panel | `edge-record-assembled-briefing` | `deadline/extensions/0/by` | `<missing>` | `"court"` |
| ninth-circuit-bankruptcy-appellate-panel | `edge-record-assembled-briefing` | `deadline/extensions/0/standard` | `<missing>` | `"GoodCause"` |
| ninth-circuit-bankruptcy-appellate-panel | `edge-record-assembled-briefing` | `deadline/extensions/0/window` | `<missing>` | `"beforeExpiry"` |
| ninth-circuit-bankruptcy-appellate-panel | `edge-record-assembled-briefing` | `deadline/extensions/1/bound` | `<missing>` | `"discretionary"` |
| ninth-circuit-bankruptcy-appellate-panel | `edge-record-assembled-briefing` | `deadline/extensions/1/by` | `<missing>` | `"court"` |
| ninth-circuit-bankruptcy-appellate-panel | `edge-record-assembled-briefing` | `deadline/extensions/1/motion/event` | `<missing>` | `"PartEightDeadlineExtensionMoved()"` |
| ninth-circuit-bankruptcy-appellate-panel | `edge-record-assembled-briefing` | `deadline/extensions/1/standard` | `<missing>` | `"GoodCause"` |
| ninth-circuit-bankruptcy-appellate-panel | `edge-record-assembled-briefing` | `deadline/extensions/1/window` | `<missing>` | `"beforeExpiry"` |
| ninth-circuit-bankruptcy-appellate-panel | `edge-record-assembled-briefing` | `deadline/extensions/2/bound` | `<missing>` | `"discretionary"` |
| ninth-circuit-bankruptcy-appellate-panel | `edge-record-assembled-briefing` | `deadline/extensions/2/by` | `<missing>` | `"court"` |
| ninth-circuit-bankruptcy-appellate-panel | `edge-record-assembled-briefing` | `deadline/extensions/2/motion/event` | `<missing>` | `"PartEightDeadlineExtensionMoved()"` |
| ninth-circuit-bankruptcy-appellate-panel | `edge-record-assembled-briefing` | `deadline/extensions/2/standard` | `<missing>` | `"ExcusableNeglect"` |
| ninth-circuit-bankruptcy-appellate-panel | `edge-record-assembled-briefing` | `deadline/extensions/2/window` | `<missing>` | `"afterExpiryOnMotion"` |
| ninth-circuit-bankruptcy-appellate-panel | `edge-rehearing-window-rehearing-ruling` | `deadline/extendable` | `<missing>` | `true` |
| ninth-circuit-bankruptcy-appellate-panel | `edge-rehearing-window-rehearing-ruling` | `deadline/extensionAuthority` | `<missing>` | `"FRBP 9006(b)(1)"` |
| ninth-circuit-bankruptcy-appellate-panel | `edge-rehearing-window-rehearing-ruling` | `deadline/extensions/0/bound` | `<missing>` | `"discretionary"` |
| ninth-circuit-bankruptcy-appellate-panel | `edge-rehearing-window-rehearing-ruling` | `deadline/extensions/0/by` | `<missing>` | `"court"` |
| ninth-circuit-bankruptcy-appellate-panel | `edge-rehearing-window-rehearing-ruling` | `deadline/extensions/0/standard` | `<missing>` | `"GoodCause"` |
| ninth-circuit-bankruptcy-appellate-panel | `edge-rehearing-window-rehearing-ruling` | `deadline/extensions/0/window` | `<missing>` | `"beforeExpiry"` |
| ninth-circuit-bankruptcy-appellate-panel | `edge-rehearing-window-rehearing-ruling` | `deadline/extensions/1/bound` | `<missing>` | `"discretionary"` |
| ninth-circuit-bankruptcy-appellate-panel | `edge-rehearing-window-rehearing-ruling` | `deadline/extensions/1/by` | `<missing>` | `"court"` |
| ninth-circuit-bankruptcy-appellate-panel | `edge-rehearing-window-rehearing-ruling` | `deadline/extensions/1/motion/event` | `<missing>` | `"PartEightDeadlineExtensionMoved()"` |
| ninth-circuit-bankruptcy-appellate-panel | `edge-rehearing-window-rehearing-ruling` | `deadline/extensions/1/standard` | `<missing>` | `"GoodCause"` |
| ninth-circuit-bankruptcy-appellate-panel | `edge-rehearing-window-rehearing-ruling` | `deadline/extensions/1/window` | `<missing>` | `"beforeExpiry"` |
| ninth-circuit-bankruptcy-appellate-panel | `edge-rehearing-window-rehearing-ruling` | `deadline/extensions/2/bound` | `<missing>` | `"discretionary"` |
| ninth-circuit-bankruptcy-appellate-panel | `edge-rehearing-window-rehearing-ruling` | `deadline/extensions/2/by` | `<missing>` | `"court"` |
| ninth-circuit-bankruptcy-appellate-panel | `edge-rehearing-window-rehearing-ruling` | `deadline/extensions/2/motion/event` | `<missing>` | `"PartEightDeadlineExtensionMoved()"` |
| ninth-circuit-bankruptcy-appellate-panel | `edge-rehearing-window-rehearing-ruling` | `deadline/extensions/2/standard` | `<missing>` | `"ExcusableNeglect"` |
| ninth-circuit-bankruptcy-appellate-panel | `edge-rehearing-window-rehearing-ruling` | `deadline/extensions/2/window` | `<missing>` | `"afterExpiryOnMotion"` |
| tenth-circuit-bankruptcy-appellate-panel | `edge-appealability-appellant-election` | `deadline/extendable` | `<missing>` | `false` |
| tenth-circuit-bankruptcy-appellate-panel | `edge-appealability-appellant-election` | `deadline/extensionAuthority` | `<missing>` | `"28 U.S.C. § 158(c)(1)"` |
| tenth-circuit-bankruptcy-appellate-panel | `edge-appealability-appellant-election` | `deadline/extensions/0/authority` | `<missing>` | `"28 U.S.C. § 158(c)(1)"` |
| tenth-circuit-bankruptcy-appellate-panel | `edge-appealability-appellant-election` | `deadline/extensions/0/bound` | `<missing>` | `"nonExtendable"` |
| tenth-circuit-bankruptcy-appellate-panel | `edge-appealability-other-election` | `deadline/extendable` | `<missing>` | `false` |
| tenth-circuit-bankruptcy-appellate-panel | `edge-appealability-other-election` | `deadline/extensionAuthority` | `<missing>` | `"28 U.S.C. § 158(c)(1)"` |
| tenth-circuit-bankruptcy-appellate-panel | `edge-appealability-other-election` | `deadline/extensions/0/authority` | `<missing>` | `"28 U.S.C. § 158(c)(1)"` |
| tenth-circuit-bankruptcy-appellate-panel | `edge-appealability-other-election` | `deadline/extensions/0/bound` | `<missing>` | `"nonExtendable"` |
| tenth-circuit-bankruptcy-appellate-panel | `edge-briefing-argument` | `deadline/extendable` | `<missing>` | `true` |
| tenth-circuit-bankruptcy-appellate-panel | `edge-briefing-argument` | `deadline/extensionAuthority` | `<missing>` | `"FRBP 9006(b)(1)"` |
| tenth-circuit-bankruptcy-appellate-panel | `edge-briefing-argument` | `deadline/extensions/0/bound` | `<missing>` | `"discretionary"` |
| tenth-circuit-bankruptcy-appellate-panel | `edge-briefing-argument` | `deadline/extensions/0/by` | `<missing>` | `"court"` |
| tenth-circuit-bankruptcy-appellate-panel | `edge-briefing-argument` | `deadline/extensions/0/standard` | `<missing>` | `"GoodCause"` |
| tenth-circuit-bankruptcy-appellate-panel | `edge-briefing-argument` | `deadline/extensions/0/window` | `<missing>` | `"beforeExpiry"` |
| tenth-circuit-bankruptcy-appellate-panel | `edge-briefing-argument` | `deadline/extensions/1/bound` | `<missing>` | `"discretionary"` |
| tenth-circuit-bankruptcy-appellate-panel | `edge-briefing-argument` | `deadline/extensions/1/by` | `<missing>` | `"court"` |
| tenth-circuit-bankruptcy-appellate-panel | `edge-briefing-argument` | `deadline/extensions/1/motion/event` | `<missing>` | `"PartEightDeadlineExtensionMoved()"` |
| tenth-circuit-bankruptcy-appellate-panel | `edge-briefing-argument` | `deadline/extensions/1/standard` | `<missing>` | `"GoodCause"` |
| tenth-circuit-bankruptcy-appellate-panel | `edge-briefing-argument` | `deadline/extensions/1/window` | `<missing>` | `"beforeExpiry"` |
| tenth-circuit-bankruptcy-appellate-panel | `edge-briefing-argument` | `deadline/extensions/2/bound` | `<missing>` | `"discretionary"` |
| tenth-circuit-bankruptcy-appellate-panel | `edge-briefing-argument` | `deadline/extensions/2/by` | `<missing>` | `"court"` |
| tenth-circuit-bankruptcy-appellate-panel | `edge-briefing-argument` | `deadline/extensions/2/motion/event` | `<missing>` | `"PartEightDeadlineExtensionMoved()"` |
| tenth-circuit-bankruptcy-appellate-panel | `edge-briefing-argument` | `deadline/extensions/2/standard` | `<missing>` | `"ExcusableNeglect"` |
| tenth-circuit-bankruptcy-appellate-panel | `edge-briefing-argument` | `deadline/extensions/2/window` | `<missing>` | `"afterExpiryOnMotion"` |
| tenth-circuit-bankruptcy-appellate-panel | `edge-record-assembled-briefing` | `deadline/extendable` | `<missing>` | `true` |
| tenth-circuit-bankruptcy-appellate-panel | `edge-record-assembled-briefing` | `deadline/extensionAuthority` | `<missing>` | `"FRBP 9006(b)(1)"` |
| tenth-circuit-bankruptcy-appellate-panel | `edge-record-assembled-briefing` | `deadline/extensions/0/bound` | `<missing>` | `"discretionary"` |
| tenth-circuit-bankruptcy-appellate-panel | `edge-record-assembled-briefing` | `deadline/extensions/0/by` | `<missing>` | `"court"` |
| tenth-circuit-bankruptcy-appellate-panel | `edge-record-assembled-briefing` | `deadline/extensions/0/standard` | `<missing>` | `"GoodCause"` |
| tenth-circuit-bankruptcy-appellate-panel | `edge-record-assembled-briefing` | `deadline/extensions/0/window` | `<missing>` | `"beforeExpiry"` |
| tenth-circuit-bankruptcy-appellate-panel | `edge-record-assembled-briefing` | `deadline/extensions/1/bound` | `<missing>` | `"discretionary"` |
| tenth-circuit-bankruptcy-appellate-panel | `edge-record-assembled-briefing` | `deadline/extensions/1/by` | `<missing>` | `"court"` |
| tenth-circuit-bankruptcy-appellate-panel | `edge-record-assembled-briefing` | `deadline/extensions/1/motion/event` | `<missing>` | `"PartEightDeadlineExtensionMoved()"` |
| tenth-circuit-bankruptcy-appellate-panel | `edge-record-assembled-briefing` | `deadline/extensions/1/standard` | `<missing>` | `"GoodCause"` |
| tenth-circuit-bankruptcy-appellate-panel | `edge-record-assembled-briefing` | `deadline/extensions/1/window` | `<missing>` | `"beforeExpiry"` |
| tenth-circuit-bankruptcy-appellate-panel | `edge-record-assembled-briefing` | `deadline/extensions/2/bound` | `<missing>` | `"discretionary"` |
| tenth-circuit-bankruptcy-appellate-panel | `edge-record-assembled-briefing` | `deadline/extensions/2/by` | `<missing>` | `"court"` |
| tenth-circuit-bankruptcy-appellate-panel | `edge-record-assembled-briefing` | `deadline/extensions/2/motion/event` | `<missing>` | `"PartEightDeadlineExtensionMoved()"` |
| tenth-circuit-bankruptcy-appellate-panel | `edge-record-assembled-briefing` | `deadline/extensions/2/standard` | `<missing>` | `"ExcusableNeglect"` |
| tenth-circuit-bankruptcy-appellate-panel | `edge-record-assembled-briefing` | `deadline/extensions/2/window` | `<missing>` | `"afterExpiryOnMotion"` |
| tenth-circuit-bankruptcy-appellate-panel | `edge-rehearing-window-rehearing-ruling` | `deadline/extendable` | `<missing>` | `true` |
| tenth-circuit-bankruptcy-appellate-panel | `edge-rehearing-window-rehearing-ruling` | `deadline/extensionAuthority` | `<missing>` | `"FRBP 9006(b)(1)"` |
| tenth-circuit-bankruptcy-appellate-panel | `edge-rehearing-window-rehearing-ruling` | `deadline/extensions/0/bound` | `<missing>` | `"discretionary"` |
| tenth-circuit-bankruptcy-appellate-panel | `edge-rehearing-window-rehearing-ruling` | `deadline/extensions/0/by` | `<missing>` | `"court"` |
| tenth-circuit-bankruptcy-appellate-panel | `edge-rehearing-window-rehearing-ruling` | `deadline/extensions/0/standard` | `<missing>` | `"GoodCause"` |
| tenth-circuit-bankruptcy-appellate-panel | `edge-rehearing-window-rehearing-ruling` | `deadline/extensions/0/window` | `<missing>` | `"beforeExpiry"` |
| tenth-circuit-bankruptcy-appellate-panel | `edge-rehearing-window-rehearing-ruling` | `deadline/extensions/1/bound` | `<missing>` | `"discretionary"` |
| tenth-circuit-bankruptcy-appellate-panel | `edge-rehearing-window-rehearing-ruling` | `deadline/extensions/1/by` | `<missing>` | `"court"` |
| tenth-circuit-bankruptcy-appellate-panel | `edge-rehearing-window-rehearing-ruling` | `deadline/extensions/1/motion/event` | `<missing>` | `"PartEightDeadlineExtensionMoved()"` |
| tenth-circuit-bankruptcy-appellate-panel | `edge-rehearing-window-rehearing-ruling` | `deadline/extensions/1/standard` | `<missing>` | `"GoodCause"` |
| tenth-circuit-bankruptcy-appellate-panel | `edge-rehearing-window-rehearing-ruling` | `deadline/extensions/1/window` | `<missing>` | `"beforeExpiry"` |
| tenth-circuit-bankruptcy-appellate-panel | `edge-rehearing-window-rehearing-ruling` | `deadline/extensions/2/bound` | `<missing>` | `"discretionary"` |
| tenth-circuit-bankruptcy-appellate-panel | `edge-rehearing-window-rehearing-ruling` | `deadline/extensions/2/by` | `<missing>` | `"court"` |
| tenth-circuit-bankruptcy-appellate-panel | `edge-rehearing-window-rehearing-ruling` | `deadline/extensions/2/motion/event` | `<missing>` | `"PartEightDeadlineExtensionMoved()"` |
| tenth-circuit-bankruptcy-appellate-panel | `edge-rehearing-window-rehearing-ruling` | `deadline/extensions/2/standard` | `<missing>` | `"ExcusableNeglect"` |
| tenth-circuit-bankruptcy-appellate-panel | `edge-rehearing-window-rehearing-ruling` | `deadline/extensions/2/window` | `<missing>` | `"afterExpiryOnMotion"` |

## Validation and tests

- Release build: `cargo build --release` exited 0 before edits (12m21s).
- Strict checks: 31 baseline JSON outputs saved under `/tmp/lg-pb1fe-34/before`; 31 after outputs under `/tmp/lg-pb1fe-34/after`. No after-check has an error other than the recorded L5002 findings.
- `litigate test` on edited packages: `frbp`, `bankr-cas`, `bankr-or`, and `bankr-ut` each exited 0 with zero failed tests.
- `just lit-sync` completed; the lock and clock regeneration was also rerun using the release binary after formatting.
- `just lit-sync-check` exited 0 after the authoring and lock commit; the final amendment changed only this report. All `Lit.lock` files and shared clock fragments matched their sources, and no generated file was tracked.
- `target/release/litigate --format json check lit cases` returned `ok: true` with 0 error diagnostics (3495 diagnostics total).
