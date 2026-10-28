
### `ca6-bap/bap-local-rules.txt` — Sixth Circuit BAP Local Rules

- Source: https://www.ca6.uscourts.gov/sites/ca6/files/documents/bap/BapLocalRules.pdf (official site; 4 pages, sha256 b876663352688b7d68812b83e8905d0624f212098622d56a52a7d5c28ad7910a).
- Edition: Local Rules, Bankruptcy Appellate Panel of the Sixth Circuit, "As Amended December 1, 2014" as stated on the document; linked from https://www.ca6.uscourts.gov/rules-and-procedures/bap-local-rules.
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 c25a1aa073f84c22d575884b82b9600b321e5ec5234b7ae2adfa4c0de5898434. Raw extract, not yet sectioned into `## <ref>` form.
- Fetched via curl (single document, first request succeeded, no block encountered; desktop UA, no Referer).
- Original: `ca6-bap/original/bap-local-rules.pdf` as served (sha256 b876663352688b7d68812b83e8905d0624f212098622d56a52a7d5c28ad7910a); extraction sha256 c25a1aa073f84c22d575884b82b9600b321e5ec5234b7ae2adfa4c0de5898434.

### `ca6-bap/bap-local-rules-sectioned.txt` — Sixth Circuit BAP Local Rules, sectioned

- Derived in the pod (lg-f3q8.57) from `ca6-bap/bap-local-rules.txt` (extract sha256 c25a1aa073f84c22d575884b82b9600b321e5ec5234b7ae2adfa4c0de5898434); nothing fetched. One `## 6th Cir. BAP LBR <n>` line is inserted before each of the 14 rule heading lines (`RULE 8001 - 1.` ... `RULE 8090- 1.`). The marker writes the rule number without the spaces of the form Rule 8026-1 prescribes ("6th Cir. BAP LBR 8014 - 1(a)" is marked `## 6th Cir. BAP LBR 8014-1`), the heading the cite parser keys `6th Cir. BAP LBR 8014-1(b)` to. Rule text was not edited: deleting the marker lines (`sed '/^## 6th Cir. BAP LBR /d'`) restores the extract byte for byte.
- Source: as the extract `ca6-bap/bap-local-rules.txt`, https://www.ca6.uscourts.gov/sites/ca6/files/documents/bap/BapLocalRules.pdf, retrieved 2026-10-27; the sectioning was done in the pod from the vendored extract.
- Edition: as the extract, "As Amended December 1, 2014".
- Sectioned sha256 7d911dce19117a07e8727571daa8b04777289258f86d857bedb829ca177ee816 (the `path_sha256` pinned by `lit/bap6/Lit.toml`).
