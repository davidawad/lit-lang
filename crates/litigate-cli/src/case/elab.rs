//! Header, parties, and events of a matter (SPEC §3.2, §5.6).
use super::{blocks, Cx};
use crate::hirx;
use crate::ws::Session;
use litigate_base::DiagnosticCode as C;
use litigate_sema::hir::{self, ActorRole, Declaration, Expr, ExprKind, Type, Value};
use litigate_sema::pkg::semver::{Requirement, Version};
use litigate_syntax::ast::{self, AstNode, CaseFile};
use litigate_syntax::SyntaxKind;

fn string(token: Option<litigate_syntax::SyntaxToken>) -> Option<String> {
    token.map(|t| ast::unescape(t.text()))
}

/// Resolves `uses` against the loaded packages (first = primary).
fn uses<'a>(cx: &mut Cx<'a>, session: &'a Session, file: &CaseFile) -> Vec<hir::PackagePin> {
    let items = file.uses().map(|u| u.items()).unwrap_or_default();
    items
        .iter()
        .filter_map(|item| {
            let span = cx.node_span(item.syntax());
            let name = item.package()?.text().to_owned();
            let req = item.requirement().and_then(|r| r.text())?;
            let requirement = Requirement::parse(&format!("{}{}", req.op, req.version));
            let candidates: Vec<&crate::ws::Pkg> =
                session.packages.iter().filter(|p| p.name == name).collect();
            let found = candidates.iter().find(|p| {
                Version::parse(&p.version)
                    .is_some_and(|v| requirement.as_ref().is_some_and(|r| r.matches(&v)))
            });
            let Some(pkg) = found else {
                let why = match candidates.first() {
                    Some(p) => format!(
                        "`{name}` {} does not satisfy `{}{}`",
                        p.version, req.op, req.version
                    ),
                    None => format!(
                        "no package named `{name}` was found (searched the --lib directories)"
                    ),
                };
                cx.error(C::UndeclaredDependency, span, why);
                return None;
            };
            let Some(hir) = pkg.check.package.as_ref() else {
                cx.error(
                    C::UndeclaredDependency,
                    span,
                    format!("`{name}` has errors; check it first"),
                );
                return None;
            };
            cx.packages.push(hir);
            Some(hir::PackagePin {
                name,
                pack_id: hir.manifest.pack_id.clone(),
                version: pkg.version.clone(),
                content_hash: pkg.digest.clone(),
            })
        })
        .collect()
}

fn sources(file: &CaseFile) -> Vec<hir::CaseSource> {
    file.sources()
        .iter()
        .map(|s| {
            let field = |kind: SyntaxKind| {
                s.fields()
                    .into_iter()
                    .find(|f| f.keyword_kind() == Some(kind))
            };
            let url = field(SyntaxKind::KwUrl)
                .and_then(|f| f.value_expr())
                .and_then(|e| ast::Literal::cast(e.syntax().clone()))
                .and_then(|l| l.string_value());
            let as_of = field(SyntaxKind::KwAsOf).and_then(|f| {
                let value = f.value()?;
                match ast::Hole::cast(value.clone()) {
                    Some(h) => h.reason(),
                    None => ast::Literal::cast(value)
                        .and_then(|l| l.token())
                        .map(|t| t.text().to_owned()),
                }
            });
            hir::CaseSource {
                title: string(s.title()).unwrap_or_default(),
                url,
                as_of,
            }
        })
        .collect()
}

fn perspective(cx: &mut Cx<'_>, file: &CaseFile) -> Option<hir::Perspective> {
    let p = file.perspective()?;
    if let Some(party) = p.party() {
        return Some(hir::Perspective::Party(party.text()));
    }
    let roles = p
        .roles()?
        .fields()
        .iter()
        .filter_map(|f| {
            let actor = f.key_text()?;
            let role = match f.value_ref().map(|r| r.text()).as_deref() {
                Some("self") => ActorRole::Protagonist,
                Some("opponent") => ActorRole::Opponent,
                Some("nature") => ActorRole::Nature,
                other => {
                    let span = cx.node_span(f.syntax());
                    cx.error(
                        C::InvalidParty,
                        span,
                        format!(
                            "`{}` is not a role; use self, opponent, or nature",
                            other.unwrap_or_default()
                        ),
                    );
                    return None;
                }
            };
            Some((actor, role))
        })
        .collect();
    Some(hir::Perspective::Actors(roles))
}

fn parties(cx: &mut Cx<'_>, file: &CaseFile) -> Vec<hir::Party> {
    let roles = cx
        .primary()
        .map(|p| p.manifest.roles.clone())
        .unwrap_or_default();
    let mut out: Vec<hir::Party> = Vec::new();
    for decl in file.parties() {
        let span = cx.node_span(decl.syntax());
        let (Some(id), Some(actor)) = (decl.name(), decl.actor()) else {
            continue;
        };
        let (id, actor) = (id.text().to_owned(), actor.text());
        if !roles.contains_key(&actor) {
            let known: Vec<&str> = roles.keys().map(String::as_str).collect();
            cx.error(
                C::InvalidParty,
                span.clone(),
                format!(
                    "`{actor}` is not an actor of the primary package; actors: {}",
                    known.join(", ")
                ),
            );
        }
        if out.iter().any(|p| p.id == id) {
            cx.error(
                C::InvalidParty,
                span,
                format!("party `{id}` is declared twice"),
            );
            continue;
        }
        out.push(hir::Party {
            id,
            actor,
            entity_type: decl.entity().map(|e| e.text()).unwrap_or_default(),
        });
    }
    out
}

/// The event type `name` declared by a used package.
fn event_type<'p>(cx: &Cx<'p>, name: &str) -> Option<&'p hir::EventType> {
    cx.packages.iter().find_map(|p| {
        hirx::declarations(p).find_map(|d| match d {
            Declaration::Event(e) if e.name == name => Some(e),
            _ => None,
        })
    })
}

/// The enum a parameter type names.
fn variants<'p>(cx: &Cx<'p>, ty: &Type) -> Option<(String, &'p [String])> {
    let Type::Named(name) = ty else { return None };
    let local = hirx::local(name).to_owned();
    cx.packages.iter().find_map(|p| {
        hirx::declarations(p).find_map(|d| match d {
            Declaration::Enum(e) if e.name == local => Some((local.clone(), e.variants.as_slice())),
            _ => None,
        })
    })
}

fn event_call(cx: &mut Cx<'_>, call: &ast::CallExpr) -> Option<Expr> {
    let span = cx.node_span(call.syntax());
    let callee = call.callee()?.text();
    let Some(ty) = event_type(cx, &callee) else {
        cx.error(
            C::InvalidCaseEvent,
            span,
            format!("`{callee}` is not an event type of the packages this case uses"),
        );
        return None;
    };
    let args = call.args();
    if args.len() != ty.params.len() {
        cx.error(
            C::TypeMismatch,
            span,
            format!(
                "`{callee}` takes {} arguments, found {}",
                ty.params.len(),
                args.len()
            ),
        );
        return None;
    }
    let mut built = Vec::new();
    for (arg, (param, pty)) in args.iter().zip(&ty.params) {
        let at = cx.node_span(arg.syntax());
        let text = arg
            .reference()
            .map_or_else(|| arg.syntax().text().to_string(), |r| r.text());
        let Some((enum_name, cases)) = variants(cx, pty) else {
            cx.error(
                C::LoweringUnsupported,
                at,
                format!(
                    "parameter `{param}` of `{callee}` is not an enum; only enum arguments replay"
                ),
            );
            return None;
        };
        if !cases.contains(&text) {
            cx.error(
                C::TypeMismatch,
                at,
                format!(
                    "expected a `{enum_name}` variant ({}), found `{text}`",
                    cases.join(", ")
                ),
            );
            return None;
        }
        built.push(Expr {
            ty: pty.clone(),
            kind: ExprKind::Literal {
                value: Value::Variant(format!("{enum_name}::{text}")),
            },
            span: Some(at),
        });
    }
    Some(Expr {
        ty: Type::Named(callee.clone()),
        kind: ExprKind::Call {
            callee,
            args: built,
        },
        span: Some(span),
    })
}

fn events(cx: &mut Cx<'_>, file: &CaseFile, parties: &[hir::Party]) -> Vec<hir::Event> {
    file.events()
        .iter()
        .filter_map(|e| {
            let span = cx.node_span(e.syntax());
            let date = e.date()?.text().to_owned();
            let action = event_call(cx, &e.call()?)?;
            let party = e.party()?.text();
            if !parties.iter().any(|p| p.id == party) {
                cx.error(
                    C::InvalidParty,
                    span,
                    format!("`{party}` is not a declared party"),
                );
                return None;
            }
            let relation = match e.mode().map(|t| t.kind()) {
                Some(SyntaxKind::KwOn) => hir::EventParty::On,
                _ => hir::EventParty::By,
            };
            Some(hir::Event {
                date,
                action,
                relation,
                party,
                authority: e.cite().and_then(|c| c.text()),
            })
        })
        .collect()
}

/// Elaborates the matter; `None` when its header cannot be read.
pub(super) fn case<'a>(
    cx: &mut Cx<'a>,
    session: &'a Session,
    file: &CaseFile,
) -> Option<hir::Case> {
    let id = file.name()?.text().to_owned();
    let pins = uses(cx, session, file);
    file.as_of()?.text().clone_into(&mut cx.as_of);
    if pins.is_empty() {
        return None;
    }
    let start = file.start().and_then(|s| s.target()).map(|r| {
        let q = cx.qualify(&r.text());
        if cx.node(&q).is_none() {
            let span = cx.node_span(r.syntax());
            cx.error(
                C::UnresolvedName,
                span,
                format!("`{q}` is not a node in force on {}", cx.as_of),
            );
        }
        q
    });
    let parties = parties(cx, file);
    let events = events(cx, file, &parties);
    Some(hir::Case {
        id,
        summary: string(file.summary()).unwrap_or_default(),
        notes: string(file.notes()),
        sources: sources(file),
        uses: pins,
        as_of: cx.as_of.clone(),
        known_at: file.known_at().map(|t| t.text().to_owned()),
        start,
        perspective: perspective(cx, file),
        parties,
        events,
        facts: blocks::facts(cx, file),
        stakes: blocks::stakes(cx, file),
        observations: blocks::beliefs(cx, file),
        analysis: blocks::analysis(cx, file),
    })
}
