
### `mt-district/mrcp.txt` — Montana Rules of Civil Procedure (M. R. Civ. P.)

- Source: https://courts.mt.gov/external/rules/civ_proc.pdf (official site; 123 pages, sha256 7502442cb52ec026409c735276d06de6149553d5d01b8586f4610f0f14cceeda).
- Edition: Montana Rules of Civil Procedure; the document states "Date of Last Amendment to these Rules: 28JUN2022", AF 07-0157, State Law Library of Montana cover sheet; linked as "Rules of Civil Procedure" from https://courts.mt.gov/courts/rules/ (page lists no later amendments).
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 e0dfc6717462f4d262d0c9d08b0d2733e4302df3309b4a4a99133247bb89b77f. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `mt-district/original/mrcp.pdf` as served (sha256 7502442cb52ec026409c735276d06de6149553d5d01b8586f4610f0f14cceeda); extraction sha256 e0dfc6717462f4d262d0c9d08b0d2733e4302df3309b4a4a99133247bb89b77f.

### `mt-district/mrcp-sectioned.txt` — Montana Rules of Civil Procedure, sectioned

- Derived in the pod (lg-f3q8.44) from `mt-district/mrcp.txt` (extract sha256 e0dfc6717462f4d262d0c9d08b0d2733e4302df3309b4a4a99133247bb89b77f); nothing fetched. The extract is single-column. One `## Rule <n>` line is inserted before each rule's own heading line after the cover sheet (a line opening, after an optional form feed and indentation, with `Rule <n>.` or `RULE <n>.` and whitespace, Rule 1 through Rule 86, 89 rules including 4.1, 4.2, 5.1, 5.2, 7.1, 23.1, 23.2, 44.1 and 62.1). Rule text was not edited: deleting the marker lines (`sed '/^## Rule /d'`) restores the extract byte for byte.
- Edition: as the extract; date of last amendment June 28, 2022.
- Sectioned sha256 055033c8c2b37cf8792e4d2f97822effa8ffe36365608ad0c5edee03015206ca (the `path_sha256` pinned by `lit/mt-district/Lit.toml`).
