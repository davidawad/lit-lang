//! Compiler diagnostics as LSP diagnostics, and their fixes as code
//! actions (machine-applicable fixes are preferred quick fixes; others are
//! offered but never preferred, as `litigate fix --include-maybe`).
use crate::analysis::Analysis;
use crate::convert;
use litigate_base::{Applicability, Diagnostic, Severity};
use std::collections::HashMap;
use tower_lsp::lsp_types::{
    CodeAction, CodeActionKind, Diagnostic as LspDiagnostic, DiagnosticRelatedInformation,
    DiagnosticSeverity, Location, NumberOrString, Range, TextEdit, Url, WorkspaceEdit,
};

/// The LSP form of `d`, positioned over `text`.
#[must_use]
pub fn to_lsp(analysis: &Analysis, d: &Diagnostic, text: &str) -> LspDiagnostic {
    let related: Vec<DiagnosticRelatedInformation> = d
        .notes
        .iter()
        .filter_map(|n| {
            let span = n.span.as_ref()?;
            let note_text = analysis.text(&span.file.0)?;
            Some(DiagnosticRelatedInformation {
                location: Location {
                    uri: analysis.uri_of(&span.file.0)?,
                    range: convert::span_range(&note_text, span),
                },
                message: n.message.clone(),
            })
        })
        .collect();
    let unplaced: Vec<&str> = d
        .notes
        .iter()
        .filter(|n| n.span.is_none())
        .map(|n| n.message.as_str())
        .collect();
    let message = std::iter::once(d.message.as_str())
        .chain(unplaced)
        .collect::<Vec<_>>()
        .join("\nnote: ");
    LspDiagnostic {
        range: convert::span_range(text, &d.span),
        severity: Some(match d.severity {
            Severity::Error => DiagnosticSeverity::ERROR,
            Severity::Warning => DiagnosticSeverity::WARNING,
            Severity::Info => DiagnosticSeverity::INFORMATION,
            Severity::Hint => DiagnosticSeverity::HINT,
        }),
        code: Some(NumberOrString::String(d.code.as_str().to_owned())),
        source: Some("litigate".into()),
        message,
        related_information: (!related.is_empty()).then_some(related),
        ..LspDiagnostic::default()
    }
}

/// Every file's LSP diagnostics, by URI.
#[must_use]
pub fn publishable(analysis: &Analysis) -> Vec<(Url, Vec<LspDiagnostic>)> {
    analysis
        .diagnostics
        .iter()
        .filter_map(|(file, ds)| {
            let text = analysis.text(file).unwrap_or_default();
            let uri = analysis.uri_of(file)?;
            Some((uri, ds.iter().map(|d| to_lsp(analysis, d, &text)).collect()))
        })
        .collect()
}

/// Quick fixes for diagnostics of `file` touching `range`.
#[must_use]
pub fn actions(analysis: &Analysis, file: &str, range: Range) -> Vec<CodeAction> {
    let Some(text) = analysis.text(file) else {
        return Vec::new();
    };
    analysis
        .diagnostics
        .get(file)
        .into_iter()
        .flatten()
        .filter(|d| convert::overlaps(&convert::span_range(&text, &d.span), &range))
        .flat_map(|d| {
            let lsp = to_lsp(analysis, d, &text);
            d.fixes.iter().filter_map(move |fix| {
                let fix_text = analysis.text(&fix.span.file.0)?;
                let edit = TextEdit {
                    range: convert::span_range(&fix_text, &fix.span),
                    new_text: fix.replacement.clone(),
                };
                let changes: HashMap<Url, Vec<TextEdit>> =
                    std::iter::once((analysis.uri_of(&fix.span.file.0)?, vec![edit])).collect();
                let title = if fix.replacement.is_empty() {
                    format!("{}: remove", d.code.as_str())
                } else {
                    format!("{}: write `{}`", d.code.as_str(), fix.replacement.trim())
                };
                Some(CodeAction {
                    title,
                    kind: Some(CodeActionKind::QUICKFIX),
                    diagnostics: Some(vec![lsp.clone()]),
                    edit: Some(WorkspaceEdit {
                        changes: Some(changes),
                        ..WorkspaceEdit::default()
                    }),
                    is_preferred: Some(fix.applicability == Applicability::MachineApplicable),
                    ..CodeAction::default()
                })
            })
        })
        .collect()
}
