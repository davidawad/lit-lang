
### `fl-state-civil/fl-rules-civil-procedure-2026-10-01.txt` — Florida Rules of Civil Procedure

- Source: https://www-media.floridabar.org/uploads/2026/10/Civil-Procedure-Rules-10-01-26-1.pdf (Chapter 1, linked from https://www.floridabar.org/rules/ctproc/) (official site; 340 pages, sha256 e2da940766f5d77153815484ed16f2103f50f6fa1812596f149b9241a3c03010).
- Edition: Florida Rules of Civil Procedure, the Florida Bar's Oct 1, 2026 edition (file name 10-01-26).
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 282ce6dd57b93e578abc272ded6759c9fe3e7c46230915ef3b314cf221f5d132. Raw extract, not yet sectioned into `## <ref>` form.
- Fetch method: BrowserOS neo; site served a bot challenge to plain automated requests (CoS ruling 2026-10-27). floridabar.org robots.txt disallows only /private/ for general agents (the www-media host serves none); the Disclaimer & Terms of Use page bars no automated access. flcourts.gov hosts were not touched.
- Original: `fl-state-civil/original/fl-rules-civil-procedure-2026-10-01.pdf` as served (sha256 e2da940766f5d77153815484ed16f2103f50f6fa1812596f149b9241a3c03010); extraction sha256 282ce6dd57b93e578abc272ded6759c9fe3e7c46230915ef3b314cf221f5d132.

### `fl-state-civil/fl-rules-civil-procedure-2026-10-01-sectioned.txt` — Florida Rules of Civil Procedure, sectioned

- Derived in the pod (lg-f3q8.14) from `fl-state-civil/fl-rules-civil-procedure-2026-10-01.txt` (extract sha256 282ce6dd57b93e578abc272ded6759c9fe3e7c46230915ef3b314cf221f5d132); nothing fetched. One `## Rule <n>` line is inserted before each rule's own heading line in the body (`RULE 1.010. SCOPE AND TITLE OF RULES` through `RULE 1.900. FORMS`, 94 rules; the table of contents is unmarked). Rule text was not edited: deleting the marker lines (`sed '/^## Rule /d'`) restores the extract byte for byte.
- Source: as the extract `fl-state-civil/fl-rules-civil-procedure-2026-10-01.txt`, https://www-media.floridabar.org/uploads/2026/10/Civil-Procedure-Rules-10-01-26-1.pdf, retrieved 2026-10-27; the sectioning was done in the pod from the vendored extract.
- Edition: as the extract, the Florida Bar's October 1, 2026 edition.
- Sectioned sha256 d769c6cb6d8b499bb1fe03349f3349e5c4fd8e315353fbce66e2337259a98f6d (the `path_sha256` pinned by `lit/fl-state-civil/Lit.toml`).
