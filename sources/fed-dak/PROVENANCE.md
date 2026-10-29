
### `fed-dak/alaska-local-civil-rules.txt` — D. Alaska Local Civil Rules

- Source: https://www.akd.uscourts.gov/sites/akd/files/local_rules/Local_Civil_Rules_10-2025.pdf (official site; 51 pages, sha256 50afcdec4b0ef640e65dadd7e755de9d62216e1692ba5bb86a0bbc280997b692).
- Edition: D. Alaska Local Civil Rules, 2025 Revised Edition, effective October 15, 2025 (supersedes January 2023 edition), linked from https://www.akd.uscourts.gov/court-info/local-rules-and-orders/local-rules. Fetched with curl (robots.txt crawl-delay 10; one request).
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 aba6826276b70db05b43cb2af84c2da67756f10bc0846ecf3db0bc6ab2b00a78. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `fed-dak/original/alaska-local-civil-rules.pdf` as served (sha256 50afcdec4b0ef640e65dadd7e755de9d62216e1692ba5bb86a0bbc280997b692); extraction sha256 aba6826276b70db05b43cb2af84c2da67756f10bc0846ecf3db0bc6ab2b00a78.

### `fed-dak/alaska-local-civil-rules-sectioned.txt` — D. Alaska Local Civil Rules, sectioned

- Derived in the pod (lg-f3q8.35) from `fed-dak/alaska-local-civil-rules.txt` (extract sha256 aba6826276b70db05b43cb2af84c2da67756f10bc0846ecf3db0bc6ab2b00a78); nothing fetched. One `## Rule <n>` line is inserted before each Local Civil Rule's own heading line in the body (`Local Civil Rule 1.1` through `Local Civil Rule 86.1`, 60 rules); the summary of amendments, the table of contents and the Rule 7.5 exemplar heading are unmarked. Rule text was not edited: deleting the marker lines (`sed '/^## Rule /d'`) restores the extract byte for byte.
- Source: as the extract `fed-dak/alaska-local-civil-rules.txt`, https://www.akd.uscourts.gov/sites/akd/files/local_rules/Local_Civil_Rules_10-2025.pdf, retrieved 2026-10-27; the sectioning was done in the pod from the vendored extract.
- Edition: as the extract, 2025 Revised Edition, effective October 15, 2025.
- Sectioned sha256 469ee51d39279b959cdfbee39085d4bfe241b77e91980beba7a840ab93e987b1 (the `path_sha256` pinned by `lit/akd/Lit.toml`).

### `fed-dak/dak-local-criminal-rules-2025-10-15.txt` — D. Alaska Local Criminal Rules (effective October 15, 2025)

- Source: https://www.akd.uscourts.gov/sites/akd/files/local_rules/Local_Criminal_Rules_10-2025_0.pdf (official site; 27 pages, sha256 c3f9e80d4ad5762414e7758715d179d982f0367ae258bedfba84fe0fbff809db).
- Edition: Local Criminal Rules, 2025 Revised Edition effective October 15, 2025, superseding the December 2020 edition (title page), listed as 'Criminal (effective October 15, 2025)' at https://www.akd.uscourts.gov/court-info/local-rules-and-orders/local-rules. robots.txt of www.akd.uscourts.gov read (Crawl-delay 10 honored where stated; no Disallow covers the fetched path); the judiciary-wide Privacy & Security Policy (https://www.uscourts.gov/privacy-security-policy) read: no prohibition of automated access. Fetched with curl and the identifying UA lit-lang-provenance/1.0, at most 1 request per 10 s.
- Criminal forum (lg-gddt): akd-criminal.
- Retrieved 2026-10-28 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 2dc784b33cd13751d9027158501d6f631ffc0cdd8deae47b8d0b27bccad66dfe. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `fed-dak/original/dak-local-criminal-rules-2025-10-15.pdf` as served (sha256 c3f9e80d4ad5762414e7758715d179d982f0367ae258bedfba84fe0fbff809db); extraction sha256 2dc784b33cd13751d9027158501d6f631ffc0cdd8deae47b8d0b27bccad66dfe.

### `fed-dak/dak-local-criminal-rules-2025-10-15-sectioned.txt` — D. Alaska Local Criminal Rules (effective October 15, 2025), sectioned

- Derived in the pod (lg-gddt.22) from `fed-dak/dak-local-criminal-rules-2025-10-15.txt` (extract sha256 2dc784b33cd13751d9027158501d6f631ffc0cdd8deae47b8d0b27bccad66dfe); nothing fetched. One marker line `## L.Crim.R. <n>` is inserted before each rule's own body heading line `Local Criminal Rule <n> <Title>` (some led by a form feed; L.Crim.R. 1.1 through 59.1, 25 rules, matching the table of contents). The summary of amendments and the table of contents carry no markers. Every other line is the extract unchanged, so deleting the marker lines (`sed '/^## L.Crim.R. /d'`) gives back the extract byte for byte.
- Source: as the extract `fed-dak/dak-local-criminal-rules-2025-10-15.txt`, https://www.akd.uscourts.gov/sites/akd/files/local_rules/Local_Criminal_Rules_10-2025_0.pdf; Retrieved 2026-10-28 with the extract (this file was derived from it in the pod, not fetched itself).
- Edition: as the extract, 2025 Revised Edition effective October 15, 2025.
- Sectioned sha256 602e11209821a43c38590b4fad2d1b28b87ef0d9e817d22e33df0e3e3d1061d4 (the `path_sha256` pinned by `lit/akd-criminal/Lit.toml`).
