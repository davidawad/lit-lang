# Agent guide

You are driving a litigation procedure engine. Read docs/ARCHITECTURE.md once;
this file is the operating manual.

## Commands

`litigate` is the one command line (lg-9lrj). Every engine op is a
subcommand on a compiled matter or package (`litigate <op> TARGET`), and
`litigate run TARGET --op JSON` runs any op, its arguments included; a
`.case` matter supplies the scenario (its `analysis`, `beliefs` and
`calibration`), a package its pack under the default scenario. There is no
raw JSON request interface: packs and named scenarios are compiled from
`lit/` and `cases/`.

```bash
cargo build --release                      # binary: target/release/litigate
cargo test                                 # unit + feature tests
litigate solve cases/cofc-1498-patent-case.case          # any op on a compiled matter
litigate sweep cases/frcp-civil-case.case --arg param=rate --set opp_rate=500
litigate explain cases/frcp-civil-case.case --arg node=frcp-civil-procedure::pre-filing
litigate chain lit/cofc                                  # a package: its pack, default scenario
litigate solve lit/links                                 # a links package: the graph it composes
litigate chain cases/cofc-1498-patent-case.case --set rate=900   # --set overrides analysis { params }
litigate calibration lit/frcp              # uncalibrated probabilities/durations, ranked by sensitivity
litigate lint lit/cofc                     # the engine's content diagnostics
litigate posterior cases/cofc-1498-patent-case.case      # credible intervals, P(option optimal)
litigate run cases/cofc-1498-patent-case.case --op '{"op":"voi","studies":[{"node":"cofc::sj-ruling","k":5,"cost":25000}]}'
                                            # what is worth paying to learn (EVPI / EVSI vs cost)
litigate run lit/cofc --op describe         # the engine's manual (ops, arguments, scenario library)
litigate run lit/cofc --op packs            # the compiled packs + data quality
litigate run cases/requests/cofc-1498-settle.case --op settle   # settle: ZOPA, Nash/Rubinstein price, when to settle, Rule 68
                                            # (needs scenario.opponent_objective; docs/SETTLEMENT.md;
                                            # the case's first line is its full `litigate run`)
```

Observations (a Bayesian update, docs/UNCERTAINTY.md), a robust objective
and real docket-derived rates (docs/CALIBRATION.md) are written in the
matter: `beliefs { observe ... }`, `analysis { objective ... }` and
`analysis { calibration: [...] }` (docs/litigate/SPEC.md §5.6).

Packs and named scenarios are **build outputs, never committed**:
`lit/<forum>/*.lit` and `cases/*.case` are the authoring surface (the
litigate language). `crates/litgraph-packs` compiles them (the `litigate
build` arguments in `crates/litgraph-packs/build.args`) at `cargo build`
time and embeds the result; `just lit-sync` writes the same build to
`target/lit-build/{packs,scenarios}` to read.
`packs/` holds only source notes: the shared clock library is `lit/core`
(lowered into `crates/litigate-engine/clock/` by `just lit-sync`), calibration
sets are `calibration` declarations in `lit/`, and runnable requests are
`cases/requests/*.case`. No JSON input remains; lowered JSON is internal IR.

```bash
cargo install --path crates/litigate-cli   # binary: litigate (or `cargo run -q -p litigate-cli --`)
litigate describe                           # the language + CLI contract as JSON — start here
litigate check lit/frcp                     # typecheck + verify one package (--profile strict for the gate)
litigate check lit cases                    # everything; diagnostics carry stable codes + fixes
litigate test lit/frcp                      # inline `test` blocks (deadline arithmetic etc.)
litigate test --impact origin/main          # what a branch moves: every tests/expect deadline, verdict
                                            # and expectation that differs from BASE, per forum (lg-8fdj.3)
litigate fix lit/frcp --dry-run             # machine-applicable fixes
litigate where frcp-civil-procedure::service-window  # id → declaration span, versions, JSON pointers
litigate explain L3004                      # what a diagnostic code means
litigate run cases/frcp-civil-case.case --op solve  # compile a matter and run an engine op
litigate calendar cases/ptab-ipr-defense.case --out popr.ics --remind 7
                                            # open deadlines as an RFC 5545 calendar (--all: every one);
                                            # an unknown due date is listed under `unscheduled`, never given one
litigate lint-filing cases/ptab-ipr-defense.case 'PreliminaryResponseFiled()' --date 2026-09-10 --by owner
                                            # replay the matter with a proposed filing: L4001 not a legal move,
                                            # L4002 late; else what it discharges and the deadlines it triggers;
                                            # each open deadline lists its `moves` (file / extend / orders),
                                            # e.g. the FRCP 54(d)(2)(B) fee clock's (lit/frcp/fees.lit)
litigate lint-filing cases/ptab-ipr-defense.case 'IprPetitionFiled()' --date 2026-09-10 --by petitioner \
  --words 13500 --certificate word-count     # the paper's limit (or --document FILE): violations with
                                            # their cite and layer; --paper KIND only if the event declares none
litigate actions cases/e2e-ttab-ok.case --party opposer
                                            # the party's action set: only the moves replay accepts from the posture
                                            # (illegal or late moves are absent); MCP tool `litigate_actions`
litigate match-docket cases/ptab-ipr-defense.case recap.json --out advanced.case
                                            # PACER/RECAP entries matched to declared event forms; only a supported,
                                            # replay-accepted match advances the matter; the rest stay unresolved
litigate calibrate-replay cases/dockets --id recap-replay --reviewed --out lit/frcp/calibration-recap-replay.lit
                                            # a `calibration` declaration from replayed, sourced docket histories (docs/CALIBRATION.md);
                                            # each estimate's method keeps its provenance; --out needs --reviewed
litigate diff lit/frcp --against target/lit-build/packs/frcp-civil-procedure.json   # semantic diff (SPEC §8.6)
litigate compat old/lit/frcp lit/frcp      # compatibility class of a release + the bump it needs (SPEC §4.4)
just lit-sync                               # regenerate every Lit.lock; build packs/, scenarios/ into target/lit-build/
just lit-sync-check                         # CI gate: fail on a stale Lit.lock or a committed generated file
just coverage                               # write docs/COVERAGE.md locally (`litigate coverage`, gitignored): per-jurisdiction
                                            # rules, precedents, expect tests (inline and tests/expect) and package status
```

`Lit.lock` files hold one row per package with only that package's own
digests (dependencies pinned as `name version`), so branches editing
different packages merge as plain text. When both sides changed one
package, its row conflicts: run `just lit-sync` and commit the result; a
lock that does not match its sources is an error (L1007), never relocked
silently. `litigate lock --root DIR` prints the lock's Merkle root.

The generated tree-sitter parser (`editors/tree-sitter-litigate/src`) is
committed and merges through the `tsgenerate` driver, which keeps ours
when both sides changed it (`just lit-sync` registers it too). After a
merge or rebase touching the grammar, run `just tree-sitter-generate` and
commit `src/`; `just tree-sitter-test` (in `just ci`) fails while it is
stale.

`docs/litigate/TUTORIAL.md` is the four-step walkthrough (write a rule,
test it, write a case, run solve); `docs/litigate/SPEC.md` is normative.
Over MCP the same toolchain is the `litigate_describe`, `litigate_check`,
`litigate_build`, `litigate_where`, `litigate_timeline`,
`litigate_actions` and `litigate_explain` tools. `litigate timeline FILE`
prints a `.case` or `.lit` file's timeline (rows, worlds, layers) as the
frozen `litigate.timeline/1` JSON (docs/litigate/TIMELINE.md). `litigate
worlds CASE --world FILE` replays a matter in every world a `.world` file
spans (its layers, one option per `[choice.ID]`, `--vary NAME` with and
without a layer) at `--as-of`/`--known-at`, and lists per filing the
worlds in which it is timely or late (docs/litigate/CHANGE.md §5.1).

Engine ops compile their packs from the `.lit` library on the disk (`--lib
DIR`, default `lit`) each run, so the library ships beside the binary,
never as JSON. The vendored L0 source corpus (`sources/*.txt`, see
`sources/PROVENANCE.md`) is embedded in the binary at build time;
override it with `LITGRAPH_SOURCES=<dir>`.

Driving this from an MCP-capable agent instead of the CLI: `litigate mcp`
is the same contract over stdio (this repository's `.mcp.json` registers
it). Every engine op is its own MCP tool run on a compiled matter
(`{target, set?, lib?, no_links?, ...op fields}`, as `litigate run`),
the `litigate_*` tools above drive the authoring language, and the compiled
packs, links, named scenarios and engine manual are MCP resources
(`litigate://packs/<id>`, `litigate://links`, `litigate://scenarios`,
`litigate://scenarios/<id>`, `litigate://describe`). `claude mcp add
litigate -- litigate mcp` wires it into Claude Code; see the README's "MCP
server" section for other clients.

Every response: `{ok, api_version, op, result, warnings, provenance,
elapsed_ms}` or `{ok:false, api_version, op, error:{code, message, hint},
elapsed_ms}`. `warnings` is grouped: `[{code, count, example, at}]`, `at`
holding up to eight locations. `api_version` is the request/response
contract version (bumped only on a breaking change; currently `1`).
Exit codes: `0` ok, `2` the request ran and failed / a document is invalid,
`1` usage or I/O error.

Parsing is strict everywhere — packs, scenarios, requests, and op
arguments reject unknown fields with an error naming the valid ones,
rather than silently ignoring a typo.

Reading the coverage result: the `just cov` verdict is its `cov:` line,
`cov: litgraph TOTAL lines P% (covered/count) >= floor 97%: pass` (or `<
... FAIL` and a nonzero exit). The verdict is exact integer arithmetic on
`covered/count` (`covered * 10000 >= floor_hundredths * count`; the floor
takes at most two decimals), never on a printed percentage; `P%` is that
ratio truncated for display, so the per-file table above it, which rounds,
can show 0.01 more. Quote that line and the recipe's exit code, never a
number from the table or from another run. `just cov-litigate` prints one
`cov-litigate: <crate> lines c/n regions c/n branches c/n: pass|FAIL` line
per crate. Both recipes start with `cargo llvm-cov clean --workspace`, so
data left in `target/llvm-cov-target` by another branch or crate cannot
skew the number, and end with a `cov self-check:` line: a floor of 100.01
or a known-low input must fail, else the recipe fails with `cov
self-check: floor not enforced`. `just cov-gate-test` (in `just ci`) runs
the gate (`scripts/cov-gate`) over fixed known-low and known-ok inputs.

## Rules

1. **Report warnings with numbers.** `payoff-not-authored`,
   `probability-fill`, `mixed-node`, `sink`, `authored-hole` mean the number
   rests on a fallback (`authored-hole`: a pack's `lowered.holes` left that
   number as an explicit `?`). Say so, or fix the input (scenario
   `payoffs`/`probabilities`) before answering.
2. **Matter facts go in the scenario, not the pack.** Packs describe a
   forum's procedure for everyone; stakes, rates, waivers, the judge's
   tendencies belong to one matter. A chance node whose outcome is actually
   knowable at filing (time-barred? already pending elsewhere? a patent
   case?) is tagged `"fact"` in the pack with an authored prior; the
   scenario resolves it for real via `scenario.facts` (`{node ref: edge
   ref}`), not by editing the pack. See `docs/PACK_SCHEMA.md#matter-facts-v2`.
3. **Counterfactuals via `compare`**, not by editing packs.
4. **Pack edits need sources, and happen in `lit/`.** Change a forum's
   procedure in its `lit/<forum>/` package (a matter in `cases/`), run
   `litigate check`, then `just lit-sync` and commit the source and its
   `Lit.lock` changes (the generated JSON is a build output, never
   committed); `just ci` fails on a stale `Lit.lock`. `docs/COVERAGE.md` is
   generated locally by `just coverage` and never committed; a promoted
   state-court package declares its row with `[package] jurisdiction_id`
   (e.g. `"alaska"`) in its own `Lit.toml`. Every new
   `authority`/`cite` must be traceable to a `sources` entry or a public
   citation (`sources[].path` is repo-relative only — never a local
   filesystem path); in `.lit` an unresolved cite is a hole. Run
   `validate`, `lint`, and `cargo test` after any pack change. Never invent rule
   numbers or deadlines; mark uncertainty `UNVERIFIED` in `note`. If
   `sources[].path` points at a file under `sources/`, `lint` verifies the
   cite mechanically (`unverifiable-cite` warning with the exact reason if
   it can't) — see `docs/PACK_SCHEMA.md`'s authoring rule 1.
   In `Lit.toml`, a binding with a vendored `path` pins that file with
   `path_sha256`; `litigate check` resolves cites only against the pinned
   text, and a cite is a string: `cite "FRCP 12(b)(6)"`. A cite whose
   vendored text is dated after its version takes effect is a hole unless
   the section's vendored amendment history (`## versions <heading>`, e.g.
   `sources/ecfr-versions-title-37.txt`) shows no amendment since.
5. **Probabilities**: only on non-self edges, with basis + vintage in
   `note`. Unauthored beats invented.
6. **Pin modes when comparing to v1** (`mixed: optimistic`,
   `prob_fill: uniform`); see tests/parity.rs.
7. **Composed graphs**: set `scenario.start` explicitly; the default start
   is the first pack's start (alphabetical when `packs` is empty).
8. **Calibrate, don't guess.** Before hand-typing a probability/duration
   teaching estimate, check whether a `calibration` set (`lit/*/calibration-*.lit`) already has
   a sourced value, or whether `{"op":"calibration"}` says this input is worth
   calibrating next. See `docs/CALIBRATION.md`.
9. **Soft probabilities are uncertain, say how much.** Matter-specific
   evidence ("this judge granted 3 of 4") goes in `scenario.observe`, not in
   `probabilities`. Before recommending a line that rests on soft
   probabilities, check `posterior` (P(option optimal), credible interval)
   or `objective: robust`'s `policy_changes`; before recommending paying for
   information, check `voi`'s EVSI against its cost. Report
   `prior-concentration-estimated` warnings: the prior strength there is a
   default estimate, not a sample size. See `docs/UNCERTAINTY.md`.

## Layout

```
crates/litigate-engine/src/
  model/      packs → compiled graph (schema, links, namespacing, continuations)
  expr/       custom-function language (lexer, parser)
  metrics.rs  built-in metrics/utilities/params (as expressions) + variable envs
  scenario/   Scenario → View (roles, masks, probabilities, node plans, belief, warnings)
  algo/       mdp (solve), chain, sim, paths (dijkstra/yen/pareto), sweep (+tornado), structure,
              robust, posterior, voi, dirichlet (uncertain probabilities)
  scenario/   Scenario → View (roles, masks, probabilities, node plans, warnings)
  algo/       mdp (solve), chain, sim, paths (dijkstra/yen/pareto), sweep (+tornado), structure,
              equilibrium (general-sum), cvar, settle (ZOPA, bargaining prices, stopping, Rule 68)
  api/        JSON request/response, describe, op dispatch
  lint.rs     content QA
  cite/       cite verification against L0 sources (normalize, corpus, fuzzy match)
crates/litigate-*/     the authoring language: base (diagnostics), syntax, sema, verify, lower,
                       import, cli (`litigate`, its `litigate mcp` stdio MCP server), lsp, wasm;
                       editors/ holds tree-sitter + editor modes
lit/                   .lit rule packages, one per forum (+ core stdlib, links, examples);
                       calibration-*.lit: calibration sets (real, sourced probabilities/durations)
cases/                 .case matters (top level → scenarios/; requests/: runnable requests, each
                       led by its `// litigate run ...` line, checked against litgraph-final)
packs/                 GENERATED forum packs, links.json, packs/clock/<pack>.json and
                       packs/calibration/<set>.json; only *-SOURCES-NOTES.md are hand-written
crates/litigate-engine/clock/ the shared clock library: lit/core lowered (rewritten by `just lit-sync`)
scenarios/             GENERATED named scenarios
sources/               vendored L0 primary-law text (see sources/PROVENANCE.md)
docs/                  ARCHITECTURE, PACK_SCHEMA, COST_FUNCTIONS, CRITIQUE, CALIBRATION, UNCERTAINTY
docs/                  ARCHITECTURE, PACK_SCHEMA, COST_FUNCTIONS, CRITIQUE, CALIBRATION, SETTLEMENT
tests/fixtures/ts-parity/   golden outputs from the original TS engine
tests/cases/           famous cases replayed through the packs (+ story snapshots)
tests/golden/          hand-checked 2-6 node graphs; tests/README.md explains the layers
```

Issue tracking: [GitHub Issues](https://github.com/davidawad/litgraph/issues)
on this repository.

## litigate (the authoring language)

`docs/litigate/DESIGN.md` is the root design and `docs/litigate/SPEC.md`
the normative spec: `.lit` rule modules and `.case` matters compile to this
engine's pack/links/scenario JSON. Read them before touching
`crates/litigate-*`, `lit/`, `cases/` or `editors/`. Rules:

1. Work only inside the paths your bead owns; shared contracts (AST, HIR,
   diagnostics, CLI envelope) change only through the bead that owns them.
2. No forum knowledge in `crates/*/src` (DESIGN §9). Forum facts live in
   `lit/` (and the packs built from it).
3. Every new crate follows the workspace lints, `deny.toml`, the 500-line
   file ceiling, and `just ci`. Add tests with the code, never after.
4. The flip (DESIGN §11) has happened, and the generated packs, links,
   clock fragments, scenarios and TypeScript bindings are build outputs
   (lg-flu.31), never committed: edit `lit/` or `cases/` and run `just
   lit-sync`. SPEC §9.4 lists exactly which files are generated, from what,
   and the legacy file names kept (`--pack-file` in
   `crates/litgraph-packs/build.args`).
   The epic is tracked in the local br store (prefix `lg`).
