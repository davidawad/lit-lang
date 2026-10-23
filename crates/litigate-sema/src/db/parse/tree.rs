//! The item tree: sema's syntax-independent view of one `.lit` file,
//! lowered from the `litigate-syntax` CST. Every later query consumes only
//! this, so CST trivia and recovery details never reach name resolution or
//! type checking.
use crate::db::span::TextRange;
use crate::hir::{BinaryOp, UnaryOp};

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
    /// `cite ...`.
    Cite(String),
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

/// Type expression `Name<Args>`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeRef {
    /// Constructor name.
    pub name: Name,
    /// Arguments.
    pub args: Vec<TypeRef>,
    /// Range.
    pub range: TextRange,
}

/// JSON value in `patch` records and `analysis`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Json {
    /// Signed decimal text.
    Number(String),
    /// String.
    Str(String),
    /// Boolean.
    Bool(bool),
    /// `unset` (JSON null).
    Unset,
    /// Object in source order.
    Object(Vec<(Name, Json)>),
    /// Array.
    Array(Vec<Json>),
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
    /// JSON record.
    Json(Json),
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

/// Test statement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TestStmt {
    /// `given e on date`.
    Given(Expr, Name),
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
