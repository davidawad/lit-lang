
### `dpr/dpr-local-rules-2024-10-16.txt` — D.P.R. Local Rules

- Source: https://www.prd.uscourts.gov/sites/default/files/local_rules/Local%20Rules%2010-16-2024-amended-rule-83A.pdf (official site; 145 pages, sha256 60feba0066bf4a790c5539cbb5f4b3900cc25733e3b842fc4069049052f7a7fb).
- Edition: Local Rules (Civil, Admiralty, Criminal) effective with amendments through October 16, 2024, as stated on the document, linked from https://www.prd.uscourts.gov/court-info/local-rules-and-orders/local-rules. Local Civil Rule 83A(a)(2) is amended by the separate October 17, 2025 order (dpr-order-2025-10-17-lcvr-83a.txt); apply it on top. robots.txt (Crawl-delay 10 honored) and privacy policy checked: no prohibition of automated access..
- Also the criminal local rules (lg-gddt): 'CRIMINAL RULES' part, from extract line 3845.
- Criminal forum (lg-gddt): prd-criminal.
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 a32ae1fdfaf10bba8cbbb2f4e9645122b197805ea8cf019030b6bfb27be0ae1a. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `dpr/original/dpr-local-rules-2024-10-16.pdf` as served (sha256 60feba0066bf4a790c5539cbb5f4b3900cc25733e3b842fc4069049052f7a7fb); extraction sha256 a32ae1fdfaf10bba8cbbb2f4e9645122b197805ea8cf019030b6bfb27be0ae1a.

### `dpr/dpr-order-2025-10-17-lcvr-83a.txt` — D.P.R. Order of Final Adoption, LCvR 83A(a)(2)

- Source: https://www.prd.uscourts.gov/sites/default/files/general-ordes/Order%20of%20Final%20Adoption%20re%20Loc%20Cv%20R%2083A%28a%29%282%29%20October%2017%202025%20Rev%202.pdf (official site; 2 pages, sha256 71d9388f5b85e87f63172d745e5bb9a2c7934a6cd1b2e20a5a444778ebb3b932).
- Edition: Order of Final Adoption of Amendments to Local Civil Rule 83A(a)(2), October 17, 2025 (Misc. No. 03-115 (RAM)), linked from https://www.prd.uscourts.gov/court-info/local-rules-and-orders. Amends dpr-local-rules-2024-10-16.txt; apply on top. robots.txt (Crawl-delay 10 honored) and privacy policy checked: no prohibition of automated access..
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 b615fdbfc4ae444bc33993764cedc76bc91b5fe61424d5666c2e426b41c60c42. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `dpr/original/dpr-order-2025-10-17-lcvr-83a.pdf` as served (sha256 71d9388f5b85e87f63172d745e5bb9a2c7934a6cd1b2e20a5a444778ebb3b932); extraction sha256 b615fdbfc4ae444bc33993764cedc76bc91b5fe61424d5666c2e426b41c60c42.

### `dpr/dpr-local-rules-2024-10-16-sectioned.txt` — D.P.R. Local Rules, civil rules sectioned

- Derived in the pod (lg-f3q8.25) from `dpr/dpr-local-rules-2024-10-16.txt` (extract sha256 a32ae1fdfaf10bba8cbbb2f4e9645122b197805ea8cf019030b6bfb27be0ae1a); nothing fetched. One `## Rule <n>` line is inserted right after each Local Civil Rule's own body heading line `RULE <n>`: 46 markers, Rule 1 through Rule 83M. The table of contents, the Admiralty Rules and the Criminal Rules are unmarked. Rule text was not edited: deleting the marker lines (`## Rule <n>` plus its newline) restores the extract byte for byte.
- Upstream: https://www.prd.uscourts.gov/sites/default/files/local_rules/Local%20Rules%2010-16-2024-amended-rule-83A.pdf, retrieved 2026-10-27, as recorded on the `dpr/dpr-local-rules-2024-10-16.txt` entry this file is derived from.
- Edition: as the extract, the Local Rules with amendments through October 16, 2024.
- Sectioned sha256 de899cedeabd0acaf69fd75bce7eecdccba923c8a175c0608bddab6df1b72a1a (the `path_sha256` pinned by `lit/prd/Lit.toml`).

### `dpr/dpr-local-criminal-rules-2024-10-16-sectioned.txt` — D.P.R. Local Criminal Rules, sectioned

- Derived in the pod (lg-gddt.6) from `dpr/dpr-local-rules-2024-10-16.txt` (extract sha256 a32ae1fdfaf10bba8cbbb2f4e9645122b197805ea8cf019030b6bfb27be0ae1a); nothing fetched. One `## Rule <n>` line is inserted right after each Local Criminal Rule's own body heading line `RULE <n>` in the Criminal Rules part (21 markers, Rule 101 through Rule 162, including 116A, 116B, 116C, 117.1, 132.1 and 144A), as the civil sectioning does. The table of contents, the Local Civil Rules and the Admiralty Rules are unmarked (Rule 162's section runs to the end of the file). Rule text was not edited: deleting the marker lines (`sed '/^## Rule /d'`) restores the extract byte for byte.
- Edition: as the extract, the Local Rules with amendments through October 16, 2024 (Rules 132, 132.1 and 144 marked "amended on 05/14/2023"). The October 17, 2025 order amends only Local Civil Rule 83A(a)(2).
- Source: as the extract, https://www.prd.uscourts.gov/sites/default/files/local_rules/Local%20Rules%2010-16-2024-amended-rule-83A.pdf, retrieved 2026-10-27; the sectioning was done from the vendored extract.
- Sectioned sha256 f6acb7ed2ce1fa4da7abaae1e6ac84a0337e245faae58690499bb419c8cf1b9f (the `path_sha256` pinned by `lit/prd-criminal/Lit.toml`).
