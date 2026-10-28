### `fiscr/fiscr-rules-of-procedure.txt` — FISCR Rules of Procedure (May 2018)

- Source: https://www.fisc.uscourts.gov/sites/default/files/Rules%20of%20Procedure.pdf (official site, linked as "Rules of Procedure" from https://www.fisc.uscourts.gov/FISCR; 26 pages, Rule 15(d) as amended by the order of May 3, 2018 below, sha256 2e96f100311f861eae9349443a80ebeec1c35b23f8f6702166e33f6889f24da8). PDF metadata: created May 3, 2018.
- Edition: Rules of Procedure of the United States Foreign Intelligence Surveillance Court of Review, Rules 1-21, every page footed "May-18". The rules state no calendar effective date: FISCR R. 21 makes them effective six months after transmittal to Congress under 50 U.S.C. § 1803(g), and the transmittal date is not printed in the document.
- Retrieved 2026-10-28 in the pod (lg-7jt1); fetched with curl; extracted with `pdftotext -layout` (poppler 25.03.0), unedited.
- Extract sha256 6385086a0ad5d2e06cff406ce8090a1c630e0d400ef30f0c8826f8a56c829998. Raw extract; the sectioned form is below.
- Original: `fiscr/original/fiscr-rules-of-procedure.pdf` as served (sha256 2e96f100311f861eae9349443a80ebeec1c35b23f8f6702166e33f6889f24da8); extraction sha256 6385086a0ad5d2e06cff406ce8090a1c630e0d400ef30f0c8826f8a56c829998.

### `fiscr/fiscr-rules-of-procedure-sectioned.txt` — FISCR Rules of Procedure (May 2018), sectioned

- Derived in the pod (lg-7jt1), retrieved 2026-10-28 as the extract above (https://www.fisc.uscourts.gov/sites/default/files/Rules%20of%20Procedure.pdf); nothing further fetched. Method: one `## Rule <n>` line is inserted before each line opening a rule (`Rule 1. Scope of Rules.` through `Rule 21. Effective Date.`, 21 rules), so each section holds its rule's heading line and text. Rule text was not edited: deleting the marker lines (`sed '/^## Rule /d'`) restores the extract byte for byte.
- Edition: as the extract (May 2018).
- Sectioned sha256 d933dd55c95ab21917ee8bc38192ad8e764b02867fac8acc4cf5318a8fb63bbf (the `path_sha256` pinned by `lit/fiscr/Lit.toml`).

### `fiscr/fiscr-order-amending-rules-2018-05-03.txt` — FISCR Order Amending the Rules of Procedure (Rule 15(d)), May 3, 2018

- Source: https://fisc.uscourts.gov/sites/default/files/FISCR%2018%2002%20180503.pdf (official site, posted May 3, 2018 at https://fisc.uscourts.gov/public-filings/order-amending-rules-procedure-court; FISCR docket 18-02; 2 pages, a scan, sha256 64b6e8033bf691648e6ea77d0377de6095b8f2e5e30294625d876f081eaad4bf).
- Edition: the order of May 3, 2018 (Bryson, Cabranes, Tallman) adopting amended Rule 15(d) (brief of an amicus curiae) after the February 2018 publication of the proposal, and directing the Clerk to transmit it to the recipients designated under 50 U.S.C. § 1803(g). The amended Rule 15(d) is the text printed in `fiscr/fiscr-rules-of-procedure.txt` (the "May-18" edition); this order dates it. It is a rule-amendment record: no package binds it.
- Retrieved 2026-10-28 in the pod (lg-7jt1); fetched with curl; extracted with `pdftotext -layout` (poppler 25.03.0) from the scan's text layer, unedited (the clerk's filing stamp is OCR noise in the first lines).
- Extract sha256 24714546574979b1514f73c6821dd4785370b5144c17c7231c80f5fd1c7d0e1f.
- Original: `fiscr/original/fiscr-order-amending-rules-2018-05-03.pdf` as served (sha256 64b6e8033bf691648e6ea77d0377de6095b8f2e5e30294625d876f081eaad4bf); extraction sha256 24714546574979b1514f73c6821dd4785370b5144c17c7231c80f5fd1c7d0e1f.
