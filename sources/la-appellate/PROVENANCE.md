
### `la-appellate/la-ccp.txt` — Louisiana Code of Civil Procedure, selected articles (6)

- Source: Louisiana State Legislature, legis.la.gov (official), table of contents https://legis.la.gov/Legis/Laws_Toc.aspx?folder=68&level=Parent; current text as shown after the 2026 Regular Session (articles carry Acts 2026 amendment notes where amended).
- Method: PM-approved bounded per-article fetch (lg-f3q8.18): only the articles cited by the outline package, one request each at about 1 per second, UA litgraph-rules-vendor/1.0; legis.la.gov has no robots.txt (404) and its Disclaimer page states no access restriction. Retrieved 2026-10-27 on carbon. Text is the page body, unedited except HTML stripped.
- Extract sha256 ebdc94a5fcf7cc7ddfe49781f6fb12b8f4412b5d66f6aed3603e25d623c73373. Raw extract, one `## La. C.C.P. art. <n>` section per article; sits in a subdirectory, so the binary does not embed it until a pack wires it.
- Article URLs:
  - art. 1841: https://legis.la.gov/Legis/Law.aspx?d=111319
  - art. 2083: https://legis.la.gov/Legis/Law.aspx?d=111376
  - art. 2121: https://legis.la.gov/Legis/Law.aspx?d=111383
  - art. 2123: https://legis.la.gov/Legis/Law.aspx?d=111385
  - art. 2127: https://legis.la.gov/Legis/Law.aspx?d=111390
  - art. 2164: https://legis.la.gov/Legis/Law.aspx?d=111403
- Original: `la-appellate/original/la-ccp/001-Legis-Law.aspx_d-111319.html` as served (sha256 ee3f55c5eb8fd64e0cbe37b72f0c75d4042c7d7192750aff1472aaabcd8b2efe); extraction sha256 ebdc94a5fcf7cc7ddfe49781f6fb12b8f4412b5d66f6aed3603e25d623c73373. Served at https://legis.la.gov/Legis/Law.aspx?d=111319; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `la-appellate/original/la-ccp/002-Legis-Law.aspx_d-111376.html` as served (sha256 ddffa845ff92dd84f99f0b30ad87170c50e6b61e38abc3438336dbc454834a05); extraction sha256 ebdc94a5fcf7cc7ddfe49781f6fb12b8f4412b5d66f6aed3603e25d623c73373. Served at https://legis.la.gov/Legis/Law.aspx?d=111376; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `la-appellate/original/la-ccp/003-Legis-Law.aspx_d-111383.html` as served (sha256 127c99bbcef9fbf2f152547526195dc5cb725762f5c5766d2575516e44812176); extraction sha256 ebdc94a5fcf7cc7ddfe49781f6fb12b8f4412b5d66f6aed3603e25d623c73373. Served at https://legis.la.gov/Legis/Law.aspx?d=111383; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `la-appellate/original/la-ccp/004-Legis-Law.aspx_d-111385.html` as served (sha256 89fb3b0f1dd035b77025806798e0574278b14f438ca23ce74435fb669d345ee4); extraction sha256 ebdc94a5fcf7cc7ddfe49781f6fb12b8f4412b5d66f6aed3603e25d623c73373. Served at https://legis.la.gov/Legis/Law.aspx?d=111385; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `la-appellate/original/la-ccp/005-Legis-Law.aspx_d-111390.html` as served (sha256 4739c481d121eac82e2ef1deb0c84a228d509a69dfa3fd7834a26a043e831471); extraction sha256 ebdc94a5fcf7cc7ddfe49781f6fb12b8f4412b5d66f6aed3603e25d623c73373. Served at https://legis.la.gov/Legis/Law.aspx?d=111390; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.
- Original: `la-appellate/original/la-ccp/006-Legis-Law.aspx_d-111403.html` as served (sha256 4ca5673197154c0a4b909e3abbf753e0483ffc31a8c8af458cef2b07c6381f2a); extraction sha256 ebdc94a5fcf7cc7ddfe49781f6fb12b8f4412b5d66f6aed3603e25d623c73373. Served at https://legis.la.gov/Legis/Law.aspx?d=111403; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.

### `la-appellate/la-uniform-rules-courts-of-appeal.txt` — Uniform Rules of Louisiana Courts of Appeal

- Source: https://www.la-fcca.org/clerks_office/UniformRulesoCtsofAppeal (redirects to the civiclive CDN file "Uniform Rules of La. Courts of Appeal - with amendments effective on July 1, 2026.pdf") (official site; 32 pages, sha256 2a99c1755c07ab9e380ecb51e0e97207dfb8454b4ab75d17b2c57f1e83601b63).
- Edition: "As amended, with amendments effective as of July 1, 2026" (as stated on the document; page https://www.la-fcca.org/clerks_office/UniformRulesoCtsofAppeal).
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 8ce01cf1ca72910ca029f8b490ab9521527b4fad0f5e78c2a14b940acabb1b87. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `la-appellate/original/la-uniform-rules-courts-of-appeal.pdf` as served (sha256 2a99c1755c07ab9e380ecb51e0e97207dfb8454b4ab75d17b2c57f1e83601b63); extraction sha256 8ce01cf1ca72910ca029f8b490ab9521527b4fad0f5e78c2a14b940acabb1b87.

### `la-appellate/la-uniform-rules-courts-of-appeal-sectioned.txt` — Uniform Rules of Louisiana Courts of Appeal, sectioned

- Derived in the pod (lg-f3q8.18) from `la-appellate/la-uniform-rules-courts-of-appeal.txt` (extract sha256 8ce01cf1ca72910ca029f8b490ab9521527b4fad0f5e78c2a14b940acabb1b87); nothing fetched. One `## Rule <r>-<n>` line is inserted before each rule heading (`Rule 1-1. Promulgation ...` through `Rule 5-4. Applicability of Rules`, 40 rules), and one `## Rule <r>-<n>.<m>` line before each numbered subrule heading (`1-1.1. Promulgation and Effective Date` through `4-1.1. Briefs on Applications for Writs`, 82 subrules, including `2-2.2 Transmission of Record`, printed without its period). Every other line is the extract unchanged, so deleting the `## Rule` lines gives back the extract byte for byte.
- Edition: as the extract, "As amended, with amendments effective as of July 1, 2026".
- Sectioned sha256 fd58ac4ae45ac05c6ef162914d19826689f9d1f19da5a61538d33be1751fc491 (the `path_sha256` pinned by `lit/la-appellate/Lit.toml`).
