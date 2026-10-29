
### `fed-ndcal/ndcal-civil-local-rules.txt` — N.D. Cal. Civil Local Rules

- Source: https://cand.uscourts.gov/sites/default/files/local-rules/CAND_Civil_Local_Rules_10_26.pdf (official site; 82 pages, sha256 1407f4d60e542d9b0233495f4df55510ec682f6432ee672b0569350555b182ec).
- Edition: N.D. Cal. Civil Local Rules, effective October 2, 2026 (document footer), linked from https://cand.uscourts.gov/rules-forms-fees/local-rules. Fetched with curl (robots.txt allows).
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 fa3fc80fb18bd36a30ca823d79db84e14da555e5e391f328c9ebc85fd17259f0. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `fed-ndcal/original/ndcal-civil-local-rules.pdf` as served (sha256 1407f4d60e542d9b0233495f4df55510ec682f6432ee672b0569350555b182ec); extraction sha256 fa3fc80fb18bd36a30ca823d79db84e14da555e5e391f328c9ebc85fd17259f0.

### `fed-ndcal/ndcal-civil-local-rules-sectioned.txt` — N.D. Cal. Civil Local Rules, sectioned

- Derived in the pod (lg-f3q8.39) from `fed-ndcal/ndcal-civil-local-rules.txt` (extract sha256 fa3fc80fb18bd36a30ca823d79db84e14da555e5e391f328c9ebc85fd17259f0); nothing fetched. One `## N.D. Cal. Civil L.R. <n>` line is inserted right after each rule's own body heading line (`<n>. <title>`, after the table of contents; 116 markers, 1-1 through 83-2, including 65.1-1). The table of contents is unmarked. Rule text was not edited: deleting the marker lines (each `## N.D. Cal. Civil L.R. ` line with its newline) restores the extract byte for byte.
- Source: as the extract `fed-ndcal/ndcal-civil-local-rules.txt`, https://cand.uscourts.gov/sites/default/files/local-rules/CAND_Civil_Local_Rules_10_26.pdf, retrieved 2026-10-27; the sectioning was done in the pod from the vendored extract.
- Edition: as the extract, Civil Local Rules effective October 2, 2026.
- Sectioned sha256 93aab4d1dd994bc5f7fbec907e8c9d5cf4c0712c1ca67cf2640c058e146009ad (the `path_sha256` pinned by `lit/cand/Lit.toml`).

### `fed-ndcal/ndcal-criminal-local-rules-rev-2026-07-29.txt` — N.D. Cal. Criminal Local Rules (rev. 7-29-2026)

- Source: https://www.cand.uscourts.gov/sites/default/files/local-rules/Local_Rules_Criminal_rev_7-29-2026.pdf (official site; 29 pages, sha256 32aa44b72876fa1c23493e5c6e22c81957411a6e3fabc809ab8dfdcacf78053d).
- Edition: Criminal Local Rules, the PDF file named 'Local_Rules_Criminal_rev_7-29-2026', listed under Local Rules at https://www.cand.uscourts.gov/rules/local-rules. robots.txt answered with a redirect, no rules read; the judiciary-wide Privacy & Security Policy (https://www.uscourts.gov/privacy-security-policy) read: no prohibition of automated access. Fetched with curl and the identifying UA lit-lang-provenance/1.0, at most 1 request per 10 s.
- Criminal forum (lg-gddt): cand-criminal.
- Retrieved 2026-10-28 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 3aca8c4503552bbc014ccb2cacc8688c853498f8a4e44b4e1de0622ddf771880. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `fed-ndcal/original/ndcal-criminal-local-rules-rev-2026-07-29.pdf` as served (sha256 32aa44b72876fa1c23493e5c6e22c81957411a6e3fabc809ab8dfdcacf78053d); extraction sha256 3aca8c4503552bbc014ccb2cacc8688c853498f8a4e44b4e1de0622ddf771880.

### `fed-ndcal/ndcal-criminal-local-rules-rev-2026-07-29-sectioned.txt` — N.D. Cal. Criminal Local Rules (rev. 7-29-2026), sectioned

- Derived in the pod (lg-gddt.22) from `fed-ndcal/ndcal-criminal-local-rules-rev-2026-07-29.txt` (extract sha256 3aca8c4503552bbc014ccb2cacc8688c853498f8a4e44b4e1de0622ddf771880); nothing fetched. One marker line `## Crim. L.R. <n>` is inserted before each rule's own body heading line `<n>. <Title>` (indented four spaces, some led by a form feed; Crim. L.R. 1-1 through 59-1, 48 rules, matching the table of contents). Every other line is the extract unchanged, so deleting the marker lines gives back the extract byte for byte.
- Source: as the extract `fed-ndcal/ndcal-criminal-local-rules-rev-2026-07-29.txt`, https://www.cand.uscourts.gov/sites/default/files/local-rules/Local_Rules_Criminal_rev_7-29-2026.pdf; Retrieved 2026-10-28 with the extract (this file was derived from it in the pod, not fetched itself).
- Edition: as the extract, the rule pages footed "Revised July 29, 2026" (the table-of-contents pages still carry the August 21, 2024 footer).
- Sectioned sha256 f65f4e7dd19f59d6f97573448706525593cc2f33157d24024eed7383a7835338 (the `path_sha256` pinned by `lit/cand-criminal/Lit.toml`).
