
### `fed-ca9/ninth-circuit-rules-handbook.txt` — Ninth Circuit Rules handbook (FRAP, Circuit Rules, Advisory Committee Notes)

- Source: https://cdn.ca9.uscourts.gov/datastore/uploads/frap-June%201%202026.pdf (official site; 200 pages, sha256 35cdf85df1ee701744bb4263a6708a1ee56b174bab9ea955a8cf133f8752536c).
- Edition: Federal Rules of Appellate Procedure, Ninth Circuit Rules and Circuit Advisory Committee Notes handbook, dated 1 June 2026 on the document; linked as the current "Download the Rules Handbook" (rules effective June 1, 2026) from https://www.ca9.uscourts.gov/rules/. Fetched with curl (robots.txt allows all). FRAP portion duplicates sources/frap/; the Circuit Rules and notes are the new content.
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 5bcbaeace671fa049912afa9b4eab81aa502f60d47b1a8728d0e16adbd3b71e4. Sectioned for cite resolution (lg-f3q8.40): a `## 9th Cir. R. N-M` marker line was inserted before the body heading (`CIRCUIT RULE N-M.`) of each numbered circuit rule; 119 markers, the table of contents, FRAP text and Circuit Advisory Committee Notes are unmarked. Rule text was not edited: deleting the marker lines (`sed '/^## 9th Cir\. R\. /d'`) restores the extract byte for byte.
- Sectioned file sha256 e47fce4d98e88dda1e46d2261128d449b92d90291308c4280bf8f40149bac805 (the `path_sha256` pinned by `lit/ca9/Lit.toml`).
- Original: `fed-ca9/original/ninth-circuit-rules-handbook.pdf` as served (sha256 35cdf85df1ee701744bb4263a6708a1ee56b174bab9ea955a8cf133f8752536c); extraction sha256 5bcbaeace671fa049912afa9b4eab81aa502f60d47b1a8728d0e16adbd3b71e4.
