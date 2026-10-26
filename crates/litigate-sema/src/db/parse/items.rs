//! Module header and declarations (SPEC §3.1).
use super::expr::{declared_type, expr_children, is_expr, path_of};
use super::tree::{
    ArmItem, EdgeItem, HolidayItem, Import, Item, ItemKind, ItemTree, LimitEntry, LimitEntryKind,
    LimitItem, Name, NodeItem, PackageHeader, TestStmt,
};
use super::{
    docs, elements, head, ident, label, name, node_of, nodes_after, nodes_of, range,
    significant_range, token_of, Lower, SyntaxElement,
};
use crate::hir::Measure;
use litigate_syntax::{SyntaxKind as K, SyntaxNode};

/// Identifier tokens directly under `node`, in order.
fn idents(node: &SyntaxNode) -> Vec<Name> {
    elements(node)
        .filter_map(SyntaxElement::into_token)
        .filter(|t| t.kind() == K::Ident)
        .map(|t| name(&t))
        .collect()
}

/// Direct child nodes before the first direct token of kind `stop`.
fn before(node: &SyntaxNode, stop: K) -> impl Iterator<Item = SyntaxNode> {
    node.children_with_tokens()
        .take_while(move |e| e.kind() != stop)
        .filter_map(SyntaxElement::into_node)
}

/// The first keyword token directly under `node` after its leading keyword.
fn second_keyword(node: &SyntaxNode) -> Option<Name> {
    elements(node)
        .filter_map(SyntaxElement::into_token)
        .filter(|t| t.kind().is_keyword())
        .nth(1)
        .map(|t| name(&t))
}

impl Lower {
    pub(crate) fn file(&mut self, root: &SyntaxNode) -> ItemTree {
        let mut tree = ItemTree::default();
        for node in root.children() {
            match node.kind() {
                K::ModuleDecl => {
                    tree.module = node_of(&node, K::Ref).map_or_else(Vec::new, |r| idents(&r));
                }
                K::ImportDecl => tree.imports.extend(import(&node)),
                K::PackageDecl => {
                    if let Some(name) = ident(&node) {
                        tree.package = Some(PackageHeader {
                            name,
                            fields: node_of(&node, K::FieldList)
                                .map(|f| self.field_list(&f))
                                .unwrap_or_default(),
                            range: significant_range(&node),
                        });
                    }
                }
                _ => tree.items.extend(self.item(&node)),
            }
        }
        tree
    }

    /// `grant g on [targets] clauses { fields }`.
    fn grant(&mut self, node: &SyntaxNode) -> Option<ItemKind> {
        Some(ItemKind::Grant(
            ident(node)?,
            self.clauses(node),
            self.field_list(&node_of(node, K::FieldList)?),
        ))
    }

    /// One declaration (top level or inside a `rule`); `None` for nodes that
    /// are not declarations or are missing a required part. Each declaration
    /// is parsed left to right and stops at its first missing part, so only
    /// the parts that can be missing are checked here.
    pub(crate) fn item(&mut self, node: &SyntaxNode) -> Option<Item> {
        let kind = match node.kind() {
            K::StartDecl => ItemKind::Start(path_of(&node_of(node, K::Ref)?)),
            K::GroupDecl => ItemKind::Group(ident(node)?, label(node)?),
            K::TypeDecl => {
                // A declaration's `where` clause names the value `it`.
                let it = Name {
                    text: "it".into(),
                    range: node_of(node, K::WhereClause)
                        .map_or(range(node.text_range()), |c| head(&c).range),
                };
                ItemKind::TypeAlias(ident(node)?, declared_type(node, it)?)
            }
            K::EnumDecl => {
                let mut names = idents(node).into_iter();
                ItemKind::Enum(names.next()?, names.collect())
            }
            K::EventDecl => {
                let name = ident(node)?;
                let params = node
                    .descendants()
                    .filter(|p| p.kind() == K::Param)
                    .map(|p| Some((head(&p), declared_type(&p, head(&p))?)))
                    .collect::<Option<_>>()?;
                ItemKind::Event(name, params, self.clauses(node))
            }
            K::StateDecl | K::DecisionDecl | K::ChanceDecl | K::TerminalDecl => {
                ItemKind::Node(self.node(node)?)
            }
            K::LinkDecl => ItemKind::Link(self.edge(node, true)?),
            K::InstanceDecl => {
                let names = idents(node);
                let fields = node_of(node, K::FieldList)
                    .map(|f| self.field_list(&f))
                    .unwrap_or_default();
                ItemKind::Instance(names.first()?.clone(), names.get(1)?.clone(), fields)
            }
            K::RuleDecl => ItemKind::Rule(
                second_keyword(node)?,
                ident(node)?,
                self.clauses(node),
                node.children().filter_map(|n| self.item(&n)).collect(),
            ),
            K::DeadlineDecl => self.deadline(node)?,
            K::CountingDecl => ItemKind::Counting(
                ident(node)?,
                self.clauses(node),
                self.field_list(&node_of(node, K::FieldList)?),
            ),
            K::CalendarDecl => ItemKind::Calendar(
                ident(node)?,
                self.clauses(node),
                nodes_of(node, K::HolidayDecl)
                    .filter_map(|h| self.holiday(&h))
                    .collect(),
            ),
            K::TransitionDecl => ItemKind::Transition(
                ident(node)?,
                path_of(&node_of(node, K::Ref)?),
                self.clauses(node),
                nodes_of(node, K::TransitionArm)
                    .map(|a| self.arm(&a))
                    .collect::<Option<_>>()?,
            ),
            K::AuthorityDecl => ItemKind::Authority(idents(node)),
            K::DefaultDecl => ItemKind::Default(
                ident(node)?,
                self.clauses_in(before(node, K::KwWhen)),
                self.expr(&nodes_after(node, K::KwWhen).next()?)?,
                self.expr(&nodes_after(node, K::FatArrow).next()?)?,
            ),
            K::ExceptionDecl => self.exception(node)?,
            K::TestDecl => self.test(node)?,
            K::GrantDecl => self.grant(node)?,
            K::LimitDecl => ItemKind::Limit(self.limit(node)?),
            K::RecharacterizeDecl => ItemKind::Recharacterize(
                ident(node)?,
                path_of(&node_of(node, K::Ref)?),
                self.clauses(node),
            ),
            K::InvalidateDecl => ItemKind::Invalidate(
                ident(node)?,
                path_of(&node_of(node, K::Ref)?),
                self.clauses(node),
            ),
            _ => return None,
        };
        Some(Item {
            doc: docs(node),
            kind,
            range: significant_range(node),
        })
    }

    /// `NodeKw Ident ":" String {NodeClause} [NodeBody]`.
    fn node(&mut self, node: &SyntaxNode) -> Option<NodeItem> {
        let mut item = NodeItem {
            keyword: head(node),
            name: ident(node)?,
            label: label(node)?,
            clauses: self.clauses(node),
            fields: Vec::new(),
            edges: Vec::new(),
        };
        for child in node_of(node, K::EdgeBlock)
            .iter()
            .flat_map(SyntaxNode::children)
        {
            match child.kind() {
                K::Field => item.fields.extend(self.field(&child)),
                K::EdgeDecl => item.edges.extend(self.edge(&child, false)),
                _ => {}
            }
        }
        Some(item)
    }

    /// `(Ident | "_") String [":" Ref] "->" Ref {EdgeClause} [EdgeFields]`
    /// (the `: Ref` source only in a `link`). An edge starts with its name
    /// and label; a link without them has no references either.
    fn edge(&mut self, node: &SyntaxNode, link: bool) -> Option<EdgeItem> {
        let first = elements(node)
            .filter_map(SyntaxElement::into_token)
            .find(|t| matches!(t.kind(), K::Ident | K::Underscore))?;
        let mut refs = nodes_of(node, K::Ref).map(|r| path_of(&r));
        let from = if link { Some(refs.next()?) } else { None };
        let (Some(label), Some(to)) = (label(node), refs.next()) else {
            return None;
        };
        Some(EdgeItem {
            name: (first.kind() == K::Ident).then(|| name(&first)),
            name_range: range(first.text_range()),
            label,
            from,
            to,
            clauses: self.clauses(node),
            fields: node_of(node, K::FieldList)
                .map(|f| self.field_list(&f))
                .unwrap_or_default(),
            range: significant_range(node),
        })
    }

    /// `deadline id: amount direction trigger counted_by Ref ...`: the
    /// direction follows the name and the amount, so it alone can be the
    /// first missing part of the header.
    fn deadline(&mut self, node: &SyntaxNode) -> Option<ItemKind> {
        let direction = elements(node)
            .filter_map(SyntaxElement::into_token)
            .find(|t| matches!(t.kind(), K::KwFrom | K::KwAfter | K::KwBefore));
        let amount = expr_children(node).next().and_then(|a| self.expr(&a));
        let (Some(direction), Some(id), Some(amount)) = (direction, ident(node), amount) else {
            return None;
        };
        Some(ItemKind::Deadline {
            name: id,
            amount,
            trigger: self.expr(&nodes_after(node, direction.kind()).next()?)?,
            direction: name(&direction),
            counted_by: path_of(&nodes_after(node, K::KwCountedBy).find(|n| n.kind() == K::Ref)?),
            clauses: self.clauses_in(nodes_after(node, K::KwCountedBy)),
            opts: node_of(node, K::FieldList)
                .map(|f| self.field_list(&f))
                .unwrap_or_default(),
        })
    }

    /// `id ":" CallExpr "effective" Interval ["observed" Ident]`: the
    /// version clause follows the name and the rule.
    fn holiday(&mut self, node: &SyntaxNode) -> Option<HolidayItem> {
        let effective = node_of(node, K::VersionClause)
            .and_then(|v| node_of(&v, K::Interval))
            .and_then(|i| super::fields::interval(&i));
        let rule = expr_children(node).next();
        let (Some(effective), Some(rule)) = (effective, rule) else {
            return None;
        };
        Some(HolidayItem {
            name: head(node),
            rule: self.expr(&rule)?,
            effective,
            observed: node_of(node, K::Field)
                .and_then(|f| node_of(&f, K::Ref))
                .map(|r| path_of(&r).head),
        })
    }

    /// `unless Expr -> Ref` or `otherwise -> Ref`.
    fn arm(&mut self, node: &SyntaxNode) -> Option<ArmItem> {
        let unless = match token_of(node, K::KwUnless) {
            Some(_) => Some(self.expr(&expr_children(node).find(|n| n.kind() != K::Ref)?)?),
            None => None,
        };
        Some(ArmItem {
            unless,
            target: path_of(&nodes_after(node, K::Arrow).find(|n| n.kind() == K::Ref)?),
            range: significant_range(node),
        })
    }

    /// `exception id (rebuts | undercuts) Ref ...`: the kind follows the
    /// name, so a missing name always comes with a missing kind.
    fn exception(&mut self, node: &SyntaxNode) -> Option<ItemKind> {
        let kind = elements(node)
            .filter_map(SyntaxElement::into_token)
            .find(|t| matches!(t.kind(), K::KwRebuts | K::KwUndercuts));
        let (Some(kind), Some(id)) = (kind, ident(node)) else {
            return None;
        };
        let priority = node
            .children()
            .find(|n| n.kind() == K::Field && token_of(n, K::KwPriority).is_some());
        let then = match token_of(node, K::FatArrow) {
            Some(_) => {
                Some(self.expr(&nodes_after(node, K::FatArrow).find(|n| is_expr(n.kind()))?)?)
            }
            None => None,
        };
        let priority = match priority {
            Some(p) => Some(self.first_operand(&p)?),
            None => None,
        };
        Some(ItemKind::Exception {
            name: id,
            kind: name(&kind),
            target: path_of(&nodes_after(node, kind.kind()).find(|n| n.kind() == K::Ref)?),
            clauses: self.clauses_in(before(node, K::KwWhen)),
            when: self.expr(&nodes_after(node, K::KwWhen).next()?)?,
            then,
            priority,
        })
    }

    /// `limit k clauses { entries } clauses`: the clauses may lead or
    /// follow the body. An entry missing its value is left out.
    fn limit(&mut self, node: &SyntaxNode) -> Option<LimitItem> {
        let name = path_of(&node_of(node, K::Ref)?);
        let entries = node_of(node, K::FieldList)?
            .children()
            .filter_map(|e| self.limit_entry(&e))
            .collect();
        Some(LimitItem {
            name,
            clauses: self.clauses(node),
            entries,
        })
    }

    fn limit_entry(&mut self, node: &SyntaxNode) -> Option<LimitEntry> {
        let kind = match node.kind() {
            K::LimitBound => LimitEntryKind::Bound(
                measure(&head(node)),
                value_child(node).and_then(|v| self.expr(&v))?,
            ),
            K::CertificateReq => LimitEntryKind::Certificate(
                path_of(&node_of(node, K::Ref)?),
                node_of(node, K::IdList)
                    .iter()
                    .flat_map(elements)
                    .filter_map(SyntaxElement::into_token)
                    .filter(|t| t.kind().is_keyword())
                    .map(|t| {
                        let n = name(&t);
                        (measure(&n), n.range)
                    })
                    .collect(),
            ),
            K::Exclusions => LimitEntryKind::Excludes(
                node_of(node, K::IdList)?
                    .children()
                    .filter_map(|e| self.expr(&e))
                    .collect(),
            ),
            K::EcfCode => LimitEntryKind::Ecf(value_child(node).and_then(|v| self.expr(&v))?),
            _ => return None,
        };
        Some(LimitEntry {
            kind,
            cites: self.clauses_in(nodes_of(node, K::CiteExpr)),
            range: significant_range(node),
        })
    }

    fn test(&mut self, node: &SyntaxNode) -> Option<ItemKind> {
        let stmts = node
            .children()
            .filter(|s| matches!(s.kind(), K::GivenStmt | K::ExpectStmt))
            .map(|s| self.stmt(&s))
            .collect::<Option<_>>()?;
        Some(ItemKind::Test(label(node)?, self.clauses(node), stmts))
    }

    fn stmt(&mut self, node: &SyntaxNode) -> Option<TestStmt> {
        let value = self.first_operand(node)?;
        if node.kind() == K::ExpectStmt {
            return Some(TestStmt::Expect(value));
        }
        let date = nodes_after(node, K::KwOn)
            .next()
            .and_then(|l| token_of(&l, K::Date))?;
        Some(TestStmt::Given(value, name(&date)))
    }
}

fn import(node: &SyntaxNode) -> Option<Import> {
    Some(Import {
        path: idents(&node_of(node, K::Ref)?),
        names: node_of(node, K::ImportList).map(|l| idents(&l)),
        range: significant_range(node),
    })
}

/// The measure keyword `n` (the grammar admits only `words`, `pages` and
/// `lines` there).
fn measure(n: &Name) -> Measure {
    match n.text.as_str() {
        "words" => Measure::Words,
        "pages" => Measure::Pages,
        _ => Measure::Lines,
    }
}

/// The value of a `limit` body entry: its first expression that is not
/// one of its cites.
fn value_child(node: &SyntaxNode) -> Option<SyntaxNode> {
    expr_children(node).find(|n| n.kind() != K::CiteExpr)
}
