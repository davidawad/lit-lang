//! Document symbols (declarations with their edges, variants, nested rule
//! declarations, parties, and holidays as children) and workspace symbols.
use crate::analysis::Analysis;
use crate::convert;
use crate::index::{Def, Space};
use litigate_syntax::SyntaxKind as K;
use tower_lsp::lsp_types::{DocumentSymbol, Location, SymbolInformation, SymbolKind};

/// The LSP symbol kind of a definition.
#[must_use]
pub fn kind(def: &Def) -> SymbolKind {
    match (&def.space, def.kind) {
        (Space::Edge(_), _) => SymbolKind::METHOD,
        (Space::Variant(_), _) => SymbolKind::ENUM_MEMBER,
        (Space::Party | Space::Role, _) => SymbolKind::VARIABLE,
        (_, K::EnumDecl) => SymbolKind::ENUM,
        (_, K::TypeDecl) => SymbolKind::STRUCT,
        (_, K::EventDecl) => SymbolKind::EVENT,
        (_, K::DeadlineDecl) => SymbolKind::CONSTANT,
        (_, K::CountingDecl | K::CalendarDecl | K::TransitionDecl) => SymbolKind::FUNCTION,
        (_, K::RuleDecl | K::DefaultDecl | K::ExceptionDecl | K::AuthorityDecl) => {
            SymbolKind::INTERFACE
        }
        (_, K::GroupDecl) => SymbolKind::NAMESPACE,
        (Space::Module, _) | (_, K::CaseFile) => SymbolKind::MODULE,
        (_, K::Param | K::HolidayDecl) => SymbolKind::FIELD,
        _ => SymbolKind::OBJECT,
    }
}

fn detail(def: &Def) -> String {
    let kind = format!("{:?}", def.kind);
    kind.trim_end_matches("Decl").to_lowercase()
}

/// The symbol of `defs[i]` with its children.
fn build(i: usize, defs: &[&Def], parents: &[Option<usize>], text: &str) -> DocumentSymbol {
    let children: Vec<DocumentSymbol> = (0..defs.len())
        .filter(|&j| parents[j] == Some(i))
        .map(|j| build(j, defs, parents, text))
        .collect();
    #[allow(deprecated)]
    DocumentSymbol {
        name: defs[i].name.clone(),
        detail: Some(detail(defs[i])),
        kind: kind(defs[i]),
        tags: None,
        deprecated: None,
        range: convert::range(text, defs[i].full_range),
        selection_range: convert::range(text, defs[i].name_range),
        children: (!children.is_empty()).then_some(children),
    }
}

/// The symbol tree of `file`: a definition nests under the innermost
/// definition whose declaration contains it.
#[must_use]
pub fn document(analysis: &Analysis, file: &str) -> Vec<DocumentSymbol> {
    let Some(text) = analysis.text(file) else {
        return Vec::new();
    };
    let defs: Vec<&Def> = analysis
        .index
        .defs
        .iter()
        .filter(|d| d.file == file && d.kind != K::CaseFile)
        .collect();
    let parent = |i: usize| -> Option<usize> {
        let d = defs[i];
        (0..defs.len())
            .filter(|&j| {
                j != i
                    && defs[j].full_range.contains_range(d.full_range)
                    && defs[j].full_range != d.full_range
            })
            .min_by_key(|&j| defs[j].full_range.len())
    };
    let parents: Vec<Option<usize>> = (0..defs.len()).map(parent).collect();
    (0..defs.len())
        .filter(|&i| parents[i].is_none())
        .map(|i| build(i, &defs, &parents, &text))
        .collect()
}

/// Workspace symbols whose name contains `query` (case-insensitive).
#[must_use]
pub fn workspace(analysis: &Analysis, query: &str) -> Vec<SymbolInformation> {
    let query = query.to_lowercase();
    analysis
        .index
        .defs
        .iter()
        .filter(|d| d.name.to_lowercase().contains(&query))
        .filter_map(|d| {
            // The index reads every definition off an analysed text.
            let text = analysis.text(&d.file).unwrap_or_default();
            #[allow(deprecated)]
            Some(SymbolInformation {
                name: d.name.clone(),
                kind: kind(d),
                tags: None,
                deprecated: None,
                location: Location {
                    uri: analysis.uri_of(&d.file)?,
                    range: convert::range(&text, d.name_range),
                },
                container_name: match &d.space {
                    Space::Edge(owner) => Some(owner.clone()),
                    Space::Variant(e) => Some(e.clone()),
                    _ => d.package.clone(),
                },
            })
        })
        .collect()
}
