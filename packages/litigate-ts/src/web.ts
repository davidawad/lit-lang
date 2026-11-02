// SPDX-License-Identifier: GPL-3.0-or-later
// Browser entry: initializes the wasm-bindgen `web` build.
import init, * as wasm from "../wasm/web/litigate_wasm.js";
import { Litgraph, type LitgraphWasm } from "./client.js";

export * from "./index.js";

/** What `init` accepts: a URL, `Response`, bytes, or a compiled module. */
export type WasmSource = Parameters<typeof init>[0];

/** @internal The initialized browser client construction, separate for tests from large WASM instantiation. */
export async function clientAfterInit(
  source: WasmSource | undefined,
  initialize: typeof init,
  bindings: LitgraphWasm,
): Promise<Litgraph> {
  await initialize(source);
  return new Litgraph(bindings);
}

/** Initializes the wasm module (fetching `litigate_wasm_bg.wasm` next to the JS by default) and returns a client. */
export async function loadWeb(source?: WasmSource): Promise<Litgraph> {
  return clientAfterInit(source, init, wasm);
}
