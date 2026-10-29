
### `ilsd/ilsd-local-rules.txt` — S.D. Illinois Local Rules

- Source: https://www.ilsd.uscourts.gov/sites/ilsd/files/LocalRules.pdf (official site; 65 pages, sha256 dd223d2b5056467544ae7450e0b0d45641727071dd894e831c18b49443397d3b).
- Edition: Local Rules, revised Local Rules and Appendices effective September 22, 2025 (page https://www.ilsd.uscourts.gov/local-court-rules); fetched by curl.
- The criminal local rules (lg-gddt) are not in this document: it lists `APPENDIX A: CRIMINAL RULES` by title only (extract line 2819, in the appendix list that closes the 2,836-line extract). Appendix A is a separate document, vendored as `ilsd/ilsd-appendix-a-criminal.txt` (lg-ru4k).
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

- Note (lg-gddt.18): the vendored extract does not hold the criminal rules. It ends at page 65 with the list of appendix titles (`APPENDIX A: CRIMINAL RULES` at extract line 2819 is a title only; the extract has 2836 lines, so the "from extract line 2883" pointer above does not resolve). Appendices A to G are separate documents on the court site; Appendix A is now vendored as `ilsd/ilsd-appendix-a-criminal.txt` (lg-ru4k), and Appendices B to G are not vendored. `lit/ilsd-criminal` encodes only the periods the main Local Rules state for criminal proceedings (SDIL-LR 1.1(b)); the Appendix A rules stay holes there until they are encoded from that file.

### `ilsd/ilsd-appendix-a-criminal.txt` — S.D. Illinois Local Rules, Appendix A: Criminal Rules

- Source: https://www.ilsd.uscourts.gov/sites/ilsd/files/Appendix-A.pdf (official site, linked as `APPENDIX A: CRIMINAL RULES` from https://www.ilsd.uscourts.gov/local-court-rules; 3 pages, sha256 3b0e062952e9af37a5275aa40be1df9101bccfb6037d3e89a7ca1182a600e2dc).
- Edition: the Appendix A that goes with the Local Rules and Appendices effective September 22, 2025 (PDF created and last modified 2025-08-22). It holds Cr17.1 (defense subpoenas), Cr32.1 (sentencing and presentence reports), Cr41.1 (public access to search warrants), Cr44.1 (attorney's obligations on appeal) and Cr50.1 (disposition of criminal cases; speedy trial).
- Retrieved 2026-10-29 in the agent pod (lg-ru4k) with plain curl (its default User-Agent, no browser headers); extracted with `pdftotext -layout` (poppler 25.03.0), unedited. Raw extract, not yet sectioned into `## <ref>` form.
- Extract sha256 4d1208236f12fa47e7f95ec67abcb524b1ad44f61e45f75854265c5b14227340.
- Original: `ilsd/original/ilsd-appendix-a-criminal.pdf` as served (sha256 3b0e062952e9af37a5275aa40be1df9101bccfb6037d3e89a7ca1182a600e2dc).
