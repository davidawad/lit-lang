//! Verification obligations for compiled litigate packages and matters
//! (SPEC §7, DESIGN §7).
//!
//! [`verify_package`] discharges every obligation of a checked, lowered
//! rule package: sema's diagnostics are attributed to the obligations they
//! violate (`well-typed`, the temporal and precedence obligations,
//! `dist-exact`), and this crate checks the rest on the lowered pack and
//! HIR (`graph-exits`, `graph-reachable`, `deadline-satisfiable`,
//! `cite-resolves`, `tests-pass`, `holes`, `rule-without-test`). The
//! result is a [`Verification`]: the [`VerificationReport`] embedded in the
//! [`Litigatable`], plus the diagnostics this crate emits (sema's are
//! aggregated, not repeated). [`check_package`] runs lowering and
//! verification together; [`verify_case`] reports a matter's `case-replay`.
//!
//! Profiles change severity and acceptance, never values: holes are L9002
//! warnings in `dev` and L5002 errors in `strict`; L9001 is a warning in
//! `dev` and an error in `strict`; every other failure is an error.
mod assertions;
mod cite;
mod context;
mod deadline;
mod diag;
mod graph;
mod lints;
mod report;
mod sema;

pub use report::{Finding, Obligation, ObligationStatus, VerificationReport, MAX_EXAMPLES};
pub use sema::obligation_of;

pub(crate) use context::{same_expr, Context};

use litgraph::cite::SourceCorpus;
use litgraph::model::Pack;
use litigate_base::{Diagnostic, DiagnosticCode as C, Profile, Severity};
use litigate_lower::{Document, LoweredPackage, Lowerer, Options, SourceMap};
use litigate_sema::{hir, PackageCheck};
use report::{Builder, Obligation as O};

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
    let cx = Context::new(input.package, input.packages, input.options, profile);
    let mut b = Builder::default();
    sema::aggregate(&mut b, &input.check.diagnostics);
    let mut out = Vec::new();
    let mut record = |b: &mut Builder, name: O, found: Vec<Diagnostic>| {
        out.extend(found.iter().cloned());
        b.check(name, found);
    };
    match input.lowered.map(|l| &l.document) {
        Ok(Document::Pack(pack)) => {
            let reached = graph::reached(pack);
            record(&mut b, O::GraphExits, graph::exits(&cx, pack));
            record(
                &mut b,
                O::GraphReachable,
                graph::unreachable(&cx, pack, &reached),
            );
            record(
                &mut b,
                O::DeadlineSatisfiable,
                deadline::unsatisfiable(&cx, &reached),
            );
            record(&mut b, O::CiteResolves, cite::unresolved(&cx, pack, corpus));
            record(&mut b, O::TestsPass, assertions::failures(&cx));
        }
        Ok(Document::Links(_)) => {
            for name in [
                O::GraphExits,
                O::GraphReachable,
                O::DeadlineSatisfiable,
                O::CiteResolves,
            ] {
                b.skip(name, "a links package has no graph or sources of its own; checked where it composes packs");
            }
            record(&mut b, O::TestsPass, assertions::failures(&cx));
        }
        Err(lowering) => {
            sema::aggregate(&mut b, lowering);
            let every: std::collections::BTreeSet<String> =
                cx.nodes().map(|n| n.meta.id.clone()).collect();
            record(
                &mut b,
                O::DeadlineSatisfiable,
                deadline::unsatisfiable(&cx, &every),
            );
            for name in [
                O::GraphExits,
                O::GraphReachable,
                O::CiteResolves,
                O::TestsPass,
            ] {
                b.skip(name, "not checked: the package did not lower");
            }
        }
    }
    record(&mut b, O::DistExact, graph::empty_distributions(&cx));
    record(&mut b, O::Holes, lints::holes(&input.check.holes, profile));
    record(&mut b, O::RuleWithoutTest, lints::untested(&cx));
    b.skip(
        O::CaseReplay,
        "a rule package has no events; replay is checked per case",
    );
    Verification::new(b, out, profile)
}

/// The report of a package sema rejected (it has no HIR): its diagnostics
/// fail their obligations, holes are still inventoried, and every
/// obligation that needs HIR is skipped.
#[must_use]
pub fn verify_unchecked(check: &PackageCheck, profile: Profile) -> Verification {
    let mut b = Builder::default();
    sema::aggregate(&mut b, &check.diagnostics);
    let holes = lints::holes(&check.holes, profile);
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

/// The report of a matter at `file`: `case-replay` from the case checker's
/// `diagnostics` (L4xxx; any other error counts against its own
/// obligation), plus `unvalued` stakes, a warning in `dev` and an L4004
/// error in `strict`.
#[must_use]
pub fn verify_case(
    case: &hir::Case,
    file: &str,
    diagnostics: &[Diagnostic],
    profile: Profile,
) -> Verification {
    let mut b = Builder::default();
    sema::aggregate(&mut b, diagnostics);
    let unvalued: Vec<Diagnostic> = case
        .stakes
        .iter()
        .filter(|(_, s)| matches!(s, hir::Stake::Unvalued))
        .map(|(node, _)| {
            let d = diag::emit(
                C::MissingStake,
                profile,
                diag::start_of(file),
                format!("stake `{node}` is `unvalued`"),
            );
            match profile {
                Profile::Dev => Diagnostic {
                    severity: Severity::Warning,
                    ..d
                },
                Profile::Strict => diag::note(
                    d,
                    None,
                    "strict requires a value for every reachable terminal",
                ),
            }
        })
        .collect();
    b.check(O::CaseReplay, unvalued.clone());
    for name in O::ALL {
        b.skip(
            name,
            "a matter is checked for replay only; its packages carry the rest",
        );
    }
    Verification::new(b, unvalued, profile)
}
