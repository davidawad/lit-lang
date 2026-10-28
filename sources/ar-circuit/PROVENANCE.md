# `ar-circuit` sources: provenance

### `ar-circuit/arcp.txt` — Arkansas Rules of Civil Procedure

- Source: https://opinions.arcourts.gov/ark/cr/en/16712/1/document.do (official court site; 196 pages, sha256 b72f17dcaf0261c1b94a76509f2a35ef4d6eceeb21c613a3aea13d682e19c0d6).
- Edition: title: "Rules of Civil Procedure (current through October 1, 2026)"; Arkansas Judiciary court rules portal https://opinions.arcourts.gov/ark/cr/en/nav_date.do lists it current to 10/01/2026.
- Retrieved 2026-10-27 on carbon (the court site refuses agent pods: lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 9e7950b1cca7fc771ef258b4ac2190375118c517646958b83e9c5bb3e2c01430. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `ar-circuit/original/arcp.pdf` as served (sha256 b72f17dcaf0261c1b94a76509f2a35ef4d6eceeb21c613a3aea13d682e19c0d6); extraction sha256 9e7950b1cca7fc771ef258b4ac2190375118c517646958b83e9c5bb3e2c01430.

### `ar-circuit/arcp-sectioned.txt` — Arkansas Rules of Civil Procedure, sectioned

- Derived in the pod (lg-f3q8.3) from `ar-circuit/arcp.txt` (extract sha256 9e7950b1cca7fc771ef258b4ac2190375118c517646958b83e9c5bb3e2c01430); nothing fetched. One `## Rule <n>` line is inserted before each rule's own heading line `Rule <n>. <Title>` (Rule 1 through Rule 88, 90 rules, dotted numbers such as `Rule 23.1` and `Rule 26.1` kept); a body line that happens to begin `Rule <n>.` (a cross-reference in a note) is unmarked because it does not follow the rules' ascending order. Rule text was not edited: deleting the marker lines (`sed '/^## Rule /d'`) restores the extract byte for byte.
- Upstream: https://opinions.arcourts.gov/ark/cr/en/16712/1/document.do, retrieved 2026-10-27, as recorded on the `ar-circuit/arcp.txt` entry this file is derived from.
- Edition: as the extract, current through October 1, 2026.
- Sectioned sha256 6c829071f46786d015e71140098aa042a70e44dcce95ad7f5b4282334a273385 (the `path_sha256` pinned by `lit/ar-circuit/Lit.toml`).
