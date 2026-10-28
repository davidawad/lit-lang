
### `bankr-nye/nye-local-rules-2019-12-13.txt` — E.D.N.Y. Local Bankruptcy Rules (effective December 13, 2019, amended September 17, 2026)

- Source: https://www.nyeb.uscourts.gov/sites/nyeb/files/USBC-EDNY-Local-Bankruptcy-Rules.pdf (official site; 91 pages, sha256 fe410ff86562cb6fb4263b5973322733f33a4fcef28f695a1f9af951bfa87056).
- Edition: Local Bankruptcy Rules 'Effective December 13, 2019, Amended September 17, 2026 (to update cover page only)' (title page), the 'New and Amended Local Bankruptcy Rules (PDF version)' at https://www.nyeb.uscourts.gov/local-bankruptcy-rules-united-states-bankruptcy-court-eastern-district-new-york. robots.txt (Crawl-delay 10 honored) and the court's privacy/security policy page (https://www.nyeb.uscourts.gov/web-policy-usbc-edny) checked: no prohibition of automated access.
- Retrieved 2026-10-28 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 beb54ac9d338a30e15abe4561949f501277dd73f7590db1dce7058f19e57873c. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `bankr-nye/original/nye-local-rules-2019-12-13.pdf` as served (sha256 fe410ff86562cb6fb4263b5973322733f33a4fcef28f695a1f9af951bfa87056); extraction sha256 beb54ac9d338a30e15abe4561949f501277dd73f7590db1dce7058f19e57873c.

### `bankr-nye/nye-local-rules-2019-12-13-sectioned.txt` — E.D.N.Y. Local Bankruptcy Rules, sectioned

- Derived in the pod (lg-u561.11) from `bankr-nye/nye-local-rules-2019-12-13.txt` (extract sha256 beb54ac9d338a30e15abe4561949f501277dd73f7590db1dce7058f19e57873c); nothing fetched. One `## LBR <n>` line is inserted before each rule's own indented heading line in the body (`1001-1  SHORT TITLE; APPLICABILITY` through `9077-1`, including the undashed `3021`; 107 sections); the table of contents is unmarked. Rule text was not edited: deleting the marker lines (`sed '/^## LBR /d'`) restores the extract byte for byte.
- Source: as the extract `bankr-nye/nye-local-rules-2019-12-13.txt`, https://www.nyeb.uscourts.gov/sites/nyeb/files/USBC-EDNY-Local-Bankruptcy-Rules.pdf, retrieved 2026-10-28; the sectioning was done in the pod from the vendored extract.
- Edition: as the extract, effective December 13, 2019; the September 17, 2026 amendment updates the cover page only.
- Sectioned sha256 d36fb31c6d5170f7e2a75e0430c2966f121739b6582c23fad6dfddf4ad34e626 (the `path_sha256` pinned by `lit/bankr-nye/Lit.toml` for source `nyeb-lbr`).
