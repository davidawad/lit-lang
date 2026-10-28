
### `bankr-co/cob-local-rules-2021-12-01.txt` — Local Bankruptcy Rules, U.S. Bankruptcy Court for the District of Colorado (effective December 1, 2021, HTML page)

- Source: https://www.cob.uscourts.gov/local-rules (official site; HTML page, sha256 908f9d880b3248b8c62ff57c52f6e990cda626c6a0e14f7b9e0f6bdcebc7fb12).
- Edition: page text: 'Local Bankruptcy Rules, effective December 1, 2021', all rules on one HTML page at https://www.cob.uscourts.gov/local-rules; CURRENCY not independently confirmed beyond the page's own statement. robots.txt (no rule covering these paths) checked; the site's 'Privacy and Security' footer link resolves to the home page and no terms or policy page was found, so no prohibition of automated access was found. Fetched with curl and the identifying User-Agent lit-lang-provenance/1.0 (angrybohr@proton.me).
- Retrieved 2026-10-28 on carbon (lg-f3q8 standing method); extracted with html2txt v1 (stdlib html.parser, tools/html2txt.py in the lg-u561 fetch scratch: drops script/style/nav/header/footer/form subtrees, block tags break lines), unedited.
- Extract sha256 323fca7c875854e0702322e3e37fa461272a1b5820d9a08e48a546105d356990. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `bankr-co/original/cob-local-rules-2021-12-01.html` as served (sha256 908f9d880b3248b8c62ff57c52f6e990cda626c6a0e14f7b9e0f6bdcebc7fb12); extraction sha256 323fca7c875854e0702322e3e37fa461272a1b5820d9a08e48a546105d356990.

### `bankr-co/cob-local-rules-2021-12-01-sectioned.txt` — Bankr. D. Colo. Local Bankruptcy Rules, sectioned

- Derived in the pod (lg-u561.27) from `bankr-co/cob-local-rules-2021-12-01.txt` (extract sha256 323fca7c875854e0702322e3e37fa461272a1b5820d9a08e48a546105d356990); nothing fetched. One `## LBR <n>` line is inserted before each rule's own heading line in the body (a line opening `L.B.R. <nnnn>-<n>` or `L.B.R. <nnnn>.<n>-<n>` followed by its title, after the table of contents; the commentary lines that open with a pinpoint such as `L.B.R. 9010-1(c)(1),` are unmarked), from `L.B.R. 1001-1.` through `L.B.R. 9070-1.`, 69 rules, the same set as the table of contents; the table of contents is unmarked. The HTML extract carries no subsection letters, so cites into it stop at the rule. Rule text was not edited: deleting the marker lines (`sed '/^## LBR /d'`) restores the extract byte for byte.
- Edition: as the extract.
- Sectioned sha256 b4f6b57ff1af5e4e5e38190f74c90ad7837900fb1587c4316c0787841f82e281 (the `path_sha256` pinned by `lit/bankr-co/Lit.toml`).
