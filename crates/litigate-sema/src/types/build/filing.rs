//! Filing requirements (SPEC §3.7): `limit k { ... }` type-checked into a
//! [`Limit`]. Every number, ECF code and excluded part is exact or an
//! explicit hole; nothing is filled in.
use super::cx::Cx;
use crate::db::parse::tree::{
    Expr, ExprKind, Field, Item, LimitEntry, LimitEntryKind, LimitItem, Name,
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
        if !l.name.is_simple() {
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
            return None;
        }
        let meta = self.meta(&l.name.head, None, item, rule);
        let mut limit = Limit {
            meta,
            bounds: Vec::new(),
            certificates: Vec::new(),
            excludes: None,
            ecf: Vec::new(),
        };
        let mut seen: BTreeMap<String, TextRange> = BTreeMap::new();
        let mut certified: Vec<(&Name, TextRange)> = Vec::new();
        for entry in &l.entries {
            let cite = self.entry_cite(&entry.cites);
            let span = self.ck.span(entry.range);
            match &entry.kind {
                LimitEntryKind::Bound(measure, max) => {
                    let Some(m) = Measure::parse(&measure.text) else {
                        continue;
                    };
                    if !self.first(&mut seen, m.as_str(), entry, "a bound on") {
                        continue;
                    }
                    if let Some(max) = self.bound_max(m, max) {
                        limit.bounds.push(LengthBound {
                            measure: m,
                            max,
                            cite,
                            span,
                        });
                    }
                }
                LimitEntryKind::Certificate(kind, measures) => {
                    if !kind.is_simple() {
                        self.report(Report::new(
                            C::InvalidField,
                            kind.range,
                            format!(
                                "certificate kind `{}` must be a plain identifier",
                                kind.display()
                            ),
                        ));
                        continue;
                    }
                    let key = format!("certificate {}", kind.head.text);
                    if !self.first(&mut seen, &key, entry, "a requirement of") {
                        continue;
                    }
                    let mut named = Vec::new();
                    for m in measures.iter().flatten() {
                        match Measure::parse(&m.text) {
                            Some(x) if named.contains(&x) => self.report(Report::new(
                                C::InvalidField,
                                m.range,
                                format!("`{}` is named twice", m.text),
                            )),
                            Some(x) => {
                                named.push(x);
                                certified.push((m, entry.range));
                            }
                            None => {}
                        }
                    }
                    limit.certificates.push(CertificateReq {
                        kind: kind.head.text.clone(),
                        measures: named,
                        cite,
                        span,
                    });
                }
                LimitEntryKind::Excludes(items) => {
                    if !self.first(&mut seen, "excludes", entry, "a second") {
                        continue;
                    }
                    limit.excludes = Some(Exclusions {
                        items: self.excluded(items),
                        cite,
                        span,
                    });
                }
                LimitEntryKind::Ecf(code) => {
                    if let Some(code) = self.ecf_code(code, &limit.ecf) {
                        limit.ecf.push(Ecf { code, cite, span });
                    }
                }
            }
        }
        // A certificate goes with bounds the body declares.
        for (m, _) in certified {
            if !limit.bounds.iter().any(|b| b.measure.as_str() == m.text) {
                self.report(Report::new(
                    C::InvalidField,
                    m.range,
                    format!(
                        "the certificate goes with `{}`, which limit `{}` does not bound",
                        m.text, limit.meta.id
                    ),
                ));
            }
        }
        Some(limit)
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

    /// The entry's cite: its first `cite` (a cite hole is recorded as a
    /// hole and cites nothing); at most one.
    fn entry_cite(&mut self, cites: &[Field]) -> Option<String> {
        for extra in cites.iter().skip(1) {
            self.report(Report::new(
                C::InvalidField,
                extra.key.range,
                "a limit entry has at most one `cite`",
            ));
        }
        let first = cites.first();
        match first.and_then(super::cx::first_expr).map(|e| &e.kind) {
            Some(ExprKind::Cite(text, _)) => Some(text.clone()),
            _ => {
                self.string_or_hole(first.as_ref());
                None
            }
        }
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
        match &e.kind {
            ExprKind::Hole(reason) => {
                self.ck.holes.push((e.range, reason.clone()));
                Some(Authored::Hole(reason.clone()))
            }
            ExprKind::Str(code) => {
                let code = Authored::Known(code.clone());
                if declared.iter().any(|d| d.code == code) {
                    self.report(Report::new(
                        C::InvalidField,
                        e.range,
                        "this ECF event code is declared twice",
                    ));
                    return None;
                }
                Some(code)
            }
            // The grammar admits only a string or a hole.
            _ => None,
        }
    }
}
