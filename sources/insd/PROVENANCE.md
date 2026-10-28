
### `insd/insd-local-rules.txt` — S.D. Indiana Local Rules

- Source: https://www.insd.uscourts.gov/sites/insd/files/Local%20Rules%207-1-26.pdf (official site; 121 pages, sha256 1e832484f765290b64b70b7ceb32c90980b3b071b1ca4690f680778bec1b0149).
- Edition: Local Rules with amendments through July 1, 2026 (https://www.insd.uscourts.gov/local-rules). Not included: public notice dated October 16, 2026 adopting amendments (e.g. LR 83-6) effective December 1, 2026, not yet in force (NoticeAmend%2012-1-2026.pdf); fetched by curl.
- Also the criminal local rules (lg-gddt): 'CRIMINAL RULES' part, from extract line 2682.
- Criminal forum (lg-gddt): insd-criminal.
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 c95c5122118a534dd8b512bd081414ba01aeb77629cf9d2279788afde6ed18d8. Raw extract, not yet sectioned into `## <ref>` form.

- Fetch method: first fetched with curl and a desktop browser User-Agent before the no-browser-identity rule (PM 2026-10-27); re-fetched on 2026-10-27 with plain curl (its default User-Agent, no browser headers): byte-identical (same sha256).
- Original: `insd/original/insd-local-rules.pdf` as served (sha256 1e832484f765290b64b70b7ceb32c90980b3b071b1ca4690f680778bec1b0149); extraction sha256 c95c5122118a534dd8b512bd081414ba01aeb77629cf9d2279788afde6ed18d8.

### `insd/insd-local-rules-sectioned.txt` — S.D. Indiana Local Rules, sectioned

- Derived in the pod (lg-f3q8.55) from `insd/insd-local-rules.txt` (extract sha256 c95c5122118a534dd8b512bd081414ba01aeb77629cf9d2279788afde6ed18d8); nothing fetched. One `## S.D. Ind. L.R. <n>` line is inserted before each local civil rule's own `Local Rule <n>` heading line in the body (66 rules, L.R. 1-1 through L.R. 87); the table of contents, the criminal rules and the appendices are unmarked. Rule text was not edited: deleting the marker lines (`sed -E '/^## S\.D\. Ind\. L\.R\. /d'`) restores the extract byte for byte.
- Source: as the extract `insd/insd-local-rules.txt`, https://www.insd.uscourts.gov/sites/insd/files/Local%20Rules%207-1-26.pdf, retrieved 2026-10-27; the sectioning was done in the pod from the vendored extract.
- Edition: as the extract, Local Rules with amendments through July 1, 2026.
- Sectioned sha256 5b55f4232b25b5dc46297a538630dff9293de8512ca2950fba002b3460763287 (the `path_sha256` pinned by `lit/insd/Lit.toml`).

### `insd/insd-criminal-rules-sectioned.txt` — S.D. Indiana Local Rules, criminal rules sectioned

- Derived in the pod (lg-gddt.18) from `insd/insd-local-rules.txt` (extract sha256 c95c5122118a534dd8b512bd081414ba01aeb77629cf9d2279788afde6ed18d8); nothing fetched. In the body of the criminal rules (after the `CRIMINAL RULES` heading at extract line 2592, before `S.D. Indiana – Appendix A`), one `## S.D. Ind. L. Cr. R. <n>` line is inserted before each rule's own heading line `Local Criminal Rule <n> - ...` (15 rules, L. Cr. R. 6-1 through 58-2; an en dash in the number is marked with a hyphen). The civil rules, the table of contents and the appendices carry no such markers. Rule text was not edited: deleting the marker lines (`sed -E '/^## S\.D\. Ind\. L\. Cr\. R\. /d'`) restores the extract byte for byte.
- Edition: as the extract, Local Rules with amendments through July 1, 2026.
- Upstream: https://www.insd.uscourts.gov/sites/insd/files/Local%20Rules%207-1-26.pdf, retrieved 2026-10-27, as recorded on `insd/insd-local-rules.txt` above; sectioning was done from the vendored extract.
- Sectioned sha256 c2fe06cd87967e0921bea134a6671d53fa45ab18e3d1401dae893b377d2dc665 (the `path_sha256` pinned by `lit/insd-criminal/Lit.toml`).
