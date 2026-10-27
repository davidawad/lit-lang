//! One source for the national rules (the PM ruling of 2026-10-27 on
//! lg-f3q8; lg-red5): a promoted `lit/<forum>` package uses `lit/frcp`
//! and `lit/frap` for the Federal Rules of Civil and Appellate Procedure
//! and adds only its own local rules over them. A package fails here when
//! it declares a deadline citing Fed. R. Civ. P. or Fed. R. App. P. that
//! the national package already declares: the same name, a cite to the
//! same rule pinpoint (one a subdivision of the other), or the same period
//! from the same trigger.
// Test helpers below may unwrap: a panic is the failed assertion.
#![allow(clippy::unwrap_used)]
mod support;

use litigate_syntax::ast::{AstNode, CallExpr, DeadlineDecl};
use litigate_syntax::{parse_lit, SyntaxKind};
use std::path::Path;

/// The national rule packages, by directory under `lit/`, and the cite
/// prefixes naming each one's rules.
const NATIONAL: [(&str, &[&str]); 2] = [
    (
        "frcp",
        &["Fed. R. Civ. P.", "Fed R Civ P", "F.R.C.P.", "FRCP"],
    ),
    (
        "frap",
        &["Fed. R. App. P.", "Fed R App P", "F.R.A.P.", "FRAP"],
    ),
];

/// Packages that are not promotions, with why. The ports of the legacy
/// JSON packs (`litigate import`) predate the ruling and restate national
/// periods on their own pack's edges; moving them onto `lit/frap` is
/// their own work, not a promotion audit's. `core` tests its counting
/// rules on FRAP periods; `links` composes packs and is no forum.
const EXEMPT: [(&str, &str); 4] = [
    ("cafc", "ported from packs/cafc-federal-circuit.json"),
    ("core", "the counting library: its tests count FRAP periods"),
    (
        "frcrimp",
        "ported from packs/frcrimp-criminal-procedure.json",
    ),
    ("links", "the links package, which composes packs"),
];

/// One `deadline` declaration.
#[derive(Debug)]
struct Deadline {
    file: String,
    name: String,
    period: String,
    trigger: String,
    cite: String,
}

/// Every deadline declared in `.lit` text `text` (of `file`).
fn deadlines(file: &str, text: &str) -> Vec<Deadline> {
    let parsed = parse_lit(text);
    parsed
        .syntax()
        .descendants()
        .filter(|n| n.kind() == SyntaxKind::DeadlineDecl)
        .filter_map(DeadlineDecl::cast)
        .map(|d| Deadline {
            file: file.to_owned(),
            name: d.name().map(|t| t.text().to_owned()).unwrap_or_default(),
            period: d.period().map(|t| t.text().to_owned()).unwrap_or_default(),
            trigger: d
                .trigger()
                .and_then(|e| CallExpr::cast(e.syntax().clone()))
                .and_then(|c| c.callee())
                .map(|r| r.text())
                .unwrap_or_default(),
            cite: d
                .cites()
                .iter()
                .filter_map(litigate_syntax::ast::Cite::text)
                .collect::<Vec<_>>()
                .join("; "),
        })
        .collect()
}

/// Every deadline of the package in `dir`.
fn package(dir: &Path) -> Vec<Deadline> {
    let mut files: Vec<_> = std::fs::read_dir(dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|e| e == "lit"))
        .collect();
    files.sort();
    files
        .iter()
        .flat_map(|p| {
            let shown = p.strip_prefix(support::repo()).unwrap_or(p);
            deadlines(
                &shown.to_string_lossy(),
                &std::fs::read_to_string(p).unwrap(),
            )
        })
        .collect()
}

/// The pinpoints (`31(a)(1)`) of `cite`'s parts citing one of `prefixes`,
/// an edition note (`FRAP (2008 ed.) 4(a)(1)`) dropped.
fn pinpoints(cite: &str, prefixes: &[&str]) -> Vec<String> {
    cite.split(';')
        .filter_map(|part| {
            let part = part.trim();
            let rest = prefixes.iter().find_map(|p| part.strip_prefix(p))?;
            let rest = rest.trim_start();
            let rest = rest.strip_prefix("(2008 ed.)").unwrap_or(rest).trim_start();
            let rest = rest.strip_prefix("Rule").unwrap_or(rest).trim_start();
            let pin: String = rest
                .chars()
                .take_while(|c| !c.is_whitespace() && *c != ',')
                .collect();
            pin.starts_with(|c: char| c.is_ascii_digit()).then_some(pin)
        })
        .collect()
}

/// One pinpoint is the other or a subdivision of it (`41(b)`, `41(b)(1)`;
/// never `4(a)` and `40(d)`).
fn related(a: &str, b: &str) -> bool {
    let within = |long: &str, short: &str| {
        long.strip_prefix(short)
            .is_some_and(|rest| rest.is_empty() || rest.starts_with('('))
    };
    within(a, b) || within(b, a)
}

/// Each deadline of `local` restating one of `national` (whose rules
/// `prefixes` cite), with the national declaration it restates.
fn restated<'a>(
    local: &'a [Deadline],
    national: &'a [Deadline],
    prefixes: &[&str],
) -> Vec<(&'a Deadline, &'a Deadline)> {
    local
        .iter()
        .filter_map(|d| {
            let pins = pinpoints(&d.cite, prefixes);
            if pins.is_empty() {
                return None;
            }
            let same = national.iter().find(|n| {
                n.name == d.name
                    || (n.period == d.period && n.trigger == d.trigger)
                    || pinpoints(&n.cite, prefixes)
                        .iter()
                        .any(|p| pins.iter().any(|q| related(p, q)))
            })?;
            Some((d, same))
        })
        .collect()
}

#[test]
fn no_promoted_package_restates_a_national_rule_deadline() {
    let lit = support::repo().join("lit");
    let national: Vec<_> = NATIONAL
        .iter()
        .map(|(dir, prefixes)| (*dir, *prefixes, package(&lit.join(dir))))
        .collect();
    for (dir, _, decls) in &national {
        assert!(!decls.is_empty(), "lit/{dir} declares no deadline");
    }
    let mut dirs: Vec<_> = std::fs::read_dir(&lit)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.join("Lit.toml").is_file())
        .collect();
    dirs.sort();
    let mut found = Vec::new();
    let mut checked = 0;
    for dir in dirs {
        let name = dir.file_name().unwrap().to_string_lossy().into_owned();
        if NATIONAL.iter().any(|(n, _)| *n == name) || EXEMPT.iter().any(|(e, _)| *e == name) {
            continue;
        }
        let local = package(&dir);
        checked += local.len();
        for (national_dir, prefixes, decls) in &national {
            for (d, n) in restated(&local, decls, prefixes) {
                found.push(format!(
                    "{}: deadline `{}` (cite \"{}\") restates lit/{national_dir}'s `{}` \
                     ({}): import it and declare only the local rule over it",
                    d.file, d.name, d.cite, n.name, n.file
                ));
            }
        }
    }
    assert!(checked > 0, "no promoted package declares a deadline");
    assert!(found.is_empty(), "{}", found.join("\n"));
}

#[test]
fn a_restated_frap_period_is_found() {
    // lit/cadc as promoted (e11d09c9): FRAP 31(a)(1)'s 40 days, declared
    // again under the circuit package's own name.
    let frap = support::repo().join("lit/frap");
    let national = package(&frap);
    let local = deadlines(
        "lit/cadc/court-of-appeals.lit",
        "module m\nimport frap.events.{ RecordFiled }\n\
         deadline opening-brief: 40.days after RecordFiled() counted_by frap26 \
         cite \"Fed. R. App. P. 31(a)(1)\"\n",
    );
    let prefixes = NATIONAL[1].1;
    let hits = restated(&local, &national, prefixes);
    assert_eq!(hits.len(), 1, "{hits:?}");
    assert_eq!(hits[0].1.name, "appellant-brief-period");

    // A local rule's own period, and a FRAP period lit/frap leaves out,
    // are not restatements.
    let own = deadlines(
        "lit/x/x.lit",
        "module m\n\
         deadline cure: 14.days after NoticeIssued() counted_by frap26 cite \"1st Cir. R. 3.0(b)\"\n\
         deadline other: 9.days after Docketed() counted_by frap26 cite \"FRAP 12(b)\"\n",
    );
    assert!(restated(&own, &national, prefixes).is_empty());
}

#[test]
fn pinpoints_compare_by_subdivision() {
    let frap = NATIONAL[1].1;
    assert_eq!(
        pinpoints("FRAP (2008 ed.) 4(a)(1)(A); FRAP 4(a)(5)(A)", frap),
        ["4(a)(1)(A)", "4(a)(5)(A)"]
    );
    assert_eq!(pinpoints("FRAP 10, 11, 12; 1st Cir. R. 31.0", frap), ["10"]);
    assert!(pinpoints("28 U.S.C. § 2107(a)", frap).is_empty());
    assert!(related("41(b)", "41(b)(1)"));
    assert!(!related("4(a)", "40(d)"));
    assert!(!related("31(a)(1)", "31(a)(2)"));
}

#[test]
fn the_circuit_packages_test_the_frap_periods_they_import() {
    for dir in ["lit/ca1", "lit/cadc"] {
        let run = support::litigate(&["test", dir]);
        assert_eq!(run.code, 0, "{dir}: {}", run.stdout);
        let result = &run.json["result"];
        assert_eq!(result["failed"], 0, "{dir}: {}", run.stdout);
        assert!(result["passed"].as_u64().is_some_and(|n| n > 0), "{dir}");
    }
}
