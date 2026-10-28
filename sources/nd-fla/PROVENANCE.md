
### `nd-fla/local-rules.txt` — N.D. Fla. Local Rules

- Source: https://flnd.uscourts.gov/sites/flnd/files/local_rules/local_rules_0.pdf (official site; 82 pages, sha256 735431487ba914dd394d074f8e1c86d56a4842d11fef865e237713043d045831).
- Edition: Effective November 24, 2015 (cover); the file linked as local_rules.pdf from https://flnd.uscourts.gov/local-rules (the page blurb still says "Revised 1995, amended ... July 2005" but the linked file is the 2015 edition); fetched with curl (plain User-Agent), no bot refusal.
- Also the criminal local rules (lg-gddt): no separate criminal part; criminal provisions include Rule 26.2 Discovery in Criminal Cases (extract line 1071).
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 5c98c49bc26451f29a3a3138ebbf8577c97deffc769e27f24ce098a03459000a. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `nd-fla/original/local-rules.pdf` as served (sha256 735431487ba914dd394d074f8e1c86d56a4842d11fef865e237713043d045831); extraction sha256 5c98c49bc26451f29a3a3138ebbf8577c97deffc769e27f24ce098a03459000a.

### `nd-fla/local-rules-sectioned.txt` — N.D. Fla. Local Rules, sectioned

- Derived in the pod (lg-f3q8.14) from `nd-fla/local-rules.txt` (extract sha256 5c98c49bc26451f29a3a3138ebbf8577c97deffc769e27f24ce098a03459000a); nothing fetched. One `## Rule <n>` line is inserted before each rule's own heading line (`Rule 1.1 Scope of the Rules; Citation` through `Rule 88.3 Patent Rules`, 43 rules). Rule text was not edited: deleting the marker lines (`sed '/^## Rule /d'`) restores the extract byte for byte.
- Edition: as the extract, effective November 24, 2015.
- Sectioned sha256 bac984b7c46a868b36137e9654224e443fde8b44201c1f31f7bcc7cfed3e52bc (the `path_sha256` pinned by `lit/flnd/Lit.toml`).
