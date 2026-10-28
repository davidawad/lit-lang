
### `md-ga/mdga-local-rules.txt` — M.D. Ga. Local Rules

- Source: https://www.gamd.uscourts.gov/sites/gamd/files/Local_Rules-12_1_2022.pdf (official site; 71 pages, sha256 6a38fa2a50e3cc6a9daea03964a09d50926a8bfd1dd2b186203c2bacbc6401f9).
- Edition: Local Rules "As Amended March 7, 2022" (most recent amendment listed on https://www.gamd.uscourts.gov/local-rules).
- Also the criminal local rules (lg-gddt): 'LOCAL CRIMINAL RULES' part, from extract line 2781.
- Criminal forum (lg-gddt): gamd-criminal.
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 e1b087a01c5f9afb41918ac8728a25ebfefde6b2c1783875c0a60e5fa5dd93a6. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `md-ga/original/mdga-local-rules.pdf` as served (sha256 6a38fa2a50e3cc6a9daea03964a09d50926a8bfd1dd2b186203c2bacbc6401f9); extraction sha256 e1b087a01c5f9afb41918ac8728a25ebfefde6b2c1783875c0a60e5fa5dd93a6.

### `md-ga/mdga-local-rules-sectioned.txt` — M.D. Ga. Local Rules, sectioned

- Derived in the pod (lg-f3q8.15) from `md-ga/mdga-local-rules.txt` (extract sha256 e1b087a01c5f9afb41918ac8728a25ebfefde6b2c1783875c0a60e5fa5dd93a6); nothing fetched. In the body (from `LOCAL RULE 3: DIVISIONS OF THE COURT` on), one `## L.R. <n>` line is inserted before each `LOCAL RULE <n>` heading line and before each numbered subrule heading line (`<n>.<m>` followed by its capitalized title, e.g. `7.2 RESPONSE.`, `54.2.2 TIME FOR FILING.`), 94 headings, the criminal rules included; the table of contents is unmarked. Every other line is the extract unchanged, so deleting the `## L.R.` lines gives back the extract byte for byte.
- Upstream: https://www.gamd.uscourts.gov/sites/gamd/files/Local_Rules-12_1_2022.pdf, retrieved 2026-10-27, as recorded on the `md-ga/mdga-local-rules.txt` entry this file is derived from.
- Edition: as the extract, Local Rules "As Amended March 7, 2022".
- Sectioned sha256 bf31258f6359b4b9827bd9408addcb97f3c24e6de452a80322a5ba9d18db6976 (the `path_sha256` pinned by `lit/gamd/Lit.toml`).
