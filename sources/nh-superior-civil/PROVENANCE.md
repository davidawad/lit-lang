
### `nh-superior-civil/superior-court-rules.txt` — NH Rules of the Superior Court

- Source: https://www.courts.nh.gov/rules-superior-court-state-new-hampshire (official site).
- Edition: as published on the court's rules page on 2026-10-27 (the page states no edition date).
- Retrieved 2026-10-27. Fetch method: BrowserOS neo; site served a bot challenge to plain automated requests (CoS ruling 2026-10-27). Read in the browser one page per request, at most one request per second; robots.txt and the site policy pages were read first and bar no such access. Each set is one online-book page whose rule text is held in the page's table widget; the text was taken from the rendered page in the browser (one rule per row, headings from the page's table of contents). courts.nh.gov robots.txt and the State of NH privacy and terms-of-use pages bar no such access.
- Extract: 84 `## <set> <rule or section>` sections of page text, unedited apart from dropping site navigation. sha256 0b947c2526348783bef150e83dc0f83e6092775bc9e12e56fa8e7d10233f2811.
- Original: `nh-superior-civil/original/superior-court-rules.html` as served (sha256 cb80f65c093111f193c19fd4907a118e6249e4ccfa30b288186a71a42976cd5b); extraction sha256 0b947c2526348783bef150e83dc0f83e6092775bc9e12e56fa8e7d10233f2811. Served at https://www.courts.nh.gov/rules-superior-court-state-new-hampshire; fetched in BrowserOS (bot challenge on plain curl), retrieved 2026-10-27.

### `nh-superior-civil/superior-court-supplemental-efiling-rules.txt` — NH Supplemental Rules of the Superior Court for Electronic Filing

- Source: https://www.courts.nh.gov/supplemental-rules-superior-court-new-hampshire-electronic-filing (official site).
- Edition: as published on the court's rules page on 2026-10-27 (the page states no edition date).
- Retrieved 2026-10-27. Fetch method: BrowserOS neo; site served a bot challenge to plain automated requests (CoS ruling 2026-10-27). Read in the browser one page per request, at most one request per second; robots.txt and the site policy pages were read first and bar no such access. Each set is one online-book page whose rule text is held in the page's table widget; the text was taken from the rendered page in the browser (one rule per row, headings from the page's table of contents). courts.nh.gov robots.txt and the State of NH privacy and terms-of-use pages bar no such access.
- Extract: 17 `## <set> <rule or section>` sections of page text, unedited apart from dropping site navigation. sha256 275eb8e2e365b9ea2028ccbc80e55b7a6dab5c27e268516efaebc499d396d5de.
- Original: `nh-superior-civil/original/superior-court-supplemental-efiling-rules.html` as served (sha256 a7d9d23cbc06363b0f55cdd092ec3a4982fcdd118719758cccd276e744564f3f); extraction sha256 275eb8e2e365b9ea2028ccbc80e55b7a6dab5c27e268516efaebc499d396d5de. Served at https://www.courts.nh.gov/supplemental-rules-superior-court-new-hampshire-electronic-filing; fetched in BrowserOS (bot challenge on plain curl), retrieved 2026-10-27.

### `nh-superior-civil/superior-court-rules-sectioned.txt` — NH Rules of the Superior Court, sectioned

- Derived in the pod (lg-f3q8.24) from `nh-superior-civil/superior-court-rules.txt` (extract sha256 0b947c2526348783bef150e83dc0f83e6092775bc9e12e56fa8e7d10233f2811); nothing fetched. One `## Rule <n>` line is inserted right after each `## NH Superior Court Rules Rule <n>. <title>` heading line (79 rules: Rule 1 through Rule 55 with 13A, 13B and 28A, Rules 201-208 and Rules 301-313). Rule text was not edited: deleting the marker lines (`sed '/^## Rule /d'`) restores the extract byte for byte.
- Edition: as the extract, as published on the court's rules page on 2026-10-27.
- Sectioned sha256 c0f01d7c9a95400693c5b89c589570a15b0cc941d1d9e74667a19914d1553874 (the `path_sha256` pinned by `lit/nh-superior-civil/Lit.toml` for source `nh-super-r`).
