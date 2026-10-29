
### `fed-cdcal/cdcal-local-civil-rules.txt` — C.D. Cal. Local Civil Rules (Chapter I)

- Source: https://www.cacd.uscourts.gov/sites/default/files/documents/2026-June-LRs-Chap-1.pdf (official site; 166 pages, sha256 d71c1f15e1552ef8d6ea3a22149b7375a40946fe36b325c47973cf00299c9ac6).
- Edition: C.D. Cal. Local Rules, Chapter I Local Civil Rules, as last amended June 1, 2026 per https://www.cacd.uscourts.gov/court-procedures/local-rules (Chapters II-IV, admiralty, criminal and bankruptcy, not vendored). Fetched with curl (robots.txt allows).
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 1eb7e8cadb675fd88a04bb96865b758dca0bf6f913f441050114dc3f466989c7. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `fed-cdcal/original/cdcal-local-civil-rules.pdf` as served (sha256 d71c1f15e1552ef8d6ea3a22149b7375a40946fe36b325c47973cf00299c9ac6); extraction sha256 1eb7e8cadb675fd88a04bb96865b758dca0bf6f913f441050114dc3f466989c7.

### `fed-cdcal/cdcal-local-civil-rules-sectioned.txt` — C.D. Cal. Local Civil Rules (Chapter I), sectioned

- Derived in the pod (lg-f3q8.39) from `fed-cdcal/cdcal-local-civil-rules.txt` (extract sha256 1eb7e8cadb675fd88a04bb96865b758dca0bf6f913f441050114dc3f466989c7); nothing fetched. One `## C.D. Cal. L.R. <n>` line is inserted right before each rule's and subrule's own body heading line (`L.R. <n> <title>`, from L.R. 1-1 after the table of contents; 501 markers, 1-1 through 85-1, subrules such as 54-2.1 included). The table of contents is unmarked. Rule text was not edited: deleting the marker lines (each `## C.D. Cal. L.R. ` line with its newline) restores the extract byte for byte.
- Source: as the extract `fed-cdcal/cdcal-local-civil-rules.txt`, https://www.cacd.uscourts.gov/sites/default/files/documents/2026-June-LRs-Chap-1.pdf, retrieved 2026-10-27; the sectioning was done in the pod from the vendored extract.
- Edition: as the extract, Chapter I as last amended June 1, 2026.
- Sectioned sha256 20b6c1ebe427503f2d1fd25117d2968671a56df4b6415e669dd65e90775527cf (the `path_sha256` pinned by `lit/cacd/Lit.toml`).

### `fed-cdcal/cdcal-local-criminal-rules-chap-iii-2025-12.txt` — C.D. Cal. Local Rules Chapter III, Local Criminal Rules (December 2025 file)

- Source: https://www.cacd.uscourts.gov/sites/default/files/documents/2025%20December%20LRs%20Chap%203.pdf (official site; 31 pages, sha256 7dfff103913ab1a60867ba248e07028bcec86640c8a596625a2196903493421e).
- Edition: Chapter III Local Criminal Rules of the Local Rules of the Central District of California, the PDF named '2025 December LRs Chap 3' linked at https://www.cacd.uscourts.gov/court-procedures/local-rules (Chapter I, the June 2026 civil rules, and Chapter II, admiralty, are separate files; the civil chapter is the document already vendored as cdcal-local-civil-rules.txt). robots.txt of www.cacd.uscourts.gov read (Crawl-delay 10 honored where stated; no Disallow covers the fetched path); the judiciary-wide Privacy & Security Policy (https://www.uscourts.gov/privacy-security-policy) read: no prohibition of automated access. Fetched with curl and the identifying UA lit-lang-provenance/1.0, at most 1 request per 10 s.
- Criminal forum (lg-gddt): cacd-criminal.
- Retrieved 2026-10-28 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 a358ca020b9c70833a117cff36581aa28ff7af5efe87f9f8ca43e02b0bfadabf. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `fed-cdcal/original/cdcal-local-criminal-rules-chap-iii-2025-12.pdf` as served (sha256 7dfff103913ab1a60867ba248e07028bcec86640c8a596625a2196903493421e); extraction sha256 a358ca020b9c70833a117cff36581aa28ff7af5efe87f9f8ca43e02b0bfadabf.

### `fed-cdcal/cdcal-local-criminal-rules-chap-iii-2025-12-sectioned.txt` — C.D. Cal. Local Criminal Rules (Chapter III, December 2025), sectioned

- Derived in the pod (lg-gddt.22) from `fed-cdcal/cdcal-local-criminal-rules-chap-iii-2025-12.txt` (extract sha256 a358ca020b9c70833a117cff36581aa28ff7af5efe87f9f8ca43e02b0bfadabf); nothing fetched. One marker line `## L.Cr.R. <n>` is inserted before each rule's own body line that opens with `L.Cr.R. <n> <Title>` (89 rules and subrules, L.Cr.R. 5-1 through 61-1, matching the table of contents); the table of contents and in-text references carry no markers. Every other line is the extract unchanged, so deleting the marker lines gives back the extract byte for byte.
- Source: as the extract `fed-cdcal/cdcal-local-criminal-rules-chap-iii-2025-12.txt`, https://www.cacd.uscourts.gov/sites/default/files/documents/2025%20December%20LRs%20Chap%203.pdf; Retrieved 2026-10-28 with the extract (this file was derived from it in the pod, not fetched itself).
- Edition: as the extract, Chapter III with every page dated 12/1/2025.
- Sectioned sha256 6785b590c664e4e3f29b634ab4228efea8e6038b0864c711cbdeb18fc18dc471 (the `path_sha256` pinned by `lit/cacd-criminal/Lit.toml`).
