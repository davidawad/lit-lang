
### `vi-superior-civil/vircp.txt` — Virgin Islands Rules of Civil Procedure

- Source: https://www.vicourts.org/common/pages/GetFile.ashx?key=wdHyAMwu (official site; 131 pages, sha256 cc4692594bd05e1d1f6128a9d97355b4162b7a3b9930e606dcea95352fb366cd).
- Edition: Virgin Islands Rules of Civil Procedure, no edition line on the document (PDF created Aug. 8, 2019), as linked "VI Rules of Civil Procedure (001)" in folder "1. Virgin Islands Rules of Civil Procedure" on https://www.vicourts.org/rules (retrieved 2026-10-27). The page states no edition or amendment date; the edition is taken from the PDF itself. Later amendments, if any, are not listed on the page.
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 208c0e6784096abb781c0d77c80ba9a51e11d0b848b41b64b1fa1779f467d304. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `vi-superior-civil/original/vircp.pdf` as served (sha256 cc4692594bd05e1d1f6128a9d97355b4162b7a3b9930e606dcea95352fb366cd); extraction sha256 208c0e6784096abb781c0d77c80ba9a51e11d0b848b41b64b1fa1779f467d304.

### `vi-superior-civil/superior-court-rules-2019-06-01.txt` — Rules Governing the Superior Court of the Virgin Islands

- Source: https://www.vicourts.org/common/pages/GetFile.ashx?key=lmL0AJ2d (official site; 35 pages, sha256 f82cf8486b3c51d2a8dc17c092a5f1b5575c31d65498e9286ad644a19ebf552d).
- Edition: Rules Governing the Superior Court of the Virgin Islands, "amended June 1, 2019" on the document, as linked "VI Superior Court Rules" in folder "10. Virgin Islands Superior Court Rules" on https://www.vicourts.org/rules (retrieved 2026-10-27). The page states no edition or amendment date; the edition is taken from the PDF itself. Later amendments, if any, are not listed on the page.
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 0343a4b4b46e55b7f17e336e8d6d473c9ee5cc0dc5689659852f76596603537d. Raw extract, not yet sectioned into `## <ref>` form.

- CURRENCY UNCONFIRMED (PM ruling 2026-10-27) for `vi-superior-civil/superior-court-rules-2019-06-01.txt` `vi-superior-civil/vircp.txt` : vicourts.org/rules states no edition or amendment date; the PDFs date from 2019 and later amendments are not shown. Treat every value taken from these as a hole until a current edition is confirmed.
- Original: `vi-superior-civil/original/superior-court-rules-2019-06-01.pdf` as served (sha256 f82cf8486b3c51d2a8dc17c092a5f1b5575c31d65498e9286ad644a19ebf552d); extraction sha256 0343a4b4b46e55b7f17e336e8d6d473c9ee5cc0dc5689659852f76596603537d.

### `vi-superior-civil/vircp-amendments-2023-09-01.txt` — Supreme Court of the Virgin Islands Promulgation Order SCT-Prom-2023-0003 (amendments to the V.I. Rules of Civil Procedure, Criminal Procedure, Evidence, Appellate Procedure and Electronic Filing Rules)

- Source: https://supreme.vicourts.org/UserFiles/Servers/Server_12810860/File/Administrative%20Services/PROMULGATION%20ORDER%202023-0003.pdf (official site; served by redirect from cdnsm5-hosted.civiclive.com; 10 pages, sha256 1ce71220627c01a68d2129788820b27781eb5a656c582a78616d119bd946466e). The same order, byte for byte, as `vi-supreme-civil/virap-amendments-2023-09-01.txt` (retrieved there by lg-w6fv); vendored here too (lg-r1zq) because most of it amends the trial-court rules.
- Edition: Order of the Court dated and entered July 14, 2023 (Promulgation No. 2023-003), effective September 1, 2023 "unless modified by further Order of this Court based on public comment"; no later order modifying it is listed. Listed as "SCT-PROM-2023-0003" in the 2023 Archive at https://supreme.vicourts.org/offices_of_the_court/office_of_the_clerk/promulgation_and_administrative_orders/2023_archive.
- Scope: amends V.I. Rules of Civil Procedure 4, 6(a), 7.1, 15(a)(1), 30(b)(5)(B), 47-1(b) and 90; V.I. Rules of Criminal Procedure 11 and 45(a)(4)(A); V.I. Rules of Evidence 106, 606(b)(2), 615 and 702; V.I. Electronic Filing Rule 7(a)(6); and VIRAP 16(b) (the VIRAP part is applied from the `vi-supreme-civil` copy).
- Layer: an amendment layer over `vi-superior-civil/vircp.txt`; apply by its effective date. It is also the 2023-09-01 layer for the V.I. Rules of Criminal Procedure, V.I. Rules of Evidence and V.I. Electronic Filing Rules: no base text of those rules is vendored yet; when one is, apply this order over it by the same date rather than vendoring the order again.
- Retrieved 2026-10-28 in the pod (lg-w6fv) with curl; extracted with `pdftotext -layout` (poppler 25.03), unedited; text layer as served.
- Extract sha256 7ef874bdf792d49f88a6396d517b4475f1c8c60173ca1d578c21cdec4db93028. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `vi-superior-civil/original/vircp-amendments-2023-09-01.pdf` as served (sha256 1ce71220627c01a68d2129788820b27781eb5a656c582a78616d119bd946466e); extraction sha256 7ef874bdf792d49f88a6396d517b4475f1c8c60173ca1d578c21cdec4db93028.
