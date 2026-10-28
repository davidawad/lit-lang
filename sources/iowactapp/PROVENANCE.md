# `iowactapp` sources: provenance

### `iowactapp/iowa-rules-appellate-procedure.txt` — Iowa Rules of Appellate Procedure (Ch. 6)

- Source: https://www.legis.iowa.gov/docs/ACO/CR/LINC/09-30-2026.chapter.6.pdf (official site; 84 pages, sha256 8d3078cdc61be6ddb556a18c8dacf85f9126c6663ed4db7be2858419daf0e8b6).
- Edition: Iowa Court Rules, Chapter 6 Rules of Appellate Procedure (running header December 2025), in the 09-30-2026 compilation linked from https://www.legis.iowa.gov/law/courtRules/courtRulesListings.
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 cab303e717c4815ebb7309ffe65ffc8d2a4afc3d35ea0910a358bd3f0f0a05a8. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `iowactapp/original/iowa-rules-appellate-procedure.pdf` as served (sha256 8d3078cdc61be6ddb556a18c8dacf85f9126c6663ed4db7be2858419daf0e8b6); extraction sha256 cab303e717c4815ebb7309ffe65ffc8d2a4afc3d35ea0910a358bd3f0f0a05a8.

### `iowactapp/iowa-rules-appellate-procedure-sectioned.txt` — sectioned

- Derived in the pod (lg-f3q8.6) from `iowactapp/iowa-rules-appellate-procedure.txt` (extract sha256 cab303e717c4815ebb7309ffe65ffc8d2a4afc3d35ea0910a358bd3f0f0a05a8); nothing fetched. One `## Rule <n>` line is inserted before the first body heading line of each rule (`Rule 6.1` through `Rule 6.1601`, 70 rules; a heading after a page's form feed keeps its form feed on the heading line, after the marker). The table of contents and the repeated timeline and chart headings are unmarked. Rule text was not edited: deleting the marker lines (`sed '/^## Rule /d'`) restores the extract byte for byte.
- Source: as the extract `iowactapp/iowa-rules-appellate-procedure.txt`, https://www.legis.iowa.gov/docs/ACO/CR/LINC/09-30-2026.chapter.6.pdf, retrieved 2026-10-27; the sectioning was done in the pod from the vendored extract.
- Edition: as the extract, Iowa Court Rules chapter 6 (running header December 2025) in the 09-30-2026 compilation.
- Sectioned sha256 a6cb0e7fcb06ccd217389445992200360cfe96182b201bea2d150cc13109326d.
