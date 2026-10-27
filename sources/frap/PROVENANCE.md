# `frap` sources: provenance

### `frap/frap.txt` — Federal Rules of Appellate Procedure

- Source: https://www.uscourts.gov/sites/default/files/document/federal-rules-of-appellate-procedure.pdf (official site; 80 pages, sha256 a6f417607a71c05a3a54df68a43ec67e691dcf276c46a415e78038ae4f9305f4).
- Edition: title page: "Federal Rules of Appellate Procedure with Forms, December 1, 2025" (House Judiciary Committee Print No. 2, 119th Cong.; foreword: "as amended to December 1, 2025"); the current FRAP PDF linked from https://www.uscourts.gov/rules-policies/current-rules-practice-procedure/federal-rules-appellate-procedure.
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with OCR (pdftoppm 300 dpi, tesseract 5.5.3, --psm 6) because the PDF's Type 3 fonts carry no text mapping (pdftotext yields glyph noise); unedited OCR, so verify any quoted value against the PDF.
- Extract sha256 1d5a3ff17813cc8cb47c3cdab65adf8db170a8a71b3aae2b187e151975415017. Raw extract, not yet sectioned into `## <ref>` form; it sits in a subdirectory, so the binary does not embed it until a pack wires a sectioned file.
- OCR guard: never fill a number from this OCR alone. Prefer `sources/frap.txt` (Dec. 1, 2024 edition verified line by line, plus the Apr. 23, 2025 amendments to Rules 6 and 39 effective Dec. 1, 2025), which matches this edition; cross-check any number taken from here against it or the uscourts.gov or Cornell LII HTML text of the same rule, cited in a note. A number with no second source stays a hole.
- Original: `frap/original/frap.pdf` as served (sha256 a6f417607a71c05a3a54df68a43ec67e691dcf276c46a415e78038ae4f9305f4); extraction sha256 1d5a3ff17813cc8cb47c3cdab65adf8db170a8a71b3aae2b187e151975415017.
