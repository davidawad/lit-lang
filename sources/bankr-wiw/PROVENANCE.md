
### `bankr-wiw/wiwb-local-rules-2024-07.txt` — Local Rules, U.S. Bankruptcy Court for the Western District of Wisconsin (effective July 2024, HTML page)

- Source: https://www.wiwb.uscourts.gov/court-info/local-rules-and-orders (official site; HTML page, sha256 4d513d6d087be9903534b4f5722c48a484431faebfd3c42d876399b37db4a527).
- Edition: page text: 'Local Rules Effective July 2024; NOTE: Local Form 3015-1.2 was revised on December 4, 2024', the rules published as an HTML page at https://www.wiwb.uscourts.gov/court-info/local-rules-and-orders. robots.txt (Crawl-delay 10 honored; no rule covering these paths) and the Privacy Policy checked: no prohibition of automated access. Fetched with curl and the identifying User-Agent lit-lang-provenance/1.0 (angrybohr@proton.me).
- Retrieved 2026-10-28 on carbon (lg-f3q8 standing method); extracted with html2txt v1 (stdlib html.parser, tools/html2txt.py in the lg-u561 fetch scratch: drops script/style/nav/header/footer/form subtrees, block tags break lines), unedited.
- Extract sha256 4e33de3e04d645b1f5333703c8047da6b1c5288533da130325f21d83a1104ab8. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `bankr-wiw/original/wiwb-local-rules-2024-07.html` as served (sha256 4d513d6d087be9903534b4f5722c48a484431faebfd3c42d876399b37db4a527); extraction sha256 4e33de3e04d645b1f5333703c8047da6b1c5288533da130325f21d83a1104ab8.

### `bankr-wiw/wiwb-local-rules-2024-07-sectioned.txt` — W.D. Wis. Local Rules, sectioned

- Derived in the pod (lg-u561.21) from `bankr-wiw/wiwb-local-rules-2024-07.txt` (extract sha256 4e33de3e04d645b1f5333703c8047da6b1c5288533da130325f21d83a1104ab8); nothing fetched. One `## LBR <n>` line is inserted before each rule heading (`RULE <n>: <Title>` or `RULE <n> <Title>`; `LBR 1007-1-2`, `1007-2`, `3015-1`, `4004-1`, `4008-1`, `7004-1`; 6 sections); the page's navigation lines and part headings are unmarked. Rule text was not edited: deleting the marker lines (`sed '/^## LBR /d'`) restores the extract byte for byte.
- Edition: as the extract, effective July 2024.
- Sectioned sha256 d09b0100c022945334505e519f77e590cd0f756170d6277c9b80feb37e356799 (the `path_sha256` pinned by `lit/bankr-wiw/Lit.toml` for source `wiwb-lr`).
