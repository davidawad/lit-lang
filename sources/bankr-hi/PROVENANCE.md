
### `bankr-hi/hib-lbr-2025-12-01.txt` — Local Bankruptcy Rules, U.S. Bankruptcy Court for the District of Hawaii (December 1, 2025)

- Source: https://www.hib.uscourts.gov/sites/hib/files/documents/local-rules/LBR2025_dec01_rev2.pdf (official site; 163 pages, sha256 302353db200f52ff227d8aade52ee48abc999fa743bedad0647859722931e33f).
- Edition: Local Bankruptcy Rules 'December 1, 2025' (cover), linked as 'Complete Set of Local Bankruptcy Rules (effective 12/01/2025)' from https://www.hib.uscourts.gov/local-rules. robots.txt (Crawl-delay 10 honored; no rule covering these paths) and the Privacy Policy checked: no prohibition of automated access. Fetched with curl and the identifying User-Agent lit-lang-provenance/1.0 (angrybohr@proton.me).
- Retrieved 2026-10-28 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 dc1d1b881abae92b2be90a262ff1164daf8d670a10358a35c2fd1e0da581ffc7. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `bankr-hi/original/hib-lbr-2025-12-01.pdf` as served (sha256 302353db200f52ff227d8aade52ee48abc999fa743bedad0647859722931e33f); extraction sha256 dc1d1b881abae92b2be90a262ff1164daf8d670a10358a35c2fd1e0da581ffc7.

### `bankr-hi/hib-lbr-2025-12-01-sectioned.txt` — Bankr. D. Haw. Local Bankruptcy Rules, sectioned

- Derived in the pod (lg-u561.25) from `bankr-hi/hib-lbr-2025-12-01.txt` (extract sha256 dc1d1b881abae92b2be90a262ff1164daf8d670a10358a35c2fd1e0da581ffc7); nothing fetched. One `## LBR <n>` line is inserted before each rule's own heading line in the body (a line opening, after an optional form feed, with `LBR <nnnn>-<n>. `; the set is every such heading after the table of contents, from `1001-1` through `9074-1`, 109 rules); rule text was not edited: deleting the marker lines (`sed '/^## LBR /d'`) restores the extract byte for byte.
- Edition: as the extract.
- Sectioned sha256 c726e7910c9e114feda5966d2636776815faa362ac33f03ca5392665d893f30f (the `path_sha256` pinned by `lit/bankr-hi/Lit.toml`).
