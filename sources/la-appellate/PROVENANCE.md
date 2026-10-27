
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

### `la-appellate/la-uniform-rules-courts-of-appeal.txt` — Uniform Rules of Louisiana Courts of Appeal

- Source: https://www.la-fcca.org/clerks_office/UniformRulesoCtsofAppeal (redirects to the civiclive CDN file "Uniform Rules of La. Courts of Appeal - with amendments effective on July 1, 2026.pdf") (official site; 32 pages, sha256 2a99c1755c07ab9e380ecb51e0e97207dfb8454b4ab75d17b2c57f1e83601b63).
- Edition: "As amended, with amendments effective as of July 1, 2026" (as stated on the document; page https://www.la-fcca.org/clerks_office/UniformRulesoCtsofAppeal).
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 8ce01cf1ca72910ca029f8b490ab9521527b4fad0f5e78c2a14b940acabb1b87. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `la-appellate/original/la-uniform-rules-courts-of-appeal.pdf` as served (sha256 2a99c1755c07ab9e380ecb51e0e97207dfb8454b4ab75d17b2c57f1e83601b63); extraction sha256 8ce01cf1ca72910ca029f8b490ab9521527b4fad0f5e78c2a14b940acabb1b87.
