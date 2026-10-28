
### `bankr-ilc/ilcb-lbr-2024-10-01.txt` — Local Rules, U.S. Bankruptcy Court for the Central District of Illinois (effective October 1, 2024)

- Source: https://www.ilcb.uscourts.gov/sites/ilcb/files/ILCB%20Local%20Rules%2010.1.2024.pdf (official site; 21 pages, sha256 a40a9b3665a420279db4543a9af278a1eb4daf9853cb2465b0eff7f8f9612be7).
- Edition: Local Rules 'Effective October 1, 2024' (cover), linked as 'Local Rules of United States Bankruptcy Court for the Central District' from https://www.ilcb.uscourts.gov/local-rules-procedures-and-standing-orders. robots.txt (Crawl-delay 10 honored; no rule covering these paths) and the Privacy Protection Policy checked: no prohibition of automated access. Fetched with curl and the identifying User-Agent lit-lang-provenance/1.0 (angrybohr@proton.me).
- Retrieved 2026-10-28 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 e9acb44577a6456b9122e6db1765e335c8f8ffaa349694e0ce9cf7381d342304. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `bankr-ilc/original/ilcb-lbr-2024-10-01.pdf` as served (sha256 a40a9b3665a420279db4543a9af278a1eb4daf9853cb2465b0eff7f8f9612be7); extraction sha256 e9acb44577a6456b9122e6db1765e335c8f8ffaa349694e0ce9cf7381d342304.
- Amending-order check (lg-u561.30): order page read 2026-10-27 (Standing Orders list on https://www.ilcb.uscourts.gov/local-rules-procedures-and-standing-orders): no amending order after 2024-10-01; the latest items are the 2025-02-10 CM/ECF-inaccessibility order and the 2024-10-01 order adopting these rules, which is the consolidation itself.

### `bankr-ilc/ilcb-lbr-2024-10-01-sectioned.txt` — C.D. Ill. Local Bankruptcy Rules, sectioned

- Derived in the pod (lg-u561.21) from `bankr-ilc/ilcb-lbr-2024-10-01.txt` (extract sha256 e9acb44577a6456b9122e6db1765e335c8f8ffaa349694e0ce9cf7381d342304); nothing fetched. One `## LBR <n>` line is inserted before each rule's body heading (`RULE <n> <TITLE>`, after the table of contents; `LBR 1001-1` through `LBR 9037-1`; 28 sections), the rule number's U+2010 hyphen written as `-`; the cover, the District Court's General Order No. 24-GO-04 and the table of contents are unmarked. Rule text was not edited: deleting the marker lines (`sed '/^## LBR /d'`) restores the extract byte for byte.
- Edition: as the extract, effective October 1, 2024.
- Sectioned sha256 fa33b1600cadd6078aa4ac2821af44970db1821eb8f20b2e1dbdf996e2ecdb51 (the `path_sha256` pinned by `lit/bankr-ilc/Lit.toml` for source `ilcb-lbr`).
