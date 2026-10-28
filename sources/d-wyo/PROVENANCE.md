
### `d-wyo/d-wyo-local-civil-rules-2022.txt` — D. Wyo. Local Civil Rules (USDCLR)

- Source: https://www.wyd.uscourts.gov/sites/wyd/files/local_rules/2022%20Civil%20Local%20Rules%20-%20FINAL.pdf (official site; 95 pages, sha256 babc2b430a2fec4469b29c6ef0078a1a6d02b8105348a0691189c64bd373aeda).
- Edition: Local Civil Rules, January 2022 (title page); https://www.wyd.uscourts.gov/court-info/local-rules-and-orders/local-rules lists "CIVIL LOCAL RULES - Revised January 2022" as current (only the criminal rules were revised August 2026); fetched with plain curl (no browser identity); robots.txt allows the path (Crawl-delay 10 honored).
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 7426b22750d4d951a550db7bb4df187bb5c5e6b564f669a579be5af36b1f9d01. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `d-wyo/original/d-wyo-local-civil-rules-2022.pdf` as served (sha256 babc2b430a2fec4469b29c6ef0078a1a6d02b8105348a0691189c64bd373aeda); extraction sha256 7426b22750d4d951a550db7bb4df187bb5c5e6b564f669a579be5af36b1f9d01.

### `d-wyo/dwyo-local-criminal-rules-2026-08-01.txt` — D. Wyo. Local Criminal Rules (current revision August 1, 2026)

- Source: https://www.wyd.uscourts.gov/sites/wyd/files/localrules-cr.pdf (official site; 38 pages, sha256 622a6585c60e644213c9013460c571669794b4c097325764177c23d0b121533b).
- Edition: Local Criminal Rules 'Current Revision: August 1, 2026' (title page), the 'CRIMINAL LOCAL RULES' file linked at https://www.wyd.uscourts.gov/court-info/local-rules-and-orders/local-rules (the page also links a 2026 redline, not vendored). robots.txt of www.wyd.uscourts.gov read (Crawl-delay 10 honored where stated; no Disallow covers the fetched path); the judiciary-wide Privacy & Security Policy (https://www.uscourts.gov/privacy-security-policy) read: no prohibition of automated access. Fetched with curl and the identifying UA lit-lang-provenance/1.0, at most 1 request per 10 s.
- Criminal forum (lg-gddt): wyd-criminal.
- Retrieved 2026-10-28 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 b29eaa75f2618c3ca1aab8268a27ff961593ed58353432ab1f5ba56b969b2b8b. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `d-wyo/original/dwyo-local-criminal-rules-2026-08-01.pdf` as served (sha256 622a6585c60e644213c9013460c571669794b4c097325764177c23d0b121533b); extraction sha256 b29eaa75f2618c3ca1aab8268a27ff961593ed58353432ab1f5ba56b969b2b8b.

### `d-wyo/d-wyo-local-civil-rules-2022-sectioned.txt` — D. Wyo. Local Civil Rules, sectioned

- Derived in the pod (lg-f3q8.71) from `d-wyo/d-wyo-local-civil-rules-2022.txt` (extract sha256 7426b22750d4d951a550db7bb4df187bb5c5e6b564f669a579be5af36b1f9d01); nothing fetched. One `## U.S.D.C.L.R. <n>` line is inserted before each rule's own `Rule <n>` heading line in the body (`U.S.D.C.L.R. 1.1` through `U.S.D.C.L.R. 86`, 68 rules); the table of contents is unmarked. Rule text was not edited: deleting the marker lines (`sed '/^## U\.S\.D\.C\.L\.R\. /d'`) restores the extract byte for byte.
- Source: as the extract, https://www.wyd.uscourts.gov/sites/wyd/files/local_rules/2022%20Civil%20Local%20Rules%20-%20FINAL.pdf, retrieved 2026-10-27.
- Edition: as the extract, Local Civil Rules, January 2022.
- Sectioned sha256 7961602f05ecd83efb529a3b713b2386b54dc8cbe9f45ebfb6bc905b2b7b1036 (the `path_sha256` pinned by `lit/wyd/Lit.toml`).
