//! Typing a case event call against the used packages' event types, and
//! their dependencies' (SPEC
//! §5.2: case events are complete constructor calls). The result is HIR in
//! the primary package's view: a name declared in the primary package is
//! bare, any other is `pack::Name`.
use super::graph::Graph;
use crate::db::parse::tree::{Expr as Src, ExprKind as S, RefPath};
use crate::db::span::{Report, TextRange};
use crate::hir::{Expr, ExprKind, Type, Value};
use crate::resolve::suggest::nearest;
use crate::types::rational;
use litigate_base::{Applicability, DiagnosticCode as C, Span};

/// Converts a range to a span.
pub(super) type ToSpan<'a> = &'a dyn Fn(TextRange) -> Span;

/// Splits `q::Name` into `(pack, Name)` when `q` is a used pack or a
/// dependency of one.
fn split(graph: &Graph<'_>, path: &RefPath) -> (String, String) {
    let text = path.display();
    match text.split_once("::") {
        Some((p, rest))
            if graph.packages.iter().any(|k| k.manifest.pack_id == p) || graph.is_dependency(p) =>
        {
            (p.to_owned(), rest.to_owned())
        }
        _ => (graph.primary().to_owned(), text),
    }
}

/// `name` declared in `pack`, spelled in the primary view.
fn spelled(graph: &Graph<'_>, pack: &str, name: &str) -> String {
    if pack == graph.primary() {
        name.to_owned()
    } else {
        format!("{pack}::{name}")
    }
}

/// Types the call `callee(args)` written at `range` as a complete event
/// call; `None` after reporting.
pub(super) fn call(
    graph: &Graph<'_>,
    (callee, args, range): (&RefPath, &[Src], TextRange),
    span: ToSpan<'_>,
    out: &mut Vec<Report>,
) -> Option<Expr> {
    let (pack, name) = split(graph, callee);
    let Some(ty) = graph.event_type(&pack, &name) else {
        let names: Vec<&str> = graph
            .event_types()
            .filter(|(p, _)| *p == pack)
            .map(|(_, t)| t.name.as_str())
            .collect();
        let mut r = Report::new(
            C::UnresolvedName,
            callee.range,
            format!("`{pack}` declares no event `{name}`"),
        );
        if let Some(best) = nearest(&name, names.iter().copied()) {
            r = r.fix(
                callee.range,
                spelled(graph, &pack, best),
                Applicability::MaybeIncorrect,
            );
        }
        out.push(r.note(format!("events: {}", names.join(", "))));
        return None;
    };
    if args.len() != ty.params.len() {
        let sig: Vec<String> = ty
            .params
            .iter()
            .map(|(p, t)| format!("{p}: {}", crate::types::ty::display(t)))
            .collect();
        out.push(
            Report::new(
                C::TypeMismatch,
                range,
                format!(
                    "`{name}` takes {} argument{}, found {}",
                    ty.params.len(),
                    if ty.params.len() == 1 { "" } else { "s" },
                    args.len()
                ),
            )
            .note(format!(
                "`{name}({})`; case events must be complete",
                sig.join(", ")
            )),
        );
        return None;
    }
    let typed: Option<Vec<Expr>> = args
        .iter()
        .zip(&ty.params)
        .map(|(a, (_, t))| argument(graph, &pack, a, t, span, out))
        .collect();
    let callee = spelled(graph, &pack, &name);
    Some(Expr {
        ty: Type::Named(callee.clone()),
        kind: ExprKind::Call {
            callee,
            args: typed?,
        },
        span: Some(span(range)),
    })
}

fn literal(value: Value, ty: Type, r: TextRange, span: ToSpan<'_>) -> Expr {
    Expr {
        ty,
        kind: ExprKind::Literal { value },
        span: Some(span(r)),
    }
}

/// An `EdgeRef` argument written as a string: L2001, with a fix unquoting
/// it when the string names an edge.
fn quoted_edge(
    graph: &Graph<'_>,
    s: &str,
    range: TextRange,
    span: ToSpan<'_>,
    out: &mut Vec<Report>,
) -> Option<Expr> {
    let mut r = Report::new(
        C::TypeMismatch,
        range,
        format!("expected an `EdgeRef` here, found the string {s:?}"),
    )
    .note("an edge is named by its id, unquoted: `edge`, or `pack::edge` in another package");
    if edge(graph, s, range, span, &mut Vec::new()).is_some() {
        r = r.fix(range, s.to_owned(), Applicability::MachineApplicable);
    }
    out.push(r);
    None
}

/// One argument against parameter type `t` (written in `pack`'s view).
fn argument(
    graph: &Graph<'_>,
    pack: &str,
    a: &Src,
    t: &Type,
    span: ToSpan<'_>,
    out: &mut Vec<Report>,
) -> Option<Expr> {
    let mismatch = |out: &mut Vec<Report>, what: &str| {
        out.push(Report::new(
            C::TypeMismatch,
            a.range,
            format!("expected {what} here"),
        ));
        None
    };
    match (t, &a.kind) {
        (Type::Named(n), S::Ref(path)) => variant(graph, pack, n, path, span, out),
        (Type::Named(n), _) => mismatch(out, &format!("a `{n}` variant")),
        // The lexer rejects a date literal on no calendar day.
        (Type::Date, S::Date(d)) => {
            Some(literal(Value::Date(d.clone()), Type::Date, a.range, span))
        }
        // An unknown date (`?`): a notice whose designated due date the
        // matter does not have. The deadline it designates is then unknown
        // (SPEC §5.3), never counted in its place.
        (Type::Date, S::Hole(reason)) => Some(literal(
            Value::Hole(reason.clone()),
            Type::Date,
            a.range,
            span,
        )),
        (Type::Date, _) => mismatch(out, "a valid date or `?`"),
        (Type::String, S::Str(s)) => Some(literal(
            Value::String(s.clone()),
            Type::String,
            a.range,
            span,
        )),
        (Type::String, _) => mismatch(out, "a string"),
        (Type::Edge, S::Ref(path)) => edge(graph, &path.display(), a.range, span, out),
        (Type::Edge, S::Str(s)) => quoted_edge(graph, s, a.range, span, out),
        (Type::Edge, _) => mismatch(out, "an edge id (`edge` or `pack::edge`)"),
        (Type::Boolean, S::Bool(b)) => {
            Some(literal(Value::Boolean(*b), Type::Boolean, a.range, span))
        }
        (Type::Boolean, _) => mismatch(out, "`true` or `false`"),
        (Type::Number, S::Number(n)) => match rational::parse_decimal(n) {
            Ok(r) => Some(literal(Value::Number(r), Type::Number, a.range, span)),
            Err(_) => mismatch(out, "an exact number"),
        },
        (Type::Number, _) => mismatch(out, "a number"),
        // A money literal's suffix and a `Money<_>` type are both `usd`, the
        // only currency (the lexer and type resolution reject any other).
        (Type::Money(cur), arg) => match arg {
            S::Money(text) => super::resolve::money(text),
            _ => None,
        }
        .map_or_else(
            || mismatch(out, &format!("a `{cur}` amount")),
            |m| {
                Some(literal(
                    Value::Money(m),
                    Type::Money(cur.clone()),
                    a.range,
                    span,
                ))
            },
        ),
        (other, _) => {
            out.push(Report::new(
                C::TypeMismatch,
                a.range,
                format!(
                    "an event argument of type `{}` cannot be written in a case",
                    crate::types::ty::display(other)
                ),
            ));
            None
        }
    }
}

/// The edge `written` names (`edge` in the primary package, else
/// `pack::edge`; any version, link edges included), as an `EdgeRef`
/// literal spelled in the primary view.
fn edge(
    graph: &Graph<'_>,
    written: &str,
    range: TextRange,
    span: ToSpan<'_>,
    out: &mut Vec<Report>,
) -> Option<Expr> {
    let qualified = edge_ref(graph, written, range, out)?;
    let spelled = graph.respell(graph.primary(), &qualified);
    Some(literal(Value::Edge(spelled), Type::Edge, range, span))
}

/// The edge `written` at `range` names, qualified (`pack::edge`); L1002
/// with the nearest id as a fix when it names none.
pub(super) fn edge_ref(
    graph: &Graph<'_>,
    written: &str,
    range: TextRange,
    out: &mut Vec<Report>,
) -> Option<String> {
    let qualified = graph.qualify(graph.primary(), written);
    let (pack, local) = qualified.split_once("::").unwrap_or(("", written));
    let ids = graph.edge_ids(pack);
    if ids.contains(local) {
        return Some(qualified);
    }
    let mut r = Report::new(
        C::UnresolvedName,
        range,
        format!("`{pack}` has no edge `{local}`"),
    )
    .note("an `EdgeRef` names an edge by its id: `edge`, or `pack::edge` in another package");
    if let Some(best) = nearest(local, ids.iter().copied()) {
        r = r.fix(
            range,
            graph.respell(graph.primary(), &format!("{pack}::{best}")),
            Applicability::MaybeIncorrect,
        );
    }
    out.push(r);
    None
}

/// A variant of enum `n` (written in `pack`'s view): `v`, `Enum::v`, or
/// `q::Enum::v`.
fn variant(
    graph: &Graph<'_>,
    pack: &str,
    n: &str,
    path: &RefPath,
    span: ToSpan<'_>,
    out: &mut Vec<Report>,
) -> Option<Expr> {
    let qualified = graph.qualify(pack, n);
    let (enum_pack, enum_name) = qualified.split_once("::").unwrap_or((pack, n));
    let variants = graph.variants(enum_pack, enum_name).unwrap_or_default();
    let written = path.display();
    let local = written.rsplit("::").next().unwrap_or(&written);
    let prefix_ok = written == local
        || written == format!("{enum_name}::{local}")
        || written == format!("{enum_pack}::{enum_name}::{local}");
    if prefix_ok && variants.iter().any(|v| v == local) {
        let ty = spelled(graph, enum_pack, enum_name);
        return Some(literal(
            Value::Variant(format!("{ty}::{local}")),
            Type::Named(ty),
            path.range,
            span,
        ));
    }
    let mut r = Report::new(
        C::TypeMismatch,
        path.range,
        format!("expected a `{enum_name}` variant, found `{written}`"),
    )
    .note(format!("variants: {}", variants.join(", ")));
    if let Some(best) = nearest(local, variants.iter().map(String::as_str)) {
        r = r.fix(path.range, best, Applicability::MaybeIncorrect);
    }
    out.push(r);
    None
}
