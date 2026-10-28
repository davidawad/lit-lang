
### `me-law-court/rules-of-appellate-procedure.txt` — Maine Rules of Appellate Procedure (with advisory notes)

- Source: https://www.courts.maine.gov/rules/text/mr_app_p_plus_2024-11-01.pdf (official site; 179 pages, sha256 2f56aa30698a5b5978097fdd607f1a03f95c075728a5507bd507a2e3042fffb4).
- Edition: Maine Rules of Appellate Procedure with advisory notes, 'Last reviewed and edited September 30, 2024; includes amendments effective November 1, 2024' (cover); listed as the current PDF on https://www.courts.maine.gov/rules/index.html, whose amendment list shows no later appellate amendment; fetched with curl (plain User-Agent), no bot refusal; courts.maine.gov robots.txt absent (404) and the site publishes no terms page (its footer links only the maine.gov accessibility page, which returned an unexplained 403 to curl and was not retried).
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 f75bf4929690ec6769b53aff177b797fff4103193ed0b38d227186b7f5263fed. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `me-law-court/original/rules-of-appellate-procedure.pdf` as served (sha256 2f56aa30698a5b5978097fdd607f1a03f95c075728a5507bd507a2e3042fffb4); extraction sha256 f75bf4929690ec6769b53aff177b797fff4103193ed0b38d227186b7f5263fed.

### `me-law-court/rules-of-appellate-procedure-sectioned.txt` — Maine Rules of Appellate Procedure, sectioned

- Derived in the pod (lg-f3q8.22) from `me-law-court/rules-of-appellate-procedure.txt` (extract sha256 f75bf4929690ec6769b53aff177b797fff4103193ed0b38d227186b7f5263fed); nothing fetched. One `## Rule <n>` line is inserted at each rule's own heading in the body (`RULE 1. SCOPE OF RULES` through `RULE 25. CERTIFICATION OF QUESTIONS OF LAW`, 31 rules, each section running through that rule's restyling and advisory notes; the table of contents and introductory notes are unmarked): before the heading line, or right after it when the heading holds a colon (Rule 7A), since the corpus reader opens a section at a `RULE ...:` line. Rule text was not edited: deleting the marker lines (`sed '/^## Rule /d'`) restores the extract byte for byte.
- Source: as the extract `me-law-court/rules-of-appellate-procedure.txt`, https://www.courts.maine.gov/rules/text/mr_app_p_plus_2024-11-01.pdf, retrieved 2026-10-27; the sectioning was done in the pod from the vendored extract.
- Edition: as the extract, the edition including amendments effective November 1, 2024.
- Sectioned sha256 c0893d371ac23ed296d1a7b7c26b34e0fed0f5b82db4239c2f0aef64d93082b9 (the `path_sha256` pinned by `lit/me-superior-civil/Lit.toml` and `lit/me-law-court/Lit.toml`).
