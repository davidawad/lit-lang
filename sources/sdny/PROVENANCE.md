
### `sdny/local-rules-joint-sdny-edny.txt` — SDNY and EDNY Joint Local Civil and Criminal Rules

- Source: https://www.nysd.uscourts.gov/sites/default/files/local_rules/2026-01-02%20-%20EDNY%20and%20SDNY%20Joint%20Local%20Rules%20As%20Amended.pdf (official site; 144 pages, sha256 439b9e2c67fd4ca1d1be65a7f28ca53707820bcbabb4b459323ade1a773607af).
- Edition: "Effective January 2, 2026" as stated on the document, listed as current on https://www.nysd.uscourts.gov/rules; joint Local Rules of the Southern and Eastern Districts of New York, vendored once under sdny and also the EDNY (forum edny) local rules.
- Also the criminal local rules (lg-gddt): 'LOCAL CRIMINAL RULES' part, from extract line 3609. Also governs the Eastern District of New York.
- Criminal forum (lg-gddt): nysd-criminal, nyed-criminal.
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 e8d380e1439c011acf9154422ac44e93ab4b84456c5248b88254136ed47d30d8. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `sdny/original/local-rules-joint-sdny-edny.pdf` as served (sha256 439b9e2c67fd4ca1d1be65a7f28ca53707820bcbabb4b459323ade1a773607af); extraction sha256 e8d380e1439c011acf9154422ac44e93ab4b84456c5248b88254136ed47d30d8.

### `sdny/local-rules-joint-sdny-edny-criminal-sectioned.txt` — SDNY and EDNY Joint Local Criminal Rules, sectioned

- Derived in the pod (lg-gddt.8) from `sdny/local-rules-joint-sdny-edny.txt` (extract sha256 e8d380e1439c011acf9154422ac44e93ab4b84456c5248b88254136ed47d30d8); nothing fetched. In the body of the Local Criminal Rules (after the `LOCAL CRIMINAL RULES` heading at extract line 3525, before the Appendix of Committee Notes), one `## Local Criminal Rule <n>` line is inserted before each rule's own heading line `Local Criminal Rule <n>. <title>` (14 rules: 1.1, 1.2, 12.4, 16.1, 16.2, 23.1, 34.1, 45.1 [withdrawn], 47.1, 49.1, 49.2, 58.1, 59.1, 62.1). The tables of contents, the civil rules and the committee notes are unmarked. Rule text was not edited: deleting the marker lines (`sed '/^## Local Criminal Rule /d'`) restores the extract byte for byte.
- Edition: as the extract, effective January 2, 2026; the Southern and the Eastern District share it.
- Upstream: https://www.nysd.uscourts.gov/sites/default/files/local_rules/2026-01-02%20-%20EDNY%20and%20SDNY%20Joint%20Local%20Rules%20As%20Amended.pdf, retrieved 2026-10-27, as recorded on `sdny/local-rules-joint-sdny-edny.txt` above; sectioning was done from the vendored extract.
- Sectioned sha256 6f1a7ac5c06034a7e203c3dfbd8898cfeaab5cc55bc5e06e61f0bb5d9a779628 (the `path_sha256` pinned by `lit/nysd-criminal/Lit.toml` and `lit/nyed-criminal/Lit.toml`).

### `sdny/local-rules-joint-sdny-edny-sectioned.txt` — S.D.N.Y./E.D.N.Y. Joint Local Rules, civil and Social Security rules sectioned

- Derived in the pod (lg-f3q8.51) from `sdny/local-rules-joint-sdny-edny.txt` (extract sha256 e8d380e1439c011acf9154422ac44e93ab4b84456c5248b88254136ed47d30d8), which was retrieved 2026-10-27 from https://www.nysd.uscourts.gov/sites/default/files/local_rules/2026-01-02%20-%20EDNY%20and%20SDNY%20Joint%20Local%20Rules%20As%20Amended.pdf as recorded above; nothing fetched in the pod. Method: one `## Local Civil Rule <n>` line is inserted before each Local Civil Rule's own heading line in the rules body (Local Civil Rule 1.1 through 87.1, 71 rules, dotted numbers such as `1.4.1`, `7.1.1` and `65.1.1` kept; the withdrawn Local Civil Rule 6.4 "Computation of Time" heading is unmarked, so `## Local Civil Rule 6.4` is the in-force Motions in Limine rule), and one `## Local Social Security Rule <n>` line before each of the five Local Social Security Rules' headings (1.1, 4.1, 5.1, 7.1, 8.1). The contents, the admiralty, criminal and patent rules and the Appendix of Committee Notes are unmarked. Rule text was not edited: deleting the marker lines (`sed '/^## Local /d'`) restores the extract byte for byte.
- Edition: as the extract, effective January 2, 2026. Also the E.D.N.Y. local rules (lit/nyed).
- Sectioned sha256 f12fd4d7c006ddd44ff015761aaaaaf13090db10db0afb4660c08f16f6812b98 (the `path_sha256` pinned by `lit/nysd/Lit.toml` and `lit/nyed/Lit.toml`).
