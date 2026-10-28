# `dc-state-civil` sources: provenance

### `dc-state-civil/scr-civil-rules.txt` — Superior Court of the District of Columbia Rules of Civil Procedure

- Source: https://www.dccourts.gov/sites/default/files/superior-rules/Superior-Court-Rules-of-Civil-Procedure-Download-All-Rules-Version.pdf (official court site; 296 pages, sha256 b46082a8edee555671302aab2203905abb57f965e98d3e13c6ccbf89976e9c43).
- Edition: the document states no edition date; it carries comments to 2026 amendments, its PDF is dated 2026-05-13, and it is the all-rules version linked from https://www.dccourts.gov/superior-court/superior-court-resources/rules-of-the-superior-court (page last updated 02/13/2026, before the PDF).
- Retrieved 2026-10-27 on carbon (the court site refuses agent pods: lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 bbf2960ed59c98a8d7a32ea831919317ad59e41f63e9786fb1f27ebf13736f23. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `dc-state-civil/original/scr-civil-rules.pdf` as served (sha256 b46082a8edee555671302aab2203905abb57f965e98d3e13c6ccbf89976e9c43); extraction sha256 bbf2960ed59c98a8d7a32ea831919317ad59e41f63e9786fb1f27ebf13736f23.

### `dc-state-civil/scr-civil-rules-sectioned.txt` — Superior Court of the District of Columbia Rules of Civil Procedure, sectioned

- Derived in the pod (lg-f3q8.115) from `dc-state-civil/scr-civil-rules.txt` (extract sha256 bbf2960ed59c98a8d7a32ea831919317ad59e41f63e9786fb1f27ebf13736f23); nothing fetched. One marker line `## Rule <n>` is inserted before each rule's heading line `Rule <n>. <Title>` (a form feed may lead the heading line; 167 rules, Rule 1 through Rule 310, including the `-I`, `-II` and `-III` local rules). Every other line is the extract unchanged, so deleting the `## Rule` lines (`sed '/^## Rule /d'`) gives back the extract byte for byte.
- Upstream: https://www.dccourts.gov/sites/default/files/superior-rules/Superior-Court-Rules-of-Civil-Procedure-Download-All-Rules-Version.pdf, retrieved 2026-10-27, as recorded on the source extract entry this file is derived from.
- Edition: as the extract (no edition date stated; comments to 2026 amendments; PDF dated 2026-05-13).
- Sectioned sha256 fe4d647085e65f0efa32cde713365878f10dd9d6d9e24a94ec504d28597f1096 (the `path_sha256` pinned by `lit/dc-state-civil/Lit.toml`).
