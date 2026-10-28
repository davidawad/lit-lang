
### `mp-supreme/nmi-supreme-court-rules.txt` — Northern Mariana Islands Supreme Court Rules (including appellate procedure)

- Source: https://cnmilaw.gov/docs/courtrules/R01.pdf (official site; 62 pages, sha256 4bfe1a18ac71d532927cbb29111535353c40423d41ae107df519a0abb2aa5880).
- Edition: NMI Supreme Court Rules (NMI SUP. CT. R., includes the Rules of Appellate Procedure), listed as dated 2024-03-15 (R01) on the CNMI Law Revision Commission Court Rules page https://cnmilaw.gov/court-rules, which the CNMI Judiciary site https://www.nmijudiciary.gov links to as its Court Rules; the PDF opens with Administrative Order 2024-ADM-0006-RUL (e-filed Mar 15 2024); no later amendment is listed.
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 179bca1b5e7d735f20386aabda6a365ea07c4e2f4bf5e7c14c8a0ef27fa138f8. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `mp-supreme/original/nmi-supreme-court-rules.pdf` as served (sha256 4bfe1a18ac71d532927cbb29111535353c40423d41ae107df519a0abb2aa5880); extraction sha256 179bca1b5e7d735f20386aabda6a365ea07c4e2f4bf5e7c14c8a0ef27fa138f8.

### `mp-supreme/nmi-supreme-court-rules-sectioned.txt` — Northern Mariana Islands Supreme Court Rules, sectioned

- Derived in the pod (lg-f3q8.46) from `mp-supreme/nmi-supreme-court-rules.txt` (extract sha256 179bca1b5e7d735f20386aabda6a365ea07c4e2f4bf5e7c14c8a0ef27fa138f8); nothing fetched. One `## Rule <n>` line is inserted before each rule's own heading line in the body (`Rule 1.` through `Rule 53.`, 70 headings including the reserved rules and the hyphenated local rules such as `Rule 3-1.`); the administrative order and table of contents are unmarked. Rule text was not edited: deleting the marker lines (`sed '/^## Rule /d'`) restores the extract byte for byte.
- Source: as the extract `mp-supreme/nmi-supreme-court-rules.txt`, https://cnmilaw.gov/docs/courtrules/R01.pdf, retrieved 2026-10-27; the sectioning was done in the pod from the vendored extract.
- Edition: as the extract, the rules as served with Administrative Order 2024-ADM-0006-RUL (March 15, 2024).
- Sectioned sha256 a10aa4e3bf1020d81cdaa148172700a92cb2df9a6a00d380ceed6b3e5e2d2f46 (the `path_sha256` pinned by `lit/mp-trial/Lit.toml` and `lit/mp-supreme/Lit.toml`).
