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
    /// A court-wide order: a general order, a chief judge's standing order
    /// (S.D.N.Y. M10-468, COFC General Order 2025-02). Court-scoped where a
    /// standing order is judge-scoped; on the same rung, since both
    /// regulate practice consistently with the rules (FRCP 83(b), RCFC
    /// 83(b)).
    GeneralOrder,
    /// A judge's standing order.
    StandingOrder,
    /// An order in one case.
    CaseOrder,
    /// A Judicial Conference emergency declaration (FRCP 87, Fed. R. Crim. P. 62,
    /// FRAP 2(b)): it grants capabilities, such as extending a locked
    /// period, to the courts it designates for a bounded window, and does
    /// nothing else.
    EmergencyDeclaration,
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
    pub const NAMES: [&'static str; 10] = [
        "base",
        "amendment",
        "local-rule",
        "general-order",
        "standing-order",
        "case-order",
        "emergency-declaration",
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
            Self::GeneralOrder => "general-order",
            Self::StandingOrder => "standing-order",
            Self::CaseOrder => "case-order",
            Self::EmergencyDeclaration => "emergency-declaration",
            Self::Guidance { .. } => "guidance",
            Self::Precedent { .. } => "precedent",
            Self::Overruling => "overruling",
        }
    }

    /// The class named `name` in `Lit.toml`: guidance binding no one yet,
    /// precedent at its widest (binding) scope. `None` for an unknown name.
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        Some(match name {
            "base" => Self::Base,
            "amendment" => Self::Amendment,
            "local-rule" => Self::LocalRule,
            "general-order" => Self::GeneralOrder,
            "standing-order" => Self::StandingOrder,
            "case-order" => Self::CaseOrder,
            "emergency-declaration" => Self::EmergencyDeclaration,
            "guidance" => Self::Guidance { binds: Vec::new() },
            "precedent" => Self::Precedent {
                scope: Scope::Binding,
            },
            "overruling" => Self::Overruling,
            _ => return None,
        })
    }

    /// The authority level a modification layer's package must have: its
    /// class names one rung of the `lit/core` lattice. `None` for the
    /// classes the lattice does not name.
    #[must_use]
    pub fn level(&self) -> Option<&'static str> {
        match self {
            Self::LocalRule => Some("LocalRule"),
            Self::GeneralOrder | Self::StandingOrder => Some("StandingOrder"),
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
            // Court-wide practice: no matter facts, no priors of its own.
            Self::GeneralOrder => Row::of(&[E::Declare, E::Prescribe, E::Override, E::Waive]),
            Self::EmergencyDeclaration => Row::of(&[E::Declare, E::GrantCap]),
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
    pub fn classes() -> [Self; 11] {
        [
            Self::Base,
            Self::Amendment,
            Self::LocalRule,
            Self::GeneralOrder,
            Self::StandingOrder,
            Self::CaseOrder,
            Self::EmergencyDeclaration,
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
            Self::Interpret => "defeat another package's default or read its deadline",
            Self::Recharacterize => "recharacterize another package's deadline",
            Self::Invalidate => "invalidate another instrument's claims",
            Self::Waive => "waive another package's requirement",
            Self::GrantCap => "give a capability",
            Self::SetPrior => "set the prior of a draw",
            Self::AddFact => "state a matter fact",
        }
    }

    /// The effect named `name` ([`Effect::name`]).
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|e| e.name() == name)
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

    /// The effects of `self` and of `other`.
    #[must_use]
    pub fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    /// Whether every effect of `self` is in `other`.
    #[must_use]
    pub fn within(self, other: Self) -> bool {
        self.0 & !other.0 == 0
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
    /// The issuing court's id in the forum-family matrix (`scotus`, `ca9`):
    /// precedent and overruling only, where binding scope is computed from
    /// it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub court: Option<String>,
    /// When a precedent was designated precedential (a board's designation
    /// order; a published opinion's date where the legend is the
    /// designation).
    /// Absent, the decision is nonprecedential and binds nowhere.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub designated: Option<String>,
    /// The precedent packages an overruling withdraws.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub overrules: Vec<String>,
    /// A precedent's binding reach: the forum families (`[family.*]` ids
    /// of `core`) where its court's precedent binds, as the court
    /// hierarchy computes it from `court` and `designated`; empty, it is
    /// persuasive everywhere.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reach: Option<Vec<String>>,
    /// A precedent's holdings: only a holding edits a rule element, so a
    /// declaration editing an element no holding names is dicta (L2016).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub holdings: Vec<Holding>,
    /// Whether the decision is the court's en banc: the only way a court
    /// sitting in panels overrules its own precedent (Miller v. Gammie,
    /// 335 F.3d 889, 899-900 (9th Cir. 2003) (en banc); 28 U.S.C. § 46(c)).
    #[serde(default, skip_serializing_if = "is_no")]
    pub en_banc: bool,
    /// The precedent packages a decision of a court whose precedent binds
    /// theirs abrogates without overruling them: its reasoning is "clearly
    /// irreconcilable" with theirs (Miller v. Gammie, 335 F.3d at 900), so
    /// they are withdrawn from its date as an overruling's are.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub abrogates: Vec<String>,
    /// False for a synthetic what-if layer: it may appear only in a
    /// counterfactual world (design review §2.5 item 2).
    #[serde(default = "yes", skip_serializing_if = "is_yes")]
    pub authoritative: bool,
}

/// One holding of a precedent (`[[package.instrument.holdings]]`): what
/// the court held, the opinion page it is held at, and the one rule
/// element it edits.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Holding {
    /// The holding, in the opinion's words.
    pub holding: String,
    /// The pin cite to the opinion page (`79 F.3d at 1215`).
    pub pin: String,
    /// The rule element it edits, as `pack::id` (a declaration of another
    /// package, or the precedent's own declaration of the doctrine it
    /// announces).
    pub edits: String,
}

fn yes() -> bool {
    true
}

#[allow(clippy::trivially_copy_pass_by_ref)] // serde's skip_serializing_if signature
fn is_yes(b: &bool) -> bool {
    *b
}

#[allow(clippy::trivially_copy_pass_by_ref)] // serde's skip_serializing_if signature
fn is_no(b: &bool) -> bool {
    !*b
}

/// One forum family's column of the class matrix (`[family.*]` in
/// `lit/core`'s `Lit.toml`; docs/litigate/CHANGE.md §3): which classes
/// exist there, what each may do there, and whose precedent binds there.
/// Data, not compiler code (design review §2.5 item 4).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Family {
    /// Its id (`district-civil`, `circuit-appellate`).
    pub id: String,
    /// What it is.
    pub title: String,
    /// The package `forum`s in it: those of the workspace's packages
    /// declaring it (`package.family`, filled by
    /// [`crate::change::families`]; empty as `core` parses it).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub forums: Vec<String>,
    /// The courts whose precedential decisions bind in it; `@circuit` is
    /// the regional circuit of a world's location, `@state` the courts
    /// its state's entry in `states` names.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub precedent: Vec<String>,
    /// The values a location's `circuit` takes in it.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub circuits: Vec<String>,
    /// The courts that sit in panels, a panel bound by its court's prior
    /// panels: only the court en banc (or a court whose precedent binds
    /// it) overrules one of its precedents (Miller v. Gammie, 335 F.3d
    /// 889, 899-900 (9th Cir. 2003) (en banc)).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub panels: Vec<String>,
    /// The values a location's `state` takes in it, each with the courts
    /// whose precedent binds there.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub states: Vec<StateRule>,
    /// One cell per class that exists in it.
    pub classes: Vec<Cell>,
}

/// A state of a family (`[family.*.state.*]`): its own courts bind only
/// in it, and on a federal question its courts follow its own rule of
/// whose federal decisions bind them.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StateRule {
    /// Its id (the postal code: `nj`, `ca`).
    pub id: String,
    /// Its courts whose precedential decisions bind throughout the state.
    pub courts: Vec<String>,
    /// The federal courts, besides the Supreme Court, whose decisions on a
    /// federal question bind its courts: none where they are persuasive
    /// only (People v. Kin Kan, 78 N.Y.2d 54, 59-60 (1991)).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub federal: Vec<String>,
    /// The decisions that settle both.
    pub cite: String,
}

/// Where in a family law is applied, for whose precedent binds there: a
/// world's location, or the seat of a court ([`Family::seat`]). `None`
/// coordinates mean anywhere in the family.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Place<'a> {
    /// The regional circuit.
    pub circuit: Option<&'a str>,
    /// The state.
    pub state: Option<&'a str>,
}

/// One cell of the matrix: a class that exists in a forum family.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Cell {
    /// The class (a [`Kind::NAMES`] entry).
    pub class: String,
    /// The instrument that makes the class exist in the family.
    pub cite: String,
    /// A real instrument of the class there (a URL).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub example: Option<String>,
    /// The effects the class has in the family, when narrower than its
    /// row ([`Kind::row`]); absent, the row.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub row: Option<Vec<Effect>>,
    /// The longest window, in days, a grant of the class may have
    /// (FRCP 87(b)(1)(C): an emergency declaration's 90 days).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub window_days: Option<u32>,
}

impl Family {
    /// Its cell for `kind`, if the class exists in it.
    #[must_use]
    pub fn cell(&self, kind: &Kind) -> Option<&Cell> {
        self.classes.iter().find(|c| c.class == kind.name())
    }

    /// Whether a precedential decision of `court` binds in the family at
    /// `at` (a world's location); a coordinate left out, whether it binds
    /// at some value of it. The Supreme Court binds wherever the family
    /// lists it; a circuit only at itself; a state's courts only in that
    /// state, and a federal court in a state only as its own rule says.
    #[must_use]
    pub fn binds(&self, court: &str, at: Place<'_>) -> bool {
        self.precedent.iter().any(|p| match p.as_str() {
            "@circuit" => at
                .circuit
                .map_or_else(|| self.circuits.iter().any(|c| c == court), |c| c == court),
            "@state" => self
                .states
                .iter()
                .filter(|s| at.state.is_none_or(|id| s.id == id))
                .any(|s| s.courts.iter().chain(&s.federal).any(|c| c == court)),
            p => p == court,
        })
    }

    /// Where `court` sits in the family: its circuit, or the state whose
    /// courts it is among.
    #[must_use]
    pub fn seat<'a>(&'a self, court: &'a str) -> Place<'a> {
        Place {
            circuit: self.circuits.iter().any(|c| c == court).then_some(court),
            state: self
                .states
                .iter()
                .find(|s| s.courts.iter().any(|c| c == court))
                .map(|s| s.id.as_str()),
        }
    }

    /// Whether `court` sits in panels bound by its prior panels.
    #[must_use]
    pub fn sits_in_panels(&self, court: &str) -> bool {
        self.panels.iter().any(|c| c == court)
    }
}

impl Cell {
    /// The effects the class has here: its `row`, else `ceiling`.
    #[must_use]
    pub fn row(&self, ceiling: Row) -> Row {
        self.row.as_ref().map_or(ceiling, |r| Row::of(r))
    }
}
