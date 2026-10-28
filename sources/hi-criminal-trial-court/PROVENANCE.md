
### `hi-criminal-trial-court/hi-rules-of-penal-procedure.txt` — Hawaii Rules of Penal Procedure (91 pp; Rule 48 is the speedy-trial rule)

- Source: https://www.courts.state.hi.us/wp-content/uploads/2024/12/hrpp_ada.pdf (official site; 91 pages, sha256 e1522722db69f47383413375ecac3e16da216ac88f5c2b72cc2bf176587c0d7c).
- Edition: Hawai'i Rules of Penal Procedure (SCRU-11-0000083), 'Effective January 1, 1977, with amendments as noted'; the [pdf] link on https://www.courts.state.hi.us/legal_references/rules/rulesOfCourt, where the page ('Last Updated: October 12, 2026') shows the HRPP row with effective date 07/01/2026 (PDF metadata modified 2026-08-07). www.courts.state.hi.us robots.txt read; no Disallow covering this path; Privacy Policy page (https://portal.ehawaii.gov/page/privacy-policy/) read: no automated-access prohibition found; fetched on David's explicit ruling 2026-10-28 despite robots/terms; tool: curl with the identifying User-Agent lit-lang-provenance/1.0 (angrybohr@proton.me).
- Retrieved 2026-10-28 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 d241e02beb03335a225404cba0efb29d77396b629abc6b0ff4eccabb08cce9cf. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `hi-criminal-trial-court/original/hi-rules-of-penal-procedure.pdf` as served (sha256 e1522722db69f47383413375ecac3e16da216ac88f5c2b72cc2bf176587c0d7c); extraction sha256 d241e02beb03335a225404cba0efb29d77396b629abc6b0ff4eccabb08cce9cf.

- Note (lg-gddt): Criminal appeals: Hawai'i Rules of Appellate Procedure, covered by sources/hi-ica/ if it carries them.

### `hi-criminal-trial-court/hi-rules-of-penal-procedure-sectioned.txt` — Hawai'i Rules of Penal Procedure, sectioned

- Derived in the pod (lg-gddt.22) from `hi-criminal-trial-court/hi-rules-of-penal-procedure.txt` (extract sha256 d241e02beb03335a225404cba0efb29d77396b629abc6b0ff4eccabb08cce9cf); nothing fetched. The extract is set in two columns, so one marker line `## Rule <n>` is inserted before each extract line that holds a rule's own body heading `Rule <n>.` followed by its capitalized title, in either column (from extract line 640, after the table of contents; 73 rules, HRPP 1 through 60; HRPP 22 has no body heading of that form). A marker therefore opens the line where its heading appears, and the column layout interleaves text from neighboring rules; the markers locate a rule, they do not separate its text. Every other line is the extract unchanged, so deleting the marker lines gives back the extract byte for byte.
- Edition: as the extract; the Judiciary's rules page lists HRPP effective 07/01/2026.
- Sectioned sha256 76f2070718af4002062c4becb85a3b7b67b90c10385e515ac329857ca380fb59 (the `path_sha256` pinned by `lit/hi-criminal-trial-court/Lit.toml`).
