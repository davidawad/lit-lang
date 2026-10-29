
### `d-nm/d-nm-local-civil-rules-2020.txt` — D.N.M. Local Rules of Civil Procedure (D.N.M.LR-Civ.)

- Source: https://www.nmd.uscourts.gov/sites/nmd/files/local_rules/Local%20Rules%20of%20Civil%20Procedure%20Adopted%20October%201%2C%202020_0.pdf (official site; 45 pages, sha256 f8526a99a16fdf0019d2648b388cc1554df02e8faab433aec9b0304868a1bf6f).
- Edition: Local Rules of Civil Procedure effective July 1, 2011, amended effective October 1, 2020 (title page); https://www.nmd.uscourts.gov/court-info/local-rules-and-orders/local-rules lists "Effective: 07/01/2011 Amended: 10/01/2020" as the current civil rules; fetched with plain curl (no browser identity); robots.txt allows the path (Crawl-delay 10 honored).
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 80234cfdc5fba4e085b4a565af926793b1806893eefe436515c352bd006dff85. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `d-nm/original/d-nm-local-civil-rules-2020.pdf` as served (sha256 f8526a99a16fdf0019d2648b388cc1554df02e8faab433aec9b0304868a1bf6f); extraction sha256 80234cfdc5fba4e085b4a565af926793b1806893eefe436515c352bd006dff85.

### `d-nm/dnm-local-criminal-rules-2025-11-14.txt` — D.N.M. Local Rules of Criminal Procedure (adopted November 14, 2025)

- Source: https://www.nmd.uscourts.gov/sites/nmd/files/local_rules/Local%20Rules%20of%20Criminal%20Procedure%20Adopted%20November%2014%2C%202025.pdf (official site; 25 pages, sha256 9b4a610265e607a6627912064ad81560ab4a4b6f6cf8f08945ce9c883538f499).
- Edition: Local Rules of Criminal Procedure adopted by administrative order 25-MC-00004-35 filed November 14, 2025 (first page), linked at https://www.nmd.uscourts.gov/court-info/local-rules-and-orders/local-rules. robots.txt of www.nmd.uscourts.gov read (Crawl-delay 10 honored where stated; no Disallow covers the fetched path); the judiciary-wide Privacy & Security Policy (https://www.uscourts.gov/privacy-security-policy) read: no prohibition of automated access. Fetched with curl and the identifying UA lit-lang-provenance/1.0, at most 1 request per 10 s.
- Criminal forum (lg-gddt): nmd-criminal.
- Retrieved 2026-10-28 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 ef23cfd806f842e7106ca2bebb85ca04f5d342a3934cbdb180517ac6ef84c003. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `d-nm/original/dnm-local-criminal-rules-2025-11-14.pdf` as served (sha256 9b4a610265e607a6627912064ad81560ab4a4b6f6cf8f08945ce9c883538f499); extraction sha256 ef23cfd806f842e7106ca2bebb85ca04f5d342a3934cbdb180517ac6ef84c003.

### `d-nm/d-nm-local-civil-rules-2020-sectioned.txt` — D.N.M. Local Rules of Civil Procedure, sectioned

- Derived in the pod (lg-f3q8.68) from `d-nm/d-nm-local-civil-rules-2020.txt` (extract sha256 80234cfdc5fba4e085b4a565af926793b1806893eefe436515c352bd006dff85); nothing fetched. One `## D.N.M.LR-Civ. <n>` line is inserted before each rule's own heading line in the body (`<n> <Title>`, after the table of contents), `D.N.M.LR-Civ. 1.1` through `D.N.M.LR-Civ. 83.13`, 83 rules. Rule text was not edited: deleting the marker lines (`perl -ne 'print unless /^## D\.N\.M\.LR-Civ\. /'`) restores the extract byte for byte.
- Source: as the extract `d-nm/d-nm-local-civil-rules-2020.txt`, https://www.nmd.uscourts.gov/sites/nmd/files/local_rules/Local%20Rules%20of%20Civil%20Procedure%20Adopted%20October%201%2C%202020_0.pdf, retrieved 2026-10-27; the sectioning was done in the pod from the vendored extract.
- Edition: as the extract (effective July 1, 2011, amended effective October 1, 2020).
- Sectioned sha256 508c16043b626f4d2e39dd35346bb7af9bdc6ced9c5b4e428fdba667fc70ee6d (the `path_sha256` pinned by `lit/nmd/Lit.toml`).

### `d-nm/dnm-local-criminal-rules-2025-11-14-sectioned.txt` — D.N.M. Local Rules of Criminal Procedure, sectioned

- Derived in the pod (lg-gddt.24) from `d-nm/dnm-local-criminal-rules-2025-11-14.txt` (extract sha256 ef23cfd806f842e7106ca2bebb85ca04f5d342a3934cbdb180517ac6ef84c003); nothing fetched. In the body (after the table of contents), one marker line `## D.N.M.LR-Cr. <n>` is inserted before each numbered rule's own first line (`<n> <Title>. ...`), D.N.M.LR-Cr. 1.1 through 58.2, 49 rules including the lettered 5.A-5.C and 32.A-32.E; the `RULE <n>` group headings carry no marker. The attached Standard Discovery Order (Local Form 1) and the other attachments fall in 58.2's section, which runs to the end of the file. Rule text was not edited: deleting the marker lines (each `## D.N.M.LR-Cr. ` line with its newline) restores the extract byte for byte.
- Source: as the extract `d-nm/dnm-local-criminal-rules-2025-11-14.txt`, https://www.nmd.uscourts.gov/sites/nmd/files/local_rules/Local%20Rules%20of%20Criminal%20Procedure%20Adopted%20November%2014%2C%202025.pdf; Retrieved 2026-10-28 with the extract (this file was derived from it in the pod, not fetched itself).
- Edition: as the extract (effective December 1, 2009, amended effective November 14, 2025).
- Sectioned sha256 b9c83e7a87ada9b8145dead5baf9945da51636d47c656d1bd79db70c7d7f3df0 (the `path_sha256` pinned by `lit/nmd-criminal/Lit.toml`).
