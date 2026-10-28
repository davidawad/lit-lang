
### `ca10/ca10-rules-2026.txt` — Federal Rules of Appellate Procedure and Tenth Circuit Rules (2026 combined)

- Source: https://www.ca10.uscourts.gov/sites/ca10/files/clerk/2026Rules.pdf (official site; 233 pages, sha256 5aced0c2e14322d1771308be6d14013b761e4a1ea922def3b2f83762218392c8).
- Edition: FRAP effective December 1, 2025 and Tenth Circuit Rules effective January 1, 2026, as stated on the title page; linked from https://www.ca10.uscourts.gov/clerk/rules ("2026 FRAP and Local Rules"); fetched with plain curl (no browser identity); the public-comment 2027 revisions listed on that page are not in force and are not vendored.
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 c6e0504a5e01939236b0cae42008e043206e41f51b1b198f3f185815653081e6. Sectioned for cite resolution (lg-f3q8.66): a `## 10th Cir. R. N` marker line was inserted before the body heading of each circuit rule (37) and of each numbered subrule such as `31.1` (103), 140 markers in all; the table of contents, the FRAP text and the appendices are unmarked. Rule text was not edited: deleting the marker lines (`sed '/^## 10th Cir\. /d'`) restores the extract byte for byte.
- Sectioned file sha256 8e8a55b6873e547f519ca65a4f09cf5e5181efaf71838a8908ef0aebd065327a (the `path_sha256` pinned by `lit/ca10/Lit.toml`).
- Original: `ca10/original/ca10-rules-2026.pdf` as served (sha256 5aced0c2e14322d1771308be6d14013b761e4a1ea922def3b2f83762218392c8); extraction sha256 c6e0504a5e01939236b0cae42008e043206e41f51b1b198f3f185815653081e6.
