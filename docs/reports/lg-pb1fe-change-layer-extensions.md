# FRAP 26(b) extension typing in the change layers

DECIDED: use the first reachable govinfo Code edition (2000) as the start of typed Rule 26(b) versions -- no earlier Code edition for the requested dates was reachable, so the earlier spans remain explicitly open rather than borrowing later text.

## Source proofs

Official U.S. Code Rule 26 pages and original downloaded HTML:

| Edition | Official URL | `as_of` | Transcription SHA-256 | Original SHA-256 |
|---|---|---|---|---|
| 2000 | <https://www.govinfo.gov/content/pkg/USCODE-2000-title28/html/USCODE-2000-title28-app-federalru-rule26.htm> | 2000-12-01 | `69fb015f59fd86b2b3890325ac5d5b99578d6b073637983e38a5e9161363ea20` | `b60cf061694b74e4b3e0a5c0c9b230174b777d63ebef80c25864eae8ce7db975` |
| 2002 | <https://www.govinfo.gov/content/pkg/USCODE-2002-title28/html/USCODE-2002-title28-app-federalru-rule26.htm> | 2002-12-01 | `c956d25165e0b6f3f2fa9ca894283c449d4d5119e2da8545e0aa4eb63c570e21` | `376b24ff236b967e9d4b3d3fad881f3f479218e9d3945487b727f226a4b20d0d` |
| 2008 | <https://www.govinfo.gov/content/pkg/USCODE-2008-title28/html/USCODE-2008-title28-app-federalru-rule26.htm> | 2008-12-01 | `3e2099086c665f26abb5c5c10f8c47a73b11305ee9a7e3d6921e5e024e24be5e` | `c9a9e2174f36cce7706030fa5fb55be3c0661960736086b1de57afc8475d8cdc` |

The text files contain the full statutory body of Rule 26, one section per official `statutory-body*` paragraph. Each transcription was compared sentence by sentence with its saved as-served HTML; only HTML entity and whitespace normalization was applied. The copies and provenance are under `sources/original/` and in `sources/PROVENANCE.md`.

Govinfo returned no edition for the 1976, 1990, or 1994 Rule 26 page URLs tried. The saved rule source therefore begins with the 2000 edition.

## Periods

FRAP 26(b) in the 2000, 2002, and 2008 editions says, for the general grant: “For good cause, the court may extend the time prescribed by these rules or by its order to perform any act, or may permit an act to be done after that time expires.” The same editions say: “But the court may not extend the time to file: (1) a notice of appeal (except as authorized in Rule 4) or a petition for permission to appeal.”

| Period | Dates | Controlling sentence | Typing |
|---|---|---|---|
| FRCP 23(f) class petition | 1998-12-01–2000-11-30 | Earlier FRAP 26(b) text not in a reachable official Code edition | Open for David; no later edition applied retroactively |
| FRCP 23(f) class petition | 2000-12-01–2002-11-30 | “...may not extend the time to file: ... a petition for permission to appeal.” | `extend { never }` lock, FRAP (2000 ed.) 26(b) |
| FRCP 23(f) class petition | 2002-12-01–2008-11-30 | Same petition prohibition | `extend { never }` lock, FRAP (2002 ed.) 26(b) |
| FRCP 23(f) class petition | 2008-12-01 onward | Same petition prohibition | `extend { never }` lock, FRAP (2008 ed.) 26(b) |
| Notice of appeal | 1968-07-01–2000-11-30 | Earlier FRAP 26(b) edition not reachable | Open for David; no later edition applied retroactively |
| Notice of appeal | 2000-12-01–2002-11-30 | “...may not extend the time to file: (1) a notice of appeal (except as authorized in Rule 4)...” | `extend { never }` lock; Rule 4(a)(5) remains the separate extension-motion move, not a second Rule 26 grant |
| Notice of appeal | 2002-12-01–2008-11-30 | Same Rule 4 exception | `extend { never }` lock, FRAP (2002 ed.) 26(b); Rule 4 move remains separate |
| Notice of appeal | 2008-12-01 onward | Same Rule 4 exception | `extend { never }` lock, FRAP (2008 ed.) 26(b); Rule 4 move remains separate |
| Rule 4(a)(5) extension motion | 1968-07-01–2000-11-30 | Earlier FRAP 26(b) edition not reachable | Open for David; no later edition applied retroactively |
| Rule 4(a)(5) extension motion | 2000-12-01–2002-11-30 | “For good cause, the court may extend the time prescribed by these rules ... to perform any act, or may permit an act to be done after that time expires.” | Good-cause grant over the motion deadline; Rule 4(a)(5) separately extends the notice period |
| Rule 4(a)(5) extension motion | 2002-12-01–2008-11-30 | Same general grant | Good-cause grant, FRAP (2002 ed.) 26(b) |
| Rule 4(a)(5) extension motion | 2008-12-01 onward | Same general grant | Good-cause grant, FRAP (2008 ed.) 26(b) |
| Rule 4(a)(6) reopening motion | 1991-12-01 onward | The reachable 2000/2002/2008 editions give the general grant quoted above, but the period starts before the earliest reachable official edition | Honest hole for the full period: older Rule 26(b) text is missing, so the later grant cannot be applied to the whole span |

Inline deadline tests cover the Rule 4(a)(5) and Rule 4(a)(6) period calculations. The inline test language has no extension-action or expected-refusal form, so it cannot exercise grant/refusal behavior; no test-only events were added.

## Checks

Before the change, dev checks reported FRCP base 1 hole (L9002 at `periods.lit:68`) and FRAP base 3 holes (L9002 at `appeal.lit:68`, `:72`, `:76`). Strict checks failed on those holes (L5002); FRAP strict also reported existing L9011 warnings at `appeal.lit:34` and `:53`.

After the change, FRCP base dev and strict checks both exited 0 with 0 holes. FRAP base dev exited 0 with 1 hole (L9002 at `appeal.lit:163`); strict exited 2 with that one L5002 error. Its two existing L9011 warnings remain at `appeal.lit:36` and `:140`. The test pins exact counts of 0 and 1 respectively and the exact remaining line.

Other measured checks: `litigate check lit cases` exited 0; `litigate test` on the two base packages and their three dependent amendment layers passed 14 tests with 0 failures; `litigate fmt --check lit cases docs/litigate/change` exited 0 with 0 files needing formatting; `cargo nextest run -p litigate-cli --no-fail-fast -E 'binary(change_files) | binary(rulings) | binary(corpus) | binary(roundtrip)'` ran 35 tests, all passed; `cargo clippy -p litigate-cli --all-targets -- -D warnings` and `cargo fmt --all --check` exited 0. The reopening-window timeliness result remained 7 days in the 1991/2005 texts and 14 days in the 2009 text.

Open for David: obtain and vendor the official pre-2000 Rule 26 text for the FRCP 23(f) 1998–2000 interval and the FRAP 4(a) 1968–2000 intervals, including the Rule 4(a)(6) reopening period beginning in 1991. Until then those spans remain untyped.
