//! `.case` matters (SPEC §3.2, §5.6): parse, elaborate to `hir::Case`
//! against the packages it `uses`, replay its events to derive the start
//! posture, and lower it to a litgraph `NamedScenario`.
//!
//! Sema does not yet build case HIR (its "sema case" bead); this bridge
//! does so from the typed AST with the same rules and codes, behind one
//! function ([`check`]) so swapping in sema's query is a one-line change.
mod blocks;
mod elab;
pub mod replay;

use crate::clock::Clocks;
use crate::ws::{CaseText, Session};
use litigate_base::{Diagnostic, FileId, LineIndex, Profile, Span};
use litigate_lower::{LoweredCase, Lowerer};
use litigate_sema::hir;
use litigate_syntax::{SyntaxNode, TextRange};
use replay::{Scheduled, Step};
use serde::Serialize;

/// A checked matter.
#[derive(Debug, Clone, Default)]
pub struct CaseCheck {
    /// Repository-relative path.
    pub path: String,
    /// Case HIR (present when elaboration succeeded).
    pub case: Option<hir::Case>,
    /// The lowered scenario (present when the matter has no errors).
    pub lowered: Option<LoweredCase>,
    /// Diagnostics, severities set for the profile.
    pub diagnostics: Vec<Diagnostic>,
    /// Replay trace.
    pub replay: Vec<Step>,
    /// Deadlines the events triggered, with their due dates.
    pub schedule: Vec<Scheduled>,
    /// Names of the packages it uses that were loaded, primary first.
    pub packages: Vec<String>,
}

/// The JSON summary `check` reports per matter.
#[derive(Debug, Clone, Serialize)]
pub struct Summary {
    /// Path.
    pub path: String,
    /// Case id.
    pub id: Option<String>,
    /// Replayed start (qualified).
    pub start: Option<String>,
    /// Replay trace.
    pub replay: Vec<Step>,
    /// Deadline schedule.
    pub deadlines: Vec<Scheduled>,
}

impl CaseCheck {
    /// The JSON summary.
    #[must_use]
    pub fn summary(&self) -> Summary {
        Summary {
            path: self.path.clone(),
            id: self.case.as_ref().map(|c| c.id.clone()),
            start: self.case.as_ref().and_then(|c| c.start.clone()),
            replay: self.replay.clone(),
            deadlines: self.schedule.clone(),
        }
    }
}

/// Elaboration context: one file, the packages it uses (primary first).
pub(crate) struct Cx<'a> {
    pub file: FileId,
    pub index: LineIndex,
    pub packages: Vec<&'a hir::Package>,
    pub as_of: String,
    pub profile: Profile,
    pub diagnostics: Vec<Diagnostic>,
}

impl<'a> Cx<'a> {
    pub fn span(&self, range: TextRange) -> Span {
        litigate_syntax::span_of(&self.file, &self.index, range)
    }

    pub fn node_span(&self, node: &SyntaxNode) -> Span {
        self.span(node.text_range())
    }

    pub fn error(
        &mut self,
        code: litigate_base::DiagnosticCode,
        span: Span,
        message: impl Into<String>,
    ) {
        self.diagnostics
            .push(Diagnostic::error(code, span, message));
    }

    pub fn primary(&self) -> Option<&'a hir::Package> {
        self.packages.first().copied()
    }

    /// `pack::id` for a reference (unqualified refs resolve in the primary package).
    pub fn qualify(&self, text: &str) -> String {
        match (text.contains("::"), self.primary()) {
            (false, Some(p)) => format!("{}::{text}", p.manifest.pack_id),
            _ => text.to_owned(),
        }
    }

    /// The node a qualified id names, in force on the case's `as_of`.
    #[must_use]
    pub fn node(&self, qualified: &str) -> Option<(&'a hir::Package, &'a hir::Node)> {
        let (pack, id) = qualified.split_once("::")?;
        // `base@origin::id` names a node of an instance of pack `base`.
        let pack = pack.split('@').next().unwrap_or(pack);
        let pkg: &'a hir::Package = self
            .packages
            .iter()
            .copied()
            .find(|p| p.manifest.pack_id == pack)?;
        crate::hirx::nodes_at(pkg, Some(&self.as_of))
            .into_iter()
            .find(|n| n.meta.id == id)
            .map(|n| (pkg, n))
    }
}

/// Checks one matter against the loaded packages.
#[must_use]
pub fn check(session: &Session, case: &CaseText, clocks: &Clocks, profile: Profile) -> CaseCheck {
    let file = FileId(case.path.clone());
    let parsed = litigate_syntax::parse_case(&case.text);
    let mut out = CaseCheck {
        path: case.path.clone(),
        ..CaseCheck::default()
    };
    if !parsed.is_ok() {
        out.diagnostics = parsed.diagnostics(&file);
        return out;
    }
    let Some(ast) = parsed.case_file() else {
        return out;
    };
    let mut cx = Cx {
        file,
        index: LineIndex::new(&case.text),
        packages: Vec::new(),
        as_of: String::new(),
        profile,
        diagnostics: Vec::new(),
    };
    let elaborated = elab::case(&mut cx, session, &ast);
    out.packages = cx
        .packages
        .iter()
        .map(|p| p.manifest.name.clone())
        .collect();
    if let Some(mut hir_case) = elaborated {
        if cx.diagnostics.is_empty() {
            let replayed = replay::replay(&mut cx, &ast, &hir_case, clocks);
            hir_case.start = replayed.start;
            out.replay = replayed.steps;
            out.schedule = replayed.schedule;
            blocks::stakes_cover(&mut cx, &ast, &hir_case);
        }
        if cx.diagnostics.is_empty() {
            match Lowerer::new(&session.hir()).case(&hir_case, &case.path) {
                Ok(lowered) => out.lowered = Some(lowered),
                Err(errors) => cx.diagnostics.extend(errors),
            }
        }
        out.case = Some(hir_case);
    }
    if profile == Profile::Strict {
        cx.diagnostics.extend(litigate_syntax::check_format(
            &case.text,
            litigate_syntax::FileKind::Case,
            &case.path.as_str().into(),
        ));
    }
    let sources = [(case.path.clone(), case.text.clone())];
    out.diagnostics = cx
        .diagnostics
        .into_iter()
        .map(|d| crate::verify::adjust(d, profile, &sources))
        .collect();
    out
}
