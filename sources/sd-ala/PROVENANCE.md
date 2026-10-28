
### `sd-ala/local-rules.txt` — S.D. Ala. Local Rules

- Source: https://www.alsd.uscourts.gov/sites/alsd/files/local-rules.pdf (official site; 155 pages, sha256 e3b7d7c414fe29cecc4ca7a26b0a777d34c514f03dfb37fa67cfea986e8bbf06).
- Edition: Effective August 1, 2015 (cover); listed as "Local Rules (August 1, 2015)" on https://www.alsd.uscourts.gov/court-info/local-rules-and-orders; fetched with curl (plain User-Agent), no bot refusal.
- Also the criminal local rules (lg-gddt): 'PART C: CRIMINAL RULES' part, from extract line 3819.
- Criminal forum (lg-gddt): alsd-criminal.
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 095b0d373a88b544a467b05f7ec87aa958a54d747471d0148ba05fbd995203c1. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `sd-ala/original/local-rules.pdf` as served (sha256 e3b7d7c414fe29cecc4ca7a26b0a777d34c514f03dfb37fa67cfea986e8bbf06); extraction sha256 095b0d373a88b544a467b05f7ec87aa958a54d747471d0148ba05fbd995203c1.

### `sd-ala/local-rules-civil-sectioned.txt` — S.D. Ala. Local Rules, general and civil rules sectioned

- Derived in the pod (lg-f3q8.12) from `sd-ala/local-rules.txt` (extract sha256 095b0d373a88b544a467b05f7ec87aa958a54d747471d0148ba05fbd995203c1); nothing fetched. One marker line is inserted before each rule's own heading line in Part A (`## GenLR <n>` before `General L.R. <n>.`, 20 rules, `GenLR 1` through `GenLR 87`) and Part B (`## CivLR <n>` before `Civil L.R. <n>.`, 32 rules, `CivLR 1` through `CivLR 105`), the forms GenLR 1(a) prescribes for citing them; a heading after a page's form feed keeps its form feed on the rule line. The criminal rules (Part C, from extract line 2870) and the forms (Part D) are unmarked. Rule text was not edited: deleting the marker lines (`sed '/^## \(Gen\|Civ\)LR /d'`) restores the extract byte for byte.
- Edition: as the extract, effective August 1, 2015.
- Sectioned sha256 7a4f621ebf7fdfaf8041bc877c5635252b9ec09a72b6d5e488fe8de097ef2ad8 (the `path_sha256` pinned by `lit/alsd/Lit.toml`).
