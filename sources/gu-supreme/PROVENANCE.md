
### `gu-supreme/guam-rules-of-appellate-procedure.txt` — Guam Rules of Appellate Procedure

- Source: https://www.guamcourts.gov/sites/default/files/GRAP_20140224.pdf (official site; 71 pages, sha256 7021c87f1aa8db5bb4a00f8c3e71a148bc3c1e10b68a4ac6a0801fda71564295).
- Edition: Guam Rules of Appellate Procedure, "as of February 24, 2014" (Promulgation Order No. 07-003-06), the edition linked from https://www.guamcourts.gov/courts-council/supreme-court/rules-procedure; the promulgation-orders page lists no later amendment. Fetched with curl (robots.txt allows).
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 f62ac9603dedf970f85014bd570a45591dbc7b4356d2dec1ea9e247918f558f0. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `gu-supreme/original/guam-rules-of-appellate-procedure.pdf` as served (sha256 7021c87f1aa8db5bb4a00f8c3e71a148bc3c1e10b68a4ac6a0801fda71564295); extraction sha256 f62ac9603dedf970f85014bd570a45591dbc7b4356d2dec1ea9e247918f558f0.

### `gu-supreme/guam-rules-of-appellate-procedure-sectioned.txt` — Guam Rules of Appellate Procedure, sectioned

- Derived in the pod (lg-f3q8.41) from `gu-supreme/guam-rules-of-appellate-procedure.txt` (extract sha256 f62ac9603dedf970f85014bd570a45591dbc7b4356d2dec1ea9e247918f558f0); nothing fetched. One `## Rule <n>` line is inserted before each rule's own heading line in the body (`Rule 1 --` through `Rule 36 --`, 42 rules including 4.1, 4.2, 7.1, 13.1, 13.2 and 21.1); the table of contents and in-text references are unmarked. Rule text was not edited: deleting the marker lines (`sed '/^## Rule /d'`) restores the extract byte for byte.
- Source: as the extract `gu-supreme/guam-rules-of-appellate-procedure.txt`, https://www.guamcourts.gov/sites/default/files/GRAP_20140224.pdf, retrieved 2026-10-27; the sectioning was done in the pod from the vendored extract.
- Edition: as the extract, as of February 24, 2014.
- Sectioned sha256 7151ec99e54e97315b88de099186256181f8350998da456f400f4140175b321c (the `path_sha256` pinned by `lit/gu-superior/Lit.toml` and `lit/gu-supreme/Lit.toml`).
