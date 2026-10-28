
### `ca2/local-rules-iop.txt` — Second Circuit Local Rules, IOPs and Appendices

- Source: https://www.ca2.uscourts.gov/clerk/case_filing/rules/pdf/LRs_IOCs_appendices_rev_2024.pdf (official site; 50 pages, sha256 403c42363fc121db6354dd08c9f9b439926b741a2af351c76dffea6bb1569629).
- Edition: Local Rules and Internal Operating Procedures of the Court of Appeals for the Second Circuit, "Effective December 2, 2024" as stated on the document, linked as the current edition from https://www.ca2.uscourts.gov/clerk/case_filing/rules/rules_home.html (no later edition listed there); includes Appendix Parts A-B.
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 1e4be10c7f48dea090b58dc7f360d9da1e406aae44335485b6703bb6723209b2. Sectioned for cite resolution (lg-f3q8.50): a `## 2d Cir. R. N` marker line was inserted before the heading line (`Local Rule N ...`) of each numbered local rule, and a `## 2d Cir. IOP N` marker before each numbered IOP heading; 50 markers. The lettered IOPs (A-I) and the appendices are unmarked. Rule text was not edited: deleting the marker lines (`sed '/^## 2d Cir\. \(R\.\|IOP\) /d'`) restores the extract byte for byte.
- Sectioned file sha256 ff0cede23c4c87ab4cc5cfd9352bed27b12a9e9dbb176e0c70993ca7d3e2085f (the `path_sha256` pinned by `lit/ca2/Lit.toml`).
- Original: `ca2/original/local-rules-iop.pdf` as served (sha256 403c42363fc121db6354dd08c9f9b439926b741a2af351c76dffea6bb1569629); extraction sha256 1e4be10c7f48dea090b58dc7f360d9da1e406aae44335485b6703bb6723209b2.
