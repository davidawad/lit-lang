# Strict dependency reachability through metadata-less declarations

## Rule and implementation

Strict dependency-hole reachability starts with every non-test declaration in each package named by the user and follows resolved declaration references transitively. The index includes declarations with and without `hir::Metadata`; this lets a path continue through events, enums, and type aliases. A reachable dependency declaration's L5002 remains an error attributed to the requested package. Holes in unreachable dependency declarations are summarized as informational diagnostics. Packages named directly are still judged in full.

Inline `test` declarations are excluded from the dependency index and are not reachability roots. A test states expectations about a rule; that rule does not depend on the test that exercises it. `Lit.toml` counting rules, recounts, and layered package dependencies continue to seed the existing additional roots.

The new fixture test builds a matter package that reaches a hole in a dependency through an event, a type alias, and its enum type. The prior attempt's logged red/green proof showed the test failing with the old metadata-only walk and passing with the new walk; that proof was not repeated. The new test passed in this run.

## Declaration-key collisions

The corpus-wide HIR index contains **120 `(package, id)` keys** shared by different declarations. The runtime census found these same-kind pairs (duplicate declaration occurrences, by kind): Authority 1, Calendar 1, Counting 44, Deadline 84, Limit 2, Node 25, and Rule 4. There were no cross-kind collisions, including between Metadata-less event, enum, and type-alias ids and Metadata ids.

The index maps one declaration per qualified key; each later declaration encountered in package/module/source order replaces the earlier value in both the declaration-reference and local-id maps. For example, `bankr-mie-local-rules::completion-objection-period` has an earlier Deadline version at `lit/bankr-mie/local-rules.lit:403` and a later version at line 408; the latter wins. Ten colliding keys from the census are:

- `bankr-mie-local-rules::completion-objection-period`
- `bankr-ri-local-rules::response-period`
- `bankr-ri-local-rules::compromise-response-period`
- `bankr-ri-local-rules::sale-response-period`
- `bankr-ri-local-rules::secured-claim-response-period`
- `bankr-ri-local-rules::abandonment-response-period`
- `bankr-ri-local-rules::compensation-response-period`
- `bankr-ri-local-rules::reconsideration-response-period`
- `bankr-ri-local-rules::codebtor-stay-response-period`
- `bankr-ri-local-rules::late-document-refile-period`

These collisions are same-kind versions of one logical declaration id, so they cannot make a different kind of declaration or an unrelated hole reachable. The current one-value index follows the last version's references; it can omit references unique to an earlier version. This is the pre-existing behavior for Metadata declarations and was not changed by the Metadata-less traversal.

DECIDED: retain the qualified bare-id index -- the corpus has no cross-kind collisions, so a kind-qualified index is not needed to prevent unrelated holes from becoming reachable.

## FRCP timing

Three release-binary runs of `litigate check --profile strict lit/frcp`; baseline is release `origin/main` at `70b15de0`, with its own `CARGO_TARGET_DIR`.

| Run | Baseline seconds | Changed seconds |
| --- | ---: | ---: |
| 1 | 4.286 | 4.302 |
| 2 | 1.728 | 1.728 |
| 3 | 1.736 | 1.717 |

The first run in each set includes cold process/source loading; the two warm runs are effectively unchanged.

## Strict corpus sweep

Release `litigate check --profile strict` was run for all 635 immediate package directories under `lit/`, with four workers, once on each binary. Pass/fail is the command's `ok` verdict.

| Binary | Passed | Failed |
| --- | ---: | ---: |
| Baseline (`origin/main` `70b15de0`) | 487 | 148 |
| Changed | 487 | 148 |

Three packages had changed diagnostic sets; none changed its pass/fail status:

| Package | Baseline → changed | Example changed diagnostic |
| --- | --- | --- |
| `lit/links` | 1 → 13 errors | New L5002 at `lit/links/../cofc/06-trial-post-trial.lit:38`, `post-judgment-motion-period` |
| `lit/ky-court-of-appeals` | 40 → 48 errors | New L5002 at `lit/ky-court-of-appeals/../ky-rules/rules.lit:19`: `RAP 31(G)` text in force is unverified |
| `lit/ky-supreme-court` | 30 → 36 errors | New L5002 at `lit/ky-supreme-court/../ky-rules/rules.lit:26`: `RAP 31(G)` text in force is unverified |

For `lit/links`, the 12 newly reachable L5002 periods are:

| Source location | Period |
| --- | --- |
| `cofc/06-trial-post-trial.lit:38` | `post-judgment-motion-period` |
| `cofc/06-trial-post-trial.lit:79` | `costs-period` |
| `frap/04-record.lit:18` | `transcript-order-period` |
| `frap/04-record.lit:32` | `transcript-filing-period` |
| `frap/05-motions.lit:16` | `motion-response-period` |
| `frap/06-briefing.lit:23` | `appellant-brief-period` (2009 version) |
| `frap/06-briefing.lit:29` | `appellee-brief-period` (2009 version) |
| `frap/06-briefing.lit:35` | `reply-brief-period` (2009 version) |
| `frap/06-briefing.lit:50` | `appellant-brief-period` (2018 version) |
| `frap/06-briefing.lit:56` | `appellee-brief-period` (2018 version) |
| `frap/06-briefing.lit:62` | `reply-brief-period` (2018 version) |
| `frap/10-mandate.lit:17` | `mandate-period` |

One actual reference chain is `links::cafc-remand-to-cofc` → `cofc::trial-bench` → `cofc::verdict-rendered` → `cofc::judgment-rcfc58-entered` → `cofc::post-trial-motion-window` → `cofc::post-judgment-motion-period`. The remand link enters the CoFC trial graph; the trial and judgment declarations lead to the post-trial motion rule and its counting period.

The 13 changed errors in `lit/links` include its pre-existing L9004 formatting error at `lit/ptab/03-trial-to-appeal.lit:66`; the other 12 are the L5002 entries above. The seven additional L5002 diagnostics in that run remain informational summaries for unreachable dependency holes.

A search of `justfile`, `.github/`, and `crates/*/tests` found no assertion that **strict** checking passes for `lit/links`. `crates/litigate-cli/tests/links_package.rs` asserts that the default-profile `check lit/links` and its tests pass. `justfile` has a generic strict-check recipe whose targets are supplied through `ARGS`.

## Verification

- `cargo nextest run -p litigate-cli --test check --no-fail-fast`: 14 tests, 7 passed and 7 failed. Every failure was on the PM's known-main-red list; no test outside that list failed. The added `strict_dependency_holes_follow_metadata_less_events_and_types` test passed separately.
- `cargo clippy -p litigate-cli --all-targets --keep-going -- -D warnings`: exit 0.
- `cargo fmt --check`: exit 0.
- No files under `lit/` or `cases/` were changed.
