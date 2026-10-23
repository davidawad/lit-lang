//! `.case` source to [`Case`] HIR plus [`CaseSpans`] (SPEC §3.2, §5.6).
//!
//! `uses` items resolve through `Lit.lock`: the locked version must satisfy
//! the requirement, and the pin records the locked version and digest.
//! References resolve in the primary package unless qualified `pack::x`.
//! Any syntax or resolution error leaves the case unresolved (`None`):
//! replaying a partly understood history would report noise.
mod blocks;

use super::event::{self, ToSpan};
use super::graph::Graph;
use super::spans::{CaseSpans, EventSpans};
use crate::db::parse::tree::RefPath;
use crate::db::parse::{ref_path, Lower};
use crate::db::span::{to_span, Report, TextRange};
use crate::hir::{
    ActorRole, Analysis, Case, CaseSource, Event, EventParty, Money, Package, PackagePin, Party,
    Perspective,
};
use crate::pkg::lock::LockFile;
use crate::pkg::semver::{Requirement, Version};
use crate::resolve::suggest::nearest;
use crate::types::rational;
use litigate_base::{Applicability, Diagnostic, DiagnosticCode as C, FileId, LineIndex};
use litigate_syntax::{SyntaxKind as K, SyntaxNode};
use std::collections::BTreeMap;

/// Resolver output.
#[derive(Debug, Clone, PartialEq)]
pub struct Resolved {
    /// The matter, when it resolved without error.
    pub case: Option<Case>,
    /// Where each part was written.
    pub spans: CaseSpans,
    /// Syntax and resolution diagnostics.
    pub diagnostics: Vec<Diagnostic>,
}

/// Money literal text (`-2_000.50.usd`) in minor units.
pub(super) fn money(text: &str) -> Option<Money> {
    let (neg, text) = text.strip_prefix('-').map_or((false, text), |t| (true, t));
    let (amount, currency) = text.rsplit_once('.')?;
    if amount.split_once('.').is_some_and(|(_, f)| f.len() > 2) {
        return None;
    }
    let minor = rational::mul(
        rational::parse_decimal(amount).ok()?,
        crate::hir::Rational::integer(100),
    )
    .ok()?;
    (minor.denominator == 1).then(|| Money {
        minor_units: if neg {
            -minor.numerator
        } else {
            minor.numerator
        },
        currency: currency.to_owned(),
    })
}

pub(super) struct R<'a, 'g> {
    graph: Option<Graph<'g>>,
    packages: &'a [&'g Package],
    lock: Option<&'a LockFile>,
    span: ToSpan<'a>,
    lower: Lower,
    out: Vec<Report>,
    spans: CaseSpans,
}

fn sig(node: &SyntaxNode) -> TextRange {
    crate::db::parse::significant_range(node)
}

fn token_text(node: &SyntaxNode, kind: K) -> Option<(String, TextRange)> {
    node.descendants_with_tokens()
        .filter_map(rowan::NodeOrToken::into_token)
        .find(|t| t.kind() == kind)
        .map(|t| {
            let n = crate::db::parse::name(&t);
            (n.text, n.range)
        })
}

fn child(node: &SyntaxNode, kind: K) -> Option<SyntaxNode> {
    node.children().find(|c| c.kind() == kind)
}

fn children(node: &SyntaxNode, kind: K) -> impl Iterator<Item = SyntaxNode> {
    node.children().filter(move |c| c.kind() == kind)
}

/// The value expression node of a `RecordField` (after its `:`).
fn field_value(f: &SyntaxNode) -> Option<SyntaxNode> {
    crate::db::parse::nodes_after(f, K::Colon).find(|n| crate::db::parse::is_expr(n.kind()))
}

/// Resolves `.case` source `text` (repository-relative `file`) against the
/// loaded `packages` and the matter's `Lit.lock`.
#[must_use]
pub fn resolve(file: &str, text: &str, packages: &[&Package], lock: Option<&LockFile>) -> Resolved {
    let file = FileId(file.to_owned());
    let index = LineIndex::new(text);
    let span = |r: TextRange| to_span(&file, &index, r);
    let parse = litigate_syntax::parse_case(text);
    let mut r = R {
        graph: None,
        packages,
        lock,
        span: &span,
        lower: Lower::default(),
        out: parse
            .errors()
            .iter()
            .map(crate::db::parse::syntax_report)
            .collect(),
        spans: CaseSpans::default(),
    };
    let syntax_ok = r.out.is_empty();
    let case = parse.case_file().and_then(|f| r.file(f.syntax(), text));
    let mut out = std::mem::take(&mut r.out);
    out.append(&mut r.lower.reports);
    let diagnostics: Vec<Diagnostic> = out.iter().map(|x| x.to_diagnostic(&file, &index)).collect();
    Resolved {
        case: case.filter(|_| syntax_ok && diagnostics.is_empty()),
        spans: r.spans,
        diagnostics,
    }
}

impl R<'_, '_> {
    fn s(&self, r: TextRange) -> litigate_base::Span {
        (self.span)(r)
    }

    fn file(&mut self, root: &SyntaxNode, text: &str) -> Option<Case> {
        let (id, id_range) = root
            .children_with_tokens()
            .filter_map(rowan::NodeOrToken::into_token)
            .find(|t| t.kind() == K::Ident)
            .map(|t| (t.text().to_owned(), crate::db::parse::range(t.text_range())))?;
        self.spans.header = Some(self.s(id_range));
        let uses = self.uses(root);
        let as_of_node = child(root, K::AsOfDecl)?;
        let (as_of, as_of_range) = token_text(&as_of_node, K::Date)?;
        self.spans.as_of = Some(self.s(as_of_range));
        let known_at = child(root, K::KnownAtDecl)
            .and_then(|n| token_text(&n, K::Date))
            .map(|t| t.0);
        let summary = child(root, K::SummaryDecl)
            .and_then(|n| token_text(&n, K::String))?
            .0;
        let notes = child(root, K::NotesDecl)
            .and_then(|n| token_text(&n, K::String))
            .map(|t| t.0);
        let sources = children(root, K::SourceDecl)
            .filter_map(|n| Self::source(&n))
            .collect();
        let uses = uses?;
        let start = child(root, K::StartDecl).and_then(|n| {
            let r = child(&n, K::Ref).and_then(|x| ref_path(&x))?;
            self.spans.start = Some((self.s(sig(&n)), self.s(r.range)));
            self.node(&r)
        });
        let perspective = child(root, K::PerspectiveDecl).and_then(|n| self.perspective(&n));
        let parties = child(root, K::PartiesBlock).map_or_else(Vec::new, |b| self.parties(&b));
        let events = child(root, K::EventsBlock).map_or_else(Vec::new, |b| self.events(&b, text));
        let facts = child(root, K::FactsBlock).map_or_else(BTreeMap::new, |b| self.facts(&b));
        let stakes =
            child(root, K::StakesBlock).map_or_else(BTreeMap::new, |b| self.stakes(&b, text));
        let observations = child(root, K::BeliefsBlock).map_or_else(Vec::new, |b| self.beliefs(&b));
        let analysis =
            child(root, K::AnalysisBlock).map_or_else(Analysis::default, |b| self.analysis(&b));
        Some(Case {
            id,
            summary,
            notes,
            sources,
            uses,
            as_of,
            known_at,
            start,
            perspective,
            parties,
            events,
            facts,
            stakes,
            observations,
            analysis,
            removals: Vec::new(),
        })
    }

    fn source(n: &SyntaxNode) -> Option<CaseSource> {
        let title = child(n, K::Literal)
            .and_then(|l| token_text(&l, K::String))?
            .0;
        let fields: Vec<SyntaxNode> = child(n, K::FieldList)
            .map(|l| children(&l, K::Field).collect())
            .unwrap_or_default();
        let field = |kw: K| {
            fields
                .iter()
                .find(|f| f.first_token().is_some_and(|t| t.kind() == kw))
        };
        let url = field(K::KwUrl)
            .and_then(|f| token_text(f, K::String))
            .map(|t| t.0);
        let as_of = field(K::KwAsOf).and_then(|f| {
            token_text(f, K::Date).map(|t| t.0).or_else(|| {
                let hole = child(f, K::HoleExpr)?;
                Some(token_text(&hole, K::String).map_or_else(String::new, |t| t.0))
            })
        });
        Some(CaseSource { title, url, as_of })
    }

    /// `uses` pins through `Lit.lock`; builds the graph of used packages.
    fn uses(&mut self, root: &SyntaxNode) -> Option<Vec<crate::hir::PackagePin>> {
        let items: Vec<SyntaxNode> =
            child(root, K::UsesDecl).map(|u| children(&u, K::UseItem).collect())?;
        let mut pins = Vec::new();
        let mut used = Vec::new();
        for item in &items {
            let range = sig(item);
            self.spans.uses.push(Some(self.s(range)));
            let (name, _) = token_text(item, K::Ident)?;
            let req_text = child(item, K::VersionReq)
                .map(|v| v.text().to_string().split_whitespace().collect::<String>())?;
            let Some(pkg) = self.packages.iter().find(|p| p.manifest.name == name) else {
                self.out.push(Report::new(
                    C::UndeclaredDependency,
                    range,
                    format!("package `{name}` is not loaded"),
                ));
                continue;
            };
            let req = Requirement::parse(&req_text);
            let locked = self
                .lock
                .and_then(|l| l.packages.iter().find(|p| p.name == name));
            match (locked, req) {
                (Some(l), Some(req))
                    if Version::parse(&l.version).is_some_and(|v| req.matches(&v)) =>
                {
                    pins.push(PackagePin {
                        name: name.clone(),
                        pack_id: pkg.manifest.pack_id.clone(),
                        version: l.version.clone(),
                        content_hash: l.digest.trim_start_matches("sha256:").to_owned(),
                    });
                    used.push(*pkg);
                }
                (Some(l), _) => self.out.push(
                    Report::new(
                        C::StaleLock,
                        range,
                        format!(
                            "Lit.lock pins `{name}` {}, which does not satisfy `{req_text}`",
                            l.version
                        ),
                    )
                    .note("run `litigate lock`"),
                ),
                (None, _) => self.out.push(
                    Report::new(
                        C::StaleLock,
                        range,
                        if self.lock.is_some() {
                            format!("`{name}` is not pinned in Lit.lock")
                        } else {
                            format!("`{name}` is not pinned: the matter has no Lit.lock")
                        },
                    )
                    .note("run `litigate lock` to pin the packages this matter uses"),
                ),
            }
        }
        (!used.is_empty()).then(|| self.graph = Some(Graph::new(used, self.packages)));
        Some(pins)
    }

    /// A node reference, qualified.
    fn node(&mut self, r: &RefPath) -> Option<String> {
        let graph = self.graph.as_ref()?;
        let q = graph.qualify(graph.primary(), &r.display());
        if graph.has_node(&q) {
            return Some(q);
        }
        let primary = graph.primary().to_owned();
        let names: Vec<String> = graph
            .packages
            .iter()
            .flat_map(|p| super::graph::declarations(p))
            .filter_map(|d| match d {
                crate::hir::Declaration::Node(n) => Some(n.meta.id.clone()),
                _ => None,
            })
            .collect();
        let mut rep = Report::new(
            C::UnresolvedName,
            r.range,
            format!("no node `{}` in `{primary}`", r.display()),
        );
        if let Some(best) = nearest(&r.display(), names.iter().map(String::as_str)) {
            rep = rep.fix(r.range, best, Applicability::MaybeIncorrect);
        }
        self.out.push(rep);
        None
    }

    fn perspective(&mut self, n: &SyntaxNode) -> Option<Perspective> {
        self.spans.perspective = Some(self.s(sig(n)));
        if let Some(r) = child(n, K::Ref).and_then(|x| ref_path(&x)) {
            return Some(Perspective::Party(r.display()));
        }
        let record = child(n, K::RecordExpr)?;
        let mut map = BTreeMap::new();
        for f in children(&record, K::RecordField) {
            let (actor, _) = token_text(&f, K::Ident)?;
            let value = field_value(&f)?;
            let role: Option<ActorRole> = serde_json::from_value(serde_json::Value::String(
                value.text().to_string().trim().to_owned(),
            ))
            .ok();
            match role {
                Some(role) => {
                    map.insert(actor, role);
                }
                None => self.out.push(Report::new(
                    C::TypeMismatch,
                    sig(&value),
                    "expected a role: `self`, `opponent`, or `nature`",
                )),
            }
        }
        Some(Perspective::Actors(map))
    }

    fn parties(&mut self, block: &SyntaxNode) -> Vec<Party> {
        children(block, K::PartyDecl)
            .filter_map(|p| {
                let (id, _) = token_text(&p, K::Ident)?;
                let refs: Vec<RefPath> =
                    children(&p, K::Ref).filter_map(|r| ref_path(&r)).collect();
                let (actor, entity) = (refs.first()?, refs.get(1)?);
                self.spans
                    .parties
                    .push((Some(self.s(sig(&p))), Some(self.s(actor.range))));
                Some(Party {
                    id,
                    actor: actor.display(),
                    entity_type: entity.display(),
                })
            })
            .collect()
    }

    fn events(&mut self, block: &SyntaxNode, text: &str) -> Vec<Event> {
        let lines: Vec<SyntaxNode> = children(block, K::CaseEvent).collect();
        self.sort_fix(&lines, text);
        let mut events = Vec::new();
        for line in &lines {
            let date = child(line, K::Literal).and_then(|l| token_text(&l, K::Date));
            let call = child(line, K::CallExpr).and_then(|c| self.lower.expr(&c));
            let party = child(line, K::Ref).and_then(|r| ref_path(&r));
            let relation = line
                .children_with_tokens()
                .filter_map(rowan::NodeOrToken::into_token)
                .find_map(|t| match t.kind() {
                    K::KwBy => Some(EventParty::By),
                    K::KwOn => Some(EventParty::On),
                    _ => None,
                });
            let authority = child(line, K::CiteExpr).map(|c| {
                token_text(&c, K::CiteText).map_or_else(
                    || token_text(&c, K::String).map_or_else(String::new, |t| t.0),
                    |t| t.0.split_whitespace().collect::<Vec<_>>().join(" "),
                )
            });
            self.spans.events.push(EventSpans {
                whole: Some(self.s(sig(line))),
                date: date.as_ref().map(|d| self.s(d.1)),
                action: call.as_ref().map(|c| self.s(c.range)),
                party: party.as_ref().map(|p| self.s(p.range)),
            });
            let (Some((date, date_range)), Some(call), Some(party), Some(relation)) =
                (date, call, party, relation)
            else {
                continue;
            };
            if !crate::types::date::is_valid(&date) {
                self.out.push(Report::new(
                    C::InvalidDate,
                    date_range,
                    format!("`{date}` is not a valid date"),
                ));
                continue;
            }
            let Some(graph) = self.graph.as_ref() else {
                continue;
            };
            let Some(action) = event::call(graph, &call, self.span, &mut self.out) else {
                continue;
            };
            events.push(Event {
                date,
                action,
                relation,
                party: party.display(),
                authority,
            });
        }
        events
    }

    /// The machine-applicable fix that sorts out-of-order event lines.
    fn sort_fix(&mut self, lines: &[SyntaxNode], text: &str) {
        let key = |n: &SyntaxNode| {
            child(n, K::Literal)
                .and_then(|l| token_text(&l, K::Date))
                .map(|d| d.0)
        };
        let keys: Vec<Option<String>> = lines.iter().map(key).collect();
        if keys.windows(2).all(|w| w[0] <= w[1]) {
            return;
        }
        let ranges: Vec<TextRange> = lines.iter().map(sig).collect();
        let slice = |r: TextRange| {
            text.get(r.start as usize..r.end as usize)
                .unwrap_or_default()
        };
        let mut order: Vec<usize> = (0..lines.len()).collect();
        order.sort_by(|&a, &b| keys[a].cmp(&keys[b]));
        let mut replacement = String::new();
        for (slot, &i) in order.iter().enumerate() {
            replacement.push_str(slice(ranges[i]));
            if let Some(next) = ranges.get(slot + 1) {
                replacement.push_str(slice(TextRange::new(ranges[slot].end, next.start)));
            }
        }
        if let (Some(first), Some(last)) = (ranges.first(), ranges.last()) {
            self.spans.sort_events = Some((self.s(first.cover(*last)), replacement));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::money;

    #[test]
    fn money_is_exact_minor_units() {
        assert_eq!(
            money("-2_000_000.usd").map(|m| m.minor_units),
            Some(-200_000_000)
        );
        assert_eq!(money("0.50.usd").map(|m| m.minor_units), Some(50));
        assert_eq!(money("1.005.usd"), None);
    }
}
