
### `tned/tned-local-rules.txt` — Local Rules, E.D. Tennessee

- Source: https://www.tned.uscourts.gov/sites/tned/files/Local%20Rules_Active_2026.04.08%20to%20Present.pdf (official site; 110 pages, sha256 c53178a09c2ffb0cbb6cb3eafb2c064564fb89fb44a08f5b05b69f73d70b8ecf).
- Edition: E.D. Tenn. Local Rules, "Effective: December 1, 2009; Revised: April 8, 2026" as stated on the document; https://www.tned.uscourts.gov/local-rules states "(Revised: April 8th, 2026)".
- Also the criminal local rules (lg-gddt): no separate criminal part; criminal provisions include LR16.2 (extract line 645), LR72.4 and LR83.11.
- Criminal forum (lg-gddt): tned-criminal.
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Fetched via curl (single document, first request succeeded, no block encountered).
- Extract sha256 7e9e572cc14452fa9ae9c89adef989bda22d38efad1915ce4533603ca40b096f. Raw extract, not yet sectioned into `## <ref>` form.

- Fetch method: first fetched with curl and a desktop browser User-Agent before the no-browser-identity rule (PM 2026-10-27); re-fetched on 2026-10-27 with plain curl (default User-Agent, no browser headers): byte-identical (same sha256).
- Original: `tned/original/tned-local-rules.pdf` as served (sha256 c53178a09c2ffb0cbb6cb3eafb2c064564fb89fb44a08f5b05b69f73d70b8ecf); extraction sha256 7e9e572cc14452fa9ae9c89adef989bda22d38efad1915ce4533603ca40b096f.

### `tned/tned-local-rules-sectioned.txt` — E.D. Tenn. Local Rules, sectioned

- Derived in the pod (lg-f3q8.62) from `tned/tned-local-rules.txt` (extract sha256 7e9e572cc14452fa9ae9c89adef989bda22d38efad1915ce4533603ca40b096f); nothing fetched. One `## LR <n>` line is inserted before each rule's own heading line in the body (`LR 1.1` through `LR 83.13`, 69 rules; the table of contents and the LR16.5 appendix headings are unmarked). Rule text was not edited: deleting the marker lines (`sed '/^## LR /d'`) restores the extract byte for byte.
- Upstream: https://www.tned.uscourts.gov/sites/tned/files/Local%20Rules_Active_2026.04.08%20to%20Present.pdf, retrieved 2026-10-27, as recorded on the `tned/tned-local-rules.txt` entry this file is derived from.
- Edition: as the extract, "Effective: December 1, 2009; Revised: April 8, 2026".
- Sectioned sha256 9b84efd66ff91cba7627f6a6e5644aaf8fccfd57c864d9b3ec296e4df3480ca9 (the `path_sha256` pinned by `lit/tned/Lit.toml`).
