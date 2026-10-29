
### `vawd/vawd-local-rules.txt` — U.S. District Court for the Western District of Virginia Local Rules

- Source: https://www.vawd.uscourts.gov/sites/Public/assets/File/court/local_rules.pdf (official site; 45 pages, sha256 1ffd80a63eaaf3c4b3ab9e405b4365f43b56ab5d8f8410b58eefb61b707e971c).
- Edition: Local Rules, August 2013 (Revised October 25, 2025) per cover; linked from https://www.vawd.uscourts.gov/court-information/local-rules-and-standing-orders/ as the Local Rules.
- Also the criminal local rules (lg-gddt): 'TITLE III - CRIMINAL RULES' part, from extract line 1758.
- Criminal forum (lg-gddt): vawd-criminal.
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 0cfd1a6b7269d123f79fb9dfa757f09380c07795f76e89cd09cd75efdfe41fe0. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `vawd/original/vawd-local-rules.pdf` as served (sha256 1ffd80a63eaaf3c4b3ab9e405b4365f43b56ab5d8f8410b58eefb61b707e971c); extraction sha256 0cfd1a6b7269d123f79fb9dfa757f09380c07795f76e89cd09cd75efdfe41fe0.

### `vawd/vawd-local-rules-sectioned.txt` — W.D. Va. Local Rules, sectioned

- Derived in the pod (lg-f3q8.31) from `vawd/vawd-local-rules.txt` (extract sha256 0cfd1a6b7269d123f79fb9dfa757f09380c07795f76e89cd09cd75efdfe41fe0); nothing fetched. One `## General Rule <n>` line is inserted before each Title I rule's own heading line in the body (`Rule 1.` through `Rule 12.`, 12 rules) and one `## Civil Rule <n>` line before each Title II rule's (`Rule 3.` through `Rule 83.`, 11 rules); the two titles number independently. The table of contents and Title III (criminal) are unmarked. Rule text was not edited: deleting the marker lines (`sed -E '/^## (General|Civil) Rule /d'`) restores the extract byte for byte.
- Source: as the extract `vawd/vawd-local-rules.txt`, https://www.vawd.uscourts.gov/sites/Public/assets/File/court/local_rules.pdf, retrieved 2026-10-27; the sectioning was done in the pod from the vendored extract.
- Edition: as the extract, August 2013 (revised October 25, 2025).
- Sectioned sha256 a97415133e7e9facfafb09ab53b48677b017ea3fe842af6d729159e8df9615bc (the `path_sha256` pinned by both bindings in `lit/vawd/Lit.toml`).

### `vawd/vawd-local-criminal-rules-sectioned.txt` — W.D. Va. Local Rules (revised October 25, 2025), criminal rules sectioned

- Derived in the pod (lg-gddt.12) from `vawd/vawd-local-rules.txt` (extract sha256 0cfd1a6b7269d123f79fb9dfa757f09380c07795f76e89cd09cd75efdfe41fe0); nothing fetched. One `## Criminal Rule <n>` line is inserted before each Title III rule's own heading line in the body (`Rule 12.`, `Rule 32.`, `Rule 58.`, `Rule 59.`, from extract line 1716 on): 4 markers. The table of contents and Titles I and II are unmarked (they are sectioned separately in `vawd/vawd-local-rules-sectioned.txt`). Rule text was not edited: deleting the marker lines (`sed '/^## Criminal Rule /d'`) restores the extract byte for byte.
- Source: as the extract `vawd/vawd-local-rules.txt`, https://www.vawd.uscourts.gov/sites/Public/assets/File/court/local_rules.pdf; Retrieved 2026-10-27 with the extract (this file was derived from it in the pod, not fetched itself).
- Edition: as the extract, August 2013 (revised October 25, 2025).
- Sectioned sha256 fb1d7cccf8ce212f27f39bf8be49730d2b81d5a7bd945d988776c493c37e5081 (the `path_sha256` pinned by `lit/vawd-criminal/Lit.toml`).
