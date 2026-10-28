
### `ohnd/ohnd-local-civil-rules.txt` — Local Civil Rules, N.D. Ohio

- Source: https://www.ohnd.uscourts.gov/sites/ohnd/files/Civil%20Rules%204-10-2026.pdf (official site; 193 pages, sha256 e2a67873799efcb70df8b6ba43b8d850a786a65321863f2d5ff2cb89250f3335).
- Edition: N.D. Ohio Local Civil Rules, full-text PDF file named "Civil Rules 4-10-2026" (the document itself shows only the 1992 base date and a revision list), linked as "To read the Local Civil Rules in full, click HERE" from https://www.ohnd.uscourts.gov/local-civil-rules (local criminal and patent rules are separate documents, not vendored).
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Fetched via curl (single document, first request succeeded, no block encountered).
- Extract sha256 312cece0863e466dff97942a2163040eb2129ba106b0287ff109548f7e92fc47. Raw extract, not yet sectioned into `## <ref>` form.

- Fetch method: first fetched with curl and a desktop browser User-Agent before the no-browser-identity rule (PM 2026-10-27); re-fetched on 2026-10-27 with plain curl (default User-Agent, no browser headers): byte-identical (same sha256).
- Original: `ohnd/original/ohnd-local-civil-rules.pdf` as served (sha256 e2a67873799efcb70df8b6ba43b8d850a786a65321863f2d5ff2cb89250f3335); extraction sha256 312cece0863e466dff97942a2163040eb2129ba106b0287ff109548f7e92fc47.

### `ohnd/ohnd-local-civil-rules-sectioned.txt` — N.D. Ohio Local Civil Rules, sectioned

- Derived in the pod (lg-f3q8.61) from `ohnd/ohnd-local-civil-rules.txt` (extract sha256 312cece0863e466dff97942a2163040eb2129ba106b0287ff109548f7e92fc47); nothing fetched. One `## LR <n>` line is inserted right after each rule's own body heading line `Rule <n> <title>` (79 markers, LR 1.1 through LR 83.10; the table of contents and historical notes before the first rule are unmarked). Rule text was not edited: deleting the marker lines (`sed '/^## LR /d'`) restores the extract byte for byte.
- Source: as the extract `ohnd/ohnd-local-civil-rules.txt`, https://www.ohnd.uscourts.gov/sites/ohnd/files/Civil%20Rules%204-10-2026.pdf, retrieved 2026-10-27; the sectioning was done in the pod from the vendored extract.
- Edition: as the extract, the full-text Local Civil Rules file "Civil Rules 4-10-2026" (newest revision LR 3.16, effective 4/10/26).
- Sectioned sha256 2c85daad3e8f82924b5084fa72e52d32b0be0e4cad63b2b128f14df0c35a6713 (the `path_sha256` pinned by `lit/ohnd/Lit.toml`).

### `ohnd/ohnd-local-criminal-rules-2024-10-01.txt` — N.D. Ohio Local Criminal Rules (full set, file dated 10-1-2024)

- Source: https://www.ohnd.uscourts.gov/sites/ohnd/files/Full_Criminal_Rules_10-1-2024.pdf (official site; 89 pages, sha256 cb7a73f4f7963e4935b64a6346acbb11c5229042e75f1ee2eac3f7421d77ff85).
- Edition: Full Criminal Rules, the PDF 'Full_Criminal_Rules_10-1-2024' linked at https://www.ohnd.uscourts.gov/local-criminal-rules; later single-rule redlines on https://www.ohnd.uscourts.gov/court-info/local-rules-and-orders (for example Local Criminal Rule 17.1.1) are not vendored. robots.txt of www.ohnd.uscourts.gov read (Crawl-delay 10 honored where stated; no Disallow covers the fetched path); the judiciary-wide Privacy & Security Policy (https://www.uscourts.gov/privacy-security-policy) read: no prohibition of automated access. Fetched with curl and the identifying UA lit-lang-provenance/1.0, at most 1 request per 10 s.
- Criminal forum (lg-gddt): ohnd-criminal.
- Retrieved 2026-10-28 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 8c27b13ef28a0e2e96cc6c051406ac6acb47e94b5d40b2ffca263c10515fb9d8. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `ohnd/original/ohnd-local-criminal-rules-2024-10-01.pdf` as served (sha256 cb7a73f4f7963e4935b64a6346acbb11c5229042e75f1ee2eac3f7421d77ff85); extraction sha256 8c27b13ef28a0e2e96cc6c051406ac6acb47e94b5d40b2ffca263c10515fb9d8.

### `ohnd/ohnd-local-criminal-rules-2024-10-01-sectioned.txt` — N.D. Ohio Local Criminal Rules, sectioned
- Derived in the pod (lg-gddt.16) from `ohnd/ohnd-local-criminal-rules-2024-10-01.txt` (extract sha256 8c27b13ef28a0e2e96cc6c051406ac6acb47e94b5d40b2ffca263c10515fb9d8); nothing fetched. One `## LCrR <n>` line is inserted right after each rule's own body heading line `Rule <n> <title>` (58 markers, LCrR 1.1 through LCrR 58.3; the table of contents is unmarked, and LCrR 58.3's section runs through the appendices to the end of the file). Rule text was not edited: deleting the marker lines (`sed '/^## LCrR /d'`) restores the extract byte for byte.
- Edition: as the extract, the full Local Criminal Rules file dated October 1, 2024.
- Sectioned sha256 05fdb1196bdf1b3ebabdaa54f8c5bf23bd0c47593d4d35f8c90102a184dac511 (the `path_sha256` pinned by `lit/ohnd-criminal/Lit.toml`).
