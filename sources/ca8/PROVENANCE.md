# Eighth Circuit Court of Appeals source provenance

- `ca8-local-rules.txt`: official Eighth Circuit Local Rules PDF, dated June 17, 2024: https://www.ca8.uscourts.gov/sites/ca8/files/Eighth%20Circuit%20Local%20Rules%206.17.24.pdf. Retrieved 2026-10-27; extracted with `pdftotext -layout`. PDF SHA-256 `26cfe150aa5813ffdc7f5c55ef5a2604c02ecc76526d5ee21669806b937a41ac`; text SHA-256 `462e6deddebce2f420f7628c80ce8c617971bdecbfebb4be817ca99772d375b8`.
- `ca8-iops.txt`: official Eighth Circuit Internal Operating Procedures, revised February 26, 2024: https://ecf.ca8.uscourts.gov/newrules/coa/iops.pdf. Retrieved 2026-10-27; extracted with `pdftotext -layout`. PDF SHA-256 `41097628db55700c9774760cb4a7334e427d086d69c429a1f4ec3edd1c9e0ff4`; text SHA-256 `358fd9ee631e7ec9423c8ff94b2f7f2c6ce82d1a48e2a567afa2c487883b89b8`.
- `../frap/frap.txt`: official Federal Rules of Appellate Procedure, as amended to December 1, 2025, House Judiciary Committee Print No. 2, 119th Congress: https://www.uscourts.gov/rules-policies/current-rules-practice-procedure/federal-rules-appellate-procedure. Retrieved 2026-10-27; OCR extraction. Text SHA-256 `1d5a3ff17813cc8cb47c3cdab65adf8db170a8a71b3aae2b187e151975415017`. Numeric periods used here (40, 14, and 7 days) were cross-checked against the non-OCR 2024 FRAP edition bound in `lit/frap`.

### `ca8/ca8-local-rules.txt` — Local Rules of the United States Court of Appeals for the Eighth Circuit

- Source: https://www.ca8.uscourts.gov/sites/ca8/files/Eighth%20Circuit%20Local%20Rules%206.17.24.pdf (official site; 39 pages, sha256 26cfe150aa5813ffdc7f5c55ef5a2604c02ecc76526d5ee21669806b937a41ac).
- Edition: dated June 17, 2024 on its first page; listed as "Local Rules of the Eighth Circuit, June 2024" on https://www.ca8.uscourts.gov/rules-procedures.
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 462e6deddebce2f420f7628c80ce8c617971bdecbfebb4be817ca99772d375b8. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `ca8/original/ca8-local-rules.pdf` as served (sha256 26cfe150aa5813ffdc7f5c55ef5a2604c02ecc76526d5ee21669806b937a41ac); extraction sha256 462e6deddebce2f420f7628c80ce8c617971bdecbfebb4be817ca99772d375b8.

### `ca8/ca8-iops.txt` — Eighth Circuit Internal Operating Procedures

- Source: https://ecf.ca8.uscourts.gov/newrules/coa/iops.pdf (official site; 42 pages, sha256 41097628db55700c9774760cb4a7334e427d086d69c429a1f4ec3edd1c9e0ff4).
- Edition: title page: "Internal Operating Procedures, Revised February 26, 2024"; the IOPs linked from https://www.ca8.uscourts.gov/rules-procedures.
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 358fd9ee631e7ec9423c8ff94b2f7f2c6ce82d1a48e2a567afa2c487883b89b8. Raw extract with no `## <ref>` headings; the cite corpus splits it on its own outline (`IOP IV`, `IOP IV.D`, `IOP IV.F`, `IOP Appendix A`), so `8th Cir. IOP IV`, `8th Cir. IOP IV.D`, `8th Cir. IOP IV.F`, and `8th Cir. IOP Appendix A` resolve.
- Original: `ca8/original/ca8-iops.pdf` as served (sha256 41097628db55700c9774760cb4a7334e427d086d69c429a1f4ec3edd1c9e0ff4); extraction sha256 358fd9ee631e7ec9423c8ff94b2f7f2c6ce82d1a48e2a567afa2c487883b89b8.
