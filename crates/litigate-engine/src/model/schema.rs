// SPDX-License-Identifier: GPL-3.0-or-later
//! Pack file schema (v1 compatible, v2 additive). See `docs/PACK_SCHEMA.md`.
//!
//! Every struct rejects unknown fields: a misspelled key in a pack is an
//! error, not a silently ignored value.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use super::cite_syntax::CiteSyntax;
use super::lowered::{check_header, Lowered};
use crate::error::{Error, Result};

/// The newest pack `schemaVersion`: v4 plus designated deadline windows
/// and extension bounds (`Deadline.designated`, `Deadline.extensions`).
pub const PACK_SCHEMA_VERSION: u32 = 5;

/// Structural kind of a node.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum NodeKind {
    /// A stable procedural posture.
    State,
    /// A fork someone must decide.
    Decision,
    /// The matter ends here (for this pack).
    Terminal,
}

/// A window within which a transition must happen: a counted `length`,
/// or (v5) a `designated` due date each matter's notice or order sets.
/// Exactly one of the two is present.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Deadline {
    /// Length of the window in `unit`s; absent (v5) for a `designated`
    /// window.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub length: Option<f64>,
    /// `calendar` (default) or `court` days, or `week`, `month`, `year`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unit: Option<String>,
    /// (v5) The due date is the one the triggering notice or order
    /// designates, not a counted period (37 C.F.R. § 2.105(a)): the pack
    /// has no fixed window to give it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub designated: Option<Designated>,
    /// Whether extensions are available; absent is undeclared, never
    /// "not extendable".
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extendable: Option<bool>,
    /// Authority for extensions.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extension_authority: Option<String>,
    /// (v5) Each extension grant the rule declares, its bound a sum: empty
    /// is undeclared, never "not extendable"; only a `nonExtendable`
    /// bound says no order extends the window beyond its other grants.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub extensions: Vec<ExtensionBound>,
    /// Free-text note (traps, US-party variants, ...).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    /// The clock-library counting rule of this window when it is not the
    /// pack's `counting` (a FRAP 4 appeal period in a district-court pack
    /// is counted under FRAP 26, not FRCP 6): a period is counted by the
    /// rule of the forum whose rules set it, never borrowed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub counting: Option<String>,
}

impl Deadline {
    /// The window's length in days, for metrics and estimates: the length
    /// itself for calendar and court days, seven per week, and the mean
    /// Gregorian month (146097/4800 days) and year (365.2425 days) for
    /// months and years, whose exact length depends on the trigger date.
    /// `None` for a `designated` window, whose length is each matter's.
    #[must_use]
    pub fn days(&self) -> Option<f64> {
        Some(
            self.length?
                * match self.unit.as_deref() {
                    Some("week") => 7.0,
                    Some("month") => 146_097.0 / 4800.0,
                    Some("year") => 365.2425,
                    _ => 1.0,
                },
        )
    }

    /// Check the window's shape in a document of `version`, where
    /// designated windows and extension bounds arrive in `since`: exactly
    /// one of `length` and `designated`, and a designated window counted
    /// in calendar days with a whole-day least time.
    ///
    /// # Errors
    /// `Error::Parse` naming the edge `at` and what does not hold.
    pub fn check(&self, at: &str, version: u32, since: u32) -> Result<()> {
        let bad = |why: String| Err(Error::Parse(format!("{at}: `deadline`: {why}")));
        if version < since && (self.designated.is_some() || !self.extensions.is_empty()) {
            return bad(format!(
                "`designated` and `extensions` need schemaVersion {since}, found {version}"
            ));
        }
        match (&self.length, &self.designated) {
            (Some(_), Some(_)) => bad("give `length` or `designated`, not both".into()),
            (None, None) => bad("needs `length` (or, a designated window, `designated`)".into()),
            (None, Some(d)) => {
                if self.unit.as_deref().is_some_and(|u| u != "calendar") {
                    return bad("a designated window's least time is in calendar days".into());
                }
                if d.min_period < 0.0 || d.min_period.fract() != 0.0 || !d.min_period.is_finite() {
                    return bad(format!(
                        "`designated.minPeriod` {} is not a non-negative integer",
                        d.min_period
                    ));
                }
                Ok(())
            }
            (Some(_), None) => Ok(()),
        }
    }
}

/// (v5) A designated window: the triggering event carries the due date
/// (SPEC §5.3, `designated p`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Designated {
    /// The least time the rule allows, in calendar days from the
    /// trigger's date ("not less than thirty days from the mailing date
    /// of the notice"); `0` allows any date from the trigger's own.
    pub min_period: f64,
    /// The qualified id of the triggering event (the notice or order).
    pub source_event: String,
    /// The `Date` argument of `sourceEvent` that designates the due date.
    pub source_event_arg: String,
    /// Present when the rule names no start for its least time and
    /// measuring `minPeriod` from the trigger's date is this
    /// interpretation (37 C.F.R. § 2.106(b)(3)(iii)): a date short of it
    /// stands, with a warning. Absent: such a date is rejected.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub interpretation: Option<String>,
}

/// (v5) One extension grant of a deadline (SPEC §5.3 `extend`): its bound
/// is a sum, so a rule that states no numeric ceiling is not forced to
/// invent one (FRCP 6(b)).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema)]
#[serde(
    tag = "bound",
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum ExtensionBound {
    /// Actor `by` may extend to a date no later than `until`.
    Bounded {
        /// The actor that may extend.
        by: String,
        /// The latest date an extension may reach.
        until: DateBound,
        /// May be exercised after the due date as well as before.
        #[serde(default, skip_serializing_if = "std::ops::Not::not")]
        after_expiry: bool,
        /// The cause standard, if the rule states one.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        standard: Option<CauseStandard>,
        /// The motion the grant requires, if any.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        motion: Option<Motion>,
        /// The interpretation the grant rests on, when the rule reaches
        /// the window only as its author reads it.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        interpretation: Option<String>,
    },
    /// Actor `by` may extend to any later date under `standard`, within
    /// `window`, with no numeric ceiling (FRCP 6(b)(1)).
    Discretionary {
        /// The actor that may extend.
        by: String,
        /// The cause standard.
        standard: CauseStandard,
        /// When the grant may be exercised.
        window: ExtensionWindow,
        /// The motion the grant requires: always present after expiry;
        /// before it, the request whose date is the window's.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        motion: Option<Motion>,
        /// The interpretation the grant rests on, when the rule reaches
        /// the window only as its author reads it (FRCP 6(b) applied to
        /// a TTAB period through 37 C.F.R. § 2.116(a)).
        #[serde(default, skip_serializing_if = "Option::is_none")]
        interpretation: Option<String>,
    },
    /// Rule `authority` forbids extending the window beyond the
    /// deadline's other grants (FRCP 6(b)(2)).
    NonExtendable {
        /// The rule that forbids the extension.
        authority: String,
    },
}

/// The cause standard of an extension grant.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
pub enum CauseStandard {
    /// Good cause (FRCP 6(b)(1)(A)).
    GoodCause,
    /// Excusable neglect (FRCP 6(b)(1)(B)).
    ExcusableNeglect,
    /// Either (FRAP 4(a)(5)(A)(ii)).
    GoodCauseOrExcusableNeglect,
    /// Extraordinary circumstances, the exception to a timing requirement
    /// on the application (Sup. Ct. R. 13.5).
    ExtraordinaryCircumstances,
}

/// When a discretionary extension may be granted.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum ExtensionWindow {
    /// On or before the due date (FRCP 6(b)(1)(A)).
    BeforeExpiry,
    /// After the due date, on motion (FRCP 6(b)(1)(B)).
    AfterExpiryOnMotion,
}

/// The motion an extension grant requires.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Motion {
    /// The motion's event pattern, in source spelling with qualified ids
    /// (`ExtensionMoved(AnswerPeriod)`).
    pub event: String,
    /// The latest date the motion may be made.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub within: Option<DateBound>,
    /// The event pattern that, once it precedes the order, bars the grant
    /// (a denial that may not be renewed, Sup. Ct. R. 30.3).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unless: Option<String>,
}

/// A date bound: `max`/`min` over calendar-day offsets after or before
/// the prescribed or current due date or the granting order's date (FRAP
/// 4(a)(5)(C); Sup. Ct. R. 30.2).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub enum DateBound {
    /// `from` plus `days` calendar days.
    Offset {
        /// The date counted from.
        from: DateRef,
        /// Calendar days after it.
        days: u64,
    },
    /// `from` minus `days` calendar days.
    Before {
        /// The date counted from.
        from: DateRef,
        /// Calendar days before it.
        days: u64,
    },
    /// The latest of the operands.
    Max(Vec<DateBound>),
    /// The earliest of the operands.
    Min(Vec<DateBound>),
}

/// A date an extension bound counts from.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum DateRef {
    /// The window's counted (or designated) due date.
    PrescribedDue,
    /// The date of the order granting the extension.
    GrantOrderEntered,
    /// The window's due date as last extended (Sup. Ct. R. 13.5, "the
    /// date the petition is due").
    CurrentDue,
}

/// Expected elapsed calendar time of a transition, in days (triangular).
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Duration {
    /// Optimistic duration.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min: Option<f64>,
    /// Most likely duration.
    pub mode: f64,
    /// Pessimistic duration.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max: Option<f64>,
}

/// A group (visual/hierarchical cluster) of nodes.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Group {
    /// Group id referenced by `RawNode::group`.
    pub id: String,
    /// Display label.
    pub label: String,
}

/// A node as authored in a pack.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RawNode {
    /// Unique within the pack (kebab-case).
    pub id: String,
    /// Structural kind (default `state`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<NodeKind>,
    /// Human label.
    pub label: String,
    /// Primary legal citation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cite: Option<String>,
    /// Commentary (traps, basis for numbers).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    /// Group id.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group: Option<String>,
    /// `good` / `caution` / `bad` / `neutral`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub valence: Option<String>,
    /// A real filed example of this posture.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub court_listener_url: Option<String>,
    /// Terminal value in USD from the protagonist's perspective (v2).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub payoff: Option<f64>,
    /// Outcome tags for terminals, e.g. `win`, `fee-eligible` (v2).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub outcome: Vec<String>,
    /// Free-form node tags, e.g. `entry`, `router` (v2).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    /// Free numeric attributes visible to custom functions (v2).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub attrs: BTreeMap<String, f64>,
    /// Terminal payoff override keyed by flag name (v2): if the compiled
    /// product-graph copy of this node carries one of these flags, its
    /// payoff is this value instead of `payoff`. Author with mutually
    /// exclusive flags; if more than one matches, the first key in sorted
    /// order wins. See `docs/PACK_SCHEMA.md#state-flags`.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub payoff_by_flag: BTreeMap<String, f64>,
    /// Fee shifting at this terminal (v2): the fraction of our accumulated
    /// cost (the litigation hours and the fee motion's own) a fee award
    /// recovers when the matter ends here, as the governing statute sets it
    /// by default (e.g. `1` for a prevailing civil-rights plaintiff under
    /// 42 U.S.C. § 1988). A payoff is the gross award; this offsets the
    /// cost of getting it. In `[0, 1]`, terminals only; a scenario's
    /// `fee_shift` or `fee_shifts` overrides it. See
    /// `docs/COST_FUNCTIONS.md#fee-shifting`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fee_shift: Option<f64>,
}

fn default_actor() -> String {
    "either".into()
}

/// An edge as authored in a pack, or a cross-pack link in `links.json`.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RawEdge {
    /// Stable id; derived as `from->to#n` when absent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// Source node id (qualified `pack::node` in links).
    pub from: String,
    /// Target node id (qualified `pack::node` in links).
    pub to: String,
    /// Human label (the move or ruling).
    pub label: String,
    /// `applicant` / `examiner` / `office` / `either`, read through `roles`.
    #[serde(default = "default_actor")]
    pub actor: String,
    /// Rule or statute authorizing the transition.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub authority: Option<String>,
    /// Window within which the transition must occur.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deadline: Option<Deadline>,
    /// Expected elapsed time.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration: Option<Duration>,
    /// Cash fees in USD.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cost: Option<f64>,
    /// Attorney hours.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hours: Option<f64>,
    /// Probability (world edges only).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub probability: Option<f64>,
    /// `good` / `caution` / `bad` / `neutral`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub valence: Option<String>,
    /// A real filed example of this transition.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub court_listener_url: Option<String>,
    /// Cross-reference to a rulepack action.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub action_id: Option<String>,
    /// Commentary (basis for numbers, traps).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    /// Machine tags, e.g. `waiver-trap`, `appeal` (v2).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    /// Free numeric attributes visible to custom functions (v2).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub attrs: BTreeMap<String, f64>,
    /// Links only: qualified edge ids this link supersedes when it applies.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub replaces: Vec<String>,
    /// State flags this edge sets when taken (v2). Compiled into a product
    /// graph over `(node, flag-set)`; see `docs/PACK_SCHEMA.md#state-flags`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sets: Vec<String>,
    /// State flags this edge clears when taken (v2).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub clears: Vec<String>,
    /// Flags that must ALL be set for this edge to exist in the compiled
    /// graph (v2). A flag no edge ever sets makes a `requires` on it always
    /// fail.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub requires: Vec<String>,
    /// Flags that must ALL be absent for this edge to exist in the compiled
    /// graph (v2).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub forbids: Vec<String>,
}

/// A primary source a pack was authored from.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Source {
    /// Short id referenced from notes.
    pub id: String,
    /// Document title.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Official URL.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// Repo-relative path (e.g. a sibling pack). Never an absolute/local path.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    /// SHA-256 of the document as retrieved.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sha256: Option<String>,
    /// Effective / amendment date of the version used.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub as_of: Option<String>,
}

/// Who controls an edge, from the analysis perspective.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    /// The protagonist whose policy is optimized.
    #[serde(rename = "self")]
    Me,
    /// An adversary with its own choices.
    Opponent,
    /// Tribunal / agency / chance.
    Nature,
}

/// One forum's procedure graph.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Pack {
    /// `1` (civ-pro statechart format), `2`, `3` (`counting`,
    /// `citeSyntax`), or `4` (`lowered`).
    pub schema_version: u32,
    /// Pack id; the namespace prefix of every node and edge id.
    pub id: String,
    /// Title.
    pub title: String,
    /// Description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Jurisdiction or rule body.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub jurisdiction: Option<String>,
    /// Short forum key, as authored (a lowercase identifier).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub forum: Option<String>,
    /// Id of the deadline-counting rule used by this pack (schema v3).
    /// The id is resolved from authored pack data, never inferred from the
    /// forum name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub counting: Option<String>,
    /// Where walks and default analyses begin.
    pub start_node_id: String,
    /// Node groups.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub groups: Vec<Group>,
    /// actor → role mapping (v2).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub roles: BTreeMap<String, Role>,
    /// Primary sources (v2).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sources: Vec<Source>,
    /// Citation syntax this pack's `cite`/`authority` strings use (v3):
    /// rule-body prefixes and shorthands, each bound to a source.
    #[serde(default, skip_serializing_if = "CiteSyntax::is_empty")]
    pub cite_syntax: CiteSyntax,
    /// Nodes.
    pub nodes: Vec<RawNode>,
    /// Edges.
    pub edges: Vec<RawEdge>,
    /// Filing requirements of the forum's papers in force at the build
    /// dates (v5, optional: data the analyses do not read).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub filing_limits: Vec<super::FilingLimit>,
    /// Build provenance and explicit holes of a compiled pack (v4).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lowered: Option<Lowered>,
}

impl Pack {
    /// Parse and version-check a pack.
    ///
    /// # Errors
    /// `Error::Parse` on malformed JSON, unknown fields, an unsupported
    /// `schemaVersion`, an invalid `citeSyntax`, or a `lowered` header that
    /// does not match the pack.
    pub fn from_json(text: &str) -> Result<Pack> {
        let pack: Pack = serde_json::from_str(text).map_err(|e| Error::Parse(e.to_string()))?;
        if !(1..=PACK_SCHEMA_VERSION).contains(&pack.schema_version) {
            return Err(Error::Parse(format!(
                "pack {}: unsupported schemaVersion {}",
                pack.id, pack.schema_version
            )));
        }
        pack.check_cite_syntax()?;
        pack.check_lowered()?;
        for (e, id) in pack
            .edges
            .iter()
            .zip(super::links::local_edge_ids(&pack.edges))
        {
            if let Some(d) = &e.deadline {
                d.check(
                    &format!("pack {}: edge {id}", pack.id),
                    pack.schema_version,
                    5,
                )?;
            }
        }
        if let Some(e) = pack.edges.iter().find(|e| !e.replaces.is_empty()) {
            return Err(Error::Parse(format!(
                "pack {}: edge {}->{} uses `replaces`, which is only valid in links.json",
                pack.id, e.from, e.to
            )));
        }
        Ok(pack)
    }

    /// Check the `lowered` header (v4) against the pack it heads.
    ///
    /// # Errors
    /// `Error::Parse` naming what does not hold.
    pub fn check_lowered(&self) -> Result<()> {
        let what = format!("pack {}", self.id);
        check_header(&what, self.schema_version, 4, self.lowered.as_ref(), self)
    }

    /// Check `citeSyntax` against this pack's own `sources` (see
    /// [`CiteSyntax::validate`]).
    ///
    /// # Errors
    /// `Error::Parse` naming the offending `citeSyntax` entry.
    pub fn check_cite_syntax(&self) -> Result<()> {
        let source_ids: Vec<&str> = self.sources.iter().map(|s| s.id.as_str()).collect();
        self.cite_syntax.validate(&self.id, &source_ids)
    }
}

/// The v1 semantics: only the applicant chooses; everything else is a draw.
#[must_use]
pub fn default_roles() -> BTreeMap<String, Role> {
    BTreeMap::from([
        ("applicant".to_string(), Role::Me),
        ("examiner".to_string(), Role::Nature),
        ("office".to_string(), Role::Nature),
        ("either".to_string(), Role::Nature),
    ])
}
