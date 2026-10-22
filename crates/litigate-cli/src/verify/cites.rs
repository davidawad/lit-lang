//! `cite-resolves` (L1010) and rule-test coverage (L9001).
//!
//! A cite resolves when one of the package's source bindings (a source's
//! `cite_prefixes`, or a `cite_syntax` rule or shorthand prefix, of the
//! package or a dependency) is a prefix of it. When the binding has a
//! `path` and a heading template, the pinpoint's heading must also occur
//! in that L0 file. Cite holes are holes, not unresolved cites.
use crate::hirx;
use litigate_base::{Diagnostic, DiagnosticCode as C, Span};
use litigate_sema::hir::{self, Declaration, Expr, ExprKind};
use litigate_sema::temporal::world::meta_of;
use std::collections::BTreeSet;

/// One way a cite prefix binds to L0.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Binding {
    prefix: String,
    path: Option<String>,
    heading: Option<String>,
}

fn bindings(manifest: &hir::Manifest) -> Vec<Binding> {
    let source = |id: &Option<String>| manifest.sources.iter().find(|s| Some(&s.id) == id.as_ref());
    let from_sources = manifest.sources.iter().flat_map(|s| {
        s.cite_prefixes.iter().map(|p| Binding {
            prefix: p.clone(),
            path: s.path.clone(),
            heading: s.section_heading.as_ref().map(|h| h.replace("{n}", "{}")),
        })
    });
    let from_rules = manifest.cite_syntax.rules.iter().flat_map(|r| {
        r.prefixes.iter().map(|p| Binding {
            prefix: p.clone(),
            path: source(&r.source).and_then(|s| s.path.clone()),
            heading: Some(r.heading.clone()),
        })
    });
    let from_shorthands = manifest.cite_syntax.shorthands.iter().map(|s| Binding {
        prefix: s.prefix.clone(),
        path: None,
        heading: None,
    });
    from_sources
        .chain(from_rules)
        .chain(from_shorthands)
        .collect()
}

/// The section number after `prefix`: up to the first `(` or space.
fn section(rest: &str) -> &str {
    let rest = rest.trim_start().trim_start_matches('§').trim_start();
    rest.split(|c: char| c == '(' || c.is_whitespace())
        .next()
        .unwrap_or_default()
}

/// Why `cite` does not resolve, or `None` when it does.
fn unresolved(cite: &str, bindings: &[Binding]) -> Option<String> {
    let bound: Vec<(&Binding, &str)> = bindings
        .iter()
        .filter_map(|b| {
            let rest = cite.strip_prefix(b.prefix.as_str())?;
            (rest.is_empty() || !rest.starts_with(|c: char| c.is_alphanumeric()))
                .then_some((b, rest))
        })
        .collect();
    if bound.is_empty() {
        let known: Vec<&str> = bindings.iter().map(|b| b.prefix.as_str()).collect();
        return Some(if known.is_empty() {
            "the package declares no source bindings (`[sources.*] cite_prefixes`)".to_owned()
        } else {
            format!(
                "no source binding has its prefix; bound prefixes: {}",
                known.join(", ")
            )
        });
    }
    let located = bound.iter().any(|(b, rest)| match (&b.path, &b.heading) {
        (Some(path), Some(heading)) => std::fs::read_to_string(path)
            .is_ok_and(|text| text.contains(&heading.replace("{}", section(rest)))),
        _ => true,
    });
    (!located).then(|| {
        let (b, rest) = bound[0];
        format!(
            "`{}` has no heading `{}`",
            b.path.clone().unwrap_or_default(),
            b.heading
                .clone()
                .unwrap_or_default()
                .replace("{}", section(rest))
        )
    })
}

/// True when the declaration at `span` carries the `/// imported` marker.
#[must_use]
pub fn imported(span: &Span, sources: &[(String, String)]) -> bool {
    let Some((_, text)) = sources.iter().find(|(p, _)| *p == span.file.0) else {
        return false;
    };
    let line = usize::try_from(span.line).unwrap_or(0);
    text.lines()
        .take(line.saturating_sub(1))
        .collect::<Vec<_>>()
        .iter()
        .rev()
        .take_while(|l| l.trim_start().starts_with("///"))
        .any(|l| l.trim() == "/// imported")
}

/// True when `outer` contains `inner`.
fn contains(outer: &Span, inner: &Span) -> bool {
    outer.file == inner.file
        && (outer.line, outer.col) <= (inner.line, inner.col)
        && (inner.end_line, inner.end_col) <= (outer.end_line, outer.end_col)
}

/// L1010 for every cite of `pkg` that does not resolve. `cite_holes` are
/// the spans of `cite ?(..)` holes, which are reported as holes instead.
#[must_use]
pub fn resolve(
    pkg: &hir::Package,
    deps: &[&hir::Package],
    cite_holes: &[Span],
    fallback: &Span,
) -> Vec<Diagnostic> {
    let all: Vec<Binding> = std::iter::once(&pkg.manifest)
        .chain(deps.iter().map(|d| &d.manifest))
        .flat_map(bindings)
        .collect();
    hirx::declarations(pkg)
        .filter_map(|d| match d {
            Declaration::Test(t) => Some((t.cite.as_ref()?, t.span.as_ref(), t.name.clone())),
            _ => {
                let m = meta_of(d)?;
                Some((m.cite.as_ref()?, m.span.as_ref(), m.id.clone()))
            }
        })
        .filter(|(_, span, _)| !span.is_some_and(|s| cite_holes.iter().any(|h| contains(s, h))))
        .filter_map(|(cite, span, id)| {
            let why = unresolved(cite, &all)?;
            Some(Diagnostic::error(
                C::UnboundCite,
                span.cloned().unwrap_or_else(|| fallback.clone()),
                format!("the cite `{cite}` of `{id}` does not resolve: {why}"),
            ))
        })
        .collect()
}

fn deadline_refs(e: &Expr, out: &mut BTreeSet<String>) {
    match &e.kind {
        ExprKind::Call { callee, args } => {
            if hirx::local(callee) == "deadline" {
                if let Some(ExprKind::Ref { target }) = args.first().map(|a| &a.kind) {
                    out.insert(hirx::local(target).to_owned());
                }
            }
            args.iter().for_each(|a| deadline_refs(a, out));
        }
        ExprKind::Unary { operand, .. } => deadline_refs(operand, out),
        ExprKind::Binary { lhs, rhs, .. } => {
            deadline_refs(lhs, out);
            deadline_refs(rhs, out);
        }
        _ => {}
    }
}

/// L9001 for each deadline no test evaluates and each counting rule no
/// tested deadline is counted by (one per declaration name).
#[must_use]
pub fn untested(pkg: &hir::Package, fallback: &Span) -> Vec<Diagnostic> {
    let mut tested = BTreeSet::new();
    for d in hirx::declarations(pkg) {
        if let Declaration::Test(t) = d {
            for x in &t.expects {
                deadline_refs(&x.lhs, &mut tested);
                deadline_refs(&x.rhs, &mut tested);
            }
        }
    }
    let counted: BTreeSet<String> = hirx::declarations(pkg)
        .filter_map(|d| match d {
            Declaration::Deadline(x) if tested.contains(&x.meta.id) => {
                Some(hirx::local(&x.counting).to_owned())
            }
            _ => None,
        })
        .collect();
    let mut seen = BTreeSet::new();
    hirx::declarations(pkg)
        .filter_map(|d| match d {
            Declaration::Deadline(x) if !tested.contains(&x.meta.id) => Some(("deadline", &x.meta)),
            Declaration::Counting(c) if !counted.contains(&c.meta.id) => {
                Some(("counting rule", &c.meta))
            }
            _ => None,
        })
        .filter(|(_, m)| seen.insert(m.id.clone()))
        .map(|(kind, m)| {
            Diagnostic::error(
                C::RuleWithoutTest,
                m.span.clone().unwrap_or_else(|| fallback.clone()),
                format!("{kind} `{}` has no inline test", m.id),
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn binding(prefix: &str) -> Binding {
        Binding {
            prefix: prefix.into(),
            path: None,
            heading: None,
        }
    }

    #[test]
    fn prefixes_bind_at_word_boundaries() {
        let b = [binding("FRCP")];
        assert_eq!(unresolved("FRCP 12(a)", &b), None);
        assert!(unresolved("FRCPX 1", &b).is_some());
        assert!(unresolved("RCFC 6", &b).is_some());
        assert!(unresolved("FRCP 1", &[]).is_some_and(|m| m.contains("no source bindings")));
        assert_eq!(section(" 12(a)(1)"), "12");
        assert_eq!(section(" § 2074"), "2074");
    }
}
