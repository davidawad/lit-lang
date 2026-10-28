
### `tx-district-court/tx-trcp.txt` — Texas Rules of Civil Procedure

- Source: https://www.txcourts.gov/media/1463426/texas-rules-of-civil-procedure.pdf (official site; 369 pages, sha256 bd9d874a40e3fdd9a25a675daad31de2fa3db30acea44d80d3e3362c9d4e178a).
- Edition: Texas Rules of Civil Procedure, PDF titled "10.01.2026"; https://www.txcourts.gov/rules-forms/rules-standards/ lists "Last Amended October 1, 2026" (current). robots.txt and https://www.txcourts.gov/site-policies/ read first: no disallow of /media/ rules PDFs, no automated-access ban. Fetch method: curl with UA litgraph-rules-vendor/1.0, no bot challenge.
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 9c9c54d579a1f3f4807a978f0a5dbad9b58706cb1a2ce7cb9832d798eb136f08. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `tx-district-court/original/tx-trcp.pdf` as served (sha256 bd9d874a40e3fdd9a25a675daad31de2fa3db30acea44d80d3e3362c9d4e178a); extraction sha256 9c9c54d579a1f3f4807a978f0a5dbad9b58706cb1a2ce7cb9832d798eb136f08.

### `tx-district-court/tx-trcp-sectioned.txt` — Texas Rules of Civil Procedure, sectioned

- Derived in the pod (lg-f3q8.87) from `tx-district-court/tx-trcp.txt` (extract sha256 9c9c54d579a1f3f4807a978f0a5dbad9b58706cb1a2ce7cb9832d798eb136f08); nothing fetched. In the body (after the table of contents), one marker line `## Tex. R. Civ. P. <n>` is inserted before each rule's own heading line `RULE <n>. ...` (644 headings, Rule 1 through Rule 822, the part headings of Rules 500 to 510 and their sub-rules `RULE 500.1.` and the like included) and before each sub-rule heading at the start of a line that numbers a sub-rule of the current rule, `<n>.<m> ...` (142 sub-rules, e.g. 91a.1, 166 to 215's discovery sub-rules, 736.1). Every other line is the extract unchanged, so deleting the `## Tex. R. Civ. P.` lines gives back the extract byte for byte.
- Upstream: https://www.txcourts.gov/media/1463426/texas-rules-of-civil-procedure.pdf, retrieved 2026-10-27, as recorded on the `tx-district-court/tx-trcp.txt` entry this file is derived from.
- Edition: as the extract, "Last Amended October 1, 2026".
- Sectioned sha256 1b10047990b61641bf4b89dd2b14a8724617f8d104236b83245e7449934d2d1a (the `path_sha256` pinned by `lit/tx-district-court/Lit.toml`).
