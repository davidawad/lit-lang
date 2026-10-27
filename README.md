# litgraph

**A litigation-procedure graph engine.** litgraph models a court/agency
forum's procedure (FRCP, FRAP, Federal Circuit, Court of Federal Claims,
PTAB, ITC § 337, USPTO prosecution, FRCrimP) as a directed graph, composes
any set of forums into one graph, and turns that into a **stochastic game**:
what should the protagonist do at each step, what will it cost, how long
will it take, how likely is each ending, what does the answer depend on
most, and what changes under any counterfactual (a waived defense, a
different judge, a changed rate, a settlement offer)?

Every number an algorithm consumes — cost, utility, probability transform,
path weight, Pareto objective, fee-shift eligibility — is a small,
inspectable **expression**, evaluated per edge or terminal. Built-in metrics
(`dollars`, `elapsed`, `surprise`, `ev`, ...) are just named expressions, so
you can read exactly what "cost" means and write your own.

> **Not legal advice.** litgraph's probabilities are teaching estimates
> (vintage noted in the pack, where authored at all) and most payoffs are
> placeholders, not real damages figures. Every response reports which
> numbers are authored law/data vs. engine fallbacks (`warnings`,
> `provenance.modes`) — read them before trusting a number. This project
> does not represent anyone, does not predict real case outcomes, and is
> not a substitute for counsel.

## Who it's for

- **Litigators and law students** who want to reason quantitatively about
  procedure: "is this appeal worth briefing at this stake?", "what does the
  § 1500 trap actually cost us?", "which of these three deadlines is the
  one to protect first?"
- **Agents** (LLM-driven tools) that need a machine-checkable model of court
  procedure to answer a specific matter's questions, with every assumption
  reported back instead of hidden in prose.
- **Builders of litigation-adjacent tools** who want a solved, sourced graph
  of a forum's procedure as a library, not a document to re-parse.

## What it does

| capability | op |
|---|---|
| best move at a node, with regret vs. every alternative | `explain` |
| value + best line for the whole matter | `solve` |
| absorption probabilities + expected cost/time under the optimal policy | `chain` |
| outcome distribution, CVaR, P(loss) via Monte Carlo | `simulate` |
| shortest / k-shortest lines to a node, a set of terminals, or an outcome tag | `path` |
| cost × time × likelihood frontier | `pareto` |
| value curve + policy breakpoints as any parameter varies | `sweep` |
| which few inputs actually drive the answer | `tornado` |
| SCCs, dominators, min-cut, betweenness, reachability | `structure` |
| settlement range (ZOPA), Nash / Rubinstein / midpoint price, when to settle, Rule 68 offers | `settle` |
| base scenario vs. a counterfactual, with deltas | `compare` |
| several ops against one compiled graph | `batch` |
| content QA: unsourced packs, missing payoffs, dead ends, unverifiable cites | `lint` |
| resolve packs + scenario without running an analysis | `validate` |

Every response is `{ok, api_version, op, result, warnings, provenance,
elapsed_ms}` or `{ok:false, api_version, op, error:{code, message, hint},
elapsed_ms}`. `warnings` is grouped by code (`{code, count, example, at}`)
rather than one entry per occurrence. `litgraph describe` is the full
machine-readable manual: every op, every scenario field, every built-in
metric with its source expression, every variable and function. Every
document kind (`request`, `response`, `scenario`, `named-scenario`, `pack`,
`links`) has a
JSON Schema via `litgraph schema <kind>`, and `litgraph validate <file|->`
checks any of them — including a pack you're authoring — without running
an analysis.

A **scenario library** (`scenarios/*.json`) ships with named matter profiles
(rates, stakes, perspective, payoffs, probabilities, policy, which packs to
load); `litgraph describe`'s `scenario_library` lists them. A request can
reference one by name (`{"scenario": "cofc-1498-patent-case", "op": {...}}`)
or compose it with inline overrides (`{"extends": "<id>", ...overrides}`,
deep-merged); the CLI's `--scenario <name|file|json>` and `--set` do the same.

## Quickstart

`litigate` is the one command line: it compiles a matter (`.case`) or a
rule package (`lit/<forum>`) and runs any engine op on the result.

```bash
cargo install --path crates/litigate-cli          # binary: litigate
litigate solve cases/cofc-1498-patent-case.case   # value + best line
litigate chain cases/cofc-1498-patent-case.case --set rate=900
litigate explain cases/frcp-civil-case.case --arg node=frcp-civil-procedure::pre-filing
litigate sweep cases/frcp-civil-case.case --arg param=rate --arg lo=100 --arg hi=1500
litigate deadlines lit/frcp --arg trigger=2025-01-06 --arg reachable=true  # a package: its pack
litigate solve lit/links                          # a links package: the graph it composes
litigate engine describe                          # the engine's raw interface: the manual
litigate engine q '{
  "packs": ["cofc", "cafc"],
  "scenario": {
    "params": { "rate": 850 },
    "payoffs": { "cofc::judgment-for-plaintiff": 4000000 },
    "objective": { "type": "cara", "a": 2e-7 },
    "fee_shift": { "fraction": 0.4 }
  },
  "op": { "op": "simulate", "runs": 50000 }
}'
```

Every op is a subcommand (`solve chain simulate path pareto sweep tornado
calibration posterior voi structure deadlines settle compare graph metric
lint`, plus `explain` given a path), and `litigate run TARGET --op '{...}'`
takes an op object. The `litgraph` binary is deprecated: it forwards to
`litigate engine` and will be removed.

Runnable requests live in [`cases/requests/`](cases/requests/), each a `.case`
matter whose first line is the `litigate run` that reproduces it — a 28
U.S.C. § 1498 patent case in the Court of Federal Claims through the Federal
Circuit:

| file | shows |
|---|---|
| `cofc-1498-chain.case` | expected cost/time/hours under the optimal policy |
| `cofc-1498-risk-averse-sim.case` | 50,000-run outcome simulation |
| `cofc-1498-tornado.case` | which inputs (rate, stakes) the answer is most sensitive to |
| `cofc-1500-trap-compare.case` | the § 1500 same-claim-pending trap as a counterfactual |
| `cofc-appeal-threshold-sweep.case` | the stake at which an appeal becomes worth briefing |
| `cofc-explain-dispositive-fork.case` | `explain` at a dispositive-motion decision point |
| `cofc-1498-settle.case` | settlement range, predicted prices, when the surplus peaks, a Rule 68 offer ([docs/SETTLEMENT.md](docs/SETTLEMENT.md)) |

```bash
litigate run cases/requests/cofc-1498-chain.case --op '{"op":"chain","metrics":["dollars","elapsed","hours"]}'
```

## Install

```bash
# homebrew (macOS, Linux)
brew install davidawad/tap/litgraph

# from source
cargo install --git https://github.com/davidawad/litgraph litgraph-cli
cargo install --git https://github.com/davidawad/litgraph litgraph-mcp  # MCP server

# nix, no clone
nix run github:davidawad/litgraph -- describe

# container
docker run --rm ghcr.io/davidawad/litgraph describe

```

Prebuilt binaries (Linux x86_64 glibc/musl, macOS arm64/x86_64) and the
WebAssembly bundles are attached to each
[GitHub Release](https://github.com/davidawad/litgraph/releases) -- each
release tarball, the container image, and the nix flake all carry both
`litgraph` and `litgraph-mcp`. See [CONTRIBUTING.md](CONTRIBUTING.md) for
building from source with nix/devenv.

## Usage

### CLI

`litigate <op> TARGET` compiles a `.case` matter, a `.lit` file's
package, or a package directory and runs one engine op on it: `--arg k=v`
sets op arguments, `--set k=v` overrides scenario params (composed onto a
matter's own via `extends`), `--no-links` leaves the links packages out.
The result is the engine's response inside `litigate`'s envelope; a target
that fails its check is not run.

`litigate engine` is the engine's raw JSON interface over packs and named
scenarios, for debugging and tools: `litigate engine q '<json>'` or
`litigate engine q - < request.json` (equivalently `litigate engine run
request.json`) sends one request envelope and prints one response
envelope. Shorthand subcommands (`describe`, `packs`, `lint`, `explain
--packs ... --arg node=...`) build the envelope for you. Its packs and
scenario library are compiled from `lit/` and `cases/` (`--lib`,
`--cases`) unless `--packs-dir`/`$LITGRAPH_PACKS` or
`--scenarios-dir`/`$LITGRAPH_SCENARIOS` names a directory. `litigate
engine schema <kind>` prints a document's JSON Schema; `litigate engine
validate <file|->` checks a pack, `links.json`, scenario, or request file
(kind auto-detected, or forced with `--kind`) and exits `2` if it's
invalid. Exit codes throughout: `0` ok, `2` the request failed or the
document is invalid, `1` usage/I/O error. The deprecated `litgraph`
binary is `litigate engine` with the packs built in.

### Library

The engine is a Rust library crate (`litigate-engine`, in
[`crates/litigate-engine`](crates/litigate-engine)), internal to the
`litigate` compiler and not published; `litigate engine` (and the
deprecated `litgraph` shim over it) is a thin layer over
`litigate_engine::api::handle`. Add it as a path or git dependency to embed
the solver, the expression language, or the pack loader directly — nothing
about the CLI is required.

### MCP server

`litgraph-mcp` ([`crates/litgraph-mcp`](crates/litgraph-mcp)) is a stdio
[MCP](https://modelcontextprotocol.io) server: a thin wrapper over
`litigate_engine::api::handle`, same as the CLI. Every op is its own tool
(`solve`, `chain`, `explain`, ...), each with an input schema sliced
straight out of the engine's own JSON Schema for `Op` — plus one generic
`litgraph` tool that takes a raw request `{packs, links, no_continuations,
scenario, op}` verbatim. Packs, `links.json`, the named scenario library,
and the manual are MCP resources: `litgraph://packs/<id>`,
`litgraph://links`, `litgraph://scenarios` (an index of every named
scenario), `litgraph://scenarios/<id>` (one full matter profile), and
`litgraph://describe`. Every tool call returns the same JSON envelope as
the CLI; an engine error (`ok: false`) comes back as a tool error carrying
that envelope. `LITGRAPH_PACKS`/`--packs-dir` and
`LITGRAPH_SCENARIOS`/`--scenarios-dir` work exactly as they do for the CLI.
The `litigate_describe`, `litigate_check`, `litigate_build`,
`litigate_where`, `litigate_timeline`, `litigate_worlds` and `litigate_explain` tools drive the authoring language
the packs are generated from (see [Packs](#packs)): they run the `litigate`
toolchain as a library and return its envelope `{ok, op, result,
diagnostics, provenance, elapsed_ms}`, with paths relative to the server's
working directory (start it in a litgraph checkout to work on `lit/` and
`cases/`).

**Claude Code:**

```bash
claude mcp add litgraph -- litgraph-mcp
# with a packs directory instead of the embedded set:
claude mcp add litgraph -e LITGRAPH_PACKS=/path/to/packs -- litgraph-mcp
```

**Claude Desktop** — add to `claude_desktop_config.json`
(`~/Library/Application Support/Claude/claude_desktop_config.json` on
macOS; `%APPDATA%\Claude\claude_desktop_config.json` on Windows;
`~/.config/Claude/claude_desktop_config.json` on Linux):

```json
{
  "mcpServers": {
    "litgraph": {
      "command": "litgraph-mcp"
    }
  }
}
```

**Any other MCP client** that reads the same `mcpServers` stdio shape
(Cursor, Windsurf, Zed, ...) uses the same block; add `"args":
["--packs-dir", "/path/to/packs", "--scenarios-dir", "/path/to/scenarios"]`
or `"env": {"LITGRAPH_PACKS": "/path/to/packs", "LITGRAPH_SCENARIOS":
"/path/to/scenarios"}` to point at pack/scenario sets other than the ones
embedded in the binary (each falls back to its own embedded set
independently).

### As the engine behind an application

[civ-pro-the-gathering](https://github.com/davidawad/civ-pro-the-gathering)
is a litigation-procedure training game and the proof-of-concept front end
for this engine: litgraph was extracted from its original TypeScript graph
library, rewritten in Rust, and verified node-for-node against the original
implementation (`crates/litigate-engine/tests/parity.rs`, golden fixtures in
[`tests/fixtures/ts-parity/`](tests/fixtures/ts-parity/)) before any
semantic changes were made. The game supplies the packs and the UI; litgraph
supplies the graph, the solver, and the JSON contract.

## Packs

Forum procedures are written in **litigate**, a typed, dated, cited rule
language: one package per forum under [`lit/`](lit/) (`lit/frcp`,
`lit/cafc`, ...; `lit/core` holds shared counting rules, calendars and
types), compiled at build time (`crates/litgraph-packs`) into the pack
JSON the engine loads. That JSON is a build output, never committed;
`just lit-sync` writes a copy to `target/lit-build/packs/` to read.
[`docs/litigate/TUTORIAL.md`](docs/litigate/TUTORIAL.md) walks through
writing a rule, testing it, writing a case and solving it;
[`docs/litigate/SPEC.md`](docs/litigate/SPEC.md) is the language reference.

`links.json` (from `lit/links`) joins them into one composed graph (e.g. a CoFC judgment linking to a
Federal Circuit appeal). They're compiled into the `litgraph` binary at
build time, so `litgraph packs` works right after install with no
`packs/` directory needed — point at a different set with
`LITGRAPH_PACKS=<dir>` or `--packs-dir <dir>` (precedence: the flag, else
the variable, else the embedded set; `just lit-sync` writes the repo's
built packs to `target/lit-build/packs`). A flag or variable naming a
directory that does not exist, or holds no pack, is an error that names it,
never a silent fallback. Run `litgraph packs` for
live counts; schema is documented in
[`docs/PACK_SCHEMA.md`](docs/PACK_SCHEMA.md). A pack (or `links.json`) that
fails to parse is skipped, not fatal to the rest — `litgraph describe`'s
`pack_load_errors` names the file and the reason, and a request naming that
pack gets an ordinary "not found" rather than every op going down. The vendored L0 source text
`sources[].path` points into (see [`sources/PROVENANCE.md`](sources/PROVENANCE.md))
is embedded the same way, overridable with `LITGRAPH_SOURCES=<dir>`.

## Scenario library

Named matter profiles are JSON built from the `.case` matters in
[`cases/`](cases/) (`just lit-sync` writes them to
`target/lit-build/scenarios/`) (a matter's posture is derived by replaying its event
history against the rules), and compiled into the binary at build time; point at a different set with
`LITGRAPH_SCENARIOS=<dir>` or `--scenarios-dir <dir>`. Shape and request
composition (`{"scenario": "<id>"}` / `{"extends": "<id>", ...overrides}`)
are documented in [`docs/PACK_SCHEMA.md`](docs/PACK_SCHEMA.md)'s "Scenario
library" section.

| pack | forum | schema | sourced (`sources` entries) | authored payoffs |
|---|---|---|---|---|
| `cofc` | Court of Federal Claims | v2 | 17, url + sha256 + as-of | 12/12 terminals |
| `cafc` | Federal Circuit | v2 | 8, url + sha256 + as-of | 18/18 terminals |
| `frcp-civil-procedure` | Federal Rules of Civil Procedure | v1 | 1, url only (FRCP 12(h)) | heuristic fallback |
| `frap-appellate-procedure` | Federal Rules of Appellate Procedure | v1 | none yet | heuristic fallback |
| `frcrimp-criminal-procedure` | Federal Rules of Criminal Procedure | v1 | none yet | heuristic fallback |
| `itc-337` | ITC § 337 unfair-import investigations | v1 | none yet | heuristic fallback |
| `mpep-prosecution` | USPTO utility patent prosecution | v1 | 1, url only (37 CFR 1.114) | heuristic fallback |
| `ptab-patent-trial-appeal-board` | PTAB trial proceedings | v1 | 1, url only (35 U.S.C. § 315) | heuristic fallback |

v1 packs load and solve like any other pack — they carry authored
deadlines, hours, and (for most) probabilities — they just don't yet have
authored terminal payoffs or a `sources` list; the engine flags this on
every response (a grouped `payoff-not-authored` warning, plus an
unsourced-pack diagnostic from `lint`). Bringing a pack to v2 (sources +
authored payoffs) is exactly the kind of contribution described in
[CONTRIBUTING.md](CONTRIBUTING.md).

## Docs

- [AGENTS.md](AGENTS.md) — operating manual for agents (and anyone) driving
  the engine: commands, response shape, rules, module layout.
- [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) — the model (`.lit` source →
  litigatables/packs → graph → `.case` matter/scenario → view → analyses →
  answers → accretion), design commitments,
  the composition model, roadmap.
- [docs/PACK_SCHEMA.md](docs/PACK_SCHEMA.md) — the pack JSON schema (v1 to
  v3) the compiler emits and the engine reads, roles, links, authoring rules.
- [docs/litigate/TUTORIAL.md](docs/litigate/TUTORIAL.md) — the authoring
  language in four steps: write a rule, test it, write a case, run solve.
- [docs/litigate/DESIGN.md](docs/litigate/DESIGN.md) and
  [docs/litigate/SPEC.md](docs/litigate/SPEC.md) — the litigate design and
  its normative specification (grammar, types, checking, lowering, CLI).
- [docs/COST_FUNCTIONS.md](docs/COST_FUNCTIONS.md) — the catalog of
  cost/time/risk/outcome/strategic functions, what's built in vs. one
  expression away vs. needs new engine capability.
- [docs/CRITIQUE.md](docs/CRITIQUE.md) — what was wrong with the original
  (v1) engine, the bugs found and fixed, what's still a known modeling
  limit.
- [docs/UNCERTAINTY.md](docs/UNCERTAINTY.md) — probabilities as Dirichlet
  beliefs: Bayesian updates from observed outcomes, the robust solve,
  posterior credible intervals, and value of information (EVPI/EVSI).
- [CONTRIBUTING.md](CONTRIBUTING.md) — dev setup, quality gates, and the
  pack-authoring guide (every cite traceable to a source, "unauthored beats
  invented").

## License

GPL-3.0-or-later. See [LICENSE](LICENSE). Contributions are accepted under
the same license (inbound = outbound); see
[CONTRIBUTING.md](CONTRIBUTING.md) for the DCO sign-off convention.
