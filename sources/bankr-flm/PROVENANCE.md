
### `bankr-flm/flmb-lbr-2025-08-15.txt` — Local Rules of the U.S. Bankruptcy Court for the Middle District of Florida (effective August 15, 2025)

- Source: https://www.flmb.uscourts.gov/localrules/Rules/localrules-FLMB.pdf (official site; 190 pages, sha256 e96e57d2570509e6ca7b06b689d722ef2b66f7ec906a2eb9d715389d01854891).
- Edition: Local Rules, 'EFFECTIVE AUGUST 15, 2025' (first page), the consolidated PDF linked from https://www.flmb.uscourts.gov/localrules/ (Local Rules page; its notes list amendments effective Aug 15, 2025). robots.txt (the only rules disallow GPTBot, which does not cover this fetch) and the Privacy Policy and Disclaimer checked: no prohibition of automated access. Fetched with curl and the identifying User-Agent lit-lang-provenance/1.0 (angrybohr@proton.me).
- Retrieved 2026-10-28 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 3eb5c661e839b370de92001a72be621b45d1d6fe62a674b92ff0b3186197592f. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `bankr-flm/original/flmb-lbr-2025-08-15.pdf` as served (sha256 e96e57d2570509e6ca7b06b689d722ef2b66f7ec906a2eb9d715389d01854891); extraction sha256 3eb5c661e839b370de92001a72be621b45d1d6fe62a674b92ff0b3186197592f.

### `bankr-flm/flmb-lbr-2025-08-15-sectioned.txt` — M.D. Fla. Local Rules, sectioned

- Derived in the pod (lg-u561.29) from `bankr-flm/flmb-lbr-2025-08-15.txt` (extract sha256 3eb5c661e839b370de92001a72be621b45d1d6fe62a674b92ff0b3186197592f); nothing fetched. One `## Rule <n>` line is inserted before each rule's own heading line in the body (the `Rule <n>` line standing alone above the rule title, `Rule 1001-1` through `Rule 9072-1`, including `Rule 1004-1.1` and the headings that follow a form feed; 74 sections, matching the table of contents); the cover and table of contents (extract lines 1-222) and the appendices of forms after Rule 9072-1 are unmarked, as are cross-references that begin a line with `Rule <n>` followed by text. Rule text, including the Notes of Advisory Committee, was not edited: deleting the marker lines (`sed '/^## Rule /d'`) restores the extract byte for byte.
- Edition: as the extract, the Local Rules with amendments effective August 15, 2025.
- Sectioned sha256 646ada77e4e62497d3005b16de4be86f4b21701344b561c67c1def00e876e31f (the `path_sha256` pinned by `lit/bankr-flm/Lit.toml` for source `flmb-lbr`).
