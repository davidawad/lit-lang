//! `litigate timeline FILE` (lg-z2m.1): the timeline of a `.case` or
//! `.lit` file as a `litigate.timeline/1` document (docs/litigate/TIMELINE.md),
//! the envelope's `result`.
use super::check::{checked, engine, Checked};
use crate::cli::Dates;
use crate::clock::Clocks;
use crate::matter::{self, At};
use crate::out::{Failure, Outcome, Report};
use crate::timeline;
use crate::ws;
use litigate_base::timeline::Timeline;
use litigate_base::{Diagnostic, Profile};
use std::path::Path;

/// `timeline`'s arguments.
#[derive(Debug, Clone, Default)]
pub struct Args {
    /// The `.case` or `.lit` file.
    pub file: String,
    /// Judge statuses (and select law) at these dates.
    pub dates: Dates,
    /// Layers to enable (`ID`, `+ID`) or disable (`-ID`).
    pub layers: Vec<String>,
    /// Only this world and its ancestors.
    pub world: Option<String>,
    /// Library directories.
    pub lib: Vec<String>,
}

/// True for a `YYYY-MM-DD` date.
fn is_date(text: &str) -> bool {
    let b = text.as_bytes();
    b.len() == 10
        && b.iter().enumerate().all(|(i, c)| match i {
            4 | 7 => *c == b'-',
            _ => c.is_ascii_digit(),
        })
}

/// The matter's timeline: checked again at the dates `args` give, if any
/// (its diagnostics then replace the first check's); `None` when the
/// matter does not resolve.
fn of_case(
    args: &Args,
    c: &Checked,
    file: &str,
    diagnostics: &mut Vec<Diagnostic>,
) -> Option<Timeline> {
    let all = c.session.hir();
    let texts = c.session.sources();
    let at = At {
        as_of: args.dates.as_of.as_deref(),
        known_at: args.dates.known_at.as_deref(),
    };
    let m = if at.as_of.is_none() && at.known_at.is_none() {
        c.cases[0].clone()
    } else {
        let redone = matter::check_at(
            &c.session,
            &c.session.cases[0],
            &engine(&all),
            &Clocks::new(&all),
            Profile::Dev,
            at,
        );
        diagnostics.retain(|d| d.span.file.0 != file);
        diagnostics.extend(redone.diagnostics.iter().cloned());
        redone
    };
    m.case
        .as_ref()
        .map(|case| timeline::of_case(&m, case, &all, &texts, diagnostics))
}

/// The `.lit` module's timeline, at `--as-of` else its package's
/// `known_at`; `None` when its package does not check.
fn of_lit(args: &Args, c: &Checked, file: &str, diagnostics: &[Diagnostic]) -> Option<Timeline> {
    let all = c.session.hir();
    let texts = c.session.sources();
    let pkg = all.iter().copied().find(|p| {
        c.session
            .packages
            .iter()
            .any(|k| k.name == p.manifest.name && k.files.iter().any(|(f, _)| f == file))
    })?;
    let as_of = args
        .dates
        .as_of
        .clone()
        .unwrap_or_else(|| pkg.manifest.in_force.known_at.clone());
    Some(timeline::of_lit(
        file,
        pkg,
        &all,
        &texts,
        (as_of, args.dates.known_at.clone()),
        diagnostics,
    ))
}

/// `litigate timeline`.
pub fn timeline(args: &Args) -> Outcome {
    for date in [&args.dates.as_of, &args.dates.known_at]
        .into_iter()
        .flatten()
    {
        if !is_date(date) {
            return Err(Failure::new(format!("`{date}` is not a YYYY-MM-DD date")));
        }
    }
    let path = Path::new(&args.file);
    let kind = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or_default();
    if kind != "case" && kind != "lit" {
        return Err(Failure::new(format!(
            "{}: not a .case or .lit file",
            args.file
        )));
    }
    // A matter's dates select its own law (`At`); the packages are checked
    // as `check` checks them. A module's dates select the law it lists.
    let options = if kind == "case" {
        Dates::default().options()
    } else {
        args.dates.options()
    };
    let c = checked(
        std::slice::from_ref(&args.file),
        Profile::Dev,
        &options,
        &args.lib,
    )?;
    let file = ws::shown(path);
    let mut diagnostics = c.diagnostics.clone();
    let built = if kind == "case" {
        of_case(args, &c, &file, &mut diagnostics)
    } else {
        of_lit(args, &c, &file, &diagnostics)
    };
    let mut report = Report {
        provenance: c.provenance(args.dates.as_of.as_deref(), args.dates.known_at.as_deref()),
        sources: c.session.sources(),
        diagnostics,
        ..Report::default()
    };
    if let Some(mut t) = built {
        let off = timeline::disabled_layers(&timeline::layer_ids(&t), &args.layers)
            .map_err(Failure::new)?;
        timeline::apply_layers(&mut t, &off);
        if let Some(world) = &args.world {
            timeline::apply_world(&mut t, world).map_err(Failure::new)?;
        }
        report.result = serde_json::to_value(&t).unwrap_or_default();
    }
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dates_are_yyyy_mm_dd() {
        assert!(is_date("2026-04-08"));
        assert!(!is_date("2026-4-08"));
        assert!(!is_date("2026/04/08"));
        assert!(!is_date("20260408xx"));
    }
}
