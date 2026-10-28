
### `bia/cfr-8-part1003.txt` — 8 C.F.R. Part 1003 (Executive Office for Immigration Review; Board of Immigration Appeals procedure)

- Source: https://www.ecfr.gov/api/versioner/v1/full/2026-10-23/title-8.xml?part=1003 (eCFR, official site of the Office of the Federal Register; XML sha256 5ef4b8a7f61596ef6dceed394a5605314a4d53235a39b169b2eeba13688707fb). Fetched with curl --compressed (the API answered 406 without Accept-Encoding); eCFR robots.txt does not disallow /api/.
- Edition: eCFR text current through 2026-10-23 (title 8 up_to_date_as_of 2026-10-23, latest_amended_on 2026-10-14, per https://www.ecfr.gov/api/versioner/v1/titles.json). The eCFR is an unofficial editorial compilation; the official edition is the annual CFR.
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); XML flattened to text, one `## 8 C.F.R. § N` heading per section (1046 paragraphs, 65 sections) by a short script, paragraph text unedited.
- Extract sha256 73881d597138882b36b902474e2d6318a4a03ee09928e304dc1395e3c8f98d7f.
- Original: `bia/original/cfr-8-part1003.xml` as served (sha256 5ef4b8a7f61596ef6dceed394a5605314a4d53235a39b169b2eeba13688707fb); extraction sha256 73881d597138882b36b902474e2d6318a4a03ee09928e304dc1395e3c8f98d7f. Re-fetched 2026-10-28 on carbon with curl --compressed from the same URL (the date-pinned versioner URL returned a byte-identical file); fetched on David's explicit ruling 2026-10-28 despite robots (eCFR robots.txt, User-agent *, `Disallow: /api/versioner/v1/full/`, which covers this URL), tool curl. Correction: the earlier statement in this entry that the eCFR robots.txt does not disallow /api/ was wrong; the 2026-10-23 fetch was within that robots Disallow, now covered retroactively by David's ruling 2026-10-28.

### `bia/cfr-8-part1003-sectioned.txt` — 8 C.F.R. Part 1003, sectioned

- Derived in the pod (lg-f3q8.34) from `bia/cfr-8-part1003.txt` (extract sha256 73881d597138882b36b902474e2d6318a4a03ee09928e304dc1395e3c8f98d7f); nothing fetched. Each of the 65 `## 8 C.F.R. § N <title>` heading lines is split into a bare `## 8 C.F.R. § N` marker, which the cite corpus matches exactly, followed by the original heading text without `## ` (`8 C.F.R. § N <title>`). Section text was not edited: `sed -E '/^## 8 C\.F\.R\. § [0-9.]+$/{N;s/^## [^\n]*\n/## /}'` restores the extract byte for byte.
- Source: as the extract `bia/cfr-8-part1003.txt`, https://www.ecfr.gov/api/versioner/v1/full/2026-10-23/title-8.xml?part=1003, retrieved 2026-10-27; the sectioning was done in the pod from the vendored extract.
- Edition: as the extract, eCFR text current through 2026-10-23.
- Sectioned sha256 57f3f034cc4a081a1e70f39ee9f2e848a0fd0b3e58fdb281de21c7f38aed67ce (the `path_sha256` pinned by `lit/bia/Lit.toml`).
