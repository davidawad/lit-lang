# @litgraph/litigate-ts

A typed TypeScript client for the litigate compiler and engine, compiled
to WebAssembly (`crates/litigate-wasm`). The wasm embeds the repository's
`lit/` and `cases/` as source and compiles them in-process (once per
instance) into the packs and named scenarios it serves. The JSON contract
is the one `litigate engine q` uses: the request goes in, `{ok,
api_version, op, result, warnings, provenance, elapsed_ms}` comes out, and
errors arrive as `ok: false` in the response rather than as exceptions.
See `AGENTS.md`.

```ts
import { loadNode, unwrap } from "@litgraph/litigate-ts/node"; // or "@litgraph/litigate-ts/web" + await loadWeb()

const lg = loadNode();                                   // the packs and scenario library compiled from lit/ and cases/
const r = lg.solve("cofc-1498-patent-case");             // Response
const sim = lg.op("simulate", { runs: 500, seed: 1 }, { extends: "frcp-civil-case", params: { rate: 900 } });
lg.validate(myPack, "pack");                             // Validation
unwrap(r);                                               // the result, or throws LitgraphError
```

## Types are generated from Rust

Every exported document type (`Request`, `Response`, `Op`, `Scenario`,
`NamedScenario`, `Pack`, `LinkFile`, `ClockLibrary`, `CalibrationSet`,
`Validation`, the litigate CLI's `LitigateEnvelope`, `LitigateRun` and `Diagnostic`, the
`*.map.json` `SourceMap`, ...) lives in `src/generated/schemas.ts`. None is
written by hand:

1. `crates/litgraph-schema-bundle` collects the `schemars` JSON Schema of
   each Rust type into one bundle, `schema/litgraph.schema.json` (also
   shipped, as `@litgraph/litigate-ts/schema.json`).
2. `scripts/gen.mjs` compiles the bundle with `json-schema-to-typescript`.

`just ts-gen` runs both. Neither file is committed (they are build
outputs, so parallel schema changes never conflict on them): `just
ts-build` and `just ts-test` regenerate them first. To change a type,
change the Rust type and rerun `just ts-gen`. `src/types.ts` only
derives names from the generated types (`OpOf<"simulate">`, `Doc<"pack">`).

`Response.result` and `provenance` are `unknown`, as they are
`serde_json::Value` in Rust: per-op result types do not exist on the Rust
side yet, so none exist here. Narrow them yourself, or add typed result
structs in Rust and the types will follow.

## Recipes

| recipe | does |
| --- | --- |
| `just ts-gen` | generate `schema/` and `src/generated/` (gitignored) |
| `just ts-build` | `just wasm`, copy it into `wasm/{node,web}`, `tsc` to `dist/` |
| `just ts-test` | `ts-build`, build the native `litigate`, then `tsc` + `vitest --typecheck` |

Tests: `test/client.test.ts` (unit, over fake and real bindings, plus a
check that `src/` has no `any`), `test/golden.test.ts` (every
named scenario, as `just lit-sync` builds it into `target/lit-build/`,
through wasm, by name and inline, for `solve`, `chain`
and a seeded `simulate`, plus `validate`, compared exactly with the native
`litigate engine` output over that build apart from `elapsed_ms`),
`test/litigate.test.ts` (`litigate check`/`build`/ops/`engine` in memory
against the native `litigate` CLI),
`test/web.test.ts` (the web build),
and `test/types.test-d.ts` (type-level, `expect-type` via vitest).

Toolchain: Node >= 20 (CI and development used Node 22), npm, the
`wasm32-unknown-unknown` Rust target, and `wasm-bindgen-cli` 0.2.127 (the
version `crates/litigate-wasm` pins). Dev dependencies are pinned in
`package-lock.json`.

## The litigate CLI in wasm

`lg.litigate` runs `litigate check`, `build`, `run`, the op subcommands
(`solve`, `chain`, ...), `engine` and the case boundary (`timeline`,
`calendar`, `actions`, `lint-filing`) over an in-memory file tree: the same
arguments, envelope (`LitigateEnvelope`) and exit code as the native CLI
over a disk holding those files, with `build`'s outputs handed back
instead of written. An op compiles the matter or package it names, and
the packages it uses, in-process, then runs the engine on the result.

```ts
const lit = loadNode().litigate;
const files = { "cases/my.case": myCase, "cases/Lit.lock": myLock }; // repository-relative path -> text
const checked = lit.check(["cases"], { files, profile: "strict" });   // LitigateResult
checked.code;                                // 0 ok, 2 an error diagnostic, 1 usage or I/O (envelope absent)
checked.envelope?.diagnostics;               // Diagnostic[]
const built = lit.build(["lit/frcp", "cases"], { files, out: "out" });
built.files["out/packs/frcp-civil-procedure.json"];  // every file the CLI would write under --out
lit.library();                               // the embedded lit/ library, by path
lit.cases();                                 // the embedded cases/ matters, by path
lit.run(["solve", "cases/my.case"], files);  // compile the matter, solve it: raw LitigateRun {code, stdout, stderr, files}
lit.run(["engine", "q", JSON.stringify(request)], files);  // a raw engine request over files' compiled lit/ and cases/
```

### The verified case boundary

DESIGN §10's shared boundary, the verified matter, runs in wasm as well,
through the same commands as the native CLI. Nothing here adds a rule:
each method only passes the CLI's flags and parses its envelope.

```ts
const t = lit.timeline("cases/my.case", { files, today: "2026-10-27" });
t.timeline;                                  // Timeline: replayed history, deadlines and statuses (generated type)
lit.calendar("cases/my.case", { files, remind: [7], out: "my.ics" });  // open deadlines as .ics (envelope result; `out` comes back in files)
lit.actions("cases/my.case", "defendant", { files });                  // the marking and the moves replay accepts for the party
lit.lintFiling("cases/my.case", { event: "Answered(complaint)", date: "2026-11-02", party: { by: "defendant" } }, { files });
```

`timeline` carries the generated `Timeline`; `calendar`, `actions` and
`lintFiling` results are `envelope.result: unknown` until their Rust
result types derive `JsonSchema`.

- The repository's `lit/` library and `cases/` matters are embedded in
  the wasm (by `crates/litigate-wasm/build.rs`); the library fills in
  every `lit/` path `files` does not give, so a matter's `uses` resolve.
  Pass `library: false` to check only what you pass.
- `check` judges deadlines at `today` (default: today's UTC date), the
  same path the native binary runs at its `LITIGATE_TODAY` or system
  date: a matter whose `as_of` is older has each deadline past due by
  then lapse, with an L4016 warning. Pass `today` (the fourth argument of
  `run`) to pin it; an explicit `asOf` turns it off.
- Only `check`, `build`, `worlds` (alternate timelines,
  docs/litigate/CHANGE.md §5.1), `run`, the op subcommands, `engine` and
  the case boundary (`timeline`, `calendar`, `actions`, `lint-filing`)
  run; any other command exits 1. `engine` serves the packs and matters compiled
  from `files` (`--lib`, `--cases`); a directory flag (`--packs-dir`,
  ...) exits 1, as there is no disk.
- On the Rust side the loaders read a `litigate_sema::pkg::source::Source`:
  `Disk` for the CLI, `MemoryTree` here (`litigate_cli::execute_in_memory`).
- `test/litigate.test.ts` checks, builds and runs ops on the repository's
  `lit/` and `cases/` through the wasm and compares with the native
  `litigate` CLI (`LITIGATE_BIN`), envelope and written files alike.
- Embedding the vendored source corpus (`sources/`, needed so cite checks
  agree with the CLI), `lit/` and `cases/` makes the wasm larger (the
  release build is about 20 MB).
