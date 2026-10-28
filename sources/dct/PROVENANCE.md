
### `dct/local-rules.txt` — District of Connecticut Local Rules of Civil Procedure, Magistrate Judges, Criminal Procedure

- Source: https://www.ctd.uscourts.gov/sites/default/files/Current-Revised-Local-Rules-10.13.26.pdf (official site; 174 pages, sha256 7276a8de62373432011f35807dca01ce7ef5a7879d87a7e7a6f3d8f90d7d6264).
- Edition: file Current-Revised-Local-Rules-10.13.26.pdf; cover reads "Amended December 1, 2009*" with per-rule amendment dates (latest seen: Amended September 25, 2026); linked as the current Local Rules from https://www.ctd.uscourts.gov/court-info/local-rules-and-orders; civil, magistrate judge and criminal rules.
- Also the criminal local rules (lg-gddt): 'LOCAL RULES OF CRIMINAL PROCEDURE' part, from extract line 4639.
- Criminal forum (lg-gddt): ctd-criminal.
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 7cc22774663a1d8b1523b4f7668dc621c205d902f0d84e475ef0607ef28d0368. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `dct/original/local-rules.pdf` as served (sha256 7276a8de62373432011f35807dca01ce7ef5a7879d87a7e7a6f3d8f90d7d6264); extraction sha256 7cc22774663a1d8b1523b4f7668dc621c205d902f0d84e475ef0607ef28d0368.

### `dct/local-rules-sectioned.txt` — D. Conn. Local Rules (civil, magistrate judge and criminal), sectioned

- Derived in the pod (lg-f3q8.49) from `dct/local-rules.txt` (extract sha256 7cc22774663a1d8b1523b4f7668dc621c205d902f0d84e475ef0607ef28d0368); nothing fetched. One `## D. Conn. L. Civ. R. <n>` line is inserted right before each numbered civil rule's own body heading line `RULE <n>` (each led by a form feed): 51 markers, D. Conn. L. Civ. R. 1 through D. Conn. L. Civ. R. 83.13; and one `## D. Conn. L. Mag. R. <n>` line before each Local Rule for Magistrate Judges (72.1, 72.2, 73): 3 markers. The table of contents, the reserved rule ranges (`RULE 13 – RULE 15` and the like), the appendix and the Local Rules of Criminal Procedure are unmarked. Rule text was not edited: deleting the marker lines (`sed '/^## D\. Conn\. L\. \(Civ\|Mag\)\. R\. /d'`) restores the extract byte for byte.
- Edition: as the extract; the latest per-rule amendment date it states is September 25, 2026.
- Sectioned sha256 4279c6f3f7e4ec4ae76e34a016745c3caa0fb0d7794367afcde6345fed708a7f (the `path_sha256` pinned by `lit/ctd/Lit.toml`).
