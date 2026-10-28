
### `gu-superior/guam-rules-of-civil-procedure.txt` — Guam Rules of Civil Procedure

- Source: https://www.guamcourts.gov/sites/default/files/GRCP%2020240401_Control.pdf (official site; 123 pages, sha256 aa44d5eab6f59a8f2effbab6bd9171d843f10ee4d784280ab0da9e82ada67b98).
- Edition: Guam Rules of Civil Procedure, "Amended pursuant to PRM 06-006-24, effective October 19, 2023" (document dated March 31, 2024), listed as the Guam Rules of Civil Procedure at https://www.guamcourts.gov/courts-council/superior-court/rules-procedure; the Supreme Court promulgation-orders page lists no later amendment. Fetched with curl (robots.txt allows).
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 5fab8b20adc791c08a309457660e423416ce624fb2984065b1cb91b40ff65f5d. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `gu-superior/original/guam-rules-of-civil-procedure.pdf` as served (sha256 aa44d5eab6f59a8f2effbab6bd9171d843f10ee4d784280ab0da9e82ada67b98); extraction sha256 5fab8b20adc791c08a309457660e423416ce624fb2984065b1cb91b40ff65f5d.

### `gu-superior/guam-rules-of-civil-procedure-sectioned.txt` — Guam Rules of Civil Procedure, sectioned

- Derived in the pod (lg-f3q8.41) from `gu-superior/guam-rules-of-civil-procedure.txt` (extract sha256 5fab8b20adc791c08a309457660e423416ce624fb2984065b1cb91b40ff65f5d); nothing fetched. One `## Rule <n>` line is inserted before each rule's own heading line at the left margin (`Rule 1.` through `Rule 93.`, 99 rules including 4.1, 23.1, 23.2, 44.1, 65.1 and 71A); the table of rules and the indented headings quoted in compiler's notes are unmarked. Rule text was not edited: deleting the marker lines (`sed '/^## Rule /d'`) restores the extract byte for byte.
- Source: as the extract `gu-superior/guam-rules-of-civil-procedure.txt`, https://www.guamcourts.gov/sites/default/files/GRCP%2020240401_Control.pdf, retrieved 2026-10-27; the sectioning was done in the pod from the vendored extract.
- Edition: as the extract, amended pursuant to PRM 06-006-24, effective October 19, 2023. A section holds the rule's text followed by its SOURCE line and any compiler's notes quoting superseded text; only the current text is cited.
- Sectioned sha256 aef8c8a414a1fa4a376339159811f8f2fd0694b150e34d8a9c3d4df71ec50541 (the `path_sha256` pinned by `lit/gu-superior/Lit.toml`).
