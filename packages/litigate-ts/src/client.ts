// SPDX-License-Identifier: GPL-3.0-or-later
// A thin typed layer over the litigate-wasm bindings (the compiler and
// engine as WebAssembly): the engine's JSON request/response contract,
// with requests and responses typed by the generated schemas.
import type { ApiError, Request, Response, Validation } from "./generated/schemas.js";
import { Litigate, type LitigateWasm } from "./litigate.js";
import type { ErrorResponse, OkResponse, OpArgs, OpName, OpOf, SchemaKind, ValidateKind } from "./types.js";

/** The functions crates/litigate-wasm exports (wasm-bindgen output). */
export interface LitgraphWasm extends LitigateWasm {
  handle(request_json: string): string;
  validate(doc_json: string, kind?: string | null): string;
  schema(kind: string): string;
  describe(): string;
}

const exported = [
  "handle",
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
 * The engine over the packs and named scenarios the wasm compiles,
 * in-process, from its embedded `lit/` and `cases/` source (once per
 * instance).
 */
export class Litgraph {
  readonly #wasm: LitgraphWasm;
  /** The `litigate` CLI (check, build, run, op subcommands) over in-memory files, through the same bindings. */
  readonly litigate: Litigate;

  constructor(wasm: LitgraphWasm) {
    this.#wasm = wasm;
    this.litigate = new Litigate(wasm);
  }

  /** A raw request JSON string in, the response JSON string out, the engine's raw request interface. */
  runJson(request: string): string {
    return this.#wasm.handle(request);
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

  /** The machine-readable manual (`litigate describe`). */
  describe(): Response {
    return parse<Response>(this.#wasm.describe());
  }

  /** Validates a document; `kind` forces its kind, else it is detected. */
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
