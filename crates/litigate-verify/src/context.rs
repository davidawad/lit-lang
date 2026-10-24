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

    /// Every cite of the package with its declaring span, the in-force
    /// window of its declaration version, and whether that declaration is
    /// `/// imported`: declaration `cite`s (and whether each is marked
    /// `historical`), a deadline's adopting rule (`forum F cite "X" quote
    /// "..."`, in its window, with the words it quotes), edge `authority`s
    /// and the cites of `basis sourced(..)` on nodes and edges (imported
    /// with their node, in its window), and test `cite`s (no window).
    pub fn cites(&self) -> impl Iterator<Item = Cite<'a>> {
        self.declarations().flat_map(|d| {
            let meta = meta_of(d).or(match d {
                Declaration::Grant(g) => Some(&g.meta),
                _ => None,
            });
            let imported = meta.is_some_and(|m| m.imported);
            let window = meta.map(|m| &m.effective);
            let own =
                meta.and_then(|m| Some((m.cite.as_deref()?, m.span.as_ref(), m.historical, None)));
            // A delegation's and a characterization's own authority, at
            // the declaration.
            let span = meta.and_then(|m| m.span.as_ref());
            let historical = meta.is_some_and(|m| m.historical);
            let delegated = meta
                .into_iter()
                .flat_map(|m| &m.delegations)
                .chain(match d {
                    Declaration::Grant(g) => g.delegations.as_slice(),
                    _ => &[],
                })
                .filter_map(move |g| Some((g.cite.as_deref()?, span, historical, None)));
            let characterized = match d {
                Declaration::Deadline(x) => x
                    .characterization
                    .as_ref()
                    .and_then(|c| Some((c.cite.as_deref()?, span, historical, None))),
                _ => None,
            };
            let test = match d {
                Declaration::Test(t) => {
                    t.cite.as_deref().map(|c| (c, t.span.as_ref(), false, None))
                }
                _ => None,
            };
            let adoption = match d {
                Declaration::Deadline(x) => x.adoption.as_ref().map(|a| {
                    (
                        a.cite.as_str(),
                        x.meta.span.as_ref(),
                        a.historical,
                        a.quote.as_deref(),
                    )
                }),
                _ => None,
            };
            let edges: Vec<CiteOf<'_>> = match d {
                Declaration::Node(n) => n
                    .edges
                    .iter()
                    .flat_map(|e| {
                        [e.authority.as_deref(), sourced(e.basis.as_ref())]
                            .into_iter()
                            .flatten()
                            .map(|c| (c, e.span.as_ref(), false, None))
                    })
                    .chain(
                        sourced(n.basis.as_ref()).map(|c| (c, n.meta.span.as_ref(), false, None)),
                    )
                    .collect(),
                _ => Vec::new(),
            };
            own.into_iter()
                .chain(test)
                .chain(adoption)
                .chain(edges)
                .chain(delegated)
                .chain(characterized)
                .map(move |(text, span, historical, quote)| Cite {
                    text,
                    span,
                    window,
                    historical,
                    imported,
                    quote,
                })
        })
    }
}

/// A cite's text, span, `historical` mark, and quoted words.
type CiteOf<'a> = (&'a str, Option<&'a Span>, bool, Option<&'a str>);

/// The cite a `basis sourced("...")` rests on.
fn sourced(basis: Option<&hir::Basis>) -> Option<&str> {
    match basis? {
        hir::Basis::Sourced { cite } => Some(cite),
        _ => None,
    }
}

/// One cite of a package.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Cite<'a> {
    /// The cite text.
    pub text: &'a str,
    /// Where it is declared.
    pub span: Option<&'a Span>,
    /// In-force window of the citing declaration version, if it has one.
    pub window: Option<&'a hir::Interval>,
    /// The cite is marked `historical`.
    pub historical: bool,
    /// Its declaration carries the `/// imported` marker.
    pub imported: bool,
    /// The words of the cited text it quotes (`quote "..."`), which its
    /// vendored section must hold.
    pub quote: Option<&'a str>,
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
