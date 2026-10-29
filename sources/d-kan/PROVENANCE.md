
### `d-kan/d-kan-local-rules-2026.txt` — D. Kan. Rules of Practice (local rules)

- Source: https://www.ksd.uscourts.gov/sites/ksd/files/MASTER%20COPY%20updated%204%2027%2026.pdf (official site; 99 pages, sha256 887b23a5a4f9a432dd85a828884b0d48482f0ee061d3fc3a5898467a9043aeab).
- Edition: Rules of Practice, linked as "Local Rules for Kansas District Court Effective April 1, 2026" from https://www.ksd.uscourts.gov/local-rules (file named "MASTER COPY updated 4 27 26"); fetched with plain curl (no browser identity); robots.txt allows the path (Crawl-delay 10 honored).
- Also the criminal local rules (lg-gddt): 'PART XV      RULES APPLICABLE TO CRIMINAL CASES' part, from extract line 36.
- Criminal forum (lg-gddt): ksd-criminal.
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 fad3f1414bc1e8663cc176e3c1be624325c6cea7164717a2c312a83bf4e68160. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `d-kan/original/d-kan-local-rules-2026.pdf` as served (sha256 887b23a5a4f9a432dd85a828884b0d48482f0ee061d3fc3a5898467a9043aeab); extraction sha256 fad3f1414bc1e8663cc176e3c1be624325c6cea7164717a2c312a83bf4e68160.

### `d-kan/d-kan-local-rules-2026-sectioned.txt` — D. Kan. Rules of Practice, sectioned

- Derived in the pod (lg-f3q8.67) from `d-kan/d-kan-local-rules-2026.txt` (extract sha256 fad3f1414bc1e8663cc176e3c1be624325c6cea7164717a2c312a83bf4e68160); nothing fetched. One `## D. Kan. Rule <n>` line is inserted right after each civil rule's own body heading line `RULE <n>` (111 markers, 1.1 through 83.8.13, including 23-A and 71A.1). The table of rules, the criminal rules (CR) and the standing orders are unmarked. Rule text was not edited: deleting the marker lines (each `## D. Kan. Rule ` line with its newline) restores the extract byte for byte.
- Source: as the extract `d-kan/d-kan-local-rules-2026.txt`, https://www.ksd.uscourts.gov/sites/ksd/files/MASTER%20COPY%20updated%204%2027%2026.pdf, retrieved 2026-10-27; the sectioning was done in the pod from the vendored extract.
- Edition: as the extract, Rules of Practice effective April 1, 2026.
- Sectioned sha256 aa997f61649e526629529161843e01d98c8691d08fce5efbb564616e30266c19 (the `path_sha256` pinned by `lit/ksd/Lit.toml`).

### `d-kan/d-kan-local-rules-2026-criminal-sectioned.txt` — D. Kan. Rules of Practice, criminal rules (Part XV) sectioned

- Derived in the pod (lg-gddt.24) from `d-kan/d-kan-local-rules-2026.txt` (extract sha256 fad3f1414bc1e8663cc176e3c1be624325c6cea7164717a2c312a83bf4e68160); nothing fetched. In Part XV, Rules Applicable to Criminal Cases, one marker line `## D. Kan. Rule CR<n>` is inserted right after each rule's own body heading line `RULE CR<n>` (13 markers, D. Kan. Rule CR1.1 through CR58.1). The table of rules, the civil rules and the standing orders are unmarked (CR58.1's section runs to the end of the file); the civil rules are cited from `d-kan/d-kan-local-rules-2026-sectioned.txt`. Rule text was not edited: deleting the marker lines (each `## D. Kan. Rule CR` line with its newline) restores the extract byte for byte.
- Source: as the extract `d-kan/d-kan-local-rules-2026.txt`, https://www.ksd.uscourts.gov/sites/ksd/files/MASTER%20COPY%20updated%204%2027%2026.pdf; Retrieved 2026-10-27 with the extract (this file was derived from it in the pod, not fetched itself).
- Edition: as the extract, Rules of Practice effective April 1, 2026.
- Sectioned sha256 ad37189826fba458eb85b5bf9d6b84f00150aef91ed5865c83c4e3452305e66d (the `path_sha256` pinned by `lit/ksd-criminal/Lit.toml`).
