//! Hover: what a name is (kind, qualified id, type), every effective
//! version of it, and the L0 text its `cite` resolves to.
use crate::analysis::Analysis;
use crate::cite::{self, Located};
use crate::index::{Def, Space};
use litgraph::cite::SourceCorpus;
use litigate_cli::hirx;
use litigate_sema::hir::{self, DayUnit, Declaration, NodeKind, Type};
use litigate_sema::temporal::world::meta_of;
use litigate_syntax::{parse, SyntaxKind as K, SyntaxToken, TextRange, TextSize};
use std::fmt::Write as _;

/// Most L0 lines shown inline.
const EXCERPT: usize = 12;

/// A type as written in source.
#[must_use]
pub fn show_type(t: &Type) -> String {
    match t {
        Type::Date => "Date".into(),
        Type::Days(u) => format!("Days<{}>", unit(*u)),
        Type::Hours => "Hours".into(),
        Type::Money(c) => format!("Money<{}>", c.to_uppercase()),
        Type::Probability => "Probability".into(),
        Type::Distribution(e) => format!("Dist<{e}>"),
        Type::Cite => "Cite".into(),
        Type::DateInterval => "Interval<Date>".into(),
        Type::Role => "Role".into(),
        Type::Boolean => "Bool".into(),
        Type::Number => "Number".into(),
        Type::String => "String".into(),
        Type::Edge => "EdgeRef".into(),
        Type::List(t) => format!("[{}]", show_type(t)),
        Type::Record(fields) => {
            let f: Vec<String> = fields
                .iter()
                .map(|(k, t)| format!("{k}: {}", show_type(t)))
                .collect();
            format!("{{ {} }}", f.join(", "))
        }
        Type::Named(n) => hirx::local(n).to_owned(),
        Type::Unit => "Unit".into(),
    }
}

fn unit(u: DayUnit) -> &'static str {
    match u {
        DayUnit::Calendar => "CalendarDays",
        DayUnit::Court => "CourtDays",
        DayUnit::Business => "BusinessDays",
        DayUnit::Week => "Weeks",
        DayUnit::Month => "Months",
        DayUnit::Year => "Years",
    }
}

fn interval(e: &hir::Interval) -> String {
    format!(
        "[{}, {}) known_at {}",
        e.from,
        e.to.as_deref().unwrap_or(".."),
        e.known_at
    )
}

/// The type line of a declaration.
fn typed(d: &Declaration) -> Option<String> {
    Some(match d {
        Declaration::Deadline(x) if x.designated.is_some() => format!(
            "`Date`: designated by {} (its `{}`), no earlier than {} `Days<{}>` after it, counted_by `{}`",
            hirx::event_text(&x.trigger),
            x.designated.as_ref().map_or("", |p| p.name.as_str()),
            x.amount,
            unit(x.unit),
            hirx::local(&x.counting)
        ),
        Declaration::Deadline(x) => format!(
            "`Days<{}>`: {} {} {} counted_by `{}`",
            unit(x.unit),
            x.amount,
            match x.direction {
                hir::Direction::Forward => "after",
                hir::Direction::Backward => "before",
            },
            hirx::event_text(&x.trigger),
            hirx::local(&x.counting)
        ),
        Declaration::Event(e) => {
            let params: Vec<String> = e
                .params
                .iter()
                .map(|(n, t)| format!("{n}: {}", show_type(t)))
                .collect();
            format!("`{}({})`", e.name, params.join(", "))
        }
        Declaration::Enum(e) => format!("`enum {}` {{ {} }}", e.name, e.variants.join(", ")),
        Declaration::TypeAlias(t) => format!("`{}` = `{}`", t.name, show_type(&t.target)),
        Declaration::Node(n) => {
            let by = n
                .actor
                .as_deref()
                .map(|a| format!(" by `{a}`"))
                .unwrap_or_default();
            let kind = match &n.kind {
                NodeKind::Chance { discretion: true } => "chance (discretion)".to_owned(),
                NodeKind::Terminal { outcomes, .. } => {
                    format!("terminal outcome [{}]", outcomes.join(", "))
                }
                _ => hirx::decl_kind(d).to_owned(),
            };
            let edges: Vec<String> = n
                .edges
                .iter()
                .map(|e| {
                    format!(
                        "`{}` -> `{}`",
                        e.id.as_deref().unwrap_or("_"),
                        hirx::local(&e.to)
                    )
                })
                .collect();
            format!("{kind}{by}; edges {}", edges.join(", "))
        }
        Declaration::Authority(a) => {
            format!("authority `{}` outranks {}", a.name, a.outranks.join(", "))
        }
        _ => return None,
    })
}

/// The L0 block for located sections.
fn l0(out: &mut String, located: &[Located]) {
    for s in located {
        let pin: String = s.pinpoint.iter().fold(String::new(), |mut pin, m| {
            let _ = write!(pin, "({m})");
            pin
        });
        let at = match (&s.path, s.lines) {
            (Some(p), Some((a, b))) if a == b => format!("`{p}:{a}`"),
            (Some(p), Some((a, b))) => format!("`{p}:{a}-{b}`"),
            (Some(p), None) => format!("`{p}`"),
            _ => format!(
                "not in the vendored corpus{}",
                s.source.as_deref().map_or_else(String::new, |id| format!(
                    " (bind `path` and `section_heading` in `[sources.{id}]` to locate it)"
                ))
            ),
        };
        let _ = writeln!(out, "\n**L0** {}{pin} — {at}", s.heading);
        if let Some(text) = &s.text {
            let lines: Vec<&str> = text.lines().take(EXCERPT).map(str::trim).collect();
            let more = if text.lines().count() > EXCERPT {
                "\n…"
            } else {
                ""
            };
            let _ = writeln!(out, "\n```text\n{}{more}\n```", lines.join("\n"));
        }
    }
}

/// The cite section of a hover, for `cite` written on one-based `line`.
fn cited(out: &mut String, analysis: &Analysis, pkg: &hir::Package, cite: &str, line: u32) {
    let verdict = analysis
        .checked
        .verdicts
        .iter()
        .find(|(n, _)| *n == pkg.manifest.name)
        .map(|(_, v)| v);
    let located = cite::locate(pkg, verdict, cite, line, SourceCorpus::embedded_ref());
    let _ = writeln!(out, "\n**cite** {cite}");
    l0(out, &located);
}

/// The one-based line of byte `offset` in `text`.
fn line_of(text: &str, offset: usize) -> u32 {
    crate::convert::line_col(text, offset).0
}

/// The declaration header (its first code line) as written, and its doc
/// comment.
fn header(text: &str, def: &Def) -> (String, String) {
    let range = std::ops::Range::<usize>::from(def.full_range);
    let src = text.get(range).unwrap_or_default();
    let docs: Vec<&str> = src
        .lines()
        .map_while(|l| l.trim_start().strip_prefix("///"))
        .map(str::trim)
        .collect();
    let code = src
        .lines()
        .find(|l| !l.trim_start().starts_with("//"))
        .unwrap_or_default()
        .trim_end_matches('{')
        .trim();
    (code.to_owned(), docs.join("\n"))
}

/// The kind, type, versions, and cited L0 text of declaration `name`.
fn declaration(out: &mut String, analysis: &Analysis, pkg: &hir::Package, name: &str) {
    let qualified = format!("{}::{name}", pkg.manifest.pack_id);
    let versions: Vec<&Declaration> = hirx::declarations(pkg)
        .filter(|d| hirx::decl_id(d).as_deref() == Some(name))
        .collect();
    if let Some(first) = versions.first() {
        let _ = writeln!(out, "\n{} `{qualified}`", hirx::decl_kind(first));
        if let Some(t) = typed(first) {
            let _ = writeln!(out, "\n{t}");
        }
    } else if let Some(g) = pkg.groups.iter().find(|g| g.id == name) {
        let _ = writeln!(out, "\ngroup `{qualified}`: {}", g.label);
    }
    let metas: Vec<&hir::Metadata> = versions.iter().filter_map(|d| meta_of(d)).collect();
    if !metas.is_empty() {
        let _ = writeln!(out, "\n**versions**");
        for m in &metas {
            let at = m
                .span
                .as_ref()
                .map(|s| format!(" — `{}:{}`", s.file.0, s.line))
                .unwrap_or_default();
            let _ = writeln!(out, "- {} · {}{at}", interval(&m.effective), m.authority);
        }
    }
    // Sema spans every declaration it builds from source.
    if let Some((cite, line)) = metas.iter().find_map(|m| {
        let line = m.span.as_ref().map_or(0, |s| s.line);
        m.cite.clone().map(|cite| (cite, line))
    }) {
        cited(out, analysis, pkg, &cite, line);
    }
}

/// Hover markdown for definition `def`.
#[must_use]
pub fn definition(analysis: &Analysis, def: &Def) -> String {
    let text = analysis.text(&def.file).unwrap_or_default();
    let (code, docs) = header(&text, def);
    let mut out = format!("```litigate\n{code}\n```\n");
    if !docs.is_empty() {
        let _ = writeln!(out, "\n{docs}");
    }
    let pkg = def.package.as_deref().and_then(|p| analysis.hir(p));
    let Some(pkg) = pkg else {
        if def.space == Space::Party {
            let _ = writeln!(out, "\nparty of this matter");
        }
        return out;
    };
    match &def.space {
        Space::Decl => declaration(&mut out, analysis, pkg, &def.name),
        Space::Edge(owner) => {
            let edge = hirx::nodes(pkg)
                .into_iter()
                .filter(|n| n.meta.id == *owner)
                .flat_map(|n| n.edges.iter())
                .find(|e| e.id.as_deref() == Some(def.name.as_str()));
            let _ = writeln!(
                out,
                "\nedge `{}::{owner}.{}`",
                pkg.manifest.pack_id, def.name
            );
            if let Some(e) = edge {
                let _ = writeln!(out, "\n-> `{}` by `{}`", hirx::local(&e.to), e.actor);
                if let Some(cite) = &e.authority {
                    let line = e.span.as_ref().map_or(0, |s| s.line);
                    cited(&mut out, analysis, pkg, cite, line);
                }
            }
        }
        Space::Variant(e) => {
            let _ = writeln!(out, "\nvariant of enum `{}::{e}`", pkg.manifest.pack_id);
        }
        Space::Role => {
            let role = pkg
                .manifest
                .roles
                .get(&def.name)
                .map(|r| format!("{r:?}").to_lowercase());
            let _ = writeln!(out, "\nactor `{}` ({})", def.name, role.unwrap_or_default());
        }
        Space::Module => {
            let _ = writeln!(out, "\nmodule of package `{}`", pkg.manifest.name);
        }
        Space::Party | Space::Local => {}
    }
    out
}

/// Hover markdown for the `cite` token `token` of `file` (holding `text`).
#[must_use]
pub fn cite_token(
    analysis: &Analysis,
    file: &str,
    text: &str,
    token: &SyntaxToken,
) -> Option<String> {
    let pkg = analysis.package_of(file)?.check.package.as_ref()?;
    let line = line_of(text, usize::from(token.text_range().start()));
    let mut out = String::new();
    let cite = litigate_syntax::ast::unescape(token.text());
    cited(&mut out, analysis, pkg, &cite, line);
    Some(out.trim_start().to_owned())
}

/// Hover markdown at `offset` of `file` (holding `text`), with the range
/// it describes: a cite's L0 text, else the definition named there (in a
/// `Lit.toml`, a `[package.roles]` key).
#[must_use]
pub fn at(
    analysis: &Analysis,
    file: &str,
    text: &str,
    offset: TextSize,
) -> Option<(String, TextRange)> {
    let Some(kind) = Analysis::kind(file) else {
        let def = &analysis.index.defs[analysis.index.at(file, offset)?];
        return Some((definition(analysis, def), def.name_range));
    };
    let token = token_at(text, kind, offset)?;
    let value = if is_cite(&token) {
        cite_token(analysis, file, text, &token)?
    } else {
        let def = analysis.index.at(file, token.text_range().start())?;
        definition(analysis, &analysis.index.defs[def])
    };
    Some((value, token.text_range()))
}

/// The token at `offset` of `text` (at most its length): the identifier or
/// cite touching it, preferring the right side.
#[must_use]
pub fn token_at(
    text: &str,
    kind: litigate_syntax::FileKind,
    offset: TextSize,
) -> Option<SyntaxToken> {
    let root = parse(text, kind).syntax();
    let at = root.token_at_offset(offset);
    let interesting = |t: &SyntaxToken| t.kind() == K::Ident || is_cite(t);
    let (left, right) = (at.clone().left_biased(), at.right_biased());
    right
        .filter(interesting)
        .or_else(|| left.filter(interesting))
}

/// The citation string of a `cite` clause.
fn is_cite(token: &SyntaxToken) -> bool {
    token.kind() == K::String && token.parent().is_some_and(|p| p.kind() == K::CiteExpr)
}

#[cfg(test)]
mod tests;
