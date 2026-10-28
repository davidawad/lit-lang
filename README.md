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
rather than one entry per occurrence. `litigate run lit/cofc --op
describe` is the engine's full machine-readable manual: every op, every
scenario field, every built-in metric with its source expression, every
variable and function; `--op validate` resolves the packs and scenario
without running an analysis.

A **scenario library** of named matter profiles (rates, stakes,
perspective, payoffs, probabilities, policy, which packs to load) is
compiled from the `.case` matters in [`cases/`](cases/); the manual's
`scenario_library` lists them. A matter is run by its file (`litigate
solve cases/<id>.case`), and `--set` overrides its params.

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
litigate run cases/cofc-1498-patent-case.case --op '{"op":"simulate","runs":50000}'
```

Every op is a subcommand (`solve chain simulate path pareto sweep tornado
calibration posterior voi structure deadlines settle compare graph metric
lint`, plus `explain` given a path), and `litigate run TARGET --op '{...}'`
takes an op object. A matter's params, payoffs, objective and fee shift
are written in its `analysis` block, never sent as a raw JSON request.

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
cargo install --git https://github.com/davidawad/litgraph litigate-cli  # litigate (and `litigate mcp`)

# nix, no clone
nix run github:davidawad/litgraph -- describe

# container
docker run --rm ghcr.io/davidawad/litgraph describe

```

Prebuilt binaries (Linux x86_64 glibc/musl, macOS arm64/x86_64) and the
WebAssembly bundles are attached to each
[GitHub Release](https://github.com/davidawad/litgraph/releases) -- each
release tarball, the container image, and the nix flake carry the one
`litigate` binary (`litigate mcp` is its MCP server). See [CONTRIBUTING.md](CONTRIBUTING.md) for
building from source with nix/devenv.

## Usage

### CLI

`litigate <op> TARGET` compiles a `.case` matter, a `.lit` file's
package, or a package directory and runs one engine op on it: `--arg k=v`
sets op arguments, `--set k=v` overrides scenario params (composed onto a
matter's own via `extends`), `--no-links` leaves the links packages out.
The result is the engine's response inside `litigate`'s envelope; a target
that fails its check is not run.

`litigate run TARGET --op <name|json>` runs any op, its arguments in
the op object; `litigate run lit/cofc --op describe` prints the engine's
manual and `--op packs` its compiled packs. There is no raw JSON request
interface: packs and named scenarios are compiled from `lit/` and `cases/`
(`--lib`). Exit codes throughout: `0` ok, `2` the request failed or the
document is invalid, `1` usage/I/O error.

### Library

The engine is a Rust library crate (`litigate-engine`, in
[`crates/litigate-engine`](crates/litigate-engine)), internal to the
`litigate` compiler and not published; `litigate`'s ops are a thin layer
over `litigate_engine::api::handle`. Add it as a path or git dependency to embed
the solver, the expression language, or the pack loader directly — nothing
about the CLI is required.

### MCP server

`litigate mcp` ([`crates/litigate-cli/src/mcp`](crates/litigate-cli/src/mcp))
is a stdio [MCP](https://modelcontextprotocol.io) server. Every engine op
is its own tool (`solve`, `chain`, `explain`, ...) run on a compiled
matter or package, as `litigate run TARGET --op ...` runs it. The
library's compiled packs, links, named scenarios and the engine manual
are MCP resources: `litigate://packs/<id>`, `litigate://links`,
`litigate://scenarios` (an index of every named scenario),
`litigate://scenarios/<id>` (one full matter profile), and
`litigate://describe`. Every tool call returns the same JSON envelope as
the CLI; a failed call (an error diagnostic, an engine `ok: false`) comes
back as a tool error carrying that envelope.
The `litigate_describe`, `litigate_check`, `litigate_build`,
`litigate_where`, `litigate_timeline`, `litigate_actions` and `litigate_explain` tools drive the authoring language
the packs are generated from (see [Packs](#packs)): they run the `litigate`
toolchain as a library and return its envelope `{ok, op, result,
diagnostics, provenance, elapsed_ms}`, with paths relative to the server's
working directory (start it in a litgraph checkout to work on `lit/` and
`cases/`).

**Claude Code:**

```bash
claude mcp add litigate -- litigate mcp
```

**Claude Desktop** — add to `claude_desktop_config.json`
(`~/Library/Application Support/Claude/claude_desktop_config.json` on
macOS; `%APPDATA%\Claude\claude_desktop_config.json` on Windows;
`~/.config/Claude/claude_desktop_config.json` on Linux):

```json
{
  "mcpServers": {
    "litigate": {
      "command": "litigate",
      "args": ["mcp"]
    }
  }
}
```

**Any other MCP client** that reads the same `mcpServers` stdio shape
(Cursor, Windsurf, Zed, ...) uses the same block.

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
Federal Circuit appeal). `litigate` compiles them from the `.lit` library
(`--lib`, default `lit`) on each run; they come only from compiled `.lit`
source (`just lit-sync` writes the repo's built packs to
`target/lit-build/packs`). Run `litigate run lit/cofc --op packs` for
live counts; schema is documented in
[`docs/PACK_SCHEMA.md`](docs/PACK_SCHEMA.md). A pack (or `links.json`) that
fails to parse is skipped, not fatal to the rest — the manual's
`pack_load_errors` names the file and the reason, and a request naming that
pack gets an ordinary "not found" rather than every op going down. The vendored L0 source text
`sources[].path` points into (see [`sources/PROVENANCE.md`](sources/PROVENANCE.md))
is embedded the same way, overridable with `LITGRAPH_SOURCES=<dir>`.

## Scenario library

Named matter profiles are JSON built from the `.case` matters in
[`cases/`](cases/) (`just lit-sync` writes them to
`target/lit-build/scenarios/`) (a matter's posture is derived by replaying its event
history against the rules). Their shape is documented in [`docs/PACK_SCHEMA.md`](docs/PACK_SCHEMA.md)'s "Scenario
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
