
### `or-court-of-appeals/orap.txt` — Oregon Rules of Appellate Procedure (ORAP)

- Source: https://www.courts.oregon.gov/rules/ORAP/ORAP_2026-FullPermanentAndTempAmendments-Eff_2026-10-16.pdf (official site; 277 pages, sha256 ba5ceab5b0e6278daf19f31dfbaac3700c72b3cdc97c1593cac01b7d4a63fcae).
- Edition: Oregon Rules of Appellate Procedure, "Permanent Amendments Effective January 1, 2025", also including temporary amendments (CJO 26-02 and others) effective through 2026-10-16; the file is named ORAP_2026-FullPermanentAndTempAmendments-Eff_2026-10-16 and is the full-rules link on https://www.courts.oregon.gov/courts/appellate/rules/Pages/orap.aspx (the link table is rendered by JavaScript, so the link was read in BrowserOS; the PDF itself was fetched with plain curl; no BrowserOS download).
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 505ecf1f472888c6b6417676d00659867ed73401c4f85de89e8d66bde4ac23a8. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `or-court-of-appeals/original/orap.pdf` as served (sha256 ba5ceab5b0e6278daf19f31dfbaac3700c72b3cdc97c1593cac01b7d4a63fcae); extraction sha256 505ecf1f472888c6b6417676d00659867ed73401c4f85de89e8d66bde4ac23a8.

### `or-court-of-appeals/orap-sectioned.txt` — Oregon Rules of Appellate Procedure (version date September 3, 2026, temporary amendments effective through 2026-10-16), sectioned

- Derived in the pod (lg-f3q8.47) from `or-court-of-appeals/orap.txt` (extract sha256 505ecf1f472888c6b6417676d00659867ed73401c4f85de89e8d66bde4ac23a8); nothing fetched. One bare `## ORAP <n>` line is inserted before each line that holds only a centered `Rule <n>` heading (166 markers, ORAP 1.05 through 16.60). Rule text was not edited: deleting the marker lines (`sed -E '/^## ORAP [0-9]+\.[0-9]+$/d'`) restores the extract byte for byte.
- Source: as the extract `or-court-of-appeals/orap.txt`, https://www.courts.oregon.gov/rules/ORAP/ORAP_2026-FullPermanentAndTempAmendments-Eff_2026-10-16.pdf; Retrieved 2026-10-27 with the extract (this file was derived from it in the pod, not fetched itself).
- Edition: as the extract; the cover states permanent amendments effective January 1, 2025, the temporary amendments of CJO 26-02, CJO 26-033 / CJO 26-01, CJO 25-01 and CJO 25-044 / CJO 25-07 incorporated by the court, and version date September 3, 2026.
- Sectioned sha256 7699edb4c6eabf5749a33fc31ecd9df2b5d5869ecaee3149a29a7f9968ae849b (the `path_sha256` pinned by `lit/or-circuit/Lit.toml` and `lit/or-court-of-appeals/Lit.toml`).
