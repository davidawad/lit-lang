//! `litigate query provenance`: list lowered numeric labels with HIR spans.
use super::check::checked;
use crate::cli::BasisArg;
use crate::hirx;
use crate::out::{Failure, Outcome, Report};
use litigate_base::Profile;
use litigate_lower::{Document, Lowerer, Options};
use litigate_sema::hir::{self, Authored, NodeKind};
use serde::Serialize;
use serde_json::{json, Value};

/// `query provenance` arguments.
pub struct Args {
    /// Package, `.lit`, or `.case` targets.
    pub paths: Vec<String>,
    /// Optional basis filter.
    pub basis: Option<BasisArg>,
    /// Keep only labels whose vintage is absent.
    pub no_vintage: bool,
    /// Library directories.
    pub lib: Vec<String>,
}

/// One authored numeric value.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Row {
    /// Package identifier.
    pub package: String,
    /// Node, edge, or link identifier.
    pub declaration: String,
    /// Numeric field.
    pub field: String,
    /// Lowered value.
    pub value: Value,
    /// Provenance lattice label.
    pub basis: String,
    /// Cite, dataset, calibration set, or expert name.
    pub source: Option<String>,
    /// Date associated with this number.
    pub vintage: Option<String>,
    /// Repository-relative `file:line`.
    pub span: String,
    #[serde(skip)]
    order: u8,
}

fn basis_name(basis: litigate_engine::model::Basis) -> (&'static str, u8) {
    use litigate_engine::model::Basis as B;
    match basis {
        B::Hole => ("hole", 0),
        B::Placeholder => ("placeholder", 1),
        B::Estimate => ("estimate", 2),
        B::Expert => ("expert", 3),
        B::Calibrated => ("calibrated", 4),
        B::Sourced => ("sourced", 5),
    }
}

fn wanted_basis(arg: BasisArg) -> &'static str {
    match arg {
        BasisArg::Sourced => "sourced",
        BasisArg::Calibrated => "calibrated",
        BasisArg::Expert => "expert",
        BasisArg::Estimate => "estimate",
        BasisArg::Placeholder => "placeholder",
        BasisArg::Hole => "hole",
    }
}

fn rational(value: hir::Rational) -> Value {
    if value.denominator == 1 {
        json!(value.numerator)
    } else {
        json!(format!("{}/{}", value.numerator, value.denominator))
    }
}

fn money(value: &hir::Money) -> Value {
    json!({"minor_units": value.minor_units, "currency": value.currency})
}

fn node_value(node: &hir::Node) -> Option<Value> {
    match &node.kind {
        NodeKind::Terminal {
            payoff: Some(Authored::Known(v)),
            ..
        } => Some(money(v)),
        NodeKind::Terminal {
            payoff: Some(Authored::Hole(_)),
            ..
        } => Some(Value::Null),
        _ => None,
    }
}

fn row(
    package: &str,
    declaration: String,
    field: String,
    value: Value,
    label: &litigate_engine::model::BasisLabel,
    span: Option<&litigate_base::Span>,
) -> Option<Row> {
    let (basis, order) = basis_name(label.basis);
    let span = span?;
    Some(Row {
        package: package.to_owned(),
        declaration,
        field,
        value,
        basis: basis.to_owned(),
        source: label.source.clone(),
        vintage: label.vintage.clone(),
        span: format!("{}:{}", span.file.0, span.line),
        order,
    })
}

fn span_sort_key(span: &str) -> (&str, u32) {
    span.rsplit_once(':').map_or((span, 0), |(file, line)| {
        (file, line.parse().unwrap_or_default())
    })
}

fn compare_rows(a: &Row, b: &Row) -> std::cmp::Ordering {
    (
        a.order,
        &a.package,
        span_sort_key(&a.span),
        &a.declaration,
        &a.field,
    )
        .cmp(&(
            b.order,
            &b.package,
            span_sort_key(&b.span),
            &b.declaration,
            &b.field,
        ))
}

fn edge_value(field: &str, edge: &hir::Edge) -> Option<Value> {
    match field {
        "probability" => Some(match edge.probability.as_ref()? {
            Authored::Known(value) => rational(*value),
            Authored::Hole(_) => Value::Null,
        }),
        "cost" => Some(match edge.cost.as_ref()? {
            Authored::Known(value) => money(value),
            Authored::Hole(_) => Value::Null,
        }),
        "hours" => Some(rational(edge.hours?)),
        "duration" => {
            let value = edge.duration.as_ref()?;
            Some(
                json!({"min": value.min.map(rational), "mode": rational(value.mode), "max": value.max.map(rational)}),
            )
        }
        _ => None,
    }
}

fn labeled_row(
    package: &str,
    pack_id: &str,
    label: &litigate_engine::model::BasisLabel,
    document: &litigate_lower::Pack,
    hir: &hir::Package,
) -> Option<Row> {
    let mut parts = label.pointer.split('/');
    let _ = parts.next();
    let section = parts.next()?;
    let index: usize = parts.next()?.parse().ok()?;
    let field = parts.next()?;
    if section == "nodes" {
        let lowered = document.nodes.get(index)?;
        let node = hirx::nodes(hir)
            .into_iter()
            .find(|n| n.meta.id == lowered.id)?;
        let value = node_value(node)?;
        return row(
            package,
            format!("{}::{}", pack_id, lowered.id),
            field.to_owned(),
            value,
            label,
            node.meta.span.as_ref(),
        );
    }
    if section != "edges" {
        return None;
    }
    let edge = document.edges.get(index)?;
    let source = hirx::nodes(hir)
        .into_iter()
        .find(|node| node.meta.id == edge.from)?;
    let prior_anon = document.edges[..index]
        .iter()
        .filter(|candidate| {
            candidate.from == edge.from && candidate.to == edge.to && candidate.id.is_none()
        })
        .count();
    let authored_edge = if let Some(id) = &edge.id {
        source
            .edges
            .iter()
            .find(|candidate| candidate.id.as_ref() == Some(id))?
    } else {
        source
            .edges
            .iter()
            .filter(|candidate| candidate.to == edge.to && candidate.id.is_none())
            .nth(prior_anon)?
    };
    let edge_id = edge
        .id
        .clone()
        .unwrap_or_else(|| format!("{}->{}#{}", edge.from, edge.to, prior_anon));
    row(
        package,
        format!("{pack_id}::{edge_id}"),
        if field == "hours" {
            "effort".to_owned()
        } else {
            field.to_owned()
        },
        edge_value(field, authored_edge)?,
        label,
        authored_edge.span.as_ref(),
    )
}

fn link_row(
    package: &str,
    pack_id: &str,
    label: &litigate_engine::model::BasisLabel,
    document: &litigate_lower::LinkFile,
    hir: &hir::Package,
) -> Option<Row> {
    let mut parts = label.pointer.split('/');
    let _ = parts.next();
    if parts.next()? != "links" {
        return None;
    }
    let index: usize = parts.next()?.parse().ok()?;
    let field = parts.next()?;
    let edge = document.links.get(index)?;
    let authored = hirx::declarations(hir)
        .filter_map(|decl| match decl {
            hir::Declaration::Link(link) => Some(link),
            _ => None,
        })
        .nth(index)?;
    let id = edge
        .id
        .clone()
        .unwrap_or_else(|| format!("{}->{}#0", edge.from, edge.to));
    row(
        package,
        format!("{pack_id}::{id}"),
        if field == "hours" {
            "effort".to_owned()
        } else {
            field.to_owned()
        },
        edge_value(field, &authored.edge)?,
        label,
        authored.edge.span.as_ref(),
    )
}

fn resolved_label(
    pointer: String,
    object: &str,
    labels: &[litigate_engine::model::BasisLabel],
    holes: &[litigate_engine::model::Hole],
) -> litigate_engine::model::BasisLabel {
    if holes.iter().any(|hole| hole.pointer == pointer) {
        return litigate_engine::model::BasisLabel {
            pointer,
            basis: litigate_engine::model::Basis::Hole,
            source: None,
            vintage: None,
        };
    }
    let mut label = labels
        .iter()
        .find(|label| label.pointer == pointer)
        .or_else(|| labels.iter().find(|label| label.pointer == object))
        .cloned()
        .unwrap_or(litigate_engine::model::BasisLabel {
            pointer: pointer.clone(),
            basis: litigate_engine::model::Basis::Estimate,
            source: None,
            vintage: None,
        });
    label.pointer = pointer;
    label
}

fn numeric_pointers(
    pack: &litigate_lower::Pack,
    holes: &[litigate_engine::model::Hole],
) -> Vec<(String, String)> {
    let mut pointers = Vec::new();
    for (i, node) in pack.nodes.iter().enumerate() {
        let pointer = format!("/nodes/{i}/payoff");
        if node.payoff.is_some() || holes.iter().any(|h| h.pointer == pointer) {
            pointers.push((pointer, format!("/nodes/{i}")));
        }
    }
    for (i, edge) in pack.edges.iter().enumerate() {
        let object = format!("/edges/{i}");
        for (field, present) in [
            ("probability", edge.probability.is_some()),
            ("cost", edge.cost.is_some()),
            ("hours", edge.hours.is_some()),
            ("duration", edge.duration.is_some()),
        ] {
            let pointer = format!("{object}/{field}");
            if present || holes.iter().any(|h| h.pointer == pointer) {
                pointers.push((pointer, object.clone()));
            }
        }
    }
    pointers
}

fn link_pointers(
    links: &litigate_lower::LinkFile,
    holes: &[litigate_engine::model::Hole],
) -> Vec<(String, String)> {
    let mut pointers = Vec::new();
    for (i, edge) in links.links.iter().enumerate() {
        let object = format!("/links/{i}");
        for (field, present) in [
            ("probability", edge.probability.is_some()),
            ("cost", edge.cost.is_some()),
            ("hours", edge.hours.is_some()),
            ("duration", edge.duration.is_some()),
        ] {
            let pointer = format!("{object}/{field}");
            if present || holes.iter().any(|h| h.pointer == pointer) {
                pointers.push((pointer, object.clone()));
            }
        }
    }
    pointers
}

fn package_rows(pkg: &hir::Package, all: &[&hir::Package]) -> Result<Vec<Row>, Failure> {
    let lowered = Lowerer::new(all)
        .package(pkg, &Options::default())
        .map_err(|diagnostics| {
            Failure::new(format!(
                "could not lower {}: {}",
                pkg.manifest.name,
                diagnostics
                    .first()
                    .map_or("unknown error", |d| d.message.as_str())
            ))
        })?;
    let (labels, holes) = match &lowered.document {
        Document::Pack(pack) => (
            pack.lowered
                .as_ref()
                .map_or_else(Vec::new, |l| l.basis.clone()),
            pack.lowered
                .as_ref()
                .map_or_else(Vec::new, |l| l.holes.clone()),
        ),
        Document::Links(links) => (
            links
                .lowered
                .as_ref()
                .map_or_else(Vec::new, |l| l.basis.clone()),
            links
                .lowered
                .as_ref()
                .map_or_else(Vec::new, |l| l.holes.clone()),
        ),
    };
    let rows = match &lowered.document {
        Document::Pack(pack) => numeric_pointers(pack, &holes)
            .iter()
            .filter_map(|(pointer, object)| {
                let label = resolved_label(pointer.clone(), object, &labels, &holes);
                labeled_row(&pkg.manifest.name, &pkg.manifest.pack_id, &label, pack, pkg)
            })
            .collect(),
        Document::Links(links) => link_pointers(links, &holes)
            .iter()
            .filter_map(|(pointer, object)| {
                let label = resolved_label(pointer.clone(), object, &labels, &holes);
                link_row(
                    &pkg.manifest.name,
                    &pkg.manifest.pack_id,
                    &label,
                    links,
                    pkg,
                )
            })
            .collect(),
    };
    Ok(rows)
}

fn table(rows: &[Row]) -> String {
    let headers = [
        "PACKAGE",
        "DECLARATION",
        "FIELD",
        "VALUE",
        "BASIS",
        "SOURCE",
        "VINTAGE",
        "SPAN",
    ];
    let values: Vec<[String; 8]> = rows
        .iter()
        .map(|r| {
            [
                r.package.clone(),
                r.declaration.clone(),
                r.field.clone(),
                r.value.to_string(),
                r.basis.clone(),
                r.source.clone().unwrap_or_default(),
                r.vintage.clone().unwrap_or_default(),
                r.span.clone(),
            ]
        })
        .collect();
    let widths: Vec<usize> = (0..headers.len())
        .map(|i| {
            values
                .iter()
                .map(|r| r[i].len())
                .max()
                .unwrap_or(0)
                .max(headers[i].len())
        })
        .collect();
    let mut out = headers
        .iter()
        .enumerate()
        .map(|(i, h)| format!("{h:width$}", width = widths[i]))
        .collect::<Vec<_>>()
        .join("  ");
    out.push('\n');
    for values in values {
        out.push_str(
            &values
                .iter()
                .enumerate()
                .map(|(i, v)| format!("{v:width$}", width = widths[i]))
                .collect::<Vec<_>>()
                .join("  "),
        );
        out.push('\n');
    }
    out
}

/// Run the provenance query over the compiler's checked packages and lowered labels.
pub fn query(args: &Args) -> Outcome {
    let roots = super::where_::roots(&args.paths, &args.lib)?;
    let checked = checked(&roots, Profile::Dev, &Options::default(), &args.lib)?;
    let all: Vec<&hir::Package> = checked
        .session
        .packages
        .iter()
        .filter_map(|p| p.check.package.as_ref())
        .collect();
    let mut rows: Vec<Row> = all
        .iter()
        .map(|pkg| package_rows(pkg, &all))
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .flatten()
        .collect();
    if let Some(basis) = args.basis {
        let wanted = wanted_basis(basis);
        rows.retain(|row| row.basis == wanted);
    }
    if args.no_vintage {
        rows.retain(|row| row.vintage.is_none());
    }
    rows.sort_by(compare_rows);
    let result_rows = rows.clone();
    let mut report = Report::of(json!({"rows": result_rows, "total": rows.len()}));
    report.provenance = checked.provenance(None, None);
    report.text = Some(table(&rows));
    Ok(report)
}
