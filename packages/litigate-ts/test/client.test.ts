import { readdirSync, readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";
import {
  isError,
  isLitgraphWasm,
  isOk,
  Litgraph,
  Litigate,
  LitgraphError,
  type LitgraphWasm,
  type LitigateRun,
  loadNode,
  nodeBindings,
  type Pack,
  packFiles,
  type Response,
  unwrap,
} from "../src/node.js";

const pack: Pack = {
  schemaVersion: 2,
  id: "t",
  title: "t",
  startNodeId: "a",
  nodes: [
    { id: "a", label: "a" },
    { id: "b", kind: "terminal", label: "b", payoff: 5 },
  ],
  edges: [{ from: "a", to: "b", label: "go", actor: "applicant", hours: 0 }],
};

/** Bindings that record what they were sent and answer with `reply`. */
function fake(reply: string): LitgraphWasm & { calls: string[][] } {
  const calls: string[][] = [];
  const answer = (name: string) => (...args: (string | null | undefined)[]) => {
    calls.push([name, ...args.map(String)]);
    return reply;
  };
  return {
    calls,
    handle: answer("handle"),
    handle_with_packs: answer("handle_with_packs"),
    validate: answer("validate"),
    schema: answer("schema"),
    describe: answer("describe"),
    litigate: (args: string, files: string, library: boolean) => {
      calls.push(["litigate", args, files, String(library)]);
      return reply;
    },
    litigate_library: answer("litigate_library"),
    litigate_cases: answer("litigate_cases"),
  };
}

const failed: Response = {
  ok: false,
  api_version: 1,
  op: "unknown",
  warnings: [],
  error: { code: "parse", message: "bad", hint: "see schema" },
  elapsed_ms: 0,
};

describe("bindings", () => {
  it("recognizes a litigate-wasm module", () => {
    expect(isLitgraphWasm(nodeBindings())).toBe(true);
    expect(nodeBindings()).toBe(nodeBindings());
    expect(isLitgraphWasm(null)).toBe(false);
    expect(isLitgraphWasm("handle")).toBe(false);
    expect(isLitgraphWasm({ handle: () => "" })).toBe(false);
  });
});

describe("Litgraph over fake bindings", () => {
  it("sends requests as JSON and routes inline packs to handle_with_packs", () => {
    const wasm = fake(JSON.stringify({ ...failed, ok: true, op: "solve", result: 1 }));
    const lg = new Litgraph(wasm);
    expect(unwrap(lg.solve("m", { links: false }))).toBe(1);
    lg.withPacks({ packs: { "t.json": pack } }).chain();
    lg.op("path", { from: "a", to: "b" }, { extends: "m", params: { rate: 1 } });
    expect(wasm.calls).toStrictEqual([
      ["handle", '{"links":false,"op":{"op":"solve"},"scenario":"m"}'],
      ["handle_with_packs", '{"op":{"op":"chain"}}', JSON.stringify({ "t.json": pack })],
      ["handle", '{"op":{"from":"a","to":"b","op":"path"},"scenario":{"extends":"m","params":{"rate":1}}}'],
    ]);
  });

  it("throws LitgraphError from validate and schema error envelopes", () => {
    const lg = new Litgraph(fake(JSON.stringify(failed)));
    expect(() => lg.validate({})).toThrow(LitgraphError);
    expect(() => lg.schema("pack")).toThrow("parse: bad");
  });

  it("returns describe as a response", () => {
    expect(new Litgraph(fake(JSON.stringify(failed))).describe()).toStrictEqual(failed);
  });
});

describe("Litigate over fake bindings", () => {
  const envelope = { ok: true, op: "build", result: {}, diagnostics: [], provenance: {}, elapsed_ms: 1 };
  const run: LitigateRun = { code: 0, stdout: `${JSON.stringify(envelope)}\n`, stderr: "", files: { "o/a.json": "{}" } };

  it("passes the CLI's flags and parses the envelope", () => {
    const wasm = fake(JSON.stringify(run));
    const lit = new Litgraph(wasm).litigate;
    const built = lit.build(["lit/frcp"], {
      files: { "lit/frcp/Lit.toml": "" },
      library: false,
      out: "o",
      packFiles: { cafc: "cafc-federal-circuit" },
      profile: "strict",
      asOf: "2025-01-01",
      knownAt: "2025-02-01",
      lib: ["lit", "more"],
    });
    expect(built).toStrictEqual({ code: 0, stderr: "", files: { "o/a.json": "{}" }, envelope });
    lit.check(["cases"]);
    expect(wasm.calls).toStrictEqual([
      [
        "litigate",
        JSON.stringify([
          "--format", "json", "build", "--out", "o", "--pack-file", "cafc=cafc-federal-circuit",
          "--profile", "strict", "--as-of", "2025-01-01", "--known-at", "2025-02-01", "--lib", "lit", "--lib", "more",
          "lit/frcp",
        ]),
        JSON.stringify({ "lit/frcp/Lit.toml": "" }),
        "false",
      ],
      ["litigate", JSON.stringify(["--format", "json", "check", "cases"]), "{}", "true"],
    ]);
  });

  it("leaves the envelope out of a usage failure", () => {
    const failure: LitigateRun = { code: 1, stdout: "", stderr: "litigate check: x: no such file or directory\n", files: {} };
    const result = new Litigate(fake(JSON.stringify(failure))).check(["x"]);
    expect(result).toStrictEqual({ code: 1, stderr: failure.stderr, files: {} });
  });
});

describe("responses", () => {
  it("narrows and unwraps", () => {
    expect(isOk(failed)).toBe(false);
    expect(isError(failed)).toBe(true);
    expect(isError({ ...failed, error: null })).toBe(false);
    const error = new LitgraphError(failed);
    expect([error.name, error.code, error.hint, error.message]).toStrictEqual([
      "LitgraphError",
      "parse",
      "see schema",
      "parse: bad",
    ]);
    expect(error.response).toBe(failed);
    expect(() => unwrap(failed)).toThrow(LitgraphError);
    const bare = new LitgraphError({ ...failed, error: null });
    expect(bare.code).toBe("unknown");
  });
});

describe("packFiles", () => {
  it("names links.json and clock/ fragments as the catalog expects", () => {
    const files = packFiles({ packs: { "t.json": pack }, links: { links: [] }, clock: { "a.json": { schemaVersion: 1 } } });
    expect(Object.keys(files)).toStrictEqual(["t.json", "links.json", "clock/a.json"]);
  });
});

describe("Litgraph over the real wasm", () => {
  const lg = loadNode();

  it("solves an inline pack", () => {
    const r = loadNode({ packs: { "t.json": pack } }).solve();
    expect(r.ok).toBe(true);
    expect(unwrap(r)).toMatchObject({ value: 5 });
  });

  it("reports failures in the response, as the CLI does", () => {
    const r = lg.runJson("not json");
    expect(JSON.parse(r)).toMatchObject({ ok: false, error: { code: "parse" } });
    const bad = lg.run({ op: { op: "graph", node: "no-such-node" }, packs: ["frcp-civil-procedure"] });
    expect(isError(bad)).toBe(true);
  });

  it("describes and validates", () => {
    expect(lg.describe()).toMatchObject({ ok: true, op: "describe" });
    expect(lg.validate({ params: { rate: 1 } }, "scenario")).toMatchObject({ kind: "scenario", valid: true });
    expect(lg.validate({ nope: 1 }, "request").valid).toBe(false);
  });

  // The generated bundle the types are generated from must be exactly
  // what the wasm build publishes for every litgraph kind.
  it("publishes the schemas the types were generated from", () => {
    const bundle = JSON.parse(readFileSync(new URL("../schema/litgraph.schema.json", import.meta.url), "utf8")) as {
      properties: Record<string, { $ref: string }>;
      $defs: Record<string, unknown>;
    };
    const kinds = ["request", "response", "scenario", "named-scenario", "pack", "links", "calibration", "clock"] as const;
    for (const kind of kinds) {
      const { $schema: _s, $defs: _d, title: _t, ...published } = lg.schema(kind);
      const name = bundle.properties[kind]?.$ref.replace("#/$defs/", "") ?? "";
      expect(published, kind).toStrictEqual(bundle.$defs[name]);
    }
  });
});

describe("source hygiene", () => {
  // `any` (explicit or as a cast) is banned; free-form JSON is `unknown`.
  it("has no `any` in src", () => {
    const dir = new URL("../src/", import.meta.url);
    const files = readdirSync(dir, { recursive: true, encoding: "utf8" }).filter((f) => f.endsWith(".ts"));
    expect(files).toContain("generated/schemas.ts");
    for (const file of files) {
      const code = readFileSync(new URL(file, dir), "utf8")
        .replace(/\/\*[\s\S]*?\*\//g, "")
        .replace(/\/\/.*$/gm, "")
        .replace(/"(?:[^"\\\n]|\\.)*"|`(?:[^`\\]|\\.)*`/g, '""');
      expect(code, file).not.toMatch(/\bany\b/);
    }
  });
});
