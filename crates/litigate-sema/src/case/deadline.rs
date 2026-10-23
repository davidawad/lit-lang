//! Deadline tracking during replay (SPEC §5.3, §5.6): an event taken
//! through an edge `within d` must fall on or before `d`'s due date,
//! counted from the latest earlier event matching `d`'s trigger. A cited
//! event alone cannot transform a deadline: this language version has no
//! typed target, extent, or capability on an excusal.
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
fn variant_args(args: &[Expr]) -> Vec<String> {
    args.iter()
        .filter_map(|a| match &a.kind {
            ExprKind::Literal {
                value: Value::Variant(v),
            } => Some(v.rsplit("::").next().unwrap_or(v).to_owned()),
            _ => None,
        })
        .collect()
}

/// The deadline `id` (written in `pack`) governing `events[at]`, with its
/// qualified id.
pub(super) fn governing<'a>(
    graph: &Graph<'a>,
    pack: &str,
    id: &str,
    events: &[MatterEvent<'_>],
    at: usize,
    known_at: Option<&str>,
) -> Option<&'a Deadline> {
    let qualified = graph.qualify(pack, id);
    match graph.governing(&qualified, events, at, known_at).ok()? {
        Declaration::Deadline(d) => Some(d),
        _ => None,
    }
}

/// The due date of deadline `d` (declared in `pack`) for the event at
/// sorted index `at`, counted from the latest earlier event matching its
/// trigger, under the counting version governing that event. `events` are
/// the sorted events in `pack`'s view. `None` when no trigger has happened
/// (no period is running).
pub(super) fn due(
    graph: &Graph<'_>,
    clock: &dyn Clock,
    pack: &str,
    d: &Deadline,
    events: &[MatterEvent<'_>],
    at: usize,
    known_at: Option<&str>,
) -> Option<Due> {
    // Only a call matches a trigger pattern.
    let (trigger, args) = (0..at).rev().find_map(|j| match &events[j].action.kind {
        ExprKind::Call { args, .. } if matches(&d.trigger, events[j].action) => Some((j, args)),
        _ => None,
    })?;
    let candidates = variant_args(args);
    let counting = match graph.governing(&graph.qualify(pack, &d.counting), events, at, known_at) {
        Ok(Declaration::Counting(c)) => Some(c),
        _ => None,
    };
    let service = match counting {
        Some(c) => candidates
            .into_iter()
            .filter(|v| c.service_extensions.contains_key(v))
            .map(|v| kebab(&v))
            .collect(),
        None => candidates.iter().map(|v| kebab(v)).collect(),
    };
    let request = DueRequest {
        counting: d
            .counting
            .rsplit("::")
            .next()
            .unwrap_or(&d.counting)
            .to_owned(),
        version: counting.map(|c| c.meta.effective.from.clone()),
        trigger: events[trigger].date.to_owned(),
        amount: d.amount,
        unit: d.unit,
        direction: d.direction,
        service,
    };
    Some(Due {
        deadline: d.meta.id.clone(),
        trigger,
        date: clock.due(&request),
        request,
    })
}

/// How a due date was counted, for notes.
pub(super) fn explain(d: &Due, trigger: &Expr) -> String {
    let unit = match d.request.unit {
        DayUnit::Calendar => "calendar",
        DayUnit::Court => "court",
        DayUnit::Business => "business",
    };
    let service = if d.request.service.is_empty() {
        String::new()
    } else {
        format!(", service by {}", d.request.service.join("/"))
    };
    format!(
        "`{}` runs {} {unit} day{} from `{}` on {} under `{}{}`{service}",
        d.deadline,
        d.request.amount,
        if d.request.amount == 1 { "" } else { "s" },
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
