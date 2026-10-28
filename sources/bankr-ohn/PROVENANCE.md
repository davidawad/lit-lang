
### `bankr-ohn/ohn-local-rules-2022-05-19.txt` — N.D. Ohio Local Bankruptcy Rules (last revised May 19, 2022)

- Source: https://www.ohnb.uscourts.gov/sites/default/files/file-list/local-bankruptcy-rules-northern-district-ohio.pdf (official site; 44 pages, sha256 2b6401935f93f0d041edc24e3673dc8e85343f46b51f6d00221634c3d71e6d9d).
- Edition: Local Bankruptcy Rules, revision history ending 'May 19, 2022' (title page), the 'local-bankruptcy-rules-northern-district-ohio.pdf' file at https://www.ohnb.uscourts.gov/file-list/local-bankruptcy-rules; the general orders page (https://www.ohnb.uscourts.gov/judges-info/general-orders) lists no order amending the rules. robots.txt (Crawl-delay 10 honored) and the court's privacy/security policy page (https://www.ohnb.uscourts.gov/security-and-privacy-notice) checked: no prohibition of automated access.
- Retrieved 2026-10-28 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 3f3987732724715dd25a1bcf85d2188ab0927b02a0d56a5e18340b6fb950fb7e. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `bankr-ohn/original/ohn-local-rules-2022-05-19.pdf` as served (sha256 2b6401935f93f0d041edc24e3673dc8e85343f46b51f6d00221634c3d71e6d9d); extraction sha256 3f3987732724715dd25a1bcf85d2188ab0927b02a0d56a5e18340b6fb950fb7e.

### `bankr-ohn/ohn-local-rules-2022-05-19-sectioned.txt` — N.D. Ohio Local Bankruptcy Rules, sectioned

- Derived in the pod (lg-u561.19) from `bankr-ohn/ohn-local-rules-2022-05-19.txt` (extract sha256 3f3987732724715dd25a1bcf85d2188ab0927b02a0d56a5e18340b6fb950fb7e); nothing fetched. One `## Rule <n>` line is inserted before each rule's own heading line in the body (`Rule <n> <TITLE>`, and the two headings written `Local Rule 7067-1` and `9036-1`; `Rule 1001-1` through `Rule 9074-1`, including the headings that begin with a form feed; 40 sections, matching the table of contents); the title page and table of contents (extract lines 1-105) are unmarked. Rule text was not edited: deleting the marker lines (`sed '/^## /d'`) restores the extract byte for byte.
- Source: as the extract `bankr-ohn/ohn-local-rules-2022-05-19.txt`, https://www.ohnb.uscourts.gov/sites/default/files/file-list/local-bankruptcy-rules-northern-district-ohio.pdf, retrieved 2026-10-28; the sectioning was done in the pod from the vendored extract.
- Edition: as the extract, the Local Bankruptcy Rules last revised May 19, 2022.
- Sectioned sha256 180e7538f8f7b295a5cf5fb71603dade61dab696c055a41864a5215af8f5d085 (the `path_sha256` pinned by `lit/bankr-ohn/Lit.toml` for source `ohnb-lbr`).
