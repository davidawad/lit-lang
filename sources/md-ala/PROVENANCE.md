
### `md-ala/local-rules.txt` — M.D. Ala. Local Rules

- Source: https://www.almd.uscourts.gov/sites/default/files/forms/ALMD%20Local%20Rules.pdf (official site; 46 pages, sha256 39ccde043f3fccdb65241d8ac8eb53deec77a13fc7a6d2a17f5f7e7a94423377).
- Edition: Effective December 1, 2010, amended October 1, 2011 (Local Rule 83.4(g)) (cover); listed with "Effective Date: December 1, 2010" on https://www.almd.uscourts.gov/forms/local-rules; fetched with curl (plain User-Agent), no bot refusal; site publishes no terms page (privacy-policy 404), robots.txt allows.
- Also the criminal local rules (lg-gddt): 'PART 2. CRIMINAL RULES' part, from extract line 117.
- Criminal forum (lg-gddt): almd-criminal.
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 4b3e8ab65153e8cab5e18720e682d113b0ec27021d139173c31debec020f66a1. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `md-ala/original/local-rules.pdf` as served (sha256 39ccde043f3fccdb65241d8ac8eb53deec77a13fc7a6d2a17f5f7e7a94423377); extraction sha256 4b3e8ab65153e8cab5e18720e682d113b0ec27021d139173c31debec020f66a1.

### `md-ala/local-rules-civil-sectioned.txt` — M.D. Ala. Local Rules, civil rules sectioned

- Derived in the pod (lg-f3q8.12) from `md-ala/local-rules.txt` (extract sha256 4b3e8ab65153e8cab5e18720e682d113b0ec27021d139173c31debec020f66a1); nothing fetched. One `## LR <n>` line is inserted before each `Local Rule <n>` heading line of Part 1 (`LR 1.1` through `LR 83.5`, 43 rules). The criminal rules (Part 2) are unmarked. Rule text was not edited: deleting the marker lines (`sed '/^## LR /d'`) restores the extract byte for byte.
- Upstream: https://www.almd.uscourts.gov/sites/default/files/forms/ALMD%20Local%20Rules.pdf, retrieved 2026-10-27, as recorded on the `md-ala/local-rules.txt` entry this file is derived from.
- Edition: as the extract, effective December 1, 2010, amended October 1, 2011.
- Sectioned sha256 80f4cb4eead6ae8fd5fe07406ec03322601fc9a14bfe3b0f33c566dc366100dd (the `path_sha256` pinned by `lit/almd/Lit.toml`).

### `md-ala/local-rules-sectioned.txt` — M.D. Ala. Local Rules, sectioned

- Derived in the pod (lg-gddt.26) from `md-ala/local-rules.txt` (extract sha256 4b3e8ab65153e8cab5e18720e682d113b0ec27021d139173c31debec020f66a1); nothing fetched. One `## LR <n>` line is inserted before each civil and general rule's own heading line in the body (`Local Rule 1.1` through `Local Rule 83.5`, 42 rules) and one `## LCrR <n>` line before each criminal rule's (`Local Criminal Rule 1.1` through `Local Criminal Rule 58.1`, 8 rules), after the rules' own citation forms "M.D. Ala. LR" and "M.D. Ala. LCrR" (LR 1.1(a), LCrR 1.1(a)); the table of rules and the index are unmarked. Rule text was not edited: deleting the marker lines (`sed '/^## L/d'`) restores the extract byte for byte.
- Source: as the extract `md-ala/local-rules.txt`, https://www.almd.uscourts.gov/sites/default/files/forms/ALMD%20Local%20Rules.pdf, retrieved 2026-10-27; the sectioning was done in the pod from the vendored extract.
- Edition: as the extract, effective December 1, 2010, amended October 1, 2011 (Local Rule 83.4(g)).
- Sectioned sha256 f3cbd555562a18710065f814741dead65eb99a6b958f5b50914d172d8e785ca4 (the `path_sha256` pinned by `lit/almd-criminal/Lit.toml`).
