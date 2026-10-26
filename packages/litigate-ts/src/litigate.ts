// SPDX-License-Identifier: GPL-3.0-or-later
// The `litigate` CLI over an in-memory file tree, through litigate-wasm's
// `litigate` binding: `check`, `build`, `run`, the op subcommands and
// `engine`, with the CLI's arguments and envelope, files passed in (and
// `build`'s outputs handed back) instead of a disk.
import type { LitigateEnvelope, LitigateRun, Profile } from "./generated/schemas.js";

/** The litigate-wasm functions this module calls. */
export interface LitigateWasm {
  litigate(args_json: string, files_json: string, with_library: boolean): string;
  litigate_library(): string;
  litigate_cases(): string;
}

/** Files by repository-relative POSIX path (`lit/frcp/Lit.toml`, `cases/x.case`, ...) to text. */
export type Files = Readonly<Record<string, string>>;

/** The `litigate check` checking profile (`--profile`), as diagnostics carry it. */
export type { Profile };

/** Options shared by {@link Litigate.check} and {@link Litigate.build}. */
export interface CheckOptions {
  /** The file tree the paths resolve in. */
  readonly files?: Files;
  /**
   * Fill in every `lit/` path `files` does not give from the `lit/`
   * library embedded in the wasm (default `true`), so a matter's `uses`
   * resolve with no library passed.
   */
  readonly library?: boolean;
  /** `--profile` (the CLI's default is `dev`). */
  readonly profile?: Profile;
  /** `--as-of DATE`. */
  readonly asOf?: string;
  /** `--known-at DATE`. */
  readonly knownAt?: string;
  /** `--lib DIR` library directories (default `lit`). */
  readonly lib?: readonly string[];
}

/** {@link Litigate.build} options. */
export interface BuildOptions extends CheckOptions {
  /** `--out DIR`: the prefix of every output path (the CLI's default is `build`). */
  readonly out?: string;
  /** `--pack-file ID=STEM` mappings, pack id to file stem. */
  readonly packFiles?: Readonly<Record<string, string>>;
}

/** One check or build: the exit code, the parsed envelope and what was written. */
export interface LitigateResult {
  /** 0 ok, 2 an error diagnostic, 1 a usage or I/O failure (`envelope` then absent). */
  readonly code: number;
  /** The JSON envelope `litigate --format json` prints; absent on a usage or I/O failure. */
  readonly envelope?: LitigateEnvelope;
  /** The usage or I/O failure message, else empty. */
  readonly stderr: string;
  /** Files `build` wrote, by path (`OUT/packs/<id>.json`, ...); empty for `check`. */
  readonly files: Files;
}

/** The `litigate` CLI over in-memory files. */
export class Litigate {
  readonly #wasm: LitigateWasm;

  constructor(wasm: LitigateWasm) {
    this.#wasm = wasm;
  }

  /**
   * Runs `litigate ARGS` (no program name) over `files`, exactly as the
   * CLI over a disk holding them. `check`, `build`, `worlds` (alternate
   * timelines), `run`, the op subcommands (`solve`, `chain`, ...) and
   * `engine` run; any other command exits 1.
   */
  run(args: readonly string[], files: Files = {}, library = true): LitigateRun {
    const json = this.#wasm.litigate(JSON.stringify(args), JSON.stringify(files), library);
    return parse<LitigateRun>(json);
  }

  /** `litigate check PATHS`: typecheck and verify packages and matters. */
  check(paths: readonly string[], options: CheckOptions = {}): LitigateResult {
    return this.#envelope(["check", ...flags(options), ...paths], options);
  }

  /** `litigate build PATHS`: check, then lower; the outputs come back in `files`. */
  build(paths: readonly string[], options: BuildOptions = {}): LitigateResult {
    const out = options.out === undefined ? [] : ["--out", options.out];
    const names = Object.entries(options.packFiles ?? {}).flatMap(([id, stem]) => ["--pack-file", `${id}=${stem}`]);
    return this.#envelope(["build", ...out, ...names, ...flags(options), ...paths], options);
  }

  /** The `lit/` library embedded in the wasm, by path. */
  library(): Files {
    return parse<Record<string, string>>(this.#wasm.litigate_library());
  }

  /** The `cases/` matters embedded in the wasm, by path. */
  cases(): Files {
    return parse<Record<string, string>>(this.#wasm.litigate_cases());
  }

  #envelope(args: readonly string[], options: CheckOptions): LitigateResult {
    const run = this.run(["--format", "json", ...args], options.files, options.library);
    const files: Record<string, string> = {};
    for (const [path, text] of Object.entries(run.files)) if (text !== undefined) files[path] = text;
    const result = { code: run.code, stderr: run.stderr, files };
    return run.stdout === "" ? result : { ...result, envelope: parse<LitigateEnvelope>(run.stdout) };
  }
}

/** The CLI flags of `options` shared by check and build. */
function flags(options: CheckOptions): string[] {
  const out: string[] = [];
  if (options.profile !== undefined) out.push("--profile", options.profile);
  if (options.asOf !== undefined) out.push("--as-of", options.asOf);
  if (options.knownAt !== undefined) out.push("--known-at", options.knownAt);
  for (const dir of options.lib ?? []) out.push("--lib", dir);
  return out;
}

/** The binding's JSON, typed by the Rust type it was serialized from. */
function parse<T>(json: string): T {
  const value: unknown = JSON.parse(json);
  return value as T;
}
