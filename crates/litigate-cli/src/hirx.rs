//! Read-only helpers over HIR: declarations by kind, version selection
//! (SPEC §5.4, §5.5, through sema's one query), and event-pattern matching
//! (SPEC §5.6).
use litigate_sema::hir::{self, Declaration, Expr, ExprKind, Metadata, Value};
use litigate_sema::precedence::select::select;
use litigate_sema::temporal::{world::meta_of, World};

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

/// True when `date` falls in `meta`'s in-force window: a lookup filter
/// (`where id@D`, `query --effective-on`). Which version applies is
/// [`selected`]'s answer, which also weighs `known_at` and supersession.
#[must_use]
pub fn in_force(meta: &Metadata, date: &str) -> bool {
    let e = &meta.effective;
    e.from.as_str() <= date && e.to.as_deref().is_none_or(|to| date < to)
}

/// The versions of `pkg`'s declarations selected at `(as_of, known_at)`
/// (`None`: the latest corpus) by sema's one query
/// (`precedence::select` over the versions `temporal::select` finds), in
/// the law `pkg` sees among `all` (itself, its dependencies and `core`: a
/// declaration of one may supersede or displace another's), in
/// declaration order. A displaced declaration is not selected.
#[must_use]
pub fn selected<'p>(
    all: &[&'p hir::Package],
    pkg: &hir::Package,
    as_of: &str,
    known_at: Option<&str>,
) -> Vec<&'p Declaration> {
    let roots: Vec<&'p hir::Package> = all
        .iter()
        .copied()
        .filter(|p| p.manifest.name == pkg.manifest.name)
        .collect();
    let world = World::closure(&roots, all);
    (0..world.decls.len())
        .filter(|&i| world.pack_of(i) == pkg.manifest.pack_id)
        .filter(|&i| select(&world, &world.key(i), as_of, known_at) == Ok(i))
        .map(|i| world.decls[i].decl)
        .collect()
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

/// The rule (`rule duty x { deadline d ... }`) declaring deadline `d`.
#[must_use]
pub fn rule_of<'p>(pkg: &'p hir::Package, d: &str) -> Option<&'p hir::Rule> {
    declarations(pkg).find_map(|x| match x {
        Declaration::Rule(r) if r.declarations.iter().any(|n| local(n) == d) => Some(r),
        _ => None,
    })
}

/// Every version of every node of `pkg`.
#[must_use]
pub fn nodes(pkg: &hir::Package) -> Vec<&hir::Node> {
    declarations(pkg)
        .filter_map(|d| match d {
            Declaration::Node(n) => Some(n),
            _ => None,
        })
        .collect()
}

/// The node versions of `pkg` selected at `(as_of, known_at)` ([`selected`]).
#[must_use]
pub fn nodes_at<'p>(
    all: &[&'p hir::Package],
    pkg: &hir::Package,
    as_of: &str,
    known_at: Option<&str>,
) -> Vec<&'p hir::Node> {
    selected(all, pkg, as_of, known_at)
        .into_iter()
        .filter_map(|d| match d {
            Declaration::Node(n) => Some(n),
            _ => None,
        })
        .collect()
}

/// The comparable key of an event argument: a variant or ref's local name,
/// a date as written, `?` for an unknown value, or the literal's JSON.
fn arg_key(e: &Expr) -> String {
    match &e.kind {
        ExprKind::Literal {
            value: Value::Variant(v),
        } => local(v).to_owned(),
        ExprKind::Ref { target } => local(target).to_owned(),
        ExprKind::Literal {
            value: Value::Date(d),
        } => d.clone(),
        ExprKind::Literal {
            value: Value::Hole(_),
        } => "?".to_owned(),
        ExprKind::Literal {
            value: Value::Edge(edge),
        } => edge.clone(),
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

    fn expr(kind: ExprKind) -> Expr {
        Expr {
            ty: Type::Unit,
            kind,
            span: None,
        }
    }

    #[test]
    fn wildcards_refs_and_literals_as_event_arguments() {
        let wildcard = call("Served", &["_"]);
        assert!(matches(&wildcard, &call("Served", &["D::x", "M::Mail"])));
        let event = expr(ExprKind::Call {
            callee: "Filed".into(),
            args: vec![
                expr(ExprKind::Ref {
                    target: "m.party".into(),
                }),
                expr(ExprKind::Literal {
                    value: Value::Date("2025-01-02".into()),
                }),
                expr(ExprKind::Literal {
                    value: Value::Hole(None),
                }),
                expr(ExprKind::Literal {
                    value: Value::Edge("e-answer".into()),
                }),
                expr(ExprKind::Literal {
                    value: Value::Boolean(true),
                }),
            ],
        });
        insta::assert_snapshot!(event_text(&event), @r#"Filed(party, 2025-01-02, ?, e-answer, {"expr":"literal","value":{"kind":"boolean","value":true}})"#);
        let literal = variant("D::x");
        assert_eq!(event_parts(&literal), None);
        assert_eq!(event_text(&literal), "");
        assert!(!matches(&literal, &event));
    }

    #[test]
    fn selection_honours_dates_and_known_at() {
        let pkg = crate::testkit::hir_with("");
        let meta = &deadlines(&pkg, "answer-period").next().unwrap().meta;
        assert!(in_force(meta, "2025-03-03"));
        assert!(!in_force(meta, "1900-01-01"));
        let mut ended = meta.clone();
        ended.effective.to = Some("2025-01-01".into());
        assert!(in_force(&ended, "2024-12-31"));
        assert!(!in_force(&ended, "2025-01-01"));
        let all = [&pkg];
        assert!(nodes_at(&all, &pkg, "2000-01-01", None).is_empty());
        assert_eq!(
            nodes(&pkg).len(),
            nodes_at(&all, &pkg, "2025-03-03", None).len()
        );
        let known = meta.effective.known_at.clone();
        assert_eq!(
            selected(&all, &pkg, "2025-03-03", Some(&known)).len(),
            selected(&all, &pkg, "2025-03-03", None).len()
        );
        assert!(selected(&all, &pkg, "2025-03-03", Some("1900-01-01")).is_empty());
    }

    #[test]
    fn every_declaration_kind_and_id() {
        let pkg = crate::testkit::hir_with(
            "default dd when 1 < 2 => 1\nexception ee undercuts dd when 2 < 3",
        );
        let kinds: Vec<(&str, Option<String>)> = declarations(&pkg)
            .map(|d| (decl_kind(d), decl_id(d)))
            .collect();
        insta::assert_debug_snapshot!(kinds);
        let targets = crate::ws::discover(&["../../lit/links".to_owned()]).unwrap();
        let session = crate::ws::Session::load(&targets, &[]).unwrap();
        let mut library: Vec<&str> = session
            .hir()
            .into_iter()
            .flat_map(declarations)
            .map(decl_kind)
            .collect();
        library.sort_unstable();
        library.dedup();
        for kind in ["instance", "link", "transition", "type"] {
            assert!(library.contains(&kind), "{kind} not in {library:?}");
        }
    }
}
