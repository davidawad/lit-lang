//! Deadline counting during replay (SPEC §5.3, §5.6): the due date of an
//! obligation instance, counted from its trigger event under the deadline
//! and counting versions governing the event it is judged at. Excusal and
//! extension are typed transforms of an instance (`replay/obligations.rs`),
//! never the mere presence of a cited event.
//!
//! Sema does not count days itself. The due date comes from a [`Clock`]:
//! in production the engine's clock interpreter running the lowered
//! counting rule (`litigate_lower::EngineClock`), so a matter is judged by
//! exactly the arithmetic its scenario will run under.
use super::graph::{display, Graph};
use crate::hir::{DayUnit, Deadline, Declaration, Direction, DueEnd, Expr, ExprKind, Value};
use crate::temporal::MatterEvent;

/// One day-counted deadline computation: the engine clock's inputs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DueRequest {
    /// Counting-rule id in the clock library (global, unqualified).
    pub counting: String,
    /// The `from` date of the governing version, when a used package
    /// declares the rule (its versions lower to different rules with one
    /// id); `None` for a rule of the shared library.
    pub version: Option<String>,
    /// Trigger date (ISO 8601).
    pub trigger: String,
    /// Period length.
    pub amount: u64,
    /// Period unit.
    pub unit: DayUnit,
    /// Counting direction.
    pub direction: Direction,
    /// Service-method candidates from the trigger's enum arguments, in
    /// kebab case (`Mail` is `mail`). When the package declares the
    /// counting rule, only its `service_extension` keys are offered;
    /// otherwise the library rule decides which one it recognizes.
    pub service: Vec<String>,
}

/// Computes due dates (SPEC §8.2: the generic interpreter over clock IR).
pub trait Clock {
    /// The due date (ISO 8601) of `request`: the adjusted end, after every
    /// roll and extension.
    ///
    /// # Errors
    /// Why the clock cannot count it (unknown rule, invalid period).
    fn due(&self, request: &DueRequest) -> Result<String, String>;

    /// The nominal end (ISO 8601) of `request`: the last day as counted,
    /// before any roll or extension (a chained period's `.nominal`).
    ///
    /// # Errors
    /// As [`Clock::due`].
    fn nominal(&self, request: &DueRequest) -> Result<String, String>;
}

/// A clock that counts nothing: every request is an error. Replay with it
/// still checks order, typestate, and transitions; any `within` edge taken
/// after a trigger is then reported as uncountable.
#[derive(Debug, Clone, Copy, Default)]
pub struct NoClock;

impl Clock for NoClock {
    fn due(&self, request: &DueRequest) -> Result<String, String> {
        Err(format!(
            "no clock library is loaded to run counting rule `{}`",
            request.counting
        ))
    }

    fn nominal(&self, request: &DueRequest) -> Result<String, String> {
        self.due(request)
    }
}

/// A computed deadline that an event is judged against.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Due {
    /// Deadline id as written on the edge.
    pub deadline: String,
    /// Index (sorted order) of the triggering event.
    pub trigger: usize,
    /// The counting request.
    pub request: DueRequest,
    /// The computed due date, or why it could not be computed.
    pub date: Result<String, String>,
    /// For a chained period, the end it runs from (`the nominal end of
    /// \`d\``); `None` when it runs from its trigger event.
    pub from: Option<String>,
}

/// `CamelCase` to `kebab-case`.
fn kebab(name: &str) -> String {
    let mut out = String::new();
    for (i, c) in name.chars().enumerate() {
        if c.is_ascii_uppercase() {
            if i > 0 {
                out.push('-');
            }
            out.push(c.to_ascii_lowercase());
        } else {
            out.push(c);
        }
    }
    out
}

/// The local names of the enum variants among a trigger's arguments.
pub(super) fn variant_args(args: &[Expr]) -> Vec<String> {
    args.iter()
        .filter_map(|a| match &a.kind {
            ExprKind::Literal {
                value: Value::Variant(v),
            } => Some(v.rsplit("::").next().unwrap_or(v).to_owned()),
            _ => None,
        })
        .collect()
}

/// The deadline `id` (written in `pack`) governing `events[at]`, with the
/// qualified id of its logical declaration (a displacing claimant's own).
pub(super) fn governing<'a>(
    graph: &Graph<'a>,
    pack: &str,
    id: &str,
    events: &[MatterEvent<'_>],
    at: usize,
    known_at: Option<&str>,
) -> Option<(String, &'a Deadline)> {
    let qualified = graph.qualify(pack, id);
    match graph.governing(&qualified, events, at, known_at).ok()? {
        (q, Declaration::Deadline(d)) => Some((q, d)),
        _ => None,
    }
}

/// The due date of deadline `d` (declared in `pack`) for an instance
/// triggered by `events[trigger]`, judged at `events[at]`: counted under
/// the counting version governing `events[at]`. `events` are the sorted
/// events in `pack`'s view (possibly with an `as_of` sentinel last);
/// `candidates` are the trigger's [`variant_args`].
#[allow(clippy::too_many_arguments)]
pub(super) fn due(
    graph: &Graph<'_>,
    clock: &dyn Clock,
    pack: &str,
    d: &Deadline,
    events: &[MatterEvent<'_>],
    trigger: (usize, &[String]),
    at: usize,
    known_at: Option<&str>,
) -> Due {
    let judged = Judged {
        graph,
        clock,
        events,
        at,
        known_at,
    };
    judged.due(pack, d, trigger, CHAIN_LIMIT)
}

/// How many chained periods a chain may follow before it is a cycle.
const CHAIN_LIMIT: usize = 16;

/// What a due date is judged against: the graph and clock, and the events
/// (judged at `events[at]`) under `known_at`.
struct Judged<'j, 'g> {
    graph: &'j Graph<'g>,
    clock: &'j dyn Clock,
    events: &'j [MatterEvent<'j>],
    at: usize,
    known_at: Option<&'j str>,
}

impl Judged<'_, '_> {
    /// The due date of `d` (declared in `pack`); a chained period counts
    /// from the end of the period it follows, at most `depth` links deep.
    fn due(&self, pack: &str, d: &Deadline, trigger: (usize, &[String]), depth: usize) -> Due {
        let Some(chain) = &d.chain else {
            return self.count(pack, d, trigger, None);
        };
        let base = governing(
            self.graph,
            pack,
            &chain.deadline,
            self.events,
            self.at,
            self.known_at,
        );
        // Replay opens a chained instance only once its whole chain
        // resolves (`root`), so this refusal guards a broken invariant.
        let unresolved = format!(
            "the chain through `{}` does not resolve on {}",
            d.meta.id, self.events[self.at].date
        );
        let date = base
            .filter(|_| depth > 0)
            .ok_or(unresolved)
            .and_then(|(q, base)| {
                let b = self.due(pack_of(&q), base, trigger, depth - 1);
                match chain.end {
                    DueEnd::Adjusted => b.date,
                    DueEnd::Nominal => b.date.and_then(|_| self.clock.nominal(&b.request)),
                }
            });
        let start = Start {
            from: format!("the {} end of `{}`", chain.end.field(), chain.deadline),
            date,
        };
        // A chained period runs from a date, not from service.
        self.count(pack, d, (trigger.0, &[]), Some(start))
    }

    /// `d` counted from its trigger event, or from a chain's `start`.
    fn count(
        &self,
        pack: &str,
        d: &Deadline,
        (trigger, candidates): (usize, &[String]),
        start: Option<Start>,
    ) -> Due {
        let (graph, events, at, known_at) = (self.graph, self.events, self.at, self.known_at);
        let clock = self.clock;
        let qualified = graph.qualify(pack, &d.counting);
        // Use the shared temporal query so replay observes the same version,
        // including known_at filtering, as test, lower, check, and the LSP.
        let counting = match graph.governing(&qualified, events, at, known_at) {
            Ok((_, Declaration::Counting(c))) => Some(c),
            _ => None,
        };
        let service = match counting {
            Some(c) => candidates
                .iter()
                .filter(|v| c.service_extensions.contains_key(*v))
                .map(|v| kebab(v))
                .collect(),
            None => candidates.iter().map(|v| kebab(v)).collect(),
        };
        let request = DueRequest {
            counting: counting.map_or_else(
                || {
                    d.counting
                        .rsplit("::")
                        .next()
                        .unwrap_or(&d.counting)
                        .to_owned()
                },
                |c| c.meta.id.clone(),
            ),
            version: counting.map(|c| c.meta.effective.from.clone()),
            trigger: match &start {
                Some(Start { date: Ok(date), .. }) => date.clone(),
                _ => events[trigger].date.to_owned(),
            },
            amount: d.amount,
            unit: d.unit,
            direction: d.direction,
            service,
        };
        // A rule a loaded package declares counts only under a version that
        // governs the event; the shared library's stands in only for a rule
        // no package declares.
        let from = start.as_ref().map(|s| s.from.clone());
        let date = if let Some(Start {
            date: Err(why),
            from,
        }) = start
        {
            Err(format!(
                "`{}` runs from {from}, which is not counted: {why}",
                d.meta.id
            ))
        } else if counting.is_none() && graph.declares(&qualified) {
            Err(format!(
                "no version of counting rule `{qualified}` governs {}",
                events[at].date
            ))
        } else {
            clock.due(&request)
        };
        Due {
            deadline: d.meta.id.clone(),
            trigger,
            date,
            request,
            from,
        }
    }
}

/// Where a chained period starts: the end it runs from, and its date (or
/// why that end is not counted).
struct Start {
    from: String,
    date: Result<String, String>,
}

/// The pack of a qualified id (`pack::name`).
fn pack_of(q: &str) -> &str {
    q.split("::").next().unwrap_or(q)
}

/// How a due date was counted, for notes.
pub(super) fn explain(d: &Due, trigger: &Expr) -> String {
    let start = d
        .from
        .clone()
        .unwrap_or_else(|| format!("`{}`", display(trigger)));
    let service = if d.request.service.is_empty() {
        String::new()
    } else {
        format!(", service by {}", d.request.service.join("/"))
    };
    if let Some(p) = &d.designated {
        return format!(
            "`{}` is due on the date `{}` on {} designates (its `{p}`), no earlier than {} from it, rolled under `{}`",
            d.deadline,
            display(trigger),
            d.request.trigger,
            d.request.unit.prose(d.request.amount),
            d.request.counting,
        );
    }
    format!(
        "`{}` runs {} from {start} on {} under `{}{}`{service}",
        d.deadline,
        d.request.unit.prose(d.request.amount),
        d.request.trigger,
        d.request.counting,
        d.request
            .version
            .as_ref()
            .map_or_else(String::new, |v| format!("@{v}")),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kebab_case_matches_engine_methods() {
        assert_eq!(kebab("Mail"), "mail");
        assert_eq!(kebab("CommercialCarrier"), "commercial-carrier");
        assert_eq!(kebab("MailForeign"), "mail-foreign");
    }

    #[test]
    fn no_clock_reports_the_rule() {
        let r = DueRequest {
            counting: "frcp6".into(),
            version: None,
            trigger: "2025-03-03".into(),
            amount: 21,
            unit: DayUnit::Calendar,
            direction: Direction::Forward,
            service: Vec::new(),
        };
        assert!(NoClock.due(&r).is_err_and(|e| e.contains("frcp6")));
    }
}
