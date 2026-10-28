
### `va-circuit-civil/va-rules-of-supreme-court.txt` — Rules of the Supreme Court of Virginia (Parts One through Nine incl. Part Three civil, Parts Five and Five A appellate)

- Source: https://www.vacourts.gov/static/courts/scv/rulesofcourt.pdf (official site; 631 pages, sha256 15a5095aba5043b863d2c7fb26f24c2556cad4908054874c0dff4f770af53a0f).
- Edition: Rules of Supreme Court of Virginia, all Parts (PDF modified 2026-09-15; no edition line on the document; https://www.vacourts.gov/courts/scv/rules states "the Rules ... as currently effective" and lists amendments through Rule 3C:2 effective October 2, 2025; the same file serves the va-appellate-civil forum, Parts Five and Five A).
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 860445fd31316518a18830c64cafdc47dd7b0cd86eb59aacf5d3b61ec140d5f4. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `va-circuit-civil/original/va-rules-of-supreme-court.pdf` as served (sha256 15a5095aba5043b863d2c7fb26f24c2556cad4908054874c0dff4f770af53a0f); extraction sha256 860445fd31316518a18830c64cafdc47dd7b0cd86eb59aacf5d3b61ec140d5f4.

### `va-circuit-civil/va-rules-of-supreme-court-sectioned.txt` — Rules of the Supreme Court of Virginia, sectioned

- Derived in the pod (lg-f3q8.31) from `va-circuit-civil/va-rules-of-supreme-court.txt` (extract sha256 860445fd31316518a18830c64cafdc47dd7b0cd86eb59aacf5d3b61ec140d5f4); nothing fetched. One `## Rule <part>:<n>` line is inserted before each rule's own heading line (a line beginning `Rule <part>:<n>` followed by its title, at a page break or after a blank line), 354 rules from `Rule 1:1` through Part Eleven, each number once; cross-references that begin a line mid-paragraph are unmarked. Rule text was not edited: deleting the marker lines (`sed '/^## Rule /d'`) restores the extract byte for byte. The same file serves `lit/va-circuit-civil` (Parts One, Three, Four) and `lit/va-court-of-appeals` (Parts Five A and Five).
- Source: as the extract `va-circuit-civil/va-rules-of-supreme-court.txt`, https://www.vacourts.gov/static/courts/scv/rulesofcourt.pdf, retrieved 2026-10-27; the sectioning was done in the pod from the vendored extract.
- Edition: as the extract, all Parts as currently effective. The latest amendment the text itself states is effective September 15, 2026 (Order dated July 17, 2026, noted under Rules 1:1C, 3A:25, 5:5, 5A:3 and 5A:31), matching the PDF's modification date of 2026-09-15; the packages date the expression `as_of = 2026-09-15`.
- Sectioned sha256 d7909399a2a10c69c9877dc83abfbeb85d9932b1ba1091139295dea303a47f95 (the `path_sha256` pinned by `lit/va-circuit-civil/Lit.toml` and `lit/va-court-of-appeals/Lit.toml`).
