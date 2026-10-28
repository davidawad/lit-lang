# `ia-district` sources: provenance

### `ia-district/iowa-rules-civil-procedure.txt` — Iowa Rules of Civil Procedure (Ch. 1)

- Source: https://www.legis.iowa.gov/docs/ACO/CR/LINC/09-30-2026.chapter.1.pdf (official site; 130 pages, sha256 ae6ecf5e9a558a4f80cd0e63a5bc3229fb7681ecd5946a4ca8fbe1773f7c56b6).
- Edition: Iowa Court Rules, Chapter 1 Rules of Civil Procedure (running header July 2023), in the 09-30-2026 compilation linked from https://www.legis.iowa.gov/law/courtRules/courtRulesListings.
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 0ac112e11fe546759c5fe480c17f12e079b47f57da08a823aad6f154ebec15fc. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `ia-district/original/iowa-rules-civil-procedure.pdf` as served (sha256 ae6ecf5e9a558a4f80cd0e63a5bc3229fb7681ecd5946a4ca8fbe1773f7c56b6); extraction sha256 0ac112e11fe546759c5fe480c17f12e079b47f57da08a823aad6f154ebec15fc.

### `ia-district/iowa-rules-civil-procedure-sectioned.txt` — sectioned

- Derived in the pod (lg-f3q8.6) from `ia-district/iowa-rules-civil-procedure.txt` (extract sha256 0ac112e11fe546759c5fe480c17f12e079b47f57da08a823aad6f154ebec15fc); nothing fetched. One `## Rule <n>` line is inserted before the first body heading line of each rule (`Rule 1.101` through `Rule 1.1901`, 309 rules; a heading after a page's form feed keeps its form feed on the heading line, after the marker). The table of contents and the repeated timeline and chart headings are unmarked. Rule text was not edited: deleting the marker lines (`sed '/^## Rule /d'`) restores the extract byte for byte.
- Edition: as the extract, Iowa Court Rules chapter 1 (running header July 2023) in the 09-30-2026 compilation.
- Sectioned sha256 5f599c69732cc275e92ca4159c1ae015c5771c7179450bd4ff90de9050d2ee66.
