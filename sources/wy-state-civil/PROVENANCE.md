
### `wy-state-civil/wrcp.txt` — W.R.C.P.

- Source: https://www.wyocourts.gov/app/uploads/2025/01/WY-Rules-of-Civ.-Procedure-Eff.-9-21-26.pdf (linked from https://www.wyocourts.gov/court-rules/wyoming-rules-of-civil-procedure/) (official site; 228 pages, sha256 a22efdb9680278b9c4fb388eefaf65fb7136cf2e8a95347c9811d32d4bd42adb).
- Edition: consolidated W.R.C.P. including the amendments effective Sept 21, 2026 (Rules 4 and 41), as the court's rule page lists them.
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 fef457faece9d4a479a3d211b3ce4cca258e587bb74b2d796b4fc51301d94d54. Raw extract, not yet sectioned into `## <ref>` form.
- Fetch method: BrowserOS neo; site served a bot challenge to plain automated requests (CoS ruling 2026-10-27). wyocourts.gov robots.txt disallows only /app/uploads/protected/ and search URLs; its privacy page bars no automated access.
- Original: `wy-state-civil/original/wrcp.pdf` as served (sha256 a22efdb9680278b9c4fb388eefaf65fb7136cf2e8a95347c9811d32d4bd42adb); extraction sha256 fef457faece9d4a479a3d211b3ce4cca258e587bb74b2d796b4fc51301d94d54.

### `wy-state-civil/wrcp-sectioned.txt` — W.R.C.P., sectioned

- Derived in the pod (lg-f3q8.71) from `wy-state-civil/wrcp.txt` (extract sha256 fef457faece9d4a479a3d211b3ce4cca258e587bb74b2d796b4fc51301d94d54); nothing fetched. One `## W.R.C.P. <n>` line is inserted before each rule's own `Rule <n>.` heading line at the start of a line (`W.R.C.P. 1` through `W.R.C.P. 86`, 100 headings). The two versions of Rules 4 and 41 marked `[Effective until September 21, 2026]` are headed `## W.R.C.P. 4 (version in effect until September 21, 2026)` and `## W.R.C.P. 41 (version in effect until September 21, 2026)`, so a cite of `W.R.C.P. 4` or `W.R.C.P. 41` binds to the version effective September 21, 2026. Rule text and the interleaved annotations were not edited: deleting the marker lines (`sed '/^## W\.R\.C\.P\. /d'`) restores the extract byte for byte.
- Source: as the extract, https://www.wyocourts.gov/app/uploads/2025/01/WY-Rules-of-Civ.-Procedure-Eff.-9-21-26.pdf, retrieved 2026-10-27.
- Edition: as the extract, consolidated W.R.C.P. including the amendments effective September 21, 2026.
- Sectioned sha256 17d84d74bc55a79fe70c5bd3bcde611b93913cdbe457c46b7de3f9a973580f17 (the `path_sha256` pinned by `lit/wy-district/Lit.toml`).
