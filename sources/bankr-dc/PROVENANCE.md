
### `bankr-dc/dc-local-rules-2026-02-01.txt` — D.D.C. Bankruptcy Local Rules (February 1, 2026)

- Source: https://www.dcb.uscourts.gov/sites/dcb/files/EFF%202-1-26_CLEAN%20LBR.pdf (official site; 147 pages, sha256 40afdcad8546e6993b257eb23ed64b5bab5f5be296a019e9356514d328255686).
- Edition: Local Rules of the U.S. Bankruptcy Court for the District of Columbia 'in effect as of February 1, 2026' (title page), the 'Local Bankruptcy Rules as of February 1, 2026' link on https://www.dcb.uscourts.gov/local-rules-orders. General Order 2026-01 (signed January 15, 2026; new LBR 2016-3, revised 9013-1 and 9014-1, effective February 1, 2026) is incorporated. The later 2026 general orders listed at https://www.dcb.uscourts.gov/general-orders (2026-02 sealed documents, 2026-03 revoking GO 2020-4, 2026-04 BNC disruption) do not amend the rules and are not vendored. robots.txt (Crawl-delay 10 honored) and the court's privacy/security policy page (https://www.dcb.uscourts.gov/privacy-policy/) checked: no prohibition of automated access.
- Retrieved 2026-10-28 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 52303f1d7a7ae8b7a7c93e4c90653c0b76687206cd20201ffeecfb86c7b027ec. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `bankr-dc/original/dc-local-rules-2026-02-01.pdf` as served (sha256 40afdcad8546e6993b257eb23ed64b5bab5f5be296a019e9356514d328255686); extraction sha256 52303f1d7a7ae8b7a7c93e4c90653c0b76687206cd20201ffeecfb86c7b027ec.

### `bankr-dc/dc-local-rules-2026-02-01-sectioned.txt` — D.D.C. Bankruptcy Local Rules, sectioned

- Derived in the pod (lg-u561.7) from `bankr-dc/dc-local-rules-2026-02-01.txt` (extract sha256 52303f1d7a7ae8b7a7c93e4c90653c0b76687206cd20201ffeecfb86c7b027ec); nothing fetched. One `## LBR <n>` line (the rule number with an ASCII hyphen, e.g. `## LBR 9013-1`) is inserted before each rule's own heading line in the body (`RULE 1001–1 Scope; Title; ...` through `RULE 9073–1 Hearings`, 98 rules; the table of contents is unmarked, and Appendices A-F fall under the last marker and are not cited). Rule text was not edited: deleting the marker lines (`sed '/^## LBR /d'`) restores the extract byte for byte.
- Source: as the extract `bankr-dc/dc-local-rules-2026-02-01.txt`, https://www.dcb.uscourts.gov/sites/dcb/files/EFF%202-1-26_CLEAN%20LBR.pdf, retrieved 2026-10-28; the sectioning was done in the pod from the vendored extract.
- Edition: as the extract, the Local Bankruptcy Rules in effect as of February 1, 2026 (General Order 2026-01 incorporated); no later order amends them, so `lit/bankr-dc` has no dated amendment layer.
- Sectioned sha256 a12e839bce3c8582facf5dc894dfde25db6c89412a1f9df664e0a6bd79f45410 (the `path_sha256` pinned by `lit/bankr-dc/Lit.toml`).
