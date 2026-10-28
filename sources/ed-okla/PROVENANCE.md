
### `ed-okla/ed-okla-local-civil-rules-2016.txt` — E.D. Okla. Local Civil Rules (LCvR)

- Source: https://www.oked.uscourts.gov/sites/oked/files/Local_Civil_Rules.pdf (official site; 53 pages, sha256 1deded6dcdb897c5677b75f92ef727f91e850ad26b389296740e4b162cc10152).
- Edition: Local Civil Rules, Effective Date July 5, 2016 (title page); https://www.oked.uscourts.gov/court-info/local-rules-and-orders lists "Local Civil Rules (effective July 5, 2016)" as the current civil rules; fetched with plain curl (no browser identity); robots.txt allows the path (Crawl-delay 10 honored).
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 f5291f852443cc56f7888e77a293905559206289165e6157ad1d2105aefb2f64. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `ed-okla/original/ed-okla-local-civil-rules-2016.pdf` as served (sha256 1deded6dcdb897c5677b75f92ef727f91e850ad26b389296740e4b162cc10152); extraction sha256 f5291f852443cc56f7888e77a293905559206289165e6157ad1d2105aefb2f64.

### `ed-okla/edokla-local-criminal-rules-2016-07-05.txt` — E.D. Okla. Local Criminal Rules (effective July 5, 2016)

- Source: https://www.oked.uscourts.gov/sites/oked/files/Local_Criminal_Rules.pdf (official site; 23 pages, sha256 c3cf512156c6efcf47f135fe5582eb42a96b85406823577b922c854dab982093).
- Edition: Local Criminal Rules 'Effective Date: July 5, 2016' (title page), linked as 'Local Criminal Rules' at https://www.oked.uscourts.gov/court-info/local-rules-and-orders. OLD EDITION: the court still presents it as current. robots.txt of www.oked.uscourts.gov read (Crawl-delay 10 honored where stated; no Disallow covers the fetched path); the judiciary-wide Privacy & Security Policy (https://www.uscourts.gov/privacy-security-policy) read: no prohibition of automated access. Fetched with curl and the identifying UA lit-lang-provenance/1.0, at most 1 request per 10 s.
- Criminal forum (lg-gddt): oked-criminal.
- Retrieved 2026-10-28 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 f056c35c58f1058a0bd3cf32cc8f9d1b33ef0eb5b014784be338244500a79628. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `ed-okla/original/edokla-local-criminal-rules-2016-07-05.pdf` as served (sha256 c3cf512156c6efcf47f135fe5582eb42a96b85406823577b922c854dab982093); extraction sha256 f056c35c58f1058a0bd3cf32cc8f9d1b33ef0eb5b014784be338244500a79628.

### `ed-okla/edokla-local-criminal-rules-2016-07-05-sectioned.txt` — E.D. Okla. Local Criminal Rules, sectioned

- Derived in the pod (lg-gddt.24) from `ed-okla/edokla-local-criminal-rules-2016-07-05.txt` (extract sha256 f056c35c58f1058a0bd3cf32cc8f9d1b33ef0eb5b014784be338244500a79628); nothing fetched. In the body (after the table of contents), one marker line `## LCrR <n>` is inserted before each rule's own heading line `LOCAL CRIMINAL RULE <n> - <TITLE>` (LCrR 1.1 through 57.2, 17 rules). The extract ends partway through LCrR 57.2(C) at the folio of page 19 of the 23-page PDF: the rest of LCrR 57.2 and LCrR 58.1, listed in the table of contents, are not in it (PARTIAL; re-extraction from the original on carbon would supply them). Rule text was not edited: deleting the marker lines (each `## LCrR ` line with its newline) restores the extract byte for byte.
- Edition: as the extract, "Effective Date: July 5, 2016".
- Sectioned sha256 06d896fc5319f2c22e0445acdadf7fd5c215ad4d86a86b2d2d072b484f82060e (the `path_sha256` pinned by `lit/oked-criminal/Lit.toml`).

### `ed-okla/ed-okla-local-civil-rules-2016-sectioned.txt` — E.D. Okla. Local Civil Rules, sectioned

- Derived in the pod (lg-f3q8.88) from `ed-okla/ed-okla-local-civil-rules-2016.txt` (extract sha256 f5291f852443cc56f7888e77a293905559206289165e6157ad1d2105aefb2f64); nothing fetched. In the body (after the table of contents), one marker line `## <n>` is inserted before each rule's own heading line `<n> <Title>.`, where `<n>` is the rule's designation as written (`LCvR 1.1` through `LCvR 84.1`, 63 rules). The table of contents carries no markers (LCvR 84.1's section runs to the end of the file, appendices included). Rule text was not edited: deleting the marker lines (each `## LCvR ` line with its newline) restores the extract byte for byte.
- Edition: as the extract, Local Civil Rules "Effective Date July 5, 2016".
- Sectioned sha256 bd4a1a6c32e665030862ed11c51837474b78ba3835ca024b34975a2408c25c96 (the `path_sha256` pinned by `lit/oked/Lit.toml`).
