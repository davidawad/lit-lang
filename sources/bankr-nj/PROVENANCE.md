
### `bankr-nj/nj-local-rules-2026-08-01.txt` — D.N.J. Local Rules (dated August 1, 2026)

- Source: https://www.njb.uscourts.gov/sites/njb/files/Local%20Rules%20Package.pdf (official site; 112 pages, sha256 c5fb89b9fe2c86dee06533e891171ff23261471bbd8cfdd81075de5ed0be6e39).
- Edition: Local Rules 'Dated: August 1, 2026' (title page), the 'Local Rules Package.pdf' at https://www.njb.uscourts.gov/Local%20Rules%20Package; the general orders table (https://www.njb.uscourts.gov/General_Orders_Table) lists General Order 2025-05 (claims agents, pending adoption of a local rule), which precedes this edition. robots.txt (Crawl-delay 10 honored) and the court's privacy/security policy page (https://www.njb.uscourts.gov/privacy-policy/) checked: no prohibition of automated access.
- Retrieved 2026-10-28 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 281b232e9bd851d3e100ec5c26bb2b555ac6a9f4172340bb511c1e18e4f6e130. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `bankr-nj/original/nj-local-rules-2026-08-01.pdf` as served (sha256 c5fb89b9fe2c86dee06533e891171ff23261471bbd8cfdd81075de5ed0be6e39); extraction sha256 281b232e9bd851d3e100ec5c26bb2b555ac6a9f4172340bb511c1e18e4f6e130.

### `bankr-nj/nj-local-rules-2026-08-01-sectioned.txt` — D.N.J. Local Bankruptcy Rules, sectioned

- Derived in the pod (lg-u561.13) from `bankr-nj/nj-local-rules-2026-08-01.txt` (extract sha256 281b232e9bd851d3e100ec5c26bb2b555ac6a9f4172340bb511c1e18e4f6e130); nothing fetched. One `## Rule <n>` line is inserted before each rule's own heading line in the body (the form-fed `D.N.J. LBR 1001-1.` through `D.N.J. LBR 9037-1.`, 84 rules, `3002.1-1` among them); the table of contents is unmarked. Rule text was not edited: deleting the marker lines (`sed '/^## Rule /d'`) restores the extract byte for byte.
- Source: as the extract `bankr-nj/nj-local-rules-2026-08-01.txt`, https://www.njb.uscourts.gov/sites/njb/files/Local%20Rules%20Package.pdf, retrieved 2026-10-28; the sectioning was done in the pod from the vendored extract.
- Edition: as the extract, dated August 1, 2026, the effective date the 2026 Comment to LBR 2016-5 states.
- Sectioned sha256 81ad4aee1d4c78050a4499d5e0f961e5652bc5b537e2f5181e9474e629784f87 (the `path_sha256` pinned by `lit/bankr-nj/Lit.toml`).
