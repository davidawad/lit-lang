//! Queries: declared variables plus labelled quantifier-free formulas over
//! linear integer/rational arithmetic, lowered to one canonical form.
//!
//! Every atom is normalised to `c + Σ aᵢ·xᵢ ⋈ 0` with `⋈ ∈ {<=, <, =}`
//! (`>=`/`>` swap sides, `!=` is `not =`), like terms combined, zero
//! coefficients dropped and variables in name order. The canonical
//! SMT-LIB text ([`Query::to_smtlib`]) is what the cache hashes and what
//! a backend solves, so equal text means an equal question.
use std::collections::BTreeMap;
use std::fmt::Write as _;

use crate::{Error, Rational};

/// The sort of a variable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Sort {
    /// Mathematical integers (dates are day numbers of this sort).
    Int,
    /// Rationals (SMT-LIB `Real`; linear constraints stay rational).
    Real,
}

/// A variable declared in a [`Query`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Var {
    name: String,
    sort: Sort,
}

impl Var {
    /// The variable's name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The variable's sort.
    #[must_use]
    pub const fn sort(&self) -> Sort {
        self.sort
    }
}

/// A linear expression `Σ cᵢ·xᵢ + c`, built infallibly; it is checked
/// (declared variables, exact arithmetic) when asserted.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LinExpr {
    /// `(None, c)` is a constant, `(Some(x), c)` the term `c·x`.
    parts: Vec<(Option<Var>, Rational)>,
}

impl LinExpr {
    /// The empty sum, `0`.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds `coef · var`.
    #[must_use]
    pub fn plus(mut self, coef: impl Into<Rational>, var: &Var) -> Self {
        self.parts.push((Some(var.clone()), coef.into()));
        self
    }

    /// Adds the constant `c`.
    #[must_use]
    pub fn plus_const(mut self, c: impl Into<Rational>) -> Self {
        self.parts.push((None, c.into()));
        self
    }
}

impl From<&Var> for LinExpr {
    fn from(var: &Var) -> Self {
        Self::new().plus(1, var)
    }
}

impl From<Rational> for LinExpr {
    fn from(c: Rational) -> Self {
        Self::new().plus_const(c)
    }
}

impl From<i64> for LinExpr {
    fn from(c: i64) -> Self {
        Self::new().plus_const(c)
    }
}

/// A comparison between two linear expressions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Cmp {
    /// `<=`
    Le,
    /// `<`
    Lt,
    /// `=`
    Eq,
    /// `!=`
    Ne,
    /// `>=`
    Ge,
    /// `>`
    Gt,
}

/// A quantifier-free formula over linear comparisons.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Formula {
    /// `lhs ⋈ rhs`.
    Cmp(LinExpr, Cmp, LinExpr),
    /// Negation.
    Not(Box<Formula>),
    /// Conjunction (empty: true).
    And(Vec<Formula>),
    /// Disjunction (empty: false).
    Or(Vec<Formula>),
}

impl Formula {
    /// `lhs ⋈ rhs`.
    pub fn cmp(lhs: impl Into<LinExpr>, op: Cmp, rhs: impl Into<LinExpr>) -> Self {
        Self::Cmp(lhs.into(), op, rhs.into())
    }

    /// `lhs <= rhs`.
    pub fn le(lhs: impl Into<LinExpr>, rhs: impl Into<LinExpr>) -> Self {
        Self::cmp(lhs, Cmp::Le, rhs)
    }

    /// `lhs < rhs`.
    pub fn lt(lhs: impl Into<LinExpr>, rhs: impl Into<LinExpr>) -> Self {
        Self::cmp(lhs, Cmp::Lt, rhs)
    }

    /// `lhs = rhs`.
    pub fn eq(lhs: impl Into<LinExpr>, rhs: impl Into<LinExpr>) -> Self {
        Self::cmp(lhs, Cmp::Eq, rhs)
    }

    /// `lhs != rhs`.
    pub fn ne(lhs: impl Into<LinExpr>, rhs: impl Into<LinExpr>) -> Self {
        Self::cmp(lhs, Cmp::Ne, rhs)
    }

    /// `lhs >= rhs`.
    pub fn ge(lhs: impl Into<LinExpr>, rhs: impl Into<LinExpr>) -> Self {
        Self::cmp(lhs, Cmp::Ge, rhs)
    }

    /// `lhs > rhs`.
    pub fn gt(lhs: impl Into<LinExpr>, rhs: impl Into<LinExpr>) -> Self {
        Self::cmp(lhs, Cmp::Gt, rhs)
    }

    /// `a ⇒ b`, as `¬a ∨ b`.
    #[must_use]
    pub fn implies(a: Self, b: Self) -> Self {
        Self::Or(vec![!a, b])
    }
}

impl std::ops::Not for Formula {
    type Output = Self;

    /// `¬f`.
    fn not(self) -> Self {
        Self::Not(Box::new(self))
    }
}

/// A normalised relation: `sum ⋈ 0`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Rel {
    Le,
    Lt,
    Eq,
}

/// A normalised atom `constant + Σ coef·var ⋈ 0`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Atom {
    /// Non-zero coefficients by variable name (name order).
    pub(crate) terms: BTreeMap<String, Rational>,
    pub(crate) constant: Rational,
    pub(crate) rel: Rel,
    /// Integer arithmetic: every variable `Int`, every number integral.
    /// Otherwise the atom is over the reals, `Int` variables cast.
    pub(crate) int: bool,
}

/// A lowered formula.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Lowered {
    Atom(Atom),
    Not(Box<Lowered>),
    And(Vec<Lowered>),
    Or(Vec<Lowered>),
}

/// A satisfiability question: declared variables and labelled assertions.
///
/// Labels name assertions in an unsat core. Assertion order is part of the
/// question (and of its content hash); variables are kept in name order.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Query {
    vars: BTreeMap<String, Sort>,
    assertions: Vec<(String, Lowered)>,
}

impl Query {
    /// An empty query (trivially satisfiable).
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Declares (or re-fetches) the integer variable `name`.
    ///
    /// # Errors
    ///
    /// [`Error::InvalidName`] for an empty name or one containing `|` or
    /// `\`; [`Error::SortConflict`] if `name` is already a `Real`.
    pub fn int(&mut self, name: &str) -> Result<Var, Error> {
        self.declare(name, Sort::Int)
    }

    /// Declares (or re-fetches) the rational variable `name`.
    ///
    /// # Errors
    ///
    /// As [`Query::int`], with the sorts swapped.
    pub fn real(&mut self, name: &str) -> Result<Var, Error> {
        self.declare(name, Sort::Real)
    }

    fn declare(&mut self, name: &str, sort: Sort) -> Result<Var, Error> {
        check_name(name)?;
        let declared = *self.vars.entry(name.to_owned()).or_insert(sort);
        if declared == sort {
            Ok(Var {
                name: name.to_owned(),
                sort,
            })
        } else {
            Err(Error::SortConflict(name.to_owned()))
        }
    }

    /// Asserts `formula`, tracked under `label` for unsat cores.
    ///
    /// # Errors
    ///
    /// [`Error::InvalidName`] or [`Error::DuplicateLabel`] for the label;
    /// [`Error::UndeclaredVar`] for a variable not declared here with that
    /// sort; [`Error::Overflow`] when normalising leaves `i64` rationals.
    /// The query is unchanged on error.
    pub fn assert(&mut self, label: &str, formula: &Formula) -> Result<(), Error> {
        check_name(label)?;
        if self.assertions.iter().any(|(l, _)| l == label) {
            return Err(Error::DuplicateLabel(label.to_owned()));
        }
        let lowered = self.lower(formula)?;
        self.assertions.push((label.to_owned(), lowered));
        Ok(())
    }

    /// Declared variables and their sorts, in name order.
    pub fn vars(&self) -> impl Iterator<Item = (&str, Sort)> {
        self.vars.iter().map(|(n, s)| (n.as_str(), *s))
    }

    /// Assertion labels, in assertion order.
    pub fn labels(&self) -> impl Iterator<Item = &str> {
        self.assertions.iter().map(|(l, _)| l.as_str())
    }

    pub(crate) fn assertions(&self) -> &[(String, Lowered)] {
        &self.assertions
    }

    fn lower(&self, f: &Formula) -> Result<Lowered, Error> {
        match f {
            Formula::Cmp(lhs, op, rhs) => self.lower_cmp(lhs, *op, rhs),
            Formula::Not(f) => self.lower(f).map(|l| Lowered::Not(Box::new(l))),
            Formula::And(fs) => self.lower_all(fs).map(Lowered::And),
            Formula::Or(fs) => self.lower_all(fs).map(Lowered::Or),
        }
    }

    fn lower_all(&self, fs: &[Formula]) -> Result<Vec<Lowered>, Error> {
        fs.iter().map(|f| self.lower(f)).collect()
    }

    fn lower_cmp(&self, lhs: &LinExpr, op: Cmp, rhs: &LinExpr) -> Result<Lowered, Error> {
        // `pos ⋈ neg` becomes `pos - neg ⋈ 0`.
        let (pos, neg, rel, negate) = match op {
            Cmp::Le => (lhs, rhs, Rel::Le, false),
            Cmp::Lt => (lhs, rhs, Rel::Lt, false),
            Cmp::Eq => (lhs, rhs, Rel::Eq, false),
            Cmp::Ne => (lhs, rhs, Rel::Eq, true),
            Cmp::Ge => (rhs, lhs, Rel::Le, false),
            Cmp::Gt => (rhs, lhs, Rel::Lt, false),
        };
        let mut terms = BTreeMap::new();
        let mut constant = Rational::ZERO;
        for (sign, side) in [(Rational::ONE, pos), (Rational::MINUS_ONE, neg)] {
            for (var, coef) in &side.parts {
                let coef = coef.checked_mul(sign)?;
                let slot = match var {
                    Some(v) => {
                        self.check_declared(v)?;
                        terms.entry(v.name.clone()).or_insert(Rational::ZERO)
                    }
                    None => &mut constant,
                };
                *slot = slot.checked_add(coef)?;
            }
        }
        terms.retain(|_, c| *c != Rational::ZERO);
        let int = constant.is_integer()
            && terms
                .iter()
                .all(|(n, c)| c.is_integer() && self.vars[n] == Sort::Int);
        let atom = Lowered::Atom(Atom {
            terms,
            constant,
            rel,
            int,
        });
        Ok(if negate {
            Lowered::Not(Box::new(atom))
        } else {
            atom
        })
    }

    fn check_declared(&self, v: &Var) -> Result<(), Error> {
        if self.vars.get(&v.name) == Some(&v.sort) {
            Ok(())
        } else {
            Err(Error::UndeclaredVar(v.name.clone()))
        }
    }

    /// The canonical SMT-LIB 2 script of this query: what the cache
    /// hashes, and a replayable script for any SMT-LIB solver.
    #[must_use]
    pub fn to_smtlib(&self) -> String {
        let mut out = String::from(
            "(set-option :produce-models true)\n\
             (set-option :produce-unsat-cores true)\n\
             (set-logic QF_LIRA)\n",
        );
        for (name, sort) in &self.vars {
            let sort = match sort {
                Sort::Int => "Int",
                Sort::Real => "Real",
            };
            let _ = writeln!(out, "(declare-const |{name}| {sort})");
        }
        for (label, f) in &self.assertions {
            out.push_str("(assert (! ");
            self.render(f, &mut out);
            // `@` keeps labels out of the variables' namespace.
            let _ = writeln!(out, " :named |@{label}|))");
        }
        out.push_str("(check-sat)\n");
        out
    }

    fn render(&self, f: &Lowered, out: &mut String) {
        match f {
            Lowered::Atom(a) => self.render_atom(a, out),
            Lowered::Not(f) => {
                out.push_str("(not ");
                self.render(f, out);
                out.push(')');
            }
            Lowered::And(fs) => self.render_nary("and", "true", fs, out),
            Lowered::Or(fs) => self.render_nary("or", "false", fs, out),
        }
    }

    /// SMT-LIB `and`/`or` take at least two arguments.
    fn render_nary(&self, head: &str, unit: &str, fs: &[Lowered], out: &mut String) {
        match fs {
            [] => out.push_str(unit),
            [f] => self.render(f, out),
            fs => {
                let _ = write!(out, "({head}");
                for f in fs {
                    out.push(' ');
                    self.render(f, out);
                }
                out.push(')');
            }
        }
    }

    fn render_atom(&self, a: &Atom, out: &mut String) {
        let rel = match a.rel {
            Rel::Le => "<=",
            Rel::Lt => "<",
            Rel::Eq => "=",
        };
        let lit = |r: Rational| if a.int { r.smt_int() } else { r.smt_real() };
        let _ = write!(out, "({rel} (+ {}", lit(a.constant));
        for (name, coef) in &a.terms {
            let var = if a.int || self.vars[name] == Sort::Real {
                format!("|{name}|")
            } else {
                format!("(to_real |{name}|)")
            };
            let _ = write!(out, " (* {} {var})", lit(*coef));
        }
        let _ = write!(out, ") {})", lit(Rational::ZERO));
    }
}

fn check_name(name: &str) -> Result<(), Error> {
    if name.is_empty() || name.contains(['|', '\\']) {
        Err(Error::InvalidName(name.to_owned()))
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn declare_is_idempotent_per_sort() {
        let mut q = Query::new();
        let x = q.int("x").unwrap();
        assert_eq!(q.int("x").unwrap(), x);
        assert_eq!((x.name(), x.sort()), ("x", Sort::Int));
        assert_eq!(q.real("x"), Err(Error::SortConflict("x".into())));
        assert_eq!(q.real("p").unwrap().sort(), Sort::Real);
        assert_eq!(
            q.vars().collect::<Vec<_>>(),
            [("p", Sort::Real), ("x", Sort::Int)]
        );
    }

    #[test]
    fn names_and_labels_are_checked() {
        let mut q = Query::new();
        for bad in ["", "a|b", "a\\b"] {
            assert_eq!(q.int(bad), Err(Error::InvalidName(bad.into())));
            assert_eq!(
                q.assert(bad, &Formula::And(vec![])),
                Err(Error::InvalidName(bad.into()))
            );
        }
        q.assert("t", &Formula::And(vec![])).unwrap();
        assert_eq!(
            q.assert("t", &Formula::Or(vec![])),
            Err(Error::DuplicateLabel("t".into()))
        );
        assert_eq!(q.labels().collect::<Vec<_>>(), ["t"]);
    }

    #[test]
    fn undeclared_or_missorted_vars_are_rejected() {
        let mut other = Query::new();
        let y = other.int("y").unwrap();
        let mut q = Query::new();
        assert_eq!(
            q.assert("a", &Formula::ge(&y, 0)),
            Err(Error::UndeclaredVar("y".into()))
        );
        q.real("y").unwrap();
        assert_eq!(
            q.assert("a", &Formula::le(0, &y)),
            Err(Error::UndeclaredVar("y".into()))
        );
        assert_eq!(q.labels().count(), 0, "failed asserts leave no trace");
    }

    #[test]
    fn normalisation_overflow_is_an_error() {
        let mut q = Query::new();
        let x = q.int("x").unwrap();
        // Combining like terms overflows.
        let big = LinExpr::new().plus(i64::MAX, &x).plus(i64::MAX, &x);
        assert_eq!(q.assert("a", &Formula::le(big, 0)), Err(Error::Overflow));
        // Negating the right-hand side overflows.
        let min = LinExpr::new().plus(i64::MIN, &x);
        assert_eq!(q.assert("b", &Formula::le(0, min)), Err(Error::Overflow));
    }

    #[test]
    fn fractional_coefficient_makes_an_int_atom_rational() {
        let mut q = Query::new();
        let x = q.int("x").unwrap();
        let half = Rational::new(1, 2).unwrap();
        q.assert("h", &Formula::le(LinExpr::new().plus(half, &x), 1))
            .unwrap();
        assert!(q
            .to_smtlib()
            .contains("(<= (+ (- 1.0) (* (/ 1.0 2.0) (to_real |x|))) 0.0)"));
    }

    #[test]
    fn canonical_text() {
        let mut q = Query::new();
        let d = q.int("d").unwrap();
        let p = q.real("p").unwrap();
        let x = q.int("x").unwrap();
        let half = Rational::new(1, 2).unwrap();
        q.assert("cmp", &Formula::lt(LinExpr::from(&x).plus(2, &d), 3))
            .unwrap();
        q.assert("mix", &Formula::gt(LinExpr::from(&p).plus(half, &x), half))
            .unwrap();
        q.assert("ne", &Formula::ne(&x, LinExpr::from(&x).plus_const(-1)))
            .unwrap();
        q.assert(
            "bool",
            &Formula::And(vec![
                Formula::implies(Formula::eq(&d, 0), Formula::ge(&p, Rational::ZERO)),
                Formula::cmp(&d, Cmp::Le, LinExpr::new().plus(2, &d).plus(-1, &d)),
            ]),
        )
        .unwrap();
        q.assert(
            "units",
            &Formula::Or(vec![Formula::And(vec![]), Formula::Or(vec![])]),
        )
        .unwrap();
        q.assert(
            "single",
            &Formula::And(vec![Formula::Or(vec![Formula::le(&d, 0)])]),
        )
        .unwrap();
        assert_eq!(
            q.to_smtlib(),
            "(set-option :produce-models true)\n\
             (set-option :produce-unsat-cores true)\n\
             (set-logic QF_LIRA)\n\
             (declare-const |d| Int)\n\
             (declare-const |p| Real)\n\
             (declare-const |x| Int)\n\
             (assert (! (< (+ (- 3) (* 2 |d|) (* 1 |x|)) 0) :named |@cmp|))\n\
             (assert (! (< (+ (/ 1.0 2.0) (* (- 1.0) |p|) (* (- (/ 1.0 2.0)) (to_real |x|))) 0.0) :named |@mix|))\n\
             (assert (! (not (= (+ 1) 0)) :named |@ne|))\n\
             (assert (! (and (or (not (= (+ 0 (* 1 |d|)) 0)) (<= (+ 0.0 (* (- 1.0) |p|)) 0.0)) (<= (+ 0) 0)) :named |@bool|))\n\
             (assert (! (or true false) :named |@units|))\n\
             (assert (! (<= (+ 0 (* 1 |d|)) 0) :named |@single|))\n\
             (check-sat)\n"
        );
    }
}
