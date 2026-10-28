
### `bankr-wie/wieb-lbr-2026-01-01.txt` — Local Rules, U.S. Bankruptcy Court for the Eastern District of Wisconsin (January 1, 2026)

- Source: https://www.wieb.uscourts.gov/sites/default/files/forms/2026%20Local%20Rules%20WIEB.pdf (official site; 52 pages, sha256 9f700de2358b703535be9579d89bc57cc5ee5a73247abca9a0ec66f9bf04a1d6).
- Edition: Local Rules 'January 1, 2026' (cover), linked as 'Local Rules' from https://www.wieb.uscourts.gov/local-rules. robots.txt (Crawl-delay 10 honored; no rule covering these paths) and the Privacy Policy checked: no prohibition of automated access. Fetched with curl and the identifying User-Agent lit-lang-provenance/1.0 (angrybohr@proton.me).
- Retrieved 2026-10-28 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 f5b943fd3845772dd7803b50d6e2516210b7963b6c7638b3560663f4224a9797. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `bankr-wie/original/wieb-lbr-2026-01-01.pdf` as served (sha256 9f700de2358b703535be9579d89bc57cc5ee5a73247abca9a0ec66f9bf04a1d6); extraction sha256 f5b943fd3845772dd7803b50d6e2516210b7963b6c7638b3560663f4224a9797.

### `bankr-wie/wieb-lbr-2026-01-01-sectioned.txt` — E.D. Wis. Local Rules, sectioned

- Derived in the pod (lg-u561.21) from `bankr-wie/wieb-lbr-2026-01-01.txt` (extract sha256 f5b943fd3845772dd7803b50d6e2516210b7963b6c7638b3560663f4224a9797); nothing fetched. One `## LBR <n>` line is inserted before each rule's body heading (`LR <n>  <Title>`, after the table of contents; `LBR 1001-1` through `LBR 9029-2`, `LBR 1004.1-1` keeping its point; 63 sections); the cover and the table of contents are unmarked. Rule text was not edited: deleting the marker lines (`sed '/^## LBR /d'`) restores the extract byte for byte.
- Edition: as the extract, effective January 1, 2026.
- Sectioned sha256 fb9b0c005a53ccc8b3f3e96825651136b91a8bf7151027f700ce2abd7197fa52 (the `path_sha256` pinned by `lit/bankr-wie/Lit.toml` for source `wieb-lr`).
