
### `or-circuit/orcp.txt` — Oregon Rules of Civil Procedure (ORCP)

- Source: https://www.oregonlegislature.gov/bills_laws/SiteAssets/ORCP.html (official site; fetched file sha256 e55957b95b25f21142acfca62e8a6c9521b79b7362289f0b308fedf6aa0b602c).
- Edition: Oregon Rules of Civil Procedure, titled "Oregon Rules of Civil Procedure (2025)" in the file; the expanded HTML linked as "View expanded ORCP" from https://www.oregonlegislature.gov/bills_laws/Pages/orcp.aspx, which the Council on Court Procedures current-rules page (https://counciloncourtprocedures.org/current-rules/, its own static link is the stale 2020 text) names as the most recent rules; no later promulgation is listed.
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); plain curl with User-Agent litgraph-rules-vendor/1.0 (host www.oregonlegislature.gov; the bare host oregonlegislature.gov answered 403); text extracted from the HTML by a script (html2txt.py), tags stripped, unedited otherwise.
- Extract sha256 8ad32b2f17d7546ef2170df5012882d7cc023777768ea690930d9f4d93801d94. Raw extract, not yet sectioned into `## <ref>` form beyond any per-rule headings.
- Original: `or-circuit/original/orcp.html` as served (sha256 e55957b95b25f21142acfca62e8a6c9521b79b7362289f0b308fedf6aa0b602c); extraction sha256 8ad32b2f17d7546ef2170df5012882d7cc023777768ea690930d9f4d93801d94.

### `or-circuit/orcp-sectioned.txt` — Oregon Rules of Civil Procedure (2025), sectioned

- Derived in the pod (lg-f3q8.47) from `or-circuit/orcp.txt` (extract sha256 8ad32b2f17d7546ef2170df5012882d7cc023777768ea690930d9f4d93801d94); nothing fetched. Past the table of contents, one bare `## ORCP <n>` line is inserted before each `RULE <n>` (or `RULES <n>`) heading line (81 markers), and within each rule one bare `## ORCP <n> <L>` line before the line opening each lettered section `<L>` (a line that is the letter alone, the letter and a capitalized word, or `<L>(1)`, after a blank line, letters ascending; 353 markers), so Oregon-style pinpoints such as `ORCP 7 C(2)` resolve. Rule text was not edited: deleting the marker lines (`sed -E '/^## ORCP [0-9]+( [A-Z])?$/d'`) restores the extract byte for byte.
- Source: as the extract `or-circuit/orcp.txt`, https://www.oregonlegislature.gov/bills_laws/SiteAssets/ORCP.html; Retrieved 2026-10-27 with the extract (this file was derived from it in the pod, not fetched itself).
- Edition: as the extract, "Oregon Rules of Civil Procedure (2025)".
- Sectioned sha256 047ba8dd8ccdf7b9a82deaef68bb90df6d51d927b9f43f7b709d824acc1f881e (the `path_sha256` pinned by `lit/or-circuit/Lit.toml`).
