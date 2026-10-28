
### `pamd/local-rules-2014-12-01.txt` — M.D. Pa. Rules of Court

- Source: https://www.pamd.uscourts.gov/sites/pamd/files/LR120114.pdf (official site; 112 pages, sha256 dc93f968e24b935be49f6f3bdcac3febaaec812442deb571f48b547e9deb9611).
- Edition: Rules of Court effective December 1, 2014, as linked "Local Rules PDF" on https://www.pamd.uscourts.gov/court-info/local-rules-and-orders, which states "These Local Rules were last amended December 1, 2014" (retrieved 2026-10-27).
- Also the criminal local rules (lg-gddt): 'CRIMINAL RULES' part, from extract line 3143.
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 400854aa7149685014178ca71c57d6f0a695ed9040a5a61bdbf890c4e64d2869. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `pamd/original/local-rules-2014-12-01.pdf` as served (sha256 dc93f968e24b935be49f6f3bdcac3febaaec812442deb571f48b547e9deb9611); extraction sha256 400854aa7149685014178ca71c57d6f0a695ed9040a5a61bdbf890c4e64d2869.

### `pamd/local-rules-2014-12-01-sectioned.txt` — M.D. Pa. Rules of Court, sectioned

- Derived in the pod (lg-f3q8.75) from `pamd/local-rules-2014-12-01.txt` (extract sha256 400854aa7149685014178ca71c57d6f0a695ed9040a5a61bdbf890c4e64d2869); nothing fetched. One `## LR <n>` line is inserted right after each rule's own body heading line `LR <n> <title>.` (246 markers, LR 1.1 through LR 83.40.7; the table of contents before the first rule is unmarked). The extract repeats the heading of LR 83.8, so it carries two `## LR 83.8` markers. Rule text was not edited: deleting the marker lines (`## LR <n>` plus its newline) restores the extract byte for byte.
- Edition: as the extract, the Rules of Court effective December 1, 2014.
- Sectioned sha256 a4ff8b7c6353c56408da5112bcdc10880673ed744a877e5b2bc68c73190bae0f (the `path_sha256` pinned by `lit/pamd/Lit.toml`).
