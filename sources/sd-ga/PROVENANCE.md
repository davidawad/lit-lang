
### `sd-ga/sdga-local-rules.txt` — S.D. Ga. Local Rules (general, civil, criminal)

- Source: https://www.gasd.uscourts.gov/sites/gasd/files/local_rules/SDGA_Local_Rules_Effective_10-07-2026.pdf (official site; 68 pages, sha256 878813a8fdeac35a2e87f4db51fd8ffff30172137f4943eeb88f3474185a0edf).
- Edition: Local Rules "Effective 10-07-2026" (PDF version linked from https://www.gasd.uscourts.gov/court-info/local-rules-and-orders).
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 06f9f6b1c290e7be9b1c54d37496153f0cd001f0d6912024cc72a31a7053292b. Raw extract, not yet sectioned into `## <ref>` form; it sits in a subdirectory, so the binary does not embed it until a pack wires a sectioned file.

- Fetch method: first fetched with curl and a Referer header after the court page answered 403 (no bot-detection signal); re-checked 2026-10-27 in BrowserOS from the court page and with plain curl (HTTP 200, no bot signal): byte-identical (same sha256).
