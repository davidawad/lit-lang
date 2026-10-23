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

/// An exact rational, or `None` once exact arithmetic overflowed: the
/// overflow travels to the end of the decision (`probability`), which
/// reports it once.
type X = Option<Rational>;

fn add(a: X, b: X) -> X {
    a.zip(b).and_then(|(a, b)| q::add(a, b).ok())
}

fn mul(a: X, b: X) -> X {
    a.zip(b).and_then(|(a, b)| q::mul(a, b).ok())
}

fn div(a: X, b: X) -> X {
    a.zip(b).and_then(|(a, b)| q::div(a, b).ok())
}

/// Renders a value; every rendered term was decided first, so none has
/// overflowed.
fn show(x: X) -> String {
    x.map(q::display).unwrap_or_default()
}

/// One unknown of a linear form: its rendering, coefficient and box.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Term {
    label: String,
    coeff: X,
    lo: X,
    hi: X,
}

/// An exact linear form `constant + Σ coeff * atom` over boxed atoms.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Linear {
    constant: X,
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
            constant: Some(c),
            terms: BTreeMap::new(),
        }
    }

    fn atom(key: String, label: String, lo: X, hi: X) -> Self {
        let term = Term {
            label,
            coeff: Some(Rational::integer(1)),
            lo,
            hi,
        };
        Self {
            constant: Some(Rational::integer(0)),
            terms: BTreeMap::from([(key, term)]),
        }
    }

    fn is_constant(&self) -> bool {
        self.terms.is_empty()
    }

    fn add(mut self, other: Self) -> Self {
        self.constant = add(self.constant, other.constant);
        for (key, t) in other.terms {
            let mine = self.terms.entry(key).or_insert(Term {
                coeff: Some(Rational::integer(0)),
                ..t.clone()
            });
            mine.coeff = add(mine.coeff, t.coeff);
        }
        self.terms
            .retain(|_, t| t.coeff != Some(Rational::integer(0)));
        self
    }

    fn scale(mut self, k: X) -> Self {
        self.constant = mul(self.constant, k);
        for t in self.terms.values_mut() {
            t.coeff = mul(t.coeff, k);
        }
        self.terms
            .retain(|_, t| t.coeff != Some(Rational::integer(0)));
        self
    }

    /// The bound of the form with each atom at the vertex that minimises
    /// (`upper = false`) or maximises it, and that vertex.
    fn extreme(&self, upper: bool) -> (X, Vec<(String, X)>) {
        let mut value = self.constant;
        let mut at = Vec::new();
        for t in self.terms.values() {
            let positive = t.coeff.is_some_and(|c| c.numerator > 0);
            let x = if positive == upper { t.hi } else { t.lo };
            value = add(value, mul(t.coeff, x));
            at.push((t.label.clone(), x));
        }
        (value, at)
    }

    /// Exact range `[lo, hi]` over the atoms' boxes.
    fn range(&self) -> (X, X) {
        (self.extreme(false).0, self.extreme(true).0)
    }

    /// Source-like rendering: `1 - p`, `a + b`, `1/2 * prob(d, A)`. Only
    /// a term over unknowns is rendered: a constant is decided directly.
    fn render(&self) -> String {
        let mut out = String::new();
        if self.constant != Some(Rational::integer(0)) {
            out.push_str(&show(self.constant));
        }
        for t in self.terms.values() {
            let negative = t.coeff.is_some_and(|c| c.numerator < 0);
            let magnitude = if negative {
                mul(t.coeff, Some(Rational::integer(-1)))
            } else {
                t.coeff
            };
            if !out.is_empty() {
                out.push_str(if negative { " - " } else { " + " });
            } else if negative {
                out.push('-');
            }
            if magnitude != Some(Rational::integer(1)) {
                let _ = write!(out, "{} * ", show(magnitude));
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
            Some(Rational::integer(0)),
            Some(Rational::integer(1)),
        ),
    }
}

/// The form of `e = l op r` (`op` one of `+ - * /`), or the divisor
/// violation found on the way.
///
/// # Errors
/// A constant divisor is 0.
pub fn combine(
    op: BinaryOp,
    l: Linear,
    r: Linear,
    e: &Expr,
) -> Result<Result<Linear, Violation>, ArithError> {
    Ok(Ok(match op {
        BinaryOp::Add => l.add(r),
        BinaryOp::Sub => l.add(r.scale(Some(Rational::integer(-1)))),
        BinaryOp::Mul if l.is_constant() => {
            let k = l.constant;
            r.scale(k)
        }
        BinaryOp::Mul if r.is_constant() => {
            let k = r.constant;
            l.scale(k)
        }
        BinaryOp::Mul => {
            let ((a, b), (c, d)) = (l.range(), r.range());
            let label = format!("({}) * ({})", l.render(), r.render());
            hull(e, label, [mul(a, c), mul(a, d), mul(b, c), mul(b, d)])
        }
        _ => return quotient(l, &r, e),
    }))
}

/// The form of `e = l / r`.
fn quotient(l: Linear, r: &Linear, e: &Expr) -> Result<Result<Linear, Violation>, ArithError> {
    if r.is_constant() {
        if r.constant == Some(Rational::integer(0)) {
            return Err(ArithError::DivisionByZero);
        }
        return Ok(Ok(l.scale(div(Some(Rational::integer(1)), r.constant))));
    }
    let ((a, b), (c, d)) = (l.range(), r.range());
    // A `Probability` divisor was itself decided into [0, 1] (so its range
    // is known), and holds 0 exactly when its lower bound is 0.
    let zero = Rational::integer(0);
    if c == Some(zero) {
        return Ok(Err(Violation::Divisor {
            form: r.render(),
            lo: zero,
            hi: d.unwrap_or(zero),
        }));
    }
    let label = format!("({}) / ({})", l.render(), r.render());
    Ok(Ok(hull(
        e,
        label,
        [div(a, c), div(a, d), div(b, c), div(b, d)],
    )))
}

/// A fresh atom for `e`, shown as `label`, boxed by the hull of `corners`
/// (unknown when any corner overflowed).
fn hull(e: &Expr, label: String, corners: [X; 4]) -> Linear {
    let all: Option<Vec<Rational>> = corners.into_iter().collect();
    let lo = all
        .as_ref()
        .and_then(|v| v.iter().copied().min_by(|x, y| q::cmp(*x, *y)));
    let hi = all.and_then(|v| v.into_iter().max_by(|x, y| q::cmp(*x, *y)));
    Linear::atom(key(e), label, lo, hi)
}

/// Decides `form : Probability`: `None` when every assignment of its
/// atoms keeps it in `[0, 1]`.
///
/// # Errors
/// Exact arithmetic overflowed while normalising or evaluating a bound.
pub fn probability(form: &Linear) -> Result<Option<Violation>, ArithError> {
    let known = |(v, at): (X, Vec<(String, X)>)| {
        let at: Option<Vec<(String, Rational)>> =
            at.into_iter().map(|(l, x)| x.map(|x| (l, x))).collect();
        v.zip(at)
    };
    let (Some((lo, at_lo)), Some((hi, at_hi))) =
        (known(form.extreme(false)), known(form.extreme(true)))
    else {
        return Err(ArithError::Overflow);
    };
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
