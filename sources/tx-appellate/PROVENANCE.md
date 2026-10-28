
### `tx-appellate/tx-trap.txt` — Texas Rules of Appellate Procedure

- Source: https://www.txcourts.gov/media/1463134/texas-rules-of-appellate-procedure-02062026.pdf (official site; 147 pages, sha256 28d0d6236d84e1740c07ceb7a369602588c89787aed8d24a513700f369ebcf51).
- Edition: Texas Rules of Appellate Procedure, PDF titled "02.06.2026"; https://www.txcourts.gov/rules-forms/rules-standards/ lists "Last Amended February 6, 2026" (current). robots.txt and site-policies read first: no ban. Fetch method: curl with UA litgraph-rules-vendor/1.0, no bot challenge. Note: two-column table of contents interleaves under pdftotext -layout.
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 427adb6389e666466863bc73592b01bebaa25500bc54def4a1ff0c4139e17e4a. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `tx-appellate/original/tx-trap.pdf` as served (sha256 28d0d6236d84e1740c07ceb7a369602588c89787aed8d24a513700f369ebcf51); extraction sha256 427adb6389e666466863bc73592b01bebaa25500bc54def4a1ff0c4139e17e4a.

### `tx-appellate/tx-trap-sectioned.txt` — Texas Rules of Appellate Procedure, column-ordered and sectioned

- Derived in the pod (lg-gddt.14) from the vendored original `tx-appellate/original/tx-trap.pdf` (sha256 28d0d6236d84e1740c07ceb7a369602588c89787aed8d24a513700f369ebcf51, read with `git sparse-checkout add`; nothing fetched from the court). `tx-trap.txt` interleaves the PDF's two text columns line by line, so it cannot be sectioned by inserting lines. This file re-extracts the same PDF in reading order with `pdftotext -layout` (poppler 25.03): each two-column page as its left half (`-x 0 -W 306 -H 792`) followed by its right half (`-x 306 -W 306`), and each single-column page (the appendices, and one table-of-contents page) whole; pages are joined by form feeds. A page counts as single-column when more than a fifth of its full-width `-layout` lines (and more than three) run text unbroken across the page's middle. The column-ordered extract (before markers) has sha256 c73fd63356f30e59a16cce647470962dfa86343345f53ce7b6fbaad558da6019.
- In the body (from `Rule 1. Scope of Rule; ...` after the table of contents, to the Appendix), one marker line `## Tex. R. App. P. <n>` is inserted before each rule's own heading line `Rule <n>. ...` and each subdivision's heading line `<n>.<m>. ...` (431 markers, Rule 1 through 80.1; 28.2 is repealed and has no heading). Deleting the `## Tex. R. App. P.` lines gives back the column-ordered extract byte for byte.
- Edition: as the extract, "Last Amended February 6, 2026".
- Sectioned sha256 4c3fb9bd4d05d8822b740e4b2fdc0c36755d2c5ee425a383ce8f4e5cbf407861 (the `path_sha256` pinned by `lit/tx-court-of-criminal-appeals/Lit.toml`).
