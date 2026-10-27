//! `litigate query KIND` (DESIGN §8): small structured answers over the
//! library, paged with `--limit` / `--cursor`.
use super::check::checked;
use crate::cli::QueryKind;
use crate::hirx::{self, local};
use crate::out::{Failure, Outcome, Report};
use litigate_base::{Profile, Span};
use litigate_lower::Options;
use litigate_sema::hir::{self, Declaration, Relation};
use litigate_sema::temporal::world::{meta_of, relation_target};
use serde::Serialize;
use serde_json::json;

/// `query` arguments.
pub struct Args {
    /// What to list.
    pub kind: QueryKind,
    /// Id/label substring.
    pub text: Option<String>,
    /// Cite substring.
    pub cite: Option<String>,
    /// In force on this date.
    pub effective_on: Option<String>,
    /// Related to this id.
    pub overrides: Option<String>,
    /// Package name or pack id.
    pub package: Option<String>,
    /// Page size.
    pub limit: usize,
    /// Page start.
    pub cursor: Option<String>,
    /// Search roots.
    pub paths: Vec<String>,
    /// Library directories.
    pub lib: Vec<String>,
}

/// One match.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Match {
    /// Qualified id.
    pub id: String,
    /// Declaration kind.
    pub kind: String,
    /// Label, when it has one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// Location.
    pub span: Option<Span>,
    /// Citation.
    pub cite: Option<String>,
    /// `{from, to}` in-force interval.
    pub in_force: Option<serde_json::Value>,
    /// The related target and relation (overrides), or the hole reason.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

fn of_meta(pkg: &hir::Package, kind: &str, m: &hir::Metadata) -> Match {
    Match {
        id: format!("{}::{}", pkg.manifest.pack_id, m.id),
        kind: kind.to_owned(),
        label: (!m.label.is_empty()).then(|| m.label.clone()),
        span: m.span.clone(),
        cite: m.cite.clone(),
        in_force: Some(json!({"from": m.effective.from, "to": m.effective.to})),
        detail: None,
    }
}

fn overriding(r: &Relation) -> bool {
    matches!(
        r,
        Relation::Override(_)
            | Relation::Notwithstanding(_)
            | Relation::Supersedes(_)
            | Relation::SubjectTo(_)
            | Relation::Revives(_)
            | Relation::Interprets(_)
    )
}

/// A candidate match and the version metadata filters apply to.
type Candidate<'p> = (Match, Option<&'p hir::Metadata>);

fn overrides(pkg: &hir::Package) -> Vec<Candidate<'_>> {
    hirx::declarations(pkg)
        .filter_map(|d| Some((d, meta_of(d)?)))
        .flat_map(|(d, m)| {
            m.relations
                .iter()
                .filter(|r| overriding(r))
                .map(move |r| (d, m, r))
        })
        .map(|(d, m, r)| {
            let relation = serde_json::to_value(r)
                .ok()
                .and_then(|v| {
                    v.get("relation")
                        .and_then(|x| x.as_str())
                        .map(str::to_owned)
                })
                .unwrap_or_default();
            let target = relation_target(r);
            (
                Match {
                    detail: Some(format!("{relation} {target}")),
                    ..of_meta(pkg, hirx::decl_kind(d), m)
                },
                Some(m),
            )
        })
        .collect()
}

/// Every edge, with litgraph's derived id (`from->to#n`) when anonymous.
fn edges(pkg: &hir::Package) -> Vec<Candidate<'_>> {
    hirx::nodes(pkg)
        .into_iter()
        .flat_map(|n| n.edges.iter().enumerate().map(move |(i, e)| (n, i, e)))
        .map(|(n, i, e)| {
            let nth = n.edges[..i]
                .iter()
                .filter(|x| x.to == e.to && x.id.is_none())
                .count();
            let id =
                e.id.clone()
                    .unwrap_or_else(|| format!("{}->{}#{nth}", n.meta.id, local(&e.to)));
            let m = Match {
                id: format!("{}::{id}", pkg.manifest.pack_id),
                kind: "edge".into(),
                label: Some(e.label.clone()),
                span: e.span.clone(),
                cite: e.authority.clone(),
                in_force: Some(json!({"from": n.meta.effective.from, "to": n.meta.effective.to})),
                detail: Some(format!("{} -> {}", n.meta.id, e.to)),
            };
            (m, Some(&n.meta))
        })
        .collect()
}

fn hole(h: &litigate_sema::Hole) -> Match {
    Match {
        id: format!("{}:{}:{}", h.span.file.0, h.span.line, h.span.col),
        kind: "hole".into(),
        label: None,
        span: Some(h.span.clone()),
        cite: None,
        in_force: None,
        detail: h.reason.clone(),
    }
}

/// Candidate matches of `kind` in `pkg`, before filters.
fn candidates<'p>(
    pkg: &'p hir::Package,
    check: &litigate_sema::PackageCheck,
    kind: QueryKind,
) -> Vec<Candidate<'p>> {
    let decls = || hirx::declarations(pkg).filter_map(|d| Some((d, meta_of(d)?)));
    match kind {
        QueryKind::Nodes => decls()
            .filter(|(d, _)| matches!(d, Declaration::Node(_)))
            .map(|(d, m)| (of_meta(pkg, hirx::decl_kind(d), m), Some(m)))
            .collect(),
        QueryKind::Deadlines => decls()
            .filter(|(d, _)| matches!(d, Declaration::Deadline(_)))
            .map(|(_, m)| (of_meta(pkg, "deadline", m), Some(m)))
            .collect(),
        QueryKind::Rules => decls()
            .filter_map(|(d, m)| match d {
                Declaration::Rule(r) => Some((
                    Match {
                        detail: Some(format!(
                            "{}: {}",
                            serde_json::to_value(r.rule_kind)
                                .ok()
                                .and_then(|k| k.as_str().map(str::to_owned))
                                .unwrap_or_default(),
                            r.declarations.join(", ")
                        )),
                        ..of_meta(pkg, "rule", m)
                    },
                    Some(m),
                )),
                _ => None,
            })
            .collect(),
        QueryKind::Cites => decls()
            .filter(|(_, m)| m.cite.is_some())
            .map(|(d, m)| (of_meta(pkg, hirx::decl_kind(d), m), Some(m)))
            .collect(),
        QueryKind::Versions => decls()
            .map(|(d, m)| {
                (
                    Match {
                        id: format!("{}::{}@{}", pkg.manifest.pack_id, m.id, m.effective.from),
                        ..of_meta(pkg, hirx::decl_kind(d), m)
                    },
                    Some(m),
                )
            })
            .collect(),
        QueryKind::Overrides => overrides(pkg),
        QueryKind::Edges => edges(pkg),
        QueryKind::Holes => check.holes.iter().map(|h| (hole(h), None)).collect(),
    }
}

/// `litigate query`.
pub fn query(args: &Args) -> Outcome {
    let roots = super::where_::roots(&args.paths, &args.lib)?;
    let c = checked(&roots, Profile::Dev, &Options::default(), &args.lib)?;
    let skip = args
        .cursor
        .as_deref()
        .map_or(Ok(0), str::parse::<usize>)
        .map_err(|_| Failure::new("--cursor must be a `next_cursor` value"))?;
    let lower = |s: &Option<String>| s.as_ref().map(|t| t.to_lowercase());
    let (text, cite) = (lower(&args.text), lower(&args.cite));
    let all: Vec<Match> = c
        .session
        .packages
        .iter()
        .filter(|p| {
            args.package.as_ref().is_none_or(|k| {
                p.name == *k
                    || p.check
                        .package
                        .as_ref()
                        .is_some_and(|h| h.manifest.pack_id == *k)
            })
        })
        .filter_map(|p| Some((p.check.package.as_ref()?, &p.check)))
        .flat_map(|(pkg, check)| candidates(pkg, check, args.kind))
        .filter(|(m, _)| {
            text.as_ref().is_none_or(|t| {
                m.id.to_lowercase().contains(t)
                    || m.label
                        .as_ref()
                        .is_some_and(|l| l.to_lowercase().contains(t))
            })
        })
        .filter(|(m, _)| {
            cite.as_ref().is_none_or(|t| {
                m.cite
                    .as_ref()
                    .is_some_and(|c| c.to_lowercase().contains(t))
            })
        })
        .filter(|(_, meta)| {
            args.effective_on
                .as_ref()
                .is_none_or(|d| meta.is_some_and(|m| hirx::in_force(m, d)))
        })
        .filter(|(_, meta)| {
            args.overrides.as_ref().is_none_or(|target| {
                meta.is_some_and(|m| {
                    m.relations
                        .iter()
                        .any(|r| local(relation_target(r)) == local(target))
                })
            })
        })
        .map(|(m, _)| m)
        .collect();
    let total = all.len();
    let page: Vec<Match> = all.into_iter().skip(skip).take(args.limit).collect();
    let next = (skip + page.len() < total).then(|| (skip + page.len()).to_string());
    Ok(Report {
        result: json!({"matches": page, "total": total, "next_cursor": next}),
        provenance: c.provenance(None, None),
        ..Report::default()
    })
}
