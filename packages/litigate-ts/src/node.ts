// SPDX-License-Identifier: GPL-3.0-or-later
// Node entry: loads the wasm-bindgen `nodejs` build synchronously.
import { createRequire } from "node:module";
import { isLitgraphWasm, Litgraph, type LitgraphWasm } from "./client.js";

export * from "./index.js";

let bindings: LitgraphWasm | undefined;

/** The litigate-wasm module (wasm/node), loaded once. */
export function nodeBindings(): LitgraphWasm {
  if (bindings === undefined) {
    const module: unknown = createRequire(import.meta.url)("../wasm/node/litigate_wasm.js");
    if (!isLitgraphWasm(module)) throw new Error("wasm/node/litigate_wasm.js is not a litigate-wasm build");
    bindings = module;
  }
  return bindings;
}

/** A client over the compiled embedded sources. */
export function loadNode(): Litgraph {
  return new Litgraph(nodeBindings());
}
