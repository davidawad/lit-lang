//! `test` blocks (SPEC §4.7, §7 `tests-pass`): each `given` event happens
//! on its date, then every `expect` is evaluated. `deadline(d)` is the due
//! date of `d`'s version in force on its latest matching trigger, computed
//! by litgraph's clock interpreter over the lowered counting rule.
use crate::clock::{self, Clocks};
use crate::hirx;
use crate::span;
use litigate_base::{Diagnostic, DiagnosticCode, Span};
use litigate_sema::hir::{self, BinaryOp, Declaration, Expr, ExprKind, UnaryOp, Value};
use litigate_sema::types::rational;
use serde::Serialize;
use std::cmp::Ordering;

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
    clocks: &'a Clocks,
    pkg: &'a hir::Package,
    givens: &'a [(Expr, String)],
    steps: Vec<String>,
}

impl Cx<'_> {
    fn deadline(&mut self, name: &str) -> Result<V, String> {
        let any = hirx::deadlines(self.pkg, name)
            .next()
            .ok_or_else(|| format!("`{name}` is not a deadline of this package"))?;
        let (event, date) = self
            .givens
            .iter()
            .filter(|(e, _)| hirx::matches(&any.trigger, e))
            .max_by(|a, b| a.1.cmp(&b.1))
            .ok_or_else(|| {
                format!(
                    "no `given` event matches the trigger {} of `{name}`",
                    hirx::event_text(&any.trigger)
                )
            })?;
        let d = hirx::deadline_at(self.pkg, name, date)?;
        let due = clock::due(self.clocks, self.pkg, d, date, Some(event))?;
        self.steps.extend(due.computed.steps.iter().cloned());
        Ok(V::Date(due.computed.due_date))
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
                ("deadline", [arg]) => match &arg.kind {
                    ExprKind::Ref { target } => self.deadline(target),
                    _ => Err("`deadline` takes a deadline name".to_owned()),
                },
                ("has_event", [pattern]) => Ok(V::Bool(
                    self.givens.iter().any(|(g, _)| hirx::matches(pattern, g)),
                )),
                (other, _) => Err(format!("`{other}(..)` cannot be evaluated in a test")),
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

/// Runs every test of `pkg` whose name contains `filter`.
#[must_use]
pub fn run(
    clocks: &Clocks,
    pkg: &hir::Package,
    sources: &[(String, String)],
    filter: Option<&str>,
) -> (Vec<TestResult>, Vec<Diagnostic>) {
    let mut diagnostics = Vec::new();
    let results = hirx::declarations(pkg)
        .filter_map(|d| match d {
            Declaration::Test(t) if filter.is_none_or(|f| t.name.contains(f)) => Some(t),
            _ => None,
        })
        .map(|t| {
            let mut cx = Cx {
                clocks,
                pkg,
                givens: &t.givens,
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
