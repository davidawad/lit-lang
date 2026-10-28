
### `nh-supreme/supreme-court-rules.txt` — NH Rules of the Supreme Court

- Source: https://www.courts.nh.gov/rules-supreme-court-state-new-hampshire (official site).
- Edition: as published on the court's rules page on 2026-10-27 (the page states no edition date).
- Retrieved 2026-10-27. Fetch method: BrowserOS neo; site served a bot challenge to plain automated requests (CoS ruling 2026-10-27). Read in the browser one page per request, at most one request per second; robots.txt and the site policy pages were read first and bar no such access. Each set is one online-book page whose rule text is held in the page's table widget; the text was taken from the rendered page in the browser (one rule per row, headings from the page's table of contents). courts.nh.gov robots.txt and the State of NH privacy and terms-of-use pages bar no such access.
- Extract: 158 `## <set> <rule or section>` sections of page text, unedited apart from dropping site navigation. sha256 0a75ee011787ee0534bd65bd50a4bc75e501497194ba12ba8da456823a321f0d.
- Original: `nh-supreme/original/supreme-court-rules.html` as served (sha256 5407d14b908cd19b3bec97795f9bd9ec9437536d47ec7755df5ce4d372ccf2e9); extraction sha256 0a75ee011787ee0534bd65bd50a4bc75e501497194ba12ba8da456823a321f0d. Served at https://www.courts.nh.gov/rules-supreme-court-state-new-hampshire; fetched in BrowserOS (bot challenge on plain curl), retrieved 2026-10-27.

### `nh-supreme/supreme-court-supplemental-efiling-rules.txt` — NH Supplemental Rules of the Supreme Court for Electronic Filing

- Source: https://www.courts.nh.gov/supplemental-rules-supreme-court-new-hampshire-electronic-filing (official site).
- Edition: as published on the court's rules page on 2026-10-27 (the page states no edition date).
- Retrieved 2026-10-27. Fetch method: BrowserOS neo; site served a bot challenge to plain automated requests (CoS ruling 2026-10-27). Read in the browser one page per request, at most one request per second; robots.txt and the site policy pages were read first and bar no such access. Each set is one online-book page whose rule text is held in the page's table widget; the text was taken from the rendered page in the browser (one rule per row, headings from the page's table of contents). courts.nh.gov robots.txt and the State of NH privacy and terms-of-use pages bar no such access.
- Extract: 22 `## <set> <rule or section>` sections of page text, unedited apart from dropping site navigation. sha256 68177c8a998b8c5a4430e2fea600cbd5c1c9ff116a0acb4fbdd62ced2159a609.
- Original: `nh-supreme/original/supreme-court-supplemental-efiling-rules.html` as served (sha256 7a790203ac900acbc645b891afe6533622f8867e76da31228f65cfc1afe7e6c8); extraction sha256 68177c8a998b8c5a4430e2fea600cbd5c1c9ff116a0acb4fbdd62ced2159a609. Served at https://www.courts.nh.gov/supplemental-rules-supreme-court-new-hampshire-electronic-filing; fetched in BrowserOS (bot challenge on plain curl), retrieved 2026-10-27.

### `nh-supreme/supreme-court-rules-sectioned.txt` — NH Rules of the Supreme Court, sectioned

- Derived in the pod (lg-f3q8.24) from `nh-supreme/supreme-court-rules.txt` (extract sha256 0a75ee011787ee0534bd65bd50a4bc75e501497194ba12ba8da456823a321f0d); nothing fetched. One `## Rule <n>` line is inserted right after each `## NH Supreme Court Rules Rule <n>. <title>` (or `RULE <n>.`) heading line, first occurrence only (88 rules, Rule 1 through Rule 61 with their lettered rules such as 1-A, 7-A, 12-D, 16-A, 21A and 37A). The Code of Judicial Conduct's decimal rules (`Rule 1.1` and the like) are unmarked. Rule text was not edited: deleting the marker lines (`sed '/^## Rule /d'`) restores the extract byte for byte.
- Upstream: https://www.courts.nh.gov/rules-supreme-court-state-new-hampshire, retrieved 2026-10-27, as recorded on the `nh-supreme/supreme-court-rules.txt` entry this file is derived from.
- Edition: as the extract, as published on the court's rules page on 2026-10-27.
- Sectioned sha256 e948076e1ebf4faea0ae9b3ac30a2fffad45f759c3617388c49a9890699c54a7 (the `path_sha256` pinned by `lit/nh-superior-civil/Lit.toml` and `lit/nh-supreme/Lit.toml` for source `nh-sup-r`).
