import { readFileSync } from "node:fs";
import { expect, it, vi } from "vitest";
import { type LitgraphWasm } from "../src/client.js";
import { loadNode } from "../src/node.js";
import { clientAfterInit, loadWeb, type WasmSource } from "../src/web.js";

it("loads the web build from bytes and agrees with the node build", async () => {
  const bytes = readFileSync(new URL("../wasm/web/litigate_wasm_bg.wasm", import.meta.url));
  const web = await loadWeb({ module_or_path: bytes });
  const request = JSON.stringify({ scenario: "cofc-bid-protest", op: { op: "solve" } });
  const strip = (json: string): unknown => ({ ...JSON.parse(json), elapsed_ms: 0 });
  expect(strip(web.runJson(request))).toStrictEqual(strip(loadNode().runJson(request)));
});

it("passes wasm bytes to the initializer and returns a client", async () => {
  const response = JSON.stringify({
    ok: true,
    api_version: 1,
    op: "describe",
    result: {},
    warnings: [],
    provenance: {},
    elapsed_ms: 1,
  });
  const wasm: LitgraphWasm = {
    handle: () => "{}",
    validate: () => "{}",
    schema: () => "{}",
    describe: () => response,
    litigate: () => "{}",
    litigate_library: () => "{}",
    litigate_cases: () => "{}",
  };
  const initialize = vi.fn(async (_source?: WasmSource) => undefined as never);
  const bytes = new Uint8Array([0, 97, 115, 109]);
  const web = await clientAfterInit(bytes, initialize, wasm);

  expect(initialize).toHaveBeenCalledWith(bytes);
  expect(web.describe()).toStrictEqual(JSON.parse(response));
});
