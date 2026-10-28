
### `de-superior-civil/rules-civ-2026-03.txt` — Delaware Superior Court Civil Rules (consolidated)

- Source: https://courts.delaware.gov/forms/download.aspx?id=304488 (official site; 117 pages, sha256 22ec3a0556439cef64be7bc81efa054093dfc04d3c4f7f421ab56c2cdb0f3ca5).
- Edition: Rules of Civil Procedure for the Superior Court of the State of Delaware, PDF created Mar. 23, 2026 (no edition line on the document), as linked "Rules of Civil Procedure, Superior Court of Delaware" on https://courts.delaware.gov/rules/ (retrieved 2026-10-27). Stale in part: it does not reflect two later amending orders listed on that page, vendored as separate documents: de-superior-civil/order-2026-03-13-rules-5g-37e-141-146.txt (order of Mar. 13, 2026, amending Rules 5(g) and 37(e) and creating Rules 141-146) and de-superior-civil/order-2026-09-30-rule-26b.txt (order of Sept. 30, 2026, amending Rule 26(b)).
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 368f275e77064126652b54e31bc783955b9675043704e87dd56f1fb20d14a329. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `de-superior-civil/original/rules-civ-2026-03.pdf` as served (sha256 22ec3a0556439cef64be7bc81efa054093dfc04d3c4f7f421ab56c2cdb0f3ca5); extraction sha256 368f275e77064126652b54e31bc783955b9675043704e87dd56f1fb20d14a329.

### `de-superior-civil/order-2026-03-13-rules-5g-37e-141-146.txt` — Delaware Superior Court order amending Civil Rules 5(g), 37(e), creating 141-146 (Mar. 13, 2026)

- Source: https://courts.delaware.gov/forms/download.aspx?id=322898 (official site; 19 pages, sha256 9955216baf9554563c5d69e5a5e9707ca2795f0d06839ed6432546b79e2edb1c).
- Edition: Order of Mar. 13, 2026 amending Superior Court Civil Rules 5(g) and 37(e) and creating Rules 141-146, as linked on https://courts.delaware.gov/rules/ (retrieved 2026-10-27). Amending order to the consolidated rules (de-superior-civil/rules-civ-2026-03.txt), which does not reflect it.
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 f430f16d86707c30d20e34dd8da8211385913c8db48180f0966d96d0ef575bce. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `de-superior-civil/original/order-2026-03-13-rules-5g-37e-141-146.pdf` as served (sha256 9955216baf9554563c5d69e5a5e9707ca2795f0d06839ed6432546b79e2edb1c); extraction sha256 f430f16d86707c30d20e34dd8da8211385913c8db48180f0966d96d0ef575bce.

### `de-superior-civil/order-2026-09-30-rule-26b.txt` — Delaware Superior Court order amending Civil Rule 26(b) (Sept. 30, 2026)

- Source: https://courts.delaware.gov/forms/download.aspx?id=345478 (official site; 4 pages, sha256 7f4c8c66297f8096fef17cfb73699954f4c97796219318806a7aec68a0a97362).
- Edition: Order of Sept. 30, 2026 amending Superior Court Civil Rule 26(b), as linked "Amendment to Superior Court Civil Rules to Amend Rules Rule 26(b)" on https://courts.delaware.gov/rules/ (retrieved 2026-10-27). Amending order to the consolidated rules (de-superior-civil/rules-civ-2026-03.txt), which does not reflect it.
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 e26a1b5ccf1a557a02db384c93dfd3839a5ab88e04c723de5ddedbc667aa6083. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `de-superior-civil/original/order-2026-09-30-rule-26b.pdf` as served (sha256 7f4c8c66297f8096fef17cfb73699954f4c97796219318806a7aec68a0a97362); extraction sha256 e26a1b5ccf1a557a02db384c93dfd3839a5ab88e04c723de5ddedbc667aa6083.

### `de-superior-civil/rules-civ-2026-03-sectioned.txt` — Delaware Superior Court Civil Rules, sectioned

- Derived in the pod (lg-f3q8.73) from `de-superior-civil/rules-civ-2026-03.txt` (extract sha256 368f275e77064126652b54e31bc783955b9675043704e87dd56f1fb20d14a329); nothing fetched. One `## Del. Super. Ct. Civ. R. <n>` line is inserted right after each rule's own body heading line `Rule <n>. <title>` (136 markers, Rule 1 through Rule 140, including 58A-58C, 71A and 104A; decimal rules such as 16.1 and 54.1 are unmarked and fall within the preceding rule's section). Rule text was not edited: deleting the marker lines (`## Del. Super. Ct. Civ. R. <n>` plus its newline) restores the extract byte for byte.
- Edition: as the extract, the consolidated rules of Mar. 2026. The amending orders of Mar. 13, 2026 (Rules 5(g), 37(e), 141-146) and Sept. 30, 2026 (Rule 26(b)) are not merged into it; `lit/de-superior-civil` cites none of the provisions they amend from this text.
- Sectioned sha256 107f229c9ddead76c1d5c790c449ab610a4559cd7240fcccc4ef3ba0d2f6ff20 (the `path_sha256` pinned by `lit/de-superior-civil/Lit.toml`).
