
### `sdd/civil-local-rules.txt` — D.S.D. Civil Local Rules of Practice

- Source: https://www.sdd.uscourts.gov/sites/sdd/files/Civil%20Local%20Rules%20-%20final%2C%20approved.pdf (official site; 34 pages, sha256 2db6120e6edcc37e1de5b70ed802e39611a00ef1c4e14b488cfc263587cf257d).
- Edition: document dated 2/28/2025 (cover); linked as "Civil Local Rules (Complete Set)" from https://www.sdd.uscourts.gov/LocalRulesCivil; fetched with curl (plain User-Agent), no bot refusal.
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 600322bebb8f15254d56f37d92bcfcbef81507abe8ebd4396c7fde305d6675b7. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `sdd/original/civil-local-rules.pdf` as served (sha256 2db6120e6edcc37e1de5b70ed802e39611a00ef1c4e14b488cfc263587cf257d); extraction sha256 600322bebb8f15254d56f37d92bcfcbef81507abe8ebd4396c7fde305d6675b7.

### `sdd/civil-local-rules-sectioned.txt` — D.S.D. Civil Local Rules of Practice, sectioned

- Derived in the pod (lg-f3q8.11) from `sdd/civil-local-rules.txt` (extract sha256 600322bebb8f15254d56f37d92bcfcbef81507abe8ebd4396c7fde305d6675b7); nothing fetched. One `## LR <n>` line is inserted before each rule's own heading line (`LR 1.1 SCOPE OF THE RULES` through `LR 83.12 FORMER LAW CLERKS`, 40 rules); every other line is the extract unchanged, so deleting the `## LR` lines gives back the extract plus a final newline.
- Source: as the extract `sdd/civil-local-rules.txt`, https://www.sdd.uscourts.gov/sites/sdd/files/Civil%20Local%20Rules%20-%20final%2C%20approved.pdf, retrieved 2026-10-27; the sectioning was done in the pod from the vendored extract.
- Edition: as the extract, document dated 2/28/2025.
- Sectioned sha256 3bfff7fb9b3f9e19f786c863e5971119bb06eeab9ffa41f569bfbd556a186575.

### `sdd/sdd-criminal-local-rules-complete-set.txt` — D.S.D. Criminal Local Rules of Practice (complete set, undated)

- Source: https://www.sdd.uscourts.gov/sites/sdd/files/Criminal%20Local%20Rules%20-%20final%2C%20approved.pdf (official site; 36 pages, sha256 c14f37a0684b4e6add374ae6a70be467f0919fe65b7caae848acdfcbf3a3c25e).
- Edition: Criminal Local Rules of Practice, 'Complete Set' (final, approved) linked at https://www.sdd.uscourts.gov/LocalRulesCriminal; CURRENCY UNCONFIRMED: neither the PDF nor the page states an effective date. robots.txt of www.sdd.uscourts.gov read (Crawl-delay 10 honored where stated; no Disallow covers the fetched path); the judiciary-wide Privacy & Security Policy (https://www.uscourts.gov/privacy-security-policy) read: no prohibition of automated access. Fetched with curl and the identifying UA lit-lang-provenance/1.0, at most 1 request per 10 s.
- Criminal forum (lg-gddt): sdd-criminal.
- Retrieved 2026-10-28 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 b98938d2f72ee568905e6c9c31c6432d19cfc3eab11e58cbd75927faef635ae8. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `sdd/original/sdd-criminal-local-rules-complete-set.pdf` as served (sha256 c14f37a0684b4e6add374ae6a70be467f0919fe65b7caae848acdfcbf3a3c25e); extraction sha256 b98938d2f72ee568905e6c9c31c6432d19cfc3eab11e58cbd75927faef635ae8.
