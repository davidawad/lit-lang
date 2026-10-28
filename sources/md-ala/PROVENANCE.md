
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
- Edition: as the extract, effective December 1, 2010, amended October 1, 2011.
- Sectioned sha256 80f4cb4eead6ae8fd5fe07406ec03322601fc9a14bfe3b0f33c566dc366100dd (the `path_sha256` pinned by `lit/almd/Lit.toml`).
