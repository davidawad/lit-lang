
### `ms-circuit-court/rules-of-civil-procedure.txt` — Mississippi Rules of Civil Procedure

- Source: https://courts.ms.gov/research/rules/msrulesofcourt/2026-07-01%20Rules%20of%20Civil%20Procedure%20with%20Advisory%20Committee%20Notes%20(with%20Forms%206-8%20attached).pdf (official site; 231 pages, sha256 b85e6543980b7f3a7d2d3623d9ec538d9aabec651a30a6589c87424ea6116095).
- Edition: Mississippi Rules of Civil Procedure with Advisory Committee Notes, file dated 2026-07-01 (filename), listed as "Mississippi Rules of Civil Procedure" on https://courts.ms.gov/research/rules/rules.php; amendments in the text run through September 10, 2025; fetched with curl (plain User-Agent) from the PDF URL; the site home page and rules index returned HTTP 500 to curl, so the court rules index page was read once in BrowserOS neo (tab closed) to find the link; site has no terms page and robots.txt does not disallow /research/rules/.
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 243063580951d6243af839734a26e74cc7f96eb3ade408863e0f55bd75db9751. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `ms-circuit-court/original/rules-of-civil-procedure.pdf` as served (sha256 b85e6543980b7f3a7d2d3623d9ec538d9aabec651a30a6589c87424ea6116095); extraction sha256 243063580951d6243af839734a26e74cc7f96eb3ade408863e0f55bd75db9751.

### `ms-circuit-court/rules-of-civil-procedure-sectioned.txt` — Mississippi Rules of Civil Procedure, sectioned

- Derived in the pod (lg-f3q8.19) from `ms-circuit-court/rules-of-civil-procedure.txt` (extract sha256 243063580951d6243af839734a26e74cc7f96eb3ade408863e0f55bd75db9751); nothing fetched. After the table of contents, one `## M.R.C.P. <n>` line is inserted before each rule's own heading line (`RULE 1. SCOPE OF RULES` through `RULE 85. TITLE`, 86 headings including 5.1, 16A, 23.1, 23.2, 44.1 and 71A), and one `## Appendix A` line before `APPENDIX A. FORMS`. Every other line is the extract unchanged, so deleting the `## ` lines gives back the extract byte for byte.
- Upstream: https://courts.ms.gov/research/rules/msrulesofcourt/2026-07-01%20Rules%20of%20Civil%20Procedure%20with%20Advisory%20Committee%20Notes%20(with%20Forms%206-8%20attached, retrieved 2026-10-27, as recorded on the `ms-circuit-court/rules-of-civil-procedure.txt` entry this file is derived from.
- Edition: as the extract (file dated 2026-07-01; amendments in the text through September 10, 2025).
- Sectioned sha256 b18aa683d774d658c98456a49c6b8c6c228b72eaaa11c178582a491e441b7328 (the `path_sha256` pinned by `lit/ms-circuit-court/Lit.toml`).
