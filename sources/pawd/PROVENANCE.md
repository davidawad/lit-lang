
### `pawd/local-rules-2016-11-01.txt` — W.D. Pa. Local Rules of Court

- Source: https://www.pawd.uscourts.gov/sites/pawd/files/lrmanual20181101.pdf (official site; 130 pages, sha256 2c19f7f9d8862a403a87f210a9f0dd8631ca2a9f24e6eec83b8dc750daeaa6fd).
- Edition: Local Rules of Court effective November 1, 2016, as linked "PAWD Local Rules of Court Effective: November 1, 2016" on https://www.pawd.uscourts.gov/court-info/local-rules-and-orders/local-rules (retrieved 2026-10-27). Proposed amendments (Administrative Order 2026-17, Sept. 11, 2026; comments due Oct. 30, 2026) are not in force and not vendored.
- Also the criminal local rules (lg-gddt): 'LOCAL CRIMINAL RULES OF COURT' part, from extract line 4550.
- Criminal forum (lg-gddt): pawd-criminal.
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 1663a9e985ed9aaf8286d55a1a381dd5a89b62bc7da57d320c5ad181b8f56667. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `pawd/original/local-rules-2016-11-01.pdf` as served (sha256 2c19f7f9d8862a403a87f210a9f0dd8631ca2a9f24e6eec83b8dc750daeaa6fd); extraction sha256 1663a9e985ed9aaf8286d55a1a381dd5a89b62bc7da57d320c5ad181b8f56667.

### `pawd/local-rules-2016-11-01-sectioned.txt` — W.D. Pa. Local Rules of Court, sectioned

- Derived in the pod (lg-f3q8.75) from `pawd/local-rules-2016-11-01.txt` (extract sha256 1663a9e985ed9aaf8286d55a1a381dd5a89b62bc7da57d320c5ad181b8f56667); nothing fetched. One `## LCvR <n>` line is inserted right after each Local Civil Rule's own body heading line `LCvR <n>   <TITLE>` (49 markers, LCvR 1.1 through LCvR 2255; the table of contents before the first rule is unmarked). Rule text was not edited: deleting the marker lines (`## LCvR <n>` plus its newline) restores the extract byte for byte.
- Edition: as the extract, the Local Rules of Court effective November 1, 2016.
- Sectioned sha256 23bcd5fc802d152052872156a53622c92185010294a4b0187855e37168f5f175 (the `path_sha256` pinned by `lit/pawd/Lit.toml`).
