//! Counting rules roll to an open day from every date (SPEC §5.3, §13
//! C46; design review §2.2 tier 2). The engine's roll and business-day
//! loops run until they find an open day, so a rule whose closed days
//! (weekends and its calendar's observed holidays) never end would not
//! terminate. Whether they end is a property of every date, decided
//! exactly by Gregorian-cycle enumeration
//! (`litgraph::clock::cycle::ClosedRun`), on the clock fragment lowering
//! emits for each counting-rule version. An unbounded or undecided run is
//! L3009 (`well-typed`).
use crate::diag::emit;
use crate::Context;
use litgraph::clock::cycle::{ClosedRun, MAX_PERIOD_YEARS};
use litigate_base::{Diagnostic, DiagnosticCode as C};
use litigate_lower::Lowerer;
use litigate_sema::hir::Declaration;
use litigate_sema::temporal::World;

/// L3009 for every counting-rule version some date never rolls out of.
pub(crate) fn unbounded(cx: &Context<'_>) -> Vec<Diagnostic> {
    let mut loaded = cx.packages.to_vec();
    if !loaded.iter().any(|p| std::ptr::eq(*p, cx.package)) {
        loaded.push(cx.package);
    }
    let packages = World::closure_packages(&[cx.package], &loaded);
    let lowerer = Lowerer::new(&packages);
    let mut out = Vec::new();
    let versions = cx.declarations().filter_map(|d| match d {
        Declaration::Counting(c) => Some(c),
        _ => None,
    });
    for counting in versions {
        // A version that does not lower reports its own diagnostics.
        let library = lowerer.clock_version(cx.package, counting).ok().flatten();
        let clocks = library.iter().flat_map(|l| {
            l.counting_rules
                .iter()
                .flat_map(move |r| l.clock(&r.id).into_iter())
        });
        for clock in clocks {
            let (message, note) = match clock.closed_run() {
                ClosedRun::Bounded { .. } => continue,
                ClosedRun::Unbounded { at } => (
                    format!(
                        "counting rule `{}` never rolls to an open day from {}: every day from there on is closed",
                        counting.meta.id,
                        at.to_iso()
                    ),
                    "refinement `Counting = {c | every date reaches an open day}` fails; remove holidays until every cycle has an open day",
                ),
                ClosedRun::Undecided => (
                    format!(
                        "the holiday cycles of counting rule `{}` combine to a period longer than {MAX_PERIOD_YEARS} years",
                        counting.meta.id
                    ),
                    "rolling to an open day is decided over one combined period of 400 years and every `every(years, ..)` cycle; use cycles whose least common multiple with 400 is at most that",
                ),
            };
            let span = cx.span_or_manifest(counting.meta.span.as_ref());
            out.push(emit(C::InvalidCounting, cx.profile, span, message).note(note));
        }
    }
    out
}
