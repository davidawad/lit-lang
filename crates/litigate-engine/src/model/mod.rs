// SPDX-License-Identifier: GPL-3.0-or-later
//! Packs (JSON) and the compiled, composed [`Graph`].
//!
//! A pack is one forum's procedure. A graph is one or more packs, their
//! `links.json` instances and cross-pack links, compiled into dense index
//! space with every id namespaced `pack::local`. Algorithms only ever see the
//! compiled graph; they never touch JSON.
//!
//! Edges may also declare `sets`/`clears`/`requires`/`forbids` state flags
//! (v2): the internal `flags` module expands those into a product graph over
//! `(node, flag-set)` so a downstream node's identity can depend on history —
//! an IPR estoppel, a waived defense, a prior RCE — without every algorithm
//! needing to know that. See `docs/PACK_SCHEMA.md#state-flags`.

mod cite_syntax;
mod continuations;
mod entities;
mod flags;
mod graph;
mod links;
mod lowered;
mod resolve;
mod schema;

pub use cite_syntax::{CiteCode, CiteRuleForm, CiteShorthand, CiteSyntax};
pub use entities::{CompileOptions, Edge, Node, PackMeta, PayoffSource, DEFAULT_MAX_PRODUCT_NODES};
pub use graph::{heuristic_payoff, qualify, EdgeIx, Graph, NodeIx};
pub use links::{local_edge_ids, merge_patch, Instance, LinkFile, LINKS_SCHEMA_VERSION};
pub(crate) use lowered::check_header;
pub use lowered::{pointer_token, world_digest, Basis, BasisLabel, Hole, Lowered, Pin};
pub use schema::{
    default_roles, CauseStandard, DateBound, DateRef, Deadline, Designated, Duration,
    ExtensionBound, ExtensionWindow, Group, Motion, NodeKind, Pack, RawEdge, RawNode, Role, Source,
    PACK_SCHEMA_VERSION,
};
