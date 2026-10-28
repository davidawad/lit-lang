
### `txsd/tx-sd-local-rules.txt` — S.D. Tex. Local Rules

- Source: https://www.txs.uscourts.gov/sites/txs/files/LR_March_2026.pdf (official site; 43 pages, sha256 932940b61024065adaf0ac67efe16817871cdd41ca2398956352eea808c75eac).
- Edition: Local Rules of the S.D. Tex., effective May 1, 2000, "Reprinted March 2026" with amendments through May 25, 2023 per cover; linked from https://www.txs.uscourts.gov/page/local-federal-rules (robots.txt Crawl-delay 10 honored; no terms-of-use page found, only a privacy page about case files). Fetch method: curl with UA litgraph-rules-vendor/1.0, no bot challenge.
- Also the criminal local rules (lg-gddt): 'CRIMINAL RULES' part, from extract line 849.
- Criminal forum (lg-gddt): txsd-criminal.
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 241a3d09ca1137e0450a773bab0d3726190b17b1eb33e96065adabea19e7df1a. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `txsd/original/tx-sd-local-rules.pdf` as served (sha256 932940b61024065adaf0ac67efe16817871cdd41ca2398956352eea808c75eac); extraction sha256 241a3d09ca1137e0450a773bab0d3726190b17b1eb33e96065adabea19e7df1a.

### `txsd/tx-sd-local-rules-sectioned.txt` — S.D. Tex. Local Rules, criminal rules sectioned

- Derived in the pod (lg-gddt.14) from `txsd/tx-sd-local-rules.txt` (extract sha256 241a3d09ca1137e0450a773bab0d3726190b17b1eb33e96065adabea19e7df1a); nothing fetched. In the body of the Criminal Rules (after the table of contents, before Appendix A), one marker line `## CrLR <n>` is inserted before each rule group's own heading line (`CrLR6. GRAND JURY WITNESSES` through `CrLR58. PROCEDURE FOR MISDEMEANORS ...`, 11 groups) and one marker line `## Cr <n>` before each sub-rule's own heading line (`Cr12.1 Implementation` through `Cr58.1 Forfeiture of Collateral in Lieu of Appearance`, 28 sub-rules). The civil rules and appendices carry no markers. Every other line is the extract unchanged, so deleting the `## Cr` lines gives back the extract byte for byte.
- Edition: as the extract (effective May 1, 2000, "Reprinted March 2026", newest listed amendment LR 16.4 effective March 27, 2026).
- Criminal forum (lg-gddt): txsd-criminal.
- Sectioned sha256 da2c8f7e181658ea39b0f0dbe367930bb16f62b9cfe663232b0eecc3d40d4523 (the `path_sha256` pinned by `lit/txsd-criminal/Lit.toml`).
