
### `fed-cdcal/cdcal-local-civil-rules.txt` — C.D. Cal. Local Civil Rules (Chapter I)

- Source: https://www.cacd.uscourts.gov/sites/default/files/documents/2026-June-LRs-Chap-1.pdf (official site; 166 pages, sha256 d71c1f15e1552ef8d6ea3a22149b7375a40946fe36b325c47973cf00299c9ac6).
- Edition: C.D. Cal. Local Rules, Chapter I Local Civil Rules, as last amended June 1, 2026 per https://www.cacd.uscourts.gov/court-procedures/local-rules (Chapters II-IV, admiralty, criminal and bankruptcy, not vendored). Fetched with curl (robots.txt allows).
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 1eb7e8cadb675fd88a04bb96865b758dca0bf6f913f441050114dc3f466989c7. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `fed-cdcal/original/cdcal-local-civil-rules.pdf` as served (sha256 d71c1f15e1552ef8d6ea3a22149b7375a40946fe36b325c47973cf00299c9ac6); extraction sha256 1eb7e8cadb675fd88a04bb96865b758dca0bf6f913f441050114dc3f466989c7.

### `fed-cdcal/cdcal-local-civil-rules-sectioned.txt` — C.D. Cal. Local Civil Rules (Chapter I), sectioned

- Derived in the pod (lg-f3q8.39) from `fed-cdcal/cdcal-local-civil-rules.txt` (extract sha256 1eb7e8cadb675fd88a04bb96865b758dca0bf6f913f441050114dc3f466989c7); nothing fetched. One `## C.D. Cal. L.R. <n>` line is inserted right before each rule's and subrule's own body heading line (`L.R. <n> <title>`, from L.R. 1-1 after the table of contents; 501 markers, 1-1 through 85-1, subrules such as 54-2.1 included). The table of contents is unmarked. Rule text was not edited: deleting the marker lines (each `## C.D. Cal. L.R. ` line with its newline) restores the extract byte for byte.
- Edition: as the extract, Chapter I as last amended June 1, 2026.
- Sectioned sha256 20b6c1ebe427503f2d1fd25117d2968671a56df4b6415e669dd65e90775527cf (the `path_sha256` pinned by `lit/cacd/Lit.toml`).
