//! Definitions and references across every module and matter of the
//! analysis, read off the lossless CST so they work in files sema rejects.
//!
//! A declaration's name is its first direct `Ident`; a reference is a
//! `Ref`. A reference resolves the way sema's scope reads it (SPEC §4.3):
//! a `pack::x` qualifier picks the package; otherwise the file's own
//! declarations win, then its package's, then any package it can see (a
//! matter sees the packages it `uses`). Edge ids resolve against their
//! node where the grammar fixes one (`facts { node: edge }`,
//! `observe node: { edge: n }`).
use litigate_cli::Checked;
use litigate_syntax::{parse, FileKind, SyntaxKind as K, SyntaxNode, TextRange, TextSize};
use std::collections::BTreeMap;
use std::path::Path;

/// The namespace a definition lives in.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Space {
    /// A top-level declaration (node, deadline, type, event, group, ...).
    Decl,
    /// An edge of the named node.
    Edge(String),
    /// A variant of the named enum.
    Variant(String),
    /// A matter's party.
    Party,
    /// An actor declared in `[package.roles]`.
    Role,
    /// A name only its own declaration uses (parameter, holiday, case id).
    Local,
}

/// One definition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Def {
    /// Declared name.
    pub name: String,
    /// Namespace.
    pub space: Space,
    /// Kind of the declaring node.
    pub kind: K,
    /// Repository-relative file.
    pub file: String,
    /// The name.
    pub name_range: TextRange,
    /// The whole declaration.
    pub full_range: TextRange,
    /// Owning package name (absent for a matter's own names).
    pub package: Option<String>,
}

/// One reference and what it resolves to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Use {
    /// Repository-relative file.
    pub file: String,
    /// The referring identifier.
    pub range: TextRange,
    /// Index into [`Index::defs`].
    pub target: Option<usize>,
}

/// Every definition and reference.
#[derive(Debug, Clone, Default)]
pub struct Index {
    /// Definitions, in file then source order.
    pub defs: Vec<Def>,
    /// References, in file then source order.
    pub uses: Vec<Use>,
    /// Pack id to package name.
    packs: BTreeMap<String, String>,
}

/// What a file sees.
struct FileScope {
    /// Its package (a `.lit` module or the package's `Lit.toml`).
    package: Option<String>,
    /// Packages it may name: its own and dependencies, or a matter's uses.
    visible: Vec<String>,
}

fn scope_of(file: &str, text: &str, checked: &Checked) -> FileScope {
    let own = checked
        .session
        .packages
        .iter()
        .find(|p| p.files.iter().any(|(f, _)| f == file));
    if let Some(p) = own {
        let mut visible = vec![p.name.clone()];
        visible.extend(
            p.check
                .package
                .as_ref()
                .map(|h| h.manifest.dependencies.keys().cloned().collect::<Vec<_>>())
                .unwrap_or_default(),
        );
        return FileScope {
            package: Some(p.name.clone()),
            visible,
        };
    }
    let visible = litigate_syntax::parse_case(text)
        .case_file()
        .and_then(|f| f.uses())
        .map(|u| {
            u.items()
                .iter()
                .filter_map(|i| i.package().map(|t| t.text().to_owned()))
                .collect()
        })
        .unwrap_or_default();
    FileScope {
        package: None,
        visible,
    }
}

const NODES: [K; 4] = [
    K::StateDecl,
    K::DecisionDecl,
    K::ChanceDecl,
    K::TerminalDecl,
];

/// Kinds whose first direct identifier declares a name.
fn declares(kind: K) -> Option<Space> {
    match kind {
        K::GroupDecl
        | K::TypeDecl
        | K::EnumDecl
        | K::EventDecl
        | K::StateDecl
        | K::DecisionDecl
        | K::ChanceDecl
        | K::TerminalDecl
        | K::InstanceDecl
        | K::RuleDecl
        | K::DeadlineDecl
        | K::CountingDecl
        | K::CalendarDecl
        | K::TransitionDecl
        | K::AuthorityDecl
        | K::DefaultDecl
        | K::ExceptionDecl => Some(Space::Decl),
        K::PartyDecl => Some(Space::Party),
        K::Param | K::HolidayDecl | K::CaseFile => Some(Space::Local),
        K::EdgeDecl => Some(Space::Edge(String::new())),
        _ => None,
    }
}

/// The direct identifiers of `node`.
fn idents(node: &SyntaxNode) -> Vec<litigate_syntax::SyntaxToken> {
    node.children_with_tokens()
        .filter_map(rowan::NodeOrToken::into_token)
        .filter(|t| t.kind() == K::Ident)
        .collect()
}

/// The name of the node declaration enclosing an edge.
fn owner(node: &SyntaxNode) -> String {
    node.ancestors()
        .find(|a| NODES.contains(&a.kind()))
        .and_then(|a| idents(&a).first().map(|t| t.text().to_owned()))
        .unwrap_or_default()
}

/// The last identifier of a `Ref` (the name) and its `pack::` qualifier.
fn ref_name(node: &SyntaxNode) -> Option<(litigate_syntax::SyntaxToken, Option<String>)> {
    let ids = idents(node);
    let qualified = node
        .children_with_tokens()
        .any(|t| t.kind() == K::ColonColon);
    let qualifier = qualified
        .then(|| ids.first().map(|t| t.text().to_owned()))
        .flatten();
    Some((ids.last()?.clone(), qualifier))
}

/// The node an edge reference is fixed to by its context, if any.
fn edge_context(r: &SyntaxNode) -> Option<String> {
    let field = r.parent().filter(|p| p.kind() == K::RecordField)?;
    let block = field.parent()?.parent()?;
    let refs: Vec<SyntaxNode> = field.children().filter(|c| c.kind() == K::Ref).collect();
    let name = |n: &SyntaxNode| ref_name(n).map(|(t, _)| t.text().to_owned());
    match block.kind() {
        K::FactsBlock if refs.len() == 2 && refs[1] == *r => name(&refs[0]),
        K::ObserveDecl => block
            .children()
            .find(|c| c.kind() == K::Ref)
            .and_then(|n| name(&n)),
        _ => None,
    }
}

/// `[package.roles]` keys of a manifest, with their byte ranges.
fn roles(text: &str) -> Vec<(String, TextRange)> {
    let mut inside = false;
    let mut offset = 0;
    let mut out = Vec::new();
    for line in text.split_inclusive('\n') {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            inside = trimmed == "[package.roles]";
        } else if let Some((key, _)) = trimmed.split_once('=').filter(|_| inside) {
            let key = key.trim();
            let start = offset + line.find(key).unwrap_or(0);
            let range = TextRange::at(
                TextSize::try_from(start).unwrap_or_default(),
                TextSize::try_from(key.len()).unwrap_or_default(),
            );
            out.push((key.to_owned(), range));
        }
        offset += line.len();
    }
    out
}

impl Index {
    /// Indexes `files` against the packages of `checked`.
    #[must_use]
    pub fn build(files: &BTreeMap<String, String>, checked: &Checked) -> Self {
        let mut index = Self {
            packs: checked
                .session
                .hir()
                .iter()
                .map(|p| (p.manifest.pack_id.clone(), p.manifest.name.clone()))
                .collect(),
            ..Self::default()
        };
        let mut refs: Vec<(String, SyntaxNode)> = Vec::new();
        let mut scopes: BTreeMap<String, FileScope> = BTreeMap::new();
        for (file, text) in files {
            let scope = scope_of(file, text, checked);
            if file.ends_with("Lit.toml") {
                index
                    .defs
                    .extend(roles(text).into_iter().map(|(name, range)| Def {
                        name,
                        space: Space::Role,
                        kind: K::PackageDecl,
                        file: file.clone(),
                        name_range: range,
                        full_range: range,
                        package: scope.package.clone(),
                    }));
            }
            if let Some(kind) = FileKind::from_path(Path::new(file)) {
                let root = parse(text, kind).syntax();
                for node in root.descendants() {
                    if node.kind() == K::Ref {
                        refs.push((file.clone(), node));
                        continue;
                    }
                    index.declare(&node, file, &scope);
                }
            }
            scopes.insert(file.clone(), scope);
        }
        for (file, node) in refs {
            let Some((token, qualifier)) = ref_name(&node) else {
                continue;
            };
            let scope = &scopes[&file];
            let target = match edge_context(&node) {
                Some(owner) => index.find(token.text(), &file, scope, None, |s| {
                    *s == Space::Edge(owner.clone())
                }),
                None => index.resolve(token.text(), &file, scope, qualifier.as_deref()),
            };
            index.uses.push(Use {
                file,
                range: token.text_range(),
                target,
            });
        }
        index
    }

    fn declare(&mut self, node: &SyntaxNode, file: &str, scope: &FileScope) {
        let Some(space) = declares(node.kind()) else {
            return;
        };
        let ids = idents(node);
        let def = |t: &litigate_syntax::SyntaxToken, space: Space| Def {
            name: t.text().to_owned(),
            space,
            kind: node.kind(),
            file: file.to_owned(),
            name_range: t.text_range(),
            full_range: node.text_range(),
            package: scope.package.clone(),
        };
        let found: Vec<Def> = match (node.kind(), ids.split_first()) {
            (_, None) => Vec::new(),
            (K::EnumDecl, Some((name, variants))) => std::iter::once(def(name, Space::Decl))
                .chain(variants.iter().map(|v| Def {
                    full_range: v.text_range(),
                    ..def(v, Space::Variant(name.text().to_owned()))
                }))
                .collect(),
            (K::AuthorityDecl, _) => ids
                .iter()
                .map(|t| Def {
                    full_range: t.text_range(),
                    ..def(t, Space::Decl)
                })
                .collect(),
            (K::EdgeDecl, Some((name, _))) => vec![def(name, Space::Edge(owner(node)))],
            (_, Some((name, _))) => vec![def(name, space)],
        };
        self.defs.extend(found);
    }

    /// Best definition named `name` in `space`s accepted by `want`, seen
    /// from `file`: same file, then same package, then visible packages.
    fn find(
        &self,
        name: &str,
        file: &str,
        scope: &FileScope,
        qualifier: Option<&str>,
        want: impl Fn(&Space) -> bool,
    ) -> Option<usize> {
        self.defs
            .iter()
            .enumerate()
            .filter(|(_, d)| d.name == name && want(&d.space))
            .filter_map(|(i, d)| {
                let same_package = d.package.is_some() && d.package == scope.package;
                let visible = d
                    .package
                    .as_ref()
                    .is_some_and(|p| scope.visible.contains(p));
                let rank = if let Some(q) = qualifier {
                    (d.package.as_deref() == Some(q)).then_some(1)?
                } else if d.file == file {
                    0
                } else if same_package {
                    1
                } else if visible {
                    2
                } else {
                    return None;
                };
                Some((rank, i))
            })
            .min()
            .map(|(_, i)| i)
    }

    /// Resolves an unqualified or `pack::`-qualified name: declarations,
    /// then parties, enum variants, roles, and finally any edge.
    fn resolve(
        &self,
        name: &str,
        file: &str,
        scope: &FileScope,
        qualifier: Option<&str>,
    ) -> Option<usize> {
        let qualifier = qualifier.map(|q| self.package_named(q));
        let qualifier = qualifier.as_deref();
        [
            |s: &Space| *s == Space::Decl,
            |s: &Space| *s == Space::Party,
            |s: &Space| matches!(s, Space::Variant(_)),
            |s: &Space| *s == Space::Role,
            |s: &Space| matches!(s, Space::Edge(_)),
        ]
        .into_iter()
        .find_map(|want| self.find(name, file, scope, qualifier, want))
    }

    /// Package name for a qualifier: names pass through; pack ids map back
    /// through `package` declarations.
    fn package_named(&self, qualifier: &str) -> String {
        self.packs
            .get(qualifier)
            .cloned()
            .unwrap_or_else(|| qualifier.to_owned())
    }

    /// The definition at byte `offset` of `file`: a name being declared,
    /// or the target of a reference there.
    #[must_use]
    pub fn at(&self, file: &str, offset: TextSize) -> Option<usize> {
        let inside = |r: &TextRange| r.contains_inclusive(offset);
        self.defs
            .iter()
            .position(|d| d.file == file && inside(&d.name_range))
            .or_else(|| {
                self.uses
                    .iter()
                    .find(|u| u.file == file && inside(&u.range))
                    .and_then(|u| u.target)
            })
    }

    /// Every reference to definition `def`.
    pub fn references(&self, def: usize) -> impl Iterator<Item = &Use> {
        self.uses.iter().filter(move |u| u.target == Some(def))
    }
}
