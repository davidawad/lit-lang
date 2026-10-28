
### `bankr-pae/pae-local-rules-2025-12-01.txt` — E.D. Pa. Local Bankruptcy Rules (effective December 1, 2025)

- Source: https://www.paeb.uscourts.gov/sites/paeb/files/PAEB%20Local%20Rules%20-2025-12-01%20Final.pdf (official site; 93 pages, sha256 43a272bf93df2977b0f85958e77b35c1400feb7c76a9aebeee2daddff9343595).
- Edition: Local Bankruptcy Rules '(Effective December 1, 2025)' (title page), linked as 'Local Bankruptcy Rules' at https://www.paeb.uscourts.gov/court-info/local-rules-and-orders, with the district court's order of November 14, 2025 approving the changes (not vendored). robots.txt (Crawl-delay 10 honored) and the court's privacy/security policy page (https://www.paeb.uscourts.gov/privacy-policy/) checked: no prohibition of automated access.
- Retrieved 2026-10-28 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 731b67a9ff4cab1a22dd531a7b645f409ed6315721a2a9008e7f5dc7d8b41000. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `bankr-pae/original/pae-local-rules-2025-12-01.pdf` as served (sha256 43a272bf93df2977b0f85958e77b35c1400feb7c76a9aebeee2daddff9343595); extraction sha256 731b67a9ff4cab1a22dd531a7b645f409ed6315721a2a9008e7f5dc7d8b41000.

### `bankr-pae/pae-local-rules-2025-12-01-sectioned.txt` — E.D. Pa. Local Bankruptcy Rules, sectioned

- Derived in the pod (lg-u561.13) from `bankr-pae/pae-local-rules-2025-12-01.txt` (extract sha256 731b67a9ff4cab1a22dd531a7b645f409ed6315721a2a9008e7f5dc7d8b41000); nothing fetched. One `## Rule <n>` line is inserted before each rule's own `Local Rule <n>` heading line in the body (`1001-1` through `9076-1`, 116 rules, `3017.1-1` and `3017.1-2` among them; the heading of 1017-1 is split as `Local Rule` over `1017-1 Conversion of a Case`, and that of 9010-1 carries its title on the line, so their markers were placed by hand before those lines); the table of contents is unmarked. Rule text was not edited: deleting the marker lines (`sed '/^## Rule /d'`) restores the extract byte for byte.
- Source: as the extract `bankr-pae/pae-local-rules-2025-12-01.txt`, https://www.paeb.uscourts.gov/sites/paeb/files/PAEB%20Local%20Rules%20-2025-12-01%20Final.pdf, retrieved 2026-10-28; the sectioning was done in the pod from the vendored extract.
- Edition: as the extract, effective December 1, 2025.
- Sectioned sha256 4f601914873bbea3a4847eeb0f9a1bb86a2504261067b9212f057f2e08795a3a (the `path_sha256` pinned by `lit/bankr-pae/Lit.toml`).
