
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
- Edition: as the extract, the full-text Local Civil Rules file "Civil Rules 4-10-2026" (newest revision LR 3.16, effective 4/10/26).
- Sectioned sha256 2c85daad3e8f82924b5084fa72e52d32b0be0e4cad63b2b128f14df0c35a6713 (the `path_sha256` pinned by `lit/ohnd/Lit.toml`).
