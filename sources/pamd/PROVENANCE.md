
### `pamd/local-rules-2014-12-01.txt` — M.D. Pa. Rules of Court

- Source: https://www.pamd.uscourts.gov/sites/pamd/files/LR120114.pdf (official site; 112 pages, sha256 dc93f968e24b935be49f6f3bdcac3febaaec812442deb571f48b547e9deb9611).
- Edition: Rules of Court effective December 1, 2014, as linked "Local Rules PDF" on https://www.pamd.uscourts.gov/court-info/local-rules-and-orders, which states "These Local Rules were last amended December 1, 2014" (retrieved 2026-10-27).
- Also the criminal local rules (lg-gddt): 'CRIMINAL RULES' part, from extract line 3143.
- Criminal forum (lg-gddt): pamd-criminal.
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 400854aa7149685014178ca71c57d6f0a695ed9040a5a61bdbf890c4e64d2869. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `pamd/original/local-rules-2014-12-01.pdf` as served (sha256 dc93f968e24b935be49f6f3bdcac3febaaec812442deb571f48b547e9deb9611); extraction sha256 400854aa7149685014178ca71c57d6f0a695ed9040a5a61bdbf890c4e64d2869.

### `pamd/local-rules-2014-12-01-sectioned.txt` — M.D. Pa. Rules of Court, sectioned

- Derived in the pod (lg-f3q8.75) from `pamd/local-rules-2014-12-01.txt` (extract sha256 400854aa7149685014178ca71c57d6f0a695ed9040a5a61bdbf890c4e64d2869); nothing fetched. One `## LR <n>` line is inserted right after each rule's own body heading line `LR <n> <title>.` (246 markers, LR 1.1 through LR 83.40.7; the table of contents before the first rule is unmarked). The extract repeats the heading of LR 83.8, so it carries two `## LR 83.8` markers. Rule text was not edited: deleting the marker lines (`## LR <n>` plus its newline) restores the extract byte for byte.
- Source: as the extract `pamd/local-rules-2014-12-01.txt`, https://www.pamd.uscourts.gov/sites/pamd/files/LR120114.pdf, retrieved 2026-10-27; the sectioning was done in the pod from the vendored extract.
- Edition: as the extract, the Rules of Court effective December 1, 2014.
- Criminal rules (lg-gddt.10): one `## LCrR <n>` line is also inserted right after each criminal rule's own body heading line `LCrR <n> <title>` in the 'CRIMINAL RULES' part: 9 markers, LCrR 7.1 through LCrR 58.3 (the criminal part of the table of contents is unmarked). Deleting both kinds of marker line (`## LR <n>` and `## LCrR <n>`, each plus its newline) restores the extract byte for byte.
- Sectioned sha256 c85ec6d3297d11f3ccfb6fbb76d453fe11c4786c04305577d1e62f02dbfb4d0a (the `path_sha256` pinned by `lit/pamd/Lit.toml` and `lit/pamd-criminal/Lit.toml`).
