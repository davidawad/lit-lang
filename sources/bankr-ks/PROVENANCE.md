
### `bankr-ks/ksb-lbr-2024-12-01.txt` — Local Rules, U.S. Bankruptcy Court for the District of Kansas (effective December 1, 2024)

- Source: https://www.ksb.uscourts.gov/sites/ksb/files/KS_LBRs_2024Dec.pdf (official site; 96 pages, sha256 1c4c7af5a509e71f7e36065c2f96996ab5493a0476066bc06941493132dfa7b8).
- Edition: Local Rules, link text 'Local Rules for Kansas Bankruptcy Court effective December 1, 2024' on https://www.ksb.uscourts.gov/local-rules. robots.txt (Crawl-delay 10 honored; no rule covering these paths) and the Privacy Policy checked: no prohibition of automated access. Fetched with curl and the identifying User-Agent lit-lang-provenance/1.0 (angrybohr@proton.me).
- Retrieved 2026-10-28 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 364365d48891cf177a6da1d124d2009ca271ae9104aa4f0d46f5b7365669476f. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `bankr-ks/original/ksb-lbr-2024-12-01.pdf` as served (sha256 1c4c7af5a509e71f7e36065c2f96996ab5493a0476066bc06941493132dfa7b8); extraction sha256 364365d48891cf177a6da1d124d2009ca271ae9104aa4f0d46f5b7365669476f.

### `bankr-ks/ksb-lbr-2024-12-01-sectioned.txt` — Bankr. D. Kan. Local Rules, sectioned

- Derived in the pod (lg-u561.27) from `bankr-ks/ksb-lbr-2024-12-01.txt` (extract sha256 364365d48891cf177a6da1d124d2009ca271ae9104aa4f0d46f5b7365669476f); nothing fetched. One `## LBR <n>` line is inserted before each rule's own centered heading line in the body (`LBR <nnnn>.<n>`, after an optional form feed), from `LBR 1001.1` through `LBR 9074.1`; the table of contents, Interim LBR 1020.1 (headed `INTERIM LBR 1020.1`), the standing orders and the forms after LBR 9074.1 are unmarked. The rules numbered with a subdivision letter (`LBR 3015(b).1`, `3015(b).2`, `3015(h).1`; `LBR 4001(a).1`, `4001(a).3`) are marked once each as `## LBR 3015` and `## LBR 4001`, the section running across the group, because the citation grammar reads `(b)` as a pinpoint; 64 markers in all. Rule text was not edited: deleting the marker lines (`sed '/^## LBR /d'`) restores the extract byte for byte.
- Edition: as the extract.
- Sectioned sha256 07d0a30338c5ccb3716b8f3db9b3006a5927cb965fc584bb9a574b7b9b9047f2 (the `path_sha256` pinned by `lit/bankr-ks/Lit.toml`).
