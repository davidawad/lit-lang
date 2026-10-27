
### `md-fla/local-rules.txt` — M.D. Fla. Local Rules

- Source: https://www.flmd.uscourts.gov/sites/flmd/files/flmd-amended-local-rules-effective-november-01-2025.pdf (official site; 54 pages, sha256 effa65b4e132c1321e34361626788a232ce8c725cc238d6a7731e344495961ed).
- Edition: Local Rules dated November 1, 2025 (cover); linked from https://www.flmd.uscourts.gov/local-rules; fetched with curl (plain User-Agent), no bot refusal.
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 92f710c73e85c36fb841a424b8d53a0752f04a473679da8909b83e6610bec4e1. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `md-fla/original/local-rules.pdf` as served (sha256 effa65b4e132c1321e34361626788a232ce8c725cc238d6a7731e344495961ed); extraction sha256 92f710c73e85c36fb841a424b8d53a0752f04a473679da8909b83e6610bec4e1.

### `md-fla/local-rules-sectioned.txt` — M.D. Fla. Local Rules, sectioned

- Derived in the pod (lg-f3q8.14) from `md-fla/local-rules.txt` (extract sha256 92f710c73e85c36fb841a424b8d53a0752f04a473679da8909b83e6610bec4e1); nothing fetched. One `## Rule <n>` line is inserted before each rule's own heading line in the body (`Rule 1.01 Purpose, Scope, and Definitions` through `Rule 7.03 Court's Registry`, 42 rules; the table of contents is unmarked). Rule text was not edited: deleting the marker lines (`sed '/^## Rule /d'`) restores the extract byte for byte.
- Edition: as the extract, Local Rules dated November 1, 2025.
- Sectioned sha256 ffd9c96a2e3ffded589f9a3034557cc7f9f916762b8c043e92a2b3cbdae4c583 (the `path_sha256` pinned by `lit/flmd/Lit.toml`).
