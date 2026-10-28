//! Filing requirements (SPEC §3.7): `limit k { ... }` type-checked into a
//! [`Limit`]. Every number, ECF code and excluded part is exact or an
//! explicit hole; nothing is filled in.
use super::cx::Cx;
use crate::db::parse::tree::{
    Expr, ExprKind, Field, Item, LimitEntry, LimitEntryKind, LimitItem, RefPath,
};
use crate::db::span::{Report, TextRange};
use crate::hir::{
    Authored, CertificateReq, Ecf, Exclusions, LengthBound, Limit, Measure, Type, Value,
};
use litigate_base::DiagnosticCode as C;
use std::collections::BTreeMap;

impl Cx<'_, '_> {
    /// `limit k clauses { entries } clauses`.
    pub fn limit(&mut self, item: &Item, rule: Option<&Item>, l: &LimitItem) -> Option<Limit> {
        if !self.simple_limit_name(l) {
            return None;
        }
        let meta = self.meta(&l.name.head, None, item, rule);
        let mut limit = Limit {
            meta,
            bounds: Vec::new(),
            certificates: Vec::new(),
            excludes: None,
            ecf: Vec::new(),
            holes: BTreeMap::new(),
        };
        let decl = l.clauses.iter().find(|f| f.key.text == "cite");
        if let Some(Expr {
            kind: ExprKind::Hole(reason),
            ..
        }) = decl.and_then(super::cx::first_expr)
        {
            limit.holes.insert("cite".to_owned(), reason.clone());
        }
        let mut seen: BTreeMap<String, TextRange> = BTreeMap::new();
        let mut certified: Vec<&(Measure, TextRange)> = Vec::new();
        for entry in &l.entries {
            let (cite, hole) = self.entry_cite(&entry.cites);
            let span = self.ck.span(entry.range);
            match &entry.kind {
                LimitEntryKind::Bound(m, max) => {
                    if !self.first(&mut seen, m.as_str(), entry, "a bound on") {
                        continue;
                    }
                    if let Some(max) = self.bound_max(*m, max) {
                        cite_hole(&mut limit.holes, "bounds", limit.bounds.len(), hole);
                        limit.bounds.push(LengthBound {
                            measure: *m,
                            max,
                            cite,
                            span,
                        });
                    }
                }
                LimitEntryKind::Certificate(kind, measures) => {
                    if let Some(c) = self.certificate(&mut seen, entry, kind, measures) {
                        certified.extend(measures);
                        let at = limit.certificates.len();
                        cite_hole(&mut limit.holes, "certificates", at, hole);
                        limit.certificates.push(CertificateReq { cite, span, ..c });
                    }
                }
                LimitEntryKind::Excludes(items) => {
                    if !self.first(&mut seen, "excludes", entry, "a second") {
                        continue;
                    }
                    if let Some(CiteHole(reason)) = hole {
                        limit.holes.insert("excludes/cite".to_owned(), reason);
                    }
                    limit.excludes = Some(Exclusions {
                        items: self.excluded(items),
                        cite,
                        span,
                    });
                }
                LimitEntryKind::Ecf(code) => {
                    if let Some(code) = self.ecf_code(code, &limit.ecf) {
                        cite_hole(&mut limit.holes, "ecf", limit.ecf.len(), hole);
                        limit.ecf.push(Ecf { code, cite, span });
                    }
                }
            }
        }
        self.unbounded(&limit, &certified);
        Some(limit)
    }

    /// Whether `limit k` names its paper kind with a plain identifier;
    /// L2007 when not.
    fn simple_limit_name(&mut self, l: &LimitItem) -> bool {
        if l.name.is_simple() {
            return true;
        }
        self.report(
            Report::new(
                C::InvalidField,
                l.name.range,
                format!(
                    "`limit {}` names a paper kind with a plain identifier",
                    l.name.display()
                ),
            )
            .note("a layer names the limit it changes with a relation: `override pack::kind`"),
        );
        false
    }

    /// L2007 for each measure a certificate goes with that `limit` does
    /// not bound.
    fn unbounded(&mut self, limit: &Limit, certified: &[&(Measure, TextRange)]) {
        for (m, range) in certified {
            if !limit.bounds.iter().any(|b| b.measure == *m) {
                self.report(Report::new(
                    C::InvalidField,
                    *range,
                    format!(
                        "the certificate goes with `{}`, which limit `{}` does not bound",
                        m.as_str(),
                        limit.meta.id
                    ),
                ));
            }
        }
    }

    /// `requires certificate kind for [measures]`, without its cite and
    /// span: a plain kind, once per body, each measure named once.
    fn certificate(
        &mut self,
        seen: &mut BTreeMap<String, TextRange>,
        entry: &LimitEntry,
        kind: &RefPath,
        measures: &[(Measure, TextRange)],
    ) -> Option<CertificateReq> {
        if !kind.is_simple() {
            self.report(Report::new(
                C::InvalidField,
                kind.range,
                format!(
                    "certificate kind `{}` must be a plain identifier",
                    kind.display()
                ),
            ));
            return None;
        }
        let key = format!("certificate {}", kind.head.text);
        if !self.first(seen, &key, entry, "a requirement of") {
            return None;
        }
        let mut named = Vec::new();
        for (m, range) in measures {
            if named.contains(m) {
                self.report(Report::new(
                    C::InvalidField,
                    *range,
                    format!("`{}` is named twice", m.as_str()),
                ));
            } else {
                named.push(*m);
            }
        }
        Some(CertificateReq {
            kind: kind.head.text.clone(),
            measures: named,
            cite: None,
            span: None,
        })
    }

    /// Whether `key` is new in this body; L2007 at `entry` when not.
    fn first(
        &mut self,
        seen: &mut BTreeMap<String, TextRange>,
        key: &str,
        entry: &LimitEntry,
        what: &str,
    ) -> bool {
        if let Some(first) = seen.get(key) {
            self.report(
                Report::new(
                    C::InvalidField,
                    entry.range,
                    format!("{what} `{key}` is declared twice in one limit"),
                )
                .note_at(*first, "first declared here"),
            );
            return false;
        }
        seen.insert(key.to_owned(), entry.range);
        true
    }

    /// The entry's cite: its first `cite`, at most one. A cite hole
    /// (`cite ?("...")`, a source whose currency is unconfirmed) is
    /// recorded as a hole and its reason stands as the cite, as on the
    /// declaration; the second value is that hole.
    fn entry_cite(&mut self, cites: &[Field]) -> (Option<String>, Option<CiteHole>) {
        for extra in cites.iter().skip(1) {
            self.report(Report::new(
                C::InvalidField,
                extra.key.range,
                "a limit entry has at most one `cite`",
            ));
        }
        let Some(first) = cites.first() else {
            return (None, None);
        };
        let hole = match super::cx::first_expr(first) {
            Some(Expr {
                kind: ExprKind::Hole(reason),
                ..
            }) => Some(CiteHole(reason.clone())),
            _ => None,
        };
        (self.cite_text(Some(&first)), hole)
    }

    /// A bound's most: a whole number above zero, or a hole.
    fn bound_max(&mut self, m: Measure, e: &Expr) -> Option<Authored<u64>> {
        if let ExprKind::Hole(reason) = &e.kind {
            self.ck.holes.push((e.range, reason.clone()));
            return Some(Authored::Hole(reason.clone()));
        }
        let n = match self.ck.check(e, &Type::Number).value {
            Some(Value::Number(v)) if v.denominator == 1 && v.numerator > 0 => {
                u64::try_from(v.numerator).ok()
            }
            _ => None,
        };
        if n.is_none() {
            self.report(Report::new(
                C::TypeMismatch,
                e.range,
                format!(
                    "a `{}` bound is a whole number above zero, or a hole `?(\"reason\")`",
                    m.as_str()
                ),
            ));
        }
        n.map(Authored::Known)
    }

    /// The parts an `excludes` list names: plain identifiers, once each,
    /// and holes for parts the vendored sources do not settle.
    fn excluded(&mut self, items: &[Expr]) -> Vec<Authored<String>> {
        let mut out: Vec<Authored<String>> = Vec::new();
        for e in items {
            match &e.kind {
                ExprKind::Hole(reason) => {
                    self.ck.holes.push((e.range, reason.clone()));
                    out.push(Authored::Hole(reason.clone()));
                }
                ExprKind::Ref(r) if r.is_simple() => {
                    let item = Authored::Known(r.head.text.clone());
                    if out.contains(&item) {
                        self.report(Report::new(
                            C::InvalidField,
                            e.range,
                            format!("`{}` is excluded twice", r.head.text),
                        ));
                    } else {
                        out.push(item);
                    }
                }
                _ => self.report(Report::new(
                    C::TypeMismatch,
                    e.range,
                    "an excluded part is a plain identifier (`cover-page`) or a hole",
                )),
            }
        }
        out
    }

    /// An ECF event code: a string or a hole, each code once.
    fn ecf_code(&mut self, e: &Expr, declared: &[Ecf]) -> Option<Authored<String>> {
        if let ExprKind::Hole(reason) = &e.kind {
            self.ck.holes.push((e.range, reason.clone()));
            return Some(Authored::Hole(reason.clone()));
        }
        // The grammar admits only a string or a hole.
        let code = literal_text(Some(e)).map(Authored::Known);
        if code
            .as_ref()
            .is_some_and(|c| declared.iter().any(|d| d.code == *c))
        {
            self.report(Report::new(
                C::InvalidField,
                e.range,
                "this ECF event code is declared twice",
            ));
            return None;
        }
        code
    }
}

/// A limit entry's cite hole, with its reason (if the hole gives one).
struct CiteHole(Option<String>);

/// Records `hole`, the cite hole of the `at`th entry of `list`, under its
/// field path (`bounds/0/cite`).
fn cite_hole(
    holes: &mut BTreeMap<String, Option<String>>,
    list: &str,
    at: usize,
    hole: Option<CiteHole>,
) {
    if let Some(CiteHole(reason)) = hole {
        holes.insert(format!("{list}/{at}/cite"), reason);
    }
}

/// The text of a string or cite literal.
fn literal_text(e: Option<&Expr>) -> Option<String> {
    match e.map(|e| &e.kind) {
        Some(ExprKind::Str(text) | ExprKind::Cite(text, _)) => Some(text.clone()),
        _ => None,
    }
}
