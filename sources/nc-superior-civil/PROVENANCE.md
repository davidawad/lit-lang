
### `nc-superior-civil/nc-rules-of-civil-procedure-ch1a.txt` — North Carolina Rules of Civil Procedure (G.S. Chapter 1A)

- Source: https://www.ncleg.gov/EnactedLegislation/Statutes/PDF/ByChapter/Chapter_1A.pdf (official site; 87 pages, sha256 4913f21d961caabb294db9b1640f89fadf69851f3e10eec5dac93f5c069dc420).
- Edition: N.C. Gen. Stat. Chapter 1A (Rules of Civil Procedure), PDF modified 2026-04-16; https://www.ncleg.gov/Laws/GeneralStatuteSections/Chapter1A states the General Statutes include changes through S.L. 2026-30; also governs North Carolina district court civil practice (nc-appellate-civil outline cites N.C. R. Civ. P. via same source).
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 5fa8a8a02f8f628a0fc6b03d3acd9e52c24762134c22ca779e1a430219ba0267. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `nc-superior-civil/original/nc-rules-of-civil-procedure-ch1a.pdf` as served (sha256 4913f21d961caabb294db9b1640f89fadf69851f3e10eec5dac93f5c069dc420); extraction sha256 5fa8a8a02f8f628a0fc6b03d3acd9e52c24762134c22ca779e1a430219ba0267.

### `nc-superior-civil/nc-rules-of-civil-procedure-ch1a-sectioned.txt` — N.C. Rules of Civil Procedure (Chapter 1A), sectioned

- Derived in the pod (lg-f3q8.29) from `nc-superior-civil/nc-rules-of-civil-procedure-ch1a.txt` (extract sha256 5fa8a8a02f8f628a0fc6b03d3acd9e52c24762134c22ca779e1a430219ba0267); nothing fetched. One `## Rule <n>` line is inserted right before each rule's own heading line `Rule <n>. <title>` (some after a form feed): 71 markers, Rule 1 through Rule 84. In-body cross-references are unmarked. Rule text was not edited: deleting the marker lines (`sed '/^## Rule /d'`) restores the extract byte for byte.
- Edition: as the extract, Chapter 1A with changes through S.L. 2026-30 (PDF modified 2026-04-16).
- Sectioned sha256 aef8620622f572b94dc6c132558bab3468f2f5fbd7012e958db8fc9349414452 (the `path_sha256` pinned by `lit/nc-superior-civil/Lit.toml`).
