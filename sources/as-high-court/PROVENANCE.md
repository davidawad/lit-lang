
### `as-high-court/as-civil-rules-asbar.txt` — UNOFFICIAL (American Samoa Bar Association posting; no official publisher) American Samoa High Court Trial Court Rules of Civil Procedure (TCRCP)

- Source: https://asbar.org/?p=16558 (1 official HTML pages, one GET each).
- Edition: UNOFFICIAL (American Samoa Bar Association posting; no official publisher). The page https://asbar.org/?p=16558 (page modified not stated per its metadata; no edition or effective date is stated on the page, CURRENCY UNCONFIRMED); the American Samoa Bar Association is not an official source and the court publishes none. fetched on David's explicit ruling 2026-10-28 despite robots/terms; barrier: none (asbar.org robots.txt has an empty Disallow and no terms ban was found; the row was held because the bar association is not an official publisher); tool: curl with the identifying User-Agent lit-lang-provenance/1.0 (angrybohr@proton.me).
- Retrieved 2026-10-28 on carbon (lg-f3q8 standing method); each page extracted with html2txt v1 (stdlib html.parser, tools in the lane scratch), unedited, concatenated in rule order under one `## <heading>` per page.
- Extract sha256 9eba052d0fa3afd7593747bcf28f14ce78a89966b0e5f9e8765cff517f38924e. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `as-high-court/original/as-civil-rules-asbar/0001-asbar-civil.html` as served (sha256 1bac82c97a4a11274c7ac3f9e501e083f51c7a1d49d6bb17e965b5aeca105a3a). Served at https://asbar.org/?p=16558.
- Extraction sha256 9eba052d0fa3afd7593747bcf28f14ce78a89966b0e5f9e8765cff517f38924e.

### `as-high-court/as-civil-rules-asbar-sectioned.txt` — American Samoa Trial Court Rules of Civil Procedure (TCRCP), sectioned

- Derived in the pod (lg-f3q8.36) from `as-high-court/as-civil-rules-asbar.txt` (extract sha256 9eba052d0fa3afd7593747bcf28f14ce78a89966b0e5f9e8765cff517f38924e); nothing fetched. One `## Rule <n>` line is inserted before each body heading line `RULE <n>.` (after the table of contents), 103 rules from `Rule 1` through `Rule 103` (with 23.1 for the page's `RULE 23-1.`, 23.2, 44.1, 65.1 and 71A), each number once; `RULES 72-76.(ABROGATED).`, the Supplemental Admiralty Rules A to F and Appendix A are unmarked. Rule text was not edited: deleting the marker lines (`sed '/^## Rule [0-9]/d'`) restores the extract byte for byte.
- Upstream: https://asbar.org/?p=16558, retrieved 2026-10-28, as recorded on the `as-high-court/as-civil-rules-asbar.txt` entry this file is derived from.
- Edition: as the extract, UNOFFICIAL and CURRENCY UNCONFIRMED. The latest effective date the text states is March 1, 1996 (Rule 11); `lit/as-high-court` dates the expression `as_of = 1996-03-01` and binds values with that caveat (David's ruling 2026-10-28, lg-f3q8.77).
- Sectioned sha256 7a4a6079bd076411bb5a3865f4d552872717546061d535c0f1e067f7f2a7f04f (the `path_sha256` pinned by `lit/as-high-court/Lit.toml`).
