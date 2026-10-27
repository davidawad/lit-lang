
### `sdd/civil-local-rules.txt` — D.S.D. Civil Local Rules of Practice

- Source: https://www.sdd.uscourts.gov/sites/sdd/files/Civil%20Local%20Rules%20-%20final%2C%20approved.pdf (official site; 34 pages, sha256 2db6120e6edcc37e1de5b70ed802e39611a00ef1c4e14b488cfc263587cf257d).
- Edition: document dated 2/28/2025 (cover); linked as "Civil Local Rules (Complete Set)" from https://www.sdd.uscourts.gov/LocalRulesCivil; fetched with curl (plain User-Agent), no bot refusal.
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 600322bebb8f15254d56f37d92bcfcbef81507abe8ebd4396c7fde305d6675b7. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `sdd/original/civil-local-rules.pdf` as served (sha256 2db6120e6edcc37e1de5b70ed802e39611a00ef1c4e14b488cfc263587cf257d); extraction sha256 600322bebb8f15254d56f37d92bcfcbef81507abe8ebd4396c7fde305d6675b7.

### `sdd/civil-local-rules-sectioned.txt` — D.S.D. Civil Local Rules of Practice, sectioned

- Derived in the pod (lg-f3q8.11) from `sdd/civil-local-rules.txt` (extract sha256 600322bebb8f15254d56f37d92bcfcbef81507abe8ebd4396c7fde305d6675b7); nothing fetched. One `## LR <n>` line is inserted before each rule's own heading line (`LR 1.1 SCOPE OF THE RULES` through `LR 83.12 FORMER LAW CLERKS`, 40 rules); every other line is the extract unchanged, so deleting the `## LR` lines gives back the extract plus a final newline.
- Edition: as the extract, document dated 2/28/2025.
- Sectioned sha256 3bfff7fb9b3f9e19f786c863e5971119bb06eeab9ffa41f569bfbd556a186575.
