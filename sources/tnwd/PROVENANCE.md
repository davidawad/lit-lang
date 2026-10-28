
### `tnwd/tnwd-local-rules.txt` — Local Rules, W.D. Tennessee

- Source: https://www.tnwd.uscourts.gov/sites/tnwd/files/Local%20Rules%20%E2%80%93%20Revised%2011.17.%202025%20%28Addition%20of%20LR%2043.1%28h%29%29.pdf (official site; 174 pages, sha256 a6dcdc4ae6458d0a3397f4063af24d6cfd7d00273ae8f4beb14906020078a958).
- Edition: W.D. Tenn. Local Rules, "Revised November 17, 2025" as stated on the document; linked from https://www.tnwd.uscourts.gov/local-rules (the only local-rules PDF on the page besides the separate Local Patent Rules, not vendored).
- Also the criminal local rules (lg-gddt): LCRR 12.1 and following (Local Criminal Rules), first listed at extract line 124.
- Criminal forum (lg-gddt): tnwd-criminal.
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Fetched via curl (single document, first request succeeded, no block encountered).
- Extract sha256 f064c65ee2748e1ed0ed7c937337552231c86cb846f13feadd8a085ea3f7af15. Raw extract, not yet sectioned into `## <ref>` form.

- Fetch method: first fetched with curl and a desktop browser User-Agent before the no-browser-identity rule (PM 2026-10-27); re-fetched on 2026-10-27 with plain curl (default User-Agent, no browser headers): byte-identical (same sha256).
- Original: `tnwd/original/tnwd-local-rules.pdf` as served (sha256 a6dcdc4ae6458d0a3397f4063af24d6cfd7d00273ae8f4beb14906020078a958); extraction sha256 f064c65ee2748e1ed0ed7c937337552231c86cb846f13feadd8a085ea3f7af15.

### `tnwd/tnwd-local-rules-sectioned.txt` — W.D. Tenn. Local Rules, sectioned

- Derived in the pod (lg-f3q8.62) from `tnwd/tnwd-local-rules.txt` (extract sha256 f064c65ee2748e1ed0ed7c937337552231c86cb846f13feadd8a085ea3f7af15); nothing fetched. One `## LR <n>` line is inserted before each civil rule's own centered heading line in the body (`LR 1.1` through `LR 83.13`, 42 rules); the table of contents, the Local Criminal Rules (LCrR) and the appendices are unmarked. Rule text was not edited: deleting the marker lines (`sed '/^## LR /d'`) restores the extract byte for byte.
- Upstream: https://www.tnwd.uscourts.gov/sites/tnwd/files/Local%20Rules%20%E2%80%93%20Revised%2011.17.%202025%20%28Addition%20of%20LR%2043.1%28h%29%29.pdf, retrieved 2026-10-27, as recorded on the `tnwd/tnwd-local-rules.txt` entry this file is derived from.
- Edition: as the extract, "Revised November 17, 2025".
- Sectioned sha256 75f557e0dcc6de99ab8512854428135b80340d4692d5dae9e5a7c1a3ad424854 (the `path_sha256` pinned by `lit/tnwd/Lit.toml`).
