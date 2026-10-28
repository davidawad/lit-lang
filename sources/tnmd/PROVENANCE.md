
### `tnmd/tnmd-local-rules.txt` — Local Rules of Court, M.D. Tennessee

- Source: https://www.tnmd.uscourts.gov/sites/tnmd/files/2025.05.15%20TNMD%20Local%20Rules%20of%20Court%20-%202025%20amendments%20_0.pdf (official site; 52 pages, sha256 9ea5870a79cced04483aaac496ccd8bc30ebee5ecd8ce46dec00fb62dad51946).
- Edition: M.D. Tenn. Local Rules of Court, "May 15, 2025" (2025 amendments) as stated on the document; the only local-rules PDF on https://www.tnmd.uscourts.gov/court-info/local-rules-and-orders/local-rules. (The /court-info/local-rules-and-orders path itself answers a Drupal access-denied 403 page; it is a permissions page, not a bot challenge).
- Also the criminal local rules (lg-gddt): 'LCrR 1.01 – LOCAL CRIMINAL RULES AND CM/ECF REGISTRATION.' part, from extract line 1768.
- Criminal forum (lg-gddt): tnmd-criminal.
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Fetched via curl (single document, first request succeeded, no block encountered).
- Extract sha256 36eb8d750d008481fd72c29db650eca85131bd7b5711d413488423e6d7a48b89. Raw extract, not yet sectioned into `## <ref>` form.

- Fetch method: first fetched with curl and a desktop browser User-Agent before the no-browser-identity rule (PM 2026-10-27); re-fetched on 2026-10-27 with plain curl (default User-Agent, no browser headers): byte-identical (same sha256).
- Original: `tnmd/original/tnmd-local-rules.pdf` as served (sha256 9ea5870a79cced04483aaac496ccd8bc30ebee5ecd8ce46dec00fb62dad51946); extraction sha256 36eb8d750d008481fd72c29db650eca85131bd7b5711d413488423e6d7a48b89.

### `tnmd/tnmd-local-rules-sectioned.txt` — M.D. Tenn. Local Rules of Court, sectioned

- Derived in the pod (lg-f3q8.62) from `tnmd/tnmd-local-rules.txt` (extract sha256 36eb8d750d008481fd72c29db650eca85131bd7b5711d413488423e6d7a48b89); nothing fetched. One `## LR <n>` line is inserted before each civil rule's own heading line in the body (`LR 1.01` through `LR 83.08`, 49 rules); the table of contents and the Local Criminal Rules (LCrR) are unmarked. Rule text was not edited: deleting the marker lines (`sed '/^## LR /d'`) restores the extract byte for byte.
- Upstream: https://www.tnmd.uscourts.gov/sites/tnmd/files/2025.05.15%20TNMD%20Local%20Rules%20of%20Court%20-%202025%20amendments%20_0.pdf, retrieved 2026-10-27, as recorded on the `tnmd/tnmd-local-rules.txt` entry this file is derived from.
- Edition: as the extract, "May 15, 2025" (2025 amendments).
- Sectioned sha256 aab56dd728558f4fdedead2aa054c568d185148b526c5526e5726ba0dd90d053 (the `path_sha256` pinned by `lit/tnmd/Lit.toml`).

### `tnmd/tnmd-local-criminal-rules-sectioned.txt` — M.D. Tenn. Local Criminal Rules, sectioned
- Derived in the pod (lg-gddt.16) from `tnmd/tnmd-local-rules.txt` (extract sha256 36eb8d750d008481fd72c29db650eca85131bd7b5711d413488423e6d7a48b89); nothing fetched. One `## LCrR <n>` line is inserted right before each Local Criminal Rule's own heading line (`LCrR <n> – <TITLE>`, extract lines 1727-2155): 21 markers, LCrR 1.01 through LCrR 58.03 (58.03's section runs to the end of the file). The table of contents and the civil Local Rules are unmarked (those are sectioned separately in `tnmd-local-rules-sectioned.txt`). Rule text was not edited: deleting the marker lines (`sed '/^## LCrR /d'`) restores the extract byte for byte.
- Edition: as the extract, "May 15, 2025" (2025 amendments).
- Sectioned sha256 a864f9c267fe7de976129b676c0fb0ea7cb4356175e12dedefd8e0c3062b85fc (the `path_sha256` pinned by `lit/tnmd-criminal/Lit.toml`).
