
### `ut-state-civil/utah-urcp-all-rules.txt` — Utah Rules of Civil Procedure (URCP)

- Source: 1 page URLs below (official site).
- Edition: Utah Rules of Civil Procedure, all rules, the live page at the Utah Courts legacy rules host (single-rule pages there read "Rule printed on October 26, 2026" and point to https://www.utcourts.gov/rules for current rules).
- Retrieved 2026-10-27 on carbon (lg-f3q8 standing method); fetched with plain curl (no browser identity); one all-rules page, converted from HTML to text (tags stripped). The page states each rule effective date inline (latest URCP 2/13/2026, URAP 5/1/2026); robots.txt on legacy.utcourts.gov disallows only /resources/rules/comments/ and /approved/. The main www.utcourts.gov site answers plain curl with 406, which is why the legacy host was used.
- Assembled text sha256 42ad1e940bc69195dc060b0d20227a17bd1e0604cc9b9d651cf38eddcb7383af. One `## <ref>` section per page; the pages fetched, once each:
  - https://legacy.utcourts.gov/rules/viewall.php?type=urcp
- Original: `ut-state-civil/original/utah-urcp-all-rules/001-rules-viewall.php_type-urcp.html` as served (sha256 5436fa7f6cfd799f058ba808c4e4686abaa1e40218046100340a99bf8f6275e7); extraction sha256 42ad1e940bc69195dc060b0d20227a17bd1e0604cc9b9d651cf38eddcb7383af. Served at https://legacy.utcourts.gov/rules/viewall.php?type=urcp; fetched with curl (User-Agent lit-lang-provenance/1.0), retrieved 2026-10-27.

### `ut-state-civil/utah-urcp-all-rules-sectioned.txt` — URCP, sectioned

- Derived in the pod (lg-f3q8.70) from `ut-state-civil/utah-urcp-all-rules.txt` (sha256 42ad1e940bc69195dc060b0d20227a17bd1e0604cc9b9d651cf38eddcb7383af); nothing fetched. One `## URCP <n>` line is inserted before each rule's own `Rule <n>. <title>` line (the one followed by its "Rule printed on" line), 113 rules. The served page repeats Rule 26 verbatim; only its first copy is marked, so the second stays inside the `URCP 26` section. Rule text was not edited: deleting the marker lines (`sed '/^## URCP /d'`) restores the extract byte for byte.
- Edition: as the extract, each rule as in force on October 26, 2026 with its own "Effective:" line.
- Sectioned sha256 7672ae2cf0b3f9f360c4d6df1c80bcf8a5566437102fea4d4700352b75144dc9 (the `path_sha256` pinned by `lit/ut-state-civil/Lit.toml`).
