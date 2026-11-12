# Main green follow-up: `lg-pb1fe.55`

## Step 0: main baseline

`origin/main` was `96aa55230e4db13da2fba734dbfe9e98d249b15d`, and the checkout started on that commit. The two timeline snapshot tests and the TTAB authority test failed on their stale recorded expectations; the impact test was already green. After the updates in Step 1, all four named tests passed on this tree.

## Step 1: positional snapshots and TTAB cite

Both timeline snapshots were re-recorded. `git diff -U0` showed 72 changed lines in `timeline__reset_absorbed_period.snap` and 24 in `timeline__frcp_filed_fixture.snap`; every changed line was a numeric `"start"`, `"end"`, or `"line"` source-location value. There were 0 non-positional changed lines.

The TTAB expected authority and replay inputs now name `FRCP (2017 ed.) 6(b)(1)(A); 37 CFR 2.116(a)`. The comment records that Rule 2.116(a) applies the FRCP as of the proceeding and the vendored text is the 2017 edition.

Targeted tests: both timeline tests and `the_generated_pack_carries_the_designated_answer_window` passed (exit 0).

## Step 2: impact test

`a_layer_edit_lists_every_moved_deadline_verdict_and_expectation` passed on `96aa5523`; the `.49` update already restored the test and its 1999 dates. No expected impact rows were changed.

The new boundary test in Step 3 also appears in the impact output. The fixture’s seven-to-three-day edit now updates its expected 1991 boundary date from Dec. 9 to Dec. 4 (exclude Dec. 1, count Dec. 2–4) and asserts both added impact rows. The impact test passes with 8 changes across 4 expect tests.

DECIDED: update the impact fixture and expected rows for the new boundary test -- the fixture deliberately changes the same rule from seven to three days, so its new assertion must use the independently computed three-day result.

## Step 3: strict L9011 diagnostics

The initial strict sweep found seven distinct L9011 declarations (the two FRAP-base diagnostics also appeared through the dependent FRAP packages). Each now has a discriminating inline test. The dates below are hand-computed from the stated counting rule; `litigate test` confirmed them.

| Diagnostic | Remedy and test | Hand computation |
|---|---|---|
| `frap-base/appeal.lit:36`, `notice-of-appeal` | Added `a notice of appeal rolls a weekend last day` | Exclude Jul. 4, 1968; count 30 days to Sat. Aug. 3; FRAP 26(a) rolls to Mon. Aug. 5. |
| `frap-base/appeal.lit:81`, `reopen-motion-window` | Added `a seven-day reopening window rolls a weekend last day` | Exclude Sun. Dec. 1, 1991; count Dec. 2–8 (7 days); roll Sunday to Mon. Dec. 9. |
| `frap-2009/rule4.lit:11`, `reopen-motion-window-2009` | Added `the 2009 reopening window rolls a weekend last day` | Exclude Dec. 5, 2009; count 14 days to Sat. Dec. 19; roll to Mon. Dec. 21. |
| `frcp-2009/periods.lit:20`, `new-trial-motion-2009` | Added `a new-trial deadline rolls a holiday and weekend` | Exclude Dec. 4, 2009; count 27 days through Dec. 31, then day 28 is New Year's Day; roll the holiday and weekend to Mon. Jan. 4, 2010. |
| `frcp-2009/periods.lit:26`, `alter-amend-motion-2009` | Added `a Rule 59(e) deadline rolls a holiday and weekend` | Same 28-day count from Dec. 4; Jan. 1 is the holiday last day, followed by Sat./Sun.; due Jan. 4. |
| `frcp-2009/periods.lit:32`, `renewed-jmol-motion-2009` | Added `a renewed JMOL deadline rolls a holiday and weekend` | Same 28-day count from Dec. 4; Jan. 1 is the holiday last day, followed by Sat./Sun.; due Jan. 4. |
| `frcp-2009/periods.lit:38`, `class-appeal-petition-2009` | Added `a class-appeal petition rolls a weekend last day` | Exclude Dec. 5, 2009; count 14 days to Sat. Dec. 19; FRCP 6(a)(1)(C) rolls to Mon. Dec. 21. |

After the edits, strict checks of all eight change packages reported 0 L9011 errors and 0 L9011 warnings. `litigate test` passed in all eight package directories.

## Step 4: strict diagnostics compared with `984e3187`

The old tree was cloned to `/tmp/oldtree`, checked out at `984e3187`, and built with its own `CARGO_TARGET_DIR=/tmp/old-target`. Strict diagnostics were then measured with that tree's release CLI. `worlds` is a scenarios directory without `Lit.toml`, so it is not a package and was skipped.

| Change directory | Error code | At `984e3187` | Now |
|---|---:|---:|---:|
| `frap-2005` | — | 0 | 0 |
| `frap-2009` | — | 0 | 0 |
| `frap-base` | L5002 | 0 | 2 |
| `frcp-1948` | L9001 | 4 | 4 |
| `frcp-2009` | — | 0 | 0 |
| `frcp-base` | — | 0 | 0 |
| `scotus-2003-criminal` | L9001 | 2 | 2 |
| `scotus-base` | L9001 | 2 | 2 |

The four FRCP 1948 L9001 errors and the two L9001 errors in each SCOTUS package are present at the baseline. The two FRAP-base L5002 errors are the statute-fixed extension and reopen periods intentionally left as holes by `.47/.49` (`appeal.lit:100` and `:104`). There are no other new strict errors after the lock refresh.

## Step 5: locks

`just lit-sync` exited 0 and updated only the three edited package locks. After those changes were committed, `just lit-sync-check` exited 0: `Lit.lock` files and the shared clock library are up to date, and no generated file is committed.

## Step 6: gates

| Gate | Result |
|---|---|
| `cargo nextest run -p litigate-cli -p litigate-lsp -p litigate-verify -p litigate-base -p litigate-syntax --no-fail-fast` | Final run exited 0: 1,150 passed, 0 failed, 0 skipped (2,994.328 seconds; 21 slow). The first full run had one impact failure before the fixture was updated; the final full run passed. |
| Release `litigate check lit cases --format json` | Exit 0; `ok: true`, 0 errors; 316,140 ms. |
| `litigate test` for the eight package directories | All exited 0: frap-2005 1/0 failed; frap-2009 2/0; frap-base 6/0; frcp-1948 0/0; frcp-2009 8/0; frcp-base 4/0; scotus-2003-criminal 0/0; scotus-base 0/0. |
| Release `litigate fmt --check lit cases docs/litigate/change` | Exit 0. |
| `cargo clippy --workspace --all-targets -- -D warnings` | Exit 0. |
| `cargo fmt --all --check` | Exit 0 after formatting the impact-test update. |

The impact test was rerun directly after its fixture and expected rows were updated; it passed (1 passed, 0 failed).
