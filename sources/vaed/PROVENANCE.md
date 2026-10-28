
### `vaed/vaed-local-rules.txt` — U.S. District Court for the Eastern District of Virginia Local Rules

- Source: https://www.vaed.uscourts.gov/sites/vaed/files/Local%20Rules%20EDVA%20December%201%202024.pdf (official site; 97 pages, sha256 b2f28b233e18ec56ec75dfdf7176716bcd737000d9aabfddb94199272e419c7f).
- Edition: Local Rules, Effective December 1, 2024 (cover; https://www.vaed.uscourts.gov/court-info/local-rules-and-orders states the Local Rules were last amended on December 1, 2024).
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 6e6029e9380ef2b8fcb6f6223feefceb534e396329adc40fe0a91431c52233e8. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `vaed/original/vaed-local-rules.pdf` as served (sha256 b2f28b233e18ec56ec75dfdf7176716bcd737000d9aabfddb94199272e419c7f); extraction sha256 6e6029e9380ef2b8fcb6f6223feefceb534e396329adc40fe0a91431c52233e8.

### `vaed/vaed-local-rules-sectioned.txt` — E.D. Va. Local Rules, sectioned

- Derived in the pod (lg-f3q8.31) from `vaed/vaed-local-rules.txt` (extract sha256 6e6029e9380ef2b8fcb6f6223feefceb534e396329adc40fe0a91431c52233e8); nothing fetched. One `## Civil Rule <n>` line is inserted before each Local Civil Rule's own heading line in the body (`LOCAL CIVIL RULE 1` through `LOCAL CIVIL RULE 83.6`, 29 rules); the table of contents and the criminal and admiralty rules are unmarked. Rule text was not edited: deleting the marker lines (`sed '/^## Civil Rule /d'`) restores the extract byte for byte.
- Edition: as the extract, effective December 1, 2024.
- Sectioned sha256 3cd040ce347ecaff738f2a084443752f628dbe1a70b186113d16391e9ab11fcd (the `path_sha256` pinned by `lit/vaed/Lit.toml`).
