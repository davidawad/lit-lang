
### `ks-state-appellate/kansas-2026-rulebook.txt` — Kansas Supreme Court 2026 Rulebook

- Source: https://www.kscourts.gov/KSCourts/media/KsCourts/Rules/2026-RuleBook.pdf (linked from https://www.kscourts.gov/Rules-Orders/Rules) (official site; 712 pages, sha256 6e897c9c92c2ca8be21ad0dc4710e0694ceac979d2d60115bbb9547b747d2bd0).
- Edition: Reports of Rules Adopted by the Supreme Court of the State of Kansas, 2026 Edition.
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 13ba212187f676520b745153fc5dedd5aeafacb006bd17f01df7b027b3e65f24. Raw extract, not yet sectioned into `## <ref>` form.
- Fetch method: BrowserOS neo; site served a bot challenge to plain automated requests (CoS ruling 2026-10-27). kscourts.gov robots.txt has no Disallow; its Terms & Conditions page bars no automated access.
- Original: `ks-state-appellate/original/kansas-2026-rulebook.pdf` as served (sha256 6e897c9c92c2ca8be21ad0dc4710e0694ceac979d2d60115bbb9547b747d2bd0); extraction sha256 13ba212187f676520b745153fc5dedd5aeafacb006bd17f01df7b027b3e65f24.

### `ks-state-appellate/kansas-2026-rulebook-sectioned.txt` — Kansas Supreme Court 2026 Rulebook, sectioned

- Derived in the pod (lg-f3q8.67) from `ks-state-appellate/kansas-2026-rulebook.txt` (extract sha256 13ba212187f676520b745153fc5dedd5aeafacb006bd17f01df7b027b3e65f24); nothing fetched. One `## Kan. S. Ct. R. <n>` line is inserted right after each rule's own centered body heading line `Rule <n>` (362 markers, Rule 1.01 through Rule 1901; no heading number repeats). Running heads, tables of contents and in-text references to rules are unmarked. Rule text was not edited: deleting the marker lines (each `## Kan. S. Ct. R. ` line with its newline) restores the extract byte for byte.
- Source: as the extract `ks-state-appellate/kansas-2026-rulebook.txt`, https://www.kscourts.gov/KSCourts/media/KsCourts/Rules/2026-RuleBook.pdf, retrieved 2026-10-27; the sectioning was done in the pod from the vendored extract.
- Edition: as the extract, Reports of Rules Adopted by the Supreme Court of the State of Kansas, 2026 Edition.
- Sectioned sha256 847c087e907f9696356fd0f57fc545bd354df58b085e8183608d0b93a9592dc9 (the `path_sha256` pinned by `lit/ks-court-of-appeals/Lit.toml`).
