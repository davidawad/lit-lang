
### `fed-dnmi/dnmi-local-rules.txt` — District Court for the Northern Mariana Islands Local Rules

- Source: https://www.nmid.uscourts.gov/sites/nmid/files/documents/localrules/LR20171101.pdf (official site; 62 pages, sha256 501739f4579d2e6c7db27e0c29393b091d3a8247e75c7cd36260ee2109ee5b3b).
- Edition: District Court for the Northern Mariana Islands Local Rules, "Effective November 1, 2017", linked as "Local Rules - Rev. 11/17" from https://www.nmid.uscourts.gov/local-and-federal-rules (the page lists no later revision of the general local rules).
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 10aa60e69e8ac7b965cd1d8fbf9dd61fd3c25d2cf290def5639b8d545a5430fc. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `fed-dnmi/original/dnmi-local-rules.pdf` as served (sha256 501739f4579d2e6c7db27e0c29393b091d3a8247e75c7cd36260ee2109ee5b3b); extraction sha256 10aa60e69e8ac7b965cd1d8fbf9dd61fd3c25d2cf290def5639b8d545a5430fc.

### `fed-dnmi/dnmi-local-rules-sectioned.txt` — District Court for the Northern Mariana Islands Local Rules, sectioned

- Derived in the pod (lg-f3q8.46) from `fed-dnmi/dnmi-local-rules.txt` (extract sha256 10aa60e69e8ac7b965cd1d8fbf9dd61fd3c25d2cf290def5639b8d545a5430fc); nothing fetched. One `## LR <n>` line is inserted before each Local Rule's own heading line in the body (`LR 1.1` through `LR 83.6`, 46 rules, including the headings that follow a form feed); the table of contents and Appendix A are unmarked. Rule text was not edited: deleting the marker lines (`sed '/^## LR /d'`) restores the extract byte for byte.
- Source: as the extract `fed-dnmi/dnmi-local-rules.txt`, https://www.nmid.uscourts.gov/sites/nmid/files/documents/localrules/LR20171101.pdf, retrieved 2026-10-27; the sectioning was done in the pod from the vendored extract.
- Edition: as the extract, Local Rules effective November 1, 2017.
- Sectioned sha256 c40e5f1e8a92ca70a998430299de4caf85022655e471b77de48c0923b2f1f13a (the `path_sha256` pinned by `lit/nmid/Lit.toml`).

### `fed-dnmi/dnmi-criminal-local-rules-2004-01-01.txt` — D.N.M.I. Criminal Local Rules (effective January 1, 2004)

- Source: https://www.nmid.uscourts.gov/sites/nmid/files/documents/localrules/LCRR20040101.pdf (official site; 7 pages, sha256 fd8584adf4a4787abd2f2f79b11d49d8192fe7729b8a5f7c9a1b4c1d0404318f).
- Edition: Criminal Local Rules 'Effective January 1, 2004' (Rev. 01/04, title page), the file LCRR20040101 linked at https://www.nmid.uscourts.gov/local-and-federal-rules. OLD EDITION: the court presents it as current. robots.txt of www.nmid.uscourts.gov read (Crawl-delay 10 honored where stated; no Disallow covers the fetched path); the judiciary-wide Privacy & Security Policy (https://www.uscourts.gov/privacy-security-policy) read: no prohibition of automated access. Fetched with curl and the identifying UA lit-lang-provenance/1.0, at most 1 request per 10 s.
- Criminal forum (lg-gddt): nmid-criminal.
- Retrieved 2026-10-28 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 157e37d65f19be8b450358de65715524b3895693b12d3b0b99c5913bfbb1eb73. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `fed-dnmi/original/dnmi-criminal-local-rules-2004-01-01.pdf` as served (sha256 fd8584adf4a4787abd2f2f79b11d49d8192fe7729b8a5f7c9a1b4c1d0404318f); extraction sha256 157e37d65f19be8b450358de65715524b3895693b12d3b0b99c5913bfbb1eb73.

### `fed-dnmi/dnmi-criminal-local-rules-2004-01-01-sectioned.txt` — D.N.M.I. Criminal Local Rules (effective January 1, 2004), sectioned

- Derived in the pod (lg-gddt.22) from `fed-dnmi/dnmi-criminal-local-rules-2004-01-01.txt` (extract sha256 157e37d65f19be8b450358de65715524b3895693b12d3b0b99c5913bfbb1eb73); nothing fetched. One marker line `## LCrR <n>` is inserted before each rule's own body heading line `LCrR <n> - <Title>` (LCrR 12.1 and 17.1.1, the two rules the table of contents lists); the table of contents, preface and index carry no markers. Every other line is the extract unchanged, so deleting the marker lines gives back the extract byte for byte.
- Source: as the extract `fed-dnmi/dnmi-criminal-local-rules-2004-01-01.txt`, https://www.nmid.uscourts.gov/sites/nmid/files/documents/localrules/LCRR20040101.pdf; Retrieved 2026-10-28 with the extract (this file was derived from it in the pod, not fetched itself).
- Edition: as the extract, effective January 1, 2004 (Rev. 01/04). OLD EDITION presented by the court as current.
- Sectioned sha256 2ef63a281046eb186e64712ccf2c56aafc25ee69cc63ace4c21a91b1357f907d (the `path_sha256` pinned by `lit/nmid-criminal/Lit.toml`).
