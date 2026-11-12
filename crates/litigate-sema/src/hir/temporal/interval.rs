/// Half-open in-force interval `[from, to)` plus corpus (`known_at`) date.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Interval {
    /// Inclusive beginning date.
    pub from: String,
    /// Exclusive end date; `None` is open-ended.
    pub to: Option<String>,
    /// Date the corpus recorded this version.
    pub known_at: String,
    /// Date the corpus retracted this version (a later version of the same
    /// window corrects it), if ever: transaction time is `[known_at,
    /// retracted)`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retracted: Option<String>,
    /// Date before which the declaration's law is not encoded: a query
    /// there with no version is a hole (L3010), not an absence of law.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unknown_before: Option<String>,
    /// The proceedings the version applies to (`commenced [from, to)`):
    /// those commenced in the window; `None`: every proceeding.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub commenced: Option<Commenced>,
    /// The version is repealed at its `to` (`repealed`): the gap from
    /// there to the next version or the package's end is declared (L3002).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub repealed: bool,
}

/// A window of proceeding commencement dates (design review §2.6:
/// applicability keyed by a proceeding attribute).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Commenced {
    /// Inclusive first commencement date.
    pub from: String,
    /// Exclusive end; `None` is open-ended.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub to: Option<String>,
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
    /// The rule adopting `forum`'s period for this deadline (`forum F cite
    /// "X" quote "..."`); `None`: no adopting rule is declared.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub adoption: Option<Adoption>,
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
    /// The bearer is a party other than the trigger's (`bearer other(A)`):
    /// FRAP 4(a)(3)'s "any other party" after the first notice of appeal.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub bearer_other: bool,
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
    /// Jurisdictional or claim-processing (`characterization ...`);
    /// `None`: not declared.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub characterization: Option<Characterization>,
    /// The cited terms the period is the sum of (`derived N.unit cite
    /// "X"`, once per term), when no one rule states it: a reply period
    /// extended by a month, or `the prior Monday` before a Monday hearing.
    /// Each term's cite is held to its count in place of the deadline's
    /// own cite; empty: one rule states the period.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub derived: Vec<Derivation>,
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
    /// C.F.R. § 2.105(a)). Under an `interpretation` the rule names no
    /// start, so such a date stands ([`Self::interpreted_shortfall`]).
    ///
    /// # Errors
    /// The date is unknown, or below the floor.
    #[must_use]
    pub fn designation(&self, date: &str, action: &Expr) -> Option<Result<String, Undesignated>> {
        self.designate(date, action).map(|r| r.map(|(due, _)| due))
    }

    /// Why the date trigger event `action` on `date` designates is short
    /// of the least time measured from `date` when only an
    /// `interpretation` measures it from there (37 C.F.R.
    /// § 2.106(b)(3)(iii), "A time, not less than thirty days, will be
    /// designated by the Board", names no start): a warning naming the
    /// interpretation, never a rejection. `None` otherwise.
    #[must_use]
    pub fn interpreted_shortfall(&self, date: &str, action: &Expr) -> Option<String> {
        self.designate(date, action)?.ok()?.1
    }

    /// The designated due date, with the shortfall an interpretation
    /// tolerates.
    fn designate(
        &self,
        date: &str,
        action: &Expr,
    ) -> Option<Result<(String, Option<String>), Undesignated>> {
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
                    "notice due date not recorded: `{event}` on {date} designates the due date of `{}` (its `{}`), and the matter records it as `?`; it is the notice's to set, never counted in its place",
                    self.meta.id, d.name
                ))))
            }
        };
        let floor = crate::types::date::add_days(date, self.amount).unwrap_or_default();
        if designated >= floor {
            return Some(Ok((designated, None)));
        }
        let cite = self
            .meta
            .cite
            .as_ref()
            .map_or_else(String::new, |c| format!(" ({c})"));
        Some(match &d.interpretation {
            None => Err(Undesignated::BelowFloor(format!(
                "`{event}` on {date} designates `{}` due {designated}, earlier than the least time the rule allows, {} calendar days from {date} ({floor}){cite}",
                self.meta.id, self.amount
            ))),
            Some(why) => {
                let short = format!(
                    "`{event}` on {date} designates `{}` due {designated}, earlier than {} calendar days from {date} ({floor}){cite}; the date stands: the rule names no start for its least time, and measuring it from {date} is an interpretation: {why}",
                    self.meta.id, self.amount
                );
                Ok((designated, Some(short)))
            }
        })
    }
}

/// The rule by which another forum's rules set a deadline's period, a
/// statute giving one court's appeals the time prescribed for another's
/// (SPEC §5.3).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Adoption {
    /// Its cite (a cite hole's reason).
    pub cite: String,
    /// The cite is marked `historical`.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub historical: bool,
    /// The words of the cited rule that adopt the period, which must
    /// appear in its vendored text.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quote: Option<String>,
}

/// One term of a period derived from several rules (SPEC §5.3): `derived
/// 3.months cite "MPEP 710.02(b)"` beside `derived 1.months cite "MPEP
/// 710.01(a)"` for a four-month extended reply period.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Derivation {
    /// The term's length.
    pub amount: u64,
    /// Its unit.
    pub unit: DayUnit,
    /// The rule stating it (a cite hole's reason).
    pub cite: String,
    /// The cite is marked `historical`.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub historical: bool,
    /// The words of the cited rule that state the term where it states no
    /// count (`on the prior Monday`), which its vendored text must hold in
    /// place of the count.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quote: Option<String>,
}

/// The trigger parameter designating a deadline's due date (SPEC §5.3).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Designated {
    /// The qualified id of the trigger's event (the notice or order).
    pub event: String,
    /// Its index among the trigger event's parameters.
    pub param: usize,
    /// Its name.
    pub name: String,
    /// The interpretation measuring the least time from the trigger's
    /// date, where the rule names no start (`interpretation "..."`): a
    /// date short of it is a warning naming this, not a rejection.
    /// `None`: the rule names the start (37 C.F.R. § 2.105(a), "from the
    /// mailing date of the notice"), and a date short of it is rejected.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub interpretation: Option<String>,
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

/// The character of a time limit (design review §2.4), an index of the
/// deadline's type: a jurisdictional limit admits no waiver, no unbounded
/// extension and no delegation (*Bowles v. Russell*, 551 U.S. 205
/// (2007)); missing a claim-processing limit is an objection the opponent
/// may raise or forfeit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Characterization {
    /// Which character.
    pub kind: CharacterKind,
    /// The authority characterizing it (`cite`); `None`: the deadline's.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cite: Option<String>,
}

/// Jurisdictional or claim-processing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CharacterKind {
    /// No waiver, forfeiture or equitable tolling.
    Jurisdictional,
    /// Enforced when the opponent objects; forfeitable.
    ClaimProcessing,
}

impl CharacterKind {
    /// Its identifier in source.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::Jurisdictional => "jurisdictional",
            Self::ClaimProcessing => "claim_processing",
        }
    }
}

/// An emergency capability grant (`grant g on [d, ...] { ... }`, design
/// review §2.4): for its bounded `effective` window, the targets carry
/// its capabilities as if declared, and `unlock` lifts their locks.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Grant {
    /// Declaration metadata (its window is `meta.effective`).
    pub meta: Metadata,
    /// The deadlines it reaches.
    pub targets: Vec<String>,
    /// Lifts each target's `extend { never ... }` lock in the window.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub unlock: bool,
    /// Extension grants added in the window.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub extend: Vec<Extension>,
    /// Waiver capability added in the window.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub waive: Option<Capability>,
    /// Delegations added in the window.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub delegations: Vec<super::logic::Delegation>,
}
