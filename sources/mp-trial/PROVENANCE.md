
### `mp-trial/nmi-rcp.txt` — Northern Mariana Islands Rules of Civil Procedure

- Source: https://cnmilaw.gov/docs/courtrules/R05.pdf (official site; 151 pages, sha256 ed102171ff0add3df38fd83137785ec8ef0de40da754d2cb083f2a20105b71c3).
- Edition: NMI Rules of Civil Procedure (NMI R. CIV. P.), listed as dated 2019-01-09 (R05, current entry; the superseded Com. R. Civ. P. 2014 is a separate old entry) on the CNMI Law Revision Commission Court Rules page https://cnmilaw.gov/court-rules, which the CNMI Judiciary site https://www.nmijudiciary.gov links to as its Court Rules; the PDF opens with the Supreme Court general errata order No. 2019-ADM-0003-RUL (e-filed Feb 25 2019); no later amendment is listed.
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 dad837e00ee658b54f70411460e43d17770b189055a503ee7d5b6430b9769a3d. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `mp-trial/original/nmi-rcp.pdf` as served (sha256 ed102171ff0add3df38fd83137785ec8ef0de40da754d2cb083f2a20105b71c3); extraction sha256 dad837e00ee658b54f70411460e43d17770b189055a503ee7d5b6430b9769a3d.

### `mp-trial/nmi-rcp-sectioned.txt` — Northern Mariana Islands Rules of Civil Procedure, sectioned

- Derived in the pod (lg-f3q8.46) from `mp-trial/nmi-rcp.txt` (extract sha256 dad837e00ee658b54f70411460e43d17770b189055a503ee7d5b6430b9769a3d); nothing fetched. One `## Rule <n>` line is inserted before each rule's own heading line in the body (`Rule 1.` through `Rule 86.`, 97 headings including the reserved rules); the errata order, adoption order and table of contents are unmarked. The extract drops the rules' subdivision letters, so cites bind at the rule level. Rule text was not edited: deleting the marker lines (`sed '/^## Rule /d'`) restores the extract byte for byte.
- Edition: as the extract, the rules adopted January 9, 2019 with the general errata order of February 25, 2019.
- Sectioned sha256 61165fb072592880ebd21303aa0fb06a48bde993ec235ca01ed4c2fd1adab424 (the `path_sha256` pinned by `lit/mp-trial/Lit.toml`).
