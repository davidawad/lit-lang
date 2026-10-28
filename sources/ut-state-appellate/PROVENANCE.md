
### `ut-state-appellate/utah-urap-all-rules.txt` — Utah Rules of Appellate Procedure (URAP)

- Source: 1 page URLs below (official site).
- Edition: Utah Rules of Appellate Procedure, all rules, the live page at the Utah Courts legacy rules host (same currency note as URCP).
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); fetched with plain curl (no browser identity); one all-rules page, converted from HTML to text (tags stripped). The page states each rule effective date inline (latest URCP 2/13/2026, URAP 5/1/2026); robots.txt on legacy.utcourts.gov disallows only /resources/rules/comments/ and /approved/. The main www.utcourts.gov site answers plain curl with 406, which is why the legacy host was used.
- Assembled text sha256 d0968b604400e51920971120e55ee40d79b9c5e36ffd08ffa8bd3c31eadbd064. One `## <ref>` section per page; the pages fetched, once each:
  - https://legacy.utcourts.gov/rules/viewall.php?type=urap
- Original: `ut-state-appellate/original/utah-urap-all-rules/001-rules-viewall.php_type-urap.html` as served (sha256 0d36f22202ea09e7f5bd334b7af79c96a4a37473ecce883c6a004115d5c037a5); extraction sha256 d0968b604400e51920971120e55ee40d79b9c5e36ffd08ffa8bd3c31eadbd064. Served at https://legacy.utcourts.gov/rules/viewall.php?type=urap; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.

### `ut-state-appellate/utah-urap-all-rules-sectioned.txt` — URAP, sectioned

- Derived in the pod (lg-f3q8.70) from `ut-state-appellate/utah-urap-all-rules.txt` (sha256 d0968b604400e51920971120e55ee40d79b9c5e36ffd08ffa8bd3c31eadbd064); nothing fetched. One `## URAP <n>` line is inserted before each rule's own `Rule <n>. <title>` line (the one followed by its "Rule printed on" line; Rule 38A's heading has no period after the number), 68 rules. Rule text was not edited: deleting the marker lines (`sed '/^## URAP /d'`) restores the extract byte for byte.
- Upstream: https://legacy.utcourts.gov/rules/viewall.php?type=urap, retrieved 2026-10-27, as recorded on the `ut-state-appellate/utah-urap-all-rules.txt` entry this file is derived from.
- Edition: as the extract, each rule as in force on October 26, 2026 with its own "Effective:" line.
- Sectioned sha256 d69fcfe874d6a0518319e08d04f4ec6e069f5727d56c3df994e57301ab2a5888 (the `path_sha256` pinned by `lit/ut-state-civil/Lit.toml` and `lit/ut-state-appellate/Lit.toml`).
