
### `nd-ga/ndga-civil-local-rules.txt` — N.D. Ga. Local Rules, Civil

- Source: https://www.gand.uscourts.gov/sites/gand/files/local_rules/NDGARulesCV_4_0_26.pdf (official site; 89 pages, sha256 ab46e4c1538c4341a574a8e73d1d4e1fc7a19a3c04c5225278cb82cbe934ed38).
- Edition: Civil Rules, "Updated 8/10/2026" on https://www.gand.uscourts.gov/court-info/local-rules-and-orders/local-rules (PDF created 2026-08-24).
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 5275df22e495d46234c37192293e267716b99881711ad30162b33ea406b26ede. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `nd-ga/original/ndga-civil-local-rules.pdf` as served (sha256 ab46e4c1538c4341a574a8e73d1d4e1fc7a19a3c04c5225278cb82cbe934ed38); extraction sha256 5275df22e495d46234c37192293e267716b99881711ad30162b33ea406b26ede.

### `nd-ga/ndga-criminal-local-rules-2024-04-02.txt` — N.D. Ga. Criminal Local Rules (updated 04/02/2024)

- Source: https://www.gand.uscourts.gov/sites/gand/files/local_rules/NDGARulesCR_3.pdf (official site; 31 pages, sha256 7ef51f4faa42a6b3a298efaffddbd439a382b1d47ba1803d02f814447e74b8c8).
- Edition: Criminal Local Rules (LCrR), listed as 'Criminal Rules, Updated 04/02/2024' at https://www.gand.uscourts.gov/court-info/local-rules-and-orders/local-rules; the PDF itself carries no edition date (LCrR 1.1 states the 1997 adoption). robots.txt of www.gand.uscourts.gov read (Crawl-delay 10 honored where stated; no Disallow covers the fetched path); the judiciary-wide Privacy & Security Policy (https://www.uscourts.gov/privacy-security-policy) read: no prohibition of automated access. Fetched with curl and the identifying UA lit-lang-provenance/1.0, at most 1 request per 10 s.
- Criminal forum (lg-gddt): gand-criminal.
- Retrieved 2026-10-28 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 a9455b9e4ed98404f282c2cf03a64e073d943b83c890b0eaa4a4a9c8b1254954. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `nd-ga/original/ndga-criminal-local-rules-2024-04-02.pdf` as served (sha256 7ef51f4faa42a6b3a298efaffddbd439a382b1d47ba1803d02f814447e74b8c8); extraction sha256 a9455b9e4ed98404f282c2cf03a64e073d943b83c890b0eaa4a4a9c8b1254954.

### `nd-ga/ndga-civil-local-rules-sectioned.txt` — N.D. Ga. Civil Local Rules, sectioned

- Derived in the pod (lg-f3q8.15) from `nd-ga/ndga-civil-local-rules.txt` (extract sha256 5275df22e495d46234c37192293e267716b99881711ad30162b33ea406b26ede); nothing fetched. One `## LR <n>` line is inserted before each rule's own heading line in the body (`LR 1.1 TITLE; EFFECTIVE DATE ...` through `LR 84.1 Appendix B ...`, 75 rules; the table of contents, the chapter headings such as `LR 3:` and the second heading numbered `LR 65.1:` are unmarked; `LR 16. 1` is marked `## LR 16.1`). Every other line is the extract unchanged, so deleting the `## LR` lines gives back the extract byte for byte.
- Upstream: https://www.gand.uscourts.gov/sites/gand/files/local_rules/NDGARulesCV_4_0_26.pdf, retrieved 2026-10-27, as recorded on the `nd-ga/ndga-civil-local-rules.txt` entry this file is derived from.
- Edition: as the extract, Civil Rules "Updated 8/10/2026".
- Sectioned sha256 52d75f2de9281d261171b2938c6169b3f046895cb2c37a047d9210cd727d3c66 (the `path_sha256` pinned by `lit/gand/Lit.toml`).

### `nd-ga/ndga-criminal-local-rules-2024-04-02-sectioned.txt` — N.D. Ga. Criminal Local Rules, sectioned

- Derived in the pod (lg-gddt.26) from `nd-ga/ndga-criminal-local-rules-2024-04-02.txt` (extract sha256 a9455b9e4ed98404f282c2cf03a64e073d943b83c890b0eaa4a4a9c8b1254954); nothing fetched. One `## LCrR <n>` line is inserted before each rule's first heading line in the body (`LCrR 1:` through `LCrR 59:`, 51 headings, the same 51 the table of contents lists; the body repeats the LCrR 59 heading, and only its first is marked); the table of contents is unmarked. Rule text was not edited: deleting the marker lines (`sed '/^## LCrR /d'`) restores the extract byte for byte.
- Source: as the extract `nd-ga/ndga-criminal-local-rules-2024-04-02.txt`, https://www.gand.uscourts.gov/sites/gand/files/local_rules/NDGARulesCR_3.pdf, retrieved 2026-10-28; the sectioning was done in the pod from the vendored extract.
- Edition: as the extract, listed as 'Criminal Rules, Updated 04/02/2024'.
- Sectioned sha256 ec91c1382ee1369aaed3c134e4f885d8592a3a900fefc4d7ce261440c849a0ae (the `path_sha256` pinned by `lit/gand-criminal/Lit.toml`).
