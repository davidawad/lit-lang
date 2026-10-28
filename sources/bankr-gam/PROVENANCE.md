
### `bankr-gam/gamb-lbr-2026-10-01.txt` — Local Rules, U.S. Bankruptcy Court for the Middle District of Georgia (as of October 1, 2026)

- Source: https://www.gamb.uscourts.gov/USCourts/sites/default/files/local_rules/Local_Rules_Oct_2026.pdf (official site; 51 pages, sha256 223154ca120dcc6573dfa6ad25957f7fe1b5cf87e0bcdf8314792899e299e885).
- Edition: Local Rules 'AS OF OCTOBER 1, 2026' (cover; summary of changes since the April 17, 2026 version), linked from https://www.gamb.uscourts.gov/USCourts/local-rules-and-clerks-instructions. robots.txt (the only rules disallow GPTBot and /calendars/, which do not cover these paths) and the Privacy & Security Policy checked: no prohibition of automated access. Fetched with curl and the identifying User-Agent lit-lang-provenance/1.0 (angrybohr@proton.me).
- Retrieved 2026-10-28 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 46080b307a110bf6a302943da8ac5d5d5b1cbe4bdb29383b49289e59b918c063. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `bankr-gam/original/gamb-lbr-2026-10-01.pdf` as served (sha256 223154ca120dcc6573dfa6ad25957f7fe1b5cf87e0bcdf8314792899e299e885); extraction sha256 46080b307a110bf6a302943da8ac5d5d5b1cbe4bdb29383b49289e59b918c063.

### `bankr-gam/gamb-lbr-2026-10-01-sectioned.txt` — M.D. Ga. Local Rules, sectioned

- Derived in the pod (lg-u561.29) from `bankr-gam/gamb-lbr-2026-10-01.txt` (extract sha256 46080b307a110bf6a302943da8ac5d5d5b1cbe4bdb29383b49289e59b918c063); nothing fetched. One `## Rule <n>` line is inserted before each rule's own heading line in the body (`LBR 1002-1. Petition – General.` through `LBR 9073-1. Hearings`, including `LBR 3002.1-1` and `3002.1-2`, the abrogated `7001-1` and `8010-1` and the headings without a closing period; 77 sections, matching the table of contents); the summary of changes and table of contents (extract lines 1-519) and extract lines 680, 1330 and 1633, cross-references to `LBR 9007-1` and `LBR 4070-1` that begin a line within other rules, are unmarked. Rule text was not edited: deleting the marker lines (`sed '/^## Rule /d'`) restores the extract byte for byte.
- Edition: as the extract, the Local Rules as of October 1, 2026.
- Sectioned sha256 91af8b23a9f2565c56725413de6afb23ee4386318a9782c3af20c38094c37cb0 (the `path_sha256` pinned by `lit/bankr-gam/Lit.toml` for source `gamb-lbr`).
