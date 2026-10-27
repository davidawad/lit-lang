//! Calibration sets (`calibration id: "Title" { ... }`): every target
//! resolves to an edge or node of a package this one sees, every value is
//! typed, and every entry names a declared source with a sample size, so
//! nothing that does not lower is dropped.
use super::cx::{first_expr, nested, Cx};
use crate::db::parse::tree::{Atom, Expr, ExprKind, Field, Item, Name, PathSep, RefPath};
use crate::db::span::Report;
use crate::hir::{
    CalibratedValue, Calibration, CalibrationEntry, CalibrationSource, Rational, Value,
};
use crate::resolve::sig::{Signature, ValueKind};
use crate::resolve::suggest;
use crate::types::rational as q;
use litigate_base::DiagnosticCode as C;

/// What an entry's target must be.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Target {
    Edge,
    Node,
}

impl Target {
    const fn noun(self) -> &'static str {
        match self {
            Self::Edge => "edge",
            Self::Node => "node",
        }
    }

    /// Whether `sig` declares a target of this kind named `local`.
    fn in_sig(self, sig: &Signature, local: &str) -> bool {
        match self {
            Self::Node => matches!(
                sig.values.get(local).map(|v| &v.kind),
                Some(ValueKind::Node { .. })
            ),
            Self::Edge => sig.values.values().any(
                |v| matches!(&v.kind, ValueKind::Node { edges, .. } if edges.iter().any(|e| e == local)),
            ),
        }
    }

    /// Every name of this kind `sig` declares.
    fn names(self, sig: &Signature) -> Vec<&str> {
        sig.values
            .iter()
            .flat_map(|(name, v)| match (&v.kind, self) {
                (ValueKind::Node { edges, .. }, Self::Edge) => {
                    edges.iter().map(String::as_str).collect()
                }
                (ValueKind::Node { .. }, Self::Node) => vec![name.as_str()],
                _ => Vec::new(),
            })
            .collect()
    }
}

impl Cx<'_, '_> {
    /// `calibration id: "Title" { note, source..., p|duration|attr... }`.
    pub fn calibration(
        &mut self,
        item: &Item,
        name: &Name,
        title: &Name,
        fields: &[Field],
    ) -> Calibration {
        let mut meta = self.meta(name, Some(title), item, None);
        meta.label.clone_from(&title.text);
        let mut description = None;
        for f in fields.iter().filter(|f| f.key.text == "note") {
            if description.is_some() {
                self.report(Report::new(
                    C::InvalidField,
                    f.key.range,
                    "`note` is given more than once",
                ));
            }
            description = Cx::string(Some(&f));
        }
        let mut sources: Vec<CalibrationSource> = Vec::new();
        for f in fields.iter().filter(|f| f.key.text == "source") {
            let Some(source) = self.calibration_source(f) else {
                continue;
            };
            if sources.iter().any(|s| s.id == source.id) {
                self.report(Report::new(
                    C::InvalidField,
                    f.key.range,
                    format!("source `{}` is declared more than once", source.id),
                ));
            } else {
                sources.push(source);
            }
        }
        let entries = fields
            .iter()
            .filter(|f| matches!(f.key.text.as_str(), "p" | "duration" | "attr"))
            .filter_map(|f| self.calibration_entry(f, &sources))
            .collect();
        Calibration {
            meta,
            description,
            sources,
            entries,
        }
    }

    /// `source id "Title" { url, vintage [from, to), retrieved date }`.
    fn calibration_source(&mut self, f: &Field) -> Option<CalibrationSource> {
        let [(Atom::Expr(id), _), (Atom::Expr(title), _), ..] = f.values.as_slice() else {
            return None;
        };
        let (ExprKind::Ref(id), ExprKind::Str(title)) = (&id.kind, &title.kind) else {
            return None;
        };
        let parts = self.once(nested(f));
        let url = Cx::string(parts.get("url"));
        let vintage = match parts.get("vintage").and_then(|v| v.values.first()) {
            Some((Atom::Interval(from, to), _)) => Some((from.text.clone(), to.as_ref())),
            _ => None,
        };
        let retrieved = match parts.get("retrieved").and_then(|v| first_expr(v)) {
            Some(Expr {
                kind: ExprKind::Date(d),
                ..
            }) => Some(d.clone()),
            _ => None,
        };
        let (Some(url), Some((from, to))) = (url, vintage) else {
            self.report(Report::new(
                C::InvalidField,
                f.key.range,
                "a calibration source needs `url` and `vintage [from, to)`",
            ));
            return None;
        };
        Some(CalibrationSource {
            id: id.display(),
            title: title.clone(),
            url,
            from,
            to: to.map(|t| t.text.clone()),
            retrieved,
        })
    }

    /// `p Ref ProbValue {..}`, `duration Ref Duration {..}` or `attr Ref
    /// name Number {..}`, with `{ source s, n N, method "..." }`.
    fn calibration_entry(
        &mut self,
        f: &Field,
        sources: &[CalibrationSource],
    ) -> Option<CalibrationEntry> {
        let (Some((Atom::Expr(target), _)), Some((Atom::Fields(opts), _))) =
            (f.values.first(), f.values.last())
        else {
            return None;
        };
        let ExprKind::Ref(target) = &target.kind else {
            return None;
        };
        let what = if f.key.text == "attr" {
            Target::Node
        } else {
            Target::Edge
        };
        let target = self.calibration_target(target, what);
        let value = self.calibrated_value(f, opts);
        let indexed = self.once(opts);
        let source = self.entry_source(f, indexed.get("source").copied(), sources);
        let n = self.sample_size(f, indexed.get("n").copied());
        Some(CalibrationEntry {
            target: target?,
            value: value?,
            source: source?,
            n: n?,
            method: Cx::string(indexed.get("method")),
            span: self.ck.span(f.range),
        })
    }

    /// The value of entry `f`, between its target and its options.
    fn calibrated_value(&mut self, f: &Field, opts: &[Field]) -> Option<CalibratedValue> {
        match (f.key.text.as_str(), &f.values[1..f.values.len() - 1]) {
            ("p", [(Atom::Expr(e), _)]) => self.calibrated_probability(e, opts, f),
            ("duration", [value]) => {
                let estimate = Field {
                    key: f.key.clone(),
                    values: vec![value.clone()],
                    range: f.range,
                };
                self.duration(&estimate)
                    .map(|value| CalibratedValue::Duration { value })
            }
            ("attr", [(Atom::Expr(name), _), (Atom::Expr(e), _)]) => {
                let ExprKind::Ref(name) = &name.kind else {
                    return None;
                };
                if !name.is_simple() {
                    self.report(Report::new(
                        C::InvalidField,
                        name.range,
                        "an attribute name is one identifier",
                    ));
                    return None;
                }
                let value = self.number(e, "an attribute value is a constant number")?;
                Some(CalibratedValue::Attr {
                    name: name.head.text.clone(),
                    value,
                })
            }
            _ => None,
        }
    }

    /// The declared source `field` names.
    fn entry_source(
        &mut self,
        f: &Field,
        field: Option<&Field>,
        sources: &[CalibrationSource],
    ) -> Option<String> {
        let Some(field) = field else {
            self.report(Report::new(
                C::InvalidField,
                f.key.range,
                "a calibrated value needs `source`, the source it was measured from",
            ));
            return None;
        };
        let Some(Expr {
            kind: ExprKind::Ref(r),
            ..
        }) = first_expr(field)
        else {
            return None;
        };
        let id = r.display();
        if sources.iter().any(|s| s.id == id) {
            return Some(id);
        }
        let mut report = Report::new(
            C::UnresolvedName,
            r.range,
            format!("no source `{id}` is declared in this calibration"),
        );
        if let Some(best) = suggest::nearest(&id, sources.iter().map(|s| s.id.as_str())) {
            report = report.note(format!("a source with a similar name exists: `{best}`"));
        }
        self.report(report);
        None
    }

    /// The positive integer sample size `field` gives.
    fn sample_size(&mut self, f: &Field, field: Option<&Field>) -> Option<u64> {
        const WHAT: &str = "`n` is a sample size: a positive integer";
        let Some(e) = field.and_then(first_expr) else {
            self.report(Report::new(
                C::InvalidField,
                f.key.range,
                "a calibrated value needs `n`, the sample size behind it",
            ));
            return None;
        };
        let r = self.number(e, WHAT)?;
        let n = (r.denominator == 1)
            .then(|| u64::try_from(r.numerator).ok())
            .flatten()
            .filter(|n| *n > 0);
        if n.is_none() {
            self.report(Report::new(C::InvalidField, e.range, WHAT));
        }
        n
    }

    /// A constant number, else `message` at `e`.
    fn number(&mut self, e: &Expr, message: &str) -> Option<Rational> {
        if let Some(Value::Number(r) | Value::Probability(r)) = self.ck.infer(e).value {
            return Some(r);
        }
        self.report(Report::new(C::TypeMismatch, e.range, message));
        None
    }

    /// A probability in [0, 1], or a hole clearing the edge's own one,
    /// which needs a `method` saying why.
    fn calibrated_probability(
        &mut self,
        e: &Expr,
        opts: &[Field],
        f: &Field,
    ) -> Option<CalibratedValue> {
        if let ExprKind::Hole(_) = &e.kind {
            if !opts.iter().any(|o| o.key.text == "method") {
                self.report(Report::new(
                    C::InvalidProbability,
                    f.key.range,
                    "a cleared probability (`?`) needs `method` saying why it is cleared",
                ));
                return None;
            }
            return Some(CalibratedValue::Probability { value: None });
        }
        let p = self.number(e, "`p` is a constant probability")?;
        if !q::is_probability(p) {
            self.report(Report::new(
                C::InvalidProbability,
                e.range,
                format!("probability {} is outside [0, 1]", q::display(p)),
            ));
            return None;
        }
        Some(CalibratedValue::Probability { value: Some(p) })
    }

    /// `(engine prefix, pack id)` of a target: `local` (this package),
    /// `pack::local`, or `base@origin::local` (an instance this package
    /// declares).
    fn target_pack(&mut self, r: &RefPath) -> Option<(String, String)> {
        let own = self.ck.scope.own;
        match (&r.instance, r.rest.as_slice()) {
            (None, []) => Some((own.pack_id.clone(), own.pack_id.clone())),
            (None, [(PathSep::Colons, _)]) => Some((r.head.text.clone(), r.head.text.clone())),
            (Some(origin), [(PathSep::Colons, _)]) => {
                let key = format!("{}@{}", r.head.text, origin.text);
                let base = self
                    .ck
                    .scope
                    .packs
                    .iter()
                    .filter(|s| s.package == own.package)
                    .find_map(|s| match s.values.get(&key).map(|v| &v.kind) {
                        Some(ValueKind::Instance(pack)) => Some(pack.clone().unwrap_or_default()),
                        _ => None,
                    });
                if base.is_none() {
                    self.report(Report::new(
                        C::UnresolvedName,
                        r.head.range.cover(origin.range),
                        format!("no instance `{key}` is declared in this package"),
                    ));
                }
                Some((key, base?))
            }
            _ => {
                self.report(Report::new(
                    C::UnresolvedName,
                    r.range,
                    format!(
                        "`{}` is not a calibration target: write `local`, `pack::local` or \
                         `base@origin::local`",
                        r.display()
                    ),
                ));
                None
            }
        }
    }

    /// The engine ref of a calibration target naming an edge or a node of
    /// its pack ([`Self::target_pack`]).
    fn calibration_target(&mut self, r: &RefPath, what: Target) -> Option<String> {
        let (prefix, pack) = self.target_pack(r)?;
        let own = self.ck.scope.own;
        let sigs: Vec<&Signature> = if r.instance.is_none() && r.rest.is_empty() {
            let ours = self.ck.scope.packs.iter().copied();
            ours.filter(|s| s.package == own.package)
                .chain([own])
                .collect()
        } else {
            let all = self.ck.scope.packs.iter().copied();
            all.filter(|s| s.pack_id == pack).collect()
        };
        if sigs.is_empty() {
            let name = Name {
                text: pack,
                range: r.head.range,
            };
            let ids: Vec<&str> = self
                .ck
                .scope
                .packs
                .iter()
                .map(|s| s.pack_id.as_str())
                .collect();
            self.unresolved_target(&name, "pack", "", &ids);
            return None;
        }
        let last = r.last();
        if sigs.iter().any(|s| what.in_sig(s, &last.text)) {
            return Some(format!("{prefix}::{}", last.text));
        }
        let candidates: Vec<&str> = sigs.iter().flat_map(|s| what.names(s)).collect();
        let place = format!(" in pack `{prefix}`");
        self.unresolved_target(last, what.noun(), &place, &candidates);
        None
    }

    /// L1002: no `noun` named `name` (`place` says where it was looked
    /// for), with the nearest of `candidates`.
    fn unresolved_target(&mut self, name: &Name, noun: &str, place: &str, candidates: &[&str]) {
        let mut report = Report::new(
            C::UnresolvedName,
            name.range,
            format!("cannot find {noun} `{}`{place}", name.text),
        );
        if let Some(best) = suggest::nearest(&name.text, candidates.iter().copied()) {
            report = report.note(format!("a name with a similar spelling exists: `{best}`"));
        }
        self.report(report);
    }
}
