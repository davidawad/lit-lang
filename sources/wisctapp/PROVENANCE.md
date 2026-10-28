
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

### `wisctapp/wis-stat-ch808-sectioned.txt` — Wis. Stat. ch. 808, Appeals and Writs of Error, sectioned

- Derived in the pod (lg-f3q8.56) from `wisctapp/wis-stat-ch808.txt` (extract sha256 13476b13519c64dd3851e0d0de00720cdccac71055ecfc2caae9ac7e7d328fee); nothing fetched. One `## § <n>` line marks each of the chapter's 11 sections after the table of contents: before the line on which the section's own heading (`808.NN Title.`) opens the left column, or right after that line when the heading opens the right column. The extract keeps the PDF's two-column layout, so a section's text can run on lines before or after its marker beside the neighbouring column; the marker binds a cite to the section number, not a span of text. Rule text was not edited: deleting the marker lines (`sed '/^## § /d'`) restores the extract byte for byte.
- Edition: as the extract, Updated 2023-24 Wis. Stats., published and certified under s. 35.18, October 1, 2026.
- Sectioned sha256 bdc5f6b0b274f25ea6bb90292c8861df60d0685f411d2c0b6cf4ea6ff91d55d7 (the `path_sha256` pinned by `lit/wi-circuit/Lit.toml` and `lit/wisctapp/Lit.toml`).

### `wisctapp/wis-stat-ch809-sectioned.txt` — Wis. Stat. ch. 809, Rules of Appellate Procedure, sectioned

- Derived in the pod (lg-f3q8.56) from `wisctapp/wis-stat-ch809.txt` (extract sha256 0a7e60956659899ae03f119c463ba49ff5361af5565ca60e38a5896ee0b9f0e0); nothing fetched. One `## § <n>` line marks each of the chapter's 48 sections after the table of contents: before the line on which the section's own heading (`809.NN Title.`) opens the left column, or right after that line when the heading opens the right column. The extract keeps the PDF's two-column layout, so a section's text can run on lines before or after its marker beside the neighbouring column; the marker binds a cite to the section number, not a span of text. Rule text was not edited: deleting the marker lines (`sed '/^## § /d'`) restores the extract byte for byte.
- Edition: as the extract, Updated 2023-24 Wis. Stats., published and certified under s. 35.18, October 1, 2026.
- Sectioned sha256 5cc2109434c64be92597fa10295efe42aaeae37826a3dfc72bcf9b83881e64aa (the `path_sha256` pinned by `lit/wi-circuit/Lit.toml` and `lit/wisctapp/Lit.toml`).
