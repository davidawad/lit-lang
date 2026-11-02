import { expect, it, vi } from "vitest";
import { type LitgraphWasm } from "../src/client.js";
import { clientAfterInit, type WasmSource } from "../src/web.js";

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
