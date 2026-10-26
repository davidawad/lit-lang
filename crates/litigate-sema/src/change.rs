//! Legal change as layered instruments (docs/litigate/CHANGE.md): the class
//! a package has where its forum family is (the `[family.*]` matrix of
//! `core`), the binding scope of a precedent computed from the court
//! hierarchy, and worlds ([`world`]): a set of layers at a location, in a
//! modality.
use crate::db::inputs::PackageInput;
use crate::db::queries::manifest;
use crate::db::resolution::core;
use crate::db::span::{Report, TextRange};
use crate::hir::{Cell, Family, Kind, Manifest, Row, Scope};
use litigate_base::DiagnosticCode as C;

pub mod world;

/// A package's class where it is checked: its kind with a precedent's
/// scope computed, and the row its forum family's cell narrows the class
/// to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClassIndex {
    /// The class, a precedent's scope computed from its family.
    pub kind: Kind,
    /// The effects its declarations may have.
    pub row: Row,
    /// Its forum family's id and title, when `core` has a matrix listing
    /// its forum.
    pub family: Option<(String, String)>,
    /// The family's cell for the class, when it exists there.
    pub cell: Option<Cell>,
    /// Why the class has no cell (L2011), when it has none.
    pub missing: Option<String>,
}

impl ClassIndex {
    /// Whether the family's cell narrows the class's row.
    #[must_use]
    pub fn narrowed(&self) -> bool {
        self.cell.as_ref().is_some_and(|c| c.row.is_some())
    }
}

/// The matrix `pkg` is checked against: `core`'s `[family.*]` tables.
pub fn families(db: &dyn salsa::Database, pkg: PackageInput) -> &[Family] {
    pkg.files(db)
        .first()
        .map(|f| f.workspace(db))
        .and_then(|ws| core(db, ws, pkg))
        .and_then(|c| manifest(db, c).as_ref().ok())
        .map_or(&[], |m| m.manifest.families.as_slice())
}

/// The family of a package whose `forum` is `forum`.
#[must_use]
pub fn family_of<'f>(families: &'f [Family], forum: &str) -> Option<&'f Family> {
    families
        .iter()
        .find(|f| f.forums.iter().any(|x| x == forum))
}

/// A precedent's scope in `family` at `circuit` (a world's location;
/// `None`: anywhere in the family): binding when it was designated
/// precedential and its court's precedent binds there, else persuasive
/// (design review §2.5 item 1: computed, never declared).
#[must_use]
pub fn scope(m: &Manifest, family: Option<&Family>, circuit: Option<&str>) -> Scope {
    let binding = m.instrument.as_ref().is_some_and(|i| {
        i.designated.is_some()
            && i.court
                .as_deref()
                .is_some_and(|court| family.is_some_and(|f| f.binds(court, circuit)))
    });
    if binding {
        Scope::Binding
    } else {
        Scope::Persuasive
    }
}

/// The class of `pkg` as its forum family has it. Without a matrix in
/// `core` (a test library) the class's own row applies; with one, a
/// non-base class must exist in the family of the package's forum.
#[salsa::tracked(returns(ref))]
pub fn class_index(db: &dyn salsa::Database, pkg: PackageInput) -> Option<ClassIndex> {
    let m = &manifest(db, pkg).as_ref().ok()?.manifest;
    let families = families(db, pkg);
    let family = family_of(families, &m.forum);
    let kind = match &m.kind {
        Kind::Precedent { .. } => Kind::Precedent {
            scope: scope(m, family, None),
        },
        other => other.clone(),
    };
    let cell = family.and_then(|f| f.cell(&kind)).cloned();
    let missing = (!kind.is_base() && !families.is_empty() && cell.is_none()).then(|| {
        family.map_or_else(
            || {
                format!(
                    "forum `{}` is in no forum family of `core`'s matrix, so a `{}` package \
                     there is not a recognised instrument",
                    m.forum,
                    kind.name()
                )
            },
            |f| {
                format!(
                    "the `{}` forum family ({}) has no `{}` class",
                    f.id,
                    f.title,
                    kind.name()
                )
            },
        )
    });
    let ceiling = kind.row();
    Some(ClassIndex {
        row: cell.as_ref().map_or(ceiling, |c| c.row(ceiling)),
        family: family.map(|f| (f.id.clone(), f.title.clone())),
        kind,
        cell,
        missing,
    })
}

/// L2011 for `pkg`'s manifest `text`: a class its forum family does not
/// have, at the `kind` key.
#[must_use]
pub fn class_reports(db: &dyn salsa::Database, pkg: PackageInput, text: &str) -> Vec<Report> {
    let Some(ix) = class_index(db, pkg) else {
        return Vec::new();
    };
    let Some(why) = &ix.missing else {
        return Vec::new();
    };
    let families = families(db, pkg);
    let mut report = Report::new(C::ClassOutsideFamily, key_range(text, "kind"), why.clone());
    if let Some(f) = ix
        .family
        .as_ref()
        .and_then(|(id, _)| families.iter().find(|f| &f.id == id))
    {
        report = report.note(format!(
            "its classes: {}",
            f.classes
                .iter()
                .map(|c| format!("`{}`", c.class))
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
    vec![report]
}

/// The range of the line of `text` setting `key` (`key = ...`), else the
/// start of the text.
#[must_use]
pub fn key_range(text: &str, key: &str) -> TextRange {
    let mut at = 0usize;
    for line in text.split_inclusive('\n') {
        let rest = line.trim_start();
        let is_key = rest
            .strip_prefix(key)
            .is_some_and(|r| r.trim_start().starts_with('='));
        if is_key {
            let start = at + (line.len() - rest.len());
            let end = at + line.trim_end().len();
            let f = |n: usize| u32::try_from(n).unwrap_or(u32::MAX);
            return TextRange::new(f(start), f(end));
        }
        at += line.len();
    }
    TextRange::default()
}
