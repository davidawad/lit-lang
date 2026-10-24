//! Inlay hints on a `.case`: after each event that triggers a deadline,
//! the due date the engine clock computes for it (`// answer-period due
//! 2025-03-26 (<counting rule cite>)`, plus `rolled` and the service
//! extension when they applied), from the same replay and counting as
//! `litigate check`'s deadline schedule.
use crate::analysis::Analysis;
use crate::convert;
use litigate_cli::matter::Scheduled;
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

/// The label of one scheduled deadline, with what the clock did: the
/// schedule's own versions and count, never a recount.
fn label(s: &Scheduled) -> String {
    let (_, local) = s.deadline.split_once("::").unwrap_or(("", &s.deadline));
    let Some(due) = &s.due else {
        // The reason's head ("notice due date not recorded"); the tooltip
        // has it whole.
        let why = s.unknown.as_deref().unwrap_or_default();
        let head = why.split_once(": ").map_or(why, |(h, _)| h);
        return format!("// {local} due unknown: {head}");
    };
    let cite = s.cite.as_deref().unwrap_or(&s.rule);
    let mut flags = String::new();
    if s.rolled {
        flags.push_str(" rolled");
    }
    if let Some(m) = &s.method {
        let _ = write!(flags, " +{m} service");
    }
    format!("// {local} due {due} ({cite}{flags})")
}

/// The hover of one scheduled deadline: its count, or why its due date
/// is unknown.
fn tooltip(s: &Scheduled) -> String {
    if let Some(why) = &s.unknown {
        return format!(
            "`{}` triggered by `{}` on {}: the due date is unknown: {why}",
            s.deadline, s.trigger, s.trigger_date
        );
    }
    format!(
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
    )
}

/// Deadline hints for the matter at `file` within `range`.
#[must_use]
pub fn hints(analysis: &Analysis, file: &str, range: Range) -> Vec<InlayHint> {
    let (Some(case), Some(text)) = (analysis.case_of(file), analysis.text(file)) else {
        return Vec::new();
    };
    let root = parse(&text, FileKind::Case).syntax();
    case.schedule
        .iter()
        .filter_map(|s| {
            let at = event_end(&root, &s.trigger_date, &s.trigger)?;
            let position = convert::position(&text, usize::from(at));
            convert::touches(&range, position).then(|| InlayHint {
                position,
                label: InlayHintLabel::String(label(s)),
                kind: None::<InlayHintKind>,
                text_edits: None,
                tooltip: Some(InlayHintTooltip::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: tooltip(s),
                })),
                padding_left: Some(true),
                padding_right: None,
                data: None,
            })
        })
        .collect()
}
