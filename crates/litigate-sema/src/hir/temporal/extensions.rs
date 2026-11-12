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
        /// The rule authorizing this extension, if cited.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        cite: Option<String>,
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
        /// The interpretation the grant rests on, if the rule reaches the
        /// period only as the author reads it (`interpretation "..."`).
        #[serde(default, skip_serializing_if = "Option::is_none")]
        interpretation: Option<String>,
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
        /// The rule authorizing this extension, if cited.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        cite: Option<String>,
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
        /// The interpretation the grant rests on, if the rule reaches the
        /// period only as the author reads it: FRCP 6(b) applied to a TTAB
        /// period through 37 C.F.R. § 2.116(a), "wherever applicable and
        /// appropriate".
        #[serde(default, skip_serializing_if = "Option::is_none")]
        interpretation: Option<String>,
    },
    /// `extend { never cite "X" }`: the rule `X` forbids extending the
    /// period beyond the deadline's own other grants, so no order, local
    /// rule or stipulation adds one (FRCP 6(b)(2); FRAP 26(b), "except as
    /// authorized in Rule 4"). `lock against [L, ...] cite "X"` is the
    /// named-authority lock (design review §2.4): it forbids the
    /// extension to the lower authorities `L` only, so an emergency grant
    /// of an authority it does not name reaches the period without
    /// `unlock`.
    NonExtendable {
        /// The rule that forbids the extension.
        authority: String,
        /// The authorities the lock binds, in declared order; empty for
        /// `never`, which binds every authority.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        against: Vec<String>,
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

    /// The interpretation the grant rests on; `None` for a grant the
    /// rule's own text makes and for [`Self::NonExtendable`].
    #[must_use]
    pub fn interpretation(&self) -> Option<&str> {
        match self {
            Self::Bounded { interpretation, .. } | Self::Discretionary { interpretation, .. } => {
                interpretation.as_deref()
            }
            Self::NonExtendable { .. } => None,
        }
    }

    /// The authority of a [`Self::NonExtendable`] lock.
    #[must_use]
    pub fn lock(&self) -> Option<&str> {
        self.locked().map(|(authority, _)| authority)
    }

    /// A [`Self::NonExtendable`] lock's authority and the levels it binds
    /// (none named: every level).
    fn locked(&self) -> Option<(&str, &[String])> {
        match self {
            Self::NonExtendable { authority, against } => Some((authority, against)),
            Self::Bounded { .. } | Self::Discretionary { .. } => None,
        }
    }

    /// Whether this is a lock binding authority level `level`: a `never`
    /// lock binds every level, a named lock (`lock against [...]`) the
    /// levels it names.
    #[must_use]
    pub fn locks(&self, level: &str) -> bool {
        self.locked()
            .is_some_and(|(_, against)| against.is_empty() || against.iter().any(|a| a == level))
    }

    /// A lock as written: `extend { never cite "X" }` or `lock against
    /// [L, ...] cite "X"`; `None` for a grant.
    #[must_use]
    pub fn lock_source(&self) -> Option<String> {
        self.locked().map(|(authority, against)| {
            if against.is_empty() {
                format!("extend {{ never cite \"{authority}\" }}")
            } else {
                format!("lock against [{}] cite \"{authority}\"", against.join(", "))
            }
        })
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
    /// Extraordinary circumstances, the exception to a timing requirement
    /// on the application: an application to extend the time to file a
    /// petition for certiorari "must be filed with the Clerk at least 10
    /// days before the date the petition is due, except in extraordinary
    /// circumstances" (Sup. Ct. R. 13.5; 30.2, "the most extraordinary
    /// circumstances").
    ExtraordinaryCircumstances,
}

impl CauseStandard {
    /// Every standard, in declaration order.
    pub const ALL: [Self; 4] = [
        Self::GoodCause,
        Self::ExcusableNeglect,
        Self::GoodCauseOrExcusableNeglect,
        Self::ExtraordinaryCircumstances,
    ];

    /// Its identifier in source.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::GoodCause => "GoodCause",
            Self::ExcusableNeglect => "ExcusableNeglect",
            Self::GoodCauseOrExcusableNeglect => "GoodCauseOrExcusableNeglect",
            Self::ExtraordinaryCircumstances => "ExtraordinaryCircumstances",
        }
    }
}

/// A date an extension bound is counted from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DateRef {
    /// The trigger event's date (`trigger_date`), for calendar-month bounds.
    TriggerDate,
    /// The instance's counted due date, raised to any `later_of` floor
    /// (`prescribed_due`): "the time prescribed by this Rule 4(a)".
    PrescribedDue,
    /// The date of the order granting the extension
    /// (`grant_order_entered`).
    GrantOrderEntered,
    /// The instance's due date as last extended (`current_due`): "the date
    /// the petition is due" (Sup. Ct. R. 13.5), which an earlier extension
    /// has moved.
    CurrentDue,
}

impl DateRef {
    /// Its identifier in source.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::TriggerDate => "trigger_date",
            Self::PrescribedDue => "prescribed_due",
            Self::GrantOrderEntered => "grant_order_entered",
            Self::CurrentDue => "current_due",
        }
    }

    /// Every date reference available to event refinements, in declaration order.
    pub const ALL: [Self; 3] = [
        Self::PrescribedDue,
        Self::GrantOrderEntered,
        Self::CurrentDue,
    ];

    /// Its date among `at`.
    fn of(self, at: BoundDates<'_>) -> &str {
        match self {
            Self::TriggerDate => at.trigger_date,
            Self::PrescribedDue => at.prescribed_due,
            Self::GrantOrderEntered => at.grant_order_entered,
            Self::CurrentDue => at.current_due,
        }
    }
}
