
### `wisctapp/wis-stat-ch809.txt` — Wis. Stat. ch. 809 Rules of Appellate Procedure

- Source: https://docs.legis.wisconsin.gov/statutes/statutes/809.pdf (official site; 46 pages, sha256 364db252eac8497b599863aaf132298c5ab7e88d00977b1af9ed479ecfccee3a).
- Edition: Updated 2023-24 Wis. Stats., published and certified under s. 35.18, October 1, 2026 (the chapter page states "2023-24 Wisconsin Statutes updated through 2025 Wis. Act 247 and through all Supreme Court Orders ... filed before and in effect on October 1, 2026"); fetched by curl.
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 0a7e60956659899ae03f119c463ba49ff5361af5565ca60e38a5896ee0b9f0e0. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `wisctapp/original/wis-stat-ch809.pdf` as served (sha256 364db252eac8497b599863aaf132298c5ab7e88d00977b1af9ed479ecfccee3a); extraction sha256 0a7e60956659899ae03f119c463ba49ff5361af5565ca60e38a5896ee0b9f0e0.

### `wisctapp/wis-stat-ch808.txt` — Wis. Stat. ch. 808, Appeals and Writs of Error

- Source: https://docs.legis.wisconsin.gov/statutes/statutes/808.pdf (official site; 6 pages, sha256 d9b5620a2fadac29b59b728390350a2dee4a2ca6af09325fec54096df23b65c7).
- Edition: Updated 2023-24 Wis. Stats., published and certified under s. 35.18, October 1, 2026 (the chapter page states "2023-24 Wisconsin Statutes updated through 2025 Wis. Act 247 and through all Supreme Court Orders ... filed before and in effect on October 1, 2026"); fetched by curl.
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 13476b13519c64dd3851e0d0de00720cdccac71055ecfc2caae9ac7e7d328fee. Raw extract, not yet sectioned into `## <ref>` form.

- Fetch method: first fetched with curl and a desktop browser User-Agent before the no-browser-identity rule (PM 2026-10-27); re-fetched on 2026-10-27 with plain curl (its default User-Agent, no browser headers): byte-identical (same sha256).
- Original: `wisctapp/original/wis-stat-ch808.pdf` as served (sha256 d9b5620a2fadac29b59b728390350a2dee4a2ca6af09325fec54096df23b65c7); extraction sha256 13476b13519c64dd3851e0d0de00720cdccac71055ecfc2caae9ac7e7d328fee.

### `wisctapp/wis-stat-ch808-sectioned.txt` — Wis. Stat. ch. 808, Appeals and Writs of Error, single-column and sectioned

- Derived in the pod (lg-f3q8.56) from `wisctapp/wis-stat-ch808.txt` (extract sha256 13476b13519c64dd3851e0d0de00720cdccac71055ecfc2caae9ac7e7d328fee); nothing fetched. The extract is two-column `pdftotext -layout` text, so each page (form feed) is first split into its left column followed by its right column, as for `ak-supreme/alaska-appellate-procedure-sectioned.txt`: the gutter is the page's most common start index (40-100) of text that follows a run of two or more spaces; a line is cut there when the preceding character is a space, otherwise at the nearest two-space run within 12 characters, a line with neither stays whole in the left column, and a page with fewer than five such starts is kept as is. Column text is otherwise unchanged (trailing spaces of left-column lines and leading spaces of right-column lines are dropped; blank right-column lines are collapsed): the multiset of whitespace-separated words equals the extract's. Then one `## § <n>` line is inserted before each of the chapter's 11 section heading lines: a line opening with `808.<digits>`, whitespace and a capitalized title word (not an all-capitals running head), taking each section's last such line so that the first page's table of contents stays unmarked. Deleting the marker lines (`sed '/^## § /d'`) restores the single-column text; the column split is not byte-reversible to the extract.
- Source: as the extract `wisctapp/wis-stat-ch808.txt`, https://docs.legis.wisconsin.gov/statutes/statutes/808.pdf, retrieved 2026-10-27; the sectioning was done in the pod from the vendored extract.
- Edition: as the extract, Updated 2023-24 Wis. Stats., published and certified under s. 35.18, October 1, 2026.
- Sectioned sha256 67b34464aff35374d59d9ed0ffbfd997ab6c084b2494655ef052359eb7b7b4dd (the `path_sha256` pinned by `lit/wi-circuit/Lit.toml` and `lit/wi-court-of-appeals/Lit.toml`).

### `wisctapp/wis-stat-ch809-sectioned.txt` — Wis. Stat. ch. 809, Rules of Appellate Procedure, single-column and sectioned

- Derived in the pod (lg-f3q8.56) from `wisctapp/wis-stat-ch809.txt` (extract sha256 0a7e60956659899ae03f119c463ba49ff5361af5565ca60e38a5896ee0b9f0e0); nothing fetched. The extract is two-column `pdftotext -layout` text, so each page (form feed) is first split into its left column followed by its right column, as for `ak-supreme/alaska-appellate-procedure-sectioned.txt`: the gutter is the page's most common start index (40-100) of text that follows a run of two or more spaces; a line is cut there when the preceding character is a space, otherwise at the nearest two-space run within 12 characters, a line with neither stays whole in the left column, and a page with fewer than five such starts is kept as is. Column text is otherwise unchanged (trailing spaces of left-column lines and leading spaces of right-column lines are dropped; blank right-column lines are collapsed): the multiset of whitespace-separated words equals the extract's. Then one `## § <n>` line is inserted before each of the chapter's 48 section heading lines: a line opening with `809.<digits>`, whitespace and a capitalized title word (not an all-capitals running head), taking each section's last such line so that the first page's table of contents stays unmarked. Deleting the marker lines (`sed '/^## § /d'`) restores the single-column text; the column split is not byte-reversible to the extract.
- Source: as the extract `wisctapp/wis-stat-ch809.txt`, https://docs.legis.wisconsin.gov/statutes/statutes/809.pdf, retrieved 2026-10-27; the sectioning was done in the pod from the vendored extract.
- Edition: as the extract, Updated 2023-24 Wis. Stats., published and certified under s. 35.18, October 1, 2026.
- Sectioned sha256 55eaa45120fd15038303810878dddae1289081531ce4a658a8f46d01a12feaad (the `path_sha256` pinned by `lit/wi-circuit/Lit.toml` and `lit/wi-court-of-appeals/Lit.toml`).
