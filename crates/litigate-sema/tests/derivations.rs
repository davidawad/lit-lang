//! Every L3xxx error carries the derivation of its defect (SPEC §9.2):
//! the precedence checks (L3003-L3006) attach the tree of the resolution
//! rules that conclude it, and every other L3xxx error the one-step
//! derivation of its code's rule over its labels.
mod legal;
use legal::{pkg, workspace, CORE, PRELUDE};
use litigate_base::{Derivation, Diagnostic};
use litigate_sema::pkg::load::LoadedPackage;
use std::collections::BTreeMap;
use std::fmt::Write as _;

fn d(id: &str, days: u32, clauses: &str) -> String {
    format!("deadline {id}: {days}.days from Served(complaint, Personal) counted_by c6 {clauses}\n")
}

fn tree(derivation: &Derivation, depth: usize, out: &mut String) {
    let at = derivation
        .span
        .as_ref()
        .map(|s| format!(" @{}:{}", s.line, s.col))
        .unwrap_or_default();
    let _ = write!(
        out,
        "\n{}{}: {}{at}",
        "  ".repeat(depth),
        derivation.rule,
        derivation.fact
    );
    for premise in &derivation.premises {
        tree(premise, depth + 1, out);
    }
}

/// `CODE message` and the derivation tree of every L3xxx diagnostic.
fn derivations(packages: &[LoadedPackage]) -> Vec<String> {
    let (db, ws) = workspace(packages);
    litigate_sema::check_workspace(&db, ws)
        .iter()
        .flat_map(|c| c.diagnostics.iter())
        .filter(|d| d.code.as_str().starts_with("L3"))
        .map(|d: &Diagnostic| {
            let mut s = format!("{} {}", d.code.as_str(), d.message);
            match &d.derivation {
                Some(derivation) => tree(derivation, 1, &mut s),
                None => s.push_str("\n  (no derivation)"),
            }
            s
        })
        .collect()
}

fn single(body: &str) -> Vec<LoadedPackage> {
    vec![pkg(
        "demo",
        "FederalRule",
        "2009-12-01",
        &[],
        &[("main.lit", &format!("{PRELUDE}{body}\n"))],
    )]
}

fn core() -> LoadedPackage {
    pkg(
        "core",
        "Statute",
        "1990-01-01",
        &[],
        &[
            ("authority.lit", CORE),
            ("start.lit", "module core.start\nstart z\nstate z: \"Z\"\n"),
        ],
    )
}

/// Package `name` of `authority` importing `imports` from `deps`'
/// `<dep>.rules` (as the precedence tables build them), then `body`.
fn rules(name: &str, authority: &str, deps: &[&str], imports: &str, body: &str) -> LoadedPackage {
    let mut text = format!("module {name}.rules\n");
    if deps.contains(&"core") {
        text.push_str(
            "import core.authority.{Statute, FederalRule, LocalRule, StandingOrder, CaseOrder}\n",
        );
    }
    for dep in deps.iter().filter(|d| **d != "core") {
        let _ = writeln!(text, "import {dep}.rules.{{{imports}}}");
    }
    let _ = write!(
        text,
        "start s\nenum Document {{ complaint }}\nevent Served(document: Document)\nstate s: \"S\"\n{C6}{body}\n"
    );
    pkg(name, authority, "2009-12-01", deps, &[("rules.lit", &text)])
}

const C6: &str = "counting c6 { holidays cal }\ncalendar cal { new-year: fixed(1, 1) effective [1870-06-28, ..) }\n";

fn served(id: &str, days: u32, clauses: &str) -> String {
    format!("deadline {id}: {days}.days from Served(complaint) counted_by c6 {clauses}\n")
}

#[test]
fn every_l3_error_carries_its_derivation() {
    let tg = d("tg", 1, "");
    let events = (0..18).fold(String::new(), |mut s, i| {
        let _ = writeln!(s, "event E{i}()");
        s
    });
    let any = |range: std::ops::Range<u32>| {
        range
            .map(|i| format!("has_event(E{i}())"))
            .collect::<Vec<_>>()
            .join(" || ")
    };
    let cases: Vec<(&str, Vec<LoadedPackage>)> = vec![
        (
            "L3003 claimants tie on every canon",
            single(&(tg.clone() + &d("a", 2, "notwithstanding tg") + &d("b", 3, "notwithstanding tg"))),
        ),
        (
            "L3004 a lower authority displaces without a delegation",
            vec![
                core(),
                rules("x", "FederalRule", &["core"], "", &served("tg", 1, "")),
                rules(
                    "y",
                    "LocalRule",
                    &["core", "x"],
                    "tg",
                    &served("o", 4, "notwithstanding tg"),
                ),
            ],
        ),
        (
            "L3004 an unranked override",
            vec![
                rules("x", "Alpha", &[], "", &served("tg", 1, "")),
                rules("y", "Beta", &["x"], "tg", &served("o", 4, "override tg")),
            ],
        ),
        (
            "L3004 a delegation to a level that is not lower",
            single(&("authority Alpha > Beta\n".to_owned() + &d("o", 1, "delegable { to Alpha, op set }"))),
        ),
        (
            "L3005 exceptions not proved disjoint",
            single(&format!(
                "{events}default dd when true => 1\nexception e1 rebuts dd when {} => 2\nexception e2 rebuts dd when {} => 3",
                any(0..9),
                any(9..18)
            )),
        ),
        (
            "L3006 claimants beating each other in a circle",
            single(&(tg.clone() + &d("a", 2, "notwithstanding tg") + &d("b", 3, "subject_to tg notwithstanding a"))),
        ),
        (
            "L3002 an undeclared gap takes its rule's one-step derivation",
            single(&(d("p", 1, "effective [2009-12-01, 2012-01-01)") + &d("p", 2, "effective [2013-01-01, ..)"))),
        ),
    ];
    let got: BTreeMap<&str, Vec<String>> = cases
        .iter()
        .map(|(name, packages)| (*name, derivations(packages)))
        .collect();
    for (name, found) in &got {
        assert!(!found.is_empty(), "{name}: no L3xxx error");
        assert!(
            found.iter().all(|s| !s.contains("(no derivation)")),
            "{name}: {found:#?}"
        );
    }
    insta::assert_debug_snapshot!(got);
}
