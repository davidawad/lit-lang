//! Read-only helpers over HIR: declarations by kind, version selection by
//! date (SPEC §5.4), and event-pattern matching (SPEC §5.6).
use litigate_sema::hir::{self, Declaration, Expr, ExprKind, Metadata, Value};
use litigate_sema::temporal::world::meta_of;

/// The last `::`- or `.`-separated segment of a qualified name.
#[must_use]
pub fn local(name: &str) -> &str {
    let tail = name.rsplit("::").next().unwrap_or(name);
    tail.rsplit('.').next().unwrap_or(tail)
}

/// Every declaration of `pkg`, in module order then source order.
pub fn declarations(pkg: &hir::Package) -> impl Iterator<Item = &Declaration> {
    pkg.modules.iter().flat_map(|m| m.declarations.iter())
}

/// True when `meta`'s version is in force on `date` and recorded by `known_at`.
#[must_use]
pub fn in_force(meta: &Metadata, date: &str, known_at: Option<&str>) -> bool {
    let e = &meta.effective;
    e.from.as_str() <= date
        && e.to.as_deref().is_none_or(|to| date < to)
        && known_at.is_none_or(|k| e.known_at.as_str() <= k)
}

/// The declaration's id (`meta.id`, or a test/type name).
#[must_use]
pub fn decl_id(decl: &Declaration) -> Option<String> {
    meta_of(decl).map(|m| m.id.clone()).or_else(|| match decl {
        Declaration::Test(t) => Some(t.name.clone()),
        Declaration::Enum(e) => Some(e.name.clone()),
        Declaration::Event(e) => Some(e.name.clone()),
        Declaration::TypeAlias(t) => Some(t.name.clone()),
        Declaration::Authority(a) => Some(a.name.clone()),
        Declaration::Instance(i) => Some(i.name.clone()),
        _ => None,
    })
}

/// The declaration kind as written in source.
#[must_use]
pub fn decl_kind(decl: &Declaration) -> &'static str {
    match decl {
        Declaration::Node(n) => match n.kind {
            hir::NodeKind::State => "state",
            hir::NodeKind::Decision => "decision",
            hir::NodeKind::Chance { .. } => "chance",
            hir::NodeKind::Terminal { .. } => "terminal",
        },
        Declaration::Link(_) => "link",
        Declaration::Instance(_) => "instance",
        Declaration::Deadline(_) => "deadline",
        Declaration::Counting(_) => "counting",
        Declaration::Calendar(_) => "calendar",
        Declaration::Rule(_) => "rule",
        Declaration::Authority(_) => "authority",
        Declaration::Default(_) => "default",
        Declaration::Exception(_) => "exception",
        Declaration::Transition(_) => "transition",
        Declaration::TypeAlias(_) => "type",
        Declaration::Enum(_) => "enum",
        Declaration::Event(_) => "event",
        Declaration::Test(_) => "test",
    }
}

/// Every version of deadline `name` (local or qualified) in `pkg`.
pub fn deadlines<'p>(pkg: &'p hir::Package, name: &str) -> impl Iterator<Item = &'p hir::Deadline> {
    let name = local(name).to_owned();
    declarations(pkg).filter_map(move |d| match d {
        Declaration::Deadline(x) if x.meta.id == name => Some(x),
        _ => None,
    })
}

/// The one version of deadline `name` in force on `date`.
///
/// # Errors
/// A message when no version, or more than one, is in force (L3007).
pub fn deadline_at<'p>(
    pkg: &'p hir::Package,
    name: &str,
    date: &str,
) -> Result<&'p hir::Deadline, String> {
    let all: Vec<&hir::Deadline> = deadlines(pkg, name).collect();
    if all.is_empty() {
        return Err(format!(
            "`{name}` is not a deadline of `{}`",
            pkg.manifest.name
        ));
    }
    let live: Vec<&hir::Deadline> = all
        .into_iter()
        .filter(|d| in_force(&d.meta, date, None))
        .collect();
    match live.as_slice() {
        [one] => Ok(one),
        [] => Err(format!("no version of `{name}` is in force on {date}")),
        _ => Err(format!(
            "{} versions of `{name}` are in force on {date}",
            live.len()
        )),
    }
}

/// The counting rule `id` declared in `pkg`, if any.
#[must_use]
pub fn counting<'p>(pkg: &'p hir::Package, id: &str) -> Option<&'p hir::Counting> {
    let id = local(id);
    declarations(pkg).find_map(|d| match d {
        Declaration::Counting(c) if c.meta.id == id => Some(c),
        _ => None,
    })
}

/// Nodes of `pkg` in force on `date` (every version when `date` is `None`).
#[must_use]
pub fn nodes_at<'p>(pkg: &'p hir::Package, date: Option<&str>) -> Vec<&'p hir::Node> {
    declarations(pkg)
        .filter_map(|d| match d {
            Declaration::Node(n) if date.is_none_or(|day| in_force(&n.meta, day, None)) => Some(n),
            _ => None,
        })
        .collect()
}

/// The comparable key of an event argument: a variant or ref's local name,
/// or the literal's JSON.
fn arg_key(e: &Expr) -> String {
    match &e.kind {
        ExprKind::Literal {
            value: Value::Variant(v),
        } => local(v).to_owned(),
        ExprKind::Ref { target } => local(target).to_owned(),
        other => serde_json::to_string(other).unwrap_or_default(),
    }
}

/// The event constructor and argument keys of a call expression.
#[must_use]
pub fn event_parts(e: &Expr) -> Option<(String, Vec<String>)> {
    match &e.kind {
        ExprKind::Call { callee, args } => {
            Some((local(callee).to_owned(), args.iter().map(arg_key).collect()))
        }
        _ => None,
    }
}

/// True when `event` matches `pattern`: the same constructor, and every
/// argument the pattern gives equal to the event's at that position.
#[must_use]
pub fn matches(pattern: &Expr, event: &Expr) -> bool {
    match (event_parts(pattern), event_parts(event)) {
        (Some((pc, pa)), Some((ec, ea))) => {
            pc == ec && pa.len() <= ea.len() && pa.iter().zip(&ea).all(|(p, e)| p == "_" || p == e)
        }
        _ => false,
    }
}

/// Renders an event call as source-like text (`Served(complaint, Mail)`).
#[must_use]
pub fn event_text(e: &Expr) -> String {
    event_parts(e).map_or_else(String::new, |(callee, args)| {
        format!("{callee}({})", args.join(", "))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use litigate_sema::hir::Type;

    fn variant(v: &str) -> Expr {
        Expr {
            ty: Type::Unit,
            kind: ExprKind::Literal {
                value: Value::Variant(v.into()),
            },
            span: None,
        }
    }

    fn call(callee: &str, args: &[&str]) -> Expr {
        Expr {
            ty: Type::Unit,
            kind: ExprKind::Call {
                callee: callee.into(),
                args: args.iter().map(|a| variant(a)).collect(),
            },
            span: None,
        }
    }

    #[test]
    fn patterns_match_by_prefix() {
        let pattern = call("Served", &["Document::complaint"]);
        assert!(matches(
            &pattern,
            &call("Served", &["Document::complaint", "M::Mail"])
        ));
        assert!(!matches(&pattern, &call("Served", &["Document::answer"])));
        assert!(!matches(&pattern, &call("Filed", &["Document::complaint"])));
        assert!(!matches(
            &call("Served", &["a", "b"]),
            &call("Served", &["a"])
        ));
        assert_eq!(event_text(&call("m.Served", &["D::x"])), "Served(x)");
        assert_eq!(local("p::m.x"), "x");
    }
}
