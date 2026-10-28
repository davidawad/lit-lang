
### `bankr-az/azb-lbr-2026-06-01.txt` — Local Rules of Bankruptcy Procedure for the District of Arizona (as revised through June 2026)

- Source: https://www.azb.uscourts.gov/sites/azb/files/Local%20Rules%20Final%20Effective%20June%201%202026.pdf (official site; 76 pages, sha256 7b7e8795655e7d9039b72335a153bcbd93d38fec53d494e1176eb36ba53d1660).
- Edition: Local Rules of Bankruptcy Procedure 'As Revised Through June 2026' (cover; file name 'Final Effective June 1 2026'), linked as 'Local Rules' from https://www.azb.uscourts.gov/local-rules. robots.txt (Crawl-delay 10 honored; no rule covering these paths) and the Privacy Policy checked: no prohibition of automated access. Fetched with curl and the identifying User-Agent lit-lang-provenance/1.0 (angrybohr@proton.me).
- Retrieved 2026-10-28 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 5f11e84a2b829523606b51bc7b26105e8b7c5b299e8cc61f7ce2cbbdb4f028a8. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `bankr-az/original/azb-lbr-2026-06-01.pdf` as served (sha256 7b7e8795655e7d9039b72335a153bcbd93d38fec53d494e1176eb36ba53d1660); extraction sha256 5f11e84a2b829523606b51bc7b26105e8b7c5b299e8cc61f7ce2cbbdb4f028a8.

### `bankr-az/azb-lbr-2026-06-01-sectioned.txt` — Bankr. D. Ariz. Local Rules of Bankruptcy Procedure, sectioned

- Derived in the pod (lg-u561.25) from `bankr-az/azb-lbr-2026-06-01.txt` (extract sha256 5f11e84a2b829523606b51bc7b26105e8b7c5b299e8cc61f7ce2cbbdb4f028a8); nothing fetched. One `## LBR <n>` line is inserted before each rule's own heading line in the body (a line opening with `Rule <nnnn>-<n>.`; the set is every such heading after the table of contents, from `1000-1` through `9076-1`, 98 rules); rule text was not edited: deleting the marker lines (`sed '/^## LBR /d'`) restores the extract byte for byte.
- Edition: as the extract.
- Sectioned sha256 00e707dfecbac3cb1db639b06d58c5f7444dfee799197c98245ed067135ed86b (the `path_sha256` pinned by `lit/bankr-az/Lit.toml`).
