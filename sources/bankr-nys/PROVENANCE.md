
### `bankr-nys/nys-local-rules-2024-09-30.txt` — S.D.N.Y. Local Bankruptcy Rules (effective September 30, 2024)

- Source: https://www.nysb.uscourts.gov/sites/default/files/2024-09-30_Amended_LBRs_Final.pdf (official site; 103 pages, sha256 1f400089afde4ccc0cafa425542d135c91115587d636e1779ae626b2c949a37d).
- Edition: Local Bankruptcy Rules 'EFFECTIVE September 30, 2024' (title page), linked as 'Local Rules effective September 30, 2024' at https://www.nysb.uscourts.gov/content/local-rules; no later amendment is listed there. robots.txt (Crawl-delay 10 honored) and the court's privacy/security policy page (https://www.nysb.uscourts.gov/privacy-policy/) checked: no prohibition of automated access.
- Retrieved 2026-10-28 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 dfc7d531a3dcb166f2365d13c0c8478dccf963f93bb9b7fffd19bd100f48fb64. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `bankr-nys/original/nys-local-rules-2024-09-30.pdf` as served (sha256 1f400089afde4ccc0cafa425542d135c91115587d636e1779ae626b2c949a37d); extraction sha256 dfc7d531a3dcb166f2365d13c0c8478dccf963f93bb9b7fffd19bd100f48fb64.

### `bankr-nys/nys-local-rules-2024-09-30-sectioned.txt` — S.D.N.Y. Local Bankruptcy Rules, sectioned

- Derived in the pod (lg-u561.11) from `bankr-nys/nys-local-rules-2024-09-30.txt` (extract sha256 dfc7d531a3dcb166f2365d13c0c8478dccf963f93bb9b7fffd19bd100f48fb64); nothing fetched. One `## LBR <n>` line is inserted before each rule's own heading line in the body (`Rule 1001-1 SHORT TITLE; APPLICABILITY; ADDITIONAL RESOURCES` through `Rule 9078-1`, including the dotted `2007.1-1`, `2007.2-1` and `4001-1.1` and the indented `Rule 5009-2`; 112 sections, matching the table of contents); the table of contents is unmarked. Rule text was not edited: deleting the marker lines (`sed '/^## LBR /d'`) restores the extract byte for byte.
- Source: as the extract `bankr-nys/nys-local-rules-2024-09-30.txt`, https://www.nysb.uscourts.gov/sites/default/files/2024-09-30_Amended_LBRs_Final.pdf, retrieved 2026-10-28; the sectioning was done in the pod from the vendored extract.
- Edition: as the extract, effective September 30, 2024.
- Sectioned sha256 682d326741397a6374e0e768376ffc942b4a2538e3d60b16063eb04b211504b2 (the `path_sha256` pinned by `lit/bankr-nys/Lit.toml` for source `nysb-lbr`).
