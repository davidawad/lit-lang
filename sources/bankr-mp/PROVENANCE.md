
### `bankr-mp/nmid-lbkr-2019-04-15.txt` — Local Bankruptcy Rules, District Court for the Northern Mariana Islands (effective April 15, 2019)

- Source: https://www.nmid.uscourts.gov/sites/nmid/files/documents/localrules/LBKR20190415.pdf (official site; 69 pages, sha256 171604598760df24fb36c46c53f8d760e1f24a5c81f0510d9633747bc67738ca).
- Edition: Local Bankruptcy Rules 'Effective April 15, 2019' (cover), linked as 'Local Bankruptcy Rules' from https://www.nmid.uscourts.gov/local-and-federal-rules. robots.txt (Crawl-delay 10 honored; no rule covering these paths) and the Privacy Policy checked: no prohibition of automated access. Fetched with curl and the identifying User-Agent lit-lang-provenance/1.0 (angrybohr@proton.me).
- Retrieved 2026-10-28 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 2a80b5499472e9f853a8438390c92f1c099d179a0e619a6b48032c3a60e525c5. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `bankr-mp/original/nmid-lbkr-2019-04-15.pdf` as served (sha256 171604598760df24fb36c46c53f8d760e1f24a5c81f0510d9633747bc67738ca); extraction sha256 2a80b5499472e9f853a8438390c92f1c099d179a0e619a6b48032c3a60e525c5.

### `bankr-mp/nmid-lbkr-2019-04-15-sectioned.txt` — Local Bankruptcy Rules, District Court for the Northern Mariana Islands, sectioned

- Derived in the pod (lg-f3q8.46) from `bankr-mp/nmid-lbkr-2019-04-15.txt` (extract sha256 2a80b5499472e9f853a8438390c92f1c099d179a0e619a6b48032c3a60e525c5); nothing fetched. One `## LBR <n>` line is inserted before each Local Bankruptcy Rule's own heading line in the body (`LBR 1001-1` through `LBR 9074-1`, 92 rules, including the headings that follow a form feed); the table of contents is unmarked. Rule text was not edited: deleting the marker lines (`sed '/^## LBR /d'`) restores the extract byte for byte.
- Edition: as the extract, Local Bankruptcy Rules effective April 15, 2019.
- Sectioned sha256 ba8d528b40f75361b5dfad4dc4825a4d29e824f9f529e6600e98dbb7903aa98b (the `path_sha256` pinned by `lit/bankr-mp/Lit.toml`).
