
### `dri/dri-local-rules.txt` — D.R.I. Local Rules

- Source: https://www.rid.uscourts.gov/sites/rid/files/documents/Approved%20Dec%202024%20Rules.pdf (official site; 179 pages, sha256 7f03a8fca5401c942b3bbb962180aa9d352237aa3a18be9a89f381d3d98c038e).
- Edition: "Effective January 1, 2006, As Amended through April 1, 2024" (as stated on the document; page https://www.rid.uscourts.gov/local-rules-and-procedures lists only "Local Rules for the District of Rhode Island - 4/1/2024"; the file name says "Approved Dec 2024" but the text does not, so currency after April 2024 is unconfirmed).
- Also the criminal local rules (lg-gddt): 'LOCAL RULES APPLICABLE TO CRIMINAL PROCEEDINGS' part, from extract line 424.
- Criminal forum (lg-gddt): rid-criminal.
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 f76a78f126cd994184efad9f82002456c6f464acb2e53885909ad861065dfa54. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `dri/original/dri-local-rules.pdf` as served (sha256 7f03a8fca5401c942b3bbb962180aa9d352237aa3a18be9a89f381d3d98c038e); extraction sha256 f76a78f126cd994184efad9f82002456c6f464acb2e53885909ad861065dfa54.

### `dri/dri-local-rules-civil-sectioned.txt` — D.R.I. Local Rules, civil rules sectioned

- Derived in the pod (lg-f3q8.26) from `dri/dri-local-rules.txt` (extract sha256 f76a78f126cd994184efad9f82002456c6f464acb2e53885909ad861065dfa54); nothing fetched. One `## LR Cv <n>` line is inserted right after each Local Rule applicable to civil proceedings' own body heading line (`LR Cv <n> <TITLE>`): 34 markers, LR Cv 5 through LR Cv 81. The table of contents and the general, admiralty and criminal rules are unmarked (LR Cv 81's section runs to the end of the file). Rule text was not edited: deleting the marker lines (`sed '/^## LR Cv /d'`) restores the extract byte for byte.
- Upstream: https://www.rid.uscourts.gov/sites/rid/files/documents/Approved%20Dec%202024%20Rules.pdf, retrieved 2026-10-27, as recorded on the `dri/dri-local-rules.txt` entry this file is derived from.
- Edition: as the extract, "Effective January 1, 2006, As Amended through April 1, 2024"; the court's local-rules page lists that edition ("Local Rules for the District of Rhode Island - 4/1/2024") as current.
- Sectioned sha256 d7336fb266673f689ca05837e781a88deb245825225c4e4e4f74b79be56d7abe (the `path_sha256` pinned by `lit/rid/Lit.toml`).

### `dri/dri-local-criminal-rules-sectioned.txt` — D.R.I. Local Rules applicable to criminal proceedings, sectioned

- Derived in the pod (lg-gddt.6) from `dri/dri-local-rules.txt` (extract sha256 f76a78f126cd994184efad9f82002456c6f464acb2e53885909ad861065dfa54); nothing fetched. One `## LR Cr <n>` line is inserted before each criminal rule's own body heading line (`LR Cr <n>   <TITLE>`): 16 markers, LR Cr 6 through LR Cr 58, matching the table of contents. The table of contents and the general, civil and admiralty rules are unmarked (LR Cr 58's section runs to the end of the file, through the Alternative Dispute Resolution Plan). Rule text was not edited: deleting the marker lines (`sed '/^## LR Cr /d'`) restores the extract byte for byte.
- Edition: as the extract, "Effective January 1, 2006, As Amended through April 1, 2024", the edition the court's local-rules page lists (LR Cr 12(c) amended effective April 1, 2024).
- Sectioned sha256 a0a615d278f9442bcf6deac6eeb503cbfa8b82a7850ad6c450ab7a1c550d3114 (the `path_sha256` pinned by `lit/rid-criminal/Lit.toml`).
