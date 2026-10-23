//! The package under verification and lookups over its HIR.
use crate::diag::start_of;
use litigate_base::{Profile, Span};
use litigate_sema::hir::{self, Declaration, RuleKind};
use litigate_sema::temporal::world::meta_of;

/// One package being verified, with the workspace it resolves in.
pub(crate) struct Context<'a> {
    pub package: &'a hir::Package,
    pub packages: &'a [&'a hir::Package],
    pub profile: Profile,
    manifest: String,
}

impl<'a> Context<'a> {
    pub fn new(
        package: &'a hir::Package,
        packages: &'a [&'a hir::Package],
        profile: Profile,
    ) -> Self {
        Self {
            manifest: manifest_file(package),
            package,
            packages,
            profile,
        }
    }

    /// Every declaration of the package, every version, in module order.
    pub fn declarations(&self) -> impl Iterator<Item = &'a Declaration> {
        declarations(self.package)
    }

    /// Every node declaration (every version).
    pub fn nodes(&self) -> impl Iterator<Item = &'a hir::Node> {
        self.declarations().filter_map(|d| match d {
            Declaration::Node(n) => Some(n),
            _ => None,
        })
    }

    /// `span`, else the package's `Lit.toml`.
    pub fn span_or_manifest(&self, span: Option<&Span>) -> Span {
        span.cloned().unwrap_or_else(|| start_of(&self.manifest))
    }

    /// The declaring span of node `id`.
    pub fn node_span(&self, id: &str) -> Span {
        let span = self
            .nodes()
            .find(|n| n.meta.id == id)
            .and_then(|n| n.meta.span.as_ref());
        self.span_or_manifest(span)
    }

    /// `target` without this package's `<pack-id>::` qualifier.
    pub fn local<'t>(&self, target: &'t str) -> &'t str {
        self.owner(target).1
    }

    /// The package declaring `target` (`<pack-id>::<id>` or a local id)
    /// and its local id there.
    pub fn owner<'t>(&self, target: &'t str) -> (&'a hir::Package, &'t str) {
        target
            .split_once("::")
            .and_then(|(pack, id)| {
                self.packages
                    .iter()
                    .copied()
                    .chain(std::iter::once(self.package))
                    .find(|p| p.manifest.pack_id == pack)
                    .map(|p| (p, id))
            })
            .unwrap_or((self.package, target))
    }

    /// Every version of deadline `target`.
    pub fn deadline_versions(&self, target: &str) -> Vec<&'a hir::Deadline> {
        let (pkg, id) = self.owner(target);
        declarations(pkg)
            .filter_map(|d| match d {
                Declaration::Deadline(x) if x.meta.id == id => Some(x),
                _ => None,
            })
            .collect()
    }

    /// Whether deadline `target` is declared inside a `rule duty`.
    pub fn is_duty(&self, target: &str) -> bool {
        let (pkg, id) = self.owner(target);
        declarations(pkg).any(|d| {
            matches!(d, Declaration::Rule(r)
                if r.rule_kind == RuleKind::Duty && r.declarations.iter().any(|x| x == id))
        })
    }

    /// Every cite of the package with its declaring span and whether its
    /// declaration is `/// imported`: declaration `cite`s, edge
    /// `authority`s (imported with their node), and test `cite`s.
    pub fn cites(&self) -> impl Iterator<Item = Cite<'a>> {
        self.declarations().flat_map(|d| {
            let meta = meta_of(d);
            let imported = meta.is_some_and(|m| m.imported);
            let own = meta.and_then(|m| Some((m.cite.as_deref()?, m.span.as_ref())));
            let test = match d {
                Declaration::Test(t) => t.cite.as_deref().map(|c| (c, t.span.as_ref())),
                _ => None,
            };
            let edges: Vec<(&str, Option<&Span>)> = match d {
                Declaration::Node(n) => n
                    .edges
                    .iter()
                    .filter_map(|e| Some((e.authority.as_deref()?, e.span.as_ref())))
                    .collect(),
                _ => Vec::new(),
            };
            own.into_iter()
                .chain(test)
                .chain(edges)
                .map(move |(text, span)| Cite {
                    text,
                    span,
                    imported,
                })
        })
    }
}

/// One cite of a package.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Cite<'a> {
    /// The cite text.
    pub text: &'a str,
    /// Where it is declared.
    pub span: Option<&'a Span>,
    /// Its declaration carries the `/// imported` marker.
    pub imported: bool,
}

fn declarations(pkg: &hir::Package) -> impl Iterator<Item = &Declaration> {
    pkg.modules.iter().flat_map(|m| m.declarations.iter())
}

/// The package's `Lit.toml`, beside its first module file.
fn manifest_file(pkg: &hir::Package) -> String {
    declarations(pkg)
        .find_map(|d| meta_of(d)?.span.as_ref())
        .and_then(|s| {
            std::path::Path::new(&s.file.0)
                .parent()
                .map(|p| p.join("Lit.toml").to_string_lossy().replace('\\', "/"))
        })
        .unwrap_or_else(|| "Lit.toml".to_owned())
}

/// Structural equality of two expressions, ignoring source spans.
pub(crate) fn same_expr(a: &hir::Expr, b: &hir::Expr) -> bool {
    let strip = |e: &hir::Expr| {
        let mut v = serde_json::to_value(e).unwrap_or_default();
        strip_spans(&mut v);
        v
    };
    strip(a) == strip(b)
}

fn strip_spans(v: &mut serde_json::Value) {
    match v {
        serde_json::Value::Object(map) => {
            map.remove("span");
            map.values_mut().for_each(strip_spans);
        }
        serde_json::Value::Array(items) => items.iter_mut().for_each(strip_spans),
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hir::{Expr, ExprKind, Type, Value};

    fn at(line: u32) -> Span {
        Span {
            line,
            ..start_of("pkgs/demo/main.lit")
        }
    }

    fn served(document: &str, line: u32) -> Expr {
        let arg = Expr {
            ty: Type::Named("Document".into()),
            kind: ExprKind::Literal {
                value: Value::Variant(document.into()),
            },
            span: Some(at(line)),
        };
        Expr {
            ty: Type::Unit,
            kind: ExprKind::Call {
                callee: "Served".into(),
                args: vec![arg],
            },
            span: Some(at(line)),
        }
    }

    #[test]
    fn expressions_compare_without_their_spans() {
        let here = served("Document::complaint", 3);
        assert!(same_expr(&here, &served("Document::complaint", 9)));
        assert!(!same_expr(&here, &served("Document::reply", 3)));
    }
}
