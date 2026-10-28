
### `fed-dmont/dmont-local-rules.txt` — District of Montana Local Rules of Procedure

- Source: https://www.mtd.uscourts.gov/sites/mtd/files/LocalRules_12012025.pdf (official site; 154 pages, sha256 dcd4f92263c596639ca3676c35707ebd54ac37c6763d0778bdc26df25b83a8d3).
- Edition: Local Rules of Procedure, "Effective December 1, 2025", linked as "U.S. District Court Local Rules (Effective 12/1/2025)" from https://www.mtd.uscourts.gov/court-info/local-rules-and-orders; the 2026 proposed amendments (LocalRules_12012026.pdf, open for public comment on the same page) are not in force and are not vendored.
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 fe0d69067e3e7e2c54465189c76a6b144a51eb0c1b57b86dff62256a469c2948. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `fed-dmont/original/dmont-local-rules.pdf` as served (sha256 dcd4f92263c596639ca3676c35707ebd54ac37c6763d0778bdc26df25b83a8d3); extraction sha256 fe0d69067e3e7e2c54465189c76a6b144a51eb0c1b57b86dff62256a469c2948.

### `fed-dmont/dmont-local-rules-sectioned.txt` — D. Montana Local Rules of Procedure, sectioned

- Derived in the pod (lg-f3q8.44) from `fed-dmont/dmont-local-rules.txt` (extract sha256 fe0d69067e3e7e2c54465189c76a6b144a51eb0c1b57b86dff62256a469c2948); nothing fetched. One `## L.R. <n>` line is inserted before each civil Local Rule's own heading line in the body (a line opening, after an optional form feed, with `<n>.<n>` and a capitalized title, from `1.1 Scope of Local Rules` through `83.9 Limited Scope Representation`, 72 rules); the table of contents, L.R. 7.1-1, the criminal rules (`CR ...`), the appendices and the forms are unmarked. Rule text was not edited: deleting the marker lines (`sed '/^## L.R. /d'`) restores the extract byte for byte.
- Edition: as the extract, Local Rules of Procedure effective December 1, 2025.
- Sectioned sha256 c5300e79defb31f7d1cf003176730640378ca2ae83050cc3455718b888b7dc58 (the `path_sha256` pinned by `lit/mtd/Lit.toml`).
