
### `nv-district/nrcp.txt` — Nevada Rules of Civil Procedure

- Source: https://www.leg.state.nv.us/Division/Legal/LawLibrary/CourtRules/NRCP.html (Nevada Legislature, Legal Division law library: an official publisher of the court rules; HTML sha256 99812c5ef90fa83a0b5ded17bb4f1649316b2e1c3cc48c9e91eaca63e2479065).
- Edition: "Effective January 1, 1953 and Including Amendments Through October 31, 2025" (page header; revision stamp "Rev. 4/15/2026"). nvcourts.gov publishes only drafts and per-rule PDFs.
- Retrieved 2026-10-27 on carbon with one plain curl GET, identifying User-Agent (PM ruling 2026-10-27). robots.txt does not disallow the court-rules path; the nv.gov privacy policy linked from the site has no automation ban. Extracted from the page HTML (decoded as Windows-1252, tags stripped, non-breaking spaces normalized, unedited otherwise). The outlines cite NRCP 3, 4, 12, 26, 38, 41, 50, 55, 56, 58 and 59.
- Extract sha256 603927b1c1d9a6a9e24e2a1531569ce8fb44bc7cf1e0684de7f9e8c1b9a3bf11.
- Original: `nv-district/original/nrcp.html` as served (sha256 99812c5ef90fa83a0b5ded17bb4f1649316b2e1c3cc48c9e91eaca63e2479065); extraction sha256 603927b1c1d9a6a9e24e2a1531569ce8fb44bc7cf1e0684de7f9e8c1b9a3bf11.

### `nv-district/nrcp-sectioned.txt` — Nevada Rules of Civil Procedure (amendments through October 31, 2025), sectioned

- Derived in the pod (lg-f3q8.45) from `nv-district/nrcp.txt` (extract sha256 603927b1c1d9a6a9e24e2a1531569ce8fb44bc7cf1e0684de7f9e8c1b9a3bf11); nothing fetched. One `## NRCP <n>` line is inserted right before each rule's own heading line, `Rule<U+2002><n>.<U+2002><U+2002><title>` (the en spaces set the headings apart from in-text references to rules): 102 markers, NRCP 1 through NRCP 86. Rule text was not edited: deleting the marker lines (`sed '/^## NRCP /d'`) restores the extract byte for byte.
- Edition: as the extract; "Effective January 1, 1953 and Including Amendments Through October 31, 2025".
- Sectioned sha256 7748a9182674e4ec40e7136fa35430334397972a5331554a6114ec25e3cc303b (the `path_sha256` pinned by `lit/nv-district/Lit.toml`).
