
### `bankr-ian/ianb-lbr-2024-01-01.txt` — Local Rules of Bankruptcy Practice and Procedure, N.D. Iowa (as revised, effective January 1, 2024)

- Source: https://www.ianb.uscourts.gov/sites/ianb/files/2023%20Local%20Rules%20-%20%28Final%29_0.pdf (official site; 49 pages, sha256 2998b2dcac4313c3e2472aa708cf6eb5cf2b73df5493f48e79ecc7308c86cabd).
- Edition: Local Rules of Bankruptcy Practice and Procedure, 'As revised, effective January 1, 2024' (cover; file name '2023 Local Rules - (Final)'), linked as 'Local Rules of Bankruptcy Practice and Procedure' from https://www.ianb.uscourts.gov/local-rules. robots.txt (Crawl-delay 10 honored; no rule covering these paths) and the Privacy Policy and Redaction Requirements page and the website Privacy Policy checked: no prohibition of automated access. Fetched with curl and the identifying User-Agent lit-lang-provenance/1.0 (angrybohr@proton.me).
- Retrieved 2026-10-28 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 48ba8e4b326960b44e842fc14348918d3689aaa91cfd3ad52ec64e3da4936f82. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `bankr-ian/original/ianb-lbr-2024-01-01.pdf` as served (sha256 2998b2dcac4313c3e2472aa708cf6eb5cf2b73df5493f48e79ecc7308c86cabd); extraction sha256 48ba8e4b326960b44e842fc14348918d3689aaa91cfd3ad52ec64e3da4936f82.
- Amending-order check (lg-u561.30): order page read 2026-10-27 (https://www.ianb.uscourts.gov/judicial-orders): the only order after the 2024-01-01 consolidation that touches the rules is Standing Order 26-03, vendored below as a layer; Administrative Orders 24-02 (vacates a COVID order), 24-03 and Standing Order 25-02 (court closures) do not amend the local rules and are not vendored.

### `bankr-ian/ianb-standing-order-26-03-ch13-fees.txt` — N.D. Iowa Standing Order 26-03 (Chapter 13 attorney fee requests, LR 2016-1(b))

- Source: https://www.ianb.uscourts.gov/sites/ianb/files/StandingOrder26-03.pdf (official site; 9 pages, sha256 82e2c32f51aea420285452b70b3dbc292d3ee0150c4eab0818f0c96c1cb11c37).
- Edition: Chapter 13 Attorney Fee Requests, Standing Order No. 26-03, effective July 1, 2026 (listing date 07/01/26 on https://www.ianb.uscourts.gov/judicial-orders); it vacates and replaces Standing Order 25-01 and sets the NO-LOOK compensation amounts that LR 2016-1(b) refers to, so it supplements rather than rewrites the rule text. Apply on top of ianb-lbr-2024-01-01.txt from 2026-07-01. OCR, verify numbers (image scan). robots.txt (Crawl-delay 10 honored; no rule covering these paths) and the Privacy Policy and Redaction Requirements page and the website Privacy Policy checked: no prohibition of automated access. Fetched with curl and the identifying User-Agent lit-lang-provenance/1.0 (angrybohr@proton.me).
- Retrieved 2026-10-28 on carbon (lg-f3q8 standing method); extracted with OCR (pdftoppm 300 dpi, tesseract 5.5.3, --psm 6) because the PDF is an image scan with no text layer; unedited OCR, so verify any quoted value against the PDF.
- Extract sha256 e0bd949bdd7125afd17c515c6b08f28acb5a5b5785fdce75c2ae306a51662e80. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `bankr-ian/original/ianb-standing-order-26-03-ch13-fees.pdf` as served (sha256 82e2c32f51aea420285452b70b3dbc292d3ee0150c4eab0818f0c96c1cb11c37); extraction sha256 e0bd949bdd7125afd17c515c6b08f28acb5a5b5785fdce75c2ae306a51662e80.

### `bankr-ian/ianb-lbr-2024-01-01-sectioned.txt` — N.D. Iowa Local Rules of Bankruptcy Practice and Procedure, sectioned

- Derived in the pod (lg-u561.23) from `bankr-ian/ianb-lbr-2024-01-01.txt` (extract sha256 48ba8e4b326960b44e842fc14348918d3689aaa91cfd3ad52ec64e3da4936f82); nothing fetched. One `## Rule <n>` line is inserted before each rule's own heading line in the body (`Rule 1002-1 Petition - General` through `Rule 9074-1 Telephone Conferences`, including the headings that follow a form feed; 71 sections, matching the table of contents); the Part 2 headings print a Unicode hyphen (U+2010, e.g. `Rule 2002‐1`), and their markers use the ASCII hyphen (`## Rule 2002-1`). The table of contents and the indented cross-reference line in LR 4001-1(c) are unmarked. Rule text was not edited: deleting the marker lines (`sed '/^## Rule /d'`) restores the extract byte for byte.
- Source: as the extract `bankr-ian/ianb-lbr-2024-01-01.txt`, https://www.ianb.uscourts.gov/sites/ianb/files/2023%20Local%20Rules%20-%20%28Final%29_0.pdf, retrieved 2026-10-28; the sectioning was done in the pod from the vendored extract.
- Edition: as the extract, the rules as revised effective January 1, 2024.
- Sectioned sha256 d98b3fcff18400928eb7b005b69a1f805eb1089a1eaa2e0372cd4edada634945 (the `path_sha256` pinned by `lit/bankr-ian/Lit.toml` for source `ianb-lbr`).
