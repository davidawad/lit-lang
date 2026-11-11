//! Verification obligations for compiled litigate packages and matters
//! (SPEC §7, DESIGN §7).
//!
//! [`verify_package`] discharges every obligation of a checked, lowered
//! rule package: sema's diagnostics are attributed to the obligations they
//! violate (`well-typed`, the temporal and precedence obligations,
//! `dist-exact`), and this crate checks the rest on the lowered pack and
//! HIR (`graph-exits`, `graph-reachable`, `deadline-satisfiable`,
//! `workflow-sound`,
//! `cite-resolves`, `tests-pass`, `holes`, `rule-without-test`). The
//! result is a [`Verification`]: the [`VerificationReport`] embedded in the
//! [`Litigatable`], plus the diagnostics this crate emits (sema's are
//! aggregated, not repeated). [`check_package`] runs lowering and
//! verification together; [`verify_case`] and [`verify_case_source`] run
//! sema's case replay on a matter and report its `case-replay`.
//!
//! Profiles change severity and acceptance, never values: holes are L9002
//! warnings in `dev` and L5002 errors in `strict`; L9001 is a warning in
//! `dev` and an error in `strict`; L9011 is a warning in both profiles.
//! Other failures are errors.
mod assertions;
mod cite;
mod context;
mod cycle;
mod deadline;
mod diag;
mod evidence;
mod graph;
mod lints;
mod matter;
mod net;
mod refinement;
mod report;
mod sema;

pub use matter::{verify_case, verify_case_source, verify_case_under, VerifiedCase};
pub use report::{
    Evidence, Finding, Obligation, ObligationStatus, VerificationReport, MAX_EXAMPLES,
};
pub use sema::obligation_of;

pub(crate) use context::{same_expr, Context};

use litigate_base::{Diagnostic, Profile};
use litigate_engine::cite::SourceCorpus;
use litigate_engine::model::Pack;
use litigate_lower::{Document, LoweredPackage, Lowerer, Options, SourceMap};
use litigate_sema::{hir, PackageCheck};
use report::{Builder, Obligation as O};
use std::time::Instant;

fn timed<T>(package: &str, phase: &str, f: impl FnOnce() -> T) -> T {
    if std::env::var_os("LITIGATE_PHASE_TIMINGS").is_none_or(|value| value != "1") {
        return f();
    }
    let started = Instant::now();
    let result = f();
    eprintln!(
        "LITIGATE_PHASE\t{package}\tverify:{phase}\t{:.3}ms",
        started.elapsed().as_secs_f64() * 1000.0
    );
    result
}

/// A lowered document with its reproducible verification report.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Litigatable<T> {
    /// Lowered engine IR.
    pub ir: T,
    /// Verification results for this artifact.
    pub verification: VerificationReport,
    /// Source map sidecar, when available.
    pub source_map: Option<SourceMap>,
}

/// The outcome of verifying one artifact.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Verification {
    /// Every obligation's finding.
    pub report: VerificationReport,
    /// Diagnostics emitted by verification, sorted (SPEC §9.1). Sema and
    /// lowering diagnostics are counted in `report`, not repeated here.
    pub diagnostics: Vec<Diagnostic>,
}

impl Verification {
    /// Whether the profile accepts the artifact: no obligation failed.
    #[must_use]
    pub fn is_accepted(&self) -> bool {
        self.report.is_accepted()
    }

    /// Embeds the report in a litigatable for `ir`.
    #[must_use]
    pub fn embed<T>(&self, ir: T, source_map: Option<SourceMap>) -> Litigatable<T> {
        Litigatable {
            ir,
            verification: self.report.clone(),
            source_map,
        }
    }

    fn new(builder: Builder, mut diagnostics: Vec<Diagnostic>, profile: Profile) -> Self {
        diagnostics.sort_by(diag::order);
        Self {
            report: builder.finish(profile),
            diagnostics,
        }
    }
}

/// One checked rule package, its lowering, and the workspace around it.
pub struct PackageInput<'a> {
    /// The package's sema result; its HIR is the package verified.
    pub package: &'a hir::Package,
    /// Its sema diagnostics and holes.
    pub check: &'a PackageCheck,
    /// Its lowering, or the lowering's diagnostics when it failed.
    pub lowered: Result<&'a LoweredPackage, &'a [Diagnostic]>,
    /// Every loaded package (it and its dependencies), as given to the
    /// lowerer.
    pub packages: &'a [&'a hir::Package],
    /// The build dates the package was lowered at.
    pub options: &'a Options,
}

/// The obligations checked on the lowered pack alone.
const ON_PACK: [O; 4] = [
    O::GraphExits,
    O::GraphReachable,
    O::WorkflowSound,
    O::CiteResolves,
];

/// Discharges every obligation of a checked rule package against the L0
/// `corpus` under `profile`.
///
/// The graph and citation obligations run on the lowered pack. When
/// lowering failed, its diagnostics fail their obligations, those checks
/// and `tests-pass` (which runs the lowered clock) are skipped, and the
/// HIR obligations still run, `deadline-satisfiable` over every node.
#[must_use]
pub fn verify_package(
    input: &PackageInput<'_>,
    corpus: &SourceCorpus,
    profile: Profile,
) -> Verification {
    let cx = Context::new(input.package, input.packages, profile);
    // Tests select versions as the build does: at `--known-at`, else the
    // package's corpus date.
    let known_at = input
        .options
        .known_at
        .as_deref()
        .unwrap_or(&input.package.manifest.in_force.known_at);
    let package = input.package.manifest.name.as_str();
    let mut b = Builder::default();
    sema::aggregate(&mut b, &input.check.diagnostics);
    evidence::hir(&mut b, &cx);
    let mut holes = input.check.holes.clone();
    let mut insensitive = std::collections::BTreeSet::new();
    let mut out = Vec::new();
    match input.lowered.map(|l| (&l.document, &l.source_map)) {
        Ok((Document::Pack(pack), map)) => {
            insensitive = verify_pack(
                &cx,
                &PackVerification {
                    input,
                    corpus,
                    known_at,
                },
                pack,
                map,
                &mut b,
                &mut out,
                &mut holes,
            );
        }
        Ok((Document::Links(_), _)) => {
            insensitive = verify_links(&cx, package, known_at, &mut b, &mut out);
        }
        Err(lowering) => {
            verify_lowering(&cx, package, lowering, &mut b, &mut out);
        }
    }
    record(
        &mut b,
        &mut out,
        O::DistExact,
        timed(package, "dist-exact", || graph::empty_distributions(&cx)),
    );
    let mut typed = timed(package, "cycle-bounds", || cycle::unbounded(&cx));
    typed.extend(timed(package, "refinement-bounds", || {
        refinement::bounds(&cx)
    }));
    record(&mut b, &mut out, O::WellTyped, typed);
    evidence::holes(&mut b, holes.len());
    record(
        &mut b,
        &mut out,
        O::Holes,
        timed(package, "holes", || lints::holes(&holes, profile)),
    );
    record(
        &mut b,
        &mut out,
        O::RuleWithoutTest,
        timed(package, "rule-without-test", || {
            lints::untested(&cx, &insensitive)
        }),
    );
    b.skip(
        O::CaseReplay,
        "a rule package has no events; replay is checked per case",
    );
    Verification::new(b, out, profile)
}

fn record(b: &mut Builder, out: &mut Vec<Diagnostic>, name: O, found: Vec<Diagnostic>) {
    out.extend(found.iter().cloned());
    b.check(name, found);
}

struct PackVerification<'a, 'p> {
    input: &'a PackageInput<'p>,
    corpus: &'a SourceCorpus,
    known_at: &'a str,
}

fn verify_pack(
    cx: &Context<'_>,
    verification: &PackVerification<'_, '_>,
    pack: &Pack,
    map: &SourceMap,
    b: &mut Builder,
    out: &mut Vec<Diagnostic>,
    holes: &mut Vec<litigate_sema::types::build::Hole>,
) -> std::collections::BTreeSet<String> {
    let package = verification.input.package.manifest.name.as_str();
    let reached = timed(package, "graph-reached", || graph::reached(pack));
    evidence::graph(b, cx, reached.len());
    evidence::tests(b, cx);
    record(
        b,
        out,
        O::GraphExits,
        timed(package, "graph-exits", || graph::exits(cx, pack)),
    );
    record(
        b,
        out,
        O::GraphReachable,
        timed(package, "graph-reachable", || {
            graph::unreachable(cx, pack, &reached)
        }),
    );
    record(
        b,
        out,
        O::DeadlineSatisfiable,
        timed(package, "deadline-satisfiable", || {
            deadline::unsatisfiable(cx, &reached)
        }),
    );
    let unsound = timed(package, "workflow-sound", || net::check(b, cx, pack, map));
    record(b, out, O::WorkflowSound, unsound);
    record(
        b,
        out,
        O::CiteResolves,
        timed(package, "cite-resolves", || {
            cite::unresolved(
                cx,
                pack,
                verification.corpus,
                &verification.input.check.holes,
            )
        }),
    );
    holes.extend(cite::unproven(
        cx,
        pack,
        verification.corpus,
        &verification.input.check.holes,
    ));
    let (failures, insensitive) = timed(package, "tests-pass+counting-insensitive", || {
        assertions::check(cx, verification.known_at)
    });
    record(b, out, O::TestsPass, failures);
    insensitive
}

fn verify_links(
    cx: &Context<'_>,
    package: &str,
    known_at: &str,
    b: &mut Builder,
    out: &mut Vec<Diagnostic>,
) -> std::collections::BTreeSet<String> {
    for name in ON_PACK.into_iter().chain([O::DeadlineSatisfiable]) {
        b.skip(
            name,
            "a links package has no graph or sources of its own; checked where it composes packs",
        );
    }
    evidence::tests(b, cx);
    let (failures, insensitive) = timed(package, "tests-pass+counting-insensitive", || {
        assertions::check(cx, known_at)
    });
    record(b, out, O::TestsPass, failures);
    insensitive
}

fn verify_lowering(
    cx: &Context<'_>,
    package: &str,
    lowering: &[Diagnostic],
    b: &mut Builder,
    out: &mut Vec<Diagnostic>,
) {
    sema::aggregate(b, lowering);
    let every: std::collections::BTreeSet<String> = cx.nodes().map(|n| n.meta.id.clone()).collect();
    evidence::deadline(b, every.len());
    record(
        b,
        out,
        O::DeadlineSatisfiable,
        timed(package, "deadline-satisfiable", || {
            deadline::unsatisfiable(cx, &every)
        }),
    );
    for name in ON_PACK.into_iter().chain([O::TestsPass]) {
        b.skip(name, "not checked: the package did not lower");
    }
}

/// The report of a package sema rejected (it has no HIR): its diagnostics
/// fail their obligations, holes are still inventoried, and every
/// obligation that needs HIR is skipped.
#[must_use]
pub fn verify_unchecked(check: &PackageCheck, profile: Profile) -> Verification {
    let mut b = Builder::default();
    sema::aggregate(&mut b, &check.diagnostics);
    let holes = lints::holes(&check.holes, profile);
    evidence::holes(&mut b, check.holes.len());
    b.check(O::Holes, holes.clone());
    for name in O::ALL {
        b.skip(name, "not checked: the package has errors");
    }
    Verification::new(b, holes, profile)
}

/// A package's sema result lowered and verified: the verification, and the
/// lowering when there is one.
#[derive(Debug, Clone, PartialEq)]
pub struct Checked {
    /// Every obligation's result.
    pub verification: Verification,
    /// The lowering, absent when sema or lowering failed.
    pub lowered: Option<LoweredPackage>,
}

impl Checked {
    /// The lowered pack with its report and source map embedded, when the
    /// package is a rule package that lowered.
    #[must_use]
    pub fn litigatable(&self) -> Option<Litigatable<Pack>> {
        let lowered = self.lowered.as_ref()?;
        match &lowered.document {
            Document::Pack(pack) => Some(
                self.verification
                    .embed((**pack).clone(), Some(lowered.source_map.clone())),
            ),
            Document::Links(_) => None,
        }
    }
}

/// Lowers `check`'s package (resolving names across `packages`, every
/// loaded package) at `options`, then verifies it.
#[must_use]
pub fn check_package(
    check: &PackageCheck,
    packages: &[&hir::Package],
    corpus: &SourceCorpus,
    options: &Options,
    profile: Profile,
) -> Checked {
    let Some(package) = check.package.as_ref() else {
        return Checked {
            verification: verify_unchecked(check, profile),
            lowered: None,
        };
    };
    let lowered = Lowerer::new(packages)
        .with_corpus(corpus)
        .package(package, options);
    let input = PackageInput {
        package,
        check,
        lowered: lowered.as_ref().map_err(Vec::as_slice),
        packages,
        options,
    };
    Checked {
        verification: verify_package(&input, corpus, profile),
        lowered: lowered.ok(),
    }
}
