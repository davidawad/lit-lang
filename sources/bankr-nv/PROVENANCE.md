
### `bankr-nv/nvb-lbr-2021-03-02.txt` — Local Rules of Bankruptcy Practice, U.S. Bankruptcy Court for the District of Nevada (updated March 2, 2021)

- Source: https://www.nvb.uscourts.gov/downloads/rules/local-rules-2021_final.pdf (official site; 80 pages, sha256 593752267d4ea8c0ea864f1c0e3ab62a97f6b0b6f0ab757ffdcab6a19a19a1d8).
- Edition: Local Rules of Bankruptcy Practice 'Updated March 2, 2021' (cover), linked as 'Local Rules - Complete Set' from https://www.nvb.uscourts.gov/rules-forms/rules/local-rules/. The court's orders page (https://www.nvb.uscourts.gov/rules-forms/rules/administrative-orders/) lists Administrative Order 2021-04 (Order Amending Local Rule 9018, 03/02/2021) as the latest local-rule amendment, which is the date of this edition; no later one. no robots.txt is served (HTTP 200 with a 404 error page), and no privacy, terms or site-policy page is linked from the site; checked the site pages, nothing found that prohibits automated access, and so no prohibition of automated access was found. Fetched with curl and the identifying User-Agent lit-lang-provenance/1.0 (angrybohr@proton.me).
- Retrieved 2026-10-28 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 430465d1d24f8cada2f2dfa79d752d153307c08e0fd301d6673d6ce683e4e0fc. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `bankr-nv/original/nvb-lbr-2021-03-02.pdf` as served (sha256 593752267d4ea8c0ea864f1c0e3ab62a97f6b0b6f0ab757ffdcab6a19a19a1d8); extraction sha256 430465d1d24f8cada2f2dfa79d752d153307c08e0fd301d6673d6ce683e4e0fc.

### `bankr-nv/nvb-lbr-2021-03-02-sectioned.txt` — Bankr. D. Nev. Local Rules of Bankruptcy Practice, sectioned

- Derived in the pod (lg-u561.25) from `bankr-nv/nvb-lbr-2021-03-02.txt` (extract sha256 430465d1d24f8cada2f2dfa79d752d153307c08e0fd301d6673d6ce683e4e0fc); nothing fetched. One `## LBR <n>` line is inserted before each rule's own heading line in the body (a line opening, after an optional form feed, with `LR <nnnn>.` or `LR <nnnn>.<n>` and the upper-case rule title, the marker carrying the rule number as written (`## LBR 9014.1`); the set is every such heading after the table of contents, from `1001` through `9037`, 106 rules); rule text was not edited: deleting the marker lines (`sed '/^## LBR /d'`) restores the extract byte for byte.
- Source: as the extract `bankr-nv/nvb-lbr-2021-03-02.txt`, https://www.nvb.uscourts.gov/downloads/rules/local-rules-2021_final.pdf, retrieved 2026-10-28; the sectioning was done in the pod from the vendored extract.
- Edition: as the extract.
- Sectioned sha256 074469c0dd5610978bb8baa25196f851d4ff268b9f0d1a7e5be802d9e489aaac (the `path_sha256` pinned by `lit/bankr-nv/Lit.toml`).
