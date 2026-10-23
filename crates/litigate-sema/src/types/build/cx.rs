//! Per-module building context: the expression checker plus field helpers
//! and declaration metadata (labels, cites, effective intervals, relations).
use crate::db::parse::tree::{Atom, Expr, ExprKind, Field, Item, Name, RefPath};
use crate::db::span::Report;
use crate::hir::{Interval, Manifest, Metadata, Relation};
use crate::resolve::sig::{clause, clauses_of, ValueKind};
use crate::types::expr::Checker;
use litigate_base::{Applicability, DiagnosticCode as C};
use std::collections::BTreeMap;

/// Building context for one module.
pub struct Cx<'s, 'a> {
    /// Expression checker (owns the reports).
    pub ck: Checker<'s, 'a>,
    /// Package manifest, when valid.
    pub manifest: Option<&'s Manifest>,
    /// `::` qualification allowed (links package).
    pub links: bool,
    /// Pack id of the link being built: its actors are that pack's.
    pub link_from: Option<String>,
}

/// The first value of a field as an expression.
#[must_use]
pub fn first_expr(f: &Field) -> Option<&Expr> {
    match f.values.first() {
        Some((Atom::Expr(e), _)) => Some(e),
        _ => None,
    }
}

/// The first value of an optional field as a reference (`None` when the
/// field is absent or its value is missing).
#[must_use]
pub fn first_ref<'f>(f: Option<&&'f Field>) -> Option<&'f RefPath> {
    match f.and_then(|f| first_expr(f)).map(|e| &e.kind) {
        Some(ExprKind::Ref(r)) => Some(r),
        _ => None,
    }
}

/// The first value of an optional field as a boolean (`extendable`).
#[must_use]
pub fn flag(f: Option<&&Field>) -> Option<bool> {
    match f.and_then(|f| first_expr(f)).map(|e| &e.kind) {
        Some(ExprKind::Bool(b)) => Some(*b),
        _ => None,
    }
}

/// Every hole written in `fields`, by dotted field path (`p`, `cost`,
/// `duration.min`, `attrs.weight`), with its `?("reason")`: a hole as a
/// field's value, as a record entry, or in a nested field list. Lowering
/// marks each in the generated document (SPEC §8.1 holes).
#[must_use]
pub fn holes_of<'f>(
    fields: impl IntoIterator<Item = &'f Field>,
) -> BTreeMap<String, Option<String>> {
    let mut out = BTreeMap::new();
    for f in fields {
        field_holes(f, &f.key.text, &mut out);
    }
    out
}

fn field_holes(f: &Field, path: &str, out: &mut BTreeMap<String, Option<String>>) {
    for (atom, _) in &f.values {
        match atom {
            Atom::Expr(e) => expr_holes(e, path, out),
            Atom::Fields(nested) => {
                for n in nested {
                    field_holes(n, &format!("{path}.{}", n.key.text), out);
                }
            }
            Atom::Interval(..) => {}
        }
    }
}

fn expr_holes(e: &Expr, path: &str, out: &mut BTreeMap<String, Option<String>>) {
    match &e.kind {
        ExprKind::Hole(reason) => {
            out.insert(path.to_owned(), reason.clone());
        }
        ExprKind::Paren(inner) => expr_holes(inner, path, out),
        ExprKind::Record(entries) => {
            for (name, value) in entries {
                expr_holes(value, &format!("{path}.{}", name.text), out);
            }
        }
        _ => {}
    }
}

/// Nested field list of a field (deadline options, triangular duration).
#[must_use]
pub fn nested(f: &Field) -> &[Field] {
    f.values
        .iter()
        .find_map(|(a, _)| match a {
            Atom::Fields(fs) => Some(fs.as_slice()),
            _ => None,
        })
        .unwrap_or(&[])
}

impl Cx<'_, '_> {
    /// Records a report (see [`crate::types::expr::Checker::report`]).
    pub fn report(&mut self, r: impl Into<Option<Report>>) {
        self.ck.report(r);
    }

    /// Indexes fields by key, reporting duplicates (L2007).
    pub fn once<'f>(&mut self, fields: &'f [Field]) -> BTreeMap<&'f str, &'f Field> {
        self.once_except(fields, "")
    }

    /// Like [`Self::once`], but `repeatable` may appear any number of times
    /// (and is left out of the index).
    pub fn once_except<'f>(
        &mut self,
        fields: &'f [Field],
        repeatable: &str,
    ) -> BTreeMap<&'f str, &'f Field> {
        let mut out = BTreeMap::new();
        for f in fields.iter().filter(|f| f.key.text != repeatable) {
            if out.insert(f.key.text.as_str(), f).is_some() {
                self.report(
                    Report::new(
                        C::InvalidField,
                        f.key.range,
                        format!("`{}` is given more than once", f.key.text),
                    )
                    .fix(f.range, "", Applicability::MaybeIncorrect),
                );
            }
        }
        out
    }

    /// Rejects a field that this declaration kind does not allow.
    pub fn not_allowed(&mut self, f: &Field, why: &str) {
        self.report(
            Report::new(
                C::InvalidField,
                f.key.range,
                format!("`{}` is not allowed {why}", f.key.text),
            )
            .fix(f.range, "", Applicability::MaybeIncorrect),
        );
    }

    /// `e` as a call `f(args)`; anything else is reported as `code`.
    pub fn call<'e>(
        &mut self,
        e: &'e Expr,
        code: C,
        message: &str,
    ) -> Option<(&'e RefPath, &'e [Expr])> {
        if let ExprKind::Call(callee, args) = &e.kind {
            return Some((callee, args));
        }
        self.report(Report::new(code, e.range, message));
        None
    }

    /// String value of a field.
    #[must_use]
    pub fn string(f: Option<&&Field>) -> Option<String> {
        match f.and_then(|f| first_expr(f)).map(|e| &e.kind) {
            Some(ExprKind::Str(s) | ExprKind::Cite(s, _)) => Some(s.clone()),
            _ => None,
        }
    }

    /// Text of a `cite` clause or `authority` field. A cite hole
    /// `cite ?("text")` gives its reason, so imported citations stay
    /// lossless (SPEC §8.1), and is recorded as a hole.
    pub fn cite_text(&mut self, f: Option<&&Field>) -> Option<String> {
        match f.and_then(|f| first_expr(f)) {
            Some(Expr {
                kind: ExprKind::Hole(reason),
                range,
            }) => {
                self.ck.holes.push((*range, reason.clone()));
                reason.clone()
            }
            _ => Self::string(f),
        }
    }

    /// Simple identifiers of an `IdList` field (opaque tags and flags).
    pub fn idents(&mut self, f: Option<&&Field>) -> Vec<String> {
        Self::refs(f)
            .into_iter()
            .filter_map(|r| {
                if r.is_simple() {
                    return Some(r.head.text.clone());
                }
                self.report(Report::new(
                    C::InvalidField,
                    r.range,
                    format!("expected a plain identifier, found `{}`", r.display()),
                ));
                None
            })
            .collect()
    }

    /// Refs of an `IdList` field.
    #[must_use]
    pub fn refs<'f>(f: Option<&&'f Field>) -> Vec<&'f RefPath> {
        match f.and_then(|f| first_expr(f)).map(|e| &e.kind) {
            Some(ExprKind::List(items)) => items
                .iter()
                .filter_map(|i| match &i.kind {
                    ExprKind::Ref(r) => Some(r),
                    _ => None,
                })
                .collect(),
            _ => Vec::new(),
        }
    }

    /// Checks that `actor` is declared in `[package.roles]` (or `either`);
    /// a link's actor is declared by the pack its `from` node belongs to.
    pub fn actor(&mut self, r: &RefPath) -> String {
        let link_actors = self.link_from.as_ref().map(|pack| {
            self.ck
                .scope
                .packs
                .iter()
                .find(|s| s.pack_id == *pack)
                .map(|s| s.actors.iter().map(String::as_str).collect::<Vec<_>>())
        });
        let declared: Option<Vec<&str>> = match link_actors {
            Some(actors) => actors,
            None => self
                .manifest
                .map(|m| m.roles.keys().map(String::as_str).collect()),
        };
        let known = r.is_simple()
            && (r.head.text == "either"
                || declared
                    .as_ref()
                    .is_none_or(|d| d.contains(&r.head.text.as_str())));
        if !known {
            let roles = declared.unwrap_or_default();
            let owner = self
                .link_from
                .as_ref()
                .map_or_else(String::new, |pack| format!(" of pack `{pack}`"));
            let mut report = Report::new(
                C::UndeclaredActor,
                r.range,
                format!(
                    "actor `{}` is not declared in `[package.roles]`{owner}",
                    r.display()
                ),
            )
            .note(format!(
                "declared actors: {}",
                if roles.is_empty() {
                    "(none)".to_owned()
                } else {
                    roles.join(", ")
                }
            ));
            if let Some(best) =
                crate::resolve::suggest::nearest(&r.head.text, roles.iter().copied())
            {
                report = report.fix(r.range, best, Applicability::MaybeIncorrect);
            }
            self.report(report);
        }
        r.display()
    }

    /// Resolves `r` to a value of the kind accepted by `want`, returning
    /// its canonical name.
    pub fn value_ref(
        &mut self,
        r: &RefPath,
        what: &str,
        want: fn(&ValueKind) -> bool,
    ) -> Option<String> {
        if !self.links
            && r.rest
                .iter()
                .any(|(s, _)| *s == crate::db::parse::tree::PathSep::Colons)
        {
            self.report(Report::new(C::UnresolvedName, r.range, format!("`{}`: `::` qualification is only valid in links packages, cases, and instance fields", r.display())));
            return None;
        }
        match self.ck.scope.value(r) {
            Ok((name, found)) if want(&found.item.kind) => Some(name),
            Ok((_, found)) => {
                self.report(Report::new(
                    C::TypeMismatch,
                    r.range,
                    format!(
                        "`{}` is a {}, expected a {what}",
                        r.display(),
                        found.item.kind.noun()
                    ),
                ));
                None
            }
            Err(report) => {
                self.report(report);
                None
            }
        }
    }

    fn interval_of(item: &Item) -> (Option<(&Name, Option<&Name>)>, Option<String>) {
        let clauses = clauses_of(item);
        let eff = match clause(clauses, "effective").and_then(|f| f.values.first()) {
            Some((Atom::Interval(a, b), _)) => Some((a, b.as_ref())),
            _ => None,
        };
        let known = match clause(clauses, "known_at")
            .and_then(first_expr)
            .map(|e| &e.kind)
        {
            Some(ExprKind::Date(d)) => Some(d.clone()),
            _ => None,
        };
        (eff, known)
    }

    /// Effective interval: own clauses, else the rule's, else the package.
    pub fn effective(&mut self, item: &Item, rule: Option<&Item>) -> Interval {
        let (own, own_known) = Self::interval_of(item);
        let (rule_eff, rule_known) = rule.map(Self::interval_of).unwrap_or_default();
        let pkg = self.manifest.map(|m| m.in_force.clone());
        let (from, to) = match own.or(rule_eff) {
            Some((from, to)) => {
                if let Some(to) = to {
                    if to.text <= from.text {
                        self.report(Report::new(
                            C::InvalidField,
                            from.range.cover(to.range),
                            format!("empty interval [{}, {})", from.text, to.text),
                        ));
                    }
                }
                (from.text.clone(), to.map(|t| t.text.clone()))
            }
            None => (
                pkg.as_ref().map(|p| p.from.clone()).unwrap_or_default(),
                pkg.as_ref().and_then(|p| p.to.clone()),
            ),
        };
        let known_at = own_known
            .or(rule_known)
            .or_else(|| pkg.map(|p| p.known_at))
            .unwrap_or_default();
        Interval { from, to, known_at }
    }

    /// Declaration metadata.
    pub fn meta(
        &mut self,
        id: &Name,
        label: Option<&Name>,
        item: &Item,
        rule: Option<&Item>,
    ) -> Metadata {
        let clauses = clauses_of(item);
        let cites: Vec<&Field> = clauses.iter().filter(|f| f.key.text == "cite").collect();
        for extra in cites.iter().skip(1) {
            self.report(Report::new(
                C::InvalidField,
                extra.key.range,
                "a declaration has at most one `cite`",
            ));
        }
        let mut relations = Vec::new();
        for f in clauses {
            let target = first_expr(f).and_then(|e| match &e.kind {
                ExprKind::Ref(r) => Some(r),
                _ => None,
            });
            let make: Option<fn(String) -> Relation> = match f.key.text.as_str() {
                "subject_to" => Some(Relation::SubjectTo),
                "notwithstanding" => Some(Relation::Notwithstanding),
                "supersedes" => Some(Relation::Supersedes),
                "revives" => Some(Relation::Revives),
                "override" => Some(Relation::Override),
                "delegable_to" => {
                    for r in Self::refs(Some(&f)) {
                        if self
                            .ck
                            .scope
                            .authority(&r.head)
                            .map_err(|e| self.ck.report(e))
                            .is_ok()
                        {
                            relations.push(Relation::DelegableTo(r.head.text.clone()));
                        }
                    }
                    None
                }
                _ => None,
            };
            if let (Some(make), Some(target)) = (make, target) {
                if let Some(name) = self.value_ref(target, "declaration", |_| true) {
                    relations.push(make(name));
                }
            }
        }
        Metadata {
            id: id.text.clone(),
            label: label
                .map(|l| l.text.clone())
                .or_else(|| unmarked_doc(item.doc.as_deref()))
                .unwrap_or_default(),
            cite: cites.first().and_then(|f| self.cite_text(Some(f))),
            historical: cites
                .first()
                .and_then(|f| first_expr(f))
                .is_some_and(|e| matches!(e.kind, ExprKind::Cite(_, true))),
            effective: self.effective(item, rule),
            authority: self
                .manifest
                .map(|m| m.authority.clone())
                .unwrap_or_default(),
            relations,
            span: self.ck.span(item.range),
            imported: item
                .doc
                .as_deref()
                .is_some_and(|d| d.lines().any(is_import_marker)),
        }
    }
}

/// A `///` line that is exactly `imported`: the marker `litigate import`
/// writes on every node and link (SPEC §8.7).
fn is_import_marker(line: &str) -> bool {
    line.trim() == "imported"
}

/// The doc comment without import-marker lines, when anything is left (a
/// declaration's fallback label is never the marker).
fn unmarked_doc(doc: Option<&str>) -> Option<String> {
    let lines: Vec<&str> = doc?.lines().filter(|l| !is_import_marker(l)).collect();
    (!lines.is_empty()).then(|| lines.join("\n"))
}
