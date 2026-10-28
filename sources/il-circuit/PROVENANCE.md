
### `il-circuit/ilcs-735-5-art2.txt` — Illinois Code of Civil Procedure (735 ILCS 5/), Article II

- Source: https://www.ilga.gov/legislation/ILCS/details?MajorTopic=&Chapter=&ActName=Code%20of%20Civil%20Procedure.&ActID=2017&ChapterID=56&ChapAct=735+ILCS+5%2F&SeqStart=1100000&SeqEnd=22400000 (official site; 81 pages, sha256 b0d82ef922bb9ff0a875f014a5f2bfee2e3717b2871696ebd9e5b617a8c83119).
- Edition: 735 ILCS 5/ Code of Civil Procedure, Article II (Civil Practice), the Illinois General Assembly ILCS database as displayed on 2026-10-27; the page states no edition date (it says the database is updated continuously and may lag or lead Public Acts). Fetched by BrowserOS (opened the one Article II page from https://www.ilga.gov/Legislation/ILCS/Articles?ActID=2017&ChapterID=56 and printed it to PDF) because curl fails the TLS chain check on ilga.gov; ilga.gov robots.txt disallows only /account, /admin, /search, /api, crawl-delay 10; one document, no per-page pass; the PDF is a browser print of the page including page header text.
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 2ebfeb1a472928128159c6daeb1acf2c9b59f9127bd3d258ba44b5c9ac169cfa. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `il-circuit/original/ilcs-735-5-art2.pdf` as served (sha256 b0d82ef922bb9ff0a875f014a5f2bfee2e3717b2871696ebd9e5b617a8c83119); extraction sha256 2ebfeb1a472928128159c6daeb1acf2c9b59f9127bd3d258ba44b5c9ac169cfa.

### `il-circuit/ilcs-735-5-art2-sectioned.txt` — 735 ILCS 5/ Article II, sectioned

- Derived in the pod (lg-f3q8.54) from `il-circuit/ilcs-735-5-art2.txt` (extract sha256 2ebfeb1a472928128159c6daeb1acf2c9b59f9127bd3d258ba44b5c9ac169cfa); nothing fetched. One `## 735 ILCS 5/2-<n>` line is inserted before each section's own `(735 ILCS 5/2-<n>)` heading line (190 sections, `2-101` through `2-2301`). A section printed in two versions (with and without an amendment held unconstitutional, e.g. 2-1105) has one heading and one marker, so both versions sit in the one section. Statute text was not edited: deleting the marker lines (`sed '/^## 735 ILCS /d'`) restores the extract byte for byte.
- Upstream: https://www.ilga.gov/legislation/ILCS/details?MajorTopic=&Chapter=&ActName=Code%20of%20Civil%20Procedure.&ActID=2017&ChapterID=56&ChapAct=735+ILCS+5%2F&SeqStart=1100000&SeqEnd=22400000, retrieved 2026-10-27, as recorded on the `il-circuit/ilcs-735-5-art2.txt` entry this file is derived from.
- Edition: as the extract, the ILCS database as displayed on 2026-10-27 (no edition date stated).
- Sectioned sha256 91de34e3946f909cd9f377bdb7fb9d1c1c660f0d605c617eae24a686bd492099 (the `path_sha256` pinned by `lit/il-circuit/Lit.toml`).
