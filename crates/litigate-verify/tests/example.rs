//! The SPEC §12 worked example (`lit/examples`), and the report embedded
//! in its litigatable.
use litigate_base::Profile;
use litigate_engine::cite::SourceCorpus;
use litigate_lower::Options;
use litigate_verify::{check_package, Checked, Obligation, ObligationStatus};
use std::path::Path;

fn example_with_holes(profile: Profile) -> Checked {
    use litigate_sema::pkg::load::{LoadedFile, LoadedPackage};
    let source = include_str!("../../../lit/examples/rule12.lit")
        .replace(
            "granted \"Motion granted\" -> dismissed p 1/2",
            "granted \"Motion granted\" -> dismissed p ?(\"no calibration yet\")",
        )
        .replace(
            "denied \"Motion denied\" -> answered p 1/2",
            "denied \"Motion denied\" -> answered p ?",
        )
        .replacen("payoff 0.usd", "payoff ?", 1);
    let package = LoadedPackage {
        texts: Default::default(),
        root: "lit/examples".to_owned(),
        manifest: include_str!("../../../lit/examples/Lit.toml").to_owned(),
        lock: None,
        files: vec![LoadedFile {
            relative: "rule12.lit".to_owned(),
            text: source,
        }],
    };
    let mut db = litigate_sema::LitDatabase::default();
    let ws = litigate_sema::db::inputs::create_workspace(&mut db, &[package]);
    let checks = litigate_sema::check_workspace(&db, ws);
    let packages: Vec<_> = checks.iter().filter_map(|c| c.package.as_ref()).collect();
    let Some(check) = checks.first() else {
        panic!("no package");
    };
    check_package(
        check,
        &packages,
        SourceCorpus::embedded_ref(),
        &Options::default(),
        profile,
    )
}

fn example(profile: Profile) -> Checked {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let (db, ws) = litigate_sema::load(&root, &[root.join("lit/examples")], None)
        .unwrap_or_else(|e| panic!("{e}"));
    let checks = litigate_sema::check_workspace(&db, ws);
    let packages: Vec<_> = checks.iter().filter_map(|c| c.package.as_ref()).collect();
    let Some(check) = checks.first() else {
        panic!("no package");
    };
    check_package(
        check,
        &packages,
        SourceCorpus::embedded_ref(),
        &Options::default(),
        profile,
    )
}

#[test]
fn dev_reports_no_holes_and_accepts() {
    let dev = example(Profile::Dev);
    assert!(dev.verification.is_accepted());
    let codes: Vec<&str> = dev
        .verification
        .diagnostics
        .iter()
        .map(|d| d.code.as_str())
        .collect();
    assert!(codes.is_empty());
    let tests = dev.verification.report.finding(Obligation::TestsPass);
    assert_eq!(
        tests.map(|f| (f.status, f.count)),
        Some((ObligationStatus::Passed, 0))
    );
    // V3: every pass states what it was checked over.
    for f in &dev.verification.report.obligations {
        if f.status == ObligationStatus::Passed {
            assert!(f.evidence.is_some(), "{f:?}");
        }
    }
    insta::assert_json_snapshot!(dev.verification.report);
}

#[test]
fn dev_reports_the_three_holes_and_accepts() {
    let dev = example_with_holes(Profile::Dev);
    assert!(dev.verification.is_accepted());
    let codes: Vec<&str> = dev
        .verification
        .diagnostics
        .iter()
        .map(|d| d.code.as_str())
        .collect();
    assert_eq!(codes, ["L9002", "L9002", "L9002"]);
    let reasons: Vec<&str> = dev
        .verification
        .diagnostics
        .iter()
        .map(|d| d.message.strip_prefix("hole: ").unwrap_or(&d.message))
        .collect();
    assert_eq!(
        reasons,
        ["no calibration yet", "no reason given", "no reason given"]
    );
}

#[test]
fn strict_accepts_the_resolved_example() {
    let strict = example(Profile::Strict);
    assert!(strict.verification.is_accepted());
    insta::assert_json_snapshot!(strict.verification.diagnostics);
}

#[test]
fn strict_rejects_each_hole() {
    let strict = example_with_holes(Profile::Strict);
    assert!(!strict.verification.is_accepted());
    let codes: Vec<&str> = strict
        .verification
        .diagnostics
        .iter()
        .map(|d| d.code.as_str())
        .collect();
    assert_eq!(codes, ["L5002", "L5002", "L5002"]);
    assert_eq!(strict.verification.diagnostics.len(), 3);
}

#[test]
fn the_report_is_embedded_in_the_litigatable() {
    let dev = example(Profile::Dev);
    let Some(litigatable) = dev.litigatable() else {
        panic!("the example lowers to a pack");
    };
    assert_eq!(litigatable.verification, dev.verification.report);
    assert_eq!(litigatable.ir.id, "frcp-rule12-example");
    let json = serde_json::to_value(&litigatable).unwrap_or_default();
    let back: litigate_verify::Litigatable<litigate_engine::model::Pack> =
        serde_json::from_value(json).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(back, litigatable);
    insta::assert_json_snapshot!(litigatable, {
        ".ir" => "[pack]",
        ".source_map" => "[source map]",
    });
}
