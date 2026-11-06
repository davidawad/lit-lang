# Strict-profile source amendment findings in `lit/core`

Audit date: 2026-11-06. The previous release CLI check reported 3 L5002 holes. Official public sources were fetched/consulted for the three cited state-law provisions, and their amendment histories are now pinned in `sources/state-rule-amendment-history.txt`, bound only by `lit/core/Lit.toml`.

## Search performed

- Hawaii: fetched the official Hawaii Legislature current HRS § 8-1 page and 2001 Session Laws Act 65. The section's official amendment credit ends with L 2001, c 65, § 1; Act 65 states an effective date of July 1, 2001 and approval April 26, 2001. Also checked introduced 2024 HB 1780, which proposed a change but is not treated as enacted law.
- New Mexico: fetched/consulted NMOneSource's current Rule Set 1 text, the New Mexico Supreme Court's official Rule 1-006 PDF, 2024 approved-amendments index, and Order S-1-RCR-2023-00046. The rule's history note records 2014 and 2024 amendments; the latter is effective December 31, 2024 and was approved November 1, 2024.
- Utah: fetched/consulted the Utah Courts' approved-amendments index and signed February 14, 2024 Supreme Court order adopting Rule 6 effective May 1, 2024, and inspected the official Rule 6 page/current rule text. Its effective date remains May 1, 2024.
- No later amendments to the cited text appear in the official current texts and amendment indexes through the respective source bindings' `as_of` dates. The pinned records and source URLs are documented in `sources/PROVENANCE.md`; no amendment or date was inferred.

## Resolved findings

| # | Diagnostic location | Cited provision | Proof begins | Bound source | Finding |
|---:|---|---|---|---|---|
| 1 | `lit/core/states/hawaii.lit:11` | HRS § 8-1 | 2023-01-01 | `hawaii-holidays` → `sources/hi-holidays/hrs-8-1.txt` | Resolved by pinned `## versions HRS § 8-1`; L 2001, c 65, § 1 is the last credited amendment and predates the cited version. |
| 2 | `lit/core/states/new-mexico.lit:16` | NMRA 1-006(A)(7) | 2024-12-31 | `nmra-1` → `sources/nm-state-civil/nmra-rule-set-1-civil-2026-sectioned.txt` | Resolved by pinned `## versions NMRA 1-006`; Order S-1-RCR-2023-00046 is effective on the cited version's first day. |
| 3 | `lit/core/states/utah.lit:17` | URCP 6(a)(6) | 2024-05-01 | `urcp` → `sources/ut-state-civil/utah-urcp-all-rules-sectioned.txt` | Resolved by pinned `## versions URCP 6`; the Utah Supreme Court's order makes the cited version effective May 1, 2024. |

Verification: `cargo build --release -p litigate-cli` exited 0; `target/release/litigate check --profile strict lit/core` exited 0 with 0 holes; `cargo build -p litgraph-packs` exited 0; `cargo nextest run -p litigate-cli --test core_stdlib` exited 0 (17 passed, 0 failed).
