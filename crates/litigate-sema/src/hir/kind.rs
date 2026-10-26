//! Instrument classes as a capability index (SPEC §4.1, §5.7; design
//! review 2026-10-23 §2.5): a package's type carries the class of the
//! instrument it encodes, each class has an effect row, and a declaration
//! type-checks only when every effect it has is in its package's row.
use serde::{Deserialize, Serialize};

/// The instrument class of a package (`kind` in `Lit.toml`). `hypothetical`
/// is not a class: it is a modality of a world (design review §2.5 item 2),
/// and a counterfactual world may hold a layer of any class.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "class", rename_all = "kebab-case")]
#[non_exhaustive]
pub enum Kind {
    /// Primary law as enacted or promulgated (a statute, a court's rules
    /// of procedure, an agency's rules of practice): every effect.
    #[default]
    Base,
    /// An amendment of base text, on its own `effective` dates.
    Amendment,
    /// A court's local rules (FRCP 83(a)(1)).
    LocalRule,
    /// A judge's standing order.
    StandingOrder,
    /// An order in one case.
    CaseOrder,
    /// Guidance binding the actors `binds` and no one else (design review
    /// §2.5 item 3): it may set the priors of their draws and nothing
    /// else, never a party's duty.
    Guidance {
        /// The bound actors (`[package.roles]` keys).
        binds: Vec<String>,
    },
    /// A judicial decision, binding or persuasive where it is applied.
    Precedent {
        /// Its effect class.
        scope: Scope,
    },
    /// A decision overruling a precedent: it retracts the precedent's
    /// claims (design review §3.4).
    Overruling,
}

/// The effect class of a precedent where it is applied. Binding scope is
/// computed from the court hierarchy and a world's location index, never
/// declared (design review §2.5 item 1), so a typo cannot make persuasive
/// authority binding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Scope {
    /// Controls the courts it binds: interprets, recharacterizes,
    /// invalidates, waives (with a capability) and moves priors.
    Binding,
    /// Evidence only: it may move a prior and nothing else.
    Persuasive,
}

impl Kind {
    /// Every class name `Lit.toml` accepts, in declaration order.
    pub const NAMES: [&'static str; 8] = [
        "base",
        "amendment",
        "local-rule",
        "standing-order",
        "case-order",
        "guidance",
        "precedent",
        "overruling",
    ];

    /// True for [`Kind::Base`] (the default, omitted from HIR JSON).
    #[must_use]
    pub fn is_base(&self) -> bool {
        *self == Self::Base
    }

    /// Its `Lit.toml` name.
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            Self::Base => "base",
            Self::Amendment => "amendment",
            Self::LocalRule => "local-rule",
            Self::StandingOrder => "standing-order",
            Self::CaseOrder => "case-order",
            Self::Guidance { .. } => "guidance",
            Self::Precedent { .. } => "precedent",
            Self::Overruling => "overruling",
        }
    }

    /// The authority level a modification layer's package must have: its
    /// class names one rung of the `lit/core` lattice. `None` for the
    /// classes the lattice does not name.
    #[must_use]
    pub fn level(&self) -> Option<&'static str> {
        match self {
            Self::LocalRule => Some("LocalRule"),
            Self::StandingOrder => Some("StandingOrder"),
            Self::CaseOrder => Some("CaseOrder"),
            _ => None,
        }
    }

    /// The effects a package of this class may have: the forum-independent
    /// ceiling, which a forum family's matrix can only narrow.
    #[must_use]
    pub fn row(&self) -> Row {
        use Effect as E;
        let layer = Row::of(&[
            E::Declare,
            E::Prescribe,
            E::Override,
            E::Interpret,
            E::Waive,
            E::SetPrior,
            E::AddFact,
        ]);
        match self {
            Self::Base => Row::ALL,
            Self::Amendment => Row::ALL.without(E::Rank),
            Self::LocalRule | Self::StandingOrder => layer.with(E::GrantCap),
            Self::CaseOrder => layer,
            Self::Precedent {
                scope: Scope::Binding,
            } => Row::of(&[
                E::Declare,
                E::Interpret,
                E::Recharacterize,
                E::Invalidate,
                E::Waive,
                E::SetPrior,
            ]),
            // Guidance moves only its addressees' priors (`may_set_prior`).
            Self::Guidance { .. }
            | Self::Precedent {
                scope: Scope::Persuasive,
            } => Row::of(&[E::Declare, E::SetPrior]),
            Self::Overruling => Row::of(&[E::Declare, E::Invalidate]),
        }
    }

    /// One value of each class, binding and persuasive precedent apart.
    #[must_use]
    pub fn classes() -> [Self; 9] {
        [
            Self::Base,
            Self::Amendment,
            Self::LocalRule,
            Self::StandingOrder,
            Self::CaseOrder,
            Self::Guidance { binds: Vec::new() },
            Self::Precedent {
                scope: Scope::Binding,
            },
            Self::Precedent {
                scope: Scope::Persuasive,
            },
            Self::Overruling,
        ]
    }

    /// Its name in diagnostics: [`Kind::name`], a precedent's scope added.
    #[must_use]
    pub fn label(&self) -> &'static str {
        match self {
            Self::Precedent {
                scope: Scope::Binding,
            } => "binding precedent",
            Self::Precedent {
                scope: Scope::Persuasive,
            } => "persuasive precedent",
            other => other.name(),
        }
    }

    /// The classes whose row has `effect`, by [`Kind::label`].
    #[must_use]
    pub fn permitting(effect: Effect) -> Vec<&'static str> {
        Self::classes()
            .iter()
            .filter(|k| k.row().has(effect))
            .map(Self::label)
            .collect()
    }

    /// Whether this class may set the prior of a draw by `actor`: its row
    /// has [`Effect::SetPrior`], and guidance binds `actor`.
    #[must_use]
    pub fn may_set_prior(&self, actor: &str) -> bool {
        self.row().has(Effect::SetPrior)
            && match self {
                Self::Guidance { binds } => binds.iter().any(|b| b == actor),
                _ => true,
            }
    }
}

/// What a declaration does to the law a world holds (design review §2.5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Effect {
    /// States structure of its own: a node, a type, an event, a rule, a
    /// default, an exception to its own default, a transition.
    Declare,
    /// Prescribes a time limit or how time is counted: a deadline, a
    /// counting rule, a calendar.
    Prescribe,
    /// Orders authority levels (`authority A > B`).
    Rank,
    /// Replaces another package's declaration in time (`supersedes`,
    /// `revives`).
    Version,
    /// Displaces another package's declaration (`override`,
    /// `notwithstanding`).
    Override,
    /// Defeats another package's default (an exception to it).
    Interpret,
    /// Changes the characterization of another package's deadline.
    Recharacterize,
    /// Annuls, repeals or retracts another instrument's claims.
    Invalidate,
    /// Waives another package's requirement.
    Waive,
    /// Gives a capability: `delegable`, an `extend` or `waive` grant, an
    /// emergency `grant`.
    GrantCap,
    /// Sets the prior of a draw (`p` on a chance node's edge).
    SetPrior,
    /// States a matter fact (a `fact` node).
    AddFact,
}

impl Effect {
    /// Every effect, in declaration order.
    pub const ALL: [Self; 12] = [
        Self::Declare,
        Self::Prescribe,
        Self::Rank,
        Self::Version,
        Self::Override,
        Self::Interpret,
        Self::Recharacterize,
        Self::Invalidate,
        Self::Waive,
        Self::GrantCap,
        Self::SetPrior,
        Self::AddFact,
    ];

    /// Its name in diagnostics.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::Declare => "declare",
            Self::Prescribe => "prescribe",
            Self::Rank => "rank",
            Self::Version => "version",
            Self::Override => "override",
            Self::Interpret => "interpret",
            Self::Recharacterize => "recharacterize",
            Self::Invalidate => "invalidate",
            Self::Waive => "waive",
            Self::GrantCap => "grant-cap",
            Self::SetPrior => "set-prior",
            Self::AddFact => "add-fact",
        }
    }

    /// What it does, completing "cannot ...".
    #[must_use]
    pub fn what(self) -> &'static str {
        match self {
            Self::Declare => "declare",
            Self::Prescribe => "prescribe a time limit or how time is counted",
            Self::Rank => "order authority levels",
            Self::Version => "replace another package's declaration in time",
            Self::Override => "displace another package's declaration",
            Self::Interpret => "defeat another package's default",
            Self::Recharacterize => "recharacterize another package's deadline",
            Self::Invalidate => "invalidate another instrument's claims",
            Self::Waive => "waive another package's requirement",
            Self::GrantCap => "give a capability",
            Self::SetPrior => "set the prior of a draw",
            Self::AddFact => "state a matter fact",
        }
    }

    fn bit(self) -> u16 {
        1 << (self as u16)
    }
}

/// A set of effects. Checking a declaration against its class is set
/// inclusion, linear in the number of effects.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub struct Row(u16);

impl Row {
    /// Every effect.
    pub const ALL: Self = Self((1 << Effect::ALL.len()) - 1);

    /// The row holding exactly `effects`.
    #[must_use]
    pub fn of(effects: &[Effect]) -> Self {
        Self(effects.iter().fold(0, |bits, e| bits | e.bit()))
    }

    /// `self` with `effect`.
    #[must_use]
    pub fn with(self, effect: Effect) -> Self {
        Self(self.0 | effect.bit())
    }

    /// `self` without `effect`.
    #[must_use]
    pub fn without(self, effect: Effect) -> Self {
        Self(self.0 & !effect.bit())
    }

    /// Whether `effect` is in the row.
    #[must_use]
    pub fn has(self, effect: Effect) -> bool {
        self.0 & effect.bit() != 0
    }

    /// The effects of the row, in [`Effect::ALL`] order.
    pub fn effects(self) -> impl Iterator<Item = Effect> {
        Effect::ALL.into_iter().filter(move |e| self.has(*e))
    }
}

/// The instrument a non-base package encodes (`[package.instrument]`):
/// its class must be justified by the instrument itself, never only
/// asserted (design review §2.5 item 1).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Instrument {
    /// The `[sources.*]` key of its vendored text.
    pub source: String,
    /// Who issued it (a court, a judge, an agency).
    pub issuer: String,
    /// The date it was issued or entered.
    pub date: String,
    /// Its designation (an order or rule number, a reporter cite).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub designation: Option<String>,
}
