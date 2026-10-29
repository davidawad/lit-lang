# engine — "Filed in Milliseconds"

What it's like to use litgraph's `litigate` CLI, shown as a docket sheet: every step is a
docket entry, the JSON is set on pleading paper (numbered lines, double
red margin rule), the parts that matter get a highlighter, and each result
gets a FILED stamp showing that request's real `elapsed_ms`.

## Files

- `marketing/engine/index.html`: the piece. Self-contained. The only
  external request is Google Fonts (Archivo Narrow + IBM Plex Mono), with
  fallback stacks. There's no `<head>`/`<meta>`, so all non-ASCII is
  escaped (HTML entities, `\u` in JS) and the page renders the same under
  any charset.
- `docs/media/litgraph-engine.mp4`: one 45 s loop, 1920×1080, 30 fps, H.264.
  Frames were rendered deterministically with `__render(t)` in headless
  Chromium and piped into ffmpeg.
  Poster: `docs/media/litgraph-engine.png` (1280×720, the index card at t = 1.5 s).

## Storyboard (45 s loop; starts and ends on the same card, so it loops seamlessly)

1. **0–3 s: index / end card.** Wordmark, thesis, `brew install davidawad/tap/litgraph`, GitHub URL, GPL-3.0, docket index of the entries, `"ok": true` stamp. This is also the no-JS and reduced-motion frame.
2. **3–8.5 s: 01 Install.** `brew install` typed, `litigate --version` → `litigate 0.6.0 (commit …, clean)`, the `packs` rows of `cases/ptab-ipr-defense.case` stream in, FILED 7.047 ms.
3. **8.5–13.5 s: 02 Request.** The `cases/requests/cofc-explain-dispositive-fork.case` matter (abridged) and the `litigate run` that queries it are typed. uses / matter facts and params / op are highlighted in sync with the caption.
4. **13.5–22 s: 03 Response.** The real envelope streams in (abridged with `…`): the `litigate` envelope with the engine's envelope as `result`, FILED 7.143 ms (the engine's `elapsed_ms`). The camera moves through policy (chosen/regret), warnings (codes + counts), and provenance (fingerprints, modes).
5. **22–37 s: 04 Capabilities.** Six 2.5 s cards, each with a real command, an animated visual, and a real fragment: compose forums, deadline math, tornado, state flags, named scenario, calibration.
6. **37–42 s: 05 MCP.** `claude mcp add litigate -- litigate mcp`, the 29 tool names from `tools/list`, then `tools/call deadlines` returns the same envelope (`due_date` 2026-11-27).
7. **42–45 s:** back to the end card.

Controls: Pause/Play, Replay, and a scrubbable rail; Space toggles.
`?t=12.5` freezes on a moment (used for screenshots). Under
`prefers-reduced-motion` it shows the composed end card and waits for
Play. Below 720 px wide it switches to a 3:4 portrait layout with the
same scenes stacked; there's no horizontal scroll at 400 px.

## Real litigate output used

Every scene is from `litigate` 0.6.0, built with
`cargo build --release -p litigate-cli` at commit 14e67db9 and run from the
repository root with `--format json`. Any of it can be regenerated with
the commands below.

- `litigate --version` → `litigate 0.6.0 (commit 14e67db963031f6420d067458c4bebf11e1b45fa, clean)`.
- The hero's 635 rule packages are the package directories under `lit/`; the 20 cross-forum links are the `link` declarations in `lit/links/links.lit`. 22 engine ops are the engine tools of `tools/list`.
- **Install:** `litigate run cases/ptab-ipr-defense.case --op packs`
  - `ptab-patent-trial-appeal-board` 55 nodes / 88 edges / 12 terminals, `cafc` 61 / 75 / 20, 7.047 ms
- **Request / response:** `litigate run cases/requests/cofc-explain-dispositive-fork.case --op '{"op":"explain","node":"cofc::patent-dispositive-motion-fork"}'`
  - value 927576.157779
  - `e-dispositivefork-sj` chosen, regret 0.0; `e-dispositivefork-trial` regret 55419.041863
  - warnings authored-hole×1, compile×69, mixed-node×4, payoff-not-authored×1, probability-fill×307
  - fingerprints `fnv1a64:952daeae3cee8de7` / `54fc31693f685e17`, 865 nodes / 964 edges, engine elapsed 7.143 ms
  - the outer envelope's `diagnostics` are 128 L9002 hole warnings
- **Compose:** `litigate graph cases/ptab-ipr-defense.case --arg node=federal-circuit-window`
  - edge `links::ptab-appeal-to-cafc` → `cafc@ptab::entry-ptab`, authority 35 U.S.C. §§ 141(c), 319; 37 C.F.R. § 90.3
  - 1,108 nodes / 1,300 edges compiled (flag states included), 8.732 ms
  - The dot grids are exactly 55 and 61 dots, matching the pack node counts.
- **Deadlines:** `litigate deadlines cases/requests/frcp-answer-deadline.case --arg trigger=2026-11-05 --arg node=service-completed`
  - FRCP 12(a)(1)(A)(i), 21 days → nominal 2026-11-26 (Thanksgiving) → due 2026-11-27, `last_day_rolled: true`, step text verbatim, 1.360 ms
- **Tornado:** `litigate tornado cases/requests/cofc-1498-tornado.case --arg 'params=["rate","stakes"]'`
  - base 1827748.310446; top five rows' v_low/v_high are drawn to scale
  - `p:cafc@cofc::panel-to-reversed` swing 873915.458347 and `p:cofc::e-prevailing-plaintiff` both with `policy_changes: true`, 252.745 ms
- **State flags:** `litigate graph cases/requests/ptab-ipr-estoppel-flag.case --arg 'node=post-fwd-options{ipr-estopped}' --no-links`
  - the three `fwd-issued` edges `sets [ipr-estopped]` (35 U.S.C. § 318(a), p .55/.25/.20, `lit/ptab/03-trial-to-appeal.lit`)
  - compile warning "104 additional node(s)", 1.315 ms
  - The on-screen claim is only that the flag is carried (CRITIQUE.md). The piece doesn't claim a blocked move.
- **Named scenario:** `litigate chain cases/cofc-1498-patent-case.case`
  - all seven absorption probabilities
  - expected dollars 252504.270312 / elapsed 302.224687 / hours 387.905781, fact-unset×2, 0.804 ms
- **Calibration:** `litigate explain cases/requests/ptab-calibrated-institution.case --arg node=institution-decision`
  - institution-granted goes from 0.65 (pack) to 0.681 (`ptab-fy2024`, n=1087, 740/1,087 from the USPTO FY24 roundup)
  - the two denial edges fall to 0.1595 each (residual split, warned), 1.646 ms
  - The 0.65 comes from the same matter without its `calibration`.
- **MCP:** `litigate mcp` over stdio, run from the repository root
  - `initialize` → `serverInfo` `litigate` 0.6.0
  - `tools/list`: 29 tools, the 22 engine ops plus `litigate_describe`, `litigate_check`, `litigate_build`, `litigate_where`, `litigate_timeline`, `litigate_actions`, `litigate_explain`
  - `resources/list`: 667 (`litigate://describe`, `litigate://links`, `litigate://scenarios`, 634 `litigate://packs/*`, 30 `litigate://scenarios/*`)
  - `tools/call deadlines {"target":"cases/requests/frcp-answer-deadline.case","trigger":"2026-11-05","node":"service-completed"}` → the `litigate` envelope with the engine response as `result`: due 2026-11-27, `last_day_rolled: true`, as on the CLI

`elapsed_ms` values come from one run on the build box and will vary run to run.

Values are shown in the engine's own units. The piece never calls them
dollars of real damages: the payoffs are placeholders, as the README says.
The footer carries "Not legal advice."

## Deliberately avoided

The `ptab-ipr-defense` named-scenario chain. Under the default scenario
it absorbs 100% into `claims-disclaimed-preinstitution`, with
not-converged warnings. That's accurate output, but it would read as a
bug in a montage, so the CoFC § 1498 scenario is used instead.

## With more time

- Portrait layout: code is ~8 px at a 400 px phone width. It's readable
  as a thumbnail; a real phone cut would trim lines to ~40 chars and
  show fewer of them.
- Add a paper-grain / ink-bleed texture to the stamps (currently flat,
  with multiply blending).
- Show an `ok:false` envelope with its `hint` (e.g. the "not found … did
  you mean" error), a strong agent-native beat left out for time.
- The brief mentions "court days": the deadline card shows FRCP 6
  calendar-day counting with a holiday roll. An RCFC/FRAP or ITC
  business-day example would show the other rule sets.
- Headless-render caveat: in this sandbox, Chromium only loaded Google
  Fonts over http (not `file://`). Browsers with system fonts are
  unaffected.
