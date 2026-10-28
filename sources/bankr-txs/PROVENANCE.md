
### `bankr-txs/txs-bankruptcy-local-rules-2025-09-22.txt` — S.D. Tex. Bankruptcy Local Rules (effective September 22, 2025)

- Source: https://www.txs.uscourts.gov/sites/txs/files/Local_Rules_September_22_2025.pdf (official site; 41 pages, sha256 bd61c3522e19172d161048029a94da06793479b0928c46dd069f9cb3b9844db1).
- Edition: Bankruptcy Local Rules 'Effective September 22, 2025' (document), the 'Local Rules: Current bankruptcy local rules' link on https://www.txs.uscourts.gov/page/bankruptcy-court (the bankruptcy court shares the district court's host; the district civil rules at /sites/txs/files/LR_March_2026.pdf were not fetched). Pending proposals listed on the home page (new Local Rule 1 on assignment of cases; local prize rules) are not in force and not vendored. robots.txt (Crawl-delay 10 honored) and the court site has no usable policy page of its own (it links no policy page), so the judiciary-wide Privacy & Security Policy (https://www.uscourts.gov/privacy-security-policy, with the robots.txt of www.uscourts.gov) was checked: no prohibition of automated access.
- Retrieved 2026-10-28 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 3b048d25496bf8520c7313e9ca0f743ae1b7f93a345280d4075f4f574f189105. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `bankr-txs/original/txs-bankruptcy-local-rules-2025-09-22.pdf` as served (sha256 bd61c3522e19172d161048029a94da06793479b0928c46dd069f9cb3b9844db1); extraction sha256 3b048d25496bf8520c7313e9ca0f743ae1b7f93a345280d4075f4f574f189105.

### `bankr-txs/txs-bankruptcy-local-rules-2025-09-22-sectioned.txt` — Bankr. S.D. Tex. Bankruptcy Local Rules, sectioned

- Derived in the pod (lg-u561.17) from `bankr-txs/txs-bankruptcy-local-rules-2025-09-22.txt` (extract sha256 3b048d25496bf8520c7313e9ca0f743ae1b7f93a345280d4075f4f574f189105); nothing fetched. One `## BLR <n>` line is inserted before each rule's own heading line in the body (a line opening at the margin, after an optional form feed, with `Local Rule <nnnn>-<n>.`, or `Rule 9017-1.` for the one rule headed without `Local`; the edition has no table of contents, so the set is every such heading, from `1001-1` through `9037-1`, 50 rules; indented cross-references are unmarked). Rule text was not edited: deleting the marker lines (`sed '/^## BLR /d'`) restores the extract byte for byte.
- Edition: as the extract.
- Sectioned sha256 ffbf381754336221d4f61b05c651da8d516913c10dbae9b4dd6836e43bea44b7 (the `path_sha256` pinned by `lit/bankr-txs/Lit.toml`).
