
### `tnwd/tnwd-local-rules.txt` — Local Rules, W.D. Tennessee

- Source: https://www.tnwd.uscourts.gov/sites/tnwd/files/Local%20Rules%20%E2%80%93%20Revised%2011.17.%202025%20%28Addition%20of%20LR%2043.1%28h%29%29.pdf (official site; 174 pages, sha256 a6dcdc4ae6458d0a3397f4063af24d6cfd7d00273ae8f4beb14906020078a958).
- Edition: W.D. Tenn. Local Rules, "Revised November 17, 2025" as stated on the document; linked from https://www.tnwd.uscourts.gov/local-rules (the only local-rules PDF on the page besides the separate Local Patent Rules, not vendored).
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Fetched via curl (single document, first request succeeded, no block encountered).
- Extract sha256 f064c65ee2748e1ed0ed7c937337552231c86cb846f13feadd8a085ea3f7af15. Raw extract, not yet sectioned into `## <ref>` form; it sits in a subdirectory, so the binary does not embed it until a pack wires a sectioned file.

- Fetch method: first fetched with curl and a desktop browser User-Agent before the no-browser-identity rule (PM 2026-10-27); re-fetched on 2026-10-27 with plain curl (default User-Agent, no browser headers): byte-identical (same sha256).
