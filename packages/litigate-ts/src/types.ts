// SPDX-License-Identifier: GPL-3.0-or-later
// Names derived from the generated types; no shape is declared here.
import type { LitgraphSchemas, Op, Response } from "./generated/schemas.js";

export type * from "./generated/schemas.js";

/** Every document kind in the schema bundle. */
export type DocumentKind = keyof LitgraphSchemas;

/** The document of a kind: `Doc<"pack">` is `Pack`. */
export type Doc<K extends DocumentKind> = LitgraphSchemas[K];

/** Kinds litgraph itself publishes a schema for (`litgraph schema <kind>`). */
export type SchemaKind = Exclude<DocumentKind, "validation" | "litigate-envelope" | "source-map">;

/** Kinds `validate` accepts as a forced kind. */
export type ValidateKind = Exclude<SchemaKind, "response">;

/** An op's tag: `"solve"`, `"chain"`, ... */
export type OpName = Op["op"];

/** The op variant with tag `N`: `OpOf<"sim">` carries `runs`, `seed`, ... */
export type OpOf<N extends OpName> = Extract<Op, { op: N }>;

/** An op's arguments without its tag. */
export type OpArgs<N extends OpName> = Omit<OpOf<N>, "op">;

/** A successful response: `result` is present and `error` absent. */
export type OkResponse = Response & { ok: true; error?: null };

/** A failed response: `error` says why. */
export type ErrorResponse = Response & { ok: false; error: NonNullable<Response["error"]> };
