
### `laed/laed-local-civil-rules.txt` — E.D. La. Local Civil Rules (LR, LAR)

- Source: https://www.laed.uscourts.gov/sites/default/files/local_rules/2022%20CIVIL%20RULES%20LAED%20w%20Amendment%202%204%2025.pdf (official site; 54 pages, sha256 429a9a4f8516ec33dd72578520ade3d5be1592da301fd60d66a2b4b40f250725).
- Edition: Effective March 1, 2022 (Revised February 4, 2025), as stated on the document and linked from https://www.laed.uscourts.gov/attorney-information/rules-and-orders/local-rules. EDLA-only text; lamd and lawd have their own separate local rules (not shared), vendored under forums lamd and lawd. robots.txt (Crawl-delay 10 honored) and privacy/security policy checked: no prohibition of automated access.
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 01ab2f682c80118c6cded012ee2a3ef043c57fc0cb77a866329783b9958938cf. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `laed/original/laed-local-civil-rules.pdf` as served (sha256 429a9a4f8516ec33dd72578520ade3d5be1592da301fd60d66a2b4b40f250725); extraction sha256 01ab2f682c80118c6cded012ee2a3ef043c57fc0cb77a866329783b9958938cf.

### `laed/laed-local-criminal-rules.txt` — E.D. La. Local Criminal Rules (LCrR)

- Source: https://www.laed.uscourts.gov/sites/default/files/local_rules/2022%20CRIMINAL%20RULES%20LAED%20w%20Amendments%203.1.22.pdf (official site; 14 pages, sha256 70dc701024aae4682a84b6e5ed7525c21066634cc46681035404def2bca3e422).
- Edition: Effective March 1, 2022 (amendments 3.1.22), as stated on the document and linked from https://www.laed.uscourts.gov/attorney-information/rules-and-orders/local-rules. Companion to laed-local-civil-rules.txt; not shared with lamd or lawd.
- Also the criminal local rules (lg-gddt): this document is the E.D. La. Local Criminal Rules (LCrR); the civil rules are laed-local-civil-rules.txt.
- Criminal forum (lg-gddt): laed-criminal.
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 1041b19c19a1bd73076bd6a8ef24a5647832bf4872d2a9524985969f52dae22c. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `laed/original/laed-local-criminal-rules.pdf` as served (sha256 70dc701024aae4682a84b6e5ed7525c21066634cc46681035404def2bca3e422); extraction sha256 1041b19c19a1bd73076bd6a8ef24a5647832bf4872d2a9524985969f52dae22c.

### `laed/laed-local-civil-rules-sectioned.txt` — E.D. La. Local Civil Rules, sectioned

- Derived in the pod (lg-f3q8.18) from `laed/laed-local-civil-rules.txt` (extract sha256 01ab2f682c80118c6cded012ee2a3ef043c57fc0cb77a866329783b9958938cf); nothing fetched. In the body (after the table of contents), one `## LR <n>` line is inserted before each local civil rule's own heading line (`LR3.1 Collateral Proceedings and Refiled Cases` through `LR 83.5.1 Cases Transferred to a District Court Outside of the Fifth Circuit`, 124 rules; the heading line of each rule's last occurrence, after the rule group's list of its rules). The Local Admiralty Rules (LAR) are not sectioned. Every other line is the extract unchanged, so deleting the `## LR` lines gives back the extract byte for byte.
- Edition: as the extract, "EFFECTIVE March 1, 2022 (Revised February 4, 2025)".
- Sectioned sha256 c2bf4474907de81650a8beebc324cf8ff031d80abdbf1243fc2580e09c34019a (the `path_sha256` pinned by `lit/laed/Lit.toml`).
