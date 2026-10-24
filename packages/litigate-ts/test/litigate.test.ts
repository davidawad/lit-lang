// `litigate check`/`build` through the wasm over the repository's lit/ and
// cases/ trees, in memory, must give the native `litigate` CLI's output over
// the disk (elapsed_ms aside), and `build` the files the CLI writes.
import { spawnSync } from "node:child_process";
import { readdirSync, readFileSync, rmSync, statSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";
import { type Files, type LitigateResult, loadNode } from "../src/node.js";

const repo = fileURLToPath(new URL("../../../", import.meta.url));
const bin = process.env["LITIGATE_BIN"] ?? `${repo}target/debug/litigate`;
const lit = loadNode().litigate;

/** Every file under `dir` (repository-relative), by repository-relative path. */
function tree(dir: string, base = repo): Record<string, string> {
  const out: Record<string, string> = {};
  for (const name of readdirSync(`${base}${dir}`)) {
    const path = `${dir}/${name}`;
    if (statSync(`${base}${path}`).isDirectory()) Object.assign(out, tree(path, base));
    else out[path] = readFileSync(`${base}${path}`, "utf8");
  }
  return out;
}

const files: Files = { ...tree("lit"), ...tree("cases") };

/** The result shape of a native run, its envelope's elapsed_ms zeroed. */
function native(args: readonly string[]): LitigateResult {
  const out = spawnSync(bin, ["--format", "json", ...args], { cwd: repo, encoding: "utf8", maxBuffer: 1 << 30 });
  if (out.error !== undefined) throw new Error(`${bin}: ${out.error.message} (set LITIGATE_BIN or run \`just ts-test\`)`);
  const result = { code: out.status ?? -1, stderr: out.stderr, files: {} };
  return out.stdout === "" ? result : { ...result, envelope: { ...JSON.parse(out.stdout), elapsed_ms: 0 } };
}

function untimed(result: LitigateResult): LitigateResult {
  return result.envelope === undefined ? result : { ...result, envelope: { ...result.envelope, elapsed_ms: 0 } };
}

describe("check matches the CLI", () => {
  it.each([
    ["everything", ["lit", "cases"], {}],
    ["everything, strict", ["lit", "cases"], { profile: "strict" }],
    ["one matter", ["cases/frcp-civil-case.case"], {}],
    ["a missing path", ["lit/nope"], {}],
  ] as const)("%s", (_, paths, options) => {
    const args = [...("profile" in options ? ["--profile", options.profile] : []), ...paths];
    const expected = native(["check", ...args]);
    expect(untimed(lit.check(paths, { ...options, files }))).toStrictEqual(expected);
  });

  it("finds a matter's packages in the embedded library", () => {
    const cases: Files = tree("cases");
    // Every matter but the one rejected by design (L4001, lg-flu.46).
    const accepted = Object.keys(cases).filter((p) => p.endsWith(".case") && p !== "cases/e2e-ttab-short.case");
    const expected = native(["check", ...accepted]);
    expect(expected.code).toBe(0);
    expect(untimed(lit.check(accepted, { files: cases }))).toStrictEqual(expected);
    expect(lit.library()["lit/frcp/Lit.toml"]).toBe(files["lit/frcp/Lit.toml"]);
    expect(lit.check(["cases"], { files: cases, library: false }).code).toBe(2);
  });
});

describe("build matches the CLI", () => {
  // A subset of the inputs and file names in crates/litgraph-packs/build.args.
  const inputs = ["lit/cafc", "lit/cofc", "lit/frap", "lit/frcp", "lit/links", "cases/frcp-civil-case.case"];
  const out = "target/litigate-ts-golden";

  it("prints the CLI's envelope and returns the files it writes", () => {
    rmSync(`${repo}${out}`, { recursive: true, force: true });
    const names = ["--pack-file", "cafc=cafc-federal-circuit", "--pack-file", "cofc=cofc-court-of-federal-claims"];
    const expected = native(["build", "--out", out, ...names, ...inputs]);
    expect(expected.code).toBe(0);
    const packFiles = { cafc: "cafc-federal-circuit", cofc: "cofc-court-of-federal-claims" };
    const built = lit.build(inputs, { files, out, packFiles });
    expect(untimed({ ...built, files: {} })).toStrictEqual(expected);
    expect(built.files).toStrictEqual(tree(out));
    expect(built.files[`${out}/packs/frcp-civil-procedure.json`]).toBe(
      readFileSync(`${repo}target/lit-build/packs/frcp-civil-procedure.json`, "utf8"),
    );
  });

  it("runs only check and build", () => {
    const run = lit.run(["explain", "L3004"]);
    expect(run).toStrictEqual({
      code: 1,
      stdout: "",
      stderr: "litigate explain: only check and build run over an in-memory source\n",
      files: {},
    });
  });
});
