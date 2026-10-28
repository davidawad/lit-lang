
### `cit/cit-rules-and-forms.txt` — USCIT Rules, Forms, Guidelines and Administrative Orders (complete)

- Source: https://www.cit.uscourts.gov/sites/cit/files/COMPLETE%20RULES%20AND%20FORMS.pdf (official site; 487 pages, sha256 6ff22a48484b96c655f5738eae66ea0cfe92941a60e69dc845a41617c56a412a).
- Edition: Rules of the U.S. Court of International Trade, "As amended August 21, 2026, eff. August 21, 2026" per cover; the complete listing of rules, forms, guidelines and administrative orders linked from https://www.cit.uscourts.gov/uscit-rules-forms-chambers-procedures-guidelines-and-administrative-orders (page dated 8/21/26). robots.txt (Crawl-delay 10 honored) and privacy-policy page read first: no automated-access ban. Fetch method: curl with UA litgraph-rules-vendor/1.0, no bot challenge.
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); extracted with `pdftotext -layout` (poppler 26.06), unedited.
- Extract sha256 094c84ab0046945d6825a63b2385646ad8e2f601359b176c2b4ee59aade61223. Sectioned for cite resolution (lg-f3q8.16): a `## USCIT R. N` marker line was inserted before the heading of each rule's body text (112 rules, Rule 1 through Rule 89; the table of contents is unmarked, and the guidelines, forms and administrative orders after Rule 89 fall in its section). Rule text was not edited: deleting the marker lines (`sed '/^## USCIT R\. /d'`) restores the extract byte for byte.
- Sectioned file sha256 c411c8e42d69f14c9c9aa7409ade836fd25727b3f4ffd47e211d72268f6e6f41 (the `path_sha256` pinned by `lit/cit/Lit.toml`).
- Original: `cit/original/cit-rules-and-forms.pdf` as served (sha256 6ff22a48484b96c655f5738eae66ea0cfe92941a60e69dc845a41617c56a412a); extraction sha256 094c84ab0046945d6825a63b2385646ad8e2f601359b176c2b4ee59aade61223.
