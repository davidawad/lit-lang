
### `nc-appellate-civil/nc-rules-of-appellate-procedure.txt` — North Carolina Rules of Appellate Procedure

- Source: https://www.nccourts.gov/assets/inline-files/North%20Carolina%20Rules%20of%20Appellate%20Procedure%20-%20Codified%202%20September%202025_2.pdf (official site; 117 pages, sha256 7d224e434190ffb4faccd84e7008351bf70c6ff7048f6bcc7f2fab3149b85145).
- Edition: North Carolina Rules of Appellate Procedure, Codified 2 September 2025 (the "Current Codification" on https://www.nccourts.gov/courts/supreme-court/court-rules/north-carolina-rules-of-appellate-procedure; no later order listed).
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 2dff556de4f437f8bc06dd4b7432d32c047c3ca4a2fdf45a4b528912dc0a6b36. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `nc-appellate-civil/original/nc-rules-of-appellate-procedure.pdf` as served (sha256 7d224e434190ffb4faccd84e7008351bf70c6ff7048f6bcc7f2fab3149b85145); extraction sha256 2dff556de4f437f8bc06dd4b7432d32c047c3ca4a2fdf45a4b528912dc0a6b36.

### `nc-appellate-civil/nc-rules-of-appellate-procedure-sectioned.txt` — N.C. Rules of Appellate Procedure (Codified 2 September 2025), sectioned

- Derived in the pod (lg-f3q8.29) from `nc-appellate-civil/nc-rules-of-appellate-procedure.txt` (extract sha256 2dff556de4f437f8bc06dd4b7432d32c047c3ca4a2fdf45a4b528912dc0a6b36); nothing fetched. One `## Rule <n>` line is inserted right before each rule's own body heading line (from extract line 571 on), the line `Rule <n>. <title>` whose number and first title word match the table of contents entry: 46 markers, Rule 1 through Rule 42. The table of contents and in-body cross-references (including a line that begins `Rule 28.1. Except`) are unmarked. Rule text was not edited: deleting the marker lines (`sed '/^## Rule /d'`) restores the extract byte for byte.
- Upstream: https://www.nccourts.gov/assets/inline-files/North%20Carolina%20Rules%20of%20Appellate%20Procedure%20-%20Codified%202%20September%202025_2.pdf, retrieved 2026-10-27, as recorded on the `nc-appellate-civil/nc-rules-of-appellate-procedure.txt` entry this file is derived from.
- Edition: as the extract, the Rules codified 2 September 2025.
- Sectioned sha256 faa02aa4899f95553e42bccb296ed5dc8d4a6ec35b5509f4e0eb535648614d69 (the `path_sha256` pinned by `lit/nc-superior-civil/Lit.toml` and `lit/nc-appellate-civil/Lit.toml`).
