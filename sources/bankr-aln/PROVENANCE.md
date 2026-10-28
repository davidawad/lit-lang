
### `bankr-aln/alnb-lbr-2023-03-01.txt` — Local Rules of the U.S. Bankruptcy Court, Northern District of Alabama (March 1, 2023)

- Source: https://www.alnb.uscourts.gov/sites/alnb/files/Local%20Rules%20Mar%201%202023.pdf (official site; 35 pages, sha256 5e5857f6dbeb5df15e26ce8ede0d9f6c5d12cf3324149a89a1a4a202d8fc3e08).
- Edition: Local Rules, file name 'Local Rules Mar 1 2023' (no date on the cover), linked as 'click here' under Local Rules from https://www.alnb.uscourts.gov/court-info/local-rules-and-orders; cite as 'Bankr. N.D. Ala. R. [Number]'. The court's combined Administrative Orders file (AdminOrdersAllCombinedFile_09302026) was fetched and checked: it contains no amendments to the local rules text. robots.txt (Crawl-delay 10 honored; no rule covering these paths) and the Privacy Policy checked: no prohibition of automated access. Fetched with curl and the identifying User-Agent lit-lang-provenance/1.0 (angrybohr@proton.me).
- Retrieved 2026-10-28 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 59d0f79054dcc82329f1e7dad2edcdb4fb49c4e26c2f347ab9f01cf70c1954b5. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `bankr-aln/original/alnb-lbr-2023-03-01.pdf` as served (sha256 5e5857f6dbeb5df15e26ce8ede0d9f6c5d12cf3324149a89a1a4a202d8fc3e08); extraction sha256 59d0f79054dcc82329f1e7dad2edcdb4fb49c4e26c2f347ab9f01cf70c1954b5.

### `bankr-aln/alnb-lbr-2023-03-01-sectioned.txt` — N.D. Ala. Local Rules, sectioned

- Derived in the pod (lg-u561.29) from `bankr-aln/alnb-lbr-2023-03-01.txt` (extract sha256 59d0f79054dcc82329f1e7dad2edcdb4fb49c4e26c2f347ab9f01cf70c1954b5); nothing fetched. One `## Rule <n>` line is inserted before each rule's own heading line in the body (`RULE 1001-1 PURPOSE AND SCOPE OF RULES` through `RULE 9070-1 EXHIBITS`, including `RULE 4001-1.1` and the headings that follow a form feed; 23 sections, matching the table of contents); the cover and table of contents (extract lines 1-77) and Appendices A-C are unmarked. Rule text was not edited: deleting the marker lines (`sed '/^## Rule /d'`) restores the extract byte for byte.
- Edition: as the extract, the Local Rules effective March 1, 2023, with each rule's own effective-date note (the latest, October 1, 2025, recording Rule 1071-1's abrogation).
- Sectioned sha256 c77145ef8f68fef90b0bb137cb3cffcdd7e3626ed97d50e7656e44a55def26e8 (the `path_sha256` pinned by `lit/bankr-aln/Lit.toml` for source `alnb-lbr`).
