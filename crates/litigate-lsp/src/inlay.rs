//! Inlay hints on a `.case`: after each event that triggers a deadline,
//! the due date the engine clock computes for it (`// answer-period due
//! 2025-03-26 (FRCP 6(a))`), from the same replay and counting as
//! `litigate check`'s deadline schedule.
use crate::analysis::Analysis;
use crate::convert;
use litigate_cli::clock::{self, Clocks};
use litigate_cli::hirx;
use litigate_cli::matter::Scheduled;
use litigate_sema::hir;
use litigate_syntax::{parse, FileKind, SyntaxKind as K, SyntaxNode, TextSize};
use std::fmt::Write as _;
use tower_lsp::lsp_types::{
    InlayHint, InlayHintKind, InlayHintLabel, InlayHintTooltip, MarkupContent, MarkupKind, Range,
};

fn squash(s: &str) -> String {
    s.chars().filter(|c| !c.is_whitespace()).collect()
}

/// The end of the `events` entry dated `date` whose call reads `call`.
fn event_end(root: &SyntaxNode, date: &str, call: &str) -> Option<TextSize> {
    root.descendants()
        .filter(|n| n.kind() == K::CaseEvent)
        .find(|n| {
            let dated = n
                .descendants_with_tokens()
                .any(|t| t.kind() == K::Date && t.to_string() == date);
            let called = n
                .children()
                .find(|c| c.kind() == K::CallExpr)
                .is_some_and(|c| squash(&c.to_string()) == squash(call));
            dated && called
        })
        .map(|n| n.text_range().end())
}

/// The label of one scheduled deadline, with what the clock did.
fn label(analysis: &Analysis, clocks: &Clocks, case: Option<&hir::Case>, s: &Scheduled) -> String {
    let (pack, local) = s.deadline.split_once("::").unwrap_or(("", &s.deadline));
    let pkg = analysis.hir_by_key(pack);
    let cite = pkg
        .and_then(|p| hirx::counting(p, &s.rule))
        .and_then(|c| c.meta.cite.clone())
        .unwrap_or_else(|| s.rule.clone());
    let event = case.and_then(|c| {
        c.events
            .iter()
            .find(|e| e.date == s.trigger_date && hirx::event_text(&e.action) == s.trigger)
    });
    let computed = pkg.zip(event).and_then(|(p, e)| {
        let d = hirx::deadline_at(p, local, &s.trigger_date).ok()?;
        clock::due(clocks, p, d, &s.trigger_date, Some(&e.action)).ok()
    });
    let flags: String = computed
        .map(|due| {
            let mut f = String::new();
            if due.computed.last_day_rolled {
                f.push_str(" rolled");
            }
            if let Some(m) = due.method {
                let _ = write!(f, " +{m} service");
            }
            f
        })
        .unwrap_or_default();
    format!("// {local} due {} ({cite}{flags})", s.due)
}

/// Deadline hints for the matter at `file` within `range`.
#[must_use]
pub fn hints(analysis: &Analysis, file: &str, range: Range) -> Vec<InlayHint> {
    let (Some(case), Some(text)) = (analysis.case_of(file), analysis.text(file)) else {
        return Vec::new();
    };
    let root = parse(&text, FileKind::Case).syntax();
    let clocks = Clocks::new(&analysis.checked.session.hir());
    case.schedule
        .iter()
        .filter_map(|s| {
            let at = event_end(&root, &s.trigger_date, &s.trigger)?;
            let position = convert::position(&text, usize::from(at));
            convert::touches(&range, position).then(|| InlayHint {
                position,
                label: InlayHintLabel::String(label(analysis, &clocks, case.case.as_ref(), s)),
                kind: None::<InlayHintKind>,
                text_edits: None,
                tooltip: Some(InlayHintTooltip::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: format!(
                        "`{}` triggered by `{}` on {}, counted by `{}`:\n\n{}",
                        s.deadline,
                        s.trigger,
                        s.trigger_date,
                        s.rule,
                        s.steps
                            .iter()
                            .map(|x| format!("- {x}"))
                            .collect::<Vec<_>>()
                            .join("\n")
                    ),
                })),
                padding_left: Some(true),
                padding_right: None,
                data: None,
            })
        })
        .collect()
}
