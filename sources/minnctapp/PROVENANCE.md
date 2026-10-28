
### `minnctapp/mnrcap-2025-04-01.txt` — Minnesota Rules of Civil Appellate Procedure (amendments effective April 1, 2025)

- Source: https://mncourts.gov/_media/migration/appellate/supreme-court/court-rules/rules-of-civil-app-proc-effective040125.pdf (official site; 109 pages, sha256 35f2dc1e2847e13cba11df37a73290ce7fb5d469024e70b77f349526dddfb068).
- Edition: Minnesota Rules of Civil Appellate Procedure 'With amendments effective April 1, 2025' (title page), linked at https://mncourts.gov/supremecourt/court-rules. The Recent Rules Orders page lists no later order amending these rules. fetched on David's explicit ruling 2026-10-28 despite robots/terms (barrier: Cloudflare challenge on mncourts.gov, cf-mitigated: challenge, against plain curl; tool: BrowserOS neo, a real browser, one download per document, intake copy trashed).
- Retrieved 2026-10-28 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 5aca074e9b5e7d80cce977fb5ebca7230b452dc40d01e68699a14ae46dce5708. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `minnctapp/original/mnrcap-2025-04-01.pdf` as served (sha256 35f2dc1e2847e13cba11df37a73290ce7fb5d469024e70b77f349526dddfb068); extraction sha256 5aca074e9b5e7d80cce977fb5ebca7230b452dc40d01e68699a14ae46dce5708.

### `minnctapp/mnrcap-2025-04-01-sectioned.txt` — Minnesota Rules of Civil Appellate Procedure, sectioned

- Derived in the pod (lg-f3q8.7) from `minnctapp/mnrcap-2025-04-01.txt` (extract sha256 5aca074e9b5e7d80cce977fb5ebca7230b452dc40d01e68699a14ae46dce5708); nothing fetched. One `## Rule <n>` line is inserted before the first line, after the table of contents and in its order, that heads each of the 171 rules and numbered subdivisions the table of contents lists (`Rule 101.` and `101.01.` through `Rule 147.`), including reserved rules. Rule text was not edited: deleting the marker lines (`sed '/^## Rule /d'`) restores the extract byte for byte.
- Upstream: https://mncourts.gov/_media/migration/appellate/supreme-court/court-rules/rules-of-civil-app-proc-effective040125.pdf, retrieved 2026-10-28, as recorded on the `minnctapp/mnrcap-2025-04-01.txt` entry this file is derived from.
- Edition: as the extract, amendments effective April 1, 2025.
- Sectioned sha256 9351c7e8731c2e496ba1ddd3c038b61c25e310f361a27c178889aff43fc6dbfa (the `path_sha256` pinned by `lit/mn-district/Lit.toml` and `lit/minnctapp/Lit.toml` for source `mnrcap`).
