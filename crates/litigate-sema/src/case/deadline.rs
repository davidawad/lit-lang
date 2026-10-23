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
use crate::hir::{DayUnit, Deadline, Declaration, Direction, Expr, ExprKind, Value};
use crate::temporal::condition::matches;
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
    /// The due date (ISO 8601) of `request`.
    ///
    /// # Errors
    /// Why the clock cannot count it (unknown rule, invalid period).
    fn due(&self, request: &DueRequest) -> Result<String, String>;
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
    /// The trigger parameter designating the due date (`designated p`);
    /// `None` when the period is counted.
    pub designated: Option<String>,
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
    (trigger, candidates): (usize, &[String]),
    at: usize,
    known_at: Option<&str>,
) -> Due {
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
        trigger: events[trigger].date.to_owned(),
        amount: d.amount,
        unit: d.unit,
        direction: d.direction,
        service,
    };
    // A rule a loaded package declares counts only under a version that
    // governs the event; the shared library's stands in only for a rule
    // no package declares.
    let date = if counting.is_none() && graph.declares(&qualified) {
        Err(format!(
            "no version of counting rule `{qualified}` governs {}",
            events[at].date
        ))
    } else {
        let event = &events[trigger];
        match d.designation(event.date, event.action) {
            // A displacing claimant counted for another deadline's
            // instance: its trigger designates nothing for it.
            Some(_) if !matches(&d.trigger, event.action) => Err(format!(
                "`{}` is not the trigger of `{}`, so it designates no due date for it",
                display(event.action),
                d.meta.id
            )),
            None => clock.due(&request),
            // The notice's date, rolled under the counting rule like any
            // last day (37 C.F.R. § 2.196): counted zero days from it.
            Some(Ok(designated)) => clock.due(&DueRequest {
                trigger: designated,
                amount: 0,
                service: Vec::new(),
                ..request.clone()
            }),
            Some(Err(why)) => Err(why.message().to_owned()),
        }
    };
    Due {
        deadline: d.meta.id.clone(),
        trigger,
        date,
        request,
        designated: d.designated.as_ref().map(|p| p.name.clone()),
    }
}

/// How a due date was counted, for notes.
pub(super) fn explain(d: &Due, trigger: &Expr) -> String {
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
        "`{}` runs {} from `{}` on {} under `{}{}`{service}",
        d.deadline,
        d.request.unit.prose(d.request.amount),
        display(trigger),
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
