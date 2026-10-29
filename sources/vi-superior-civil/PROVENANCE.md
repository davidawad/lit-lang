
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
- Layer: an amendment layer over `vi-superior-civil/vircp.txt`; apply by its effective date. It is also the 2023-09-01 layer for the V.I. Rules of Criminal Procedure (base `vi-criminal-trial-court/vi-rules-of-criminal-procedure-2019.txt`), the V.I. Rules of Evidence (base `vi-superior-civil/vi-rules-of-evidence.txt`) and the V.I. Electronic Filing Rules (base `vi-superior-civil/vi-e-file-rules-2020-08-31.txt`) (lg-u8ho): apply this order over each by the same date rather than vendoring the order again.
- Retrieved 2026-10-28 in the pod (lg-w6fv) with curl; extracted with `pdftotext -layout` (poppler 25.03), unedited; text layer as served.
- Extract sha256 7ef874bdf792d49f88a6396d517b4475f1c8c60173ca1d578c21cdec4db93028. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `vi-superior-civil/original/vircp-amendments-2023-09-01.pdf` as served (sha256 1ce71220627c01a68d2129788820b27781eb5a656c582a78616d119bd946466e); extraction sha256 7ef874bdf792d49f88a6396d517b4475f1c8c60173ca1d578c21cdec4db93028.

### `vi-superior-civil/vircp-sectioned.txt` — Virgin Islands Rules of Civil Procedure, sectioned

- Derived in the pod (lg-f3q8.76) from `vi-superior-civil/vircp.txt` (extract sha256 208c0e6784096abb781c0d77c80ba9a51e11d0b848b41b64b1fa1779f467d304); nothing fetched. One marker line `## Rule <n>` is inserted before the first heading line `Rule <n>. <Title>` of each rule (113 rules, Rule 1 through Rule 91, including hyphenated and decimal rules such as 6-1 and 7.1; a heading may begin with the form feed of a page break, and the marker goes before that line). Every other line is the extract unchanged, so deleting the `## Rule` lines (`sed '/^## Rule /d'`) gives back the extract byte for byte.
- Upstream: https://www.vicourts.org/common/pages/GetFile.ashx?key=wdHyAMwu, retrieved 2026-10-27, as recorded on the `vi-superior-civil/vircp.txt` entry this file is derived from.
- Edition: as the extract; CURRENCY UNCONFIRMED stands. `lit/vi-superior-civil` cites it for structure only and takes no value from it.
- Sectioned sha256 874ffdca30fc2290a9d48eb34af507e5e1f8ff3d2609d087ad829a3dd17d1bdd (the `path_sha256` pinned by `lit/vi-superior-civil/Lit.toml`).

### `vi-superior-civil/vircp-amendments-2023-09-01-sectioned.txt` — SCT-Prom-2023-0003, sectioned

- Derived in the pod (lg-f3q8.76) from `vi-superior-civil/vircp-amendments-2023-09-01.txt` (extract sha256 7ef874bdf792d49f88a6396d517b4475f1c8c60173ca1d578c21cdec4db93028); nothing fetched. One marker line is inserted before each "ORDERED that Rule ... SHALL BE AMENDED" paragraph: `## Rule <n>` for the Rules of Civil Procedure (4, 6, 7.1, 15, 30, 90, 47-1) and `## V.I. R. Crim. P. <n>`, `## V.I. R. Evid. <n>`, `## V.I. R. App. P. 16` and `## V.I. E-Filing R. 7` for the other rule sets, which only bound the civil sections. Every other line is the extract unchanged, so deleting the `## ` marker lines (`sed '/^## /d'`) gives back the extract byte for byte.
- Upstream: https://supreme.vicourts.org/UserFiles/Servers/Server_12810860/File/Administrative%20Services/PROMULGATION%20ORDER%202023-0003.pdf, retrieved 2026-10-28, as recorded on the `vi-superior-civil/vircp-amendments-2023-09-01.txt` entry this file is derived from.
- Edition: as the extract (entered July 14, 2023; effective September 1, 2023).
- Sectioned sha256 ead76ebf64f5880f2962f7f03638a28a89581799f85d029d650594efae8e1327 (the `path_sha256` pinned by `lit/vi-superior-civil/Lit.toml`).

### `vi-superior-civil/vi-rules-of-evidence.txt` — Virgin Islands Rules of Evidence

- Source: https://www.vicourts.org/common/pages/DisplayFile.aspx?itemId=16185792 (official site; served `content-disposition: inline;filename="VI Rules of Evidence.pdf"`; 40 pages, sha256 79b1950a3080bc741c54d0a5d61628a7e737a4982c71e5f1d4b381f909fe13cc). Found by web search of vicourts.org; the item sits beside the V.I. Rules of Criminal Procedure (itemId=16185774, the same PDF byte for byte as `vi-criminal-trial-court/original/vi-rules-of-criminal-procedure-2019.pdf`) and the VIRAP (itemId=16185765) in the 2019-11 upload set behind https://www.vicourts.org/rules.
- Edition: Virgin Islands Rules of Evidence, no edition line on the document (PDF created May 21, 2019, modified November 6, 2019). Rule history lines read "Effective March 31, 2017 (Promulgation Order 2017-002)" (one, "Effective April 4, 2017 (Promulgation Order 2017-002)"); no later amendment line appears.
- Currency: SCT-Prom-2023-0003 (`vi-superior-civil/vircp-amendments-2023-09-01.txt`, effective September 1, 2023) amends Rules 106, 606(b)(2), 615 and 702; apply it as the dated layer over this base. Other promulgation orders between 2017 and 2025 are not checked for Evidence Rule amendments: CURRENCY UNCONFIRMED beyond that layer.
- Retrieved 2026-10-29 in the pod (lg-u8ho) with curl (User-Agent lit-lang-provenance/1.0 (angrybohr@proton.me)); extracted with `pdftotext -layout` (poppler 25.03), unedited; text layer as served.
- Extract sha256 ebd3e848aa919387d3cfc50125c42839ad6deba32b48f3dfe3c82df7f631f6fd. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `vi-superior-civil/original/vi-rules-of-evidence.pdf` as served (sha256 79b1950a3080bc741c54d0a5d61628a7e737a4982c71e5f1d4b381f909fe13cc); extraction sha256 ebd3e848aa919387d3cfc50125c42839ad6deba32b48f3dfe3c82df7f631f6fd.

### `vi-superior-civil/vi-e-file-rules-2020-08-31.txt` — Interim Virgin Islands Electronic Filing Rules (V.I. E-FILE R.)

- Source: https://www.vicourts.org/e-_services/e-_filing/court_rules_governing_electronic_filing (the "Court Rules Governing Electronic Filing" page, linked from the E-Services menu of https://www.vicourts.org/rules), which redirects to https://cdnsm5-hosted.civiclive.com/UserFiles/Servers/Server_9784218/File/eServices/FINAL%20Interim%20VI%20EFiling%20Rules%20(002).pdf (official site; 14 pages, sha256 b4f36023047af95612e0af2cf94e4fae00a21b2b8ca20a0c1febb83f9715871f).
- Edition: "INTERIM Virgin Islands Electronic Filing Rules", Rules 1-14, no edition line (PDF created August 26, 2020). Adopted by S. Ct. Prom. No. 2020-0004 (`vi-supreme-civil/virap-amendments-2020-08-31.txt`), effective August 31, 2020, for all Supreme Court proceedings and the non-criminal Superior Court proceedings; the PDF postdates the August 14, 2020 erratum order that replaced the order's Exhibit 1, so it is taken as the rules as corrected, but it is not compared with the erratum's exhibit (not vendored). Rule 7(a)(6) here is the pre-2023 text ("Filers are strongly encouraged to not submit proposed orders in PDF format ...").
- Currency: SCT-Prom-2023-0003 (`vi-superior-civil/vircp-amendments-2023-09-01.txt`, effective September 1, 2023) amends Rule 7(a)(6); apply it as the dated layer over this base. Other promulgation orders after 2020 are not checked for E-File Rule amendments: CURRENCY UNCONFIRMED beyond that layer.
- Retrieved 2026-10-29 in the pod (lg-u8ho) with curl (User-Agent lit-lang-provenance/1.0 (angrybohr@proton.me)); extracted with `pdftotext -layout` (poppler 25.03), unedited; text layer as served.
- Extract sha256 7c8305d14aa67eb9a3112268a2679da8b6f5c6b43b162a28c886d3beb8d52330. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `vi-superior-civil/original/vi-e-file-rules-2020-08-31.pdf` as served (sha256 b4f36023047af95612e0af2cf94e4fae00a21b2b8ca20a0c1febb83f9715871f); extraction sha256 7c8305d14aa67eb9a3112268a2679da8b6f5c6b43b162a28c886d3beb8d52330.
