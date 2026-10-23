//! Ground refinements (SPEC §6, §13.1; design review §2.2, tier 1).
//!
//! `Probability` is the refinement `{q: Q | 0 <= q <= 1}`. A constant is
//! checked by evaluating it. A term that does not fold (a reference to a
//! `Probability` default, `prob` of a distribution that is not constant, a
//! `match`) is an *atom*: an unknown that its own type keeps in a box
//! `[lo, hi]`. Arithmetic over atoms is normalised to an exact linear form
//! `c + k1*x1 + ... + kn*xn`. Over a box a linear form attains its extremes
//! at a vertex, so its exact range, and an assignment attaining a violated
//! bound, follow from the coefficients alone: deciding the refinement needs
//! no solver, only exact rational evaluation. The same atom (one name)
//! denotes one value, so `p + (1 - p)` is exactly 1.
//!
//! A product or quotient of two non-constant terms leaves the linear
//! fragment. It becomes a fresh atom boxed by exact interval arithmetic,
//! which is sound (it contains every value the term takes) though not
//! complete. A divisor whose range contains 0 is rejected: it may divide by
//! zero.
use super::expr::Checked;
use super::rational::{self as q, ArithError};
use crate::hir::{BinaryOp, Expr, ExprKind, Rational, Value};
use std::collections::BTreeMap;
use std::fmt::Write as _;

/// One unknown of a linear form: its rendering, coefficient and box.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Term {
    label: String,
    coeff: Rational,
    lo: Rational,
    hi: Rational,
}

/// An exact linear form `constant + Σ coeff * atom` over boxed atoms.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Linear {
    constant: Rational,
    /// Atoms keyed by a span-free identity, so a name used twice is one
    /// unknown.
    terms: BTreeMap<String, Term>,
}

/// Why a `Probability` term fails its refinement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Violation {
    /// The term can leave `[0, 1]`.
    Range {
        /// The normalised term.
        form: String,
        /// Its exact range.
        lo: Rational,
        /// Its exact range.
        hi: Rational,
        /// Atom values attaining the violated bound.
        witness: Vec<(String, Rational)>,
        /// The value at the witness.
        value: Rational,
    },
    /// A non-constant divisor can be 0.
    Divisor {
        /// The normalised divisor.
        form: String,
        /// Its exact range.
        lo: Rational,
        /// Its exact range.
        hi: Rational,
    },
}

impl Violation {
    /// Main message.
    #[must_use]
    pub fn message(&self) -> String {
        match self {
            Self::Range { form, lo, hi, .. } => format!(
                "probability `{form}` is not provably in [0, 1]: it ranges over [{}, {}]",
                q::display(*lo),
                q::display(*hi)
            ),
            Self::Divisor { form, lo, hi } => format!(
                "divisor `{form}` may be 0: it ranges over [{}, {}]",
                q::display(*lo),
                q::display(*hi)
            ),
        }
    }

    /// The refinement and the ground counterexample, as a note.
    #[must_use]
    pub fn note(&self) -> String {
        match self {
            Self::Range {
                form,
                witness,
                value,
                ..
            } => {
                let at = witness
                    .iter()
                    .map(|(x, v)| format!("{x} = {}", q::display(*v)))
                    .collect::<Vec<_>>()
                    .join(", ");
                let side = if q::cmp(*value, Rational::integer(0)).is_lt() {
                    "< 0"
                } else {
                    "> 1"
                };
                format!(
                    "refinement `Probability = {{q | 0 <= q <= 1}}` fails when {at}: `{form}` = {} {side}",
                    q::display(*value)
                )
            }
            Self::Divisor { .. } => {
                "refinement `{d | d != 0}` on a divisor fails; divide by a constant, or by a term bounded away from 0"
                    .to_owned()
            }
        }
    }
}

impl Linear {
    fn constant(c: Rational) -> Self {
        Self {
            constant: c,
            terms: BTreeMap::new(),
        }
    }

    fn atom(key: String, label: String, lo: Rational, hi: Rational) -> Self {
        let term = Term {
            label,
            coeff: Rational::integer(1),
            lo,
            hi,
        };
        Self {
            constant: Rational::integer(0),
            terms: BTreeMap::from([(key, term)]),
        }
    }

    fn as_constant(&self) -> Option<Rational> {
        self.terms.is_empty().then_some(self.constant)
    }

    fn add(mut self, other: Self) -> Result<Self, ArithError> {
        self.constant = q::add(self.constant, other.constant)?;
        for (key, t) in other.terms {
            match self.terms.get_mut(&key) {
                Some(mine) => mine.coeff = q::add(mine.coeff, t.coeff)?,
                None => {
                    self.terms.insert(key, t);
                }
            }
        }
        self.terms.retain(|_, t| t.coeff.numerator != 0);
        Ok(self)
    }

    fn scale(mut self, k: Rational) -> Result<Self, ArithError> {
        self.constant = q::mul(self.constant, k)?;
        for t in self.terms.values_mut() {
            t.coeff = q::mul(t.coeff, k)?;
        }
        self.terms.retain(|_, t| t.coeff.numerator != 0);
        Ok(self)
    }

    /// The bound of the form with each atom at the vertex that minimises
    /// (`upper = false`) or maximises it, and that vertex.
    fn extreme(&self, upper: bool) -> Result<(Rational, Vec<(String, Rational)>), ArithError> {
        let mut value = self.constant;
        let mut at = Vec::new();
        for t in self.terms.values() {
            let positive = t.coeff.numerator > 0;
            let x = if positive == upper { t.hi } else { t.lo };
            value = q::add(value, q::mul(t.coeff, x)?)?;
            at.push((t.label.clone(), x));
        }
        Ok((value, at))
    }

    /// Exact range `[lo, hi]` over the atoms' boxes.
    fn range(&self) -> Result<(Rational, Rational), ArithError> {
        Ok((self.extreme(false)?.0, self.extreme(true)?.0))
    }

    /// Source-like rendering: `1 - p`, `a + b`, `1/2 * prob(d, A)`.
    fn render(&self) -> String {
        let mut out = String::new();
        let zero = Rational::integer(0);
        if self.constant != zero || self.terms.is_empty() {
            out.push_str(&q::display(self.constant));
        }
        for t in self.terms.values() {
            let negative = t.coeff.numerator < 0;
            let magnitude = if negative {
                q::neg(t.coeff).unwrap_or(t.coeff)
            } else {
                t.coeff
            };
            if !out.is_empty() {
                out.push_str(if negative { " - " } else { " + " });
            } else if negative {
                out.push('-');
            }
            if magnitude != Rational::integer(1) {
                let _ = write!(out, "{} * ", q::display(magnitude));
            }
            out.push_str(&t.label);
        }
        out
    }
}

/// Short label of an atom for messages.
fn label(e: &Expr) -> String {
    match &e.kind {
        ExprKind::Ref { target } => target.rsplit("::").next().unwrap_or(target).to_owned(),
        ExprKind::Call { callee, args } => {
            let args: Vec<String> = args.iter().map(label).collect();
            format!("{callee}({})", args.join(", "))
        }
        _ => "<expr>".to_owned(),
    }
}

/// Span-free identity of an atom: the same term denotes the same value
/// (the language is pure), so `a * b` used twice is one unknown.
fn key(e: &Expr) -> String {
    match &e.kind {
        ExprKind::Ref { target } => target.clone(),
        ExprKind::Literal { value } => format!("{value:?}"),
        ExprKind::Call { callee, args } => {
            let args: Vec<String> = args.iter().map(key).collect();
            format!("{callee}({})", args.join(", "))
        }
        ExprKind::Binary { lhs, op, rhs } => format!("({} {op:?} {})", key(lhs), key(rhs)),
        // Spans keep two such terms apart: sound, if not complete.
        kind => format!("{kind:?}"),
    }
}

/// The linear form of a checked `Probability` operand: its constant, the
/// form its own operator built, or an atom in the unit box.
#[must_use]
pub fn operand(c: &Checked) -> Linear {
    match (&c.value, &c.form) {
        (Some(Value::Probability(v)), _) => Linear::constant(*v),
        (_, Some(form)) => form.clone(),
        _ => Linear::atom(
            key(&c.hir),
            label(&c.hir),
            Rational::integer(0),
            Rational::integer(1),
        ),
    }
}

/// The form of `e = l op r` (`op` one of `+ - * /`), or the divisor
/// violation found on the way.
///
/// # Errors
/// Exact arithmetic overflowed while normalising, or a constant divisor
/// is 0.
pub fn combine(
    op: BinaryOp,
    l: Linear,
    r: Linear,
    e: &Expr,
) -> Result<Result<Linear, Violation>, ArithError> {
    Ok(Ok(match op {
        BinaryOp::Add => l.add(r)?,
        BinaryOp::Sub => l.add(r.scale(Rational::integer(-1))?)?,
        BinaryOp::Mul => match (l.as_constant(), r.as_constant()) {
            (Some(k), _) => r.scale(k)?,
            (_, Some(k)) => l.scale(k)?,
            _ => {
                let ((a, b), (c, d)) = (l.range()?, r.range()?);
                let corners = [q::mul(a, c)?, q::mul(a, d)?, q::mul(b, c)?, q::mul(b, d)?];
                hull(e, format!("({}) * ({})", l.render(), r.render()), corners)
            }
        },
        _ => return quotient(l, &r, e),
    }))
}

/// The form of `e = l / r`.
fn quotient(l: Linear, r: &Linear, e: &Expr) -> Result<Result<Linear, Violation>, ArithError> {
    if let Some(k) = r.as_constant() {
        return Ok(Ok(l.scale(q::div(Rational::integer(1), k)?)?));
    }
    let ((a, b), (c, d)) = (l.range()?, r.range()?);
    // A `Probability` divisor was itself checked into [0, 1], so its range
    // holds 0 exactly when its lower bound is 0.
    if c.numerator == 0 {
        return Ok(Err(Violation::Divisor {
            form: r.render(),
            lo: c,
            hi: d,
        }));
    }
    let corners = [q::div(a, c)?, q::div(a, d)?, q::div(b, c)?, q::div(b, d)?];
    Ok(Ok(hull(
        e,
        format!("({}) / ({})", l.render(), r.render()),
        corners,
    )))
}

/// A fresh atom for `e`, shown as `label`, boxed by the hull of `corners`.
fn hull(e: &Expr, label: String, corners: [Rational; 4]) -> Linear {
    let [first, rest @ ..] = corners;
    let (lo, hi) = rest.into_iter().fold((first, first), |(lo, hi), x| {
        (
            if q::cmp(x, lo).is_lt() { x } else { lo },
            if q::cmp(x, hi).is_gt() { x } else { hi },
        )
    });
    Linear::atom(key(e), label, lo, hi)
}

/// Decides `form : Probability`: `None` when every assignment of its
/// atoms keeps it in `[0, 1]`.
///
/// # Errors
/// Exact arithmetic overflowed while evaluating a bound.
pub fn probability(form: &Linear) -> Result<Option<Violation>, ArithError> {
    let (lo, at_lo) = form.extreme(false)?;
    let (hi, at_hi) = form.extreme(true)?;
    let below = q::cmp(lo, Rational::integer(0)).is_lt();
    let above = q::cmp(hi, Rational::integer(1)).is_gt();
    Ok((below || above).then(|| {
        let (witness, value) = if below { (at_lo, lo) } else { (at_hi, hi) };
        Violation::Range {
            form: form.render(),
            lo,
            hi,
            witness,
            value,
        }
    }))
}
