
### `bankr-txe/txe-local-rules-2022-08-22.txt` — E.D. Tex. Bankruptcy Local Rules (revised August 22, 2022)

- Source: https://www.txeb.uscourts.gov/sites/txeb/files/2022%20LOCAL%20RULES_rev%20as%20of%208-22-22.pdf (official site; 84 pages, sha256 927afc1315d0e15d34fc2a4cfb62d7cb510036e903306c07db146c61e6a94417).
- Edition: Local Rules of Bankruptcy Procedure 'Revised August 22, 2022' (title page), linked as 'Local Rules' at https://www.txeb.uscourts.gov/court-info/local-rules-and-orders. OLD EDITION: the general orders page (https://www.txeb.uscourts.gov/court-info/local-rules-and-orders/general-orders) lists later General Orders 22-2 (apportionment of adversary proceedings), 24-1 (fee payment), 26-1 and 26-2 (BNC outage); none amends the rules' text and none is vendored. The SBRA interim local rules file on that page is not vendored. robots.txt (Crawl-delay 10 honored) and the court's privacy/security policy page (https://www.txeb.uscourts.gov/privacy-policy/) checked: no prohibition of automated access.
- Retrieved 2026-10-28 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 48f55a499913430af1a3e9a9267ff87962d4c8012007f7847ca5635553e65e71. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `bankr-txe/original/txe-local-rules-2022-08-22.pdf` as served (sha256 927afc1315d0e15d34fc2a4cfb62d7cb510036e903306c07db146c61e6a94417); extraction sha256 48f55a499913430af1a3e9a9267ff87962d4c8012007f7847ca5635553e65e71.

### `bankr-txe/txe-local-rules-2022-08-22-sectioned.txt` — Bankr. E.D. Tex. Local Rules of Bankruptcy Procedure, sectioned

- Derived in the pod (lg-u561.17) from `bankr-txe/txe-local-rules-2022-08-22.txt` (extract sha256 48f55a499913430af1a3e9a9267ff87962d4c8012007f7847ca5635553e65e71); nothing fetched. One `## LBR <n>` line is inserted before each rule's own heading line in the body (a line opening, after an optional form feed, with `Local Rule of Bankruptcy Procedure <nnnn>-<n>.` or `<nnnn>.<n>-<n>.`; the edition has no table of contents, so the set is every such heading, from `1001-1` through `9027-1`, 63 rules); rule text was not edited: deleting the marker lines (`sed '/^## LBR /d'`) restores the extract byte for byte.
- Source: as the extract `bankr-txe/txe-local-rules-2022-08-22.txt`, https://www.txeb.uscourts.gov/sites/txeb/files/2022%20LOCAL%20RULES_rev%20as%20of%208-22-22.pdf, retrieved 2026-10-28; the sectioning was done in the pod from the vendored extract.
- Edition: as the extract.
- Sectioned sha256 7f5356644d3f66a359281f591b674b2aacfd903c796d81da92d7675261e0abc6 (the `path_sha256` pinned by `lit/bankr-txe/Lit.toml`).
