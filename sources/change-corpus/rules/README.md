# Change corpus A: FRCP 6 and FRAP 26 version history (lg-fq2.2)

Raw primary-source version history of the federal deadline-computation rules, for the
layered legal-change work (lg-fq2). Everything here is a government publication:
Supreme Court orders and the U.S. Reports back matter (govinfo.gov), the Supreme Court's
rules orders (supremecourt.gov), and Rules Committee records and congressional packages
(uscourts.gov). No treatise, law-review or commentary text is used as input; Cornell LII
was used only as a finding aid for amendment dates.

`MANIFEST.json` has one entry per artefact: `id, title, kind, rule, effective_date,
source_url, retrieved_at (UTC download time), sha256 (of the stored file), pages (pin
cites), notes`, plus `pending_case_language` (verbatim from the order), `source_sha256`
(of the full original when the stored file is an excerpt) and `text_file`/`text_sha256`
for stored PDFs. This follows `sources/PROVENANCE.md` (source URL, retrieval date,
sha256). Nothing under `sources/` top level is touched; the build embeds only top-level
`sources/*.txt`, so these subdirectories are not compiled in.

Layout:

- `us-reports/` text extracts of the Supreme Court's adopting order (kind
  adopting-order, with the Chief Justice's transmittal letter) and the amended rule text
  (kind amendment-text) from the GPO volume PDFs of the U.S. Reports back matter. The
  volume PDFs are 40-80 MB (over the 20 MB cap), so only the extraction is stored. Re-derive:
  `curl -O https://www.govinfo.gov/content/pkg/USREPORTS-<vol>/pdf/USREPORTS-<vol>.pdf`, check
  `source_sha256` in the manifest, then `pdftotext -layout -f A -l B <file> -` for the PDF
  page ranges in each entry's `pages`.
- `scotus/` Supreme Court order PDFs from supremecourt.gov (2005, 2007, 2009).
- `congress-packages/` Judicial Conference/Supreme Court packages transmitted to
  Congress or the Court (2015, 2016, 2019, 2022, 2023), with orders, rule text, Notes.
- `committee-reports/` Standing Committee and Advisory Committee reports (PDF as
  downloaded plus pdftotext). Reports over 20 MB (ST9-1997, ST9-2001) are stored as text only.
- `committee-notes/` page excerpts of the Committee Notes from the reports above.
  Older scans carry OCR noise; the PDF is authoritative.

No git LFS is configured in this repo (no `.gitattributes`); PDFs are committed plain.

## Timeline (all effective dates are December 1 unless stated)

Pending-case language is quoted in `pending_case_language` in the manifest.

FRCP 6

| Effective | What changed | Adopting order / pending-case clause |
| --- | --- | --- |
| 1938-09-16 | Original Rule 6 (6(a) computation, 6(b) enlargement, 6(e) 3 days after service by mail) | Rule 86: govern actions brought after, "and also all further proceedings in actions then pending, except to the extent that ... not be feasible or would work injustice" |
| 1948-03-19 (see gaps) | 6(b),(c) amended (order Dec. 27, 1946) | Rule 86(b) as amended, same clause |
| 1963-07-01 | 6(a): Saturday treated like Sunday/legal holiday (orders Jan. 21 and Mar. 18, 1963) | Rule 86(e): "all further proceedings in actions then pending, except ... not feasible or would work injustice" |
| 1968-07-01 | 6(b) amended (Dec. 4, 1967 order, with adoption of FRAP) | Clause stated for the Appellate Rules; civil amendments just "effective July 1, 1968" |
| 1971-07-01 | 6(a): Columbus Day added to legal holidays (Mar. 1, 1971) | none ("effective July 1, 1971") |
| 1983-08-01 | 6(b) (Apr. 28, 1983) | "govern all civil proceedings thereafter commenced and, insofar as just and practicable, in proceedings then pending" |
| 1985-08-01 | 6(a): intermediate Saturdays, Sundays, holidays excluded for periods under 11 days (was under 7) (Apr. 29, 1985) | "all proceedings in civil actions thereafter commenced and, insofar as just and practicable, all proceedings in civil actions then pending" |
| 1987-08-01 | 6(e) technical, gender-neutral (Mar. 2, 1987) | none |
| 1999-12-01 | 6(b): deletes Rule 74(a) reference (Apr. 26, 1999) | "thereafter commenced and, insofar as just and practicable, all proceedings in civil cases then pending" |
| 2001-12-01 | 6(e): 3 days added also for service under Rule 5(b)(2)(B),(C),(D) (clerk, mail, consented incl. electronic) (Apr. 23, 2001) | "thereafter commenced and, insofar as just and practicable, all proceedings then pending" |
| 2005-12-01 | 6(e): 3 days added after the period "would otherwise expire under subdivision (a)" (Apr. 25, 2005) | same |
| 2007-12-01 | Restyling (Apr. 30, 2007) | same |
| 2009-12-01 | Time-computation overhaul, "days are days": 6(a) rewritten (count every day, last-day rule, hours, inaccessible clerk, 6(a)(4) end of last day, 6(a)(5) next day, 6(a)(6) legal holiday); 6(d) 3 days; 10-day periods became 14 (Mar. 26, 2009) | "shall govern in all proceedings thereafter commenced and, insofar as just and practicable, all proceedings then pending" |
| 2016-12-01 | 6(d): service by electronic means (Rule 5(b)(2)(E)) removed from the 3-day rule; only mail (C), clerk (D), other consented means (F) (Apr. 28, 2016) | same as 2009 |
| 2023-12-01 | 6(a)(6): Juneteenth National Independence Day added (Apr. 24, 2023) | "shall govern in all proceedings thereafter commenced and, insofar as just and practicable, all proceedings then pending" |

FRAP 26 (original FRAP adopted Dec. 4, 1967, effective July 1, 1968; clause includes the
"not feasible or would work injustice" exception)

| Effective | What changed |
| --- | --- |
| 1968-07-01 | Original 26(a) computation (periods under 7 days exclude intermediate weekends/holidays), 26(b) enlargement, 26(c) 3 days after mail |
| 1971-07-01 | 26(a): Columbus Day (Mar. 1, 1971 order) |
| 1986-07-01 | 26(a): Martin Luther King Jr. Day; 26(c) technical (Mar. 10, 1986) |
| 1989-12-01 | 26(a): clerk's office inaccessible extension (Apr. 25, 1989) |
| 1991-12-01 | 26(a) amended (Apr. 30, 1991; no Note found) |
| 1996-12-01 | 26(c): 3 calendar days when delivery is later than the service date (commercial carrier) (Apr. 23, 1996) |
| 1998-12-01 | Restyle; 26(a) applies to local-rule periods (Apr. 24, 1998) |
| 2002-12-01 | 26(a)(2) aligned; 26(c): 3 calendar days also for electronic service (Apr. 29, 2002) |
| 2005-12-01 | 26(a)(4) Washington's Birthday wording (Apr. 25, 2005) |
| 2009-12-01 | Time-computation overhaul (same as FRCP 6); 26(c) "3 days" added after the period expires (Mar. 26, 2009) |
| 2016-12-01 | 26(c): electronic service treated as delivered on the service date, so no added 3 days (Apr. 28, 2016) |
| 2019-12-01 | 26(c) simplified to follow Rule 25(d) (Apr. 25, 2019) |
| 2023-12-01 | 26(a)(6): Juneteenth (Apr. 24, 2023) |

Every FRAP order above uses the form "shall govern ... thereafter commenced and, insofar as
just and practicable, all proceedings ... then pending" (1967: "all such proceedings then
pending, except ... not be feasible or would work injustice").

## Gaps (not found from a primary source)

- No Committee Note retrieved for: FRCP 6 1938, 1946, 1963, 1968, 1971, 1983 (only the
  Mar. 1982 draft `rpt-civ-1982-03`, poor OCR), 1987 and 2007 restyle (Rule 6 note); FRAP 26 1968, 1971,
  1991, 2005 (Washington's Birthday; only the Apr. 2002 Appellate Committee minutes
  located, not stored). Pre-1983 Notes are in the Judicial Conference reports of those years, which
  are not in the uscourts.gov online archive (earliest online: ST03-1983).
- Congress's House Documents (govinfo) for the transmittals were not retrieved; the
  Chief Justice's transmittal letters are inside the stored order/package files instead.
- Effective date of the 1946 civil amendments (Mar. 19, 1948) is from the rule's history line
  (secondary); Rule 86(b) in the order says Sept. 1, 1947 absent congressional action.
- supremecourt.gov hosts orders only for 2005-2009 under stable names; other years come
  from the U.S. Reports back matter (same orders, GPO). The 2016 civil order PDF in the
  congressional package has a blank date line; the dated order is ur578-civ-2016-order.
- Pre-1938-era text, the 1993 and 2003 FRCP 6 non-amendments, and local rules/standing
  orders are out of scope for this bead.
