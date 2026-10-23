// SPDX-License-Identifier: GPL-3.0-or-later
// Browser entry: initializes the wasm-bindgen `web` build.
import init, * as wasm from "../wasm/web/litgraph_wasm.js";
import { Litgraph, type PackSet } from "./client.js";

export * from "./index.js";

/** What `init` accepts: a URL, `Response`, bytes, or a compiled module. */
export type WasmSource = Parameters<typeof init>[0];

/** Initializes the wasm module (fetching `litgraph_wasm_bg.wasm` next to the JS by default) and returns a client. */
export async function loadWeb(source?: WasmSource, packs?: PackSet): Promise<Litgraph> {
  await init(source);
  return new Litgraph(wasm, packs);
}
