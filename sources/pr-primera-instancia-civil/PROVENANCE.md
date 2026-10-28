
### `pr-primera-instancia-civil/pr-reglas-procedimiento-civil.txt` — Reglas de Procedimiento Civil de Puerto Rico (Spanish)

- Source: https://poderjudicial.pr/Documentos/SecretariadoConf/Reglas-de-Proc-Civil.pdf (official site; 126 pages, sha256 1502b6ede6639914dbe2fafc696a09e12a4adeaef21c6c284cdd3e6493e2615a).
- Edition: Reglas de Procedimiento Civil de Puerto Rico (2009, segun enmendadas), updated by the Secretariado de la Conferencia Judicial y Notarial with statutory amendments "hasta marzo de 2023" (as stated on the document); linked as "Reglas de Procedimiento Civil de Puerto Rico" from https://poderjudicial.pr/leyes-y-reglamentos-de-la-rama-judicial/, which lists no later edition.
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 281f438594db8db68e4c3e9d11480138293e45f10f81d339345fe4b853a9795d. Raw extract, not yet sectioned into `## <ref>` form.
- Currency: recorded CURRENCY UNCONFIRMED by the PM ruling of 2026-10-27 (the poderjudicial.pr page names no edition after "hasta marzo de 2023"). Revised by the PM ruling of 2026-10-28: this compilation, with statutory amendments through March 2023, is the latest the judicial branch publishes and is the base edition; values it states are bound, each package notes that the compilation runs to March 2023, and a later statutory amendment is added as a dated layer, never an edit of the base.
- Original: `pr-primera-instancia-civil/original/pr-reglas-procedimiento-civil.pdf` as served (sha256 1502b6ede6639914dbe2fafc696a09e12a4adeaef21c6c284cdd3e6493e2615a); extraction sha256 281f438594db8db68e4c3e9d11480138293e45f10f81d339345fe4b853a9795d.

### `pr-primera-instancia-civil/pr-reglas-procedimiento-civil-sectioned.txt` — Reglas de Procedimiento Civil, sectioned

- Derived in the pod (lg-f3q8.25) from `pr-primera-instancia-civil/pr-reglas-procedimiento-civil.txt` (extract sha256 281f438594db8db68e4c3e9d11480138293e45f10f81d339345fe4b853a9795d); nothing fetched. One `## Regla <n>` line is inserted right after each rule's own body heading line `Regla <n>. <title>`: 334 markers, Regla 1 through Regla 74 (with 3.1, 4.3, 10.1, 36.3, 52.2, 68.1 and the like). The table of contents is unmarked, and so is the in-text cross-reference to Regla 45 that opens a line inside Regla 60. Rule text was not edited: deleting the marker lines (`## Regla <n>` plus its newline) restores the extract byte for byte.
- Upstream: https://poderjudicial.pr/Documentos/SecretariadoConf/Reglas-de-Proc-Civil.pdf, retrieved 2026-10-27, as recorded on the `pr-primera-instancia-civil/pr-reglas-procedimiento-civil.txt` entry this file is derived from.
- Edition: as the extract, the 2009 rules as compiled with statutory amendments through March 2023 (base edition per the PM ruling of 2026-10-28).
- Sectioned sha256 a15b24afbeccbfe00fe8fd2ab49f3e10df5218d9e1dd705c6a61a941cbc797f2 (the `path_sha256` pinned by `lit/pr-primera-instancia-civil/Lit.toml`).
