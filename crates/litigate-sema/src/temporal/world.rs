//! The declaration index the temporal and precedence checks run over: every
//! rule-bearing declaration of a package and its dependencies, keyed by
//! `(pack id, local id)`, with version sets and source locations.
use super::interval::Window;
use super::version::{Version, VersionSet};
use crate::hir::{self, Declaration, Metadata, Relation, RuleKind};
use litigate_base::{Applicability, Diagnostic, DiagnosticCode, FileId, Fix, Note, Span};
use std::collections::BTreeMap;

/// Package-independent identity of a logical declaration.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Key {
    /// Owning pack id (`Lit.toml` `pack_id`).
    pub pack: String,
    /// Local declaration id.
    pub name: String,
}

impl Key {
    /// Spelling as seen from package `pack` (local ids unqualified).
    #[must_use]
    pub fn display(&self, pack: &str) -> String {
        if self.pack == pack {
            self.name.clone()
        } else {
            format!("{}::{}", self.pack, self.name)
        }
    }
}

/// A relation or reference target: declaration plus optional `@date`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Target {
    /// The logical declaration.
    pub key: Key,
    /// The version selected by `@from-date`, if any.
    pub version: Option<String>,
}

/// One package's place in the world.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackInfo {
    /// Package name.
    pub name: String,
    /// Pack id.
    pub pack: String,
    /// Default authority level.
    pub authority: String,
    /// The package lifetime (`in_force` to `in_force_until`).
    pub lifetime: Window,
}

/// One declaration with its context.
#[derive(Debug, Clone, Copy)]
pub struct Decl<'a> {
    /// Owning package (index into [`World::packs`]).
    pub pack: usize,
    /// The declaration.
    pub decl: &'a Declaration,
    /// Its metadata.
    pub meta: &'a Metadata,
    /// The enclosing `rule`, if nested in one.
    pub rule: Option<(&'a str, RuleKind)>,
}

/// An authority chain link `above > below` and where it was declared.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rank {
    /// The higher level.
    pub above: String,
    /// The lower level.
    pub below: String,
    /// Owning package (index into [`World::packs`]).
    pub pack: usize,
}

/// Where a declaration's clauses are in source; implemented over the salsa
/// item trees, or not at all for HIR-only worlds.
pub trait Locate {
    /// The `key` clause of `d` and its first value, when written.
    fn clause(&self, d: &Decl<'_>, key: &str) -> Option<(Span, Option<Span>)>;
    /// An empty span at the end of `d`'s source.
    fn end(&self, d: &Decl<'_>) -> Option<Span>;
    /// The `authority` chain declaring `above > below`.
    fn rank(&self, rank: &Rank) -> Option<Span>;
}

/// A [`Locate`] that knows nothing (spans fall back to declaration spans).
#[derive(Debug, Clone, Copy, Default)]
pub struct NoSource;

impl Locate for NoSource {
    fn clause(&self, _: &Decl<'_>, _: &str) -> Option<(Span, Option<Span>)> {
        None
    }
    fn end(&self, _: &Decl<'_>) -> Option<Span> {
        None
    }
    fn rank(&self, _: &Rank) -> Option<Span> {
        None
    }
}

/// Every declaration of a set of packages.
#[derive(Debug, Default)]
pub struct World<'a> {
    /// Packages in load order.
    pub packs: Vec<PackInfo>,
    /// Declarations with metadata, in package/module/source order.
    pub decls: Vec<Decl<'a>>,
    /// Authority order links.
    pub ranks: Vec<Rank>,
    by_key: BTreeMap<Key, Vec<usize>>,
    incoming: BTreeMap<Key, Vec<Incoming<'a>>>,
    exceptions: BTreeMap<Key, Vec<usize>>,
    transitions: BTreeMap<(Key, String), usize>,
}

/// A relation pointing at a declaration: who declares it, and on which
/// version (`@D`) if any.
#[derive(Debug, Clone)]
pub struct Incoming<'a> {
    /// The declaring declaration (index into [`World::decls`]).
    pub by: usize,
    /// The relation.
    pub relation: &'a Relation,
    /// The `@D` version selector of the target, if any.
    pub version: Option<String>,
}

/// The target text of a relation that names a declaration.
#[must_use]
pub fn relation_target(r: &Relation) -> Option<&String> {
    match r {
        Relation::SubjectTo(t)
        | Relation::Notwithstanding(t)
        | Relation::Supersedes(t)
        | Relation::Revives(t)
        | Relation::Override(t) => Some(t),
        Relation::DelegableTo(_) => None,
    }
}

/// The `rule` enclosing the declaration with `meta`: the rule whose source
/// span contains it, or (for HIR without spans) the rule that lists its name.
fn enclosing<'r>(rules: &[&'r hir::Rule], meta: &Metadata) -> Option<(&'r str, RuleKind)> {
    let inside = |outer: &Span, inner: &Span| {
        outer.file == inner.file
            && outer != inner
            && (outer.line, outer.col) <= (inner.line, inner.col)
            && (inner.end_line, inner.end_col) <= (outer.end_line, outer.end_col)
    };
    rules
        .iter()
        .find(|r| match (&r.meta.span, &meta.span) {
            (Some(outer), Some(inner)) => inside(outer, inner),
            _ => r.declarations.contains(&meta.id),
        })
        .map(|r| (r.meta.id.as_str(), r.rule_kind))
}

/// The metadata of a rule-bearing declaration.
#[must_use]
pub fn meta_of(d: &Declaration) -> Option<&Metadata> {
    match d {
        Declaration::Node(n) => Some(&n.meta),
        Declaration::Deadline(x) => Some(&x.meta),
        Declaration::Counting(x) => Some(&x.meta),
        Declaration::Calendar(x) => Some(&x.meta),
        Declaration::Rule(x) => Some(&x.meta),
        Declaration::Default(x) => Some(&x.meta),
        Declaration::Exception(x) => Some(&x.meta),
        Declaration::Transition(x) => Some(&x.meta),
        _ => None,
    }
}

impl<'a> World<'a> {
    /// Adds package `info` with `modules`.
    pub fn add(&mut self, info: PackInfo, modules: impl IntoIterator<Item = &'a hir::Module>) {
        let pack = self.packs.len();
        self.packs.push(info);
        for module in modules {
            let rules: Vec<&hir::Rule> = module
                .declarations
                .iter()
                .filter_map(|d| match d {
                    Declaration::Rule(r) => Some(r),
                    _ => None,
                })
                .collect();
            for decl in &module.declarations {
                if let Declaration::Authority(a) = decl {
                    if let Some(below) = a.outranks.first() {
                        self.ranks.push(Rank {
                            above: a.name.clone(),
                            below: below.clone(),
                            pack,
                        });
                    }
                }
                let Some(meta) = meta_of(decl) else {
                    continue;
                };
                let key = Key {
                    pack: self.packs[pack].pack.clone(),
                    name: meta.id.clone(),
                };
                let i = self.decls.len();
                let own = key.pack.clone();
                self.by_key.entry(key.clone()).or_default().push(i);
                for relation in &meta.relations {
                    if let Some(text) = relation_target(relation) {
                        let t = self.target(&own, text);
                        self.incoming.entry(t.key).or_default().push(Incoming {
                            by: i,
                            relation,
                            version: t.version,
                        });
                    }
                }
                match decl {
                    Declaration::Exception(e) => {
                        let t = self.target(&own, &e.target).key;
                        self.exceptions.entry(t).or_default().push(i);
                    }
                    Declaration::Transition(t) => {
                        let a = self.target(&own, &t.amendment);
                        if let Some(v) = a.version {
                            self.transitions.entry((a.key, v)).or_insert(i);
                        }
                    }
                    _ => {}
                }
                self.decls.push(Decl {
                    pack,
                    decl,
                    meta,
                    rule: enclosing(&rules, meta),
                });
            }
        }
    }

    /// A world over compiled packages (no source locations).
    #[must_use]
    pub fn from_packages(packages: &[&'a hir::Package]) -> Self {
        let mut world = Self::default();
        for p in packages {
            world.add(PackInfo::of(&p.manifest), &p.modules);
        }
        world
    }

    /// Relations that name `key`, in declaration order.
    #[must_use]
    pub fn incoming(&self, key: &Key) -> &[Incoming<'a>] {
        self.incoming.get(key).map_or(&[], Vec::as_slice)
    }

    /// Every declaration some relation names.
    pub fn targets(&self) -> impl Iterator<Item = (&Key, &[Incoming<'a>])> + '_ {
        self.incoming.iter().map(|(k, v)| (k, v.as_slice()))
    }

    /// Exceptions (declaration indexes) whose target is `key`.
    #[must_use]
    pub fn exceptions_on(&self, key: &Key) -> &[usize] {
        self.exceptions.get(key).map_or(&[], Vec::as_slice)
    }

    /// Every exception target.
    pub fn exception_targets(&self) -> impl Iterator<Item = &Key> + '_ {
        self.exceptions.keys()
    }

    /// The first transition amending `key@from`.
    #[must_use]
    pub fn transition_for(&self, key: &Key, from: &str) -> Option<usize> {
        self.transitions
            .get(&(key.clone(), from.to_owned()))
            .copied()
    }

    /// The key of declaration `i`.
    #[must_use]
    pub fn key(&self, i: usize) -> Key {
        Key {
            pack: self.packs[self.decls[i].pack].pack.clone(),
            name: self.decls[i].meta.id.clone(),
        }
    }

    /// Every logical declaration and its versions (declaration indexes).
    pub fn sets(&self) -> impl Iterator<Item = (&Key, VersionSet<usize>)> + '_ {
        self.by_key
            .iter()
            .map(|(k, ids)| (k, self.versions_of(ids)))
    }

    /// The versions of `key` (empty when unknown).
    #[must_use]
    pub fn versions(&self, key: &Key) -> VersionSet<usize> {
        self.by_key
            .get(key)
            .map_or_else(VersionSet::default, |ids| self.versions_of(ids))
    }

    fn versions_of(&self, ids: &[usize]) -> VersionSet<usize> {
        VersionSet::new(
            ids.iter()
                .map(|&i| Version {
                    in_force: Window::of(&self.decls[i].meta.effective),
                    known_at: self.decls[i].meta.effective.known_at.clone(),
                    decl: i,
                })
                .collect(),
        )
    }

    /// Resolves a canonical reference written in pack `pack` (`x`, `x@D`,
    /// `q::x`, `q::x@D`).
    #[must_use]
    pub fn target(&self, pack: &str, text: &str) -> Target {
        let (base, version) = match text.rsplit_once('@') {
            Some((b, d)) if crate::types::date::is_valid(d) => (b, Some(d.to_owned())),
            _ => (text, None),
        };
        let key = match base.split_once("::") {
            Some((p, n)) => Key {
                pack: p.to_owned(),
                name: n.to_owned(),
            },
            None => Key {
                pack: pack.to_owned(),
                name: base.to_owned(),
            },
        };
        Target { key, version }
    }

    /// The pack id of declaration `i`.
    #[must_use]
    pub fn pack_of(&self, i: usize) -> &str {
        &self.packs[self.decls[i].pack].pack
    }

    /// Declaration `i`'s spelling as seen from declaration `from`.
    #[must_use]
    pub fn name(&self, i: usize, from: usize) -> String {
        self.key(i).display(self.pack_of(from))
    }

    /// Declaration `i`'s spelling with its version, `x@from`.
    #[must_use]
    pub fn versioned(&self, i: usize, from: usize) -> String {
        format!(
            "{}@{}",
            self.name(i, from),
            self.decls[i].meta.effective.from
        )
    }
}

impl PackInfo {
    /// The world entry of a manifest.
    #[must_use]
    pub fn of(m: &hir::Manifest) -> Self {
        Self {
            name: m.name.clone(),
            pack: m.pack_id.clone(),
            authority: m.authority.clone(),
            lifetime: Window::of(&m.in_force),
        }
    }
}

/// The best span for declaration `d`: its `key` clause, else the whole
/// declaration.
#[must_use]
pub fn span_of(loc: &dyn Locate, d: &Decl<'_>, key: &str) -> Span {
    loc.clause(d, key)
        .map(|(s, _)| s)
        .or_else(|| d.meta.span.clone())
        .unwrap_or_else(|| nowhere(&d.meta.id))
}

/// A placeholder span for HIR without source locations.
#[must_use]
pub fn nowhere(label: &str) -> Span {
    Span {
        file: FileId(label.to_owned()),
        line: 1,
        col: 1,
        end_line: 1,
        end_col: 1,
    }
}

/// An error diagnostic.
#[must_use]
pub fn error(code: DiagnosticCode, span: Span, message: impl Into<String>) -> Diagnostic {
    Diagnostic::error(code, span, message)
}

/// Builder helpers on [`Diagnostic`].
pub trait DiagnosticExt {
    /// Adds a note, optionally located.
    #[must_use]
    fn with_note(self, span: Option<Span>, message: impl Into<String>) -> Self;
    /// Adds a fix.
    #[must_use]
    fn with_fix(self, span: Span, replacement: impl Into<String>, a: Applicability) -> Self;
}

impl DiagnosticExt for Diagnostic {
    fn with_note(mut self, span: Option<Span>, message: impl Into<String>) -> Self {
        self.notes.push(Note {
            span,
            message: message.into(),
        });
        self
    }

    fn with_fix(mut self, span: Span, replacement: impl Into<String>, a: Applicability) -> Self {
        self.fixes.push(Fix {
            span,
            replacement: replacement.into(),
            applicability: a,
        });
        self
    }
}
