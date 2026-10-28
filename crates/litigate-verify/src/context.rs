//! The package under verification and lookups over its HIR.
use crate::diag::start_of;
use litigate_base::{Profile, Span};
use litigate_sema::hir::{self, Declaration, RuleKind};
use litigate_sema::temporal::world::meta_of;
use std::borrow::Cow;

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
        owner(self.package, self.packages, target)
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
    /// with their node, in its window), a transition's cites (in the window
    /// of the version it amends), test `cite`s (no window), and the entries
    /// of a filing limit with the words each must find ([`limit_cites`]).
    pub fn cites(&self) -> impl Iterator<Item = Cite<'a>> {
        let (package, packages) = (self.package, self.packages);
        self.declarations().flat_map(move |d| {
            let meta = meta_of(d).or(match d {
                Declaration::Grant(g) => Some(&g.meta),
                Declaration::Recharacterize(r) => Some(&r.meta),
                Declaration::Invalidate(i) => Some(&i.meta),
                _ => None,
            });
            let imported = meta.is_some_and(|m| m.imported);
            // A transition has no version of its own: it speaks from the
            // version it amends, not from its package's `in_force`.
            let amended = match d {
                Declaration::Transition(t) => amended(package, packages, t),
                _ => None,
            };
            let window = amended.or(meta.map(|m| &m.effective));
            let own = meta.and_then(|m| {
                Some((
                    m.cite.as_deref()?,
                    m.span.as_ref(),
                    m.historical,
                    Vec::new(),
                ))
            });
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
                .filter_map(move |g| Some((g.cite.as_deref()?, span, historical, Vec::new())));
            let characterized = match d {
                Declaration::Deadline(x) => x
                    .characterization
                    .as_ref()
                    .and_then(|c| Some((c.cite.as_deref()?, span, historical, Vec::new()))),
                _ => None,
            };
            let test = match d {
                Declaration::Test(t) => t
                    .cite
                    .as_deref()
                    .map(|c| (c, t.span.as_ref(), false, Vec::new())),
                _ => None,
            };
            let adoption = match d {
                Declaration::Deadline(x) => x.adoption.as_ref().map(|a| {
                    (
                        a.cite.as_str(),
                        x.meta.span.as_ref(),
                        a.historical,
                        a.quote.as_deref().map(Cow::Borrowed).into_iter().collect(),
                    )
                }),
                _ => None,
            };
            let edges = edge_cites(d);
            let limited = match d {
                Declaration::Limit(l) => limit_cites(l),
                _ => Vec::new(),
            };
            own.into_iter()
                .chain(test)
                .chain(limited)
                .chain(adoption)
                .chain(edges)
                .chain(delegated)
                .chain(characterized)
                .map(move |(text, span, historical, quotes)| Cite {
                    text,
                    span,
                    window,
                    historical,
                    imported,
                    quotes,
                })
        })
    }
}

/// The `authority` and `basis sourced(..)` cites on a node and its edges.
fn edge_cites(d: &Declaration) -> Vec<CiteOf<'_>> {
    match d {
        Declaration::Node(n) => n
            .edges
            .iter()
            .flat_map(|e| {
                [e.authority.as_deref(), sourced(e.basis.as_ref())]
                    .into_iter()
                    .flatten()
                    .map(|c| (c, e.span.as_ref(), false, Vec::new()))
            })
            .chain(sourced(n.basis.as_ref()).map(|c| (c, n.meta.span.as_ref(), false, Vec::new())))
            .collect(),
        _ => Vec::new(),
    }
}

/// A cite's text, span, `historical` mark, and quoted words (any one of
/// which its vendored text must hold).
type CiteOf<'a> = (&'a str, Option<&'a Span>, bool, Vec<Cow<'a, str>>);

/// The cites of a filing limit's entries (SPEC §3.7), each its own or
/// else the declaration's, with the words its vendored text must hold: a
/// bound's number and measure as the rules write them, in digits
/// (`13,000 words`), in words (`thirty pages`) or both (`thirty (30)
/// pages`), each also with one word between the count and the measure
/// (D.S.C. Local Civ. Rule 7.05(B): `Thirty-five (35) double-spaced
/// pages`, [`GAP`]), and each excluded part named, hyphens read as spaces
/// (`cover page`).
/// A hole cites nothing and quotes nothing.
fn limit_cites(l: &hir::Limit) -> Vec<CiteOf<'_>> {
    let decl = l.meta.cite.as_deref();
    let span = l.meta.span.as_ref();
    let mut out = Vec::new();
    for b in &l.bounds {
        let Some(text) = b.cite.as_deref().or(decl) else {
            continue;
        };
        let quotes = match &b.max {
            hir::Authored::Known(n) => {
                let (digits, words, measure) = (grouped(*n), spelled(*n), b.measure.as_str());
                [digits.clone(), words.clone(), format!("{words} ({digits})")]
                    .into_iter()
                    .flat_map(|count| {
                        [
                            format!("{count} {measure}"),
                            format!("{count} {GAP} {measure}"),
                        ]
                    })
                    .map(Cow::Owned)
                    .collect()
            }
            hir::Authored::Hole(_) => Vec::new(),
        };
        out.push((text, b.span.as_ref().or(span), false, quotes));
    }
    for c in &l.certificates {
        if let Some(text) = c.cite.as_deref() {
            out.push((text, c.span.as_ref().or(span), false, Vec::new()));
        }
    }
    if let Some(x) = &l.excludes {
        if let Some(text) = x.cite.as_deref().or(decl) {
            let at = x.span.as_ref().or(span);
            out.push((text, at, false, Vec::new()));
            out.extend(x.items.iter().filter_map(|item| match item {
                hir::Authored::Known(part) => {
                    Some((text, at, false, vec![Cow::Owned(part.replace('-', " "))]))
                }
                hir::Authored::Hole(_) => None,
            }));
        }
    }
    for e in &l.ecf {
        if let Some(text) = e.cite.as_deref() {
            out.push((text, e.span.as_ref().or(span), false, Vec::new()));
        }
    }
    out
}

/// In a quote, any one word of the cited text: a limit's count and
/// measure with a word between them (`35 double-spaced pages`).
pub(crate) const GAP: char = '\u{2026}';

/// `n` with its thousands grouped by commas, as the rules write counts.
fn grouped(n: u64) -> String {
    let digits = n.to_string();
    let mut out = String::new();
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(c);
    }
    out
}

/// `n` in words, as the rules spell counts: `thirty`, `twenty-five`,
/// `one hundred fifty`, `thirteen thousand`.
fn spelled(n: u64) -> String {
    const ONES: [&str; 20] = [
        "zero",
        "one",
        "two",
        "three",
        "four",
        "five",
        "six",
        "seven",
        "eight",
        "nine",
        "ten",
        "eleven",
        "twelve",
        "thirteen",
        "fourteen",
        "fifteen",
        "sixteen",
        "seventeen",
        "eighteen",
        "nineteen",
    ];
    const TENS: [&str; 10] = [
        "", "", "twenty", "thirty", "forty", "fifty", "sixty", "seventy", "eighty", "ninety",
    ];
    const SCALES: [&str; 7] = [
        "",
        "thousand",
        "million",
        "billion",
        "trillion",
        "quadrillion",
        "quintillion",
    ];
    // Under a thousand, `n` is never zero here.
    let hundreds = |n: u64| {
        let mut words = Vec::new();
        if n >= 100 {
            words.push(format!("{} hundred", ONES[(n / 100) as usize]));
        }
        let rest = (n % 100) as usize;
        match rest {
            0 => {}
            1..20 => words.push(ONES[rest].to_owned()),
            _ if rest.is_multiple_of(10) => words.push(TENS[rest / 10].to_owned()),
            _ => words.push(format!("{}-{}", TENS[rest / 10], ONES[rest % 10])),
        }
        words.join(" ")
    };
    if n == 0 {
        return ONES[0].to_owned();
    }
    let mut groups = Vec::new();
    let mut rest = n;
    for scale in SCALES {
        let group = rest % 1000;
        if group > 0 {
            groups.push(match scale {
                "" => hundreds(group),
                _ => format!("{} {scale}", hundreds(group)),
            });
        }
        rest /= 1000;
    }
    groups.reverse();
    groups.join(" ")
}

/// The cite a `basis sourced("...")` rests on.
fn sourced(basis: Option<&hir::Basis>) -> Option<&str> {
    match basis? {
        hir::Basis::Sourced { cite } => Some(cite),
        _ => None,
    }
}

/// One cite of a package.
#[derive(Debug, Clone)]
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
    /// The words of the cited text it quotes (`quote "..."`), any one of
    /// which its vendored section must hold; none when it quotes nothing.
    pub quotes: Vec<Cow<'a, str>>,
}

/// The package declaring `target` (`<pack-id>::<id>` or a local id) among
/// `package` and `packages`, and its local id there.
fn owner<'a, 't>(
    package: &'a hir::Package,
    packages: &[&'a hir::Package],
    target: &'t str,
) -> (&'a hir::Package, &'t str) {
    target
        .split_once("::")
        .and_then(|(pack, id)| {
            packages
                .iter()
                .copied()
                .chain(std::iter::once(package))
                .find(|p| p.manifest.pack_id == pack)
                .map(|p| (p, id))
        })
        .unwrap_or((package, target))
}

/// The in-force window of the version transition `t` amends (`id@from`).
fn amended<'a>(
    package: &'a hir::Package,
    packages: &[&'a hir::Package],
    t: &hir::Transition,
) -> Option<&'a hir::Interval> {
    let (target, from) = t.amendment.rsplit_once('@')?;
    let (pkg, id) = owner(package, packages, target);
    declarations(pkg)
        .filter_map(meta_of)
        .find(|m| m.id == id && m.effective.from == from)
        .map(|m| &m.effective)
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

    #[test]
    fn counts_are_spelled_as_the_rules_write_them() {
        for (n, words) in [
            (0, "zero"),
            (5, "five"),
            (15, "fifteen"),
            (30, "thirty"),
            (25, "twenty-five"),
            (100, "one hundred"),
            (150, "one hundred fifty"),
            (1_300, "one thousand three hundred"),
            (13_000, "thirteen thousand"),
            (2_000_017, "two million seventeen"),
        ] {
            assert_eq!(spelled(n), words, "{n}");
        }
    }
}
