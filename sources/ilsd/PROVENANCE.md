
### `ilsd/ilsd-local-rules.txt` — S.D. Illinois Local Rules

- Source: https://www.ilsd.uscourts.gov/sites/ilsd/files/LocalRules.pdf (official site; 65 pages, sha256 dd223d2b5056467544ae7450e0b0d45641727071dd894e831c18b49443397d3b).
- Edition: Local Rules, revised Local Rules and Appendices effective September 22, 2025 (page https://www.ilsd.uscourts.gov/local-court-rules); fetched by curl.
- Also the criminal local rules (lg-gddt): 'APPENDIX A: CRIMINAL RULES' part, from extract line 2883.
- Criminal forum (lg-gddt): ilsd-criminal.
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 f150a1d84ecf1dcc529eb49365d7fcb74392616d7653c47541fe633c44899d3c. Raw extract, not yet sectioned into `## <ref>` form.

- Fetch method: first fetched with curl and a desktop browser User-Agent before the no-browser-identity rule (PM 2026-10-27); re-fetched on 2026-10-27 with plain curl (its default User-Agent, no browser headers): byte-identical (same sha256).
- Original: `ilsd/original/ilsd-local-rules.pdf` as served (sha256 dd223d2b5056467544ae7450e0b0d45641727071dd894e831c18b49443397d3b); extraction sha256 f150a1d84ecf1dcc529eb49365d7fcb74392616d7653c47541fe633c44899d3c.

### `ilsd/ilsd-local-rules-sectioned.txt` — S.D. Illinois Local Rules, sectioned

- Derived in the pod (lg-f3q8.54) from `ilsd/ilsd-local-rules.txt` (extract sha256 f150a1d84ecf1dcc529eb49365d7fcb74392616d7653c47541fe633c44899d3c); nothing fetched. One `## Rule <n>` line is inserted before each rule's own heading line after the table of contents (`Rule 1.1` through `Rule 83.14`, 44 rules); in-text references (`SDIL-LR 23.1 takes priority.`) are unmarked, and the appendices are not sectioned. Rule text was not edited: deleting the marker lines (`sed '/^## Rule /d'`) restores the extract byte for byte.
- Upstream: https://www.ilsd.uscourts.gov/sites/ilsd/files/LocalRules.pdf, retrieved 2026-10-27, as recorded on the `ilsd/ilsd-local-rules.txt` entry this file is derived from.
- Edition: as the extract, revised Local Rules effective September 22, 2025.
- Sectioned sha256 2f93faac3661abe55136b45631a7d3ec496e0142942d8bff0dcf52295793f4bb (the `path_sha256` pinned by `lit/ilsd/Lit.toml`).
