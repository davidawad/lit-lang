
### `ut-state-civil/utah-urcp-all-rules.txt` — Utah Rules of Civil Procedure (URCP)

- Source: 1 page URLs below (official site).
- Edition: Utah Rules of Civil Procedure, all rules, the live page at the Utah Courts legacy rules host (single-rule pages there read "Rule printed on October 26, 2026" and point to https://www.utcourts.gov/rules for current rules).
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); fetched with plain curl (no browser identity); one all-rules page, converted from HTML to text (tags stripped). The page states each rule effective date inline (latest URCP 2/13/2026, URAP 5/1/2026); robots.txt on legacy.utcourts.gov disallows only /resources/rules/comments/ and /approved/. The main www.utcourts.gov site answers plain curl with 406, which is why the legacy host was used.
- Assembled text sha256 42ad1e940bc69195dc060b0d20227a17bd1e0604cc9b9d651cf38eddcb7383af. One `## <ref>` section per page; the pages fetched, once each:
  - https://legacy.utcourts.gov/rules/viewall.php?type=urcp
