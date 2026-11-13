# lg-pb1fe.58 — engine red fixes

`origin/main` was `31d7666fb3f236a33cdd400219d3957fad15db54` at Step 0; `30d2114ec` is an ancestor. Rebased the previous attempt onto this commit, resolving `lit/frcp/Lit.lock` by taking upstream's row and regenerating it from source. The initial `cargo nextest run -p litigate-engine --no-fail-fast` result on the previous attempt was 648 passed, 1 skipped, 1 failed: `calibration::every_embedded_calibration_entry_resolves`. The source provenance test passed after the six provenance records were repaired.

DECIDED: Preserve the authored defendant summary-judgment edge reference and resolve calibration refs by base edge id -- the calibration describes that outcome, while the compiled graph retains it only as state-flag copies.

## Source provenance

The six text digests below are recorded in `sources/PROVENANCE.md`. Each extraction was compared to the cited as-served original; no source text needed correction.

| Text | SHA-256 recorded | Original comparison |
|---|---|---|
| `sources/cfr-37-part2-answer-time-history.txt` | `5a83643d57d5ce150c3e2af7a9bb73afa7ae5fa34ee671001b5db9f66c0f7d2c` | `sources/original/cfr-37-part2-answer-time-history/001-CFR-2016-title37-vol1-part2.xml:240-241`; § 2.116 reads “Federal Rules of Civil Procedure.” The added version history is separately sourced to the eCFR API original, whose title reads “§ 2.116   Federal Rules of Civil Procedure.” |
| `sources/ecfr-versions-title-19.txt` | `d04afe302737f0904deb0595bb720432fc5b5825a861e5588253c36ae1014dce` | The added § 210.6 block matches `sources/original/ecfr-versions-title-19/018-api-versioner-v1-versions-title-19.json_section-210.6.json:1`; its name sentence is “§ 210.6   Computation of time, additional hearings, postponements, continuances, and extensions of time.” |
| `sources/ecfr-versions-title-37.txt` | `996b9ca2890196301e015f252bd593ad74efe4dfe13ddc367d11a984db714369` | The added § 2.116 version list matches `sources/original/ecfr-versions-title-37/060-api-versioner-v1-versions-title-37.json_section-2.116.json:1`; its title reads “§ 2.116   Federal Rules of Civil Procedure.” |
| `sources/usc-28-2000-frap-rule26.txt` | `7722e4a1467ea16820efb8cd340f4550d46142fe57d85a1971468b9f865aaca4` | `sources/original/usc-28-2000-frap-rule26.html:31` includes “(As amended Mar. 1, 1971, eff. July 1, 1971; … Apr. 24, 1998, eff. Dec. 1, 1998.)” verbatim. |
| `sources/usc-28-2002-frap-rule26.txt` | `8ae71f441bd39ec678a8af7619d05b04f1585def3031aa4ff597f9fbee336309` | `sources/original/usc-28-2002-frap-rule26.html:31` carries the same amendment credit, adding “Apr. 29, 2002, eff. Dec. 1, 2002.” |
| `sources/usc-28-2008-frap-rule26.txt` | `2133d8ea75e56413d8efe5ad058fffba2d27526d74b5c3e23437d2576ed42188` | `sources/original/usc-28-2008-frap-rule26.html:31` carries the 2002 credit and adds “Apr. 25, 2005, eff. Dec. 1, 2005.” |

For the three eCFR histories, provenance records the API URL, retrieval date, as-served response hash, and the text extraction method (response blocks copied byte for byte). The FRAP extractions retain the full Rule 26 statutory body and amendment-credit line; provenance records the official GovInfo edition URLs and whitespace/entity normalization. The CFR Part 2 history provenance records the 2016 and 2017 annual CFR originals and explains that the 2017 version list is eCFR history metadata while the rule heading remains unchanged.

## Calibration

The skipped `frcp-fy2024` entry named `frcp-civil-procedure::edge-sj-ruling-summary-judgment-for-defendant`. The compiled graph has the defendant edges `frcp-civil-procedure::edge-sj-ruling-summary-judgment-for-defendant{sj-by-defendant}` and `frcp-civil-procedure::edge-sj-ruling-summary-judgment-for-defendant{sj-by-defendant,waived-12h-defenses}`; it has no unflagged edge with the authored reference. `Graph::edge_family_by_base_id` now resolves those copies when the exact unflagged id is absent. The same lookup applies to duration and probability calibration; an id with no copies remains skipped. The `.lit` and JSON fixture retain the original defendant edge ref. No number, sample size, source, or method sentence changed, and no solve-value fixture moved.

The matching `crates/litigate-cli/tests/litgraph-final/calibration/frcp-fy2024.json` fixture was updated in the fourth entry's `ref` field only. Numeric fields (`mode: 410.91`, `n: 21861`) and every other field are unchanged; therefore no solve-value fixture moved.

## Documentation correction

`docs/reports/lg-pb1fe-blended-itc.md` now attributes the amended R2 to the PM's ruling on `lg-pb1fe.53`.

## Checks

- `cargo nextest run -p litigate-engine --no-fail-fast`: exit 0, 650 passed, 1 skipped. Includes `calibration::every_embedded_calibration_entry_resolves`, `source_provenance::every_vendored_text_has_its_sha256_recorded`, and `base_id_calibration_reaches_flagged_only_edge_copies`.
- `just lit-sync`: exit 0; `just lit-sync-check`: exit 0, `Lit.lock` files and shared clock current, no generated file committed. The upstream FRCP lock row already matched the restored source reference, so regeneration produced no `Lit.lock` diff.
- `cargo nextest run -p litigate-engine -p litigate-cli --no-fail-fast`: exit 0, 1,319 passed, 1 skipped, no failures (18 slow tests).
- `cargo clippy --workspace --all-targets -- -D warnings`: exit 0 after applying the lint suggestions in `api/calibration/apply.rs`.
- `cargo fmt --all --check`: exit 0.
- `cargo build --release -p litigate-cli`: exit 0, finished in 8m03s.
- `target/release/litigate check lit cases`: exit 0, `ok: true`, no errors.
- `target/release/litigate test lit/frcp`: exit 0, 51 passed, 0 failed.
- `cargo nextest run -p litigate-base --test line_ceiling`: exit 0, 3 passed, 0 skipped.
