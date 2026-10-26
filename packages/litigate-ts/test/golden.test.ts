// Every named scenario through the wasm client, whose packs and scenarios
// the wasm compiles in-process from the lit/ and cases/ source it embeds,
// must give the native `litigate engine` output (elapsed_ms aside) over
// the packs and scenarios `just lit-sync` (a `just ts-test` dependency)
// builds into target/lit-build/: same compiler, same engine, same contract.
import { spawnSync } from "node:child_process";
import { readdirSync, readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";
import { loadNode, type NamedScenario, type Request } from "../src/node.js";

const repo = fileURLToPath(new URL("../../../", import.meta.url));
const bin = process.env["LITIGATE_BIN"] ?? `${repo}target/debug/litigate`;
const build = ["--packs-dir", "target/lit-build/packs", "--scenarios-dir", "target/lit-build/scenarios"];
const scenarioDir = `${repo}target/lit-build/scenarios/`;
const files = readdirSync(scenarioDir).filter((f) => f.endsWith(".json")).sort();

function native(args: readonly string[]): unknown {
  const out = spawnSync(bin, ["engine", ...args, ...build, "--compact"], { cwd: repo, encoding: "utf8", maxBuffer: 1 << 30 });
  if (out.error !== undefined) throw new Error(`${bin}: ${out.error.message} (set LITIGATE_BIN or run \`just ts-test\`)`);
  if (out.status !== 0 && out.status !== 2) throw new Error(`${bin} ${args.join(" ")}: exit ${String(out.status)}: ${out.stderr}`);
  return withoutTiming(JSON.parse(out.stdout));
}

function withoutTiming(value: unknown): unknown {
  if (typeof value === "object" && value !== null && "elapsed_ms" in value) {
    const { elapsed_ms: _, ...rest } = value;
    return rest;
  }
  return value;
}

const lg = loadNode();
const ops = [{ op: "solve" }, { op: "chain" }, { op: "simulate", runs: 200, seed: 7 }] as const;

it("finds the scenario library", () => {
  expect(files.length).toBeGreaterThan(0);
});

describe.each(files)("%s", (file) => {
  const text = readFileSync(`${scenarioDir}${file}`, "utf8");
  const named = JSON.parse(text) as NamedScenario;
  const requests: [string, Request][] = ops.flatMap((op): [string, Request][] => [
    [`${op.op} by name`, { scenario: named.id, op }],
    [`${op.op} inline`, { packs: named.packs ?? [], scenario: named.scenario ?? null, op }],
  ]);

  it.each(requests)("%s matches the CLI", (_, request) => {
    const json = JSON.stringify(request);
    const wasm = withoutTiming(JSON.parse(lg.runJson(json)));
    expect(wasm).toMatchObject({ ok: true });
    expect(wasm).toStrictEqual(native(["q", json]));
  });

  it("validates as the CLI does", () => {
    expect(lg.validate(named)).toStrictEqual(native(["validate", `target/lit-build/scenarios/${file}`]));
  });
});
