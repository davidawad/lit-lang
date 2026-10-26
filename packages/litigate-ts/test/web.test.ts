import { readFileSync } from "node:fs";
import { expect, it } from "vitest";
import { loadNode } from "../src/node.js";
import { isError, loadWeb } from "../src/web.js";

it("loads the web build from bytes and agrees with the node build", async () => {
  const bytes = readFileSync(new URL("../wasm/web/litigate_wasm_bg.wasm", import.meta.url));
  const web = await loadWeb({ module_or_path: bytes });
  const request = JSON.stringify({ scenario: "cofc-bid-protest", op: { op: "solve" } });
  const strip = (json: string): unknown => ({ ...JSON.parse(json), elapsed_ms: 0 });
  expect(strip(web.runJson(request))).toStrictEqual(strip(loadNode().runJson(request)));
  // A second load reuses the initialized module; an empty pack set has nothing to run.
  const empty = await loadWeb(undefined, { packs: {} });
  const r = empty.solve();
  expect(isError(r) && r.error.message).toContain("no packs given");
});
