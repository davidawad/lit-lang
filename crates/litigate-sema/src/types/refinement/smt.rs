//! Tier 3: refinements over open values, decided by `litigate-smt`.
//!
//! Each variable becomes an SMT constant: `Int` for dates (day numbers),
//! durations, money (minor units), `Bool`s (0 or 1) and enums (a variant's
//! index), `Real` for numbers, probabilities and hours. Their domains are
//! asserted beside the predicates (a date in `0001-01-01..=9999-12-31`, a
//! probability in `[0, 1]`, an enum one of its variants), so a model is
//! always a value a program could write. The query is quantifier-free
//! linear arithmetic, which Z3 decides under a deterministic work budget;
//! running out of it is `unknown`, a hole (L2015), never a pass. A wasm
//! build links no solver: there every question is `unknown`.
use crate::hir::{self, Type};
#[cfg(not(target_arch = "wasm32"))]
use {
    super::eval::{show, Val},
    crate::hir::{BinaryOp, ExprKind, UnaryOp, Value},
    crate::types::date,
    litigate_smt::{Cmp, Formula, LinExpr, Query, Rational as R},
    std::collections::BTreeMap,
};

/// A variable of a question: its name, type, and (for an enum) its
/// variants, which bound its domain and index its constants (a question
/// comparing an enum whose variants are not given is `unknown`).
#[derive(Debug, Clone, Copy)]
pub struct Var<'a> {
    /// The name predicates reference it by.
    pub name: &'a str,
    /// Its type.
    pub ty: &'a Type,
    /// An enum's variants (by name), when known.
    pub variants: Option<&'a [String]>,
}

/// The solver tier's answer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Decision {
    /// Every predicate holds together: one witness, each variable's value
    /// in source spelling, in the order given.
    Sat(Vec<(String, String)>),
    /// They cannot all hold: the indices of a subset that already cannot
    /// (an unsat core), ascending.
    Unsat(Vec<usize>),
    /// Not decided (the solver's reason): a hole, never a pass.
    Unknown(String),
}

/// The solver's work budget: `litigate-smt`'s default, a deterministic
/// resource limit rather than a timeout.
pub const RLIMIT: u32 = 10_000_000;

/// Whether `preds` (typed `Bool`, over `vars`) can all hold at once.
#[must_use]
pub fn decide(vars: &[Var<'_>], preds: &[hir::Expr]) -> Decision {
    decide_within(RLIMIT, vars, preds)
}

/// [`decide`] under work budget `rlimit`.
#[cfg(target_arch = "wasm32")]
#[must_use]
pub fn decide_within(_rlimit: u32, _vars: &[Var<'_>], _preds: &[hir::Expr]) -> Decision {
    Decision::Unknown("this build links no SMT solver (wasm)".into())
}

/// [`decide`] under work budget `rlimit` (`litigate_smt::Z3::with_rlimit`).
#[cfg(not(target_arch = "wasm32"))]
#[must_use]
pub fn decide_within(rlimit: u32, vars: &[Var<'_>], preds: &[hir::Expr]) -> Decision {
    use litigate_smt::{Backend, Outcome};
    let outcome = encode(vars, preds).and_then(|query| {
        litigate_smt::Z3::with_rlimit(rlimit).and_then(|mut z3| z3.check(&query))
    });
    match outcome {
        Ok(Outcome::Sat(model)) => Decision::Sat(witness(vars, &model)),
        Ok(Outcome::Unsat(core)) => Decision::Unsat(
            core.iter()
                .filter_map(|l| l.strip_prefix('p')?.parse().ok())
                .collect(),
        ),
        Ok(Outcome::Unknown(why)) => Decision::Unknown(why),
        Err(e) => Decision::Unknown(e.to_string()),
    }
}

/// Each variable's value in `model`, in source spelling.
#[cfg(not(target_arch = "wasm32"))]
fn witness(vars: &[Var<'_>], model: &litigate_smt::Model) -> Vec<(String, String)> {
    vars.iter()
        .map(|v| {
            // A model values every declared variable.
            let r = model.get(v.name).copied().unwrap_or(R::ZERO);
            let n = crate::hir::Rational {
                numerator: r.numer(),
                denominator: r.denom().unsigned_abs(),
            };
            let value = match v.ty {
                Type::Boolean => Val::Bool(n.numerator != 0),
                // Its domain holds an enum's index among its variants.
                Type::Named(_) => Val::Variant(
                    v.variants
                        .and_then(|names| {
                            names.get(usize::try_from(n.numerator).unwrap_or(usize::MAX))
                        })
                        .cloned()
                        .unwrap_or_default(),
                ),
                _ => Val::Num(n),
            };
            (v.name.to_owned(), show(v.ty, &value))
        })
        .collect()
}

/// A linear form `Σ cᵢ·xᵢ + c` over the question's variables.
#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Clone)]
struct Lin {
    terms: BTreeMap<String, R>,
    constant: R,
}

#[cfg(not(target_arch = "wasm32"))]
impl Lin {
    fn constant(c: R) -> Self {
        Self {
            terms: BTreeMap::new(),
            constant: c,
        }
    }

    fn scale(&self, k: R) -> Result<Self, litigate_smt::Error> {
        let mut out = Self::constant(self.constant.checked_mul(k)?);
        for (n, c) in &self.terms {
            out.terms.insert(n.clone(), c.checked_mul(k)?);
        }
        Ok(out)
    }

    fn plus(mut self, other: &Self) -> Result<Self, litigate_smt::Error> {
        self.constant = self.constant.checked_add(other.constant)?;
        for (n, c) in &other.terms {
            let slot = self.terms.entry(n.clone()).or_insert(R::ZERO);
            *slot = slot.checked_add(*c)?;
        }
        Ok(self)
    }

    fn as_constant(&self) -> Option<R> {
        self.terms
            .values()
            .all(|c| *c == R::ZERO)
            .then_some(self.constant)
    }
}

/// The question: domains, then each predicate labelled `p<i>`.
#[cfg(not(target_arch = "wasm32"))]
fn encode(vars: &[Var<'_>], preds: &[hir::Expr]) -> Result<Query, litigate_smt::Error> {
    let mut e = Encoder {
        query: Query::new(),
        vars: BTreeMap::new(),
    };
    for v in vars {
        e.declare(v)?;
    }
    for (i, p) in preds.iter().enumerate() {
        let f = e.formula(p)?;
        e.query.assert(&format!("p{i}"), &f)?;
    }
    Ok(e.query)
}

#[cfg(not(target_arch = "wasm32"))]
struct Encoder<'v> {
    query: Query,
    /// Declared variables with their enum variants.
    vars: BTreeMap<String, (litigate_smt::Var, Option<&'v [String]>)>,
}

#[cfg(not(target_arch = "wasm32"))]
impl<'v> Encoder<'v> {
    fn declare(&mut self, v: &Var<'v>) -> Result<(), litigate_smt::Error> {
        let real = matches!(v.ty, Type::Number | Type::Probability | Type::Hours);
        let x = if real {
            self.query.real(v.name)?
        } else {
            self.query.int(v.name)?
        };
        let int = |n: i64| LinExpr::from(n);
        let range =
            |lo: LinExpr, hi: LinExpr| Formula::And(vec![Formula::le(lo, &x), Formula::le(&x, hi)]);
        let domain = match (v.ty, v.variants) {
            (Type::Date, _) => {
                let first = date::day_number("0001-01-01").unwrap_or_default();
                let last = date::day_number("9999-12-31").unwrap_or_default();
                Some(range(int(first), int(last)))
            }
            (Type::Probability | Type::Boolean, _) => Some(range(int(0), int(1))),
            (Type::Named(_), Some(names)) => {
                let n = i64::try_from(names.len()).unwrap_or(i64::MAX);
                Some(range(int(0), int(n - 1)))
            }
            _ => None,
        };
        if let Some(domain) = domain {
            self.query.assert(&format!("domain {}", v.name), &domain)?;
        }
        self.vars.insert(v.name.to_owned(), (x, v.variants));
        Ok(())
    }

    fn formula(&self, e: &hir::Expr) -> Result<Formula, litigate_smt::Error> {
        Ok(match &e.kind {
            ExprKind::Literal {
                value: Value::Boolean(b),
            } if *b => Formula::And(vec![]),
            ExprKind::Literal { .. } => Formula::Or(vec![]),
            ExprKind::Ref { target } => Formula::eq(self.var(target)?, 1),
            ExprKind::Unary { operand, .. } => !self.formula(operand)?,
            ExprKind::Binary {
                op: op @ (BinaryOp::And | BinaryOp::Or),
                lhs,
                rhs,
            } => {
                let parts = vec![self.formula(lhs)?, self.formula(rhs)?];
                if *op == BinaryOp::And {
                    Formula::And(parts)
                } else {
                    Formula::Or(parts)
                }
            }
            ExprKind::Binary { op, lhs, rhs } => {
                let cmp = match op {
                    BinaryOp::Eq => Cmp::Eq,
                    BinaryOp::Ne => Cmp::Ne,
                    BinaryOp::Lt => Cmp::Lt,
                    BinaryOp::Le => Cmp::Le,
                    BinaryOp::Gt => Cmp::Gt,
                    _ => Cmp::Ge,
                };
                let variants = self.variants_of(lhs).or_else(|| self.variants_of(rhs));
                let (l, r) = (self.term(lhs, variants)?, self.term(rhs, variants)?);
                Formula::cmp(self.lin(&l), cmp, self.lin(&r))
            }
            _ => return Err(litigate_smt::Error::Solver("not a predicate".into())),
        })
    }

    fn var(&self, name: &str) -> Result<&litigate_smt::Var, litigate_smt::Error> {
        self.vars
            .get(name)
            .map(|(v, _)| v)
            .ok_or_else(|| litigate_smt::Error::UndeclaredVar(name.to_owned()))
    }

    fn variants_of(&self, e: &hir::Expr) -> Option<&'v [String]> {
        match &e.kind {
            ExprKind::Ref { target } => self.vars.get(target).and_then(|(_, v)| *v),
            _ => None,
        }
    }

    fn lin(&self, l: &Lin) -> LinExpr {
        // A term's variables are declared: `term` checked each.
        l.terms
            .iter()
            .fold(LinExpr::from(l.constant), |acc, (n, c)| {
                acc.plus(*c, &self.vars[n].0)
            })
    }

    fn term(&self, e: &hir::Expr, variants: Option<&[String]>) -> Result<Lin, litigate_smt::Error> {
        Ok(match &e.kind {
            ExprKind::Literal { value } => Lin::constant(match super::eval::of(value) {
                Some(Val::Num(n)) => rational(n)?,
                Some(Val::Bool(b)) => R::int(i64::from(b)),
                // An enum constant is its index among the other side's
                // variants.
                Some(Val::Variant(name)) => {
                    let index = variants
                        .and_then(|vs| vs.iter().position(|x| *x == name))
                        .ok_or_else(|| litigate_smt::Error::UndeclaredVar(name.clone()))?;
                    R::int(i64::try_from(index).unwrap_or(i64::MAX))
                }
                None => return Err(litigate_smt::Error::Solver("not a term".into())),
            }),
            ExprKind::Ref { target } => {
                self.var(target)?;
                Lin {
                    terms: BTreeMap::from([(target.clone(), R::ONE)]),
                    constant: R::ZERO,
                }
            }
            ExprKind::Unary {
                op: UnaryOp::Neg,
                operand,
            } => self.term(operand, variants)?.scale(R::MINUS_ONE)?,
            ExprKind::Binary { op, lhs, rhs } => {
                let (l, r) = (self.term(lhs, variants)?, self.term(rhs, variants)?);
                match op {
                    BinaryOp::Add => l.plus(&r)?,
                    BinaryOp::Sub => l.plus(&r.scale(R::MINUS_ONE)?)?,
                    BinaryOp::Mul => match (l.as_constant(), r.as_constant()) {
                        (Some(k), _) => r.scale(k)?,
                        (_, Some(k)) => l.scale(k)?,
                        _ => return Err(litigate_smt::Error::Solver("nonlinear".into())),
                    },
                    _ => {
                        let k = r
                            .as_constant()
                            .ok_or_else(|| litigate_smt::Error::Solver("nonlinear".into()))?;
                        l.scale(R::new(k.denom(), k.numer())?)?
                    }
                }
            }
            _ => return Err(litigate_smt::Error::Solver("not a term".into())),
        })
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn rational(n: crate::hir::Rational) -> Result<R, litigate_smt::Error> {
    R::new(
        n.numerator,
        i64::try_from(n.denominator).map_err(|_| litigate_smt::Error::Overflow)?,
    )
}
