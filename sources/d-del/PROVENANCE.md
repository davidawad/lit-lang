
### `d-del/local-rules-2026-09-01.txt` — District of Delaware Local Rules of Civil Practice and Procedure

- Source: https://www.ded.uscourts.gov/sites/ded/files/2026%20District%20of%20Delaware%20Revised%20Local%20Rules%20Effective%20September%201.pdf (official site; 47 pages, sha256 af3b341e6b0c9de46e2de2fb92447d654b44b46d4a82c1f9d1947dbb2270b769).
- Edition: Local Rules of Civil Practice and Procedure, amended effective September 1, 2026, as linked "2026 Revised Local Rules (Effective September 1, 2026)" on https://www.ded.uscourts.gov/local-rules (retrieved 2026-10-27); the page lists no later amendment. Redline of Feb. 2026 proposal not vendored.
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 5fff80527ba4ae3a38f0e3d377f193d260fa079099e091555e883aab32ae0ca4. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `d-del/original/local-rules-2026-09-01.pdf` as served (sha256 af3b341e6b0c9de46e2de2fb92447d654b44b46d4a82c1f9d1947dbb2270b769); extraction sha256 5fff80527ba4ae3a38f0e3d377f193d260fa079099e091555e883aab32ae0ca4.

### `d-del/ddel-lr-crim-speedy-trial-1987-amended-2011-05-02.txt` — D. Del. Time Limits and Procedures for Achieving Prompt Disposition of Criminal Cases (amended May 2, 2011)

- Source: https://www.ded.uscourts.gov/sites/ded/files/local-rules/LR-Crim.pdf (official site; 21 pages, sha256 1f9ba30eb7afcbba8bfda7839b108a81851f8ec8613b509a3e3fe503ff3a63ff).
- Edition: Section II of the District Plan for Disposition of Criminal Cases under the Speedy Trial Act, 'AMENDED MAY 2, 2011', the file LR-Crim linked as 'Time Limits and Procedures for Achieving Prompt Disposition of Criminal Cases' at https://www.ded.uscourts.gov/local-rules (the only criminal local-rule document on that page; the civil local rules effective September 1, 2026 are vendored as local-rules-2026-09-01.txt). robots.txt of www.ded.uscourts.gov read (Crawl-delay 10 honored where stated; no Disallow covers the fetched path); the judiciary-wide Privacy & Security Policy (https://www.uscourts.gov/privacy-security-policy) read: no prohibition of automated access. Fetched with curl and the identifying UA lit-lang-provenance/1.0, at most 1 request per 10 s.
- Criminal forum (lg-gddt): ded-criminal.
- Retrieved 2026-10-28 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 049fe639690eae8dd3b9c46057c9c26f152a935d37fc15fea5cd406103eb3ace. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `d-del/original/ddel-lr-crim-speedy-trial-1987-amended-2011-05-02.pdf` as served (sha256 1f9ba30eb7afcbba8bfda7839b108a81851f8ec8613b509a3e3fe503ff3a63ff); extraction sha256 049fe639690eae8dd3b9c46057c9c26f152a935d37fc15fea5cd406103eb3ace.

### `d-del/local-rules-2026-09-01-sectioned.txt` — D. Del. Local Rules of Civil Practice and Procedure, sectioned

- Derived in the pod (lg-f3q8.73) from `d-del/local-rules-2026-09-01.txt` (extract sha256 5fff80527ba4ae3a38f0e3d377f193d260fa079099e091555e883aab32ae0ca4); nothing fetched. One `## Rule <n>` line is inserted right after each rule's own body heading line `RULE <n>. <title>.` (68 markers, Rule 1.1 through Rule 83.7; the table of contents is unmarked). Rule text was not edited: deleting the marker lines (`## Rule <n>` plus its newline) restores the extract byte for byte.
- Upstream: https://www.ded.uscourts.gov/sites/ded/files/2026%20District%20of%20Delaware%20Revised%20Local%20Rules%20Effective%20September%201.pdf, retrieved 2026-10-27, as recorded on the `d-del/local-rules-2026-09-01.txt` entry this file is derived from.
- Edition: as the extract, the Local Rules of Civil Practice and Procedure amended effective September 1, 2026.
- Sectioned sha256 a8ac9b22563275310851fb21a4b43b82788a371e47869c64251ca6c1a837b967 (the `path_sha256` pinned by `lit/ded/Lit.toml`).

### `d-del/ddel-lr-crim-speedy-trial-1987-amended-2011-05-02-sectioned.txt` — D. Del. Time Limits and Procedures for Achieving Prompt Disposition of Criminal Cases, sectioned

- Derived in the pod (lg-gddt.10) from `d-del/ddel-lr-crim-speedy-trial-1987-amended-2011-05-02.txt` (extract sha256 049fe639690eae8dd3b9c46057c9c26f152a935d37fc15fea5cd406103eb3ace); nothing fetched. One marker line `## D. Del. Crim. Plan § <n>` is inserted before each section heading line `<n>. <Title>` of the 1987 plan (§§ 1 to 14). Section 8 is in the file three times: the 1987 text is marked `§ 8-1987`, the order of December 6, 1994 rewriting it is marked `§ 8-1994` (before the order's caption), and the order of May 2, 2011 rewriting it again, the text in force, is marked `§ 8` (before its caption). Every other line is the extract unchanged, so deleting the marker lines (`sed '/^## D. Del. Crim. Plan § /d'`) gives back the extract byte for byte.
- Edition: as the extract, amended May 2, 2011. The PDF is a scan and its text layer has recognition errors in places (the 1987 § 8 and parts of the orders' captions are garbled); `lit/ded-criminal` takes only periods that are legible, and the 2011 § 8 periods are written both in words and in figures.
- Source: as the extract, https://www.ded.uscourts.gov/sites/ded/files/local-rules/LR-Crim.pdf, retrieved 2026-10-28; the sectioning was done from the vendored extract.
- Sectioned sha256 5f79463467a60cc6c6e6e1f25d97b17d5667b2bcd0e223b6dbd571314d8f9319 (the `path_sha256` pinned by `lit/ded-criminal/Lit.toml`).
