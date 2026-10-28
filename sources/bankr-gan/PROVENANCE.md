
### `bankr-gan/ganb-lbr-2019-09-01.txt` — Local Rules of the U.S. Bankruptcy Court for the Northern District of Georgia (effective September 1, 2019)

- Source: https://www.ganb.uscourts.gov/sites/cst/ganb/files/documents/local_rules_effective_9-1-2019.pdf (official site; 48 pages, sha256 fb77da772c2c8547ecded0613c00882040abc9ba878bba03787b4924b2a592ad).
- Edition: Local Rules '(Effective September 1, 2019)' (cover), linked as 'Local Rules Complete PDF' from https://www.ganb.uscourts.gov/court/local-rules. Later general orders (e.g. Fifth Amended and Restated General Order No. 24-2018) were not found to amend the rule text and are not vendored. robots.txt (no rule covering these paths) and the Disclaimer page checked: no prohibition of automated access. Fetched with curl and the identifying User-Agent lit-lang-provenance/1.0 (angrybohr@proton.me).
- Retrieved 2026-10-28 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 33ac5709556111538de4d882a68c4fc77e108b28e8d2e31490565f64114ab69e. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `bankr-gan/original/ganb-lbr-2019-09-01.pdf` as served (sha256 fb77da772c2c8547ecded0613c00882040abc9ba878bba03787b4924b2a592ad); extraction sha256 33ac5709556111538de4d882a68c4fc77e108b28e8d2e31490565f64114ab69e.

### `bankr-gan/ganb-lbr-2019-09-01-sectioned.txt` — N.D. Ga. Local Rules, sectioned

- Derived in the pod (lg-u561.29) from `bankr-gan/ganb-lbr-2019-09-01.txt` (extract sha256 33ac5709556111538de4d882a68c4fc77e108b28e8d2e31490565f64114ab69e); nothing fetched. One `## Rule <n>` line is inserted before each rule's own heading line in the body (`BLR 1001-1. Title.` through `BLR 9029-1 Forms.`, including `BLR 5005-2.` (Reserved), the headings indented or following a form feed and those without a closing period; 80 sections, matching the table of contents, whose `BLR 2004.1`, `5071.3` and `5071.4` are the body's `2004-1`, `5071-3` and `5071-4`); the table of contents (extract lines 1-177) and extract line 255, a cross-reference to `BLR 9015-2.` that begins a line within BLR 1070-1(b), are unmarked. Rule text was not edited: deleting the marker lines (`sed '/^## Rule /d'`) restores the extract byte for byte.
- Edition: as the extract, the Local Rules effective September 1, 2019.
- Sectioned sha256 f4322f29f710dbedd0db79647c28ce3b6ccc0b4fd3935bd11e2f61297ead0aff (the `path_sha256` pinned by `lit/bankr-gan/Lit.toml` for source `ganb-lbr`).
