
### `bankr-ne/neb-lbr-2026-01-01.txt` — Nebraska Rules of Bankruptcy Procedure (local rules, effective January 1, 2026, updated January 7, 2026)

- Source: https://www.neb.uscourts.gov/sites/neb/files/Local%20Rules%20Bankruptcy%20Court%202026%20FINAL%20AMENDED.pdf (official site; 35 pages, sha256 7707dface831a58cc1be81685114dd8e237c1e34c1a9de4d06e1f08b73fbc1e5).
- Edition: Nebraska Rules of Bankruptcy Procedure, 'Adopted by the United States District Court for the District of Nebraska October 8, 2025; Effective: January 1, 2026; Updated: January 7, 2026' (cover), linked as 'Final Version with Appendix A-H' from https://www.neb.uscourts.gov/local-rules. robots.txt (Crawl-delay 10 honored; no rule covering these paths) and the Privacy Policy checked: no prohibition of automated access. Fetched with curl and the identifying User-Agent lit-lang-provenance/1.0 (angrybohr@proton.me).
- Retrieved 2026-10-28 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 230e0df295628e3ad2f41cb931bac74dcc049e2437bd56174c306f07d064df20. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `bankr-ne/original/neb-lbr-2026-01-01.pdf` as served (sha256 7707dface831a58cc1be81685114dd8e237c1e34c1a9de4d06e1f08b73fbc1e5); extraction sha256 230e0df295628e3ad2f41cb931bac74dcc049e2437bd56174c306f07d064df20.

### `bankr-ne/neb-lbr-2026-01-01-sectioned.txt` — Nebraska Rules of Bankruptcy Procedure, sectioned

- Derived in the pod (lg-u561.23) from `bankr-ne/neb-lbr-2026-01-01.txt` (extract sha256 230e0df295628e3ad2f41cb931bac74dcc049e2437bd56174c306f07d064df20); nothing fetched. One `## Rule <n>` line is inserted before each rule's own heading line in the body (`Rule 1001-1.` through `Rule 9072-1.`, including the headings that follow a form feed; 48 sections, matching the table of contents; the wrapped body line `Rule 9013-1. If all bankruptcy judges ...` in Rule 5011-1 is not a heading and is unmarked), and one `## Appendix <X>` line before each appendix heading (`Appendix A` through `Appendix H`); the table of contents is unmarked. Rule text was not edited: deleting the marker lines (`sed -E '/^## (Rule|Appendix) /d'`) restores the extract byte for byte.
- Edition: as the extract, the rules effective January 1, 2026, updated January 7, 2026.
- Sectioned sha256 a7defdbf20ec08146c42ec3b6e885d09312e74f7ae7cf0f068fc14b83afb7939 (the `path_sha256` pinned by `lit/bankr-ne/Lit.toml` for source `neb-lbr`).
