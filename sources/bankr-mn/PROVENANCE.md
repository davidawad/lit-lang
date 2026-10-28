
### `bankr-mn/mnb-lbr-2025-09-03.txt` — Local Rules of the U.S. Bankruptcy Court, District of Minnesota (complete set, effective September 3, 2025)

- Source: https://www.mnb.uscourts.gov/sites/mnb/files/MNB%20Local%20Rules%20Complete.pdf (official site; 64 pages, sha256 cd5a2f0e5e8c1f339883a59aaf16a1ca466c978bd03e3d77c9eaaab0fc1052a7).
- Edition: Local Rules, 'Effective September 3, 2025' (cover), linked as 'Complete Set of Local Rules' from https://www.mnb.uscourts.gov/content/local-rules. robots.txt (Crawl-delay 10 honored; no rule covering these paths) and the Privacy Policy checked: no prohibition of automated access. Fetched with curl and the identifying User-Agent lit-lang-provenance/1.0 (angrybohr@proton.me).
- Retrieved 2026-10-28 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 fa210ac35aa63edc3ec5160c96edb1986cd2690742d18e3cd37ade8c25ccf79f. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `bankr-mn/original/mnb-lbr-2025-09-03.pdf` as served (sha256 cd5a2f0e5e8c1f339883a59aaf16a1ca466c978bd03e3d77c9eaaab0fc1052a7); extraction sha256 fa210ac35aa63edc3ec5160c96edb1986cd2690742d18e3cd37ade8c25ccf79f.
- Amending-order check (lg-u561.30): order page read 2026-10-27 (https://www.mnb.uscourts.gov/en-banc-standing-orders and https://www.mnb.uscourts.gov/local-rule-amendment-redlines): no order rewrites the rule text after 2025-09-03 (the newest redlines page is September 3, 2025 = the consolidation). One en banc order supplements LR 1007-3-1 and is vendored below as a layer; the case-assignment order (2026-10-05) and the sealed-documents en banc order (2026-05-29) do not amend the local rules and are not vendored.

### `bankr-mn/mnb-en-banc-order-claims-review-2026-02-11.txt` — Minnesota Bankruptcy Amended En Banc Order, Required Review of Claims (supplements LR 1007-3-1)

- Source: https://www.mnb.uscourts.gov/sites/mnb/files/En%20banc%20order%20re%20claims%20review%20-%202.11.26.pdf (official site; 2 pages, sha256 612726c95e5097b0929ea0268fb70cb72f6e8137cb2fa784cc363ee35cf89574).
- Edition: Required Review of Claims by Attorney for Debtor (Amended En Banc Order), listed as 'August 1, 2025 (for claims) February 23, 2026 (for Notice of Responsibilities Form)' on https://www.mnb.uscourts.gov/en-banc-standing-orders; file dated 2.11.26. It supplements LR 1007-3-1 (the Notice of Responsibilities form) and says the Practice Committee is developing local rule and form amendments to take effect in 2026; it does not rewrite the rule text. Apply on top of mnb-lbr-2025-09-03.txt from 2025-08-01 (claims review) and 2026-02-23 (Notice of Responsibilities forms). robots.txt (Crawl-delay 10 honored; no rule covering these paths) and the Privacy Policy checked: no prohibition of automated access. Fetched with curl and the identifying User-Agent lit-lang-provenance/1.0 (angrybohr@proton.me).
- Retrieved 2026-10-28 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 5897edc9c836803c70e8e88c37860af82ea482ca95b3a35b9095d1e6a13d6c9d. Raw extract, not yet sectioned into `## <ref>` form.
- Original: `bankr-mn/original/mnb-en-banc-order-claims-review-2026-02-11.pdf` as served (sha256 612726c95e5097b0929ea0268fb70cb72f6e8137cb2fa784cc363ee35cf89574); extraction sha256 5897edc9c836803c70e8e88c37860af82ea482ca95b3a35b9095d1e6a13d6c9d.

### `bankr-mn/mnb-lbr-2025-09-03-sectioned.txt` — D. Minn. Local Rules, sectioned

- Derived in the pod (lg-u561.23) from `bankr-mn/mnb-lbr-2025-09-03.txt` (extract sha256 fa210ac35aa63edc3ec5160c96edb1986cd2690742d18e3cd37ade8c25ccf79f); nothing fetched. One `## Rule <n>` line is inserted before each rule's own heading line in the body (`Rule 1002-1. [ABROGATED]` through `Rule 9070-1. Exhibits`, including the abrogated rules, `Rule 1007-3-1.` and the headings that follow a form feed; 96 sections, matching the table of contents); the table of contents (whose lines carry dot leaders) is unmarked. Rule text was not edited: deleting the marker lines (`sed '/^## Rule /d'`) restores the extract byte for byte.
- Edition: as the extract, the Local Rules effective September 3, 2025.
- Sectioned sha256 1603cdec27b6a6ea932debe1af7bd9c4c3c61440bfda275eecae52d2565065de (the `path_sha256` pinned by `lit/bankr-mn/Lit.toml` for source `mnb-lbr`).

### `bankr-mn/mnb-en-banc-order-claims-review-2026-02-11-sectioned.txt` — Amended En Banc Order (claims review), sectioned

- Derived in the pod (lg-u561.23) from `bankr-mn/mnb-en-banc-order-claims-review-2026-02-11.txt` (extract sha256 5897edc9c836803c70e8e88c37860af82ea482ca95b3a35b9095d1e6a13d6c9d); nothing fetched. One line `## En Banc Order LR 1007-3-1` is inserted before `IT IS HEREBY ORDERED that:`, naming the rule the order supplements (its paragraph (b) is "To comply with Local Rule 1007-3-1"), so that cites to the order's paragraphs resolve. Order text was not edited: deleting the marker line (`sed '/^## En Banc Order /d'`) restores the extract byte for byte.
- Sectioned sha256 25946c9a75cd128ec52eb701308d93f9de0ff8d12f4f2cfecf8cfc674ba32870 (the `path_sha256` pinned by `lit/bankr-mn/Lit.toml` for source `mnb-en-banc-claims-review`).
