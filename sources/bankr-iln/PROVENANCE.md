
### `bankr-iln/ilnb-lbr-2024-09-01.txt` — Local Rules, U.S. Bankruptcy Court for the Northern District of Illinois (as amended, effective September 1, 2024)

- Source: https://www.ilnb.uscourts.gov/sites/ilnb/files/Local-Rules-9-1-2024.pdf (official site; 63 pages, sha256 7ce5a0204662e8118cda4f23cb28099806f93f0d7a8626bae29e003989dc385a).
- Edition: Local Rules 'As amended, Effective September 1, 2024' (cover), linked as 'Local Rules (As amended, effective September 1, 2024)' from https://www.ilnb.uscourts.gov/court-info/local-rules-and-orders. robots.txt (Crawl-delay 10 honored; no rule covering these paths) and the national Judiciary Privacy Policy linked from the site (uscourts.gov) checked: no prohibition of automated access. Fetched with curl and the identifying User-Agent lit-lang-provenance/1.0 (angrybohr@proton.me).
- Retrieved 2026-10-28 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 5b3edbb3c3ef0a131eccc6aade14f6ae75ae3f6aba007523d62da4b4658e87a4. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `bankr-iln/original/ilnb-lbr-2024-09-01.pdf` as served (sha256 7ce5a0204662e8118cda4f23cb28099806f93f0d7a8626bae29e003989dc385a); extraction sha256 5b3edbb3c3ef0a131eccc6aade14f6ae75ae3f6aba007523d62da4b4658e87a4.

### `bankr-iln/ilnb-lbr-2024-09-01-sectioned.txt` — N.D. Ill. Local Bankruptcy Rules, sectioned

- Derived in the pod (lg-u561.21) from `bankr-iln/ilnb-lbr-2024-09-01.txt` (extract sha256 5b3edbb3c3ef0a131eccc6aade14f6ae75ae3f6aba007523d62da4b4658e87a4); nothing fetched. One `## LBR <n>` line is inserted before each rule's body heading (`RULE <n>  <TITLE>`, after the table of contents; `LBR 1000-1` through `LBR 9090-3`, with `LBR 9029-4A`–`4C`, `LBR 9013-3` for the reserved 9013-3 through 8 and `LBR 9038`; 83 sections); the cover and the table of contents are unmarked. Rule text was not edited: deleting the marker lines (`sed '/^## LBR /d'`) restores the extract byte for byte.
- Source: as the extract, https://www.ilnb.uscourts.gov/sites/ilnb/files/Local-Rules-9-1-2024.pdf (official site), retrieved 2026-10-28 on carbon; the sectioning was done in the pod from the vendored extract.
- Edition: as the extract, as amended effective September 1, 2024.
- Sectioned sha256 012922732be9ce263ffac16fa571469adb1f6488f662b4d74a8a2eae8afcdedd (the `path_sha256` pinned by `lit/bankr-iln/Lit.toml` for source `ilnb-lbr`).
