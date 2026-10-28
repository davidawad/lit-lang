
### `sd-ga/sdga-local-rules.txt` — S.D. Ga. Local Rules (general, civil, criminal)

- Source: https://www.gasd.uscourts.gov/sites/gasd/files/local_rules/SDGA_Local_Rules_Effective_10-07-2026.pdf (official site; 68 pages, sha256 878813a8fdeac35a2e87f4db51fd8ffff30172137f4943eeb88f3474185a0edf).
- Edition: Local Rules "Effective 10-07-2026" (PDF version linked from https://www.gasd.uscourts.gov/court-info/local-rules-and-orders).
- Also the criminal local rules (lg-gddt): 'LOCAL CRIMINAL RULES' part, from extract line 948.
- Criminal forum (lg-gddt): gasd-criminal.
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 06f9f6b1c290e7be9b1c54d37496153f0cd001f0d6912024cc72a31a7053292b. Raw extract, not yet sectioned into `## <ref>` form.

- Fetch method: first fetched with curl and a Referer header after the court page answered 403 (no bot-detection signal); re-checked 2026-10-27 in BrowserOS from the court page and with plain curl (HTTP 200, no bot signal): byte-identical (same sha256).
- Original: `sd-ga/original/sdga-local-rules.pdf` as served (sha256 878813a8fdeac35a2e87f4db51fd8ffff30172137f4943eeb88f3474185a0edf); extraction sha256 06f9f6b1c290e7be9b1c54d37496153f0cd001f0d6912024cc72a31a7053292b.

### `sd-ga/sdga-local-rules-sectioned.txt` — S.D. Ga. Local Rules, sectioned

- Derived in the pod (lg-f3q8.15) from `sd-ga/sdga-local-rules.txt` (extract sha256 06f9f6b1c290e7be9b1c54d37496153f0cd001f0d6912024cc72a31a7053292b); nothing fetched. In the body (after the table of contents), one `## LR <Gen.|Civ.|Crim.> <n>` line is inserted before each rule's own heading line (`LR Gen. 1A. Scope, Effective Date, and Modification.` through `LR Crim. 58. Forfeiture of Collateral.`, 53 rules; the heading's trailing period is dropped in the marker). Every other line is the extract unchanged, so deleting the `## LR` lines gives back the extract byte for byte.
- Upstream: https://www.gasd.uscourts.gov/sites/gasd/files/local_rules/SDGA_Local_Rules_Effective_10-07-2026.pdf, retrieved 2026-10-27, as recorded on the `sd-ga/sdga-local-rules.txt` entry this file is derived from.
- Edition: as the extract, Local Rules "Effective 10-07-2026" (LR Gen. 1A: "These Local Rules take effect on October 7, 2026").
- Sectioned sha256 6f5214a2bb86045dcae24edf5172748170dd920f2478c5c5f0240f4bd56c5f7b (the `path_sha256` pinned by `lit/gasd/Lit.toml`).

- Derived in the pod (lg-gddt.26) from `sd-ga/sdga-local-rules.txt` (extract sha256 06f9f6b1c290e7be9b1c54d37496153f0cd001f0d6912024cc72a31a7053292b); nothing fetched. One `## LR Gen. <n>`, `## LR Civ. <n>` or `## LR Crim. <n>` line is inserted before each rule's own heading line in the body (`LR Gen. 1A.` through `LR Crim. 58.`, 53 rules, the same 53 the table of contents lists); the table of contents is unmarked. Rule text was not edited: deleting the marker lines (`sed '/^## LR \(Crim\|Civ\|Gen\)\. /d'`) restores the extract byte for byte.
- Source: as the extract `sd-ga/sdga-local-rules.txt`, https://www.gasd.uscourts.gov/sites/gasd/files/local_rules/SDGA_Local_Rules_Effective_10-07-2026.pdf, retrieved 2026-10-27; the sectioning was done in the pod from the vendored extract.
- Edition: as the extract, Local Rules effective October 7, 2026.
- Sectioned sha256 6f5214a2bb86045dcae24edf5172748170dd920f2478c5c5f0240f4bd56c5f7b (the `path_sha256` pinned by `lit/gasd-criminal/Lit.toml`).
