//! Deadlines, counting rules, holiday calendars, versions, and transitions.
use super::{DayUnit, Expr, Metadata};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Half-open in-force interval `[from, to)` plus corpus (`known_at`) date.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Interval {
    /// Inclusive beginning date.
    pub from: String,
    /// Exclusive end date; `None` is open-ended.
    pub to: Option<String>,
    /// Date the corpus recorded this version.
    pub known_at: String,
}

/// Typed deadline declaration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Deadline {
    /// Declaration metadata.
    pub meta: Metadata,
    /// Period length.
    pub amount: u64,
    /// Period unit.
    pub unit: DayUnit,
    /// Counting direction (`from`/`after` forward, `before` backward).
    pub direction: Direction,
    /// Trigger event expression; for a chained period, the end it runs
    /// from (`deadline(d).nominal`), which no event matches.
    pub trigger: Expr,
    /// The period this one runs from the end of (`from
    /// deadline(d).nominal`), instead of from an event.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chain: Option<Chain>,
    /// Qualified counting-rule id.
    pub counting: String,
    /// The forum whose rules set the period (`forum F`, else the declaring
    /// package's): its counting rule must be one of that forum's.
    pub forum: String,
    /// Whether extensions are available (`Deadline.extendable`).
    pub extendable: Option<bool>,
    /// Extension authority text.
    pub extension_authority: Option<String>,
    /// Lowered `Deadline.note`.
    pub note: Option<String>,
    /// The actor bearing each obligation instance (`bearer A`); `None`:
    /// whoever takes the `within` edge.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bearer: Option<String>,
    /// How a trigger is treated while an instance is open (`trigger ...`).
    #[serde(default, skip_serializing_if = "TriggerPolicy::is_first")]
    pub policy: TriggerPolicy,
    /// Declared in a `rule duty`: an instance is linear, consumed exactly
    /// once by a discharge or a breach. Otherwise affine: it may lapse.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub linear: bool,
    /// Who may extend an instance, when, and how far (`extend { ... }`,
    /// once per grant); empty: no extension power is declared.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub extend: Vec<Extension>,
    /// Who may waive an instance (`waive { ... }`); `None`: it cannot be
    /// waived.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub waive: Option<Capability>,
    /// The trigger's `Date` parameter that designates each instance's due
    /// date (`designated p`): the issuing notice or order sets the date,
    /// and `amount`/`unit` are only the least time the rule allows it (37
    /// C.F.R. § 2.105(a), "a time, not less than thirty days from the
    /// mailing date of the notice"). `None`: the period is counted.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub designated: Option<Designated>,
}

/// Why a designated deadline has no due date (SPEC §5.3).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Undesignated {
    /// The trigger leaves the date unknown (`?`, or no argument): the due
    /// date is unknown, never counted in its place.
    Unknown(String),
    /// The trigger designates a date earlier than the rule allows: the
    /// notice is rejected.
    BelowFloor(String),
}

impl Undesignated {
    /// The explanation.
    #[must_use]
    pub fn message(&self) -> &str {
        match self {
            Self::Unknown(m) | Self::BelowFloor(m) => m,
        }
    }
}

impl Deadline {
    /// The due date that trigger event `action` on `date` designates, before
    /// any roll (`designated p`); `None` when the period is counted. The
    /// caller has matched `action` against the trigger. A date
    /// earlier than `amount` calendar days from `date` is refused: "a time,
    /// not less than thirty days from the mailing date of the notice" (37
    /// C.F.R. § 2.105(a)).
    ///
    /// # Errors
    /// The date is unknown, or below the floor.
    #[must_use]
    pub fn designation(&self, date: &str, action: &Expr) -> Option<Result<String, Undesignated>> {
        let d = self.designated.as_ref()?;
        let (event, args) = match &action.kind {
            super::ExprKind::Call { callee, args } => (callee.as_str(), args.as_slice()),
            _ => ("the trigger", &[][..]),
        };
        let designated = match args.get(d.param).map(|a| &a.kind) {
            Some(super::ExprKind::Literal {
                value: super::Value::Date(v),
            }) => v.clone(),
            _ => {
                return Some(Err(Undesignated::Unknown(format!(
                    "`{event}` designates no due date for `{}` (its `{}` is unknown), so the due date is unknown: it is the notice's to set, never counted in its place",
                    self.meta.id, d.name
                ))))
            }
        };
        let floor = crate::types::date::add_days(date, self.amount).unwrap_or_default();
        if designated < floor {
            let cite = self
                .meta
                .cite
                .as_ref()
                .map_or_else(String::new, |c| format!(" ({c})"));
            return Some(Err(Undesignated::BelowFloor(format!(
                "`{event}` on {date} designates `{}` due {designated}, earlier than the least time the rule allows, {} calendar days from {date} ({floor}){cite}",
                self.meta.id, self.amount
            ))));
        }
        Some(Ok(designated))
    }
}

/// The trigger parameter designating a deadline's due date (SPEC §5.3).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Designated {
    /// Its index among the trigger event's parameters.
    pub param: usize,
    /// Its name.
    pub name: String,
}

/// A chained period's trigger: the end of another deadline's period
/// (TBMP 112: an extension "from the actual due date ..., not the adjusted
/// date"). A chain always says which end.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Chain {
    /// Qualified id of the deadline whose period this one follows.
    pub deadline: String,
    /// Which end of that period it runs from.
    pub end: DueEnd,
}

/// One end of a counted period (`Due {nominal, adjusted}`, SPEC §5.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DueEnd {
    /// The last day as counted, before any roll or extension.
    Nominal,
    /// The date after every roll and service extension.
    Adjusted,
}

impl DueEnd {
    /// The `Due` field naming this end.
    #[must_use]
    pub fn field(self) -> &'static str {
        match self {
            Self::Nominal => "nominal",
            Self::Adjusted => "adjusted",
        }
    }
}

/// How a matching trigger creates obligation instances (SPEC §5.3).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "policy", content = "of", rename_all = "snake_case")]
pub enum TriggerPolicy {
    /// The first trigger runs the period; later ones are ignored while
    /// its instance is open.
    #[default]
    First,
    /// A later trigger restarts the period, superseding the open instance.
    Latest,
    /// Every trigger creates its own instance.
    Each,
    /// Like `latest`, and due no earlier than the open instance of the
    /// named deadline, which it absorbs (FRCP 15(a)(3), FRAP 4(a)(3)).
    LaterOf(String),
}

impl TriggerPolicy {
    /// Whether this is the default policy.
    #[must_use]
    pub fn is_first(&self) -> bool {
        *self == Self::First
    }
}

/// A capability to waive an obligation instance (review §2.4): the
/// principal that holds it and whether it reaches an expired instance.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Capability {
    /// The actor that may exercise it (`by A`).
    pub by: String,
    /// May be exercised after the instance's due date (`after_expiry`).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub after_expiry: bool,
}

/// One `extend` grant of a deadline (review §2.4): its bound is a sum, so
/// a rule that states no numeric ceiling is not forced to invent one
/// (FRCP 6(b)).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "bound", rename_all = "snake_case")]
pub enum Extension {
    /// `extend { by A, max N.days | to B [, discretion S] [, after_expiry]
    /// [, on M() [, within W]] }`: no later than the date bound `B` (`max
    /// N.days` is `to prescribed_due + N.days`), a cap the declaring file
    /// imposes (a local rule allowing one stipulated extension of N days,
    /// FRAP 4(a)(5)(C)).
    Bounded {
        /// The actor that may extend.
        by: String,
        /// The latest date an extension may reach.
        until: Bound,
        /// May be exercised after the instance's due date as well as before.
        #[serde(default, skip_serializing_if = "std::ops::Not::not")]
        after_expiry: bool,
        /// The cause standard the extender applies, if the rule states one
        /// (FRAP 4(a)(5)(A)(ii)).
        #[serde(default, skip_serializing_if = "Option::is_none")]
        standard: Option<CauseStandard>,
        /// The motion the grant requires, if any (FRAP 4(a)(5)(A)(i)).
        #[serde(default, skip_serializing_if = "Option::is_none")]
        motion: Option<Motion>,
    },
    /// `extend { by A, discretion S [, after_expiry] [, on M()] }`: the
    /// court's discretion under cause standard `S`, with no numeric ceiling
    /// (FRCP 6(b)(1)); after expiry only on motion `M` (FRCP 6(b)(1)(B)).
    /// Before expiry with a motion, the grant is on request: the window is
    /// judged on the request's date, so a request made before expiry
    /// licenses a later order (FRCP 6(b)(1)(A); 37 C.F.R. § 90.3(c)(1)(i)).
    Discretionary {
        /// The actor that may extend.
        by: String,
        /// The cause standard.
        standard: CauseStandard,
        /// When the grant may be exercised.
        window: ExtensionWindow,
        /// The motion the grant requires: always present for
        /// [`ExtensionWindow::AfterExpiryOnMotion`]; for
        /// [`ExtensionWindow::BeforeExpiry`], the request whose date is the
        /// window's.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        motion: Option<Motion>,
    },
    /// `extend { never cite "X" }`: the rule `X` forbids extending the
    /// period beyond the deadline's own other grants, so no order, local
    /// rule or stipulation adds one (FRCP 6(b)(2); FRAP 26(b), "except as
    /// authorized in Rule 4").
    NonExtendable {
        /// The rule that forbids the extension.
        authority: String,
    },
}

impl Extension {
    /// The actor holding the grant; `None` for [`Self::NonExtendable`].
    #[must_use]
    pub fn by(&self) -> Option<&str> {
        match self {
            Self::Bounded { by, .. } | Self::Discretionary { by, .. } => Some(by),
            Self::NonExtendable { .. } => None,
        }
    }

    /// The authority of a [`Self::NonExtendable`] lock.
    #[must_use]
    pub fn lock(&self) -> Option<&str> {
        match self {
            Self::NonExtendable { authority } => Some(authority),
            Self::Bounded { .. } | Self::Discretionary { .. } => None,
        }
    }
}

/// When a discretionary extension may be granted, relative to the
/// instance's due date.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExtensionWindow {
    /// On or before the due date, with or without motion (FRCP
    /// 6(b)(1)(A)).
    BeforeExpiry,
    /// After the due date, on motion (FRCP 6(b)(1)(B)).
    AfterExpiryOnMotion,
}

/// The cause standard an extension grant applies: a closed set, so a
/// misspelled standard is a compile error (L2007).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CauseStandard {
    /// Good cause (FRCP 6(b)(1)(A)).
    GoodCause,
    /// Excusable neglect (FRCP 6(b)(1)(B)).
    ExcusableNeglect,
    /// Either excusable neglect or good cause (FRAP 4(a)(5)(A)(ii)).
    GoodCauseOrExcusableNeglect,
}

impl CauseStandard {
    /// Every standard, in declaration order.
    pub const ALL: [Self; 3] = [
        Self::GoodCause,
        Self::ExcusableNeglect,
        Self::GoodCauseOrExcusableNeglect,
    ];

    /// Its identifier in source.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::GoodCause => "GoodCause",
            Self::ExcusableNeglect => "ExcusableNeglect",
            Self::GoodCauseOrExcusableNeglect => "GoodCauseOrExcusableNeglect",
        }
    }
}

/// A date an extension bound is counted from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DateRef {
    /// The instance's counted due date, raised to any `later_of` floor
    /// (`prescribed_due`): "the time prescribed by this Rule 4(a)".
    PrescribedDue,
    /// The date of the order granting the extension
    /// (`grant_order_entered`).
    GrantOrderEntered,
}

impl DateRef {
    /// Its identifier in source.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::PrescribedDue => "prescribed_due",
            Self::GrantOrderEntered => "grant_order_entered",
        }
    }
}

/// A date bound: a closed expression language of `max`/`min` over
/// `DateRef + N.days` (FRAP 4(a)(5)(C): "30 days after the prescribed time
/// or 14 days after the date when the order granting the motion is
/// entered, whichever is later").
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Bound {
    /// `r + N.days`, calendar days.
    Offset {
        /// The date counted from.
        from: DateRef,
        /// Calendar days after it.
        days: u64,
    },
    /// The latest of the operands.
    Max(Vec<Bound>),
    /// The earliest of the operands.
    Min(Vec<Bound>),
}

impl Bound {
    /// The bound's date, given the dates of `prescribed_due` and
    /// `grant_order_entered`; `None` if a date cannot be counted.
    #[must_use]
    pub fn date(&self, due: &str, order: &str) -> Option<String> {
        match self {
            Self::Offset { from, days } => {
                let at = match from {
                    DateRef::PrescribedDue => due,
                    DateRef::GrantOrderEntered => order,
                };
                crate::types::date::add_days(at, *days)
            }
            Self::Max(xs) | Self::Min(xs) => {
                let dates: Option<Vec<String>> = xs.iter().map(|x| x.date(due, order)).collect();
                let latest = matches!(self, Self::Max(_));
                dates.and_then(|d| {
                    if latest {
                        d.into_iter().max()
                    } else {
                        d.into_iter().min()
                    }
                })
            }
        }
    }

    /// Its source form.
    #[must_use]
    pub fn display(&self) -> String {
        match self {
            Self::Offset { from, days } => format!("{} + {days}.days", from.name()),
            Self::Max(xs) | Self::Min(xs) => format!(
                "{}({})",
                if matches!(self, Self::Max(_)) {
                    "max"
                } else {
                    "min"
                },
                xs.iter().map(Self::display).collect::<Vec<_>>().join(", ")
            ),
        }
    }
}

/// The motion an extension grant requires (`on M() [, within W]`): an
/// event of pattern `M` after the instance opens and no later than the
/// order, and no later than bound `W` when given (FRAP 4(a)(5)(A)(i): "no
/// later than 30 days after the time prescribed by this Rule 4(a)
/// expires").
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Motion {
    /// The motion's event pattern.
    pub event: Expr,
    /// The latest date the motion may be made.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub within: Option<Bound>,
}

/// Deadline counting direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Direction {
    /// Count forward from the trigger.
    Forward,
    /// Count backward from the trigger.
    Backward,
}

/// Data-driven day-counting rule (lowered to `Pack.counting`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Counting {
    /// Declaration metadata.
    pub meta: Metadata,
    /// Exclude the trigger day.
    pub exclude_trigger_day: bool,
    /// Include the last day (subject to rolling).
    pub include_last_day: bool,
    /// Excluded last days that roll in the counting direction.
    pub roll_on: Vec<RollOn>,
    /// What each service method adds, forward counting only: total over
    /// [`SERVICE_METHODS`].
    pub service_extensions: BTreeMap<String, ServiceTime>,
    /// Qualified holiday-calendar id (closes either direction).
    pub calendar: String,
    /// Qualified id of a calendar closing only periods counted forward
    /// (`forward_holidays`, FRCP 6(a)(6)(C)).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub forward_calendar: Option<String>,
    /// Qualified id of a calendar of declared days the forum closes on in
    /// either direction (`declared_holidays`, FRCP 6(a)(6)(B)).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub declared_calendar: Option<String>,
    /// The forums whose periods the rule computes (`forums [...]`, else the
    /// declaring package's forum).
    pub forums: Vec<String>,
    /// Where counting starts.
    pub anchor: Anchor,
    /// Periods shorter than `threshold` count in `unit` instead.
    pub short_period: Option<ShortPeriod>,
    /// Whether hour periods (FRCP 6(a)(2)) are permitted.
    pub hours: bool,
    /// End-of-month policy; month and year periods need one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub end_of_month: Option<EndOfMonth>,
}

/// The service methods, a closed set: the variants of `core.time`'s
/// `ServiceMethod` and the engine's service vocabulary. A counting rule's
/// `service_extension` names each exactly once (design review 2026-10-23,
/// P8).
pub const SERVICE_METHODS: [&str; 10] = [
    "Personal",
    "Electronic",
    "Mail",
    "Clerk",
    "OtherConsented",
    "CommercialCarrier",
    "PrisonMail",
    "MailForeign",
    "Express",
    "ExpressForeign",
];

/// What one service method adds under a counting rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ServiceTime {
    /// The rule recognizes the method and adds this many days (zero
    /// included) after the period would otherwise expire.
    Adds(u64, DayUnit),
    /// The rule does not name the method: a period served that way is not
    /// counted.
    Unrecognized,
}

/// Where a month or year period ends when its last month has no day
/// corresponding to the trigger's (January 31 plus one month).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EndOfMonth {
    /// The last day of that month (MPEP 710.01(a)).
    Clamp,
    /// The first day of the following month.
    Roll,
}

/// Day kinds that cannot be a last day.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RollOn {
    /// Saturday or Sunday.
    Weekend,
    /// A holiday of the referenced calendar.
    LegalHoliday,
    /// A day the clerk's office is inaccessible (case-supplied dates).
    Inaccessible,
}

/// Counting anchor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Anchor {
    /// Count from the trigger day itself.
    TriggerDay,
    /// Count from the first business day on or after the trigger.
    NextBusinessDay,
}

/// Short-period rule (e.g. 19 CFR 201.14: under 7 days count business days).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShortPeriod {
    /// Periods with `amount < threshold` are affected.
    pub threshold: u64,
    /// Unit used for affected periods.
    pub unit: DayUnit,
}

/// Holiday calendar (lowered to `Pack.calendars[]`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Calendar {
    /// Declaration metadata.
    pub meta: Metadata,
    /// Holidays in source order.
    pub holidays: Vec<Holiday>,
}

/// One holiday definition.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Holiday {
    /// Stable holiday id.
    pub id: String,
    /// Date rule.
    pub date: HolidayDate,
    /// In-force interval.
    pub in_force: Interval,
    /// Weekend observance shift.
    pub observed: Option<Observance>,
}

/// Holiday date rule (SPEC §8.2).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "rule", rename_all = "snake_case")]
pub enum HolidayDate {
    /// `fixed(month, day)`.
    Fixed {
        /// Month 1-12.
        month: u8,
        /// Day 1-31.
        day: u8,
    },
    /// `nth(n, weekday, month)`: the n-th (1-4) weekday of the month.
    Nth {
        /// Ordinal 1-4.
        n: u8,
        /// Lowercase English weekday.
        weekday: String,
        /// Month 1-12.
        month: u8,
    },
    /// `last(weekday, month)`.
    Last {
        /// Lowercase English weekday.
        weekday: String,
        /// Month 1-12.
        month: u8,
    },
    /// `every(years, from_year, month, day)`: years `from_year + k*years`, k >= 0.
    Every {
        /// Period in years.
        years: u16,
        /// First year.
        from_year: u16,
        /// Month 1-12.
        month: u8,
        /// Day 1-31.
        day: u8,
    },
    /// `once(date)`: a single proclaimed day.
    Once {
        /// ISO date.
        date: String,
    },
}

/// Observance shift for a holiday falling on a weekend.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Observance {
    /// Saturday to Friday, Sunday to Monday.
    NearestWeekday,
    /// Sunday to Monday only.
    NextDay,
}

/// Version-selection rule for proceedings pending across an amendment.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Transition {
    /// Declaration metadata.
    pub meta: Metadata,
    /// Amended declaration, version-qualified (`id@from-date`).
    pub amendment: String,
    /// Arms in source order; the last arm is `otherwise`.
    pub arms: Vec<TransitionArm>,
}

/// One transition arm.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TransitionArm {
    /// `unless` condition; `None` for `otherwise`.
    pub unless: Option<Expr>,
    /// Version-qualified declaration selected.
    pub selects: String,
}
