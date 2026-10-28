
### `fisc/fisc-rules-of-procedure.txt` — FISC Rules of Procedure (2010)

- Source: https://www.fisc.uscourts.gov/sites/default/files/FISC%20Rules%20of%20Procedure.pdf (official site; 18 pages, sha256 f6cccb36e8711d7d5de9989ae1e3757f68c139d525117b9c18bac716c9abec15).
- Edition: Rules of Procedure, effective November 1, 2010 (as printed on the cover; the page https://www.fisc.uscourts.gov/rules-procedure states the current rules took effect November 1, 2010). Shared with the Foreign Intelligence Surveillance Court of Review package (fiscr). Fetched with curl..
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 da4ecc0dce0060279f0a45ca75d4270e7737bfc17a291ee5926d59d174cc6165. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `fisc/original/fisc-rules-of-procedure.pdf` as served (sha256 f6cccb36e8711d7d5de9989ae1e3757f68c139d525117b9c18bac716c9abec15); extraction sha256 da4ecc0dce0060279f0a45ca75d4270e7737bfc17a291ee5926d59d174cc6165.

### `fisc/fisc-rules-of-procedure-sectioned.txt` — FISC Rules of Procedure (2010), sectioned

- Derived in the pod (lg-f3q8.34) from `fisc/fisc-rules-of-procedure.txt` (extract sha256 da4ecc0dce0060279f0a45ca75d4270e7737bfc17a291ee5926d59d174cc6165); nothing fetched. One `## Rule <n>` line is inserted before each line opening a rule (`Rule 1. Scope of Rules.` through `Rule 63. Practice Before Court.`, 63 rules), so each section holds its rule's heading line and text. Rule text was not edited: deleting the marker lines (`sed '/^## Rule /d'`) restores the extract byte for byte.
- Edition: as the extract, effective November 1, 2010.
- Sectioned sha256 52659378ae78d618d899e20d75b920fe1b503d69f6bab03f2c5588c3aa7e950e (the `path_sha256` pinned by `lit/fisc/Lit.toml` and `lit/fiscr/Lit.toml`).
