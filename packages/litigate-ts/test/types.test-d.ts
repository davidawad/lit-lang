// Type-level tests: the generated types are as precise as the Rust schemas.
import { describe, expectTypeOf, it } from "vitest";
import type {
  Diagnostic,
  DiagnosticCode,
  Doc,
  ErrorResponse,
  LinkFile,
  Litgraph,
  LitigateEnvelope,
  NamedScenario,
  OkResponse,
  Op,
  OpArgs,
  OpName,
  OpOf,
  Pack,
  Request,
  Response,
  Scenario,
  SchemaKind,
  SourceMap,
  Validation,
  ValidateKind,
} from "../src/index.js";
import type { loadNode } from "../src/node.js";
import type { loadWeb } from "../src/web.js";

describe("generated types", () => {
  it("maps every kind to its document", () => {
    expectTypeOf<Doc<"pack">>().toEqualTypeOf<Pack>();
    expectTypeOf<Doc<"links">>().toEqualTypeOf<LinkFile>();
    expectTypeOf<Doc<"named-scenario">>().toEqualTypeOf<NamedScenario>();
    expectTypeOf<Doc<"litigate-envelope">>().toEqualTypeOf<LitigateEnvelope>();
    expectTypeOf<Doc<"source-map">>().toEqualTypeOf<SourceMap>();
    expectTypeOf<SchemaKind>().toEqualTypeOf<
      "request" | "response" | "scenario" | "named-scenario" | "pack" | "links" | "calibration" | "clock"
    >();
    expectTypeOf<ValidateKind>().not.toEqualTypeOf<SchemaKind>();
  });

  it("keeps free-form JSON unknown", () => {
    expectTypeOf<Response["result"]>().toEqualTypeOf<unknown>();
    expectTypeOf<LitigateEnvelope["result"]>().toEqualTypeOf<unknown>();
  });

  it("types the request's scenario forms", () => {
    expectTypeOf<"cofc-1498-patent-case">().toExtend<NonNullable<Request["scenario"]>>();
    expectTypeOf<Scenario>().toExtend<NonNullable<Request["scenario"]>>();
    expectTypeOf<{ extends: string; params: { rate: number } }>().toExtend<NonNullable<Request["scenario"]>>();
  });

  it("discriminates ops by tag", () => {
    expectTypeOf<OpName>().toExtend<string>();
    expectTypeOf<"solve" | "chain" | "simulate" | "settle">().toExtend<OpName>();
    expectTypeOf<"sim">().not.toExtend<OpName>();
    expectTypeOf<OpOf<"simulate">>().toExtend<Op>();
    expectTypeOf<OpArgs<"simulate">>().not.toHaveProperty("op");
  });

  it("closes strict documents", () => {
    // @ts-expect-error unknown request fields are rejected, as by serde
    const bad: Request = { op: { op: "solve" }, scenrio: "x" };
    expectTypeOf(bad).toEqualTypeOf<Request>();
  });

  it("narrows responses", () => {
    expectTypeOf<OkResponse["ok"]>().toEqualTypeOf<true>();
    expectTypeOf<ErrorResponse["error"]["code"]>().toEqualTypeOf<string>();
  });

  it("types litigate diagnostics with the closed code set", () => {
    expectTypeOf<Diagnostic["code"]>().toEqualTypeOf<DiagnosticCode>();
    expectTypeOf<"L3004">().toExtend<DiagnosticCode>();
    expectTypeOf<"L9999">().not.toExtend<DiagnosticCode>();
    expectTypeOf<Diagnostic["severity"]>().toEqualTypeOf<"error" | "warning" | "info" | "hint">();
  });
});

describe("client", () => {
  it("types every method", () => {
    expectTypeOf<Litgraph["run"]>().parameter(0).toEqualTypeOf<Request>();
    expectTypeOf<Litgraph["run"]>().returns.toEqualTypeOf<Response>();
    expectTypeOf<Litgraph["validate"]>().returns.toEqualTypeOf<Validation>();
    expectTypeOf((lg: Litgraph) => lg.op("simulate", { runs: 10, seed: 1 })).returns.toEqualTypeOf<Response>();
    expectTypeOf<typeof loadNode>().returns.toEqualTypeOf<Litgraph>();
    expectTypeOf<typeof loadWeb>().returns.toEqualTypeOf<Promise<Litgraph>>();
  });

  it("rejects an unknown op name", () => {
    const op = (lg: Litgraph) =>
      // @ts-expect-error `sim` is not an op
      lg.op("sim", {});
    expectTypeOf(op).toBeFunction();
  });
});
