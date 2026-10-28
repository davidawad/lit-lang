
### `bankr-wae/waeb-lbr-2026-07-17.txt` — Local Rules, U.S. Bankruptcy Court for the Eastern District of Washington (complete set, file dated 2026-07-17)

- Source: https://www.waeb.uscourts.gov/sites/waeb/files/Complete_Set_Local%20Rules_20260717.pdf (official site; 142 pages, sha256 81f5dd678a3287cb8bd5f272ce74ac4aa8eb68c87f0428709bba6416714f0f8f).
- Edition: Complete Set of Local Rules; the cover carries no date and the file name is 'Complete_Set_Local Rules_20260717'; linked as 'Complete Set of Local Rules' from https://www.waeb.uscourts.gov/local-bankruptcy-rules. Edition date taken from the file name only. robots.txt (Crawl-delay 10 honored; no rule covering these paths) and the Privacy Protections page checked: no prohibition of automated access. Fetched with curl and the identifying User-Agent lit-lang-provenance/1.0 (angrybohr@proton.me).
- Retrieved 2026-10-28 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 4522e6f5521b4bbe233290fa99bac134a85f8ce90f78540c56a16cf366cb1651. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `bankr-wae/original/waeb-lbr-2026-07-17.pdf` as served (sha256 81f5dd678a3287cb8bd5f272ce74ac4aa8eb68c87f0428709bba6416714f0f8f); extraction sha256 4522e6f5521b4bbe233290fa99bac134a85f8ce90f78540c56a16cf366cb1651.

### `bankr-wae/waeb-lbr-2026-07-17-sectioned.txt` — Bankr. E.D. Wash. Local Rules, sectioned

- Derived in the pod (lg-u561.25) from `bankr-wae/waeb-lbr-2026-07-17.txt` (extract sha256 4522e6f5521b4bbe233290fa99bac134a85f8ce90f78540c56a16cf366cb1651); nothing fetched. One `## LBR <n>` line is inserted before each rule's own heading line in the body (an indented line, after an optional form feed, holding only `Rule <nnnn>-<n>` (a rule numbered `<nnnn>.<n>-<n>` is marked by its first number, `## LBR 3002.1`, as cite pinpoints read it; a rule number listed twice, abrogated and current, is marked once, at its first heading); the set is every such heading after the table of contents, from `1006-1` through `9073-1`, 83 rules); rule text was not edited: deleting the marker lines (`sed '/^## LBR /d'`) restores the extract byte for byte.
- Source: as the extract `bankr-wae/waeb-lbr-2026-07-17.txt`, https://www.waeb.uscourts.gov/sites/waeb/files/Complete_Set_Local%20Rules_20260717.pdf, retrieved 2026-10-28; the sectioning was done in the pod from the vendored extract.
- Edition: as the extract.
- Sectioned sha256 5244f77e7de78615f5e9713f238de5d463e362c46dd0f8e24f38b908a2e3efa5 (the `path_sha256` pinned by `lit/bankr-wae/Lit.toml`).
