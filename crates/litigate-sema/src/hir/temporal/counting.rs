/// The dates a [`Bound`] is counted from, one per [`DateRef`].
#[derive(Debug, Clone, Copy)]
pub struct BoundDates<'a> {
    /// `trigger_date`.
    pub trigger_date: &'a str,
    /// `prescribed_due`.
    pub prescribed_due: &'a str,
    /// `current_due`.
    pub current_due: &'a str,
    /// `grant_order_entered`.
    pub grant_order_entered: &'a str,
}

/// A date bound: a closed expression language of `max`/`min` over
/// day offsets from prescribed/current due dates or the order date, and
/// calendar months from the instance trigger. Day offsets include `N.days` (FRAP 4(a)(5)(C): "30 days
/// after the prescribed time or 14 days after the date when the order
/// granting the motion is entered, whichever is later"; Sup. Ct. R. 30.2,
/// "at least 10 days before the specified final filing date").
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Bound {
    /// `trigger_date + N.months` (counted by the deadline's clock rule).
    Counted {
        /// Number of calendar months from the trigger.
        months: u64,
    },
    /// `r + N.days`, calendar days.
    Offset {
        /// The date counted from.
        from: DateRef,
        /// Calendar days after it.
        days: u64,
    },
    /// `r - N.days`, calendar days.
    Before {
        /// The date counted from.
        from: DateRef,
        /// Calendar days before it.
        days: u64,
    },
    /// The latest of the operands.
    Max(Vec<Bound>),
    /// The earliest of the operands.
    Min(Vec<Bound>),
}

impl Bound {
    /// The first counted month term in this bound, if any.
    #[must_use]
    pub fn counted_months(&self) -> Option<u64> {
        match self {
            Self::Counted { months } => Some(*months),
            Self::Max(xs) | Self::Min(xs) => xs.iter().find_map(Self::counted_months),
            _ => None,
        }
    }

    /// The bound's date, given the dates it is counted from; `None` if a
    /// date cannot be counted.
    #[must_use]
    pub fn date(&self, at: BoundDates<'_>) -> Option<String> {
        self.date_with(at, &|_| None)
    }

    /// Date bound with calendar-month arithmetic supplied by the declaring
    /// deadline's existing counting rule.
    pub fn date_with(
        &self,
        at: BoundDates<'_>,
        months: &dyn Fn(u64) -> Option<String>,
    ) -> Option<String> {
        match self {
            Self::Counted { months: n } => months(*n),
            Self::Offset { from, days } => crate::types::date::add_days(from.of(at), *days),
            Self::Before { from, days } => crate::types::date::sub_days(from.of(at), *days),
            Self::Max(xs) | Self::Min(xs) => {
                let dates: Option<Vec<String>> =
                    xs.iter().map(|x| x.date_with(at, months)).collect();
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
            Self::Counted { months } => format!("trigger_date + {months}.months"),
            Self::Offset { from, days } => format!("{} + {days}.days", from.name()),
            Self::Before { from, days } => format!("{} - {days}.days", from.name()),
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

/// The motion an extension grant requires (`on M() [, within W] [,
/// unless D()]`): an event of pattern `M` after the instance opens and no
/// later than the order, and no later than bound `W` when given (FRAP
/// 4(a)(5)(A)(i): "no later than 30 days after the time prescribed by this
/// Rule 4(a) expires"), with no event of pattern `D` from the instance's
/// opening to the order (Sup. Ct. R. 30.3).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Motion {
    /// The motion's event pattern.
    pub event: Expr,
    /// The latest date the motion may be made.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub within: Option<Bound>,
    /// The event that, once it precedes the order, bars the grant (`unless
    /// D()`): a denial of the motion, which may not be renewed ("Once
    /// denied, such an application may not be renewed", Sup. Ct. R. 30.3).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unless: Option<Expr>,
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
    /// Sunday only: Saturday is a day like any other (FRCP 6(a) before
    /// July 1, 1963).
    Sunday,
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
    /// `offset(n, weekday, month, days)` shifts a weekday date by calendar days.
    Offset {
        /// Weekday occurrence, 1-4.
        n: u8,
        /// Anchor weekday.
        weekday: String,
        /// Anchor month, 1-12.
        month: u8,
        /// Calendar-day displacement.
        days: i32,
    },
    /// Good Friday, two days before Gregorian Easter Sunday.
    GoodFriday,
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
