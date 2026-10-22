//! `litigate explain-value CASE VALUE` (DESIGN §8): trace a deadline or
//! an edge of a matter back through the counting steps, the rule version
//! in force, any override and its delegation, to the cited L0 text.
use super::check::checked;
use super::where_::{cite_sections, overrides};
use crate::hirx::{self, local};
use crate::matter::Scheduled;
use crate::out::{Failure, Outcome, Report};
use litigate_base::Profile;
use litigate_sema::hir::{self, DeadlineWindow, Relation};
use serde_json::{json, Value};

fn step(rule: impl Into<String>, cite: Option<&String>, detail: impl Into<String>) -> Value {
    json!({"rule": rule.into(), "cite": cite, "detail": detail.into()})
}

fn name_of<T: serde::Serialize>(value: T) -> String {
    serde_json::to_value(value)
        .ok()
        .and_then(|v| v.as_str().map(str::to_owned))
        .unwrap_or_default()
}

/// The rule (`rule duty x { deadline d ... }`) declaring deadline `d`.
fn rule_of<'p>(pkg: &'p hir::Package, d: &str) -> Option<&'p hir::Rule> {
    hirx::declarations(pkg).find_map(|x| match x {
        hir::Declaration::Rule(r) if r.declarations.iter().any(|n| local(n) == d) => Some(r),
        _ => None,
    })
}

/// What one build of a trace needs.
struct Trace<'a> {
    all: &'a [&'a hir::Package],
    pkg: &'a hir::Package,
    schedule: &'a [Scheduled],
    as_of: &'a str,
}

impl Trace<'_> {
    /// The steps behind version `v` of deadline `d`.
    fn steps(&self, d: &str, v: &hir::Deadline, scheduled: Option<&Scheduled>) -> Vec<Value> {
        let rule = rule_of(self.pkg, d);
        let cite = v
            .meta
            .cite
            .as_ref()
            .or(rule.and_then(|r| r.meta.cite.as_ref()));
        let direction = match v.direction {
            hir::Direction::Forward => "after",
            hir::Direction::Backward => "before",
        };
        let mut steps = vec![step(
            format!("{d}@{}", v.meta.effective.from),
            cite,
            format!(
                "{} {} day(s) {direction} {}, counted by {}; in force [{}, {})",
                v.amount,
                name_of(v.unit),
                hirx::event_text(&v.trigger),
                v.counting,
                v.meta.effective.from,
                v.meta.effective.to.as_deref().unwrap_or("..")
            ),
        )];
        if let Some(r) = rule {
            steps.push(step(
                r.meta.id.clone(),
                r.meta.cite.as_ref(),
                format!(
                    "{} rule declaring `{d}` ({} authority)",
                    name_of(r.rule_kind),
                    r.meta.authority
                ),
            ));
        }
        steps.extend(
            v.meta
                .relations
                .iter()
                .filter_map(|relation| match relation {
                    Relation::DelegableTo(level) => Some(step(
                        d.to_owned(),
                        None,
                        format!("delegable to {level}: a {level} may override it"),
                    )),
                    _ => None,
                }),
        );
        let counting_cite =
            hirx::counting(self.pkg, &v.counting).and_then(|c| c.meta.cite.as_ref());
        match scheduled {
            Some(s) => {
                steps.push(step(
                    s.rule.clone(),
                    counting_cite,
                    format!("triggered by {} on {}", s.trigger, s.trigger_date),
                ));
                steps.extend(
                    s.steps
                        .iter()
                        .map(|t| step(s.rule.clone(), counting_cite, t.clone())),
                );
            }
            None => steps.push(step(
                v.counting.clone(),
                None,
                "no event of the matter triggers it yet, so it has no due date",
            )),
        }
        if let Some(cite) = cite {
            let sections =
                cite_sections(self.all, self.pkg, cite).unwrap_or(Value::Array(Vec::new()));
            steps.push(json!({"rule": "L0", "cite": cite, "detail": format!("primary-law text: {cite}"), "sections": sections}));
        }
        steps
    }

    /// `(due date, steps, versions)` of deadline `d` in the matter.
    fn deadline(&self, d: &str) -> (Value, Vec<Value>, Vec<Value>) {
        let versions: Vec<&hir::Deadline> = hirx::deadlines(self.pkg, d).collect();
        let qualified = format!("{}::{d}", self.pkg.manifest.pack_id);
        let scheduled = self.schedule.iter().find(|s| s.deadline == qualified);
        let date = scheduled.map_or(self.as_of, |s| s.trigger_date.as_str());
        let chosen = versions
            .iter()
            .find(|v| hirx::in_force(&v.meta, date, None))
            .copied();
        let steps = chosen
            .map(|v| self.steps(d, v, scheduled))
            .unwrap_or_default();
        let value = scheduled.map_or(Value::Null, |s| Value::String(s.due.clone()));
        let versions = versions
            .iter()
            .map(|v| {
                json!({
                    "in_force": {"from": v.meta.effective.from, "to": v.meta.effective.to},
                    "known_at": v.meta.effective.known_at,
                    "span": v.meta.span,
                    "selected": chosen.is_some_and(|c| std::ptr::eq(c, *v)),
                })
            })
            .collect();
        (value, steps, versions)
    }

    /// The result for edge `name` of `node`.
    fn edge(&self, node: &hir::Node, edge: &hir::Edge, name: &str) -> Value {
        let mut steps = vec![step(
            format!("{}@{}", node.meta.id, node.meta.effective.from),
            node.meta.cite.as_ref(),
            format!(
                "edge `{name}` of `{}` -> `{}`, taken by {}",
                node.meta.id, edge.to, edge.actor
            ),
        )];
        let mut due = Value::Null;
        if let Some(DeadlineWindow::Declared { deadline }) =
            edge.deadline.as_ref().map(|d| &d.window)
        {
            let (value, more, _) = self.deadline(local(deadline));
            steps.push(step(
                name.to_owned(),
                None,
                format!("must be taken within `{deadline}`"),
            ));
            steps.extend(more);
            due = value;
        }
        json!({
            "value": {
                "probability": edge.probability, "hours": edge.hours, "cost": edge.cost,
                "deadline": edge.deadline, "due": due, "authority": edge.authority,
            },
            "type": "Edge",
            "id": format!("{}::{name}", self.pkg.manifest.pack_id),
            "span": edge.span,
            "steps": steps,
            "versions": [{"in_force": {"from": node.meta.effective.from, "to": node.meta.effective.to}, "known_at": node.meta.effective.known_at, "span": node.meta.span}],
            "overrides": overrides(self.all, name),
        })
    }
}

/// `litigate explain-value`.
pub fn explain_value(case: Option<&str>, value: Option<&str>, lib: &[String]) -> Outcome {
    let (Some(case), Some(id)) = (case, value) else {
        return Err(Failure::new("give a .case file and a deadline or edge id"));
    };
    let c = checked(&[case.to_owned()], Profile::Dev, None, lib)?;
    let matter = c
        .cases
        .first()
        .ok_or_else(|| Failure::new(format!("{case}: not a .case file")))?;
    let mut report = Report {
        provenance: c.provenance(None, None),
        sources: c.session.sources(),
        diagnostics: c.diagnostics.clone(),
        ..Report::default()
    };
    let Some(hcase) = matter.case.as_ref().filter(|_| !report.has_errors()) else {
        return Ok(report);
    };
    let all = c.session.hir();
    let (pack, name) = id
        .split_once("::")
        .map_or((None, id), |(p, n)| (Some(p), n));
    let used: Vec<&hir::Package> = all
        .iter()
        .copied()
        .filter(|p| matter.packages.contains(&p.manifest.name))
        .filter(|p| pack.is_none_or(|k| p.manifest.pack_id == k || p.manifest.name == k))
        .collect();
    let trace = |pkg| Trace {
        all: &all,
        pkg,
        schedule: &matter.schedule,
        as_of: &hcase.as_of,
    };
    if let Some(pkg) = used
        .iter()
        .copied()
        .find(|p| hirx::deadlines(p, name).next().is_some())
    {
        let (value, steps, versions) = trace(pkg).deadline(name);
        report.result = json!({
            "value": value, "type": "Date", "id": format!("{}::{name}", pkg.manifest.pack_id),
            "steps": steps, "versions": versions, "overrides": overrides(&all, name),
        });
        return Ok(report);
    }
    let edge = used.iter().copied().find_map(|p| {
        hirx::nodes_at(p, Some(&hcase.as_of))
            .into_iter()
            .find_map(|n| {
                n.edges
                    .iter()
                    .find(|e| e.id.as_deref() == Some(name))
                    .map(|e| (p, n, e))
            })
    });
    let (pkg, node, edge) = edge.ok_or_else(|| {
        Failure::new(format!(
            "`{id}` is neither a deadline nor an edge of the packages `{case}` uses"
        ))
    })?;
    report.result = trace(pkg).edge(node, edge, name);
    Ok(report)
}
