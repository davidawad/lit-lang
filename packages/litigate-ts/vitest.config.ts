import { readFileSync } from "node:fs";
import { defineConfig } from "vitest/config";

// Gate-only quarantine (lg-zmqc.24): under LITIGATE_TS_GATE=1, skip every test named in
// gate-quarantine.txt, matched by its full name ("describe > it").
const quarantined = (): string[] =>
  readFileSync(new URL("./gate-quarantine.txt", import.meta.url), "utf8")
    .split("\n")
    .map((line) => /^# lg-zmqc\.\d+ expires=\S+ \| litigate-ts::\S+ (.+)$/.exec(line)?.[1])
    .filter((name): name is string => name !== undefined);

const escape = (s: string) => s.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");

const gateSkip = (names: string[]) =>
  names.length === 0 ? {} : { testNamePattern: new RegExp(`^(?!(?:${names.map(escape).join("|")})$)`) };

export default defineConfig({
  test: {
    include: ["test/**/*.test.ts"],
    testTimeout: 120_000,
    ...(process.env["LITIGATE_TS_GATE"] === "1" ? gateSkip(quarantined()) : {}),
    typecheck: {
      include: ["test/**/*.test-d.ts"],
      tsconfig: "./tsconfig.json",
    },
  },
});
