
### `mt-supreme/mrap.txt` — Montana Rules of Appellate Procedure (M. R. App. P.)

- Source: https://courts.mt.gov/external/rules/mtr_app_pro.pdf (official site; 58 pages, sha256 e1e2f5bc2dcd9b5f6a733719b8a43f3f8be70b330372e34eb9498f22ecbbaea8).
- Edition: Montana Rules of Appellate Procedure; the document states "Date of Last Amendment to these Rules: 12OCT2021", AF 07-0016, State Law Library of Montana cover sheet; linked as "Montana Rules of Appellate Procedure" from https://courts.mt.gov/courts/rules/ (page lists no later amendments).
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 8b554bb69b6fdd2e675a99ef0e5fb0a8e5815cfc3482342310113bd887ba662d. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `mt-supreme/original/mrap.pdf` as served (sha256 e1e2f5bc2dcd9b5f6a733719b8a43f3f8be70b330372e34eb9498f22ecbbaea8); extraction sha256 8b554bb69b6fdd2e675a99ef0e5fb0a8e5815cfc3482342310113bd887ba662d.

### `mt-supreme/mrap-sectioned.txt` — Montana Rules of Appellate Procedure, sectioned

- Derived in the pod (lg-f3q8.44) from `mt-supreme/mrap.txt` (extract sha256 8b554bb69b6fdd2e675a99ef0e5fb0a8e5815cfc3482342310113bd887ba662d); nothing fetched. The extract is single-column. One `## Rule <n>` line is inserted before each rule's own heading line after the cover sheet (a line opening, after an optional form feed, with `Rule <n>.` and whitespace, Rule 1 through Rule 30, 30 rules); the appendix of forms is unmarked. Rule text was not edited: deleting the marker lines (`sed '/^## Rule /d'`) restores the extract byte for byte.
- Edition: as the extract; date of last amendment October 12, 2021.
- Sectioned sha256 8565e044af165525fbe7aa6f34d39bb48e12c22c22b681d36d356e8a1d31e217 (the `path_sha256` pinned by `lit/mt-district/Lit.toml` and `lit/mt-supreme/Lit.toml`).
