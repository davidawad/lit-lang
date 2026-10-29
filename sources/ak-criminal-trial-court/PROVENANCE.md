
### `ak-criminal-trial-court/ak-rules-of-criminal-procedure.txt` — Alaska Rules of Court: Rules of Criminal Procedure (80 pp; includes Rule 45, speedy trial)

- Source: https://courts.alaska.gov/rules/docs/crpro.pdf (official site; 80 pages, sha256 a1473c8e4e827d5f9007124a814bada560defaf75d5db434f9f2e5e711f282e3).
- Edition: Alaska Rules of Criminal Procedure, the 'Criminal Procedure' PDF on https://courts.alaska.gov/rules/index.htm; the page and PDF carry no edition date (PDF metadata creation date 2026-10-09; latest amendment notes cite Supreme Court Orders), CURRENCY UNCONFIRMED. courts.alaska.gov robots.txt read; no Disallow covering this path; Legal Notices page (https://courts.alaska.gov/notices/index.htm) read: no automated-access prohibition found; fetched on David's explicit ruling 2026-10-28 despite robots/terms; tool: curl with the identifying User-Agent lit-lang-provenance/1.0 (angrybohr@proton.me).
- Retrieved 2026-10-28 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 e1ad8741ed5f24bc3a7ea6bb63997594c86926b3471c8a9498a07f048bf4a3bc. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `ak-criminal-trial-court/original/ak-rules-of-criminal-procedure.pdf` as served (sha256 a1473c8e4e827d5f9007124a814bada560defaf75d5db434f9f2e5e711f282e3); extraction sha256 e1ad8741ed5f24bc3a7ea6bb63997594c86926b3471c8a9498a07f048bf4a3bc.

- Note (lg-gddt): Criminal appeals are governed by the Alaska Rules of Appellate Procedure, covered by sources/ak-supreme/ (alaska-appellate-procedure.txt).

### `ak-criminal-trial-court/ak-rules-of-criminal-procedure-sectioned.txt` — Alaska Rules of Criminal Procedure, sectioned

- Derived in the pod (lg-gddt.22) from `ak-criminal-trial-court/ak-rules-of-criminal-procedure.txt` (extract sha256 e1ad8741ed5f24bc3a7ea6bb63997594c86926b3471c8a9498a07f048bf4a3bc); nothing fetched. The extract is set in two columns, so one marker line `## Rule <n>` is inserted before each extract line that holds a rule's own body heading `Rule <n>.` followed by its capitalized title, in either column (from extract line 560, after the table of contents; 73 rules, the repeated Rule 38 heading marked once). A marker opens the line where its heading appears; the column layout interleaves text from neighboring rules, so the markers locate a rule, they do not separate its text. Every other line is the extract unchanged, so deleting the marker lines gives back the extract byte for byte.
- Source: as the extract `ak-criminal-trial-court/ak-rules-of-criminal-procedure.txt`, https://courts.alaska.gov/rules/docs/crpro.pdf; Retrieved 2026-10-28 with the extract (this file was derived from it in the pod, not fetched itself).
- Edition: as the extract; CURRENCY UNCONFIRMED stands. `lit/ak-criminal-trial-court` cites it for structure only and takes no value from it.
- Sectioned sha256 618490e856ceac010c49aeab7733c6ba8aca98fae7786848e2b9747d0f93ef43 (the `path_sha256` pinned by `lit/ak-criminal-trial-court/Lit.toml`).
