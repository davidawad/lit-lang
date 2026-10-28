
### `dnh/dnh-local-rules.txt` — D.N.H. Local Rules (with Supplemental Rules)

- Source: https://www.nhd.uscourts.gov/pdf/2025%20Local%20Rules%20-%20District%20of%20New%20Hampshire.pdf (official site; 171 pages, sha256 0c8a71f758a67912be5801f49cfdd23f9071dd7799b80c1c4e86a8f3feb94390).
- Edition: Local Rules (PDF Version), "effective December 1, 2025 (includes Supplemental Rules)" per https://www.nhd.uscourts.gov/Local-Rules-0.
- Also the criminal local rules (lg-gddt): Local Criminal Rules of the District of New Hampshire (LCrR), from extract line 4578.
- Criminal forum (lg-gddt): nhd-criminal.
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 b0f302edf1ea01da5bfe79ba0f91665a4660797e0f998610f68179066c89957d. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `dnh/original/dnh-local-rules.pdf` as served (sha256 0c8a71f758a67912be5801f49cfdd23f9071dd7799b80c1c4e86a8f3feb94390); extraction sha256 b0f302edf1ea01da5bfe79ba0f91665a4660797e0f998610f68179066c89957d.

### `dnh/dnh-local-rules-sectioned.txt` — D.N.H. Local Rules, civil rules sectioned

- Derived in the pod (lg-f3q8.24) from `dnh/dnh-local-rules.txt` (extract sha256 b0f302edf1ea01da5bfe79ba0f91665a4660797e0f998610f68179066c89957d); nothing fetched. One `## LR <n>` line is inserted before each civil Local Rule's own body heading line (`1.1      General Rules` through `83.17   [reserved]`, 96 rules). The table of contents, the Supplemental Rules and the Local Criminal Rules are unmarked. Rule text was not edited: deleting the marker lines (`sed '/^## LR /d'`) restores the extract byte for byte.
- Upstream: https://www.nhd.uscourts.gov/pdf/2025%20Local%20Rules%20-%20District%20of%20New%20Hampshire.pdf, retrieved 2026-10-27, as recorded on the `dnh/dnh-local-rules.txt` entry this file is derived from.
- Edition: as the extract, the Local Rules as amended through December 1, 2025.
- Sectioned sha256 ca001157d492636c92d6d8ff4c7112664bef52554253ba1125c167a6ef1d6382 (the `path_sha256` pinned by `lit/nhd/Lit.toml`).

### `dnh/dnh-local-criminal-rules-sectioned.txt` — D.N.H. Local Criminal Rules (LCrR), sectioned

- Derived in the pod (lg-gddt.6) from `dnh/dnh-local-rules.txt` (extract sha256 b0f302edf1ea01da5bfe79ba0f91665a4660797e0f998610f68179066c89957d); nothing fetched. One `## LCrR <n>` line is inserted before each Local Criminal Rule's own body heading line in Part XII, Criminal Rules (`1.1      General Rules` through `58.1    Procedure for Misdemeanors & Other Petty Offenses`, 37 rules including 12.4.1 and 17.1.1 and the reserved and renumbered rules; the headings of 11.2, 16.3 and 32.4 follow a form feed, and the marker goes before that line). The table of contents, the civil Local Rules, the Supplemental Rules and the administrative procedures are unmarked (LCrR 58.1's section runs to the end of the file). Rule text was not edited: deleting the marker lines (`sed '/^## LCrR /d'`) restores the extract byte for byte.
- Edition: as the extract, the Local Rules as amended through December 1, 2025 (LCrR 1.1(b): "Effective January 1, 1996, as amended December 1, 2025").
- Source: as the extract, https://www.nhd.uscourts.gov/pdf/2025%20Local%20Rules%20-%20District%20of%20New%20Hampshire.pdf, retrieved 2026-10-27; the sectioning was done from the vendored extract.
- Sectioned sha256 e411bf616d6e3f2fc695871ea7b83a3c7db36592462b93667417d3508f6c6df4 (the `path_sha256` pinned by `lit/nhd-criminal/Lit.toml`).
