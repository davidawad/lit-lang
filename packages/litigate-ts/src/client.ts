// SPDX-License-Identifier: GPL-3.0-or-later
// A thin typed layer over the litigate-wasm bindings (the compiler and
// engine as WebAssembly): the same JSON request/response contract as
// `litigate engine q`, with requests and responses typed by the generated
// schemas.
import type {
  ApiError,
  ClockLibrary,
  LinkFile,
  Pack,
  Request,
  Response,
  Validation,
} from "./generated/schemas.js";
import { Litigate, type LitigateWasm } from "./litigate.js";
import type { ErrorResponse, OkResponse, OpArgs, OpName, OpOf, SchemaKind, ValidateKind } from "./types.js";

/** The functions crates/litigate-wasm exports (wasm-bindgen output). */
export interface LitgraphWasm extends LitigateWasm {
  handle(request_json: string): string;
  handle_with_packs(request_json: string, packs_json: string): string;
  validate(doc_json: string, kind?: string | null): string;
  schema(kind: string): string;
  describe(): string;
}

const exported = [
  "handle",
  "handle_with_packs",
  "validate",
  "schema",
  "describe",
  "litigate",
  "litigate_library",
  "litigate_cases",
] as const;

/** True when `module` exports every litigate-wasm function. */
export function isLitgraphWasm(module: unknown): module is LitgraphWasm {
  if (typeof module !== "object" || module === null) return false;
  return exported.every((name) => typeof Reflect.get(module, name) === "function");
}

/** Packs to run against instead of the compiled embedded ones. */
export interface PackSet {
  /** File name (`my-pack.json`) to pack. */
  readonly packs: Readonly<Record<string, Pack>>;
  /** The cross-pack links (`links.json`). */
  readonly links?: LinkFile;
  /** Clock-library fragments by file name; the embedded library when absent. */
  readonly clock?: Readonly<Record<string, ClockLibrary>>;
}

/** The `{file name: document}` object `handle_with_packs` takes. */
export function packFiles(set: PackSet): Record<string, Pack | LinkFile | ClockLibrary> {
  const files: Record<string, Pack | LinkFile | ClockLibrary> = { ...set.packs };
  if (set.links !== undefined) files["links.json"] = set.links;
  for (const [name, fragment] of Object.entries(set.clock ?? {})) files[`clock/${name}`] = fragment;
  return files;
}

/** A failed response, thrown by {@link unwrap} and the client's non-`run` helpers. */
export class LitgraphError extends Error {
  readonly code: string;
  readonly hint: string;
  readonly response: Response;

  constructor(response: Response) {
    const error: ApiError = response.error ?? { code: "unknown", message: "request failed", hint: "" };
    super(`${error.code}: ${error.message}`);
    this.name = "LitgraphError";
    this.code = error.code;
    this.hint = error.hint;
    this.response = response;
  }
}

/** Narrows a response to success. */
export function isOk(response: Response): response is OkResponse {
  return response.ok;
}

/** Narrows a response to failure. */
export function isError(response: Response): response is ErrorResponse {
  return !response.ok && response.error !== undefined && response.error !== null;
}

/** The result of a successful response; throws {@link LitgraphError} otherwise. */
export function unwrap(response: Response): unknown {
  if (!isOk(response)) throw new LitgraphError(response);
  return response.result;
}

/** Request fields other than the scenario and op. */
export type RequestOptions = Omit<Request, "scenario" | "op">;

/**
 * The engine over one set of packs: by default the packs and named
 * scenarios the wasm compiles, in-process, from its embedded `lit/` and
 * `cases/` source (once per instance), as `litigate engine` at the
 * repository root.
 */
export class Litgraph {
  readonly #wasm: LitgraphWasm;
  readonly #packs: string | undefined;
  /** The `litigate` CLI (check, build, run, op subcommands, engine) over in-memory files, through the same bindings. */
  readonly litigate: Litigate;

  constructor(wasm: LitgraphWasm, packs?: PackSet) {
    this.#wasm = wasm;
    this.litigate = new Litigate(wasm);
    this.#packs = packs === undefined ? undefined : JSON.stringify(packFiles(packs));
  }

  /** A client over the same bindings that runs against `packs` instead. */
  withPacks(packs: PackSet): Litgraph {
    return new Litgraph(this.#wasm, packs);
  }

  /** A raw request JSON string in, the response JSON string out, exactly as `litigate engine q`. */
  runJson(request: string): string {
    return this.#packs === undefined
      ? this.#wasm.handle(request)
      : this.#wasm.handle_with_packs(request, this.#packs);
  }

  /** Runs a request. Failures come back as `ok: false`, never thrown. */
  run(request: Request): Response {
    return parse<Response>(this.runJson(JSON.stringify(request)));
  }

  /** Runs `op` with `args` on `scenario` (a library name, inline scenario, or `{extends}`). */
  op<N extends OpName>(op: N, args: OpArgs<N>, scenario?: Request["scenario"], options?: RequestOptions): Response {
    // `args` is the variant minus its tag, so restoring the tag yields the variant.
    const request: Request = { ...options, op: { ...args, op } as OpOf<N> };
    if (scenario !== undefined) request.scenario = scenario;
    return this.run(request);
  }

  /** `solve`: the optimal policy and its value. */
  solve(scenario?: Request["scenario"], options?: RequestOptions): Response {
    return this.op("solve", {}, scenario, options);
  }

  /** `chain`: absorption probabilities, expected costs and durations. */
  chain(scenario?: Request["scenario"], options?: RequestOptions): Response {
    return this.op("chain", {}, scenario, options);
  }

  /** The machine-readable manual (`litigate engine describe`), always over the compiled embedded sources. */
  describe(): Response {
    return parse<Response>(this.#wasm.describe());
  }

  /** Validates a document; `kind` forces its kind, else it is detected. Compiled embedded sources only. */
  validate(document: unknown, kind?: ValidateKind): Validation {
    const out = parse<Validation | Response>(this.#wasm.validate(JSON.stringify(document), kind ?? null));
    if ("ok" in out) throw new LitgraphError(out);
    return out;
  }

  /** The JSON Schema (draft 2020-12) the engine publishes for `kind`. */
  schema(kind: SchemaKind): Readonly<Record<string, unknown>> {
    const json = this.#wasm.schema(kind);
    const out = parse<Readonly<Record<string, unknown>>>(json);
    if (out["ok"] === false) throw new LitgraphError(parse<Response>(json));
    return out;
  }
}

/**
 * The engine's JSON, typed by the Rust schema it was serialized from. The
 * bindings only ever return serde output of the named type, so this is the
 * one trusted boundary.
 */
function parse<T>(json: string): T {
  const value: unknown = JSON.parse(json);
  return value as T;
}
