
### `fed-dnmi/dnmi-local-rules.txt` — District Court for the Northern Mariana Islands Local Rules

- Source: https://www.nmid.uscourts.gov/sites/nmid/files/documents/localrules/LR20171101.pdf (official site; 62 pages, sha256 501739f4579d2e6c7db27e0c29393b091d3a8247e75c7cd36260ee2109ee5b3b).
- Edition: District Court for the Northern Mariana Islands Local Rules, "Effective November 1, 2017", linked as "Local Rules - Rev. 11/17" from https://www.nmid.uscourts.gov/local-and-federal-rules (the page lists no later revision of the general local rules).
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 10aa60e69e8ac7b965cd1d8fbf9dd61fd3c25d2cf290def5639b8d545a5430fc. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `fed-dnmi/original/dnmi-local-rules.pdf` as served (sha256 501739f4579d2e6c7db27e0c29393b091d3a8247e75c7cd36260ee2109ee5b3b); extraction sha256 10aa60e69e8ac7b965cd1d8fbf9dd61fd3c25d2cf290def5639b8d545a5430fc.

### `fed-dnmi/dnmi-local-rules-sectioned.txt` — District Court for the Northern Mariana Islands Local Rules, sectioned

- Derived in the pod (lg-f3q8.46) from `fed-dnmi/dnmi-local-rules.txt` (extract sha256 10aa60e69e8ac7b965cd1d8fbf9dd61fd3c25d2cf290def5639b8d545a5430fc); nothing fetched. One `## LR <n>` line is inserted before each Local Rule's own heading line in the body (`LR 1.1` through `LR 83.6`, 46 rules, including the headings that follow a form feed); the table of contents and Appendix A are unmarked. Rule text was not edited: deleting the marker lines (`sed '/^## LR /d'`) restores the extract byte for byte.
- Edition: as the extract, Local Rules effective November 1, 2017.
- Sectioned sha256 c40e5f1e8a92ca70a998430299de4caf85022655e471b77de48c0923b2f1f13a (the `path_sha256` pinned by `lit/nmid/Lit.toml`).
