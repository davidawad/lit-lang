
### `wy-state-appellate/wrap.txt` — W.R.A.P.

- Source: https://www.wyocourts.gov/app/uploads/2025/01/WRAP-eff-Dec-1.pdf (linked from https://www.wyocourts.gov/court-rules/wyoming-rules-of-appellate-procedure/) (official site; 125 pages, sha256 ad75efc446f5956b7b94edda881b67fa6a5b0b1a55195fcc5c8e264c6e156874).
- Edition: consolidated W.R.A.P. including the amendments effective Jan 19, 2026 (Rules 3.02, 3.09), as the court's rule page lists them.
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 0a1490005b7f9e6066bf3d27a648c2a493a3c8b82639700bf7e797a34e7388a8. Raw extract, not yet sectioned into `## <ref>` form.
- Fetch method: BrowserOS neo; site served a bot challenge to plain automated requests (CoS ruling 2026-10-27). wyocourts.gov robots.txt disallows only /app/uploads/protected/ and search URLs; its privacy page bars no automated access.
- Original: `wy-state-appellate/original/wrap.pdf` as served (sha256 ad75efc446f5956b7b94edda881b67fa6a5b0b1a55195fcc5c8e264c6e156874); extraction sha256 0a1490005b7f9e6066bf3d27a648c2a493a3c8b82639700bf7e797a34e7388a8.

### `wy-state-appellate/wrap-sectioned.txt` — W.R.A.P., sectioned

- Derived in the pod (lg-f3q8.71) from `wy-state-appellate/wrap.txt` (extract sha256 0a1490005b7f9e6066bf3d27a648c2a493a3c8b82639700bf7e797a34e7388a8); nothing fetched. One `## W.R.A.P. <n>` line is inserted before each rule's own `Rule <n>.` heading line at the start of a line (`W.R.A.P. 1.01` through `W.R.A.P. 29`, 114 headings); extract line 1028, a cross-reference to Rule 1.03 inside Rule 2.01(a) that wraps to the start of a line, is not a heading and is unmarked. The two versions of Rules 3.02 and 3.09 marked `[Effective until January 19, 2026]` are headed `## W.R.A.P. 3.02 (version in effect until January 19, 2026)` and `## W.R.A.P. 3.09 (version in effect until January 19, 2026)`. Rule text and the interleaved annotations were not edited: deleting the marker lines (`sed '/^## W\.R\.A\.P\. /d'`) restores the extract byte for byte.
- Source: as the extract, https://www.wyocourts.gov/app/uploads/2025/01/WRAP-eff-Dec-1.pdf, retrieved 2026-10-27.
- Edition: as the extract, consolidated W.R.A.P. including the amendments effective January 19, 2026.
- Sectioned sha256 515a3474a1e5c02778cdc2a99fb7c78fee285cf2fd627532f1d25ce131f62a9a (the `path_sha256` pinned by `lit/wy-district/Lit.toml` and `lit/wy-supreme/Lit.toml`).
