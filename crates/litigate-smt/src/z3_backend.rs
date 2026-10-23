//! [`Backend`] on the pinned, statically linked Z3.
use std::collections::BTreeMap;

use z3::ast::{Bool, Int, Real};
use z3::{Config, Params, SatResult, Solver};

use crate::query::{Atom, Lowered, Rel};
use crate::{Backend, Error, Model, Outcome, Query, Rational, Sort};

/// The Z3 release `build.rs` links (`major.minor.build`).
pub const PINNED_Z3_VERSION: &str = "5.1.0";

/// Default resource limit (Z3 `rlimit`), a deterministic work budget
/// unlike a wall-clock timeout. Linear queries from the type checker
/// finish far below it.
pub const DEFAULT_RLIMIT: u32 = 10_000_000;

/// Z3, configured for reproducible answers.
///
/// Each check runs in a fresh Z3 context on the calling thread with a
/// fixed random seed, one thread, a deterministic resource limit instead
/// of a timeout, and minimised unsat cores; see the crate README.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Z3 {
    rlimit: u32,
}

impl Z3 {
    /// The backend with [`DEFAULT_RLIMIT`].
    ///
    /// # Errors
    ///
    /// [`Error::VersionMismatch`] if the linked Z3 is not
    /// [`PINNED_Z3_VERSION`] (for example a system `libz3` won the link).
    pub fn new() -> Result<Self, Error> {
        Self::with_rlimit(DEFAULT_RLIMIT)
    }

    /// The backend with resource limit `rlimit` (`0` means unlimited).
    ///
    /// # Errors
    ///
    /// As [`Z3::new`].
    pub fn with_rlimit(rlimit: u32) -> Result<Self, Error> {
        check_version(&linked_version()).map(|()| Self { rlimit })
    }

    fn solve(&self, query: &Query) -> Result<Outcome, Error> {
        let solver = Solver::new();
        let mut params = Params::new();
        params.set_u32("random_seed", 0);
        params.set_u32("rlimit", self.rlimit);
        params.set_bool("core.minimize", true);
        solver.set_params(&params);

        let mut ints = BTreeMap::new();
        let mut reals = BTreeMap::new();
        for (name, sort) in query.vars() {
            let real = match sort {
                Sort::Int => {
                    let x = Int::new_const(name);
                    let real = x.to_real();
                    ints.insert(name, x);
                    real
                }
                Sort::Real => Real::new_const(name),
            };
            reals.insert(name, real);
        }
        let consts = Consts { ints, reals };
        for (i, (_, f)) in query.assertions().iter().enumerate() {
            solver.assert_and_track(consts.formula(f), &Bool::new_const(format!("a{i}")));
        }

        match solver.check() {
            SatResult::Sat => solver
                .get_model()
                .ok_or(Error::Solver("sat without a model".into()))
                .and_then(|m| consts.model(&m)),
            SatResult::Unsat => {
                let mut core: Vec<usize> = solver
                    .get_unsat_core()
                    .iter()
                    .filter_map(|b| b.to_string().trim_start_matches('a').parse().ok())
                    .collect();
                core.sort_unstable();
                let labels = query.assertions();
                Ok(Outcome::Unsat(
                    core.into_iter().map(|i| labels[i].0.clone()).collect(),
                ))
            }
            SatResult::Unknown => Ok(Outcome::Unknown(
                solver.get_reason_unknown().unwrap_or_default(),
            )),
        }
    }
}

impl Backend for Z3 {
    fn fingerprint(&self) -> String {
        format!(
            "z3 {} random_seed=0 threads=1 core.minimize=true rlimit={}",
            linked_version(),
            self.rlimit
        )
    }

    fn check(&mut self, query: &Query) -> Result<Outcome, Error> {
        let mut cfg = Config::new();
        cfg.set_model_generation(true);
        cfg.set_param_value("unsat_core", "true");
        // A fresh context per check: no state carries between queries.
        z3::with_z3_config(&cfg, || self.solve(query))
    }
}

/// The linked Z3's `major.minor.build` (its full version reads
/// `Z3 major.minor.build.revision`).
fn linked_version() -> String {
    version_of(z3::full_version())
}

fn version_of(full: &str) -> String {
    let numbers = full.rsplit(' ').next().unwrap_or_default();
    numbers.split('.').take(3).collect::<Vec<_>>().join(".")
}

fn check_version(found: &str) -> Result<(), Error> {
    if found == PINNED_Z3_VERSION {
        Ok(())
    } else {
        Err(Error::VersionMismatch {
            expected: PINNED_Z3_VERSION.to_owned(),
            found: found.to_owned(),
        })
    }
}

/// The query's variables as Z3 constants: `Int`s by name, and every
/// variable as a `Real` (an `Int` cast) for rational atoms.
struct Consts<'q> {
    ints: BTreeMap<&'q str, Int>,
    reals: BTreeMap<&'q str, Real>,
}

impl Consts<'_> {
    fn formula(&self, f: &Lowered) -> Bool {
        match f {
            Lowered::Atom(a) if a.int => self.int_atom(a),
            Lowered::Atom(a) => self.real_atom(a),
            Lowered::Not(f) => self.formula(f).not(),
            Lowered::And(fs) => Bool::and(&self.formulas(fs)),
            Lowered::Or(fs) => Bool::or(&self.formulas(fs)),
        }
    }

    fn formulas(&self, fs: &[Lowered]) -> Vec<Bool> {
        fs.iter().map(|f| self.formula(f)).collect()
    }

    fn int_atom(&self, a: &Atom) -> Bool {
        let mut sum = vec![Int::from_i64(a.constant.numer())];
        for (name, c) in &a.terms {
            sum.push(Int::from_i64(c.numer()) * &self.ints[name.as_str()]);
        }
        let (sum, zero) = (Int::add(&sum), Int::from_i64(0));
        match a.rel {
            Rel::Le => sum.le(&zero),
            Rel::Lt => sum.lt(&zero),
            Rel::Eq => sum.eq(&zero),
        }
    }

    fn real_atom(&self, a: &Atom) -> Bool {
        let lit = |r: Rational| Real::from_rational(r.numer(), r.denom());
        let mut sum = vec![lit(a.constant)];
        for (name, c) in &a.terms {
            sum.push(lit(*c) * &self.reals[name.as_str()]);
        }
        let (sum, zero) = (Real::add(&sum), lit(Rational::ZERO));
        match a.rel {
            Rel::Le => sum.le(&zero),
            Rel::Lt => sum.lt(&zero),
            Rel::Eq => sum.eq(&zero),
        }
    }

    fn model(&self, m: &z3::Model) -> Result<Outcome, Error> {
        self.reals
            .iter()
            .map(|(&name, x)| {
                let value = match self.ints.get(name) {
                    Some(x) => m.eval(x, true).and_then(|v| v.as_i64()).map(Rational::int),
                    None => m
                        .eval(x, true)
                        .and_then(|v| v.as_rational())
                        .and_then(|(n, d)| Rational::new(n, d).ok()),
                };
                value
                    .map(|v| (name.to_owned(), v))
                    .ok_or_else(|| Error::ValueOutOfRange(name.to_owned()))
            })
            .collect::<Result<Model, Error>>()
            .map(Outcome::Sat)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn linked_version_is_pinned() {
        assert_eq!(z3::full_version(), "Z3 5.1.0.0");
        assert_eq!(linked_version(), PINNED_Z3_VERSION);
        assert_eq!(version_of("4.8.12"), "4.8.12");
    }

    #[test]
    fn version_check() {
        assert_eq!(check_version(PINNED_Z3_VERSION), Ok(()));
        assert_eq!(
            check_version("4.8.12"),
            Err(Error::VersionMismatch {
                expected: "5.1.0".into(),
                found: "4.8.12".into()
            })
        );
    }
}
