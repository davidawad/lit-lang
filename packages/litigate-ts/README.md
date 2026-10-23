# @litgraph/litigate-ts

A typed TypeScript client for the litgraph engine, compiled to WebAssembly
(`crates/litgraph-wasm`). The JSON contract is the one `litgraph q` uses:
the request goes in, `{ok, api_version, op, result, warnings, provenance,
elapsed_ms}` comes out, and errors arrive as `ok: false` in the response
rather than as exceptions. See `AGENTS.md`.

```ts
import { loadNode, unwrap } from "@litgraph/litigate-ts/node"; // or "@litgraph/litigate-ts/web" + await loadWeb()

const lg = loadNode();                                   // the packs and scenario library built into the wasm
const r = lg.solve("cofc-1498-patent-case");             // Response
const sim = lg.op("simulate", { runs: 500, seed: 1 }, { extends: "frcp-civil-case", params: { rate: 900 } });
const mine = lg.withPacks({ packs: { "my.json": myPack }, links: myLinks }).chain();
lg.validate(myPack, "pack");                             // Validation
unwrap(r);                                               // the result, or throws LitgraphError
```

## Types are generated from Rust

Every exported document type (`Request`, `Response`, `Op`, `Scenario`,
`NamedScenario`, `Pack`, `LinkFile`, `ClockLibrary`, `CalibrationSet`,
`Validation`, the litigate CLI's `LitigateEnvelope` and `Diagnostic`, the
`*.map.json` `SourceMap`, ...) lives in `src/generated/schemas.ts`. None is
written by hand:

1. `crates/litgraph-schema-bundle` collects the `schemars` JSON Schema of
   each Rust type into one bundle, `schema/litgraph.schema.json` (also
   shipped, as `@litgraph/litigate-ts/schema.json`).
2. `scripts/gen.mjs` compiles the bundle with `json-schema-to-typescript`.

`just ts-gen` runs both; `just ts-gen-check` (part of `just ci` and the CI
`litigate-ts` job) fails when the committed files are stale. To change a
type, change the Rust type and rerun `just ts-gen`. `src/types.ts` only
derives names from the generated types (`OpOf<"simulate">`, `Doc<"pack">`).

`Response.result` and `provenance` are `unknown`, as they are
`serde_json::Value` in Rust: per-op result types do not exist on the Rust
side yet, so none exist here. Narrow them yourself, or add typed result
structs in Rust and the types will follow.

## Recipes

| recipe | does |
| --- | --- |
| `just ts-gen` | regenerate `schema/` and `src/generated/` |
| `just ts-gen-check` | regenerate and fail if anything changed |
| `just ts-build` | `just wasm`, copy it into `wasm/{node,web}`, `tsc` to `dist/` |
| `just ts-test` | `ts-build`, build the native `litgraph`, then `tsc` + `vitest --typecheck` |

Tests: `test/client.test.ts` (unit, over fake and real bindings, plus a
check that `src/` has no `any`), `test/golden.test.ts` (every
`scenarios/*.json` through wasm, by name and inline, for `solve`, `chain`
and a seeded `simulate`, plus `validate`, compared exactly with the native
CLI output apart from `elapsed_ms`), `test/web.test.ts` (the web build),
and `test/types.test-d.ts` (type-level, `expect-type` via vitest).

Toolchain: Node >= 20 (CI and development used Node 22), npm, the
`wasm32-unknown-unknown` Rust target, and `wasm-bindgen-cli` 0.2.127 (the
version `crates/litgraph-wasm` pins). Dev dependencies are pinned in
`package-lock.json`.

## litigate check/build in wasm: not yet

`litigate-cli` does compile for `wasm32-unknown-unknown` once `getrandom`
gets its `js` feature (as `litgraph-wasm` already does; it reaches
litigate-cli through `litgraph`'s `rand`). It is not exposed because it
cannot do anything useful there: every command loads its inputs from disk.

- `litigate_sema::pkg::load` reads package trees with `std::fs::read_dir` /
  `read`, and `Session::load` (`crates/litigate-cli/src/ws.rs`) resolves
  targets against `std::env::current_dir`, `canonicalize`s them, and reads
  `.case` files and `Lit.lock` files directly. On `wasm32-unknown-unknown`
  these calls fail at run time.
- The `core` stdlib and library packages (`lit/`) are found by walking the
  filesystem; nothing embeds them the way packs are embedded.
- `build` writes its output with `std::fs::write` and `rename`.

Exposing `check`/`build` needs an in-memory source set (files as
`(path, text)` pairs) threaded through `litigate_sema::load` and
`Session::load`, an embedded `lit/` library, and `build` returning its files
instead of writing them. The envelope and source-map types it would return
are already generated here (`LitigateEnvelope`, `SourceMap`).
