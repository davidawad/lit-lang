# FRAP 26(b) extension typing in the change layers

DECIDED: retain the notice period before the 1997 edition's 1996-12-01 `as_of` as an untyped extension question because no earlier version is proved by the vendored Rule 26 text; type the 1997 and later notice periods only where the printed Rule 26(b) text forbids enlargement.

DECIDED: use `extend { never }` on the notice period only for Rule 26(b)'s independent enlargement authority. Rule 4(a)(5) remains its own move, authorized by Rule 4 and 28 U.S.C. § 2107(c).

## Source proofs

The 1997 page's Rule 26(b) sentence is read from the transcription itself: “The court for good cause shown may upon motion enlarge the time prescribed by these rules or by its order for doing any act, or may permit an act to be done after the expiration of such time; but the court may not enlarge the time for filing a notice of appeal, a petition for allowance, or a petition for permission to appeal.” Its printed history line ends with “Apr. 23, 1996, eff. Dec. 1, 1996.” The page's 1996 committee note discusses only subdivision (c); it supplies no earlier effective date for subdivision (b).

| Source | Official URL | Edition and `as_of` | History proving `as_of` | Transcription SHA-256 | As-served original SHA-256 |
|---|---|---|---|---|---|
| FRAP Rule 26 | <https://www.govinfo.gov/content/pkg/USCODE-1997-title28/html/USCODE-1997-title28-app-federalru-rule26.htm> | 1997; `1996-12-01` | “Apr. 23, 1996, eff. Dec. 1, 1996.” (`sources/usc-28-app-1997-time.txt:6`) | `4915165f4a157fdd8c4c5bae85519368f9d6ed00591f11815b8b1fcf320c5553` | `a0cd0764bc78a848b77ff9067604fd7fe4e70795978a6fa2a0dcd9f24329ca67` |
| FRAP Rule 26 | <https://www.govinfo.gov/content/pkg/USCODE-2000-title28/html/USCODE-2000-title28-app-federalru-rule26.htm> | 2000; `1998-12-01` | “Apr. 24, 1998, eff. Dec. 1, 1998.” | `7722e4a1467ea16820efb8cd340f4550d46142fe57d85a1971468b9f865aaca4` | `b60cf061694b74e4b3e0a5c0c9b230174b777d63ebef80c25864eae8ce7db975` |
| FRAP Rule 26 | <https://www.govinfo.gov/content/pkg/USCODE-2002-title28/html/USCODE-2002-title28-app-federalru-rule26.htm> | 2002; `2002-12-01` | “Apr. 29, 2002, eff. Dec. 1, 2002.” | `8ae71f441bd39ec678a8af7619d05b04f1585def3031aa4ff597f9fbee336309` | `376b24ff236b967e9d4b3d3fad881f3f479218e9d3945487b727f226a4b20d0d` |
| FRAP Rule 26 | <https://www.govinfo.gov/content/pkg/USCODE-2008-title28/html/USCODE-2008-title28-app-federalru-rule26.htm> | 2008; `2005-12-01` | “Apr. 25, 2005, eff. Dec. 1, 2005.” | `2133d8ea75e56413d8efe5ad058fffba2d27526d74b5c3e23437d2576ed42188` | `c9a9e2174f36cce7706030fa5fb55be3c0661960736086b1de57afc8475d8cdc` |
| 28 U.S.C. § 2107 | <https://www.govinfo.gov/content/pkg/USCODE-2008-title28/html/USCODE-2008-title28-partV-chap133-sec2107.htm> | 2008 ed.; source page | Statute text is the 2008 Code page; no rule-version boundary is inferred from it. | `d55b39782aeca1dc5d33a36b544569db9746833338a8c2b484f5785ca452426b` | `d44f221964296f6036179e7b8746e5019953acc7440e56abcf975bdeaaee805b` |

The 2000, 2002, and 2008 source transcriptions now include the source-credit history line as well as the rule text. Their Rule 26(b) notice-of-appeal exception is identical; the 2002 and 2008 edition boundaries are therefore not period-version boundaries. The 1997 transcription and original are recorded under `sources/PROVENANCE.md`; the 2008 § 2107 transcription and original are recorded there as well.

## Periods and typing

| Period | Dates | Controlling sentence | Typing |
|---|---|---|---|
| FRAP notice of appeal | 1968-07-01–1996-11-30 | Earlier Rule 26(b) text is not established by the reachable 1997 edition. | Open for David; no extension classification asserted. |
| FRAP notice of appeal | 1996-12-01–1998-11-30 | 1997 Rule 26(b): “the court may not enlarge the time for filing a notice of appeal.” | `extend { never }` under Rule 26; Rule 4(a)(5) remains the separate statute-authorized move. |
| FRAP notice of appeal | 1998-12-01 onward | 2000 Rule 26(b): the court may not extend a notice-of-appeal filing time “except as authorized in Rule 4.” The sentence is unchanged in 2002 and 2008. | One `extend { never }` Rule 26 lock citing the 2000 edition. It blocks a second Rule 26 enlargement, not Rule 4(a)(5). |
| FRCP 23(f) class-appeal petition | 1998-12-01 onward | 2000 Rule 26(b): “a petition for permission to appeal” is among the prohibited extensions; the 2002 and 2008 text is unchanged. | One `extend { never }` lock from the petition's first day, citing FRAP (2000 ed.) 26(b). |
| FRAP 4(a)(5) extension-motion period | 1968-07-01 onward | 28 U.S.C. § 2107(c): motion “not later than 30 days after the expiration” of the otherwise-set appeal time. Rule 26(b) (2000 ed.) separately bars extending the notice period except under Rule 4. | Undeclared statutory period; no general Rule 26 grant and no self-triggering `on AppealExtensionMoved()` extension. |
| FRAP 4(a)(6) reopening-motion period | 1991-12-01 onward | 28 U.S.C. § 2107(c): motion “within 180 days after entry ... or within 7 days after receipt of such notice, whichever is earlier.” | Undeclared statutory period. The 180-day outer bound is also part of this statute-fixed motion limit. |

## Holes and checks

The inherited lg-pb1fe.47 report recorded FRCP base at 0 holes and FRAP base at 1 hole (the reopening option). In this pod, the initial baseline CLI attempts returned 127 because `litigate` was not installed; the inherited report's diagnostics were FRAP dev L9002 at `appeal.lit:163`, L9011 warnings at `:36` and `:140`, and strict L5002 at `:163`.

After this change, compilation reports FRCP base 0 holes and FRAP base 2 holes. The FRAP dev diagnostics are L9002 at `appeal.lit:100` (the `extend` option) and `:104` (the `reopen` option); strict reports corresponding L5002 errors. Both are the statutory motion-period holes required by § 2107(c). Existing FRAP L9011 counting-rule warnings remain. The FRCP package reports no diagnostics in `periods.lit`.

The restored inline test is named “before 2005 a motion to reopen is due 7 days after notice of entry, weekends skipped” and uses `1999-05-28` / `1999-06-04`. Its earlier check failure was not a date error: the declarations had applied Rule 26(b)'s general grant to periods whose motion limits are set by § 2107(c). The corrected declarations retain the expected L9002/L5002 holes instead of moving the test dates to silence the checker.

## Open for David

- The Rule 4(a)(5) 30-day motion period and Rule 4(a)(6) 180-day/7-day motion period are fixed by 28 U.S.C. § 2107(c); they remain undeclared unless a source expressly forbids enlargement in terms.
- The notice-period extension status before 1996-12-01 remains open because the 1997 edition's history only proves its printed Rule 26 text from 1996-12-01. The 1997 Rule 26(b) sentence supports `extend { never }` from that date. From 1998-12-01 the 2000 text's Rule 4 exception supports the same Rule 26 lock because Rule 4(a)(5) is modeled as its own move; the lock does not disable that move.

The release checks completed after lock regeneration: `litigate check lit cases` exited 0 with 0 errors (1,688 warnings across the full tree); dev checks exited 0 for both packages, strict FRCP exited 0, and strict FRAP exited 2 with the two intended L5002 holes. `litigate test` passed 14 of 14 inline tests across the eight change packages (the other four packages contain no inline tests). `litigate fmt --check lit cases docs/litigate/change`, `cargo fmt --all --check`, and `cargo clippy -p litigate-cli --all-targets -- -D warnings` exited 0. The requested `cargo nextest run -p litigate-cli --no-fail-fast -E 'binary(change_files) | binary(rulings) | binary(corpus) | binary(roundtrip)'` passed all 35 tests, including the ruling replay suite.
