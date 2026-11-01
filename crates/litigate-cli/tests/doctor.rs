//! Integration coverage for the diagnostic command.
use litigate_cli::execute;
use serde_json::Value;

#[test]
fn doctor_reports_environment_and_workspace_in_standard_envelope() {
    let printed = execute(["litigate", "doctor"]);
    assert_eq!(printed.code, 0, "{}", printed.stderr);
    let response: Value = serde_json::from_str(&printed.stdout).unwrap();
    assert_eq!(response["ok"], true);
    assert_eq!(response["op"], "doctor");
    let result = &response["result"];
    assert!(result["version"].is_string());
    assert!(result["platform"].is_string());
    assert!(result["packs"].as_array().is_some_and(|items| !items.is_empty()));
    assert!(result["cases"].as_array().is_some_and(|items| !items.is_empty()));
    assert!(result["binaries"]["litigate-lsp"].is_boolean());
    assert!(result["binaries"]["litigate"].is_boolean());
}

#[test]
fn doctor_is_in_help() {
    let printed = execute(["litigate", "--help"]);
    assert_eq!(printed.code, 0);
    assert!(printed.stdout.contains("doctor"));
}
