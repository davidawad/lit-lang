
### `ca6/ca6-full-rules.txt` — Sixth Circuit FRAP, Circuit Rules, IOPs and e-filing guide (consolidated)

- Source: https://www.ca6.uscourts.gov/sites/ca6/files/documents/rules_procedures/Full%20Rules%20w%20FRAP_0.pdf (official site; 147 pages, sha256 3da82c2fc6e2dac77dc7202a840fd318ebb9c0c8898b4488d3022b35a2a8c91e).
- Edition: Sixth Circuit "Full Rules w FRAP" compilation dated July 13, 2026: FRAP [Last Amended December 1, 2025]; 6 Cir. R. [Last Amended July 13, 2026]; I.O.P. [Last Amended October 1, 2025]; E-filing Guide [Last Amended December 1, 2024]; linked from https://www.ca6.uscourts.gov/rules-and-procedures/local-rules-frap-local-rules-iops.
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 ff570075b1e925d47db246c09df8ccdeee46b29512c75fead10ea1bffba9a5e3. Sectioned for cite resolution (lg-f3q8.58): a `## 6 Cir. R. N` or `## 6 Cir. I.O.P. N` marker line was inserted before the body heading of each circuit rule and internal operating procedure (108 markers; the one body heading printed `6 Cir I.O.P. 36` is marked `## 6 Cir. I.O.P. 36`); the table of contents, the FRAP text and the e-filing guide are unmarked. Rule text was not edited: deleting the marker lines (`sed '/^## 6 Cir\. /d'`) restores the extract byte for byte.
- Sectioned file sha256 d6efc05f17fe53d6a5133cf383385156fb42f81078f7a83d6dc3541dc64e0227 (the `path_sha256` pinned by `lit/ca6/Lit.toml`).
- Fetched via curl (single document, first request succeeded, no block encountered; desktop UA, no Referer).

- Fetch method: first fetched with curl and a desktop browser User-Agent before the no-browser-identity rule (PM 2026-10-27); re-fetched on 2026-10-27 with plain curl (default User-Agent, no browser headers): byte-identical (same sha256).
- Original: `ca6/original/ca6-full-rules.pdf` as served (sha256 3da82c2fc6e2dac77dc7202a840fd318ebb9c0c8898b4488d3022b35a2a8c91e); extraction sha256 ff570075b1e925d47db246c09df8ccdeee46b29512c75fead10ea1bffba9a5e3.
