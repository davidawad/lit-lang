
### `fed-dak/alaska-local-civil-rules.txt` — D. Alaska Local Civil Rules

- Source: https://www.akd.uscourts.gov/sites/akd/files/local_rules/Local_Civil_Rules_10-2025.pdf (official site; 51 pages, sha256 50afcdec4b0ef640e65dadd7e755de9d62216e1692ba5bb86a0bbc280997b692).
- Edition: D. Alaska Local Civil Rules, 2025 Revised Edition, effective October 15, 2025 (supersedes January 2023 edition), linked from https://www.akd.uscourts.gov/court-info/local-rules-and-orders/local-rules. Fetched with curl (robots.txt crawl-delay 10; one request).
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 aba6826276b70db05b43cb2af84c2da67756f10bc0846ecf3db0bc6ab2b00a78. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `fed-dak/original/alaska-local-civil-rules.pdf` as served (sha256 50afcdec4b0ef640e65dadd7e755de9d62216e1692ba5bb86a0bbc280997b692); extraction sha256 aba6826276b70db05b43cb2af84c2da67756f10bc0846ecf3db0bc6ab2b00a78.

### `fed-dak/alaska-local-civil-rules-sectioned.txt` — D. Alaska Local Civil Rules, sectioned

- Derived in the pod (lg-f3q8.35) from `fed-dak/alaska-local-civil-rules.txt` (extract sha256 aba6826276b70db05b43cb2af84c2da67756f10bc0846ecf3db0bc6ab2b00a78); nothing fetched. One `## Rule <n>` line is inserted before each Local Civil Rule's own heading line in the body (`Local Civil Rule 1.1` through `Local Civil Rule 86.1`, 60 rules); the summary of amendments, the table of contents and the Rule 7.5 exemplar heading are unmarked. Rule text was not edited: deleting the marker lines (`sed '/^## Rule /d'`) restores the extract byte for byte.
- Edition: as the extract, 2025 Revised Edition, effective October 15, 2025.
- Sectioned sha256 469ee51d39279b959cdfbee39085d4bfe241b77e91980beba7a840ab93e987b1 (the `path_sha256` pinned by `lit/akd/Lit.toml`).
