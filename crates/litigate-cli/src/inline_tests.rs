//! `test` blocks (SPEC §4.7, §7 `tests-pass`): each `given` event happens
//! on its date, then every `expect` is evaluated. `deadline(d)` is the due
//! date of the declaration selected for `d` (its version in force, or a
//! claimant displacing it, SPEC §5.5) on the matching trigger its trigger
//! policy picks (the first under `first`, else the latest), computed by
//! litgraph's clock interpreter over the lowered counting rule:
//! `deadline(d).adjusted` (or bare `deadline(d)`) after every roll and
//! extension, `deadline(d).nominal` the last day as counted. A later
//! `given` whose type `extends d` extends it as replay would: licensed by
//! one of `d`'s grants held by the actor it names (`given e by court on
//! date`), within the grant's window, motion and bound.
use crate::clock::{self, Clocks};
use crate::hirx;
use crate::span;
use litigate_base::{Diagnostic, DiagnosticCode, Span};
use litigate_sema::case::grant::{self, Given};
use litigate_sema::hir::{self, BinaryOp, Declaration, Expr, ExprKind, UnaryOp, Value};
use litigate_sema::temporal::{Key, MatterEvent, World};
use litigate_sema::types::rational;
use serde::Serialize;
use std::cmp::Ordering;
use std::collections::BTreeMap;

/// One evaluated `expect`.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Expectation {
    /// Source text of the assertion.
    pub text: String,
    /// Whether it holds.
    pub passed: bool,
    /// The left-hand value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actual: Option<serde_json::Value>,
    /// The right-hand value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected: Option<serde_json::Value>,
    /// Counting steps behind any `deadline(..)` in it.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub steps: Vec<String>,
    /// Why it could not be evaluated, if it could not.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// One test block's outcome.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TestResult {
    /// Package name.
    pub package: String,
    /// Test name.
    pub name: String,
    /// Its citation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cite: Option<String>,
    /// Location.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub span: Option<Span>,
    /// Every expectation held.
    pub passed: bool,
    /// Per-expectation results.
    pub expectations: Vec<Expectation>,
}

/// An evaluated value.
#[derive(Debug, Clone, PartialEq)]
enum V {
    Date(String),
    Bool(bool),
    Number(hir::Rational),
    Other(serde_json::Value),
}

impl V {
    fn json(&self) -> serde_json::Value {
        match self {
            Self::Date(d) => serde_json::Value::String(d.clone()),
            Self::Bool(b) => serde_json::Value::Bool(*b),
            Self::Number(r) => serde_json::Value::String(rational::display(*r)),
            Self::Other(v) => v.clone(),
        }
    }
}

struct Cx<'a> {
    clocks: &'a Clocks<'a>,
    /// The law the package sees ([`Clocks::scope`]).
    world: &'a World<'a>,
    pkg: &'a hir::Package,
    /// The corpus date versions are selected at.
    known_at: &'a str,
    givens: &'a [(Expr, String)],
    /// The actor of each `given` naming one, by index.
    actors: &'a BTreeMap<usize, String>,
    steps: Vec<String>,
}

impl Cx<'_> {
    fn deadline(&mut self, name: &str, nominal: bool) -> Result<V, String> {
        // A deadline the package imports (`pack::id`) is its declarer's.
        let owner = name
            .split_once("::")
            .and_then(|(pack, _)| self.clocks.package(pack))
            .unwrap_or(self.pkg);
        let any = hirx::deadlines(owner, name)
            .next()
            .ok_or_else(|| format!("`{name}` is not a deadline of this package"))?;
        // A chained period starts with the event starting the one it follows.
        let trigger =
            clock::root_trigger(self.world, &owner.manifest.pack_id, any).unwrap_or(&any.trigger);
        let matching = self
            .givens
            .iter()
            .enumerate()
            .filter(|(_, (e, _))| hirx::matches(trigger, e));
        // The trigger policy picks the event that runs the period: the
        // first under `first` (the default), else the latest (SPEC §5.3).
        let picked = if any.policy.is_first() {
            matching.min_by(|a, b| a.1 .1.cmp(&b.1 .1))
        } else {
            matching.max_by(|a, b| a.1 .1.cmp(&b.1 .1))
        };
        let (at, (event, date)) = picked.ok_or_else(|| {
            format!(
                "no `given` event matches the trigger {} of `{name}`",
                hirx::event_text(trigger)
            )
        })?;
        let events = [MatterEvent::on(date, event)];
        let known_at = Some(self.known_at);
        let governing = clock::deadline(self.world, owner, name, &events, 0, known_at)?;
        let due = clock::due_of(self.clocks, self.world, governing, &events, 0, known_at)?;
        self.steps.extend(due.computed.steps.iter().cloned());
        if nominal {
            return Ok(V::Date(due.computed.nominal_date));
        }
        let key = Key {
            pack: owner.manifest.pack_id.clone(),
            name: hirx::local(name).to_owned(),
        };
        self.extended(governing.1, (&key, name), at, due.computed.due_date)
    }

    /// The due date of `d` (`name`, keyed `key`) triggered by `given` `at`, counted
    /// `counted`, after the later `given`s extending it ([`grant::extensions`]).
    fn extended(
        &mut self,
        d: &hir::Deadline,
        (key, name): (&Key, &str),
        at: usize,
        counted: String,
    ) -> Result<V, String> {
        let mut order: Vec<usize> = (0..self.givens.len()).collect();
        order.sort_by(|&a, &b| self.givens[a].1.cmp(&self.givens[b].1).then(a.cmp(&b)));
        let history: Vec<Given<'_>> = order
            .iter()
            .map(|&i| Given {
                date: &self.givens[i].1,
                action: &self.givens[i].0,
                actor: self.actors.get(&i).map(String::as_str),
            })
            .collect();
        let trigger = order.iter().position(|&i| i == at).unwrap_or_default();
        let world = self.world;
        let pkg = self.pkg;
        let applied = grant::extensions(d, name, &history, trigger, &counted, |action| {
            grant::extends(world, pkg, key, action)
        })?;
        let Some((_, last)) = applied.last() else {
            return Ok(V::Date(counted));
        };
        for (k, new) in &applied {
            let g = history[*k];
            self.steps.push(format!(
                "extended to {new} by `{}` on {}",
                hirx::event_text(g.action),
                g.date
            ));
        }
        Ok(V::Date(last.clone()))
    }

    /// `deadline(d)` (adjusted) or its `field`.
    fn due(&mut self, arg: &Expr, field: &str) -> Result<V, String> {
        match &arg.kind {
            ExprKind::Ref { target } => self.deadline(target, field == "nominal"),
            _ => Err("`deadline` takes a deadline name".to_owned()),
        }
    }

    fn eval(&mut self, e: &Expr) -> Result<V, String> {
        match &e.kind {
            ExprKind::Literal { value } => Ok(match value {
                Value::Date(d) => V::Date(d.clone()),
                Value::Boolean(b) => V::Bool(*b),
                Value::Number(r) => V::Number(*r),
                other => V::Other(serde_json::to_value(other).unwrap_or_default()),
            }),
            ExprKind::Call { callee, args } => match (hirx::local(callee), args.as_slice()) {
                ("deadline", [arg]) => self.due(arg, "adjusted"),
                ("has_event", [pattern]) => Ok(V::Bool(
                    self.givens.iter().any(|(g, _)| hirx::matches(pattern, g)),
                )),
                (other, _) => Err(format!("`{other}(..)` cannot be evaluated in a test")),
            },
            ExprKind::Field { base, field } => match &base.kind {
                ExprKind::Call { callee, args } if hirx::local(callee) == "deadline" => {
                    match args.as_slice() {
                        [arg] => self.due(arg, field),
                        _ => Err("`deadline` takes a deadline name".to_owned()),
                    }
                }
                _ => Err("only a due date's fields can be evaluated in a test".to_owned()),
            },
            ExprKind::Unary {
                op: UnaryOp::Not,
                operand,
            } => match self.eval(operand)? {
                V::Bool(b) => Ok(V::Bool(!b)),
                _ => Err("`!` needs a Bool".to_owned()),
            },
            ExprKind::Binary { op, lhs, rhs } => {
                let (a, b) = (self.eval(lhs)?, self.eval(rhs)?);
                binary(*op, &a, &b).map(V::Bool)
            }
            _ => Err("this expression form cannot be evaluated in a test".to_owned()),
        }
    }
}

fn binary(op: BinaryOp, a: &V, b: &V) -> Result<bool, String> {
    let order = match (a, b) {
        (V::Date(x), V::Date(y)) => Some(x.cmp(y)),
        (V::Number(x), V::Number(y)) => Some(rational::cmp(*x, *y)),
        _ => None,
    };
    match (op, a, b) {
        (BinaryOp::And, V::Bool(x), V::Bool(y)) => Ok(*x && *y),
        (BinaryOp::Or, V::Bool(x), V::Bool(y)) => Ok(*x || *y),
        (BinaryOp::Eq, ..) => Ok(a == b),
        (BinaryOp::Ne, ..) => Ok(a != b),
        (BinaryOp::Lt | BinaryOp::Le | BinaryOp::Gt | BinaryOp::Ge, ..) => {
            let order = order.ok_or("ordering needs two dates or two numbers")?;
            Ok(match op {
                BinaryOp::Lt => order == Ordering::Less,
                BinaryOp::Le => order != Ordering::Greater,
                BinaryOp::Gt => order == Ordering::Greater,
                _ => order != Ordering::Less,
            })
        }
        _ => Err(format!("`{op:?}` is not a test assertion operator")),
    }
}

/// The source span from `a`'s start to `b`'s end.
fn joined(a: Option<&Span>, b: Option<&Span>) -> Option<Span> {
    let (a, b) = (a?, b?);
    Some(Span {
        end_line: b.end_line,
        end_col: b.end_col,
        ..a.clone()
    })
}

fn expectation(
    cx: &mut Cx<'_>,
    e: &hir::Expectation,
    sources: &[(String, String)],
) -> (Expectation, Option<Span>) {
    let at = joined(e.lhs.span.as_ref(), e.rhs.span.as_ref());
    let text = at
        .as_ref()
        .and_then(|s| {
            let (_, src) = sources.iter().find(|(p, _)| *p == s.file.0)?;
            Some(span::slice(src, s).to_owned())
        })
        .unwrap_or_default();
    cx.steps.clear();
    let evaluated = cx.eval(&e.lhs).and_then(|a| {
        let b = cx.eval(&e.rhs)?;
        Ok((binary(e.op, &a, &b)?, a, b))
    });
    let steps = std::mem::take(&mut cx.steps);
    let result = match evaluated {
        Ok((passed, a, b)) => Expectation {
            text,
            passed,
            actual: Some(a.json()),
            expected: Some(b.json()),
            steps,
            error: None,
        },
        Err(error) => Expectation {
            text,
            passed: false,
            actual: None,
            expected: None,
            steps,
            error: Some(error),
        },
    };
    (result, at)
}

/// Runs every test of `pkg` whose name contains `filter`, selecting
/// versions among those recorded by `known_at` (default: the package's
/// `known_at`, as its build does).
#[must_use]
pub fn run(
    clocks: &Clocks,
    pkg: &hir::Package,
    sources: &[(String, String)],
    filter: Option<&str>,
    known_at: Option<&str>,
) -> (Vec<TestResult>, Vec<Diagnostic>) {
    let known_at = known_at.unwrap_or(&pkg.manifest.in_force.known_at);
    let world = clocks.scope(&[&pkg.manifest.name]);
    let mut diagnostics = Vec::new();
    let results = hirx::declarations(pkg)
        .filter_map(|d| match d {
            Declaration::Test(t) if filter.is_none_or(|f| t.name.contains(f)) => Some(t),
            _ => None,
        })
        .map(|t| {
            let mut cx = Cx {
                clocks,
                world: &world,
                pkg,
                known_at,
                givens: &t.givens,
                actors: &t.actors,
                steps: Vec::new(),
            };
            let expectations: Vec<Expectation> = t
                .expects
                .iter()
                .map(|e| {
                    let (x, at) = expectation(&mut cx, e, sources);
                    if !x.passed {
                        let at = at
                            .or_else(|| t.span.clone())
                            .unwrap_or_else(|| nowhere(pkg));
                        let why = x.error.clone().unwrap_or_else(|| {
                            format!(
                                "found {}",
                                x.actual
                                    .as_ref()
                                    .map(ToString::to_string)
                                    .unwrap_or_default()
                            )
                        });
                        diagnostics.push(Diagnostic::error(
                            DiagnosticCode::TestFailed,
                            at,
                            format!("test \"{}\": `{}` does not hold: {why}", t.name, x.text),
                        ));
                    }
                    x
                })
                .collect();
            TestResult {
                package: pkg.manifest.name.clone(),
                name: t.name.clone(),
                cite: t.cite.clone(),
                span: t.span.clone(),
                passed: expectations.iter().all(|x| x.passed),
                expectations,
            }
        })
        .collect();
    (results, diagnostics)
}

/// A span at the start of the package manifest, for span-less findings.
#[must_use]
pub fn nowhere(pkg: &hir::Package) -> Span {
    let file = hirx::declarations(pkg)
        .find_map(|d| litigate_sema::temporal::world::meta_of(d)?.span.clone())
        .map_or_else(|| "Lit.toml".to_owned(), |s| s.file.0);
    let manifest = std::path::Path::new(&file).parent().map_or_else(
        || "Lit.toml".to_owned(),
        |p| p.join("Lit.toml").to_string_lossy().replace('\\', "/"),
    );
    Span {
        file: manifest.as_str().into(),
        line: 1,
        col: 1,
        end_line: 1,
        end_col: 1,
    }
}

#[cfg(test)]
mod tests;
