//! The SPEC §12 worked example (`lit/examples`), and the report embedded
//! in its litigatable.
use litigate_base::Profile;
use litigate_engine::cite::SourceCorpus;
use litigate_lower::Options;
use litigate_verify::{check_package, Checked, Obligation, ObligationStatus};
use std::path::Path;

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
fn dev_reports_the_three_holes_and_accepts() {
    let dev = example(Profile::Dev);
    assert!(dev.verification.is_accepted());
    let codes: Vec<&str> = dev
        .verification
        .diagnostics
        .iter()
        .map(|d| d.code.as_str())
        .collect();
    assert_eq!(codes, ["L9002", "L9002", "L9002"]);
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
fn strict_rejects_each_hole() {
    let strict = example(Profile::Strict);
    assert!(!strict.verification.is_accepted());
    insta::assert_json_snapshot!(strict.verification.diagnostics);
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
