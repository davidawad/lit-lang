//! `describe` is schema-checked: its result deserializes into the typed
//! `Describe` document (every struct denies unknown fields), its JSON
//! Schema is a snapshot, and every command it lists is invocable.
// Test helpers below may unwrap: a panic is the failed assertion.
#![allow(clippy::unwrap_used)]
mod support;

use litigate_cli::describe::Describe;
use support::litigate;

#[test]
fn describe_conforms_to_its_schema() {
    let run = litigate(&["describe"]);
    assert_eq!(run.code, 0);
    let doc: Describe = serde_json::from_value(run.json["result"].clone()).unwrap();
    assert_eq!(doc.language_version, 1);
    assert_eq!(
        doc.diagnostic_codes.len(),
        litigate_base::DiagnosticCode::ALL.len()
    );
    assert!(doc.grammar.iter().any(|p| p.name == "CaseFile"));
    assert!(doc.constructs.iter().filter(|c| c.file == "case").count() >= 14);
    assert!(doc
        .stdlib
        .iter()
        .any(|p| p.name == "frcp-rule12-example" && p.errors == 0));
    let reserialized = serde_json::to_value(&doc).unwrap();
    assert_eq!(
        reserialized, run.json["result"],
        "describe has fields the schema does not"
    );
    insta::assert_json_snapshot!("describe_schema", litigate_cli::describe::schema());
}

#[test]
fn every_listed_command_exists() {
    let run = litigate(&["describe"]);
    let doc: Describe = serde_json::from_value(run.json["result"].clone()).unwrap();
    let names: Vec<&str> = doc.commands.iter().map(|c| c.name.as_str()).collect();
    for wanted in [
        "check",
        "build",
        "test",
        "fmt",
        "run",
        "where",
        "explain",
        "explain-value",
        "diff",
        "query",
        "import",
        "describe",
        "fix",
        "lock",
        "calendar",
        "lint-filing",
    ] {
        assert!(names.contains(&wanted), "{wanted} missing");
    }
    for name in names {
        let help = assert_cmd::Command::cargo_bin("litigate")
            .unwrap()
            .args([name, "--help"])
            .output()
            .unwrap();
        assert!(help.status.success(), "{name} --help");
    }
}
