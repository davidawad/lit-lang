//! Serializable high-level IR shared by semantic analysis, verification, and
//! lowering (SPEC §11).
//!
//! HIR is plain data: every name is resolved, every literal is exact, and no
//! value is inferred. Local ids keep their declared kebab-case spelling;
//! cross-package references are fully qualified as `<pack-id>::<local-id>`.
//! Sema produces it, `litigate-verify` checks it, `litigate-lower` maps it
//! field-for-field onto the litgraph JSON schema.
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

mod calibration;
mod case;
pub(crate) use case::own_name;
mod expr;
mod filing;
mod graph;
mod kind;
mod logic;
pub mod spans;
mod temporal;
mod value;

pub use calibration::{CalibratedValue, Calibration, CalibrationEntry, CalibrationSource};
pub use case::{
    Analysis, Case, CaseSource, EngineExpr, EngineOption, Event, EventParty, Observation,
    OpponentModel, PackagePin, Param, Party, Perspective, Prior, Rate, RateUnit, Removal, Stake,
};
pub use expr::{BinaryOp, Expr, ExprKind, Pattern, UnaryOp};
pub use filing::{CertificateReq, Counts, Ecf, Exclusions, LengthBound, Limit, Measure, Verdict};
pub use graph::{
    Basis, DeadlineWindow, Edge, EdgeDeadline, EdgePatch, GuardOp, GuardOpKind, Instance, Link,
    Node, NodeKind, TriangularDuration, Valence,
};
pub use kind::{Cell, Effect, Family, Holding, Instrument, Kind, Row, Scope};
pub use logic::{
    Authority, DefaultRule, DelegatedOp, Delegation, EnumDecl, EventEffect, EventType, Exception,
    ExceptionKind, Expectation, Invalidation, Metadata, Recharacterization, Refinement, Relation,
    Rule, RuleKind, Test, TypeAlias,
};
pub use temporal::{
    Adoption, Anchor, Bound, BoundDates, Calendar, Capability, CauseStandard, Chain, CharacterKind,
    Characterization, Commenced, Counting, DateRef, Deadline, Designated, Direction, DueEnd,
    EndOfMonth, Extension, ExtensionWindow, Grant, Holiday, HolidayDate, Interval, Motion,
    Observance, RollOn, ServiceTime, ShortPeriod, Transition, TransitionArm, TriggerPolicy,
    Undesignated, SERVICE_METHODS,
};
pub use value::{Authored, DayUnit, Money, Rational, Type, Value};

/// One compiled package: manifest plus every module, lowered to one pack.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Package {
    /// Resolved `Lit.toml`.
    pub manifest: Manifest,
    /// SHA-256 source digest (SPEC §4), lowercase hex.
    pub digest: String,
    /// Modules sorted by canonical module name.
    pub modules: Vec<Module>,
    /// The single `start` node id of the package.
    pub start: String,
    /// Group declarations in source order (module order, then position).
    pub groups: Vec<Group>,
    /// The exact version of each package the dependency resolution
    /// selected (transitively, without the package itself; SPEC §4.2).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub resolved: BTreeMap<String, String>,
}

/// Resolved `Lit.toml` contents.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Manifest {
    /// Package name (`package.name`).
    pub name: String,
    /// Exact `SemVer` version.
    pub version: String,
    /// Language version; must be `1`.
    pub language: u32,
    /// Explicit legacy litgraph pack id (`package.pack_id`).
    pub pack_id: String,
    /// Forum key emitted as `Pack.forum`.
    pub forum: String,
    /// Default authority level of the package's declarations.
    pub authority: String,
    /// Pack title.
    pub title: String,
    /// Pack description.
    pub description: Option<String>,
    /// Pack jurisdiction.
    pub jurisdiction: Option<String>,
    /// The US state, DC or territory of a state-court pack
    /// (`package.jurisdiction_id`, a `litigate coverage` jurisdiction such
    /// as `alaska`); `None` for a federal pack.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub jurisdiction_id: Option<String>,
    /// Declared actors and their default litgraph roles (`[package.roles]`).
    pub roles: BTreeMap<String, ActorRole>,
    /// Default in-force interval and corpus date for every declaration.
    pub in_force: Interval,
    /// Primary-source bindings in declared order (`[sources.<id>]`).
    pub sources: Vec<SourceBinding>,
    /// Dependency name to `SemVer` requirement.
    pub dependencies: BTreeMap<String, String>,
    /// Counting-rule id (`package.counting`, litgraph `Pack.counting`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub counting: Option<String>,
    /// Dependencies (package names) whose periods counted by their own
    /// package's `counting` rule are counted by this package's instead in a
    /// matter that uses it (`package.recounts`, litgraph `Pack.recounts`):
    /// a court whose rules close a forward-counted period on its state's
    /// holidays (FRBP 9006(a)(6)(C)) counts the national rule set's
    /// periods with them.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub recounts: Vec<String>,
    /// Citation syntax (`[[cite_syntax.*]]`, litgraph `Pack.citeSyntax`).
    #[serde(default, skip_serializing_if = "CiteSyntax::is_empty")]
    pub cite_syntax: CiteSyntax,
    /// True for a links package's manifest (SPEC §4.1, §8.4): it has no
    /// pack identity, so `pack_id`, `forum`, `authority`, and `title` are
    /// empty and `roles` and `sources` are empty.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub links: bool,
    /// The instrument class (`package.kind`, SPEC §4.1): the index that
    /// decides which effects the package's declarations may have (§5.7).
    #[serde(default, skip_serializing_if = "Kind::is_base")]
    pub kind: Kind,
    /// The instrument justifying a class other than `base`
    /// (`[package.instrument]`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub instrument: Option<Instrument>,
    /// The forum-family class matrix (`[family.*]`), which only `core`
    /// declares (docs/litigate/CHANGE.md §3).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub families: Vec<Family>,
    /// Which law the package states (`package.law`, SPEC §4.1): a court's
    /// procedure, or a sovereign's substantive law, which a matter's
    /// `governing_law` composes with its forum's procedure (SPEC §5.6).
    #[serde(default, skip_serializing_if = "Law::is_procedure")]
    pub law: Law,
}

/// What a package's law governs (SPEC §4.1, §5.6): under Erie R. Co. v.
/// Tompkins, 304 U.S. 64 (1938), a federal court applies its own
/// procedure and the governing state's substantive law, and a statute of
/// limitations is substantive for that purpose (Guaranty Trust Co. v.
/// York, 326 U.S. 99 (1945)).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Law {
    /// A court's rules of procedure: the forum's govern.
    #[default]
    Procedure,
    /// A sovereign's substantive law: the governing law's govern.
    Substantive,
}

impl Law {
    /// Whether this is [`Law::Procedure`].
    #[must_use]
    pub fn is_procedure(&self) -> bool {
        *self == Self::Procedure
    }
}

/// `[[cite_syntax.*]]` tables in declared order.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CiteSyntax {
    /// `[[cite_syntax.rules]]`.
    pub rules: Vec<CiteRule>,
    /// `[[cite_syntax.shorthands]]`.
    pub shorthands: Vec<CiteShorthand>,
}

impl CiteSyntax {
    /// No rules and no shorthands.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.rules.is_empty() && self.shorthands.is_empty()
    }
}

/// One `[[cite_syntax.rules]]` entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CiteRule {
    /// Citation prefixes (`FRCP`, `Fed. R. Civ. P.`).
    pub prefixes: Vec<String>,
    /// Heading template with exactly one `{}`.
    pub heading: String,
    /// Bound `[sources.*]` key.
    pub source: Option<String>,
}

/// One `[[cite_syntax.shorthands]]` entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CiteShorthand {
    /// Shorthand prefix (`cfr`).
    pub prefix: String,
    /// `usc` or `cfr`.
    pub code: String,
    /// Title number.
    pub title: u32,
    /// Dotted section parts.
    pub section_parts: u32,
    /// Bound `[sources.*]` key.
    pub source: Option<String>,
}

/// Game-theoretic reading of an actor (litgraph `Role`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActorRole {
    /// The protagonist (`"self"` in litgraph JSON).
    #[serde(rename = "self")]
    Protagonist,
    /// An adversary with its own choices.
    Opponent,
    /// Tribunal, agency, or chance.
    Nature,
}

/// A bound primary source (`Pack.sources[]`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceBinding {
    /// Source id referenced by cites.
    pub id: String,
    /// Document title.
    pub title: Option<String>,
    /// Public URL.
    pub url: Option<String>,
    /// Repository-relative vendored path.
    pub path: Option<String>,
    /// SHA-256 of the document retrieved from `url`: recorded provenance,
    /// never fetched or checked (the compiler is offline).
    pub sha256: Option<String>,
    /// SHA-256 of the vendored file at `path`: pins the expression its
    /// cites resolve against, verified by `cite-resolves` (SPEC §4.3).
    pub path_sha256: Option<String>,
    /// Version date of the document.
    pub as_of: Option<String>,
    /// Citation prefixes this binding resolves (`FRCP`, `35 U.S.C.`).
    pub cite_prefixes: Vec<String>,
    /// Heading template locating a pinpoint in `path`; `{n}` is the section.
    pub section_heading: Option<String>,
}

/// Node group (`Pack.groups[]`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Group {
    /// Group id.
    pub id: String,
    /// Display label.
    pub label: String,
}

/// One resolved rule module.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Module {
    /// Canonical dotted module name.
    pub name: String,
    /// Owning package name.
    pub package: String,
    /// Imports resolved to canonical module names.
    pub imports: Vec<String>,
    /// Declarations in source order.
    pub declarations: Vec<Declaration>,
}

/// Rule-language declaration represented in HIR.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "decl", rename_all = "snake_case")]
pub enum Declaration {
    /// Graph node of any kind with its outgoing edges.
    Node(Node),
    /// Cross-package link (only in a links package).
    Link(Box<Link>),
    /// Namespaced pack instance (only in a links package).
    Instance(Instance),
    /// Typed deadline rule.
    Deadline(Deadline),
    /// Date counting behavior.
    Counting(Counting),
    /// Holiday set.
    Calendar(Calendar),
    /// Duty, power, or liability wrapper.
    Rule(Rule),
    /// Authority-order declaration.
    Authority(Authority),
    /// Default logic declaration.
    Default(DefaultRule),
    /// Rebutting or undercutting exception.
    Exception(Exception),
    /// Cross-version transition declaration.
    Transition(Transition),
    /// Nominal type alias.
    TypeAlias(TypeAlias),
    /// Closed enum declaration.
    Enum(EnumDecl),
    /// Event constructor declaration.
    Event(EventType),
    /// Inline test block.
    Test(Test),
    /// Emergency capability grant.
    Grant(Grant),
    /// A paper kind's filing requirements (SPEC §3.7).
    Limit(Limit),
    /// A new characterization of another package's deadline.
    Recharacterize(Recharacterization),
    /// The withdrawal of another package's declaration.
    Invalidate(Invalidation),
    /// Calibration set.
    Calibration(Calibration),
}

#[cfg(test)]
mod tests;
