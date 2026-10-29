# `moed` sources: provenance

### `moed/local-rules.txt` — Local Rules and Patent Rules, E.D. Mo.

- Source: https://www.moed.uscourts.gov/sites/moed/files/CMECF_localrule.pdf (official site; 137 pages, sha256 f3039857f47617d8bf625a1f584428eaf59c914e7c14b63aadf129239465aa12).
- Edition: Local Rules and Patent Rules of the U.S. District Court for the Eastern District of Missouri; the cover states no edition date (PDF metadata modified 2026-10-16); the document's own revision table (extract lines 3560-3640, "Revised Local Rules December 2025") lists its latest revisions as approved 10-1-25 and effective 12-15-25 (Rules 2.09, 3.01, 6.03, 12.01 and the Patent Rules), so the edition is current through effective 12-15-25 (the court's separate Revision History PDF lists revisions only through effective 3-1-23 and lags the document); linked as the local rules PDF from https://www.moed.uscourts.gov/local-rules.
- Also the criminal local rules (lg-gddt): no separate criminal part; criminal provisions include Rule 3.05 Disclosure and Motions in Criminal Cases (extract line 834).
- Criminal forum (lg-gddt): moed-criminal.
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 ae524d7184dfe84ecba6dbe1cb3fd9b644837ebfb2a585b83bb768ce565bbbea. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `moed/original/local-rules.pdf` as served (sha256 f3039857f47617d8bf625a1f584428eaf59c914e7c14b63aadf129239465aa12); extraction sha256 ae524d7184dfe84ecba6dbe1cb3fd9b644837ebfb2a585b83bb768ce565bbbea.

### `moed/local-rules-sectioned.txt` — Local Rules, E.D. Mo., sectioned

- Derived in the pod (lg-gddt.20) from `moed/local-rules.txt` (extract sha256 ae524d7184dfe84ecba6dbe1cb3fd9b644837ebfb2a585b83bb768ce565bbbea); nothing fetched. One `## L.R. <n>` line is inserted before each Local Rule's own heading line in the body (`Rule <n> (<federal rule>) <title>`, some after a page's form feed, which stays on the heading line after the marker): `L.R. 1.01` through `L.R. 13.05`, 70 rules, the repealed Rule 4.05 included. The table of contents is unmarked, and the Local Patent Rules and the revision table are not sectioned (L.R. 13.05's section runs to the end of the file). Rule text was not edited: deleting the marker lines (`sed '/^## L\.R\. /d'`) restores the extract byte for byte.
- Edition: as the extract; the cover states no date, and the document's own revision table (extract lines 3560-3640) lists its latest revisions as effective December 15, 2025 (Rules 2.09, 3.01, 6.03, 12.01 and the Patent Rules), as recorded on `moed/local-rules.txt` above.
- Upstream: https://www.moed.uscourts.gov/sites/moed/files/CMECF_localrule.pdf, retrieved 2026-10-27, as recorded on `moed/local-rules.txt` above; sectioning was done from the vendored extract.
- Sectioned sha256 10fa27fbd06ff35c764a88538b7038d18b8905881f8ccf015a63634f81683886 (the `path_sha256` pinned by `lit/moed-criminal/Lit.toml`).
