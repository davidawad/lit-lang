//! Module header and declarations (SPEC §3.1).
use super::expr::{expr_children, is_expr, ref_path, type_ref};
use super::tree::{
    ArmItem, EdgeItem, HolidayItem, Import, Item, ItemKind, ItemTree, Name, NodeItem,
    PackageHeader, TestStmt,
};
use super::{
    docs, elements, ident, label, name, node_of, nodes_after, nodes_of, range, significant_range,
    token_of, Lower, SyntaxElement,
};
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

    /// One declaration (top level or inside a `rule`); `None` for nodes that
    /// are not declarations or are missing a required part.
    pub(crate) fn item(&mut self, node: &SyntaxNode) -> Option<Item> {
        let kind = match node.kind() {
            K::StartDecl => ItemKind::Start(ref_path(&node_of(node, K::Ref)?)?),
            K::GroupDecl => ItemKind::Group(ident(node)?, label(node)?),
            K::TypeDecl => {
                ItemKind::TypeAlias(ident(node)?, type_ref(&node_of(node, K::TypeExpr)?)?)
            }
            K::EnumDecl => {
                let mut names = idents(node).into_iter();
                ItemKind::Enum(names.next()?, names.collect())
            }
            K::EventDecl => {
                let params = node_of(node, K::ParamList)?;
                let params = nodes_of(&params, K::Param)
                    .map(|p| Some((ident(&p)?, type_ref(&node_of(&p, K::TypeExpr)?)?)))
                    .collect::<Option<_>>()?;
                ItemKind::Event(ident(node)?, params)
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
                ref_path(&node_of(node, K::Ref)?)?,
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
            _ => return None,
        };
        Some(Item {
            doc: docs(node),
            kind,
            range: significant_range(node),
        })
    }

    fn node(&mut self, node: &SyntaxNode) -> Option<NodeItem> {
        let keyword = elements(node).find_map(SyntaxElement::into_token)?;
        let body = node_of(node, K::EdgeBlock);
        let mut item = NodeItem {
            keyword: name(&keyword),
            name: ident(node)?,
            label: label(node)?,
            clauses: self.clauses(node),
            fields: Vec::new(),
            edges: Vec::new(),
        };
        for child in body.iter().flat_map(SyntaxNode::children) {
            match child.kind() {
                K::Field => item.fields.extend(self.field(&child)),
                K::EdgeDecl => item.edges.extend(self.edge(&child, false)),
                _ => {}
            }
        }
        Some(item)
    }

    /// `(Ident | "_") String ["{:} Ref] "->" Ref {EdgeClause} [EdgeFields]`.
    fn edge(&mut self, node: &SyntaxNode, link: bool) -> Option<EdgeItem> {
        let first = elements(node)
            .filter_map(SyntaxElement::into_token)
            .find(|t| matches!(t.kind(), K::Ident | K::Underscore))?;
        let mut refs = nodes_of(node, K::Ref);
        let from = if link {
            Some(ref_path(&refs.next()?)?)
        } else {
            None
        };
        Some(EdgeItem {
            name: (first.kind() == K::Ident).then(|| name(&first)),
            name_range: range(first.text_range()),
            label: label(node)?,
            from,
            to: ref_path(&refs.next()?)?,
            clauses: self.clauses(node),
            fields: node_of(node, K::FieldList)
                .map(|f| self.field_list(&f))
                .unwrap_or_default(),
            range: significant_range(node),
        })
    }

    fn deadline(&mut self, node: &SyntaxNode) -> Option<ItemKind> {
        let direction = elements(node)
            .filter_map(SyntaxElement::into_token)
            .find(|t| matches!(t.kind(), K::KwFrom | K::KwAfter | K::KwBefore))?;
        Some(ItemKind::Deadline {
            name: ident(node)?,
            amount: self.expr(&expr_children(node).next()?)?,
            trigger: self.expr(&nodes_after(node, direction.kind()).next()?)?,
            direction: name(&direction),
            counted_by: ref_path(&nodes_after(node, K::KwCountedBy).find(|n| n.kind() == K::Ref)?)?,
            clauses: self.clauses_in(nodes_after(node, K::KwCountedBy)),
            opts: node_of(node, K::FieldList)
                .map(|f| self.field_list(&f))
                .unwrap_or_default(),
        })
    }

    fn holiday(&mut self, node: &SyntaxNode) -> Option<HolidayItem> {
        let effective = node_of(node, K::VersionClause)
            .and_then(|v| node_of(&v, K::Interval))
            .and_then(|i| super::fields::interval(&i))?;
        Some(HolidayItem {
            name: ident(node)?,
            rule: self.expr(&expr_children(node).next()?)?,
            effective,
            observed: node_of(node, K::Field)
                .and_then(|f| node_of(&f, K::Ref))
                .and_then(|r| ref_path(&r))
                .map(|r| r.head),
        })
    }

    fn arm(&mut self, node: &SyntaxNode) -> Option<ArmItem> {
        let unless = if token_of(node, K::KwUnless).is_some() {
            Some(self.expr(&expr_children(node).find(|n| n.kind() != K::Ref)?)?)
        } else {
            token_of(node, K::KwOtherwise)?;
            None
        };
        Some(ArmItem {
            unless,
            target: ref_path(&nodes_after(node, K::Arrow).find(|n| n.kind() == K::Ref)?)?,
            range: significant_range(node),
        })
    }

    fn exception(&mut self, node: &SyntaxNode) -> Option<ItemKind> {
        let kind = elements(node)
            .filter_map(SyntaxElement::into_token)
            .find(|t| matches!(t.kind(), K::KwRebuts | K::KwUndercuts))?;
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
            Some(p) => Some(self.expr(&expr_children(&p).next()?)?),
            None => None,
        };
        Some(ItemKind::Exception {
            name: ident(node)?,
            kind: name(&kind),
            target: ref_path(&nodes_after(node, kind.kind()).find(|n| n.kind() == K::Ref)?)?,
            clauses: self.clauses_in(before(node, K::KwWhen)),
            when: self.expr(&nodes_after(node, K::KwWhen).next()?)?,
            then,
            priority,
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
        let value = self.expr(&expr_children(node).next()?)?;
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
