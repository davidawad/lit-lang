//! `.case` matters (SPEC §5.6, DESIGN §5): source resolution, `uses` pins
//! against `Lit.lock`, typed parties, event replay under the versions
//! governing each event's date, deadline tracking through the engine
//! clock, typestate, facts, stakes exhaustiveness, and beliefs.
//!
//! [`check_source`] runs the whole pipeline on `.case` text and returns
//! diagnostics located in that text. [`check`] does the same for a matter
//! already in HIR (spans then come from a [`CaseSpans`], possibly empty).
//! A checked matter's `start` is the replayed posture and its `removals`
//! are the options its cited events took away: both are derived, never
//! asserted.
pub mod deadline;
mod drawn;
mod event;
mod graph;
mod prior;
mod replay;
pub mod resolve;
pub mod spans;
mod validate;

pub use deadline::{Clock, DueRequest, NoClock};
pub use graph::display as display_event;
pub use replay::docket_order;
pub use resolve::{resolve, Resolved};
pub use spans::{CaseSpans, EventSpans};

use crate::db::span::sort_diagnostics;
use crate::hir::{self, Package};
use crate::pkg::lock::LockFile;
use graph::Graph;
use litigate_base::{Diagnostic, DiagnosticCode as C, Profile, Severity};
use replay::Replayed;
use spans::error;

/// A checked matter.
#[derive(Debug, Clone, PartialEq)]
pub struct CheckedCase {
    /// The matter with its replayed posture in `start`; `None` when the
    /// source did not resolve.
    pub case: Option<hir::Case>,
    /// Every diagnostic, sorted by location then code.
    pub diagnostics: Vec<Diagnostic>,
    /// The transitions replay made, in order; empty when replay did not
    /// run, and ending at the first illegal event when it stopped.
    pub trace: Vec<Step>,
    /// Every obligation instance the replayed triggers created, in
    /// creation order, with how each ended (or that it is still open at
    /// `as_of`).
    pub obligations: Vec<Obligation>,
}

/// One obligation instance (SPEC §5.6): created by a trigger event,
/// consumed once by a discharge, a breach, a waiver, or a later trigger.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct Obligation {
    /// Qualified deadline id (`pack::deadline`).
    pub deadline: String,
    /// Source index of the trigger event in [`hir::Case::events`].
    pub trigger: usize,
    /// Trigger date.
    pub date: String,
    /// The party bearing it, when the deadline names a `bearer` played
    /// by a declared party.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bearer: Option<String>,
    /// Declared in a `rule duty`: it must be discharged or it is breached.
    pub linear: bool,
    /// The due date it may not end before (`later_of`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub floor: Option<String>,
    /// The due date a cited `extends` event moved it to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extended_to: Option<String>,
    /// The cause standard of the grant that licensed the extension, when
    /// it states one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extended_under: Option<hir::CauseStandard>,
    /// When the request (motion) the licensing grant read was made: the
    /// date of its event, distinct from the order's. It, not the order
    /// date, chooses the window (FRCP 6(b)(1)(A) or (B); 37 C.F.R. §§
    /// 90.3(c)(1), 2.145(e)(1)).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_made: Option<String>,
    /// How it ended, or `open`.
    pub status: Status,
    /// Source index of the event that ended it; `None` while open and for
    /// a breach or lapse found at `as_of`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub closed_by: Option<usize>,
    /// Sorted index of the trigger event (replay-internal).
    #[serde(skip)]
    pub(crate) at: usize,
    /// The trigger's service-method candidates (replay-internal).
    #[serde(skip)]
    pub(crate) candidates: Vec<String>,
}

/// How an obligation instance ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Status {
    /// Not yet due, or due but not countable, at `as_of`.
    Open,
    /// A timely act by its bearer took a `within` edge.
    Discharged,
    /// An act took the `within` edge after the due date (L4002).
    Late,
    /// An act took the `within` edge, but the trigger left the designated
    /// due date unknown (`?`): neither timely nor late (L4010).
    Unjudged,
    /// A duty borne by a party, past due at `as_of` with no act:
    /// breached, a liability of its bearer (L4009).
    Breached,
    /// Past due at `as_of` with no act, and not a breach: a power, or a
    /// period no party bears.
    Lapsed,
    /// A cited `waives` event released it.
    Waived,
    /// A later trigger restarted the period (`trigger latest`).
    Superseded,
    /// A `later_of` instance absorbed it.
    Merged,
}

/// One transition of a replay (SPEC §5.6).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct Step {
    /// Event date; empty for a fact followed before an event.
    pub date: String,
    /// The event as written (`Served(complaint, Personal)`), or `fact`.
    pub event: String,
    /// What the event did.
    pub effect: Effect,
    /// The qualified stage before.
    pub from: String,
    /// The qualified stage after.
    pub to: String,
    /// The qualified edge taken, if it has an id (or the fact's edge).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub edge: Option<String>,
}

/// How a replay step moved the matter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Effect {
    /// The event took an edge.
    Took,
    /// The event took no edge and fired the trigger of an in-force
    /// deadline: it created (or, under `first`, re-fired) an obligation.
    Triggered,
    /// The event took no edge and is matter history a transition reads.
    History,
    /// A `removes` event took an option away.
    Removed,
    /// An `elects` event removed the alternatives to the option chosen.
    Elected,
    /// An `extends` event moved an obligation's due date.
    Extended,
    /// A `waives` event released an obligation.
    Waived,
    /// A `when` guard or a `facts` entry resolved a fact node.
    Fact,
}

impl CheckedCase {
    /// Whether no error was reported.
    #[must_use]
    pub fn ok(&self) -> bool {
        self.case.is_some()
            && !self
                .diagnostics
                .iter()
                .any(|d| d.severity == Severity::Error)
    }
}

/// What a check needs besides the matter.
#[derive(Clone, Copy)]
pub struct Env<'a> {
    /// Every loaded package; `uses` selects among them by name.
    pub packages: &'a [&'a Package],
    /// The matter's `Lit.lock`, when there is one.
    pub lock: Option<&'a LockFile>,
    /// The clock that counts deadlines.
    pub clock: &'a dyn Clock,
    /// `dev` or `strict`.
    pub profile: Profile,
}

/// Resolves and checks `.case` source `text` (repository-relative `file`).
#[must_use]
pub fn check_source(file: &str, text: &str, env: Env<'_>) -> CheckedCase {
    let resolved = resolve(file, text, env.packages, env.lock);
    let Some(case) = resolved.case else {
        let mut diagnostics = resolved.diagnostics;
        sort_diagnostics(&mut diagnostics);
        return CheckedCase {
            case: None,
            diagnostics,
            trace: Vec::new(),
            obligations: Vec::new(),
        };
    };
    let mut checked = check(case, &resolved.spans, env);
    checked.diagnostics.extend(resolved.diagnostics);
    sort_diagnostics(&mut checked.diagnostics);
    checked
}

/// Checks a resolved matter against the loaded packages.
#[must_use]
pub fn check(mut case: hir::Case, spans: &CaseSpans, env: Env<'_>) -> CheckedCase {
    let mut out = Vec::new();
    case.removals.clear();
    dates(&case, spans, &mut out);
    let Some(used) = validate::pins(&case, env.packages, env.lock, spans, &mut out) else {
        return done(case, out, Replayed::default());
    };
    let graph = Graph::new(used, env.packages);
    validate::parties(&case, &graph, spans, &mut out);
    let start = case
        .start
        .clone()
        .unwrap_or_else(|| graph.qualify(graph.primary(), &graph.packages[0].start));
    if !graph.has_node(&start) || !out.is_empty() {
        if !graph.has_node(&start) {
            out.push(error(
                C::InvalidStart,
                spans.at(spans.start.as_ref().map(|s| &s.1), &case.id),
                format!("replay cannot begin at `{start}`: no such node"),
            ));
        }
        return done(case, out, Replayed::default());
    }
    let mut replayed = replay::replay(&graph, &case, spans, env.clock, &start);
    out.append(&mut replayed.diagnostics);
    case.removals = std::mem::take(&mut replayed.removals);
    validate::facts_and_beliefs(&case, &graph, spans, &mut out);
    prior::priors(&case, &graph, spans, &mut out);
    // A fact the history derives at the replayed posture is the matter's,
    // as if stated (SPEC §5.6).
    case.facts.append(&mut replayed.derived);
    if let Some(posture) = replayed.posture.take() {
        validate::start(&case, &posture, spans, &mut out);
        validate::stakes(&case, &graph, &posture, env.profile, spans, &mut out);
        case.start = Some(posture);
    }
    done(case, out, replayed)
}

fn done(case: hir::Case, mut diagnostics: Vec<Diagnostic>, replayed: Replayed) -> CheckedCase {
    sort_diagnostics(&mut diagnostics);
    CheckedCase {
        case: Some(case),
        diagnostics,
        trace: replayed.trace,
        obligations: replayed.obligations,
    }
}

/// `as_of`, `known_at`, and event dates are valid dates (L0004).
fn dates(case: &hir::Case, spans: &CaseSpans, out: &mut Vec<Diagnostic>) {
    let bad = |d: &str| !crate::types::date::is_valid(d);
    if bad(&case.as_of) {
        out.push(error(
            C::InvalidDate,
            spans.at(spans.as_of.as_ref(), &case.id),
            format!("as_of `{}` is not a valid date", case.as_of),
        ));
    }
    if case.known_at.as_deref().is_some_and(bad) {
        out.push(
            error(
                C::InvalidDate,
                spans.at(None, &case.id),
                "known_at is not a valid date",
            )
            .note("dates are proleptic Gregorian YYYY-MM-DD"),
        );
    }
    for (i, e) in case.events.iter().enumerate() {
        if bad(&e.date) {
            out.push(error(
                C::InvalidDate,
                spans.event(i, |s| s.date.as_ref(), &case.id),
                format!("`{}` is not a valid date", e.date),
            ));
        }
    }
}
