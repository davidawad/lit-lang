//! The item tree: sema's syntax-independent view of one `.lit` file,
//! lowered from the `litigate-syntax` CST. Every later query consumes only
//! this, so CST trivia and recovery details never reach name resolution or
//! type checking.
use crate::db::span::TextRange;
use crate::hir::{BinaryOp, Measure, UnaryOp};

/// Identifier (or date/text atom) with its range.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Name {
    /// Source text.
    pub text: String,
    /// Range.
    pub range: TextRange,
}

/// Separator in a reference path.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PathSep {
    /// `.` (module path or field).
    Dot,
    /// `::` (pack or instance qualifier).
    Colons,
}

/// `Ident ["@" Ident] {("::" | ".") Ident} ["@" Date]`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RefPath {
    /// First identifier.
    pub head: Name,
    /// `@origin` instance suffix on the head.
    pub instance: Option<Name>,
    /// Remaining segments.
    pub rest: Vec<(PathSep, Name)>,
    /// `@date` version selector.
    pub version: Option<Name>,
    /// Whole range.
    pub range: TextRange,
}

impl RefPath {
    /// The final segment.
    #[must_use]
    pub fn last(&self) -> &Name {
        self.rest.last().map_or(&self.head, |(_, n)| n)
    }

    /// Source spelling without the version suffix.
    #[must_use]
    pub fn display(&self) -> String {
        let mut s = self.head.text.clone();
        if let Some(i) = &self.instance {
            s.push('@');
            s.push_str(&i.text);
        }
        for (sep, n) in &self.rest {
            s.push_str(if *sep == PathSep::Dot { "." } else { "::" });
            s.push_str(&n.text);
        }
        s
    }

    /// True for a single plain identifier.
    #[must_use]
    pub fn is_simple(&self) -> bool {
        self.instance.is_none() && self.rest.is_empty() && self.version.is_none()
    }
}

/// Expression with range.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Expr {
    /// Shape.
    pub kind: ExprKind,
    /// Range.
    pub range: TextRange,
}

/// Expression shapes (SPEC §3.3).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExprKind {
    /// Decimal literal text.
    Number(String),
    /// Money literal text (`2_000.usd`).
    Money(String),
    /// Duration literal text (`21.days`).
    Duration(String),
    /// String value (unescaped).
    Str(String),
    /// Date text.
    Date(String),
    /// `true` / `false`.
    Bool(bool),
    /// `unit`.
    Unit,
    /// `?` / `?("reason")`.
    Hole(Option<String>),
    /// `cite "..." [historical]`: the citation string and whether it is
    /// marked `historical`.
    Cite(String, bool),
    /// Name reference.
    Ref(RefPath),
    /// Call.
    Call(RefPath, Vec<Expr>),
    /// Unary operator.
    Unary(UnaryOp, Box<Expr>),
    /// Binary operator.
    Binary(BinaryOp, Box<Expr>, Box<Expr>),
    /// Parenthesized.
    Paren(Box<Expr>),
    /// `{ f: e, ... }`.
    Record(Vec<(Name, Expr)>),
    /// `[e, ...]`.
    List(Vec<Expr>),
    /// Projection `e.f` on a non-path base.
    Field(Box<Expr>, Name),
    /// `match e { p => e }`.
    Match(Box<Expr>, Vec<(Pattern, Expr)>),
}

/// Match pattern.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Pattern {
    /// Variant reference.
    Ref(RefPath),
    /// `_`.
    Wildcard(TextRange),
}

/// Type expression `Name<Args>`, with the predicates refining it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeRef {
    /// Constructor name.
    pub name: Name,
    /// Arguments.
    pub args: Vec<TypeRef>,
    /// Range.
    pub range: TextRange,
    /// Refinements (SPEC §3.5), innermost first: `{x: T | p}` around the
    /// base, then a `where p` clause after it.
    pub refinements: Vec<Qualifier>,
}

/// One refinement predicate: `p` of `{x: T | p}` (binder `x`) or of a
/// `where p` clause (binder the parameter, or `it` in a type
/// declaration).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Qualifier {
    /// The name the predicate gives the refined value.
    pub binder: Name,
    /// The predicate.
    pub predicate: Expr,
    /// The whole form: `{x: T | p}` or `where p`.
    pub range: TextRange,
}

/// One field value atom.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Atom {
    /// Expression-shaped value (literals, refs, lists, records, calls).
    Expr(Expr),
    /// Nested field list (`{ extendable true }`, triangular durations).
    Fields(Vec<Field>),
    /// `[from, to)` interval; `to` is `None` for `..`.
    Interval(Name, Option<Name>),
}

/// `key value...` (also used for clauses such as `by X`, `cite ...`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Field {
    /// Key.
    pub key: Name,
    /// Values in order with their ranges.
    pub values: Vec<(Atom, TextRange)>,
    /// Whole range.
    pub range: TextRange,
}

/// Edge or link declaration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EdgeItem {
    /// Id; `None` for `_`.
    pub name: Option<Name>,
    /// Range of the id or `_`.
    pub name_range: TextRange,
    /// Label.
    pub label: Name,
    /// Link source (links only).
    pub from: Option<RefPath>,
    /// Target.
    pub to: RefPath,
    /// `by`/`p` clauses.
    pub clauses: Vec<Field>,
    /// Braced fields.
    pub fields: Vec<Field>,
    /// Range.
    pub range: TextRange,
}

/// `state|decision|chance|terminal` declaration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeItem {
    /// Keyword.
    pub keyword: Name,
    /// Id.
    pub name: Name,
    /// Label.
    pub label: Name,
    /// Header clauses.
    pub clauses: Vec<Field>,
    /// Body fields.
    pub fields: Vec<Field>,
    /// Outgoing edges.
    pub edges: Vec<EdgeItem>,
}

/// Holiday line inside a calendar.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HolidayItem {
    /// Id.
    pub name: Name,
    /// Date rule call.
    pub rule: Expr,
    /// `effective` interval.
    pub effective: (Name, Option<Name>),
    /// `observed` kind.
    pub observed: Option<Name>,
}

/// Transition arm.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArmItem {
    /// `unless` condition; `None` for `otherwise`.
    pub unless: Option<Expr>,
    /// Selected version.
    pub target: RefPath,
    /// Range.
    pub range: TextRange,
}

/// `limit k clauses { entries } clauses` (SPEC §3.7).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LimitItem {
    /// The paper kind, as written (a plain identifier declares one).
    pub name: RefPath,
    /// Version, relation and cite clauses on either side of the body.
    pub clauses: Vec<Field>,
    /// Body entries in source order.
    pub entries: Vec<LimitEntry>,
}

/// One entry of a `limit` body, with its own cites.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LimitEntry {
    /// What the entry says.
    pub kind: LimitEntryKind,
    /// Its `cite` clauses.
    pub cites: Vec<Field>,
    /// Range.
    pub range: TextRange,
}

/// The shapes of a `limit` body entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LimitEntryKind {
    /// `words|pages|lines <= N` (`N` a number or a hole).
    Bound(Measure, Expr),
    /// `requires certificate c [for [measures]]`, each measure with its
    /// range (none without `for`).
    Certificate(RefPath, Vec<(Measure, TextRange)>),
    /// `excludes [item, ?("reason"), ...]`.
    Excludes(Vec<Expr>),
    /// `ecf "code"` or `ecf ?("reason")`.
    Ecf(Expr),
}

/// Test statement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TestStmt {
    /// `given e [by actor] on date`.
    Given(Expr, Option<RefPath>, Name),
    /// `expect e`.
    Expect(Expr),
}

/// Declaration kinds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ItemKind {
    /// `start x`.
    Start(RefPath),
    /// `group g: "Label"`.
    Group(Name, Name),
    /// `type T = U`.
    TypeAlias(Name, TypeRef),
    /// `enum E { a, b }`.
    Enum(Name, Vec<Name>),
    /// `event E(p: T) [effect]`: the effect clause as fields
    /// (`removes p`, `extends d`, `to p`, `waives d`).
    Event(Name, Vec<(Name, TypeRef)>, Vec<Field>),
    /// Graph node.
    Node(NodeItem),
    /// `link`.
    Link(EdgeItem),
    /// `instance base@origin { ... }`.
    Instance(Name, Name, Vec<Field>),
    /// `rule kind name clauses { items }`.
    Rule(Name, Name, Vec<Field>, Vec<Item>),
    /// `deadline d: N.unit dir trigger counted_by c clauses [opts]`.
    Deadline {
        /// Id.
        name: Name,
        /// Duration literal.
        amount: Expr,
        /// `from` / `after` / `before`.
        direction: Name,
        /// Trigger pattern.
        trigger: Expr,
        /// Counting rule.
        counted_by: RefPath,
        /// Version/relation/cite clauses.
        clauses: Vec<Field>,
        /// Deadline options.
        opts: Vec<Field>,
    },
    /// `counting c clauses { items }`.
    Counting(Name, Vec<Field>, Vec<Field>),
    /// `calendar c clauses { holidays }`.
    Calendar(Name, Vec<Field>, Vec<HolidayItem>),
    /// `transition t amends d clauses { arms }`.
    Transition(Name, RefPath, Vec<Field>, Vec<ArmItem>),
    /// `authority A > B`.
    Authority(Vec<Name>),
    /// `default d clauses when C => V`.
    Default(Name, Vec<Field>, Expr, Expr),
    /// `exception e rebuts|undercuts d clauses when C [=> V] [priority N]`.
    Exception {
        /// Id.
        name: Name,
        /// `rebuts` / `undercuts`.
        kind: Name,
        /// Target.
        target: RefPath,
        /// Cite clauses.
        clauses: Vec<Field>,
        /// Condition.
        when: Expr,
        /// Conclusion.
        then: Option<Expr>,
        /// Priority literal.
        priority: Option<Expr>,
    },
    /// `test "name" [cite] { stmts }`.
    Test(Name, Vec<Field>, Vec<TestStmt>),
    /// `grant g on [targets] clauses { fields }`: header clauses (the
    /// `on` targets among them), then the body.
    Grant(Name, Vec<Field>, Vec<Field>),
    /// `limit k clauses { entries } clauses`.
    Limit(LimitItem),
    /// `recharacterize r: d characterization c clauses`: the
    /// `characterization` field among the clauses.
    Recharacterize(Name, RefPath, Vec<Field>),
    /// `invalidate i: d clauses`.
    Invalidate(Name, RefPath, Vec<Field>),
}

/// Declaration with doc comment and range.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Item {
    /// Joined `///` lines.
    pub doc: Option<String>,
    /// Kind.
    pub kind: ItemKind,
    /// Range.
    pub range: TextRange,
}

/// `import a.b` or `import a.b.{x, y}`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Import {
    /// Module path.
    pub path: Vec<Name>,
    /// Imported names, if any.
    pub names: Option<Vec<Name>>,
    /// Range.
    pub range: TextRange,
}

/// `package name { forum f, authority a }`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageHeader {
    /// Package name.
    pub name: Name,
    /// `forum` / `authority` fields.
    pub fields: Vec<Field>,
    /// Range.
    pub range: TextRange,
}

/// One parsed `.lit` file.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ItemTree {
    /// `module a.b` path.
    pub module: Vec<Name>,
    /// Imports in order.
    pub imports: Vec<Import>,
    /// Optional package header.
    pub package: Option<PackageHeader>,
    /// Items in source order.
    pub items: Vec<Item>,
}

/// Joins a module path.
#[must_use]
pub fn dotted(path: &[Name]) -> String {
    path.iter()
        .map(|n| n.text.as_str())
        .collect::<Vec<_>>()
        .join(".")
}

/// Every [`TextRange`] of an item-tree value, visited mutably: how
/// lowering anchors the tree (see [`crate::db::span`]).
pub trait Ranges {
    /// Calls `f` on every range, in no particular order.
    fn ranges_mut(&mut self, f: &mut dyn FnMut(&mut TextRange));
}

impl Ranges for TextRange {
    fn ranges_mut(&mut self, f: &mut dyn FnMut(&mut TextRange)) {
        f(self);
    }
}

impl<T: Ranges> Ranges for Vec<T> {
    fn ranges_mut(&mut self, f: &mut dyn FnMut(&mut TextRange)) {
        for x in self {
            x.ranges_mut(f);
        }
    }
}

impl<T: Ranges> Ranges for Option<T> {
    fn ranges_mut(&mut self, f: &mut dyn FnMut(&mut TextRange)) {
        if let Some(x) = self {
            x.ranges_mut(f);
        }
    }
}

impl<T: Ranges> Ranges for Box<T> {
    fn ranges_mut(&mut self, f: &mut dyn FnMut(&mut TextRange)) {
        (**self).ranges_mut(f);
    }
}

impl<A: Ranges, B: Ranges> Ranges for (A, B) {
    fn ranges_mut(&mut self, f: &mut dyn FnMut(&mut TextRange)) {
        self.0.ranges_mut(f);
        self.1.ranges_mut(f);
    }
}

impl Ranges for Name {
    fn ranges_mut(&mut self, f: &mut dyn FnMut(&mut TextRange)) {
        f(&mut self.range);
    }
}

impl Ranges for RefPath {
    fn ranges_mut(&mut self, f: &mut dyn FnMut(&mut TextRange)) {
        let Self {
            head,
            instance,
            rest,
            version,
            range,
        } = self;
        head.ranges_mut(f);
        instance.ranges_mut(f);
        for (_, n) in rest {
            n.ranges_mut(f);
        }
        version.ranges_mut(f);
        f(range);
    }
}

impl Ranges for Expr {
    fn ranges_mut(&mut self, f: &mut dyn FnMut(&mut TextRange)) {
        f(&mut self.range);
        match &mut self.kind {
            ExprKind::Number(_)
            | ExprKind::Money(_)
            | ExprKind::Duration(_)
            | ExprKind::Str(_)
            | ExprKind::Date(_)
            | ExprKind::Bool(_)
            | ExprKind::Unit
            | ExprKind::Hole(_)
            | ExprKind::Cite(..) => {}
            ExprKind::Ref(r) => r.ranges_mut(f),
            ExprKind::Call(r, args) => {
                r.ranges_mut(f);
                args.ranges_mut(f);
            }
            ExprKind::Unary(_, e) | ExprKind::Paren(e) => e.ranges_mut(f),
            ExprKind::Binary(_, a, b) => {
                a.ranges_mut(f);
                b.ranges_mut(f);
            }
            ExprKind::Record(entries) => entries.ranges_mut(f),
            ExprKind::List(items) => items.ranges_mut(f),
            ExprKind::Field(e, n) => {
                e.ranges_mut(f);
                n.ranges_mut(f);
            }
            ExprKind::Match(e, arms) => {
                e.ranges_mut(f);
                arms.ranges_mut(f);
            }
        }
    }
}

impl Ranges for Pattern {
    fn ranges_mut(&mut self, f: &mut dyn FnMut(&mut TextRange)) {
        match self {
            Self::Ref(r) => r.ranges_mut(f),
            Self::Wildcard(r) => f(r),
        }
    }
}

impl Ranges for TypeRef {
    fn ranges_mut(&mut self, f: &mut dyn FnMut(&mut TextRange)) {
        let Self {
            name,
            args,
            range,
            refinements,
        } = self;
        name.ranges_mut(f);
        args.ranges_mut(f);
        f(range);
        refinements.ranges_mut(f);
    }
}

impl Ranges for Qualifier {
    fn ranges_mut(&mut self, f: &mut dyn FnMut(&mut TextRange)) {
        let Self {
            binder,
            predicate,
            range,
        } = self;
        binder.ranges_mut(f);
        predicate.ranges_mut(f);
        f(range);
    }
}

impl Ranges for Atom {
    fn ranges_mut(&mut self, f: &mut dyn FnMut(&mut TextRange)) {
        match self {
            Self::Expr(e) => e.ranges_mut(f),
            Self::Fields(fields) => fields.ranges_mut(f),
            Self::Interval(from, to) => {
                from.ranges_mut(f);
                to.ranges_mut(f);
            }
        }
    }
}

impl Ranges for Field {
    fn ranges_mut(&mut self, f: &mut dyn FnMut(&mut TextRange)) {
        let Self { key, values, range } = self;
        key.ranges_mut(f);
        values.ranges_mut(f);
        f(range);
    }
}

impl Ranges for EdgeItem {
    fn ranges_mut(&mut self, f: &mut dyn FnMut(&mut TextRange)) {
        let Self {
            name,
            name_range,
            label,
            from,
            to,
            clauses,
            fields,
            range,
        } = self;
        name.ranges_mut(f);
        f(name_range);
        label.ranges_mut(f);
        from.ranges_mut(f);
        to.ranges_mut(f);
        clauses.ranges_mut(f);
        fields.ranges_mut(f);
        f(range);
    }
}

impl Ranges for NodeItem {
    fn ranges_mut(&mut self, f: &mut dyn FnMut(&mut TextRange)) {
        let Self {
            keyword,
            name,
            label,
            clauses,
            fields,
            edges,
        } = self;
        keyword.ranges_mut(f);
        name.ranges_mut(f);
        label.ranges_mut(f);
        clauses.ranges_mut(f);
        fields.ranges_mut(f);
        edges.ranges_mut(f);
    }
}

impl Ranges for HolidayItem {
    fn ranges_mut(&mut self, f: &mut dyn FnMut(&mut TextRange)) {
        let Self {
            name,
            rule,
            effective,
            observed,
        } = self;
        name.ranges_mut(f);
        rule.ranges_mut(f);
        effective.ranges_mut(f);
        observed.ranges_mut(f);
    }
}

impl Ranges for ArmItem {
    fn ranges_mut(&mut self, f: &mut dyn FnMut(&mut TextRange)) {
        let Self {
            unless,
            target,
            range,
        } = self;
        unless.ranges_mut(f);
        target.ranges_mut(f);
        f(range);
    }
}

impl Ranges for TestStmt {
    fn ranges_mut(&mut self, f: &mut dyn FnMut(&mut TextRange)) {
        match self {
            Self::Given(e, by, date) => {
                e.ranges_mut(f);
                by.ranges_mut(f);
                date.ranges_mut(f);
            }
            Self::Expect(e) => e.ranges_mut(f),
        }
    }
}

impl Ranges for ItemKind {
    #[allow(clippy::too_many_lines)] // one arm per declaration kind
    fn ranges_mut(&mut self, f: &mut dyn FnMut(&mut TextRange)) {
        match self {
            Self::Start(r) => r.ranges_mut(f),
            Self::Group(a, b) => {
                a.ranges_mut(f);
                b.ranges_mut(f);
            }
            Self::TypeAlias(n, t) => {
                n.ranges_mut(f);
                t.ranges_mut(f);
            }
            Self::Enum(n, variants) => {
                n.ranges_mut(f);
                variants.ranges_mut(f);
            }
            Self::Event(n, params, effect) => {
                n.ranges_mut(f);
                params.ranges_mut(f);
                effect.ranges_mut(f);
            }
            Self::Node(n) => n.ranges_mut(f),
            Self::Link(e) => e.ranges_mut(f),
            Self::Instance(base, origin, fields) => {
                base.ranges_mut(f);
                origin.ranges_mut(f);
                fields.ranges_mut(f);
            }
            Self::Rule(kind, name, clauses, items) => {
                kind.ranges_mut(f);
                name.ranges_mut(f);
                clauses.ranges_mut(f);
                items.ranges_mut(f);
            }
            Self::Deadline {
                name,
                amount,
                direction,
                trigger,
                counted_by,
                clauses,
                opts,
            } => {
                name.ranges_mut(f);
                amount.ranges_mut(f);
                direction.ranges_mut(f);
                trigger.ranges_mut(f);
                counted_by.ranges_mut(f);
                clauses.ranges_mut(f);
                opts.ranges_mut(f);
            }
            Self::Counting(n, clauses, fields) => {
                n.ranges_mut(f);
                clauses.ranges_mut(f);
                fields.ranges_mut(f);
            }
            Self::Calendar(n, clauses, holidays) => {
                n.ranges_mut(f);
                clauses.ranges_mut(f);
                holidays.ranges_mut(f);
            }
            Self::Transition(n, amends, clauses, arms) => {
                n.ranges_mut(f);
                amends.ranges_mut(f);
                clauses.ranges_mut(f);
                arms.ranges_mut(f);
            }
            Self::Authority(chain) => chain.ranges_mut(f),
            Self::Default(n, clauses, when, then) => {
                n.ranges_mut(f);
                clauses.ranges_mut(f);
                when.ranges_mut(f);
                then.ranges_mut(f);
            }
            Self::Exception {
                name,
                kind,
                target,
                clauses,
                when,
                then,
                priority,
            } => {
                name.ranges_mut(f);
                kind.ranges_mut(f);
                target.ranges_mut(f);
                clauses.ranges_mut(f);
                when.ranges_mut(f);
                then.ranges_mut(f);
                priority.ranges_mut(f);
            }
            Self::Test(name, clauses, stmts) => {
                name.ranges_mut(f);
                clauses.ranges_mut(f);
                stmts.ranges_mut(f);
            }
            Self::Grant(name, clauses, fields) => {
                name.ranges_mut(f);
                clauses.ranges_mut(f);
                fields.ranges_mut(f);
            }
            Self::Limit(l) => l.ranges_mut(f),
            Self::Recharacterize(name, target, clauses)
            | Self::Invalidate(name, target, clauses) => {
                name.ranges_mut(f);
                target.ranges_mut(f);
                clauses.ranges_mut(f);
            }
        }
    }
}

impl Ranges for LimitItem {
    fn ranges_mut(&mut self, f: &mut dyn FnMut(&mut TextRange)) {
        let Self {
            name,
            clauses,
            entries,
        } = self;
        name.ranges_mut(f);
        clauses.ranges_mut(f);
        entries.ranges_mut(f);
    }
}

impl Ranges for LimitEntry {
    fn ranges_mut(&mut self, f: &mut dyn FnMut(&mut TextRange)) {
        let Self { kind, cites, range } = self;
        match kind {
            LimitEntryKind::Bound(_, max) => max.ranges_mut(f),
            LimitEntryKind::Certificate(kind, measures) => {
                kind.ranges_mut(f);
                for (_, range) in measures {
                    f(range);
                }
            }
            LimitEntryKind::Excludes(items) => items.ranges_mut(f),
            LimitEntryKind::Ecf(code) => code.ranges_mut(f),
        }
        cites.ranges_mut(f);
        f(range);
    }
}

impl Ranges for Item {
    fn ranges_mut(&mut self, f: &mut dyn FnMut(&mut TextRange)) {
        let Self {
            doc: _,
            kind,
            range,
        } = self;
        kind.ranges_mut(f);
        f(range);
    }
}

impl Ranges for Import {
    fn ranges_mut(&mut self, f: &mut dyn FnMut(&mut TextRange)) {
        let Self { path, names, range } = self;
        path.ranges_mut(f);
        names.ranges_mut(f);
        f(range);
    }
}

impl Ranges for PackageHeader {
    fn ranges_mut(&mut self, f: &mut dyn FnMut(&mut TextRange)) {
        let Self {
            name,
            fields,
            range,
        } = self;
        name.ranges_mut(f);
        fields.ranges_mut(f);
        f(range);
    }
}

impl Ranges for ItemTree {
    fn ranges_mut(&mut self, f: &mut dyn FnMut(&mut TextRange)) {
        let Self {
            module,
            imports,
            package,
            items,
        } = self;
        module.ranges_mut(f);
        imports.ranges_mut(f);
        package.ranges_mut(f);
        items.ranges_mut(f);
    }
}
