
### `cgcca/cgcca-rules-jrap-2025-03.txt` — CGCCA Rules of Appellate Procedure with JRAP

- Source: https://www.uscg.mil/Portals/0/Headquarters/Legal/cca/CGCCA%20Rules%20of%20Appellate%20Procedure%20with%20JRAP%20(March%202025).pdf (linked from https://www.uscg.mil/Resources/Legal/Court-of-Criminal-Appeals/) (official site; 33 pages, sha256 8d4d0ee1e4d1c512a34530cd87e9e877611358088aa9ccd39fc8593d5b3d024e).
- Edition: CGCCA Rules of Appellate Procedure, Rev. 03/25, published together with the Joint Rules of Appellate Procedure effective 27 May 2024; the court's page labels it 'MAR 2025' and links no later edition.
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 722c1c9a158d535dd681add788049a434d7d7fc4a84aee8ef55a8710eba55521. Raw extract, not yet sectioned into `## <ref>` form.
- Fetch method: BrowserOS neo; site served a bot challenge to plain automated requests (CoS ruling 2026-10-27). uscg.mil robots.txt does not disallow /Portals/; its Disclaimer page bars no automated access.
- Original: `cgcca/original/cgcca-rules-jrap-2025-03.pdf` as served (sha256 8d4d0ee1e4d1c512a34530cd87e9e877611358088aa9ccd39fc8593d5b3d024e); extraction sha256 722c1c9a158d535dd681add788049a434d7d7fc4a84aee8ef55a8710eba55521.

### `cgcca/cgcca-rules-jrap-2025-03-sectioned.txt` — CGCCA Rules of Appellate Procedure with JRAP, sectioned

- Derived in the pod (lg-f3q8.134) from `cgcca/cgcca-rules-jrap-2025-03.txt` (extract sha256 722c1c9a158d535dd681add788049a434d7d7fc4a84aee8ef55a8710eba55521); nothing fetched. One marker line is inserted right after each rule's own body heading line: `## JRAP <n>` after each Joint Rule's all-capitals heading `Rule <n>. <TITLE>` (32 markers, JRAP 1 through 32), and `## CGCCA R. <n.m>` after each Coast Guard rule's heading `Rule <n.m>. <Title>` (38 markers), so a Joint Rule's section ends where the Coast Guard's own rule begins. The table of contents is unmarked. Rule text was not edited: deleting the marker lines (each `## ` line plus its newline) restores the extract byte for byte.
- The Joint Rules of Appellate Procedure for Courts of Criminal Appeals (as amended 17 May 2024, effective 27 May 2024, JRAP 4) govern all four Courts of Criminal Appeals (JRAP 1(a), 2(a)); `lit/acca` cites them from this file because the Army Court's own publication is not vendored (lg-okuc/#21).
- Upstream: as recorded on the `cgcca/cgcca-rules-jrap-2025-03.txt` entry this file is derived from.
- Sectioned sha256 8f268f8fd8b83c513b99608e7e960366739e7f9cd138f90957e8aaf9bce82a92 (the `path_sha256` pinned by `lit/acca/Lit.toml` and `lit/cgcca/Lit.toml`, whose `cgcca-rules` binding reads the `
