
### `dme/local-rules.txt` — D. Me. Local Rules

- Source: https://www.med.uscourts.gov/sites/med/files/LocalRules.pdf (official site; 163 pages, sha256 131911bfe696ae77d1d4828385d2f8c83cbc533c2317a5128483d5049f152232).
- Edition: Local Rules "current as of April 1, 2025" (cover); listed as "Local Rules effective April 1, 2025" on https://www.med.uscourts.gov/local-rules; fetched with curl (plain User-Agent), no bot refusal.
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 16641390d368857b9f175e31a6d5d41b3954f6e31805f4cb00237d867aa575bd. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `dme/original/local-rules.pdf` as served (sha256 131911bfe696ae77d1d4828385d2f8c83cbc533c2317a5128483d5049f152232); extraction sha256 16641390d368857b9f175e31a6d5d41b3954f6e31805f4cb00237d867aa575bd.

### `dme/local-rules-sectioned.txt` — D. Me. Local Rules, sectioned

- Derived in the pod (lg-f3q8.22) from `dme/local-rules.txt` (extract sha256 16641390d368857b9f175e31a6d5d41b3954f6e31805f4cb00237d867aa575bd); nothing fetched. One `## Rule <n>` line is inserted before each rule's own heading line in the body (`Rule 1      Scope and Organization` through `Rule 158 Petty Offenses`, 64 rules; the table of contents is unmarked). Rule text was not edited: deleting the marker lines (`sed '/^## Rule /d'`) restores the extract byte for byte.
- Edition: as the extract, the Local Rules current as of April 1, 2025.
- Sectioned sha256 82b0cb10a256f615971c8e76af39de5957a7be11dc2a302448ffede7efa5617b (the `path_sha256` pinned by `lit/med/Lit.toml`).
