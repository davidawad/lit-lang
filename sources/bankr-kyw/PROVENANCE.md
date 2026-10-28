
### `bankr-kyw/kyw-local-rules-2025-12-01.txt` — W.D. Ky. Bankruptcy Local Rules (effective December 1, 2025)

- Source: https://www.kywb.uscourts.gov/sites/kywb/files/2025%20Local%20Rules%20Final%26Approved.pdf (official site; 23 pages, sha256 221bacccf32b83bb311ac1dcd670aa6f62ca92001680c85d0392a47a6113b38c).
- Edition: Local Rules effective December 1, 2025 (title page), linked as 'Current Local Rules' at https://www.kywb.uscourts.gov/local-rules. The general orders page lists no later order amending the rules. robots.txt (Crawl-delay 10 honored) and the court's privacy/security policy page (https://www.kywb.uscourts.gov/privacy-policy) checked: no prohibition of automated access.
- Retrieved 2026-10-28 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 24be285f04df8bee52387b6fc452ea5b8c3f6e4eeda4c5e06823ce4d7d267444. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `bankr-kyw/original/kyw-local-rules-2025-12-01.pdf` as served (sha256 221bacccf32b83bb311ac1dcd670aa6f62ca92001680c85d0392a47a6113b38c); extraction sha256 24be285f04df8bee52387b6fc452ea5b8c3f6e4eeda4c5e06823ce4d7d267444.

### `bankr-kyw/kyw-local-rules-2025-12-01-sectioned.txt` — W.D. Ky. Bankruptcy Local Rules, sectioned

- Derived in the pod (lg-u561.19) from `bankr-kyw/kyw-local-rules-2025-12-01.txt` (extract sha256 24be285f04df8bee52387b6fc452ea5b8c3f6e4eeda4c5e06823ce4d7d267444); nothing fetched. One `## Rule <n>` line is inserted before each rule's own heading line in the body (`RULE 1002-1` through `RULE 9074-1`, including `RULE 1019` and `RULE 7007.1-1` and the headings that begin with a form feed; 45 sections, matching the table of contents); the title page and table of contents (extract lines 1-61) are unmarked. Rule text was not edited: deleting the marker lines (`sed '/^## /d'`) restores the extract byte for byte.
- Source: as the extract `bankr-kyw/kyw-local-rules-2025-12-01.txt`, https://www.kywb.uscourts.gov/sites/kywb/files/2025%20Local%20Rules%20Final%26Approved.pdf, retrieved 2026-10-28; the sectioning was done in the pod from the vendored extract.
- Edition: as the extract, the Local Rules effective December 1, 2025.
- Sectioned sha256 e7c2b84b3267b8db2507c354a2fe91b32dbbcaafa7325826558a39816b44ae1b (the `path_sha256` pinned by `lit/bankr-kyw/Lit.toml` for source `kywb-lbr`).
