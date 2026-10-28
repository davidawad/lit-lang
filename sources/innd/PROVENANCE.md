
### `innd/innd-local-rules.txt` — N.D. Indiana Local Rules

- Source: https://www.innd.uscourts.gov/sites/innd/files/CurrentLocalRules.pdf (official site; 136 pages, sha256 380ac564b7712f1836d4d0b8a3b89fae0e7de750abb0658d524212dfb016e097).
- Edition: Local Rules effective February 25, 2022 (the "Current Local Rules" link on https://www.innd.uscourts.gov/court-info/local-rules-and-orders; page lists no later adopted amendments); fetched by curl.
- Also the criminal local rules (lg-gddt): N.D. Ind. L. Cr. R. 1-1 and following, from extract line 2554.
- Criminal forum (lg-gddt): innd-criminal.
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 bed2beeced8a3fd2c10b8759752b1b160d4be978f9243e9fbf6e304f5f27783a. Raw extract, not yet sectioned into `## <ref>` form.

- Fetch method: first fetched with curl and a desktop browser User-Agent before the no-browser-identity rule (PM 2026-10-27); re-fetched on 2026-10-27 with plain curl (its default User-Agent, no browser headers): byte-identical (same sha256).
- Original: `innd/original/innd-local-rules.pdf` as served (sha256 380ac564b7712f1836d4d0b8a3b89fae0e7de750abb0658d524212dfb016e097); extraction sha256 bed2beeced8a3fd2c10b8759752b1b160d4be978f9243e9fbf6e304f5f27783a.

### `innd/innd-local-rules-sectioned.txt` — N.D. Indiana Local Rules, sectioned

- Derived in the pod (lg-f3q8.55) from `innd/innd-local-rules.txt` (extract sha256 bed2beeced8a3fd2c10b8759752b1b160d4be978f9243e9fbf6e304f5f27783a); nothing fetched. One `## N.D. Ind. L.R. <n>` line is inserted before each local civil rule's own heading line in the body (55 rules, L.R. 1-1 through L.R. 200-1; the en dash of `L.R. 1–1` is marked with a hyphen); the table of contents, the criminal rules and the appendices are unmarked. Rule text was not edited: deleting the marker lines (`sed -E '/^## N\.D\. Ind\. L\.R\. /d'`) restores the extract byte for byte.
- Source: as the extract `innd/innd-local-rules.txt`, https://www.innd.uscourts.gov/sites/innd/files/CurrentLocalRules.pdf, retrieved 2026-10-27; the sectioning was done in the pod from the vendored extract.
- Edition: as the extract, Local Rules effective February 25, 2022.
- Sectioned sha256 36c328d79f09abc98ab4ba2db6e38be53bdb8989d70326fd2559e566f9b753ea (the `path_sha256` pinned by `lit/innd/Lit.toml`).

### `innd/innd-criminal-rules-sectioned.txt` — N.D. Indiana Local Rules, criminal rules sectioned

- Derived in the pod (lg-gddt.18) from `innd/innd-local-rules.txt` (extract sha256 bed2beeced8a3fd2c10b8759752b1b160d4be978f9243e9fbf6e304f5f27783a); nothing fetched. One `## N.D. Ind. L. Cr. R. <n>` line is inserted before each local criminal rule's own heading line `N.D. Ind. L. Cr. R. <n> ...` in the body (13 rules, L. Cr. R. 1-1 through 58-1; the en dash of `1–1` is marked with a hyphen). The civil and patent rules, the table of contents and the appendices carry no such markers. Rule text was not edited: deleting the marker lines (`sed -E '/^## N\.D\. Ind\. L\. Cr\. R\. /d'`) restores the extract byte for byte.
- Edition: as the extract, Local Rules effective February 25, 2022.
- Sectioned sha256 894b3c2b4570e93f4a2ff644af4a67225ffa56bd953ca88433b52f13218616cd (the `path_sha256` pinned by `lit/innd-criminal/Lit.toml`).
