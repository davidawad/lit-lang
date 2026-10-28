
### `txwd/tx-wd-local-rules.txt` — W.D. Tex. Local Court Rules, complete copy

- Source: https://www.txwd.uscourts.gov/wp-content/uploads/2026/10/TXWD-Local-Rules-Full-Copy-081926.pdf (official site; 245 pages, sha256 475cd12040bc439f1f9041a6e17fe38845025076bba29546e8b67046ec52b647).
- Edition: Local Court Rules of the W.D. Tex., cover "Effective: April 26, 2023" (civil rules effective June 24, 2021 per its table of contents); complete copy linked from https://www.txwd.uscourts.gov/court-information/lcr-introduction/ (no robots.txt: 404; privacy-statement has no automated-access ban). Fetch method: curl with UA litgraph-rules-vendor/1.0, no bot challenge.
- Also the criminal local rules (lg-gddt): 'SECTION II - CRIMINAL RULES' part, from extract line 1181.
- Criminal forum (lg-gddt): txwd-criminal.
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 51af4ff7a502bf528d3d5a37491cb8424747bbc71addfedc74956d0ca00e65ca. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `txwd/original/tx-wd-local-rules.pdf` as served (sha256 475cd12040bc439f1f9041a6e17fe38845025076bba29546e8b67046ec52b647); extraction sha256 51af4ff7a502bf528d3d5a37491cb8424747bbc71addfedc74956d0ca00e65ca.

### `txwd/tx-wd-local-rules-sectioned.txt` — W.D. Tex. Local Court Rules, criminal rules sectioned

- Derived in the pod (lg-gddt.14) from `txwd/tx-wd-local-rules.txt` (extract sha256 51af4ff7a502bf528d3d5a37491cb8424747bbc71addfedc74956d0ca00e65ca); nothing fetched. In the body of Section II - Criminal Rules (after the table of contents, before Section III - Attorneys), one marker line `## CR-<n>` is inserted before each rule's own heading line `RULE CR-<n>. ...` (CR-1 through CR-58, 19 rules; the committee notes and appended forms stay in the rule they follow). The civil, attorney and other parts carry no markers. Every other line is the extract unchanged, so deleting the `## CR-` lines gives back the extract byte for byte.
- Upstream: https://www.txwd.uscourts.gov/wp-content/uploads/2026/10/TXWD-Local-Rules-Full-Copy-081926.pdf, retrieved 2026-10-27, as recorded on the `txwd/tx-wd-local-rules.txt` entry this file is derived from.
- Edition: as the extract (cover "Effective: April 26, 2023").
- Sectioned sha256 779483d215e8dff84763bdb9bc9dd90205700f18340a960b635d1f885cc63fa6 (the `path_sha256` pinned by `lit/txwd-criminal/Lit.toml`).

### `txwd/tx-wd-local-rules-civil-sectioned.txt` — W.D. Tex. Local Court Rules, civil rules sectioned

- Derived in the pod (lg-f3q8.87) from `txwd/tx-wd-local-rules.txt` (extract sha256 51af4ff7a502bf528d3d5a37491cb8424747bbc71addfedc74956d0ca00e65ca); nothing fetched. In the body of Section I - Civil Rules (after the table of contents, before Section II - Criminal Rules), one marker line `## CV-<n>` is inserted before each rule's own heading line `RULE CV-<n>. ...` (CV-1 through CV-88, 20 rules; the deleted CV-6 and CV-55 have no heading). The criminal, attorney and appendix parts carry no markers. Every other line is the extract unchanged, so deleting the `## CV-` lines gives back the extract byte for byte.
- Upstream: https://www.txwd.uscourts.gov/wp-content/uploads/2026/10/TXWD-Local-Rules-Full-Copy-081926.pdf, retrieved 2026-10-27, as recorded on the `txwd/tx-wd-local-rules.txt` entry this file is derived from.
- Edition: as the extract (cover "Effective: April 26, 2023"; civil rules "Effective June 24, 2021").
- Sectioned sha256 67cca82c99afdd4244e8bd94d7806d0b2ca68041c857e8f0eb7ef7a97e66dcbb (the `path_sha256` pinned by `lit/txwd/Lit.toml`).
